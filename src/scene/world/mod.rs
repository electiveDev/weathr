mod decorations;
mod ground;
mod house;
mod style;

use crate::render::TerminalRenderer;
use crate::scene::{ChimneyPosition, Scene, SceneContext, SceneLayout};
use crate::ui::Rect;
use decorations::{DecorationLayout, Decorations};
use ground::Ground;
use house::House;
use std::io;
use style::WorldSceneStyle;

pub struct WorldScene {
    house: House,
    ground: Ground,
    decorations: Decorations,
    viewport: Rect,
}

impl WorldScene {
    const GROUND_HEIGHT: u16 = 7;

    fn ground_height(scene_height: u16) -> u16 {
        Self::GROUND_HEIGHT.min(scene_height.saturating_sub(House.height()))
    }

    pub fn new(viewport: Rect) -> Self {
        Self {
            house: House,
            ground: Ground,
            decorations: Decorations,
            viewport,
        }
    }
}

impl Scene for WorldScene {
    fn id(&self) -> &'static str {
        "world"
    }

    fn update_size(&mut self, viewport: Rect) {
        self.viewport = viewport;
    }

    fn layout(&self) -> SceneLayout {
        let width = self.viewport.width;
        let height = self.viewport.height;
        let ground_height = Self::ground_height(height);
        let ground_y = height.saturating_sub(ground_height);
        let house_x = (width / 2).saturating_sub(self.house.width() / 2);
        let house_y = ground_y.saturating_sub(self.house.height());
        let chimney_x = house_x + House::CHIMNEY_X_OFFSET;

        SceneLayout {
            viewport: self.viewport,
            ground_y,
            chimney_pos: Some(ChimneyPosition {
                x: chimney_x,
                y: house_y,
            }),
            width,
            height,
        }
    }

    fn render(&self, renderer: &mut TerminalRenderer, ctx: &SceneContext<'_>) -> io::Result<()> {
        let layout = self.layout();
        let house_x = (layout.width / 2).saturating_sub(self.house.width() / 2);
        let house_y = layout.ground_y.saturating_sub(self.house.height());
        let style = WorldSceneStyle::resolve(ctx);

        self.ground.render(
            renderer,
            layout.width,
            Self::ground_height(layout.height),
            layout.ground_y,
            &style,
        )?;
        self.house.render(renderer, house_x, house_y, &style)?;
        self.decorations.render(
            renderer,
            &DecorationLayout {
                horizon_y: layout.ground_y,
                house_x,
                house_width: self.house.width(),
                width: layout.width,
            },
            &style,
        )?;

        Ok(())
    }
}
