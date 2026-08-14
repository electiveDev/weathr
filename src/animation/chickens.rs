use crate::animation::{AnimationSystem, FrameCommands, FrameContext, RenderLayer, TerminalSize};
use crate::render::TerminalRenderer;
use crossterm::style::Color;
use rand::Rng;
use std::io;

const FRAME_STEP: u8 = 4;
const SPRITE_WIDTH: u16 = 5;
const SPRITE_HEIGHT: u16 = 3;
const CHICKEN_FRAMES: [[&str; 3]; 4] = [
    ["  __ ", " (o)>", " / \\ "],
    [" ___ ", "(o )>", " /\\  "],
    ["  __ ", " (o)>", " _/  "],
    [" _v_ ", " (o)>", " / \\ "],
];

struct Chicken {
    x: f32,
    direction: i8,
    speed: f32,
    frame_index: usize,
    frame_tick: u8,
}

impl Chicken {
    fn new(x: f32, direction: i8, speed: f32) -> Self {
        Self {
            x,
            direction,
            speed,
            frame_index: 0,
            frame_tick: 0,
        }
    }

    fn advance(&mut self, max_x: u16) {
        let min_x = 1.0;
        let max_x = max_x as f32;

        if max_x > min_x {
            self.x += self.speed * self.direction as f32;
            if self.x <= min_x {
                self.x = min_x;
                self.direction = 1;
            } else if self.x >= max_x {
                self.x = max_x;
                self.direction = -1;
            }
        } else {
            self.x = min_x;
            self.direction = 1;
        }

        self.frame_tick = self.frame_tick.saturating_add(1);
        if self.frame_tick >= FRAME_STEP {
            self.frame_index = (self.frame_index + 1) % CHICKEN_FRAMES.len();
            self.frame_tick = 0;
        }
    }
}

pub struct ChickenSystem {
    chickens: Vec<Chicken>,
    terminal_width: u16,
}

impl ChickenSystem {
    pub fn new(terminal_width: u16, _terminal_height: u16) -> Self {
        let max_x = max_x(terminal_width);
        let midpoint = max_x / 2;
        let second_x = midpoint.saturating_add(8).min(max_x).max(1);

        Self {
            chickens: vec![
                Chicken::new(1.0, 1, 0.10),
                Chicken::new(second_x as f32, -1, 0.08),
            ],
            terminal_width,
        }
    }

    fn update_chickens(&mut self) {
        let max_x = max_x(self.terminal_width);
        for chicken in &mut self.chickens {
            chicken.advance(max_x);
        }
    }

    fn render_chickens(&self, renderer: &mut TerminalRenderer, ground_y: u16) -> io::Result<()> {
        let y = ground_y.saturating_sub(SPRITE_HEIGHT - 1);
        for chicken in &self.chickens {
            render_sprite(
                renderer,
                &CHICKEN_FRAMES[chicken.frame_index],
                chicken.x.max(0.0) as u16,
                y,
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
        let max_x = max_x(size.width) as f32;
        for chicken in &mut self.chickens {
            chicken.x = chicken.x.clamp(1.0, max_x.max(1.0));
        }
    }

    fn update(
        &mut self,
        _ctx: &FrameContext<'_>,
        _rng: &mut dyn Rng,
        _commands: &mut FrameCommands,
    ) {
        self.update_chickens();
    }

    fn render(
        &mut self,
        renderer: &mut TerminalRenderer,
        ctx: &FrameContext<'_>,
    ) -> io::Result<()> {
        self.render_chickens(renderer, ctx.horizon_y)
    }
}

fn max_x(width: u16) -> u16 {
    width.saturating_sub(SPRITE_WIDTH + 1)
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
        'o' => Color::DarkGrey,
        _ => Color::White,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chicken_frames_have_fixed_dimensions() {
        for frame in CHICKEN_FRAMES {
            for line in frame {
                assert_eq!(line.chars().count(), SPRITE_WIDTH as usize);
            }
        }
    }

    #[test]
    fn chicken_turns_around_at_the_lawn_edges() {
        let mut chicken = Chicken::new(1.1, -1, 0.2);
        chicken.advance(20);
        assert_eq!(chicken.direction, 1);
        assert_eq!(chicken.x, 1.0);

        chicken.x = 19.9;
        chicken.direction = 1;
        chicken.advance(20);
        assert_eq!(chicken.direction, -1);
        assert_eq!(chicken.x, 20.0);
    }

    #[test]
    fn chicken_mirror_changes_facing_symbols() {
        assert_eq!(mirror_char('>'), '<');
        assert_eq!(mirror_char('/'), '\\');
        assert_eq!(mirror_char('o'), 'o');
    }
}
