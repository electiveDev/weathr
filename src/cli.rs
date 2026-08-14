use clap::Parser;
use clap::builder::{PossibleValue, PossibleValuesParser};
use clap_complete::Shell;

use crate::weather::WeatherCondition;

const LONG_VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "\n\nWeather data provided by Open-Meteo.com (https://open-meteo.com/)\n",
    "Data licensed under CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/)\n\n",
    "Geocoding powered by Nominatim/OpenStreetMap (https://nominatim.openstreetmap.org/)\n",
    "Data \u{00a9} OpenStreetMap contributors, ODbL (https://www.openstreetmap.org/copyright)"
);

const ABOUT: &str = concat!(
    "Terminal-based ASCII weather application\n\n",
    "Weather data provided by Open-Meteo.com (https://open-meteo.com/)\n",
    "Data licensed under CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/)\n\n",
    "Geocoding powered by Nominatim/OpenStreetMap (https://nominatim.openstreetmap.org/)\n",
    "Data \u{00a9} OpenStreetMap contributors, ODbL (https://www.openstreetmap.org/copyright)"
);

fn simulate_parser() -> PossibleValuesParser {
    PossibleValuesParser::new(
        WeatherCondition::ALL
            .iter()
            .map(|c| PossibleValue::new(c.as_str()).help(c.description())),
    )
}

fn test_parser() -> PossibleValuesParser {
    PossibleValuesParser::new([PossibleValue::new("bunny").help("Run the bunny animation test")])
}

#[derive(Parser)]
#[command(version, long_version = LONG_VERSION, about = ABOUT, long_about = None)]
pub struct Cli {
    #[arg(
        short,
        long,
        value_name = "CONDITION",
        value_parser = simulate_parser(),
        help = "Simulate weather condition (clear, rain, drizzle, snow, etc.)"
    )]
    pub simulate: Option<String>,

    #[arg(
        short = 't',
        long,
        value_name = "ANIMATION",
        value_parser = test_parser(),
        conflicts_with = "simulate",
        help = "Run an offline animation test (bunny)"
    )]
    pub test: Option<String>,

    #[arg(
        short,
        long,
        help = "Simulate night time (for testing moon, stars, fireflies)"
    )]
    pub night: bool,

    #[arg(short, long, help = "Enable falling autumn leaves")]
    pub leaves: bool,

    #[arg(long, help = "Auto-detect location via IP (uses ipinfo.io)")]
    pub auto_location: bool,

    #[arg(long, help = "Hide location coordinates in UI")]
    pub hide_location: bool,

    #[arg(long, help = "Hide HUD (status line)")]
    pub hide_hud: bool,

    #[arg(
        long,
        conflicts_with = "metric",
        help = "Use imperial units (°F, mph, inch)"
    )]
    pub imperial: bool,

    #[arg(
        long,
        conflicts_with = "imperial",
        help = "Use metric units (°C, km/h, mm)"
    )]
    pub metric: bool,

    #[arg(long, help = "Run silently (suppress non-error output)")]
    pub silent: bool,

    #[arg(long, value_name = "SHELL", value_enum)]
    pub completions: Option<Shell>,
}

impl Cli {
    pub fn is_bunny_test(&self) -> bool {
        self.test.as_deref() == Some("bunny")
    }
}

pub fn extract_simulate_missing_value(err: clap::Error) -> clap::Error {
    let msg = err.to_string();
    if msg.contains("--simulate") && msg.contains("value is required") {
        err
    } else {
        err.exit()
    }
}

pub fn print_simulate_help() {
    let mut current_group = "";

    eprintln!("Available weather conditions:");
    for condition in WeatherCondition::ALL {
        let group = condition.group();
        if group != current_group {
            eprintln!();
            eprintln!("  {}:", group);
            current_group = group;
        }
        eprintln!(
            "    {:<18} - {}",
            condition.as_str(),
            condition.description()
        );
    }

    eprintln!();
    eprintln!("Examples:");
    eprintln!("  weathr --simulate rain");
    eprintln!("  weathr --simulate snow --night");
    eprintln!("  weathr -s thunderstorm -n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bunny_test_accepts_animation_name() {
        let cli = Cli::try_parse_from(["weathr", "--test", "bunny"]).unwrap();

        assert!(cli.is_bunny_test());
        assert_eq!(cli.test.as_deref(), Some("bunny"));
        assert_eq!(cli.simulate, None);
    }

    #[test]
    fn bunny_test_accepts_night_mode() {
        let cli = Cli::try_parse_from(["weathr", "-t", "bunny", "--night"]).unwrap();

        assert!(cli.is_bunny_test());
        assert!(cli.night);
    }

    #[test]
    fn bunny_test_conflicts_with_weather_simulation() {
        let result = Cli::try_parse_from(["weathr", "--test", "bunny", "--simulate", "clear"]);

        assert!(result.is_err());
    }

    #[test]
    fn bunny_test_rejects_unknown_animation() {
        let result = Cli::try_parse_from(["weathr", "--test", "cat"]);

        assert!(result.is_err());
    }
}
