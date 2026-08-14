use crate::animation::{AnimationSystem, FrameCommands, FrameContext, RenderLayer, TerminalSize};
use crate::render::TerminalRenderer;
use crossterm::style::Color;
use rand::Rng;
use std::io;

const FRAME_STEP: u8 = 5;
const SPRITE_WIDTH: u16 = 9;
const SPRITE_HEIGHT: u16 = 3;
const CAT_WALK_FRAMES: [[&str; 3]; 4] = [
    [" /\\_/\\\\  ", "( o.o )__", "  / \\    "],
    [" /\\_/\\\\  ", "( o.o )_/", "   /\\    "],
    [" /\\_/\\\\  ", "( -.- )__", "  \\ /    "],
    [" /\\_/\\\\  ", "( o.o )\\_", "  /  \\   "],
];
const CAT_SLEEP_FRAMES: [[&str; 3]; 3] = [
    [" /\\_/\\\\  ", "( -.- )__", "  > ^ <  "],
    [" /\\_/\\\\  ", "( -.- )__", "   ^ ^   "],
    [" /\\_/\\\\  ", "( o.o )__", "  > ^ <  "],
];

struct Cat {
    x: f32,
    direction: i8,
    speed: f32,
    frame_index: usize,
    frame_tick: u8,
    sleeping: bool,
}

impl Cat {
    fn new(terminal_width: u16) -> Self {
        let max_x = max_x(terminal_width);
        let x = (max_x / 2).max(1) as f32;
        Self {
            x,
            direction: -1,
            speed: 0.07,
            frame_index: 0,
            frame_tick: 0,
            sleeping: false,
        }
    }

    fn advance(&mut self, terminal_width: u16, is_day: bool) {
        self.sleeping = !is_day;
        let max_x = max_x(terminal_width) as f32;

        if !self.sleeping && max_x > 1.0 {
            self.x += self.speed * self.direction as f32;
            if self.x <= 1.0 {
                self.x = 1.0;
                self.direction = 1;
            } else if self.x >= max_x {
                self.x = max_x;
                self.direction = -1;
            }
        }

        self.frame_tick = self.frame_tick.saturating_add(1);
        if self.frame_tick >= FRAME_STEP {
            let frame_count = if self.sleeping {
                CAT_SLEEP_FRAMES.len()
            } else {
                CAT_WALK_FRAMES.len()
            };
            self.frame_index = (self.frame_index + 1) % frame_count;
            self.frame_tick = 0;
        }
    }
}

pub struct CatSystem {
    cat: Cat,
    terminal_width: u16,
}

impl CatSystem {
    pub fn new(terminal_width: u16, _terminal_height: u16) -> Self {
        Self {
            cat: Cat::new(terminal_width),
            terminal_width,
        }
    }

    fn render_cat(&self, renderer: &mut TerminalRenderer, ground_y: u16) -> io::Result<()> {
        let y = ground_y.saturating_sub(SPRITE_HEIGHT - 1);
        let frames: &[[&str; 3]] = if self.cat.sleeping {
            &CAT_SLEEP_FRAMES
        } else {
            &CAT_WALK_FRAMES
        };
        let frame = &frames[self.cat.frame_index % frames.len()];
        render_sprite(
            renderer,
            frame,
            self.cat.x.max(0.0) as u16,
            y,
            self.cat.direction,
        )
    }
}

impl AnimationSystem for CatSystem {
    fn id(&self) -> &'static str {
        "cat"
    }

    fn layer(&self) -> RenderLayer {
        RenderLayer::Foreground
    }

    fn on_resize(&mut self, size: TerminalSize) {
        self.terminal_width = size.width;
        self.cat.x = self.cat.x.clamp(1.0, max_x(size.width).max(1) as f32);
    }

    fn update(
        &mut self,
        ctx: &FrameContext<'_>,
        _rng: &mut dyn Rng,
        _commands: &mut FrameCommands,
    ) {
        self.terminal_width = ctx.size.width;
        self.cat
            .advance(self.terminal_width, ctx.conditions.sun.is_day);
    }

    fn render(
        &mut self,
        renderer: &mut TerminalRenderer,
        ctx: &FrameContext<'_>,
    ) -> io::Result<()> {
        self.render_cat(renderer, ctx.horizon_y)
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
    fn cat_sleeps_at_night_and_wakes_during_day() {
        let mut cat = Cat::new(80);
        cat.advance(80, false);
        assert!(cat.sleeping);
        let sleeping_x = cat.x;

        cat.advance(80, true);
        assert!(!cat.sleeping);
        assert_ne!(cat.x, sleeping_x);
    }

    #[test]
    fn cat_mirror_preserves_non_directional_symbols() {
        assert_eq!(mirror_char('/'), '\\');
        assert_eq!(mirror_char('o'), 'o');
        assert_eq!(mirror_char('^'), '^');
    }
}
