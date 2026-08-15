use crate::render::TerminalRenderer;
use crate::scene::world::style::{TreeAppearance, WorldSceneStyle};
use std::io;

const TREE_ASCII: &str = include_str!("assets/tree.txt");
const SPRING_TREE_ASCII: &str = include_str!("assets/tree_spring.txt");
const AUTUMN_TREE_ASCII: &str = include_str!("assets/tree_autumn.txt");
const WINTER_TREE_ASCII: &str = include_str!("assets/tree_winter.txt");
const FENCE_ASCII: &str = include_str!("assets/fence.txt");
const MAILBOX_ASCII: &str = include_str!("assets/mailbox.txt");
const PINE_TREE_ASCII: &str = include_str!("assets/pine_tree.txt");
const SPRING_BLOSSOM_SPOTS: [(u16, u16); 7] =
    [(7, 0), (4, 1), (10, 1), (3, 2), (12, 2), (6, 3), (10, 4)];
const AUTUMN_LEAF_SPOTS: [(u16, u16); 7] = [(7, 0), (4, 1), (9, 1), (2, 2), (9, 2), (6, 3), (4, 4)];
const SUMMER_FRUIT_SPOTS: [(u16, u16); 9] = [
    (10, 1),
    (15, 2),
    (7, 3),
    (13, 3),
    (18, 4),
    (5, 5),
    (14, 5),
    (10, 6),
    (17, 6),
];

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
        let tree_ascii = match style.tree_appearance {
            TreeAppearance::Bare => WINTER_TREE_ASCII,
            TreeAppearance::Blossoming => SPRING_TREE_ASCII,
            TreeAppearance::Autumn => AUTUMN_TREE_ASCII,
            TreeAppearance::Full => TREE_ASCII,
        };
        let line_count = tree_ascii.lines().count() as u16;
        let tree_y = layout.horizon_y.saturating_sub(line_count);
        render_art(renderer, tree_ascii, tree_x, tree_y, style.tree_foliage)?;

        match style.tree_appearance {
            TreeAppearance::Blossoming => Self::render_tree_accents(
                renderer,
                tree_x,
                tree_y,
                '*',
                &SPRING_BLOSSOM_SPOTS,
                &style.flower_colors,
            )?,
            TreeAppearance::Autumn => Self::render_tree_accents(
                renderer,
                tree_x,
                tree_y,
                '*',
                &AUTUMN_LEAF_SPOTS,
                &style.flower_colors,
            )?,
            TreeAppearance::Full => Self::render_tree_accents(
                renderer,
                tree_x,
                tree_y,
                'o',
                &SUMMER_FRUIT_SPOTS,
                &style.fruit_colors,
            )?,
            TreeAppearance::Bare => {}
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
    fn render_tree_accents(
        renderer: &mut TerminalRenderer,
        tree_x: u16,
        tree_y: u16,
        symbol: char,
        spots: &[(u16, u16)],
        colors: &[crossterm::style::Color],
    ) -> io::Result<()> {
        for (index, (offset_x, offset_y)) in spots.iter().enumerate() {
            renderer.render_char(
                tree_x.saturating_add(*offset_x),
                tree_y.saturating_add(*offset_y),
                symbol,
                colors[index % colors.len()],
            )?;
        }
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_spots_are_filled(tree: &str, spots: &[(u16, u16)]) {
        for &(x, y) in spots {
            let character = tree
                .lines()
                .nth(y as usize)
                .and_then(|line| line.chars().nth(x as usize));
            assert!(
                matches!(character, Some(character) if character != ' '),
                "tree accent spot ({x}, {y}) must be inside the tree"
            );
        }
    }

    #[test]
    fn seasonal_tree_assets_have_distinct_growth_stages() {
        assert_eq!(SPRING_TREE_ASCII.lines().count(), 8);
        assert_eq!(TREE_ASCII.lines().count(), 10);
        assert_eq!(AUTUMN_TREE_ASCII.lines().count(), 8);
        assert_eq!(WINTER_TREE_ASCII.lines().count(), 8);
        assert!(TREE_ASCII.lines().count() > SPRING_TREE_ASCII.lines().count());
        assert!(SPRING_TREE_ASCII.contains("###"));
        assert!(AUTUMN_TREE_ASCII.contains("  "));
        assert!(WINTER_TREE_ASCII.contains('|'));
    }

    #[test]
    fn seasonal_tree_accents_fit_their_assets() {
        assert_spots_are_filled(SPRING_TREE_ASCII, &SPRING_BLOSSOM_SPOTS);
        assert_spots_are_filled(TREE_ASCII, &SUMMER_FRUIT_SPOTS);
        assert_spots_are_filled(AUTUMN_TREE_ASCII, &AUTUMN_LEAF_SPOTS);
    }
}
