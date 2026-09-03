use crate::render::TerminalRenderer;
use crate::theme::VisualPalette;
use crate::ui::hud::fit_text;
use crate::ui::layout::Rect;
use std::io;

pub fn render(
    renderer: &mut TerminalRenderer,
    rect: Rect,
    attribution: &str,
    offline: bool,
    details_visible: bool,
    palette: VisualPalette,
) -> io::Result<()> {
    if rect.is_empty() {
        return Ok(());
    }
    renderer.set_viewport(Some(rect));
    let state = if offline { "[OFFLINE]" } else { "[LIVE]" };
    let source = if attribution.trim().is_empty() {
        "Awaiting weather data"
    } else {
        attribution
    };
    let suffix = if details_visible {
        "  F1 details  q quit"
    } else {
        ""
    };
    let left = format!("{}  {}", state, source);
    let available = rect.width as usize;
    let line = fit_text(&format!("{}{}", left, suffix), available);
    renderer.render_line_colored(0, 0, &line, palette.text_muted)?;
    renderer.clear_viewport();
    Ok(())
}
