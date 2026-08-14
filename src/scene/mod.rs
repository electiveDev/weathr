pub mod overlay;
pub mod world;

use crate::render::TerminalRenderer;
use crate::theme::Palette;
use crate::weather::WeatherConditions;
use std::collections::HashMap;
use std::io;

pub struct SceneContext<'a> {
    pub conditions: &'a WeatherConditions,
    pub palette: &'a Palette,
}

/// The inclusive rectangle in which outdoor animals may place their sprites.
///
/// `left`/`top` and `right`/`bottom` are terminal cell coordinates. Keeping
/// this geometry in the scene module lets animation systems use the same lawn
/// as the world renderer without storing a second, frame-local layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LawnBounds {
    pub left: u16,
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
}

impl LawnBounds {
    pub const fn width(self) -> u16 {
        self.right.saturating_sub(self.left).saturating_add(1)
    }

    pub const fn height(self) -> u16 {
        self.bottom.saturating_sub(self.top).saturating_add(1)
    }
}

pub const POND_WIDTH: u16 = 13;
pub const POND_HEIGHT: u16 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PondBounds {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[allow(dead_code)]
impl PondBounds {
    pub const fn right(self) -> u16 {
        self.x.saturating_add(self.width.saturating_sub(1))
    }

    pub const fn bottom(self) -> u16 {
        self.y.saturating_add(self.height.saturating_sub(1))
    }

    pub const fn center_x(self) -> u16 {
        self.x.saturating_add(self.width / 2)
    }
}

/// Return the lower green meadow and brown ground rendered from `ground_y`,
/// while reserving the attribution/status rows at the terminal bottom.
pub fn lawn_bounds(width: u16, height: u16, ground_y: u16) -> Option<LawnBounds> {
    if width == 0 || height < 3 {
        return None;
    }

    let top = ground_y;
    let bottom = height.saturating_sub(2);
    if top > bottom {
        return None;
    }

    Some(LawnBounds {
        left: 0,
        top,
        right: width.saturating_sub(1),
        bottom,
    })
}

/// Place a small, stable pond at the lower-right of the meadow. It is
/// intentionally derived from the same bounds used by animals, so resize
/// cannot make it drift outside the scene.
pub fn pond_bounds(width: u16, height: u16, ground_y: u16) -> Option<PondBounds> {
    let lawn = lawn_bounds(width, height, ground_y)?;
    if lawn.width() < POND_WIDTH.saturating_add(4) || lawn.height() < POND_HEIGHT {
        return None;
    }

    let x = lawn.right.saturating_sub(POND_WIDTH.saturating_add(2));
    let y = lawn.bottom.saturating_sub(POND_HEIGHT.saturating_sub(1));
    Some(PondBounds {
        x,
        y,
        width: POND_WIDTH,
        height: POND_HEIGHT,
    })
}

#[derive(Clone, Copy)]
pub struct SceneLayout {
    pub ground_y: u16,
    pub chimney_pos: Option<ChimneyPosition>,
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, Copy)]
pub struct ChimneyPosition {
    pub x: u16,
    pub y: u16,
}

pub trait Scene: Send + Sync {
    fn id(&self) -> &'static str;
    fn update_size(&mut self, width: u16, height: u16);
    fn render(&self, renderer: &mut TerminalRenderer, ctx: &SceneContext<'_>) -> io::Result<()>;
    fn layout(&self) -> SceneLayout;
}

pub struct SceneRegistry {
    scenes: HashMap<&'static str, Box<dyn Scene>>,
}

impl SceneRegistry {
    pub fn new() -> Self {
        Self {
            scenes: HashMap::new(),
        }
    }

    pub fn register(&mut self, scene: Box<dyn Scene>) {
        self.scenes.insert(scene.id(), scene);
    }

    pub fn get(&self, id: &str) -> Option<&dyn Scene> {
        self.scenes.get(id).map(|scene| scene.as_ref())
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut dyn Scene> {
        self.scenes
            .get_mut(id)
            .map(|scene| -> &mut dyn Scene { scene.as_mut() })
    }
}

impl Default for SceneRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lawn_covers_green_and_brown_ground_without_status_row() {
        let lawn = lawn_bounds(80, 24, 17).expect("normal terminal has a lawn");
        assert_eq!(lawn.top, 17);
        assert_eq!(lawn.bottom, 22);
        assert_eq!(lawn.height(), 6);
        assert_eq!(lawn.right, 79);
    }

    #[test]
    fn pond_is_stable_and_inside_lawn() {
        let lawn = lawn_bounds(80, 24, 17).unwrap();
        let pond = pond_bounds(80, 24, 17).unwrap();
        assert!(pond.x >= lawn.left);
        assert!(pond.right() <= lawn.right);
        assert!(pond.y >= lawn.top);
        assert!(pond.bottom() <= lawn.bottom);
        assert_eq!(pond.width, POND_WIDTH);
        assert_eq!(pond.height, POND_HEIGHT);
    }

    #[test]
    fn tiny_terminals_hide_geometry_instead_of_underflowing() {
        assert!(lawn_bounds(0, 24, 17).is_none());
        assert!(lawn_bounds(70, 2, 1).is_none());
        assert!(pond_bounds(15, 24, 17).is_none());
    }
}
