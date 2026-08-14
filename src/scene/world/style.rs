use crate::scene::SceneContext;
use crate::season::Season;
use crossterm::style::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeAppearance {
    Full,
    Blossoming,
    Autumn,
    Bare,
}
#[derive(Clone, Copy)]
pub struct WorldSceneStyle {
    pub roof: Color,
    pub wood: Color,
    pub door: Color,
    pub window: Color,
    pub trim: Color,
    pub grass_primary: Color,
    pub grass_secondary: Color,
    pub flower_colors: [Color; 4],
    pub flower_chance: u32,
    pub soil: Color,
    pub tree_foliage: Color,
    pub tree_appearance: TreeAppearance,
    pub fence: Color,
    pub mailbox: Color,
    pub water: Color,
    pub water_edge: Color,
}

impl WorldSceneStyle {
    pub fn resolve(ctx: &SceneContext<'_>) -> Self {
        let palette = ctx.palette;
        let mut style = if ctx.conditions.sun.is_day {
            Self {
                roof: palette.accent_primary,
                wood: palette.accent_secondary,
                door: Color::Rgb {
                    r: 139,
                    g: 69,
                    b: 19,
                },
                window: Color::Cyan,
                trim: Color::DarkGrey,
                grass_primary: palette.ground_day,
                grass_secondary: Color::DarkGreen,
                flower_colors: [Color::Magenta, Color::Red, Color::Cyan, Color::Yellow],
                flower_chance: 5,
                soil: Color::Rgb {
                    r: 101,
                    g: 67,
                    b: 33,
                },
                tree_foliage: Color::DarkGreen,
                tree_appearance: TreeAppearance::Full,
                fence: Color::White,
                mailbox: Color::Blue,
                water: Color::Blue,
                water_edge: Color::Cyan,
            }
        } else {
            Self {
                roof: Color::DarkMagenta,
                wood: Color::Rgb {
                    r: 100,
                    g: 70,
                    b: 50,
                },
                door: Color::Rgb {
                    r: 80,
                    g: 40,
                    b: 10,
                },
                window: Color::Yellow,
                trim: Color::DarkGrey,
                grass_primary: palette.ground_night,
                grass_secondary: Color::Rgb { r: 0, g: 50, b: 0 },
                flower_colors: [
                    Color::DarkMagenta,
                    Color::DarkRed,
                    Color::Blue,
                    Color::DarkYellow,
                ],
                flower_chance: 5,
                soil: Color::Rgb {
                    r: 60,
                    g: 40,
                    b: 20,
                },
                tree_foliage: Color::Rgb { r: 0, g: 50, b: 0 },
                tree_appearance: TreeAppearance::Full,
                fence: Color::Grey,
                mailbox: Color::DarkBlue,
                water: Color::DarkBlue,
                water_edge: Color::Blue,
            }
        };

        style.apply_season(ctx.season, ctx.conditions.sun.is_day);
        style
    }

    fn apply_season(&mut self, season: Season, is_day: bool) {
        match season {
            Season::Spring => {
                if is_day {
                    self.grass_primary = Color::Rgb {
                        r: 100,
                        g: 190,
                        b: 75,
                    };
                    self.grass_secondary = Color::Rgb {
                        r: 45,
                        g: 130,
                        b: 55,
                    };
                    self.flower_colors = [Color::Magenta, Color::Red, Color::Yellow, Color::White];
                    self.tree_foliage = Color::Green;
                } else {
                    self.grass_primary = Color::DarkGreen;
                    self.grass_secondary = Color::Rgb {
                        r: 20,
                        g: 70,
                        b: 25,
                    };
                    self.flower_colors = [
                        Color::DarkMagenta,
                        Color::DarkRed,
                        Color::DarkYellow,
                        Color::Blue,
                    ];
                    self.tree_foliage = Color::Rgb { r: 0, g: 50, b: 0 };
                }
                self.flower_chance = 18;
                self.tree_appearance = TreeAppearance::Blossoming;
            }
            Season::Summer => {
                if is_day {
                    self.grass_primary = Color::Green;
                    self.grass_secondary = Color::DarkGreen;
                    self.tree_foliage = Color::Green;
                } else {
                    self.grass_primary = Color::DarkGreen;
                    self.grass_secondary = Color::Rgb { r: 0, g: 40, b: 0 };
                    self.tree_foliage = Color::Rgb { r: 0, g: 50, b: 0 };
                }
                self.flower_chance = 3;
                self.tree_appearance = TreeAppearance::Full;
            }
            Season::Autumn => {
                if is_day {
                    self.grass_primary = Color::DarkYellow;
                    self.grass_secondary = Color::Rgb {
                        r: 139,
                        g: 69,
                        b: 19,
                    };
                    self.flower_colors = [
                        Color::DarkRed,
                        Color::Rgb {
                            r: 128,
                            g: 45,
                            b: 15,
                        },
                        Color::DarkYellow,
                        Color::Rgb {
                            r: 139,
                            g: 69,
                            b: 19,
                        },
                    ];
                    self.tree_foliage = Color::DarkYellow;
                } else {
                    self.grass_primary = Color::Rgb {
                        r: 80,
                        g: 55,
                        b: 15,
                    };
                    self.grass_secondary = Color::Rgb {
                        r: 80,
                        g: 40,
                        b: 10,
                    };
                    self.flower_colors = [
                        Color::DarkRed,
                        Color::DarkYellow,
                        Color::DarkMagenta,
                        Color::Rgb {
                            r: 80,
                            g: 40,
                            b: 10,
                        },
                    ];
                    self.tree_foliage = Color::Rgb {
                        r: 80,
                        g: 50,
                        b: 15,
                    };
                }
                self.flower_chance = 2;
                self.tree_appearance = TreeAppearance::Autumn;
            }
            Season::Winter => {
                self.flower_chance = 0;
                self.tree_foliage = if is_day {
                    Color::Rgb {
                        r: 100,
                        g: 70,
                        b: 45,
                    }
                } else {
                    Color::Rgb {
                        r: 55,
                        g: 40,
                        b: 30,
                    }
                };
                self.tree_appearance = TreeAppearance::Bare;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::SceneContext;
    use crate::theme::catalogue::DEFAULT_PALETTE;
    use crate::weather::WeatherConditions;

    fn style_for(season: Season) -> WorldSceneStyle {
        let conditions = WeatherConditions::default();
        let context = SceneContext {
            conditions: &conditions,
            palette: &DEFAULT_PALETTE,
            season,
        };
        WorldSceneStyle::resolve(&context)
    }
    fn style_for_time(season: Season, is_day: bool) -> WorldSceneStyle {
        let mut conditions = WeatherConditions::default();
        conditions.sun = crate::weather::types::CelestialEvents::from_bool(is_day);
        let context = SceneContext {
            conditions: &conditions,
            palette: &DEFAULT_PALETTE,
            season,
        };
        WorldSceneStyle::resolve(&context)
    }

    #[test]
    fn seasonal_styles_select_expected_vegetation() {
        let spring = style_for(Season::Spring);
        assert_eq!(spring.tree_appearance, TreeAppearance::Blossoming);
        assert_eq!(spring.flower_chance, 18);

        let summer = style_for(Season::Summer);
        assert_eq!(summer.tree_appearance, TreeAppearance::Full);
        assert_eq!(summer.flower_chance, 3);

        let autumn = style_for(Season::Autumn);
        assert_eq!(autumn.tree_appearance, TreeAppearance::Autumn);
        assert_eq!(autumn.flower_chance, 2);
        assert!(autumn.flower_chance < summer.flower_chance);
        assert_eq!(
            autumn.flower_colors,
            [
                Color::DarkRed,
                Color::Rgb {
                    r: 128,
                    g: 45,
                    b: 15,
                },
                Color::DarkYellow,
                Color::Rgb {
                    r: 139,
                    g: 69,
                    b: 19,
                },
            ]
        );

        let winter = style_for(Season::Winter);
        assert_eq!(winter.tree_appearance, TreeAppearance::Bare);
        assert_eq!(winter.flower_chance, 0);
        assert_eq!(winter.grass_primary, DEFAULT_PALETTE.ground_day);
        let spring_night = style_for_time(Season::Spring, false);
        assert_eq!(spring_night.grass_primary, Color::DarkGreen);
        assert_eq!(spring_night.tree_foliage, Color::Rgb { r: 0, g: 50, b: 0 });
    }
}
