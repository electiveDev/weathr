use crossterm::style::Color;
use std::env;
use std::io::IsTerminal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSupport {
    None,
    Basic,
    Ansi256,
    TrueColor,
}

#[derive(Debug, Clone)]
pub struct TerminalCapabilities {
    pub color_support: ColorSupport,
    #[allow(dead_code)]
    pub is_tty: bool,
    pub unicode: bool,
}

impl TerminalCapabilities {
    pub fn detect() -> Self {
        let is_tty = std::io::stdout().is_terminal();

        if env::var("NO_COLOR").is_ok() {
            return Self {
                color_support: ColorSupport::None,
                is_tty,
                unicode: false,
            };
        }

        if env::var("TERM").is_ok_and(|term| term == "dumb") {
            return Self {
                color_support: ColorSupport::None,
                is_tty,
                unicode: false,
            };
        }

        if !is_tty {
            return Self {
                color_support: ColorSupport::None,
                is_tty,
                unicode: false,
            };
        }

        let color_support = if let Ok(colorterm) = env::var("COLORTERM") {
            if colorterm == "truecolor" || colorterm == "24bit" {
                ColorSupport::TrueColor
            } else {
                check_term_for_256()
            }
        } else {
            check_term_for_256()
        };

        Self {
            color_support,
            is_tty,
            unicode: supports_unicode(),
        }
    }

    pub fn supports_unicode(&self) -> bool {
        self.unicode
    }

    pub fn adjust_color(&self, color: Color) -> Color {
        if self.color_support == ColorSupport::None {
            return Color::Reset;
        }

        match self.color_support {
            ColorSupport::None => Color::Reset,
            ColorSupport::Basic => match color {
                Color::Rgb { .. } => Color::White,
                _ => color,
            },
            ColorSupport::Ansi256 => color,
            ColorSupport::TrueColor => color,
        }
    }
}

fn supports_unicode() -> bool {
    if env::var("WEATHR_ASCII").is_ok() {
        return false;
    }

    let locale_is_utf8 = ["LC_ALL", "LC_CTYPE", "LANG"]
        .into_iter()
        .filter_map(|name| env::var(name).ok())
        .any(|value| {
            let value = value.to_ascii_lowercase();
            value.contains("utf-8") || value.contains("utf8")
        });

    if cfg!(windows) {
        env::var("WT_SESSION").is_ok() || env::var("TERM_PROGRAM").is_ok() || locale_is_utf8
    } else {
        locale_is_utf8 || env::var("TERM").is_ok_and(|term| term != "dumb")
    }
}

fn check_term_for_256() -> ColorSupport {
    if env::var("TERM").is_ok_and(|term| term.contains("256color")) {
        return ColorSupport::Ansi256;
    }
    ColorSupport::Basic
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::style::Color;

    #[test]
    fn test_adjust_color_none() {
        let caps = TerminalCapabilities {
            color_support: ColorSupport::None,
            is_tty: true,
            unicode: false,
        };
        assert_eq!(caps.adjust_color(Color::Red), Color::Reset);
        assert_eq!(
            caps.adjust_color(Color::Rgb { r: 255, g: 0, b: 0 }),
            Color::Reset
        );
    }

    #[test]
    fn test_adjust_color_basic() {
        let caps = TerminalCapabilities {
            color_support: ColorSupport::Basic,
            is_tty: true,
            unicode: false,
        };
        assert_eq!(caps.adjust_color(Color::Red), Color::Red);
        assert_eq!(
            caps.adjust_color(Color::Rgb { r: 255, g: 0, b: 0 }),
            Color::White
        );
    }

    #[test]
    fn test_adjust_color_ansi256() {
        let caps = TerminalCapabilities {
            color_support: ColorSupport::Ansi256,
            is_tty: true,
            unicode: true,
        };
        assert_eq!(caps.adjust_color(Color::Red), Color::Red);
        let rgb = Color::Rgb { r: 255, g: 0, b: 0 };
        assert_eq!(caps.adjust_color(rgb), rgb);
    }

    #[test]
    fn test_adjust_color_truecolor() {
        let caps = TerminalCapabilities {
            color_support: ColorSupport::TrueColor,
            is_tty: true,
            unicode: true,
        };
        assert_eq!(caps.adjust_color(Color::Red), Color::Red);
        let rgb = Color::Rgb { r: 255, g: 0, b: 0 };
        assert_eq!(caps.adjust_color(rgb), rgb);
    }
}
