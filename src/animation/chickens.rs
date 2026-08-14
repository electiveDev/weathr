use crate::animation::{AnimationSystem, FrameCommands, FrameContext, RenderLayer, TerminalSize};
use crate::render::TerminalRenderer;
use crate::scene::{LawnBounds, PondBounds, lawn_bounds, pond_bounds};
use crossterm::style::Color;
use rand::{Rng, RngExt};
use std::io;

const FRAME_STEP: u8 = 4;
pub const SPRITE_WIDTH: u16 = 5;
pub const SPRITE_HEIGHT: u16 = 3;
const WANDER_MIN: u16 = 70;
const WANDER_MAX: u16 = 160;
const PECK_MIN: u16 = 45;
const PECK_MAX: u16 = 90;
const SUNBATHE_MIN: u16 = 90;
const SUNBATHE_MAX: u16 = 180;
const DRINK_MIN: u16 = 60;
const DRINK_MAX: u16 = 120;
const DRINK_TRAVEL_SPEED: f32 = 0.70;

const CHICKEN_WALK_FRAMES: [[&str; 3]; 4] = [
    ["  __ ", " (o)>", " / \\ "],
    [" ___ ", "(o )>", " /\\  "],
    ["  __ ", " (o)>", " _/  "],
    [" _v_ ", " (o)>", " / \\ "],
];
const CHICKEN_PECK_FRAMES: [[&str; 3]; 2] =
    [["  __ ", " (o)_", " / \\ "], ["  __ ", "(o)  ", " /\\  "]];
const CHICKEN_SUNBATHE_FRAMES: [[&str; 3]; 2] =
    [["  __ ", " (o) ", " / \\ "], [" _v_ ", " (o) ", " / \\ "]];
const CHICKEN_DRINK_FRAMES: [[&str; 3]; 2] =
    [["  __ ", " (o) ", " /~\\ "], ["  __ ", " (o)_", " /~\\ "]];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChickenActivity {
    Wandering,
    Pecking,
    Sunbathing,
    Drinking,
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

struct Chicken {
    x: f32,
    y: f32,
    target_x: f32,
    target_y: f32,
    direction: i8,
    speed: f32,
    state: ChickenActivity,
    state_timer: u16,
    target_reached: bool,
    frame_index: usize,
    frame_tick: u8,
}

impl Chicken {
    fn new(x: f32, direction: i8, speed: f32) -> Self {
        Self {
            x,
            y: 0.0,
            target_x: x,
            target_y: 0.0,
            direction: if direction < 0 { -1 } else { 1 },
            speed,
            state: ChickenActivity::Wandering,
            state_timer: 0,
            target_reached: false,
            frame_index: 0,
            frame_tick: 0,
        }
    }

    fn reset_frame(&mut self) {
        self.frame_index = 0;
        self.frame_tick = 0;
    }

    fn start_ground_activity(&mut self, state: ChickenActivity, timer: u16, bounds: AnchorBounds) {
        self.state = state;
        self.state_timer = timer;
        self.target_x = bounds.clamp_x(self.x);
        self.target_y = bounds.clamp_y(self.y);
        self.target_reached = true;
        self.reset_frame();
    }

    fn start_drinking(&mut self, timer: u16, bounds: AnchorBounds, pond: PondBounds) {
        self.state = ChickenActivity::Drinking;
        self.state_timer = timer;
        self.target_x = bounds.clamp_x(pond.center_x().saturating_sub(SPRITE_WIDTH / 2) as f32);
        self.target_y = bounds.clamp_y(pond.y.saturating_sub(SPRITE_HEIGHT - 1) as f32);
        self.target_reached = false;
        self.update_direction();
        self.reset_frame();
    }

    fn choose_activity(
        &mut self,
        rng: &mut (impl Rng + ?Sized),
        bounds: AnchorBounds,
        pond: Option<PondBounds>,
        sunny: bool,
    ) {
        let roll = rng.random_range(0..100u8);
        if sunny && roll < 20 {
            self.start_ground_activity(
                ChickenActivity::Sunbathing,
                rng.random_range(SUNBATHE_MIN..=SUNBATHE_MAX),
                bounds,
            );
        } else if let Some(pond) = pond.filter(|_| roll < 32) {
            self.start_drinking(rng.random_range(DRINK_MIN..=DRINK_MAX), bounds, pond);
        } else if roll < 64 {
            self.start_ground_activity(
                ChickenActivity::Pecking,
                rng.random_range(PECK_MIN..=PECK_MAX),
                bounds,
            );
        } else {
            self.state = ChickenActivity::Wandering;
            self.state_timer = rng.random_range(WANDER_MIN..=WANDER_MAX);
            self.target_reached = false;
            self.target_x = rng.random_range(bounds.left..=bounds.right) as f32;
            self.target_y = rng.random_range(bounds.top..=bounds.bottom) as f32;
            self.update_direction();
            self.reset_frame();
        }
    }

    fn update_direction(&mut self) {
        let dx = self.target_x - self.x;
        if dx > 0.05 {
            self.direction = 1;
        } else if dx < -0.05 {
            self.direction = -1;
        }
    }

    fn move_towards_target(&mut self, bounds: AnchorBounds) {
        let dx = self.target_x - self.x;
        let dy = self.target_y - self.y;
        let distance = (dx * dx + dy * dy).sqrt();
        let speed = if self.state == ChickenActivity::Drinking {
            self.speed.max(DRINK_TRAVEL_SPEED)
        } else {
            self.speed.max(0.05)
        };
        if distance <= speed {
            self.x = self.target_x;
            self.y = self.target_y;
            self.target_reached = true;
            return;
        }

        let step = speed / distance;
        self.x = bounds.clamp_x(self.x + dx * step);
        self.y = bounds.clamp_y(self.y + dy * step);
        self.update_direction();
    }

    fn advance(
        &mut self,
        lawn: LawnBounds,
        pond: Option<PondBounds>,
        sunny: bool,
        rng: &mut (impl Rng + ?Sized),
    ) {
        let Some(bounds) = AnchorBounds::from_lawn(lawn) else {
            return;
        };
        self.x = bounds.clamp_x(self.x);
        self.y = bounds.clamp_y(self.y);

        match self.state {
            ChickenActivity::Wandering => {
                if self.state_timer == 0 {
                    self.choose_activity(rng, bounds, pond, sunny);
                } else {
                    self.move_towards_target(bounds);
                    self.state_timer = self.state_timer.saturating_sub(1);
                    if self.target_reached {
                        self.choose_activity(rng, bounds, pond, sunny);
                    }
                }
            }
            ChickenActivity::Drinking => {
                if !self.target_reached {
                    self.move_towards_target(bounds);
                } else if self.state_timer == 0 {
                    self.choose_activity(rng, bounds, pond, sunny);
                } else {
                    self.state_timer = self.state_timer.saturating_sub(1);
                }
            }
            ChickenActivity::Pecking => {
                if self.state_timer == 0 {
                    self.choose_activity(rng, bounds, pond, sunny);
                } else {
                    self.state_timer = self.state_timer.saturating_sub(1);
                }
            }
            ChickenActivity::Sunbathing => {
                if !sunny || self.state_timer == 0 {
                    self.choose_activity(rng, bounds, pond, sunny);
                } else {
                    self.state_timer = self.state_timer.saturating_sub(1);
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

pub struct ChickenSystem {
    chickens: Vec<Chicken>,
    terminal_width: u16,
    terminal_height: u16,
}

impl ChickenSystem {
    pub fn new(terminal_width: u16, terminal_height: u16) -> Self {
        let max_x = max_x(terminal_width).max(1);
        let third = (max_x / 3).max(1);
        let second_x = (third * 2).min(max_x);

        let mut chickens = vec![
            Chicken::new(1.0, 1, 0.14),
            Chicken::new(third as f32, -1, 0.16),
            Chicken::new(second_x as f32, 1, 0.12),
        ];
        let initial_ground_y = terminal_height.saturating_sub(7);
        if let Some(bounds) = lawn_bounds(terminal_width, terminal_height, initial_ground_y)
            .and_then(AnchorBounds::from_lawn)
        {
            let middle_y = bounds
                .top
                .saturating_add(bounds.bottom.saturating_sub(bounds.top) / 2);
            let y_positions = [bounds.top, middle_y, bounds.bottom];
            for (index, chicken) in chickens.iter_mut().enumerate() {
                chicken.y = y_positions[index] as f32;
                chicken.target_y = chicken.y;
            }

            chickens[0].start_ground_activity(ChickenActivity::Pecking, PECK_MAX, bounds);
            chickens[2].start_ground_activity(ChickenActivity::Sunbathing, SUNBATHE_MAX, bounds);

            if let Some(pond) = pond_bounds(terminal_width, terminal_height, initial_ground_y) {
                chickens[1].x =
                    bounds.clamp_x(pond.center_x().saturating_sub(SPRITE_WIDTH / 2) as f32);
                chickens[1].y = bounds.clamp_y(pond.y.saturating_sub(SPRITE_HEIGHT - 1) as f32);
                chickens[1].start_drinking(DRINK_MAX, bounds, pond);
            } else {
                chickens[1].start_ground_activity(ChickenActivity::Pecking, PECK_MAX, bounds);
            }
        }

        Self {
            chickens,
            terminal_width,
            terminal_height,
        }
    }

    fn update_chickens(&mut self, ctx: &FrameContext<'_>, rng: &mut dyn Rng) {
        self.terminal_width = ctx.size.width;
        self.terminal_height = ctx.size.height;
        let Some(lawn) = lawn_bounds(ctx.size.width, ctx.size.height, ctx.horizon_y) else {
            return;
        };
        let pond = pond_bounds(ctx.size.width, ctx.size.height, ctx.horizon_y);
        let sunny =
            ctx.conditions.sun.is_day && !ctx.conditions.is_cloudy && !ctx.conditions.is_foggy;
        for chicken in &mut self.chickens {
            chicken.advance(lawn, pond, sunny, rng);
        }
    }

    fn render_chickens(
        &self,
        renderer: &mut TerminalRenderer,
        ctx: &FrameContext<'_>,
    ) -> io::Result<()> {
        let Some(lawn) = lawn_bounds(ctx.size.width, ctx.size.height, ctx.horizon_y) else {
            return Ok(());
        };
        let Some(bounds) = AnchorBounds::from_lawn(lawn) else {
            return Ok(());
        };

        for chicken in &self.chickens {
            if chicken.x < bounds.left as f32
                || chicken.x > bounds.right as f32
                || chicken.y < bounds.top as f32
                || chicken.y > bounds.bottom as f32
            {
                continue;
            }
            let visible_state =
                if chicken.state == ChickenActivity::Drinking && !chicken.target_reached {
                    ChickenActivity::Wandering
                } else {
                    chicken.state
                };
            let frames = frames_for(visible_state);
            let frame = &frames[chicken.frame_index % frames.len()];
            render_sprite(
                renderer,
                frame,
                chicken.x.round() as u16,
                chicken.y.round() as u16,
                chicken.direction,
            )?;
        }
        Ok(())
    }
}

impl AnimationSystem for ChickenSystem {
    fn id(&self) -> &'static str {
        "chickens"
    }

    fn layer(&self) -> RenderLayer {
        RenderLayer::Foreground
    }

    fn is_active(&self, ctx: &FrameContext<'_>) -> bool {
        ctx.conditions.sun.is_day
            && !ctx.conditions.is_raining
            && !ctx.conditions.is_snowing
            && !ctx.conditions.is_thunderstorm
    }

    fn on_resize(&mut self, size: TerminalSize) {
        self.terminal_width = size.width;
        self.terminal_height = size.height;

        let ground_y = size.height.saturating_sub(7);
        let bounds =
            lawn_bounds(size.width, size.height, ground_y).and_then(AnchorBounds::from_lawn);
        let pond = pond_bounds(size.width, size.height, ground_y);
        let max_x = max_x(size.width) as f32;
        let fallback_max_y = size.height.saturating_sub(SPRITE_HEIGHT + 2) as f32;

        for chicken in &mut self.chickens {
            if let Some(bounds) = bounds {
                chicken.x = bounds.clamp_x(chicken.x);
                chicken.y = bounds.clamp_y(chicken.y);
                chicken.target_x = bounds.clamp_x(chicken.target_x);
                chicken.target_y = bounds.clamp_y(chicken.target_y);

                if chicken.state == ChickenActivity::Drinking {
                    if let Some(pond) = pond {
                        chicken.target_x =
                            bounds.clamp_x(pond.center_x().saturating_sub(SPRITE_WIDTH / 2) as f32);
                        chicken.target_y =
                            bounds.clamp_y(pond.y.saturating_sub(SPRITE_HEIGHT - 1) as f32);
                        chicken.target_reached = false;
                        chicken.update_direction();
                    } else {
                        chicken.state = ChickenActivity::Wandering;
                        chicken.state_timer = 0;
                        chicken.target_reached = false;
                    }
                }
            } else {
                chicken.x = chicken.x.clamp(0.0, max_x);
                chicken.y = chicken.y.clamp(0.0, fallback_max_y);
                chicken.target_x = chicken.target_x.clamp(0.0, max_x);
                chicken.target_y = chicken.target_y.clamp(0.0, fallback_max_y);
            }
        }
    }

    fn update(&mut self, ctx: &FrameContext<'_>, rng: &mut dyn Rng, _commands: &mut FrameCommands) {
        self.update_chickens(ctx, rng);
    }

    fn render(
        &mut self,
        renderer: &mut TerminalRenderer,
        ctx: &FrameContext<'_>,
    ) -> io::Result<()> {
        self.render_chickens(renderer, ctx)
    }
}

fn max_x(width: u16) -> u16 {
    width.saturating_sub(SPRITE_WIDTH)
}

fn frames_for(state: ChickenActivity) -> &'static [[&'static str; 3]] {
    match state {
        ChickenActivity::Wandering => &CHICKEN_WALK_FRAMES,
        ChickenActivity::Pecking => &CHICKEN_PECK_FRAMES,
        ChickenActivity::Sunbathing => &CHICKEN_SUNBATHE_FRAMES,
        ChickenActivity::Drinking => &CHICKEN_DRINK_FRAMES,
    }
}

fn render_sprite(
    renderer: &mut TerminalRenderer,
    frame: &[&str; 3],
    x: u16,
    y: u16,
    direction: i8,
) -> io::Result<()> {
    for (row, line) in frame.iter().enumerate() {
        let line_width = line.chars().count() as u16;
        for (column, source_char) in line.chars().enumerate() {
            if source_char == ' ' {
                continue;
            }

            let character = if direction < 0 {
                mirror_char(source_char)
            } else {
                source_char
            };
            let column = column as u16;
            let draw_x = if direction < 0 {
                x.saturating_add(line_width.saturating_sub(1).saturating_sub(column))
            } else {
                x.saturating_add(column)
            };
            renderer.render_char(
                draw_x,
                y.saturating_add(row as u16),
                character,
                chicken_color(character),
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

fn chicken_color(character: char) -> Color {
    match character {
        '<' | '>' => Color::Yellow,
        '~' => Color::Blue,
        'o' => Color::DarkGrey,
        _ => Color::White,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    fn test_lawn() -> LawnBounds {
        lawn_bounds(80, 24, 17).unwrap()
    }

    #[test]
    fn chicken_frames_have_fixed_dimensions() {
        for frames in [
            &CHICKEN_WALK_FRAMES[..],
            &CHICKEN_PECK_FRAMES[..],
            &CHICKEN_SUNBATHE_FRAMES[..],
            &CHICKEN_DRINK_FRAMES[..],
        ] {
            for frame in frames {
                for line in frame {
                    assert_eq!(line.chars().count(), SPRITE_WIDTH as usize);
                }
            }
        }
    }
    #[test]
    fn initial_activities_are_long_enough_to_observe() {
        let system = ChickenSystem::new(80, 24);
        let states: Vec<_> = system
            .chickens
            .iter()
            .map(|chicken| chicken.state)
            .collect();
        assert_eq!(
            states,
            vec![
                ChickenActivity::Pecking,
                ChickenActivity::Drinking,
                ChickenActivity::Sunbathing
            ]
        );

        let lawn = test_lawn();
        let bounds = AnchorBounds::from_lawn(lawn).unwrap();
        let pond = pond_bounds(80, 24, 17).unwrap();
        for chicken in &system.chickens {
            assert!(chicken.y >= bounds.top as f32);
            assert!(chicken.y <= bounds.bottom as f32);
            assert!(chicken.state_timer >= PECK_MIN);
        }
        assert_eq!(
            system.chickens[1].target_x,
            bounds.clamp_x(pond.center_x().saturating_sub(SPRITE_WIDTH / 2) as f32)
        );
        assert_eq!(
            system.chickens[1].target_y,
            bounds.clamp_y(pond.y.saturating_sub(SPRITE_HEIGHT - 1) as f32)
        );
        assert_ne!(system.chickens[0].y, system.chickens[2].y);
    }

    #[test]
    fn activity_sprites_make_observed_states_distinct() {
        assert!(
            frames_for(ChickenActivity::Pecking)
                .iter()
                .flatten()
                .any(|line| line.contains('_'))
        );
        assert!(
            frames_for(ChickenActivity::Drinking)
                .iter()
                .flatten()
                .any(|line| line.contains('~'))
        );
        assert!(
            frames_for(ChickenActivity::Sunbathing)
                .iter()
                .flatten()
                .any(|line| line.contains("_v_"))
        );
    }

    #[test]
    fn chicken_targets_stay_on_lower_lawn_and_can_reach_pond() {
        let lawn = test_lawn();
        let bounds = AnchorBounds::from_lawn(lawn).unwrap();
        let pond = pond_bounds(80, 24, 17).unwrap();
        let mut rng = StdRng::seed_from_u64(7);
        let mut chicken = Chicken::new(5.0, 1, 0.2);
        chicken.choose_activity(&mut rng, bounds, Some(pond), false);
        assert!(chicken.target_x >= bounds.left as f32);
        assert!(chicken.target_x <= bounds.right as f32);
        assert!(chicken.target_y >= bounds.top as f32);
        assert!(chicken.target_y <= bounds.bottom as f32);

        chicken.state = ChickenActivity::Drinking;
        chicken.target_reached = false;
        chicken.target_x = pond.center_x() as f32;
        chicken.target_y = pond.y.saturating_sub(SPRITE_HEIGHT - 1) as f32;
        chicken.x = chicken.target_x;
        chicken.y = chicken.target_y;
        chicken.state_timer = 2;
        chicken.advance(lawn, Some(pond), false, &mut rng);
        assert!(chicken.target_reached);
    }
    #[test]
    fn drinking_reaches_the_visible_pond_in_a_bounded_window() {
        let lawn = test_lawn();
        let bounds = AnchorBounds::from_lawn(lawn).unwrap();
        let pond = pond_bounds(80, 24, 17).unwrap();
        let mut chicken = Chicken::new(bounds.left as f32, 1, 0.12);
        chicken.y = bounds.top as f32;
        let mut rng = StdRng::seed_from_u64(19);
        chicken.start_drinking(DRINK_MAX, bounds, pond);

        for _ in 0..120 {
            chicken.advance(lawn, Some(pond), true, &mut rng);
            if chicken.target_reached {
                break;
            }
        }

        assert!(chicken.target_reached);
        assert_eq!(
            chicken.x,
            bounds.clamp_x(pond.center_x().saturating_sub(SPRITE_WIDTH / 2) as f32)
        );
        assert_eq!(
            chicken.y,
            bounds.clamp_y(pond.y.saturating_sub(SPRITE_HEIGHT - 1) as f32)
        );
    }

    #[test]
    fn chickens_choose_independent_targets_and_directions() {
        let bounds = AnchorBounds::from_lawn(test_lawn()).unwrap();
        let mut rng = StdRng::seed_from_u64(11);
        let mut first = Chicken::new(2.0, 1, 0.2);
        let mut second = Chicken::new(50.0, -1, 0.2);
        first.choose_activity(&mut rng, bounds, None, false);
        second.choose_activity(&mut rng, bounds, None, false);
        assert!(
            first.target_x != second.target_x
                || first.target_y != second.target_y
                || first.direction != second.direction
                || first.state != second.state
        );
    }

    #[test]
    fn chicken_stops_sunbathing_when_sun_is_gone() {
        let lawn = test_lawn();
        let bounds = AnchorBounds::from_lawn(lawn).unwrap();
        let mut chicken = Chicken::new(10.0, 1, 0.2);
        chicken.x = bounds.left as f32;
        chicken.y = bounds.top as f32;
        chicken.state = ChickenActivity::Sunbathing;
        chicken.state_timer = 20;
        let mut rng = StdRng::seed_from_u64(31);
        chicken.advance(lawn, None, false, &mut rng);
        assert_ne!(chicken.state, ChickenActivity::Sunbathing);
    }

    #[test]
    fn resize_clamps_chickens_to_the_visible_lawn() {
        let mut system = ChickenSystem::new(100, 40);
        system.chickens[0].x = 99.0;
        system.chickens[0].y = 39.0;
        system.on_resize(TerminalSize {
            width: 12,
            height: 8,
        });
        let bounds = AnchorBounds::from_lawn(lawn_bounds(12, 8, 1).unwrap()).unwrap();
        assert!(system.chickens[0].x <= bounds.right as f32);
        assert!(system.chickens[0].y <= bounds.bottom as f32);
        assert!(system.chickens[0].y >= bounds.top as f32);
    }

    #[test]
    fn chicken_turns_around_at_a_target() {
        let lawn = test_lawn();
        let bounds = AnchorBounds::from_lawn(lawn).unwrap();
        let mut chicken = Chicken::new(10.0, 1, 0.2);
        chicken.target_x = 10.0;
        chicken.y = bounds.top as f32;
        chicken.state = ChickenActivity::Wandering;
        chicken.state_timer = 20;
        let mut rng = StdRng::seed_from_u64(3);
        chicken.advance(lawn, None, false, &mut rng);
        assert!(chicken.state_timer > 0);
    }

    #[test]
    fn chicken_mirror_changes_facing_symbols() {
        assert_eq!(mirror_char('>'), '<');
        assert_eq!(mirror_char('/'), '\\');
        assert_eq!(mirror_char('o'), 'o');
    }
}
