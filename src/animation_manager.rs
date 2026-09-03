use crate::animation::{
    AnimationSystem, ChimneyPosition, FrameCommands, FrameContext, RenderLayer, TerminalSize, Wind,
    airplanes::AirplaneSystem, birds::BirdSystem, chimney::ChimneySmoke, clouds::CloudSystem,
    fireflies::FireflySystem, fog::FogSystem, leaves::FallingLeaves, moon::MoonSystem,
    raindrops::RaindropSystem, snow::SnowSystem, stars::StarSystem, sunny::SunSystem,
    thunderstorm::ThunderstormSystem,
};
use crate::app_state::AppState;
use crate::render::TerminalRenderer;
use crate::scene::SceneLayout;
use crate::theme::VisualPalette;
use crate::weather::{FogIntensity, RainIntensity, SnowIntensity, WeatherConditions};
use rand::Rng;
use std::io;

pub struct AnimationManager {
    systems: Vec<Box<dyn AnimationSystem>>,
    show_leaves: bool,
}

impl AnimationManager {
    pub fn new(term_width: u16, term_height: u16, show_leaves: bool) -> Self {
        let systems: Vec<Box<dyn AnimationSystem>> = vec![
            // Background (code-defined order)
            Box::new(StarSystem::new(term_width, term_height)),
            Box::new(MoonSystem::new(term_width, term_height, None)),
            Box::new(FireflySystem::new(term_width, term_height)),
            Box::new(BirdSystem::new(term_width, term_height)),
            Box::new(SunSystem::new()),
            Box::new(CloudSystem::new(term_width, term_height)),
            Box::new(AirplaneSystem::new(term_width, term_height)),
            // Post-scene
            Box::new(ChimneySmoke::new()),
            // Foreground
            Box::new(RaindropSystem::new(
                term_width,
                term_height,
                RainIntensity::Light,
            )),
            Box::new(ThunderstormSystem::new(term_width, term_height)),
            Box::new(SnowSystem::new(
                term_width,
                term_height,
                SnowIntensity::Light,
            )),
            Box::new(FogSystem::new(term_width, term_height, FogIntensity::Light)),
            Box::new(FallingLeaves::new(term_width, term_height)),
        ];

        debug_assert!(
            {
                let mut seen = std::collections::HashSet::<&'static str>::new();
                systems.iter().all(|s| seen.insert(s.id()))
            },
            "duplicate animation system ids"
        );

        Self {
            systems,
            show_leaves,
        }
    }

    pub fn on_resize(&mut self, width: u16, height: u16) {
        let size = TerminalSize { width, height };
        for system in &mut self.systems {
            system.on_resize(size);
        }
    }

    pub fn update_moon_phase(&mut self, phase: f64) {
        for system in &mut self.systems {
            system.on_moon_phase(phase);
        }
    }

    pub fn update_rain_intensity(&mut self, intensity: RainIntensity) {
        for system in &mut self.systems {
            system.on_rain_intensity(intensity);
        }
    }

    pub fn update_snow_intensity(&mut self, intensity: SnowIntensity) {
        for system in &mut self.systems {
            system.on_snow_intensity(intensity);
        }
    }

    pub fn update_wind(&mut self, speed_kmh: f32, direction_deg: f32) {
        let wind = Wind {
            speed_kmh,
            direction_deg,
        };
        for system in &mut self.systems {
            system.on_wind(wind);
        }
    }

    pub fn update_fog_intensity(&mut self, intensity: FogIntensity) {
        for system in &mut self.systems {
            system.on_fog_intensity(intensity);
        }
    }

    fn make_context<'a>(
        &self,
        conditions: &'a WeatherConditions,
        state: &'a AppState,
        layout: &SceneLayout,
        visual: VisualPalette,
    ) -> FrameContext<'a> {
        let chimney = layout
            .chimney_pos
            .map(|pos| ChimneyPosition { x: pos.x, y: pos.y });

        FrameContext {
            size: TerminalSize {
                width: layout.width,
                height: layout.height,
            },
            scene_viewport: layout.viewport,
            visual,
            horizon_y: layout.ground_y,
            conditions,
            state,
            show_leaves: self.show_leaves,
            chimney,
        }
    }

    fn render_layer(
        &mut self,
        renderer: &mut TerminalRenderer,
        layer: RenderLayer,
        ctx: &FrameContext<'_>,
        rng: &mut impl Rng,
    ) -> io::Result<()> {
        let mut commands = FrameCommands::default();
        renderer.set_viewport(Some(ctx.scene_viewport));
        let result = (|| {
            if ctx.size.width == 0 || ctx.size.height == 0 {
                return Ok(());
            }

            for system in &mut self.systems {
                if system.layer() != layer || !system.is_active(ctx) {
                    continue;
                }
                system.update(ctx, rng, &mut commands);
                system.render(renderer, ctx)?;
            }

            if commands.flash_screen {
                renderer.flash_screen()?;
            }
            Ok(())
        })();
        renderer.clear_viewport();
        result
    }

    fn render_layers(
        &mut self,
        renderer: &mut TerminalRenderer,
        layers: &[RenderLayer],
        ctx: &FrameContext<'_>,
        rng: &mut impl Rng,
    ) -> io::Result<()> {
        for layer in layers {
            self.render_layer(renderer, *layer, ctx, rng)?;
        }
        Ok(())
    }

    pub fn render_background(
        &mut self,
        renderer: &mut TerminalRenderer,
        conditions: &WeatherConditions,
        state: &AppState,
        layout: &SceneLayout,
        visual: VisualPalette,
        rng: &mut impl Rng,
    ) -> io::Result<()> {
        let ctx = self.make_context(conditions, state, layout, visual);
        self.render_layers(
            renderer,
            &[
                RenderLayer::Sky,
                RenderLayer::Celestial,
                RenderLayer::Clouds,
            ],
            &ctx,
            rng,
        )
    }

    pub fn render_chimney_smoke(
        &mut self,
        renderer: &mut TerminalRenderer,
        conditions: &WeatherConditions,
        state: &AppState,
        layout: &SceneLayout,
        visual: VisualPalette,
        rng: &mut impl Rng,
    ) -> io::Result<()> {
        let ctx = self.make_context(conditions, state, layout, visual);
        self.render_layers(renderer, &[RenderLayer::PostScene], &ctx, rng)
    }

    pub fn render_foreground(
        &mut self,
        renderer: &mut TerminalRenderer,
        conditions: &WeatherConditions,
        state: &AppState,
        layout: &SceneLayout,
        visual: VisualPalette,
        rng: &mut impl Rng,
    ) -> io::Result<()> {
        let ctx = self.make_context(conditions, state, layout, visual);
        self.render_layers(
            renderer,
            &[RenderLayer::Weather, RenderLayer::Foreground],
            &ctx,
            rng,
        )
    }
}
