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
    pub fruit_colors: [Color; 3],
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
                fruit_colors: [
                    Color::Red,
                    Color::Yellow,
                    Color::Rgb {
                        r: 210,
                        g: 60,
                        b: 35,
                    },
                ],
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
                fruit_colors: [
                    Color::DarkRed,
                    Color::DarkYellow,
                    Color::Rgb {
                        r: 130,
                        g: 45,
                        b: 25,
                    },
                ],
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
                    self.soil = Color::Rgb {
                        r: 82,
                        g: 68,
                        b: 45,
                    };
                    self.water = Color::Rgb {
                        r: 40,
                        g: 135,
                        b: 190,
                    };
                    self.water_edge = Color::Rgb {
                        r: 80,
                        g: 190,
                        b: 180,
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
                    self.soil = Color::Rgb {
                        r: 42,
                        g: 36,
                        b: 28,
                    };
                    self.water = Color::DarkBlue;
                    self.water_edge = Color::Blue;
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
                    self.soil = Color::Rgb {
                        r: 128,
                        g: 94,
                        b: 48,
                    };
                    self.water = Color::Rgb {
                        r: 30,
                        g: 135,
                        b: 205,
                    };
                    self.water_edge = Color::Cyan;
                    self.tree_foliage = Color::Green;
                    self.fruit_colors = [
                        Color::Red,
                        Color::Yellow,
                        Color::Rgb {
                            r: 210,
                            g: 60,
                            b: 35,
                        },
                    ];
                } else {
                    self.grass_primary = Color::DarkGreen;
                    self.grass_secondary = Color::Rgb { r: 0, g: 40, b: 0 };
                    self.soil = Color::Rgb {
                        r: 60,
                        g: 45,
                        b: 25,
                    };
                    self.water = Color::DarkBlue;
                    self.water_edge = Color::Blue;
                    self.tree_foliage = Color::Rgb { r: 0, g: 50, b: 0 };
                    self.fruit_colors = [
                        Color::DarkRed,
                        Color::DarkYellow,
                        Color::Rgb {
                            r: 130,
                            g: 45,
                            b: 25,
                        },
                    ];
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
                    self.soil = Color::Rgb {
                        r: 112,
                        g: 70,
                        b: 34,
                    };
                    self.water = Color::Rgb {
                        r: 35,
                        g: 115,
                        b: 165,
                    };
                    self.water_edge = Color::Rgb {
                        r: 70,
                        g: 160,
                        b: 160,
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
                    self.soil = Color::Rgb {
                        r: 62,
                        g: 40,
                        b: 22,
                    };
                    self.water = Color::DarkBlue;
                    self.water_edge = Color::Blue;
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
                if is_day {
                    self.grass_primary = Color::Rgb {
                        r: 145,
                        g: 155,
                        b: 125,
                    };
                    self.grass_secondary = Color::Rgb {
                        r: 105,
                        g: 115,
                        b: 95,
                    };
                    self.soil = Color::Rgb {
                        r: 135,
                        g: 130,
                        b: 115,
                    };
                    self.water = Color::Rgb {
                        r: 90,
                        g: 130,
                        b: 155,
                    };
                    self.water_edge = Color::Rgb {
                        r: 165,
                        g: 190,
                        b: 195,
                    };
                    self.tree_foliage = Color::Rgb {
                        r: 100,
                        g: 70,
                        b: 45,
                    };
                } else {
                    self.grass_primary = Color::Rgb {
                        r: 55,
                        g: 65,
                        b: 55,
                    };
                    self.grass_secondary = Color::Rgb {
                        r: 40,
                        g: 48,
                        b: 40,
                    };
                    self.soil = Color::Rgb {
                        r: 65,
                        g: 65,
                        b: 60,
                    };
                    self.water = Color::Rgb {
                        r: 55,
                        g: 80,
                        b: 100,
                    };
                    self.water_edge = Color::Rgb {
                        r: 100,
                        g: 125,
                        b: 135,
                    };
                    self.tree_foliage = Color::Rgb {
                        r: 55,
                        g: 40,
                        b: 30,
                    };
                }
                self.flower_chance = 0;
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
        assert_eq!(
            spring.soil,
            Color::Rgb {
                r: 82,
                g: 68,
                b: 45
            }
        );

        let summer = style_for(Season::Summer);
        assert_eq!(summer.tree_appearance, TreeAppearance::Full);
        assert_eq!(summer.flower_chance, 3);
        assert_eq!(
            summer.soil,
            Color::Rgb {
                r: 128,
                g: 94,
                b: 48
            }
        );
        assert_eq!(summer.fruit_colors[0], Color::Red);

        let autumn = style_for(Season::Autumn);
        assert_eq!(autumn.tree_appearance, TreeAppearance::Autumn);
        assert_eq!(autumn.flower_chance, 2);
        assert!(autumn.flower_chance < summer.flower_chance);
        assert_eq!(
            autumn.soil,
            Color::Rgb {
                r: 112,
                g: 70,
                b: 34
            }
        );
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
        assert_eq!(
            winter.grass_primary,
            Color::Rgb {
                r: 145,
                g: 155,
                b: 125,
            }
        );
        assert_eq!(
            winter.soil,
            Color::Rgb {
                r: 135,
                g: 130,
                b: 115,
            }
        );
        assert_eq!(
            winter.water_edge,
            Color::Rgb {
                r: 165,
                g: 190,
                b: 195,
            }
        );
    }

    #[test]
    fn seasonal_ground_and_night_styles_are_distinct() {
        let day_styles = [
            style_for(Season::Spring),
            style_for(Season::Summer),
            style_for(Season::Autumn),
            style_for(Season::Winter),
        ];
        assert_ne!(day_styles[0].soil, day_styles[1].soil);
        assert_ne!(day_styles[1].soil, day_styles[2].soil);
        assert_ne!(day_styles[2].soil, day_styles[3].soil);
        assert_ne!(day_styles[0].water_edge, day_styles[3].water_edge);

        let spring_night = style_for_time(Season::Spring, false);
        assert_eq!(spring_night.grass_primary, Color::DarkGreen);
        assert_eq!(spring_night.tree_foliage, Color::Rgb { r: 0, g: 50, b: 0 });
        assert_eq!(
            spring_night.soil,
            Color::Rgb {
                r: 42,
                g: 36,
                b: 28
            }
        );

        let winter_night = style_for_time(Season::Winter, false);
        assert_eq!(
            winter_night.water,
            Color::Rgb {
                r: 55,
                g: 80,
                b: 100,
            }
        );
        assert_ne!(spring_night.soil, winter_night.soil);
    }
}
