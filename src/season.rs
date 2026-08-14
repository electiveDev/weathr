use chrono::{Datelike, Local};
use clap::ValueEnum;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
#[value(rename_all = "lower")]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    pub fn from_month(month: u32) -> Self {
        match month {
            3..=5 => Self::Spring,
            6..=8 => Self::Summer,
            9..=11 => Self::Autumn,
            _ => Self::Winter,
        }
    }

    pub fn local() -> Self {
        Self::from_month(Local::now().month())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_calendar_months_to_seasons() {
        assert_eq!(Season::from_month(1), Season::Winter);
        assert_eq!(Season::from_month(2), Season::Winter);
        assert_eq!(Season::from_month(3), Season::Spring);
        assert_eq!(Season::from_month(5), Season::Spring);
        assert_eq!(Season::from_month(6), Season::Summer);
        assert_eq!(Season::from_month(8), Season::Summer);
        assert_eq!(Season::from_month(9), Season::Autumn);
        assert_eq!(Season::from_month(11), Season::Autumn);
        assert_eq!(Season::from_month(12), Season::Winter);
    }
}
