use crate::animation::{AnimationSystem, FrameCommands, FrameContext, RenderLayer, TerminalSize};
use crate::render::TerminalRenderer;
use crate::scene::{HouseBounds, LawnBounds, house_bounds, lawn_bounds};
use crossterm::style::Color;
use rand::{Rng, RngExt};
use std::io;

const FRAME_STEP: u8 = 15;
pub const SPRITE_WIDTH: u16 = 9;
pub const SPRITE_HEIGHT: u16 = 3;
const SLEEP_MIN: u16 = 180;
const SLEEP_MAX: u16 = 420;
const WALK_MIN: u16 = 90;
const WALK_MAX: u16 = 180;
const RETURN_TIMEOUT: u16 = 600;
const WAKE_CHANCE_PERCENT: u8 = 32;
const HUNT_CHANCE_PERCENT: u8 = 20;
const CAT_MOVE_SPEED: f32 = 0.12;

const CAT_WALK_FRAMES: [[&str; 3]; 4] = [
    [" /\\_/\\\\  ", "( o.o )__", "  / \\    "],
    [" /\\_/\\\\  ", "( o.o )_/", "   /\\    "],
    [" /\\_/\\\\  ", "( -.- )__", "  \\ /    "],
    [" /\\_/\\\\  ", "( o.o )\\_", "  /  \\   "],
];
const CAT_SLEEP_FRAMES: [[&str; 3]; 3] = [
    [" /\\_/\\\\  ", "( -.- )__", "  > ^ <  "],
    [" /\\_/\\\\  ", "( -.- )__", "   ^ ^   "],
    [" /\\_/\\\\  ", "( -.- )__", "  > ^ <  "],
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatActivity {
    Sleeping,
    Wandering,
    Hunting,
    Returning,
}

#[derive(Clone, Copy)]
struct AnchorBounds {
    left: u16,
    right: u16,
    top: u16,
    bottom: u16,
}

impl AnchorBounds {
    fn from_lawn(lawn: LawnBounds) -> Option<Self> {
        if lawn.width() < SPRITE_WIDTH || lawn.height() < SPRITE_HEIGHT {
            return None;
        }
        Some(Self {
            left: lawn.left,
            right: lawn.right.saturating_sub(SPRITE_WIDTH - 1),
            top: lawn.top,
            bottom: lawn.bottom.saturating_sub(SPRITE_HEIGHT - 1),
        })
    }

    fn clamp_x(self, x: f32) -> f32 {
        x.clamp(self.left as f32, self.right as f32)
    }

    fn clamp_y(self, y: f32) -> f32 {
        y.clamp(self.top as f32, self.bottom as f32)
    }
}

struct Cat {
    x: f32,
    y: f32,
    sleep_x: f32,
    sleep_y: f32,
    target_x: f32,
    target_y: f32,
    direction: i8,
    speed: f32,
    state: CatActivity,
    state_timer: u16,
    frame_index: usize,
    frame_tick: u8,
    anchor_initialized: bool,
    has_woken: bool,
}

impl Cat {
    fn new(_terminal_width: u16) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            sleep_x: 0.0,
            sleep_y: 0.0,
            target_x: 0.0,
            target_y: 0.0,
            direction: 1,
            speed: CAT_MOVE_SPEED,
            state: CatActivity::Sleeping,
            state_timer: SLEEP_MIN,
            frame_index: 0,
            frame_tick: 0,
            anchor_initialized: false,
            has_woken: false,
        }
    }

    fn set_lawn_spot(&mut self, bounds: AnchorBounds, house: HouseBounds, initial: bool) {
        // Prefer the right side only when the complete sprite fits there.
        // At 80 columns the house leaves eight cells on each side, so the
        // left anchor (x=0) remains the stable, visible choice.
        let right_clearance = bounds.right.saturating_sub(house.right);
        let new_sleep_x = if right_clearance >= SPRITE_WIDTH {
            bounds.clamp_x(house.right.saturating_add(1) as f32)
        } else {
            bounds.clamp_x(house.left.saturating_sub(SPRITE_WIDTH) as f32)
        };
        let new_sleep_y = bounds.clamp_y(house.bottom.saturating_add(1) as f32);
        if initial {
            self.sleep_x = new_sleep_x;
            self.sleep_y = new_sleep_y;
            self.x = self.sleep_x;
            self.y = self.sleep_y;
            self.target_x = self.sleep_x;
            self.target_y = self.sleep_y;
            self.anchor_initialized = true;
        } else {
            self.sleep_x = new_sleep_x;
            self.sleep_y = new_sleep_y;
            if self.state == CatActivity::Sleeping {
                self.x = self.sleep_x;
                self.y = self.sleep_y;
            } else if self.state == CatActivity::Returning {
                self.target_x = self.sleep_x;
                self.target_y = self.sleep_y;
                self.update_direction();
            }
        }
    }

    fn reset_animation(&mut self) {
        self.frame_index = 0;
        self.frame_tick = 0;
    }

    fn choose_sleep_duration(&mut self, rng: &mut (impl Rng + ?Sized)) {
        self.state_timer = rng.random_range(SLEEP_MIN..=SLEEP_MAX);
    }

    fn update_direction(&mut self) {
        let dx = self.target_x - self.x;
        if dx > 0.05 {
            self.direction = 1;
        } else if dx < -0.05 {
            self.direction = -1;
        }
    }

    fn start_wandering(&mut self, bounds: AnchorBounds, rng: &mut (impl Rng + ?Sized)) {
        self.state = CatActivity::Wandering;
        self.state_timer = rng.random_range(WALK_MIN..=WALK_MAX);
        self.target_x = rng.random_range(bounds.left..=bounds.right) as f32;
        self.target_y = rng.random_range(bounds.top..=bounds.bottom) as f32;
        self.update_direction();
        self.reset_animation();
    }

    fn start_hunting(&mut self, rng: &mut (impl Rng + ?Sized)) {
        self.state = CatActivity::Hunting;
        self.direction = if rng.random_bool(0.5) { -1 } else { 1 };
        self.state_timer = RETURN_TIMEOUT;
        self.reset_animation();
    }

    fn start_returning(&mut self) {
        self.state = CatActivity::Returning;
        self.state_timer = RETURN_TIMEOUT;
        self.target_x = self.sleep_x;
        self.target_y = self.sleep_y;
        self.update_direction();
        self.reset_animation();
    }

    fn move_towards_target(&mut self, bounds: AnchorBounds) -> bool {
        let dx = self.target_x - self.x;
        let dy = self.target_y - self.y;
        let distance = (dx * dx + dy * dy).sqrt();
        if distance <= self.speed.max(0.05) {
            self.x = self.target_x;
            self.y = self.target_y;
            return true;
        }

        let step = self.speed / distance;
        self.x = self.x + dx * step;
        self.y = bounds.clamp_y(self.y + dy * step);
        self.update_direction();
        false
    }

    fn finish_return(&mut self, rng: &mut (impl Rng + ?Sized)) {
        self.x = self.sleep_x;
        self.y = self.sleep_y;
        self.state = CatActivity::Sleeping;
        self.direction = 1;
        self.choose_sleep_duration(rng);
        self.reset_animation();
    }

    fn advance(
        &mut self,
        lawn: LawnBounds,
        house: HouseBounds,
        terminal_width: u16,
        is_day: bool,
        weather_safe: bool,
        rng: &mut (impl Rng + ?Sized),
    ) {
        let Some(bounds) = AnchorBounds::from_lawn(lawn) else {
            return;
        };
        let initial_spot = !self.anchor_initialized;
        self.set_lawn_spot(bounds, house, initial_spot);
        let outdoor_allowed = is_day && weather_safe;
        if !outdoor_allowed && !matches!(self.state, CatActivity::Sleeping | CatActivity::Returning)
        {
            self.start_returning();
        }

        match self.state {
            CatActivity::Sleeping => {
                self.x = self.sleep_x;
                self.y = self.sleep_y;
                if outdoor_allowed {
                    if self.state_timer == 0 {
                        // The first wake is guaranteed to make the normal
                        // TUI cycle observable; later sleep cycles remain
                        // probabilistic and predominantly stay asleep.
                        let first_wake = !self.has_woken;
                        if first_wake || rng.random_range(0..100u8) < WAKE_CHANCE_PERCENT {
                            self.has_woken = true;
                            if !first_wake && rng.random_range(0..100u8) < HUNT_CHANCE_PERCENT {
                                self.start_hunting(rng);
                            } else {
                                self.start_wandering(bounds, rng);
                            }
                        } else {
                            self.choose_sleep_duration(rng);
                        }
                    } else {
                        self.state_timer = self.state_timer.saturating_sub(1);
                    }
                }
            }
            CatActivity::Wandering => {
                if !outdoor_allowed {
                    self.start_returning();
                } else if self.state_timer == 0 || self.move_towards_target(bounds) {
                    if rng.random_range(0..100u8) < HUNT_CHANCE_PERCENT {
                        self.start_hunting(rng);
                    } else {
                        self.start_returning();
                    }
                } else {
                    self.state_timer = self.state_timer.saturating_sub(1);
                }
            }
            CatActivity::Hunting => {
                self.x += self.speed * self.direction as f32;
                self.state_timer = self.state_timer.saturating_sub(1);
                let off_left = self.x <= -(SPRITE_WIDTH as f32);
                let off_right = self.x >= terminal_width as f32;
                if off_left || off_right || self.state_timer == 0 || !outdoor_allowed {
                    self.start_returning();
                }
            }
            CatActivity::Returning => {
                if self.move_towards_target(bounds) {
                    self.finish_return(rng);
                } else {
                    self.state_timer = self.state_timer.saturating_sub(1);
                    // Never resolve an overdue return by teleporting. The
                    // timer only prevents a stale state from living forever;
                    // a long return gets another movement window.
                    if self.state_timer == 0 {
                        self.state_timer = RETURN_TIMEOUT;
                    }
                }
            }
        }

        self.frame_tick = self.frame_tick.saturating_add(1);
        if self.frame_tick >= FRAME_STEP {
            self.frame_index = self.frame_index.saturating_add(1);
            self.frame_tick = 0;
        }
    }
}

pub struct CatSystem {
    cat: Cat,
    terminal_width: u16,
    terminal_height: u16,
}

impl CatSystem {
    pub fn new(terminal_width: u16, terminal_height: u16) -> Self {
        Self {
            cat: Cat::new(terminal_width),
            terminal_width,
            terminal_height,
        }
    }
}

impl AnimationSystem for CatSystem {
    fn id(&self) -> &'static str {
        "cat"
    }

    fn layer(&self) -> RenderLayer {
        RenderLayer::Foreground
    }

    fn is_active(&self, _ctx: &FrameContext<'_>) -> bool {
        // The cat also needs updates during rain/night to return home instead
        // of being hidden in an off-screen state forever.
        true
    }

    fn on_resize(&mut self, size: TerminalSize) {
        self.terminal_width = size.width;
        self.terminal_height = size.height;
        let ground_y = size.height.saturating_sub(7);
        if let (Some(house), Some(bounds)) = (
            house_bounds(size.width, size.height, ground_y),
            lawn_bounds(size.width, size.height, ground_y).and_then(AnchorBounds::from_lawn),
        ) {
            self.cat
                .set_lawn_spot(bounds, house, !self.cat.anchor_initialized);
        }
        let max_y = size.height.saturating_sub(1) as f32;
        self.cat.y = self.cat.y.clamp(0.0, max_y);
        self.cat.sleep_x = self
            .cat
            .sleep_x
            .clamp(0.0, size.width.saturating_sub(SPRITE_WIDTH) as f32);
        self.cat.target_x = self
            .cat
            .target_x
            .clamp(-(SPRITE_WIDTH as f32), size.width as f32);
        self.cat.target_y = self.cat.target_y.clamp(0.0, max_y);
    }

    fn update(&mut self, ctx: &FrameContext<'_>, rng: &mut dyn Rng, _commands: &mut FrameCommands) {
        self.terminal_width = ctx.size.width;
        self.terminal_height = ctx.size.height;
        let Some(lawn) = lawn_bounds(ctx.size.width, ctx.size.height, ctx.horizon_y) else {
            return;
        };
        let Some(house) = ctx.house_bounds() else {
            return;
        };
        let weather_safe = !ctx.conditions.is_raining
            && !ctx.conditions.is_snowing
            && !ctx.conditions.is_thunderstorm;
        self.cat.advance(
            lawn,
            house,
            ctx.size.width,
            ctx.conditions.sun.is_day,
            weather_safe,
            rng,
        );
    }

    fn render(
        &mut self,
        renderer: &mut TerminalRenderer,
        ctx: &FrameContext<'_>,
    ) -> io::Result<()> {
        let Some(lawn) = lawn_bounds(ctx.size.width, ctx.size.height, ctx.horizon_y) else {
            return Ok(());
        };
        let Some(bounds) = AnchorBounds::from_lawn(lawn) else {
            return Ok(());
        };
        if self.cat.y < bounds.top as f32 || self.cat.y > bounds.bottom as f32 {
            return Ok(());
        }
        let frames: &[[&str; 3]] = if self.cat.state == CatActivity::Sleeping {
            &CAT_SLEEP_FRAMES
        } else {
            &CAT_WALK_FRAMES
        };
        let frame = &frames[self.cat.frame_index % frames.len()];
        render_sprite(
            renderer,
            frame,
            self.cat.x,
            self.cat.y,
            self.cat.direction,
            ctx.size.width,
            ctx.size.height,
        )
    }
}

fn render_sprite(
    renderer: &mut TerminalRenderer,
    frame: &[&str; 3],
    x: f32,
    y: f32,
    direction: i8,
    terminal_width: u16,
    terminal_height: u16,
) -> io::Result<()> {
    if !x.is_finite() || !y.is_finite() {
        return Ok(());
    }
    let anchor_x = x.floor().clamp(i32::MIN as f32, i32::MAX as f32) as i32;
    let anchor_y = y.floor().clamp(i32::MIN as f32, i32::MAX as f32) as i32;
    let width = terminal_width as i32;
    let height = terminal_height as i32;

    for (row, line) in frame.iter().enumerate() {
        let line_width = line.chars().count() as i32;
        let draw_y = anchor_y + row as i32;
        if draw_y < 0 || draw_y >= height {
            continue;
        }
        for (column, source_char) in line.chars().enumerate() {
            if source_char == ' ' {
                continue;
            }
            let character = if direction < 0 {
                mirror_char(source_char)
            } else {
                source_char
            };
            let column = column as i32;
            let draw_x = if direction < 0 {
                anchor_x + line_width.saturating_sub(1).saturating_sub(column)
            } else {
                anchor_x + column
            };
            if draw_x < 0 || draw_x >= width {
                continue;
            }
            renderer.render_char(
                draw_x as u16,
                draw_y as u16,
                character,
                cat_color(character),
            )?;
        }
    }
    Ok(())
}

fn mirror_char(character: char) -> char {
    match character {
        '(' => ')',
        ')' => '(',
        '/' => '\\',
        '\\' => '/',
        '<' => '>',
        '>' => '<',
        _ => character,
    }
}

fn cat_color(character: char) -> Color {
    match character {
        'o' => Color::Yellow,
        '.' | '-' => Color::DarkGrey,
        _ => Color::Grey,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::house_bounds;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    fn test_lawn() -> LawnBounds {
        lawn_bounds(80, 24, 17).unwrap()
    }

    fn test_house() -> HouseBounds {
        house_bounds(80, 24, 17).unwrap()
    }

    #[test]
    fn cat_frames_have_fixed_dimensions() {
        for frames in [&CAT_WALK_FRAMES[..], &CAT_SLEEP_FRAMES[..]] {
            for frame in frames {
                for line in frame {
                    assert_eq!(line.chars().count(), SPRITE_WIDTH as usize);
                }
            }
        }
    }

    #[test]
    fn cat_sleep_frames_keep_eyes_closed() {
        for frame in CAT_SLEEP_FRAMES {
            assert!(frame[1].contains("-.-"));
            assert!(!frame[1].contains("o.o"));
        }
    }

    #[test]
    fn cat_sleeps_at_a_house_adjacent_anchor() {
        let lawn = test_lawn();
        let house = test_house();
        let mut cat = Cat::new(80);
        let mut rng = StdRng::seed_from_u64(21);
        cat.advance(lawn, house, 80, true, true, &mut rng);

        assert_eq!(house.left, 8);
        assert_eq!(house.bottom, 16);
        assert_eq!(cat.sleep_x, 0.0);
        assert_eq!(cat.sleep_y, 17.0);
        assert_eq!(cat.x, cat.sleep_x);
        assert_eq!(cat.y, cat.sleep_y);
    }

    #[test]
    fn cat_anchor_uses_right_side_when_wide_and_clamps_when_tiny() {
        let mut wide = Cat::new(100);
        let wide_lawn = lawn_bounds(100, 24, 17).unwrap();
        let wide_house = house_bounds(100, 24, 17).unwrap();
        let mut rng = StdRng::seed_from_u64(3);
        wide.advance(wide_lawn, wide_house, 100, true, true, &mut rng);
        assert_eq!(wide.sleep_x, 82.0);

        let mut tiny = Cat::new(12);
        let tiny_lawn = lawn_bounds(12, 8, 1).unwrap();
        let tiny_house = house_bounds(12, 8, 1).unwrap();
        tiny.advance(tiny_lawn, tiny_house, 12, true, true, &mut rng);
        let tiny_bounds = AnchorBounds::from_lawn(tiny_lawn).unwrap();
        assert!(tiny.sleep_x >= tiny_bounds.left as f32);
        assert!(tiny.sleep_x <= tiny_bounds.right as f32);
        assert!(tiny.sleep_y >= tiny_bounds.top as f32);
        assert!(tiny.sleep_y <= tiny_bounds.bottom as f32);
    }

    #[test]
    fn cat_resize_recomputes_house_adjacent_anchor() {
        let mut system = CatSystem::new(80, 24);
        system.on_resize(TerminalSize {
            width: 80,
            height: 24,
        });
        assert_eq!(system.cat.sleep_x, 0.0);
        assert_eq!(system.cat.sleep_y, 17.0);

        system.on_resize(TerminalSize {
            width: 100,
            height: 24,
        });
        assert_eq!(system.cat.sleep_x, 82.0);
    }

    #[test]
    fn cat_sleeps_mostly_and_does_not_redecide_each_frame() {
        let mut cat = Cat::new(80);
        let lawn = test_lawn();
        let house = test_house();
        let mut rng = StdRng::seed_from_u64(21);
        cat.advance(lawn, house, 80, true, true, &mut rng);
        assert_eq!(cat.state, CatActivity::Sleeping);
        let timer = cat.state_timer;
        cat.advance(lawn, house, 80, true, true, &mut rng);
        assert_eq!(cat.state, CatActivity::Sleeping);
        assert_eq!(cat.state_timer, timer.saturating_sub(1));
    }
    #[test]
    fn cat_first_wake_enters_wandering_within_bounded_cycle() {
        let lawn = test_lawn();
        let house = test_house();
        let mut cat = Cat::new(80);
        let mut rng = StdRng::seed_from_u64(99);
        cat.advance(lawn, house, 80, true, true, &mut rng);
        cat.state_timer = 0;
        cat.advance(lawn, house, 80, true, true, &mut rng);
        assert_eq!(cat.state, CatActivity::Wandering);
        assert!(cat.has_woken);
        assert!(WALK_MIN <= WALK_MAX);
    }

    #[test]
    fn cat_wandering_moves_with_slow_bounded_steps() {
        let lawn = test_lawn();
        let house = test_house();
        let bounds = AnchorBounds::from_lawn(lawn).unwrap();
        let mut cat = Cat::new(80);
        let mut rng = StdRng::seed_from_u64(1);
        cat.advance(lawn, house, 80, true, true, &mut rng);
        cat.state = CatActivity::Wandering;
        cat.target_x = bounds.right as f32;
        cat.target_y = bounds.top as f32;
        cat.state_timer = WALK_MAX;
        cat.update_direction();
        let start_x = cat.x;
        let start_y = cat.y;
        cat.advance(lawn, house, 80, true, true, &mut rng);
        assert_eq!(cat.state, CatActivity::Wandering);
        assert!(cat.x != start_x || cat.y != start_y);
        assert!(CAT_MOVE_SPEED <= 0.12);
        assert!(FRAME_STEP >= 12);
    }

    #[test]
    fn cat_hunting_can_leave_screen_and_returns_to_sleep_spot() {
        let lawn = test_lawn();
        let house = test_house();
        let mut cat = Cat::new(80);
        let mut rng = StdRng::seed_from_u64(1);
        cat.advance(lawn, house, 80, true, true, &mut rng);
        let bounds = AnchorBounds::from_lawn(lawn).unwrap();
        cat.set_lawn_spot(bounds, house, false);
        cat.state = CatActivity::Hunting;
        cat.direction = -1;
        cat.x = -(SPRITE_WIDTH as f32 + 0.5);
        cat.state_timer = RETURN_TIMEOUT;
        cat.advance(lawn, house, 80, true, true, &mut rng);
        assert_eq!(cat.state, CatActivity::Returning);
        assert!(cat.x < 0.0);

        for _ in 0..300 {
            cat.advance(lawn, house, 80, true, true, &mut rng);
            if cat.state == CatActivity::Sleeping {
                break;
            }
        }
        assert_eq!(cat.state, CatActivity::Sleeping);
        assert!((cat.x - cat.sleep_x).abs() < f32::EPSILON);
    }

    #[test]
    fn cat_night_forces_return_without_teleporting() {
        let lawn = test_lawn();
        let house = test_house();
        let mut cat = Cat::new(80);
        let mut rng = StdRng::seed_from_u64(2);
        cat.advance(lawn, house, 80, true, true, &mut rng);
        cat.state = CatActivity::Hunting;
        cat.x = 82.0;
        cat.direction = 1;
        let x_before = cat.x;
        cat.advance(lawn, house, 80, false, true, &mut rng);
        assert_eq!(cat.state, CatActivity::Returning);
        assert!(cat.x < x_before);
        assert!(cat.x > cat.sleep_x);
    }

    #[test]
    fn overdue_return_keeps_moving_instead_of_teleporting() {
        let lawn = test_lawn();
        let house = test_house();
        let mut cat = Cat::new(80);
        let mut rng = StdRng::seed_from_u64(9);
        cat.advance(lawn, house, 80, true, true, &mut rng);
        cat.state = CatActivity::Returning;
        cat.x = -100.0;
        cat.state_timer = 1;
        let x_before = cat.x;
        cat.advance(lawn, house, 80, false, true, &mut rng);
        assert_eq!(cat.state, CatActivity::Returning);
        assert!(cat.x > x_before);
        assert!(cat.x < cat.sleep_x);
    }

    #[test]
    fn cat_mirror_preserves_non_directional_symbols() {
        assert_eq!(mirror_char('/'), '\\');
        assert_eq!(mirror_char('o'), 'o');
        assert_eq!(mirror_char('^'), '^');
    }
}
