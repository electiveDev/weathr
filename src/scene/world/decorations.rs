use crate::render::TerminalRenderer;
use crate::scene::world::style::{TreeAppearance, WorldSceneStyle};
use std::io;

const TREE_ASCII: &str = include_str!("assets/tree.txt");
const FENCE_ASCII: &str = include_str!("assets/fence.txt");
const MAILBOX_ASCII: &str = include_str!("assets/mailbox.txt");
const PINE_TREE_ASCII: &str = include_str!("assets/pine_tree.txt");
const BARE_TREE_ASCII: &str =
    "       |       \n      /|\\      \n       |       \n       |       \n     __|__";
const TREE_ACCENT_SPOTS: [(u16, u16); 7] =
    [(7, 0), (5, 1), (10, 1), (4, 2), (11, 2), (7, 3), (10, 3)];

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
        self.render_mailbox(renderer, layout, style)?;

        if layout.width > 120 {
            self.render_pine_tree(renderer, layout, style)?;
        }

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
        let (tree_ascii, accent) = match style.tree_appearance {
            TreeAppearance::Bare => (BARE_TREE_ASCII, None),
            TreeAppearance::Blossoming | TreeAppearance::Autumn => (TREE_ASCII, Some('*')),
            TreeAppearance::Full => (TREE_ASCII, None),
        };
        let line_count = tree_ascii.lines().count() as u16;
        let tree_y = layout.horizon_y.saturating_sub(line_count);
        render_art(renderer, tree_ascii, tree_x, tree_y, style.tree_foliage)?;

        if let Some(symbol) = accent {
            for (index, (offset_x, offset_y)) in TREE_ACCENT_SPOTS.iter().enumerate() {
                renderer.render_char(
                    tree_x.saturating_add(*offset_x),
                    tree_y.saturating_add(*offset_y),
                    symbol,
                    style.flower_colors[index % style.flower_colors.len()],
                )?;
            }
        }

        Ok(())
    }
    fn render_fence(
        &self,
        renderer: &mut TerminalRenderer,
        layout: &DecorationLayout,
        style: &WorldSceneStyle,
    ) -> io::Result<()> {
        let fence_x = layout.house_x + layout.house_width + 2;
        if fence_x >= layout.width {
            return Ok(());
        }
        let line_count = FENCE_ASCII.lines().count() as u16;
        let fence_y = layout.horizon_y.saturating_sub(line_count);
        render_art(renderer, FENCE_ASCII, fence_x, fence_y, style.fence)
    }

    fn render_mailbox(
        &self,
        renderer: &mut TerminalRenderer,
        layout: &DecorationLayout,
        style: &WorldSceneStyle,
    ) -> io::Result<()> {
        let tree_x = layout.house_x.saturating_sub(20);
        let Some(mailbox_x) = tree_x.checked_sub(10) else {
            return Ok(());
        };
        if mailbox_x >= layout.width {
            return Ok(());
        }
        let line_count = MAILBOX_ASCII.lines().count() as u16;
        let mailbox_y = layout.horizon_y.saturating_sub(line_count);
        render_art(renderer, MAILBOX_ASCII, mailbox_x, mailbox_y, style.mailbox)
    }

    fn render_pine_tree(
        &self,
        renderer: &mut TerminalRenderer,
        layout: &DecorationLayout,
        style: &WorldSceneStyle,
    ) -> io::Result<()> {
        let pine_x = layout.house_x + layout.house_width + 18;
        if pine_x + 10 >= layout.width {
            return Ok(());
        }
        let line_count = PINE_TREE_ASCII.lines().count() as u16;
        let pine_y = layout.horizon_y.saturating_sub(line_count);
        render_art(
            renderer,
            PINE_TREE_ASCII,
            pine_x,
            pine_y,
            style.tree_foliage,
        )
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
