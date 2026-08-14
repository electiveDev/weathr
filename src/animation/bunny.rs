use crate::animation::{AnimationSystem, FrameCommands, FrameContext, RenderLayer, TerminalSize};
use crate::render::TerminalRenderer;
use crate::scene::{LawnBounds, lawn_bounds};
use crossterm::style::Color;
use rand::{Rng, RngExt};
use std::io;

const HOP_FRAME_STEP: u8 = 6;
const BUNNY_SPEED: f32 = 0.5;
const INITIAL_SPAWN_MIN: u16 = 1;
const INITIAL_SPAWN_MAX: u16 = 45;
const BETWEEN_RUNS_MIN: u16 = 450;
const BETWEEN_RUNS_MAX: u16 = 1_200;

pub const SPRITE_WIDTH: u16 = 9;
pub const SPRITE_HEIGHT: u16 = 3;

// The right-facing frame is mirrored when the bunny travels left.  Every row
// is deliberately nine cells wide so an off-screen anchor can be clipped
// without changing the sprite's footprint.
const BUNNY_FRAMES: [[&str; SPRITE_HEIGHT as usize]; 2] = [
    ["  /\\_/\\\\ ", " (o.o)>  ", "  /\\ /\\  "],
    ["  /\\_/\\\\ ", "  (o.o)> ", " /\\  /\\  "],
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BunnyActivity {
    Waiting,
    Hopping,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BunnyBounds {
    anchor_y: u16,
}

impl BunnyBounds {
    fn from_lawn(lawn: LawnBounds) -> Option<Self> {
        if lawn.width() < SPRITE_WIDTH || lawn.height() < SPRITE_HEIGHT {
            return None;
        }

        Some(Self {
            anchor_y: lawn.bottom.saturating_sub(SPRITE_HEIGHT - 1),
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct Bunny {
    x: f32,
    y: f32,
    direction: i8,
    activity: BunnyActivity,
    wait_timer: u16,
    initial_delay_selected: bool,
    hop_frame: usize,
    hop_tick: u8,
}

impl Bunny {
    fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            direction: 1,
            activity: BunnyActivity::Waiting,
            wait_timer: 0,
            initial_delay_selected: false,
            hop_frame: 0,
            hop_tick: 0,
        }
    }

    fn spawn(&mut self, terminal_width: u16, bounds: BunnyBounds, rng: &mut (impl Rng + ?Sized)) {
        self.direction = if rng.random_bool(0.5) { 1 } else { -1 };
        self.x = if self.direction > 0 {
            -(SPRITE_WIDTH as f32)
        } else {
            terminal_width as f32
        };
        self.y = bounds.anchor_y as f32;
        self.activity = BunnyActivity::Hopping;
        self.hop_frame = 0;
        self.hop_tick = 0;
    }

    fn finish_run(&mut self, rng: &mut (impl Rng + ?Sized)) {
        self.activity = BunnyActivity::Waiting;
        self.wait_timer = rng.random_range(BETWEEN_RUNS_MIN..=BETWEEN_RUNS_MAX);
        self.hop_frame = 0;
        self.hop_tick = 0;
    }

    fn advance(&mut self, terminal_width: u16, bounds: BunnyBounds, rng: &mut (impl Rng + ?Sized)) {
        self.y = bounds.anchor_y as f32;

        if self.activity == BunnyActivity::Waiting {
            if self.wait_timer > 0 {
                self.wait_timer = self.wait_timer.saturating_sub(1);
                return;
            }

            // Select the first delay lazily so construction stays deterministic
            // while the first appearance remains random and quick in the TUI.
            if !self.initial_delay_selected {
                self.initial_delay_selected = true;
                self.wait_timer = rng.random_range(INITIAL_SPAWN_MIN..=INITIAL_SPAWN_MAX);
                return;
            }

            self.spawn(terminal_width, bounds, rng);
            return;
        }

        self.x += self.direction as f32 * BUNNY_SPEED;
        self.hop_tick = self.hop_tick.saturating_add(1);
        if self.hop_tick >= HOP_FRAME_STEP {
            self.hop_frame = (self.hop_frame + 1) % BUNNY_FRAMES.len();
            self.hop_tick = 0;
        }

        let fully_off_left = self.x <= -(SPRITE_WIDTH as f32);
        let fully_off_right = self.x >= terminal_width as f32;
        if fully_off_left || fully_off_right {
            self.finish_run(rng);
        }
    }
}

pub struct BunnySystem {
    bunny: Bunny,
    terminal_width: u16,
}

impl BunnySystem {
    pub fn new(terminal_width: u16, _terminal_height: u16) -> Self {
        Self {
            bunny: Bunny::new(),
            terminal_width,
        }
    }
}

impl AnimationSystem for BunnySystem {
    fn id(&self) -> &'static str {
        "bunny"
    }

    fn layer(&self) -> RenderLayer {
        RenderLayer::Foreground
    }

    fn on_resize(&mut self, size: TerminalSize) {
        self.terminal_width = size.width;
    }

    fn update(&mut self, ctx: &FrameContext<'_>, rng: &mut dyn Rng, _commands: &mut FrameCommands) {
        self.terminal_width = ctx.size.width;
        let terminal_width = self.terminal_width;

        let Some(lawn) = lawn_bounds(ctx.size.width, ctx.size.height, ctx.horizon_y) else {
            return;
        };
        let Some(bounds) = BunnyBounds::from_lawn(lawn) else {
            return;
        };

        self.bunny.advance(terminal_width, bounds, rng);
    }

    fn render(
        &mut self,
        renderer: &mut TerminalRenderer,
        ctx: &FrameContext<'_>,
    ) -> io::Result<()> {
        let Some(lawn) = lawn_bounds(ctx.size.width, ctx.size.height, ctx.horizon_y) else {
            return Ok(());
        };
        let Some((x, y)) = render_anchor(lawn, &self.bunny) else {
            return Ok(());
        };

        let frame = &BUNNY_FRAMES[self.bunny.hop_frame % BUNNY_FRAMES.len()];
        render_sprite(
            renderer,
            frame,
            x,
            y,
            self.bunny.direction,
            ctx.size.width,
            ctx.size.height,
        )
    }
}

fn render_anchor(lawn: LawnBounds, bunny: &Bunny) -> Option<(f32, f32)> {
    let bounds = BunnyBounds::from_lawn(lawn)?;
    if bunny.activity != BunnyActivity::Hopping
        || !bunny.x.is_finite()
        || !bunny.y.is_finite()
        || bunny.y < bounds.anchor_y as f32
        || bunny.y > bounds.anchor_y as f32
    {
        return None;
    }
    Some((bunny.x, bunny.y))
}

fn render_sprite(
    renderer: &mut TerminalRenderer,
    frame: &[&str; SPRITE_HEIGHT as usize],
    x: f32,
    y: f32,
    direction: i8,
    terminal_width: u16,
    terminal_height: u16,
) -> io::Result<()> {
    if !x.is_finite() || !y.is_finite() || terminal_width == 0 || terminal_height == 0 {
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

            let column = column as i32;
            let (draw_x, character) = if direction < 0 {
                (
                    anchor_x + line_width.saturating_sub(1).saturating_sub(column),
                    mirror_char(source_char),
                )
            } else {
                (anchor_x + column, source_char)
            };
            if draw_x < 0 || draw_x >= width {
                continue;
            }

            renderer.render_char(
                draw_x as u16,
                draw_y as u16,
                character,
                bunny_color(character),
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

fn bunny_color(character: char) -> Color {
    match character {
        'o' => Color::White,
        '.' | '_' => Color::DarkGrey,
        '<' | '>' => Color::Yellow,
        _ => Color::Grey,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::lawn_bounds;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    fn test_bounds() -> BunnyBounds {
        BunnyBounds::from_lawn(lawn_bounds(80, 24, 17).unwrap()).unwrap()
    }

    #[test]
    fn bunny_frames_have_fixed_dimensions() {
        for frame in BUNNY_FRAMES {
            assert_eq!(frame.len(), SPRITE_HEIGHT as usize);
            for line in frame {
                assert_eq!(line.chars().count(), SPRITE_WIDTH as usize);
            }
        }
    }

    #[test]
    fn bunny_mirroring_reverses_directional_symbols() {
        assert_eq!(mirror_char('/'), '\\');
        assert_eq!(mirror_char('\\'), '/');
        assert_eq!(mirror_char('>'), '<');
        assert_eq!(mirror_char('<'), '>');
        assert_eq!(mirror_char('('), ')');
        assert_eq!(mirror_char(')'), '(');
        assert_eq!(mirror_char('o'), 'o');

        let source = BUNNY_FRAMES[0][1];
        let mirrored: String = source.chars().rev().map(mirror_char).collect();
        assert_eq!(mirrored, "  <(o.o) ");
    }

    #[test]
    fn bunny_hop_frame_changes_after_step_interval() {
        let bounds = test_bounds();
        let mut bunny = Bunny::new();
        bunny.initial_delay_selected = true;
        let mut rng = StdRng::seed_from_u64(7);
        bunny.advance(80, bounds, &mut rng);
        assert_eq!(bunny.activity, BunnyActivity::Hopping);
        assert_eq!(bunny.hop_frame, 0);

        for _ in 0..HOP_FRAME_STEP {
            bunny.advance(80, bounds, &mut rng);
        }
        assert_eq!(bunny.hop_frame, 1);
        assert_ne!(BUNNY_FRAMES[0], BUNNY_FRAMES[1]);
    }

    #[test]
    fn bunny_spawns_at_edge_moves_and_despawns_after_crossing() {
        let bounds = test_bounds();
        let mut bunny = Bunny::new();
        let mut rng = StdRng::seed_from_u64(9);
        bunny.spawn(80, bounds, &mut rng);
        let direction = bunny.direction;
        let spawn_x = bunny.x;
        assert_eq!(spawn_x, if direction > 0 { -9.0 } else { 80.0 });

        bunny.advance(80, bounds, &mut rng);
        assert!(bunny.activity == BunnyActivity::Hopping);
        assert_ne!(bunny.x, spawn_x);

        for _ in 0..400 {
            if bunny.activity == BunnyActivity::Waiting {
                break;
            }
            bunny.advance(80, bounds, &mut rng);
        }
        assert_eq!(bunny.activity, BunnyActivity::Waiting);
        assert!(if direction > 0 {
            bunny.x >= 80.0
        } else {
            bunny.x <= -9.0
        });
        assert!(bunny.wait_timer >= BETWEEN_RUNS_MIN);
    }

    #[test]
    fn tiny_lawn_has_no_render_anchor() {
        let mut bunny = Bunny::new();
        bunny.activity = BunnyActivity::Hopping;
        bunny.x = 0.0;
        bunny.y = 0.0;

        let narrow_lawn = lawn_bounds(8, 24, 17).unwrap();
        assert!(render_anchor(narrow_lawn, &bunny).is_none());

        let shallow_lawn = lawn_bounds(80, 4, 2).unwrap();
        assert!(render_anchor(shallow_lawn, &bunny).is_none());
    }
}
