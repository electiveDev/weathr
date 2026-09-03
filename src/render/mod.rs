mod capabilities;

use crate::error::TerminalError;
use crate::ui::Rect;
use capabilities::TerminalCapabilities;
use crossterm::{
    cursor, execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, BufWriter, IsTerminal, Stdout, Write};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const MIN_TERMINAL_WIDTH: u16 = 70;
const MIN_TERMINAL_HEIGHT: u16 = 20;

const MAX_TERMINAL_WIDTH: u16 = 1000;
const MAX_TERMINAL_HEIGHT: u16 = 500;

fn clamp_terminal_size(width: u16, height: u16) -> (u16, u16) {
    (
        width.min(MAX_TERMINAL_WIDTH),
        height.min(MAX_TERMINAL_HEIGHT),
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Cell {
    character: char,
    color: Color,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            character: ' ',
            color: Color::Reset,
        }
    }
}

pub struct TerminalRenderer {
    stdout: BufWriter<Stdout>,
    width: u16,
    height: u16,
    buffer: Vec<Cell>,
    last_buffer: Vec<Cell>,
    capabilities: TerminalCapabilities,
    viewport: Option<Rect>,
}

impl TerminalRenderer {
    pub fn new() -> Result<Self, TerminalError> {
        if !io::stdout().is_terminal() {
            return Err(TerminalError::NotATty);
        }

        let (width, height) = terminal::size().map_err(TerminalError::SizeError)?;

        if width < MIN_TERMINAL_WIDTH || height < MIN_TERMINAL_HEIGHT {
            return Err(TerminalError::TooSmall {
                width,
                height,
                min_width: MIN_TERMINAL_WIDTH,
                min_height: MIN_TERMINAL_HEIGHT,
            });
        }

        let (width, height) = clamp_terminal_size(width, height);

        let stdout = BufWriter::new(io::stdout());
        let buffer_size = (width as usize) * (height as usize);
        let capabilities = TerminalCapabilities::detect();

        Ok(Self {
            stdout,
            width,
            height,
            buffer: vec![Cell::default(); buffer_size],
            last_buffer: vec![Cell::default(); buffer_size],
            capabilities,
            viewport: None,
        })
    }

    pub fn init(&mut self) -> Result<(), TerminalError> {
        terminal::enable_raw_mode().map_err(TerminalError::RawModeError)?;
        execute!(self.stdout, EnterAlternateScreen, cursor::Hide)
            .map_err(TerminalError::InitError)?;
        Ok(())
    }

    pub fn cleanup(&mut self) -> io::Result<()> {
        execute!(self.stdout, LeaveAlternateScreen, cursor::Show, ResetColor)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    pub fn manual_resize(&mut self, width: u16, height: u16) -> io::Result<()> {
        let (width, height) = clamp_terminal_size(width, height);
        if width != self.width || height != self.height {
            self.width = width;
            self.height = height;
            self.viewport = None;
            let buffer_size = (width as usize) * (height as usize);
            self.buffer = vec![Cell::default(); buffer_size];
            self.last_buffer = vec![Cell::default(); buffer_size];
            execute!(self.stdout, Clear(ClearType::All))?;
        }
        Ok(())
    }

    pub fn get_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    pub fn supports_unicode(&self) -> bool {
        self.capabilities.supports_unicode()
    }

    pub fn set_viewport(&mut self, viewport: Option<Rect>) {
        self.viewport = viewport;
    }

    pub fn clear_viewport(&mut self) {
        self.viewport = None;
    }

    pub fn fill_viewport(&mut self, color: Color) -> io::Result<()> {
        let viewport = self
            .viewport
            .unwrap_or(Rect::new(0, 0, self.width, self.height));
        for y in 0..viewport.height {
            for x in 0..viewport.width {
                self.render_char(x, y, ' ', color)?;
            }
        }
        Ok(())
    }

    pub fn clear(&mut self) -> io::Result<()> {
        self.buffer.fill(Cell::default());
        Ok(())
    }

    pub fn render_centered_colored(
        &mut self,
        lines: &[String],
        start_row: u16,
        color: Color,
    ) -> io::Result<()> {
        let viewport = self
            .viewport
            .unwrap_or(Rect::new(0, 0, self.width, self.height));
        let max_width = lines.iter().map(|l| l.width()).max().unwrap_or(0);
        let start_col = if viewport.width as usize > max_width {
            (viewport.width as usize - max_width) / 2
        } else {
            0
        };

        for (idx, line) in lines.iter().enumerate() {
            let row = start_row.saturating_add(idx as u16);
            let mut col = start_col as u16;
            for ch in line.chars() {
                self.render_char(col, row, ch, color)?;
                col = col.saturating_add(ch.width().unwrap_or(0) as u16);
            }
        }

        Ok(())
    }

    pub fn render_line_colored(
        &mut self,
        x: u16,
        y: u16,
        text: &str,
        color: Color,
    ) -> io::Result<()> {
        let mut col = x;
        for ch in text.chars() {
            self.render_char(col, y, ch, color)?;
            col = col.saturating_add(ch.width().unwrap_or(0) as u16);
        }
        Ok(())
    }

    pub fn render_char(&mut self, x: u16, y: u16, ch: char, color: Color) -> io::Result<()> {
        let Some((absolute_x, absolute_y)) = self.viewport_position(x, y) else {
            return Ok(());
        };
        if absolute_x < self.width && absolute_y < self.height {
            let buffer_idx = (absolute_y as usize) * (self.width as usize) + (absolute_x as usize);
            if buffer_idx < self.buffer.len() {
                self.buffer[buffer_idx] = Cell {
                    character: ch,
                    color: self.capabilities.adjust_color(color),
                };
            }
        }
        Ok(())
    }

    pub fn flash_screen(&mut self) -> io::Result<()> {
        let flash_color = self.capabilities.adjust_color(Color::White);
        if let Some(viewport) = self.viewport {
            let max_x = viewport.right().min(self.width);
            let max_y = viewport.bottom().min(self.height);
            for y in viewport.y..max_y {
                for x in viewport.x..max_x {
                    let index = y as usize * self.width as usize + x as usize;
                    if let Some(cell) = self.buffer.get_mut(index) {
                        cell.color = flash_color;
                    }
                }
            }
        } else {
            for cell in &mut self.buffer {
                cell.color = flash_color;
            }
        }
        Ok(())
    }

    fn viewport_position(&self, x: u16, y: u16) -> Option<(u16, u16)> {
        let (absolute_x, absolute_y) = if let Some(viewport) = self.viewport {
            (viewport.x.saturating_add(x), viewport.y.saturating_add(y))
        } else {
            (x, y)
        };

        if let Some(viewport) = self.viewport {
            viewport
                .contains(absolute_x, absolute_y)
                .then_some((absolute_x, absolute_y))
        } else {
            Some((absolute_x, absolute_y))
        }
    }

    pub fn flush(&mut self) -> io::Result<()> {
        let mut current_color = Color::Reset;
        let mut last_pos: Option<(u16, u16)> = None;

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y as usize) * (self.width as usize) + (x as usize);

                if idx >= self.buffer.len() || idx >= self.last_buffer.len() {
                    continue;
                }

                let cell = self.buffer[idx];
                let last_cell = self.last_buffer[idx];

                if cell != last_cell {
                    let expected_pos = last_pos.map(|(lx, ly)| (lx + 1, ly));
                    if expected_pos != Some((x, y)) {
                        queue!(self.stdout, cursor::MoveTo(x, y))?;
                    }

                    if cell.color != current_color {
                        queue!(self.stdout, SetForegroundColor(cell.color))?;
                        current_color = cell.color;
                    }

                    queue!(self.stdout, Print(cell.character))?;
                    last_pos = Some((x, y));
                }
            }
        }

        if current_color != Color::Reset {
            queue!(self.stdout, ResetColor)?;
        }

        self.stdout.flush()?;
        self.last_buffer.copy_from_slice(&self.buffer);
        Ok(())
    }
}

impl Drop for TerminalRenderer {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
