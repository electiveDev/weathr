use crate::app_state::AppState;
use crate::render::TerminalRenderer;
use crate::theme::VisualPalette;
use crate::ui::layout::{LayoutTier, Rect, RenderRegions};
use crate::weather::{format_precipitation, format_temperature, format_wind_speed};
use std::io;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const HORIZONTAL_ASCII: char = '-';

#[derive(Clone, Copy)]
struct FrameGlyphs {
    top_left: char,
    top_right: char,
    bottom_left: char,
    bottom_right: char,
    vertical: char,
    horizontal: char,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HudModel {
    pub location: Option<String>,
    pub condition: String,
    pub temperature: String,
    pub wind: String,
    pub precipitation: String,
    pub details: Vec<String>,
    pub offline: bool,
}

impl HudModel {
    pub fn from_state(state: &AppState, attribution: &str, show_details: bool) -> Self {
        let Some(weather) = state.current_weather.as_ref() else {
            return Self {
                location: state.location_label(),
                condition: "Loading".to_string(),
                temperature: "--".to_string(),
                wind: "--".to_string(),
                precipitation: "--".to_string(),
                details: if show_details {
                    vec!["Awaiting weather data".to_string()]
                } else {
                    Vec::new()
                },
                offline: state.is_offline,
            };
        };

        let (temperature, temperature_unit) =
            format_temperature(weather.temperature, state.units.temperature);
        let (wind, wind_unit) = format_wind_speed(weather.wind_speed, state.units.wind_speed);
        let (precipitation, precipitation_unit) =
            format_precipitation(weather.precipitation, state.units.precipitation);
        let mut details = Vec::new();

        if show_details {
            details.push(format!(
                "LOC {}",
                state
                    .location_label()
                    .unwrap_or_else(|| "hidden".to_string())
            ));
            details.push(format!("TIME {}", weather.timestamp));
            details.push(format!("DIR {:.0} deg", weather.wind_direction));
            if !attribution.trim().is_empty() {
                details.push(format!("SRC {}", attribution));
            }
            if state.is_offline {
                details.push("MODE OFFLINE".to_string());
            }
        }

        Self {
            location: state.location_label(),
            condition: state.get_condition_text().to_string(),
            temperature: format!("{temperature:.1}{temperature_unit}"),
            wind: format!(
                "{wind:.1} {wind_unit} {}",
                wind_direction_label(weather.wind_direction)
            ),
            precipitation: format!("{precipitation:.1}{precipitation_unit}"),
            details,
            offline: state.is_offline,
        }
    }
}

pub fn render(
    renderer: &mut TerminalRenderer,
    state: &AppState,
    attribution: &str,
    regions: &RenderRegions,
    palette: VisualPalette,
    show_details: bool,
) -> io::Result<()> {
    let Some(rect) = regions.hud else {
        return Ok(());
    };

    let model = HudModel::from_state(state, attribution, show_details);
    renderer.set_viewport(Some(rect));
    renderer.fill_viewport(palette.surface)?;
    let unicode = renderer.supports_unicode();
    let glyphs = if unicode {
        FrameGlyphs {
            top_left: '┌',
            top_right: '┐',
            bottom_left: '└',
            bottom_right: '┘',
            vertical: '│',
            horizontal: '─',
        }
    } else {
        FrameGlyphs {
            top_left: '+',
            top_right: '+',
            bottom_left: '+',
            bottom_right: '+',
            vertical: '|',
            horizontal: HORIZONTAL_ASCII,
        }
    };

    draw_frame(renderer, rect, palette.border, glyphs)?;
    let inner = Rect::new(
        1,
        1,
        rect.width.saturating_sub(2),
        rect.height.saturating_sub(2),
    );
    let result = match regions.tier {
        LayoutTier::Large => render_large(renderer, inner, &model, palette),
        LayoutTier::Medium => render_compact(renderer, inner, &model, palette),
        LayoutTier::Small => render_small(renderer, inner, &model, palette),
    };
    renderer.clear_viewport();
    result
}

fn render_large(
    renderer: &mut TerminalRenderer,
    rect: Rect,
    model: &HudModel,
    palette: VisualPalette,
) -> io::Result<()> {
    let mut row = 0;
    write_fit(renderer, rect, 0, row, "WEATHER / NOW", palette.text_muted)?;
    row += 2;
    write_fit(
        renderer,
        rect,
        0,
        row,
        &model.temperature,
        palette.temperature,
    )?;
    row += 2;
    write_fit(
        renderer,
        rect,
        0,
        row,
        &model.condition.to_uppercase(),
        palette.condition,
    )?;
    row += 2;
    if let Some(location) = &model.location {
        write_fit(renderer, rect, 0, row, location, palette.text_primary)?;
        row += 2;
    }
    write_fit(
        renderer,
        rect,
        0,
        row,
        &format!("WIND  {}", model.wind),
        palette.wind,
    )?;
    row += 1;
    write_fit(
        renderer,
        rect,
        0,
        row,
        &format!("RAIN  {}", model.precipitation),
        palette.precipitation,
    )?;
    row += 2;
    for detail in &model.details {
        if row >= rect.height {
            break;
        }
        write_fit(renderer, rect, 0, row, detail, palette.text_muted)?;
        row += 1;
    }
    Ok(())
}

fn render_compact(
    renderer: &mut TerminalRenderer,
    rect: Rect,
    model: &HudModel,
    palette: VisualPalette,
) -> io::Result<()> {
    let first = if let Some(location) = &model.location {
        format!("{}  {}  {}", model.temperature, model.condition, location)
    } else {
        format!("{}  {}", model.temperature, model.condition)
    };
    write_fit(renderer, rect, 0, 0, &first, palette.temperature)?;
    if rect.height > 1 {
        write_fit(
            renderer,
            rect,
            0,
            1,
            &format!("WIND {}   RAIN {}", model.wind, model.precipitation),
            palette.wind,
        )?;
    }
    for (index, detail) in model.details.iter().enumerate() {
        let row = index as u16 + 2;
        if row >= rect.height {
            break;
        }
        write_fit(renderer, rect, 0, row, detail, palette.text_muted)?;
    }
    Ok(())
}

fn render_small(
    renderer: &mut TerminalRenderer,
    rect: Rect,
    model: &HudModel,
    palette: VisualPalette,
) -> io::Result<()> {
    write_fit(
        renderer,
        rect,
        0,
        0,
        &format!("{}  {}", model.temperature, model.condition),
        palette.temperature,
    )?;
    if rect.height > 1 {
        let location = model.location.as_deref().unwrap_or("location hidden");
        write_fit(
            renderer,
            rect,
            0,
            1,
            &format!("{}  W {}  R {}", location, model.wind, model.precipitation),
            palette.text_primary,
        )?;
    }
    for (index, detail) in model.details.iter().enumerate() {
        let row = index as u16 + 2;
        if row >= rect.height {
            break;
        }
        write_fit(renderer, rect, 0, row, detail, palette.text_muted)?;
    }
    Ok(())
}

fn draw_frame(
    renderer: &mut TerminalRenderer,
    rect: Rect,
    color: crossterm::style::Color,
    glyphs: FrameGlyphs,
) -> io::Result<()> {
    if rect.width < 2 || rect.height < 2 {
        return Ok(());
    }
    renderer.render_char(0, 0, glyphs.top_left, color)?;
    renderer.render_char(rect.width - 1, 0, glyphs.top_right, color)?;
    renderer.render_char(0, rect.height - 1, glyphs.bottom_left, color)?;
    renderer.render_char(rect.width - 1, rect.height - 1, glyphs.bottom_right, color)?;
    for x in 1..rect.width.saturating_sub(1) {
        renderer.render_char(x, 0, glyphs.horizontal, color)?;
        renderer.render_char(x, rect.height - 1, glyphs.horizontal, color)?;
    }
    for y in 1..rect.height.saturating_sub(1) {
        renderer.render_char(0, y, glyphs.vertical, color)?;
        renderer.render_char(rect.width - 1, y, glyphs.vertical, color)?;
    }
    Ok(())
}

fn write_fit(
    renderer: &mut TerminalRenderer,
    rect: Rect,
    x: u16,
    y: u16,
    text: &str,
    color: crossterm::style::Color,
) -> io::Result<()> {
    if y >= rect.height || x >= rect.width {
        return Ok(());
    }
    let available = rect.width.saturating_sub(x) as usize;
    let fitted = fit_text(text, available);
    renderer.render_line_colored(
        rect.x.saturating_add(x),
        rect.y.saturating_add(y),
        &fitted,
        color,
    )
}

pub fn fit_text(text: &str, max_width: usize) -> String {
    if text.width() <= max_width {
        return text.to_string();
    }
    if max_width <= 3 {
        return text
            .chars()
            .scan(0usize, |width, ch| {
                let ch_width = ch.width().unwrap_or(0);
                if *width + ch_width > max_width {
                    None
                } else {
                    *width += ch_width;
                    Some(ch)
                }
            })
            .collect();
    }
    let mut result = String::new();
    let mut width = 0;
    for ch in text.chars() {
        let ch_width = ch.width().unwrap_or(0);
        if width + ch_width + 3 > max_width {
            break;
        }
        result.push(ch);
        width += ch_width;
    }
    while result.ends_with(char::is_whitespace) {
        result.pop();
    }
    result.push_str("...");
    result
}

fn wind_direction_label(degrees: f64) -> &'static str {
    const DIRECTIONS: [&str; 8] = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];
    let normalized = degrees.rem_euclid(360.0);
    DIRECTIONS[((normalized + 22.5) / 45.0) as usize % DIRECTIONS.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LocationDisplay;
    use crate::weather::types::CelestialEvents;
    use crate::weather::{WeatherCondition, WeatherData, WeatherLocation, WeatherUnits};

    fn state() -> AppState {
        let mut state = AppState::new(
            WeatherLocation {
                latitude: 49.1,
                longitude: 10.75,
                elevation: None,
            },
            Some("Gunzenhausen".to_string()),
            LocationDisplay::City,
            false,
            WeatherUnits::metric(),
        );
        state.update_weather(WeatherData {
            condition: WeatherCondition::Rain,
            temperature: 12.5,
            precipitation: 3.2,
            wind_speed: 4.0,
            wind_direction: 225.0,
            sun: CelestialEvents::from_bool(true),
            moon_phase: None,
            timestamp: "2025-01-01T12:00:00Z".to_string(),
            attribution: "test provider".to_string(),
        });
        state
    }

    #[test]
    fn model_uses_only_available_weather_values() {
        let model = HudModel::from_state(&state(), "test provider", true);
        assert_eq!(model.temperature, "12.5°C");
        assert_eq!(model.wind, "14.4 km/h SW");
        assert!(model.details.iter().any(|line| line.contains("TIME")));
        assert!(!model.details.iter().any(|line| line.contains("humidity")));
    }

    #[test]
    fn model_uses_selected_units_and_marks_offline_data() {
        let mut state = state();
        state.units = WeatherUnits::imperial();
        state.set_offline_mode(true);

        let model = HudModel::from_state(&state, "test provider", true);

        assert_eq!(model.temperature, "54.5°F");
        assert_eq!(model.wind, "8.9 mph SW");
        assert_eq!(model.precipitation, "0.1in");
        assert!(model.offline);
        assert!(model.details.iter().any(|line| line == "MODE OFFLINE"));
    }

    #[test]
    fn model_covers_all_weather_conditions_without_inventing_fields() {
        assert_eq!(WeatherCondition::ALL.len(), 14);

        for condition in WeatherCondition::ALL {
            let mut state = state();
            let mut weather = state.current_weather.take().expect("test weather");
            weather.condition = *condition;
            state.update_weather(weather);

            let model = HudModel::from_state(&state, "", false);

            assert!(!model.condition.is_empty());
            assert!(model.details.is_empty());
        }
    }

    #[test]
    fn long_text_is_fitted_without_losing_the_ascii_ellipsis() {
        assert_eq!(fit_text("A very long location", 10), "A very...");
        assert_eq!(fit_text("abcdef", 3), "abc");
    }

    #[test]
    fn hidden_location_is_not_copied_into_the_hud_model() {
        let mut state = state();
        state.hide_location = true;
        let model = HudModel::from_state(&state, "provider", true);
        assert_eq!(model.location, None);
        assert!(model.details.iter().any(|line| line == "LOC hidden"));
        assert!(
            !model
                .details
                .iter()
                .any(|line| line.contains("Gunzenhausen"))
        );
    }
}
