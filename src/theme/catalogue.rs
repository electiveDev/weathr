use crate::theme::{Palette, Theme, ThemeRegistry};
use crossterm::style::Color;

pub const DEFAULT_PALETTE: Palette = Palette {
    sky_day: Color::Rgb {
        r: 108,
        g: 170,
        b: 190,
    },
    sky_night: Color::Rgb {
        r: 22,
        g: 38,
        b: 72,
    },
    ground_day: Color::Rgb {
        r: 45,
        g: 120,
        b: 70,
    },
    ground_night: Color::Rgb {
        r: 20,
        g: 55,
        b: 38,
    },
    accent_primary: Color::Rgb {
        r: 178,
        g: 54,
        b: 47,
    },
    accent_secondary: Color::Rgb {
        r: 191,
        g: 154,
        b: 103,
    },
    atmosphere: Some(Color::Rgb {
        r: 118,
        g: 143,
        b: 148,
    }),
    text_primary: Color::Rgb {
        r: 225,
        g: 232,
        b: 223,
    },
    text_muted: Color::Rgb {
        r: 135,
        g: 154,
        b: 151,
    },
    border: Color::Rgb {
        r: 75,
        g: 125,
        b: 132,
    },
    surface: Color::Rgb {
        r: 21,
        g: 37,
        b: 45,
    },
    temperature: Color::Rgb {
        r: 241,
        g: 198,
        b: 120,
    },
    condition: Color::Rgb {
        r: 128,
        g: 200,
        b: 186,
    },
    wind: Color::Rgb {
        r: 144,
        g: 178,
        b: 210,
    },
    precipitation: Color::Rgb {
        r: 123,
        g: 182,
        b: 222,
    },
    cloud: Color::Rgb {
        r: 150,
        g: 164,
        b: 172,
    },
    sun: Color::Rgb {
        r: 240,
        g: 202,
        b: 111,
    },
    moon: Color::Rgb {
        r: 205,
        g: 219,
        b: 229,
    },
    rain: Color::Rgb {
        r: 114,
        g: 170,
        b: 212,
    },
    snow: Color::Rgb {
        r: 215,
        g: 229,
        b: 235,
    },
    thunderstorm: Color::Rgb {
        r: 225,
        g: 198,
        b: 103,
    },
    fog: Color::Rgb {
        r: 147,
        g: 158,
        b: 160,
    },
    vegetation: Color::Rgb {
        r: 60,
        g: 144,
        b: 76,
    },
    soil: Color::Rgb {
        r: 101,
        g: 76,
        b: 53,
    },
    tree: Color::Rgb {
        r: 47,
        g: 122,
        b: 63,
    },
    blossom: Color::Rgb {
        r: 235,
        g: 166,
        b: 185,
    },
    fruit: Color::Rgb {
        r: 205,
        g: 86,
        b: 44,
    },
    smoke: Color::Rgb {
        r: 174,
        g: 184,
        b: 181,
    },
};

fn default_theme() -> Theme {
    Theme {
        id: "default",
        display_name: "Default",
        scene_id: "world",
        overlay_id: None,
        palette: DEFAULT_PALETTE,
    }
}

pub fn register_all(registry: &mut ThemeRegistry) {
    registry.register(default_theme());
}
