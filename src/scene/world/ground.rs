use crate::render::TerminalRenderer;
use crate::scene::world::style::WorldSceneStyle;
use crate::scene::{POND_HEIGHT, POND_WIDTH, PondBounds};
use std::io;

pub struct Ground;

const POND_ART: [&str; POND_HEIGHT as usize] = ["   .~~~~~.   ", "  ( ~~~~~ )  ", "   `-----'   "];

impl Ground {
    pub fn render(
        &self,
        renderer: &mut TerminalRenderer,
        width: u16,
        height: u16,
        y_start: u16,
        style: &WorldSceneStyle,
    ) -> io::Result<()> {
        let width = width as usize;
        let height = height as usize;

        for y in 0..height {
            for x in 0..width {
                let r = pseudo_rand(x, y);
                let (ch, color) = if y == 0 {
                    if r < 5 {
                        (
                            '*',
                            style.flower_colors[(x + y) % style.flower_colors.len()],
                        )
                    } else if r < 15 {
                        (',', style.grass_secondary)
                    } else {
                        ('^', style.grass_primary)
                    }
                } else {
                    let ch = if r < 20 {
                        '~'
                    } else if r < 25 {
                        '.'
                    } else {
                        ' '
                    };
                    (ch, style.soil)
                };

                renderer.render_char(x as u16, y_start + y as u16, ch, color)?;
            }
        }

        Ok(())
    }

    pub fn render_pond(
        &self,
        renderer: &mut TerminalRenderer,
        pond: Option<PondBounds>,
        style: &WorldSceneStyle,
    ) -> io::Result<()> {
        let Some(pond) = pond else {
            return Ok(());
        };
        debug_assert_eq!(pond.width, POND_WIDTH);
        debug_assert_eq!(pond.height, POND_HEIGHT);

        for (row, line) in POND_ART.iter().enumerate() {
            for (column, character) in line.chars().enumerate() {
                if character == ' ' {
                    continue;
                }
                let color = if character == '~' {
                    style.water
                } else {
                    style.water_edge
                };
                renderer.render_char(
                    pond.x.saturating_add(column as u16),
                    pond.y.saturating_add(row as u16),
                    character,
                    color,
                )?;
            }
        }

        Ok(())
    }
}

fn pseudo_rand(x: usize, y: usize) -> u32 {
    ((x as u32 ^ 0x5DEECE6).wrapping_mul(y as u32 ^ 0xB)) % 100
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pond_art_has_fixed_dimensions() {
        assert_eq!(POND_ART.len(), POND_HEIGHT as usize);
        for line in POND_ART {
            assert_eq!(line.chars().count(), POND_WIDTH as usize);
        }
    }
}
