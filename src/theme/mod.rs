pub mod catalogue;

use std::collections::HashMap;
use std::fmt;

use crate::season::Season;
use crate::weather::WeatherConditions;
use crossterm::style::Color;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub sky_day: Color,
    pub sky_night: Color,
    pub ground_day: Color,
    pub ground_night: Color,
    pub accent_primary: Color,
    pub accent_secondary: Color,
    pub atmosphere: Option<Color>,
    pub text_primary: Color,
    pub text_muted: Color,
    pub border: Color,
    pub surface: Color,
    pub temperature: Color,
    pub condition: Color,
    pub wind: Color,
    pub precipitation: Color,
    pub cloud: Color,
    pub sun: Color,
    pub moon: Color,
    pub rain: Color,
    pub snow: Color,
    pub thunderstorm: Color,
    pub fog: Color,
    pub vegetation: Color,
    pub soil: Color,
    pub tree: Color,
    pub blossom: Color,
    pub fruit: Color,
    pub smoke: Color,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VisualPalette {
    pub sky: Color,
    pub ground: Color,
    pub text_primary: Color,
    pub text_muted: Color,
    pub border: Color,
    pub surface: Color,
    pub temperature: Color,
    pub condition: Color,
    pub wind: Color,
    pub precipitation: Color,
    pub cloud: Color,
    pub sun: Color,
    pub moon: Color,
    pub rain: Color,
    pub snow: Color,
    pub thunderstorm: Color,
    pub fog: Color,
    pub vegetation: Color,
    pub soil: Color,
    pub tree: Color,
    pub blossom: Color,
    pub fruit: Color,
    pub smoke: Color,
}

impl VisualPalette {
    pub fn resolve(palette: &Palette, conditions: &WeatherConditions, season: Season) -> Self {
        let is_day = conditions.sun.is_day;
        let mut visual = Self {
            sky: if is_day {
                palette.sky_day
            } else {
                palette.sky_night
            },
            ground: if is_day {
                palette.ground_day
            } else {
                palette.ground_night
            },
            text_primary: palette.text_primary,
            text_muted: palette.text_muted,
            border: palette.border,
            surface: palette.surface,
            temperature: palette.temperature,
            condition: palette.condition,
            wind: palette.wind,
            precipitation: palette.precipitation,
            cloud: palette.cloud,
            sun: palette.sun,
            moon: palette.moon,
            rain: palette.rain,
            snow: palette.snow,
            thunderstorm: palette.thunderstorm,
            fog: palette.fog,
            vegetation: palette.vegetation,
            soil: palette.soil,
            tree: palette.tree,
            blossom: palette.blossom,
            fruit: palette.fruit,
            smoke: palette.smoke,
        };

        if !is_day {
            visual.text_primary = Color::Rgb {
                r: 190,
                g: 205,
                b: 210,
            };
            visual.vegetation = Color::Rgb {
                r: 36,
                g: 78,
                b: 58,
            };
            visual.tree = Color::Rgb {
                r: 28,
                g: 62,
                b: 52,
            };
            visual.soil = Color::Rgb {
                r: 42,
                g: 48,
                b: 48,
            };
        }

        match season {
            Season::Spring => {
                visual.vegetation = if is_day {
                    Color::Rgb {
                        r: 92,
                        g: 174,
                        b: 89,
                    }
                } else {
                    Color::Rgb {
                        r: 35,
                        g: 86,
                        b: 54,
                    }
                };
                visual.tree = if is_day {
                    Color::Rgb {
                        r: 52,
                        g: 132,
                        b: 74,
                    }
                } else {
                    Color::Rgb {
                        r: 24,
                        g: 67,
                        b: 44,
                    }
                };
                visual.blossom = if is_day {
                    Color::Rgb {
                        r: 235,
                        g: 166,
                        b: 185,
                    }
                } else {
                    Color::Rgb {
                        r: 145,
                        g: 92,
                        b: 126,
                    }
                };
            }
            Season::Summer => {
                visual.vegetation = if is_day {
                    Color::Rgb {
                        r: 46,
                        g: 148,
                        b: 73,
                    }
                } else {
                    Color::Rgb {
                        r: 28,
                        g: 82,
                        b: 48,
                    }
                };
                visual.tree = if is_day {
                    Color::Rgb {
                        r: 38,
                        g: 118,
                        b: 59,
                    }
                } else {
                    Color::Rgb {
                        r: 24,
                        g: 70,
                        b: 42,
                    }
                };
            }
            Season::Autumn => {
                visual.vegetation = if is_day {
                    Color::Rgb {
                        r: 164,
                        g: 134,
                        b: 55,
                    }
                } else {
                    Color::Rgb {
                        r: 82,
                        g: 67,
                        b: 30,
                    }
                };
                visual.tree = if is_day {
                    Color::Rgb {
                        r: 176,
                        g: 112,
                        b: 42,
                    }
                } else {
                    Color::Rgb {
                        r: 92,
                        g: 58,
                        b: 28,
                    }
                };
                visual.fruit = Color::Rgb {
                    r: 194,
                    g: 82,
                    b: 39,
                };
            }
            Season::Winter => {
                visual.vegetation = if is_day {
                    Color::Rgb {
                        r: 145,
                        g: 158,
                        b: 142,
                    }
                } else {
                    Color::Rgb {
                        r: 58,
                        g: 70,
                        b: 68,
                    }
                };
                visual.tree = if is_day {
                    Color::Rgb {
                        r: 102,
                        g: 83,
                        b: 63,
                    }
                } else {
                    Color::Rgb {
                        r: 66,
                        g: 53,
                        b: 46,
                    }
                };
                visual.soil = if is_day {
                    Color::Rgb {
                        r: 208,
                        g: 216,
                        b: 215,
                    }
                } else {
                    Color::Rgb {
                        r: 83,
                        g: 94,
                        b: 96,
                    }
                };
            }
        }

        if conditions.is_foggy {
            visual.sky = Color::Rgb {
                r: 108,
                g: 125,
                b: 130,
            };
            visual.cloud = palette.fog;
        } else if conditions.is_thunderstorm {
            visual.sky = if is_day {
                Color::Rgb {
                    r: 62,
                    g: 78,
                    b: 102,
                }
            } else {
                Color::Rgb {
                    r: 18,
                    g: 24,
                    b: 45,
                }
            };
            visual.cloud = palette.thunderstorm;
        } else if conditions.is_raining {
            visual.sky = if is_day {
                Color::Rgb {
                    r: 91,
                    g: 124,
                    b: 145,
                }
            } else {
                Color::Rgb {
                    r: 24,
                    g: 45,
                    b: 66,
                }
            };
        } else if conditions.is_snowing {
            visual.sky = if is_day {
                Color::Rgb {
                    r: 166,
                    g: 186,
                    b: 198,
                }
            } else {
                Color::Rgb {
                    r: 32,
                    g: 52,
                    b: 72,
                }
            };
        }

        visual
    }
}

impl Default for VisualPalette {
    fn default() -> Self {
        Self::resolve(
            &crate::theme::catalogue::DEFAULT_PALETTE,
            &WeatherConditions::default(),
            Season::Summer,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visual_palette_distinguishes_day_night_and_seasons() {
        let day = WeatherConditions {
            sun: crate::weather::types::CelestialEvents::from_bool(true),
            ..WeatherConditions::default()
        };
        let night = WeatherConditions {
            sun: crate::weather::types::CelestialEvents::from_bool(false),
            ..WeatherConditions::default()
        };

        let summer_day = VisualPalette::resolve(&catalogue::DEFAULT_PALETTE, &day, Season::Summer);
        let winter_day = VisualPalette::resolve(&catalogue::DEFAULT_PALETTE, &day, Season::Winter);
        let summer_night =
            VisualPalette::resolve(&catalogue::DEFAULT_PALETTE, &night, Season::Summer);

        assert_ne!(summer_day.sky, summer_night.sky);
        assert_ne!(summer_day.vegetation, winter_day.vegetation);
        assert_ne!(summer_day.tree, winter_day.tree);
    }

    #[test]
    fn weather_overlays_adjust_the_sky_without_changing_the_data_model() {
        let clear = WeatherConditions {
            sun: crate::weather::types::CelestialEvents::from_bool(true),
            ..WeatherConditions::default()
        };
        let mut storm = clear;
        storm.is_thunderstorm = true;

        let clear_palette =
            VisualPalette::resolve(&catalogue::DEFAULT_PALETTE, &clear, Season::Summer);
        let storm_palette =
            VisualPalette::resolve(&catalogue::DEFAULT_PALETTE, &storm, Season::Summer);

        assert_ne!(clear_palette.sky, storm_palette.sky);
        assert_eq!(
            storm_palette.thunderstorm,
            catalogue::DEFAULT_PALETTE.thunderstorm
        );
    }
}

pub struct Theme {
    pub id: &'static str,
    #[allow(dead_code)]
    pub display_name: &'static str,
    pub scene_id: &'static str,
    pub overlay_id: Option<&'static str>,
    pub palette: Palette,
}

#[derive(Debug)]
pub enum ThemeError {
    NotFound(String),
    #[allow(dead_code)]
    SceneNotRegistered {
        theme: &'static str,
        scene: &'static str,
    },
}

impl fmt::Display for ThemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThemeError::NotFound(id) => write!(f, "theme '{}' is not registered", id),
            ThemeError::SceneNotRegistered { theme, scene } => {
                write!(
                    f,
                    "theme '{}' references unregistered scene '{}'",
                    theme, scene
                )
            }
        }
    }
}

pub struct ThemeRegistry {
    themes: HashMap<&'static str, Theme>,
    active: &'static str,
}

impl ThemeRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            themes: HashMap::new(),
            active: "default",
        };
        catalogue::register_all(&mut registry);
        registry
    }

    pub fn register(&mut self, theme: Theme) {
        self.themes.insert(theme.id, theme);
    }

    pub fn set_active(&mut self, id: &str) -> Result<(), ThemeError> {
        match self.themes.get_key_value(id) {
            Some((&static_id, _)) => {
                self.active = static_id;
                Ok(())
            }
            None => Err(ThemeError::NotFound(id.to_owned())),
        }
    }

    pub fn active(&self) -> &Theme {
        self.themes
            .get(self.active)
            .expect("active theme id must always reference a registered theme")
    }

    pub fn get(&self, id: &str) -> Option<&Theme> {
        self.themes.get(id)
    }
}

impl Default for ThemeRegistry {
    fn default() -> Self {
        Self::new()
    }
}
