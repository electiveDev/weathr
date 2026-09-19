use crate::render::TerminalRenderer;
use crate::scene::world::style::WorldSceneStyle;
use std::io;

// These silhouettes intentionally match the original Veirt/weathr world scene.
const TREE_ASCII: &str = include_str!("assets/tree.txt");
const FENCE_ASCII: &str = include_str!("assets/fence.txt");

pub struct Decorations;

pub struct DecorationLayout {
    pub horizon_y: u16,
    pub house_x: u16,
    pub house_width: u16,
    pub width: u16,
}

impl Decorations {
    pub fn render(
        &self,
        renderer: &mut TerminalRenderer,
        layout: &DecorationLayout,
        style: &WorldSceneStyle,
    ) -> io::Result<()> {
        self.render_tree(renderer, layout, style)?;
        self.render_fence(renderer, layout, style)?;

        Ok(())
    }

    fn render_tree(
        &self,
        renderer: &mut TerminalRenderer,
        layout: &DecorationLayout,
        style: &WorldSceneStyle,
    ) -> io::Result<()> {
        let tree_x = layout.house_x.saturating_sub(20);
        if tree_x == 0 {
            return Ok(());
        }
        let line_count = TREE_ASCII.lines().count() as u16;
        let tree_y = layout.horizon_y.saturating_sub(line_count);
        render_art(renderer, TREE_ASCII, tree_x, tree_y, style.tree_foliage)
    }

    fn render_fence(
        &self,
        renderer: &mut TerminalRenderer,
        layout: &DecorationLayout,
        style: &WorldSceneStyle,
    ) -> io::Result<()> {
        let fence_x = layout.house_x.saturating_add(layout.house_width + 2);
        if fence_x >= layout.width {
            return Ok(());
        }
        let line_count = FENCE_ASCII.lines().count() as u16;
        let fence_y = layout.horizon_y.saturating_sub(line_count);
        render_art(renderer, FENCE_ASCII, fence_x, fence_y, style.fence)
    }
}

fn render_art(
    renderer: &mut TerminalRenderer,
    ascii: &str,
    x: u16,
    y: u16,
    color: crossterm::style::Color,
) -> io::Result<()> {
    for (i, line) in ascii.lines().enumerate() {
        for (j, ch) in line.chars().enumerate() {
            if ch != ' ' {
                renderer.render_char(x + j as u16, y + i as u16, ch, color)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_assets_match_the_original_silhouettes() {
        assert_eq!(TREE_ASCII.lines().count(), 5);
        assert!(TREE_ASCII.contains("####"));
        assert!(TREE_ASCII.contains("_||_"));
    }
}
