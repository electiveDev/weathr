use std::fmt;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }

    pub fn right(self) -> u16 {
        self.x.saturating_add(self.width)
    }

    pub fn bottom(self) -> u16 {
        self.y.saturating_add(self.height)
    }

    pub fn contains(self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutTier {
    Large,
    Medium,
    Small,
}

impl fmt::Display for LayoutTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Large => "large",
            Self::Medium => "medium",
            Self::Small => "small",
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderRegions {
    pub screen: Rect,
    pub scene: Rect,
    pub hud: Option<Rect>,
    pub status: Rect,
    pub tier: LayoutTier,
}

impl RenderRegions {
    pub fn calculate(width: u16, height: u16, hud_visible: bool, details_visible: bool) -> Self {
        let screen = Rect::new(0, 0, width, height);
        let status = if height == 0 {
            Rect::default()
        } else {
            Rect::new(0, height - 1, width, 1)
        };
        let body = Rect::new(0, 0, width, height.saturating_sub(1));
        let tier = if width >= 120 && height >= 28 {
            LayoutTier::Large
        } else if width >= 90 && height >= 23 {
            LayoutTier::Medium
        } else {
            LayoutTier::Small
        };

        if !hud_visible || body.is_empty() {
            return Self {
                screen,
                scene: body,
                hud: None,
                status,
                tier,
            };
        }

        // Keep the HUD close to the original terminal presentation: one fitted
        // summary line at the top, with an optional second line for F1 details.
        // The scene always starts below it, regardless of the responsive tier.
        let hud_height = if details_visible { 2 } else { 1 }.min(body.height);
        let scene = Rect::new(0, hud_height, width, body.height.saturating_sub(hud_height));
        let hud = Rect::new(0, 0, width, hud_height);

        Self {
            screen,
            scene,
            hud: (!hud.is_empty()).then_some(hud),
            status,
            tier,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_the_three_layout_tiers_at_the_declared_boundaries() {
        assert_eq!(
            RenderRegions::calculate(70, 20, true, false).tier,
            LayoutTier::Small
        );
        let small = RenderRegions::calculate(70, 20, true, false);
        assert_eq!(small.hud, Some(Rect::new(0, 0, 70, 1)));
        assert_eq!(small.scene, Rect::new(0, 1, 70, 18));
        assert_eq!(
            RenderRegions::calculate(70, 20, true, true)
                .hud
                .unwrap()
                .height,
            2
        );
        assert_eq!(
            RenderRegions::calculate(70, 20, true, true).scene,
            Rect::new(0, 2, 70, 17)
        );
        assert_eq!(
            RenderRegions::calculate(89, 22, true, false).tier,
            LayoutTier::Small
        );
        assert_eq!(
            RenderRegions::calculate(90, 23, true, false).tier,
            LayoutTier::Medium
        );
        assert_eq!(
            RenderRegions::calculate(119, 27, true, false).tier,
            LayoutTier::Medium
        );
        assert_eq!(
            RenderRegions::calculate(120, 28, true, false).tier,
            LayoutTier::Large
        );
    }

    #[test]
    fn hud_is_a_top_line_for_every_responsive_tier() {
        for (width, height) in [(70, 20), (90, 23), (120, 28), (180, 50)] {
            let summary = RenderRegions::calculate(width, height, true, false);
            let details = RenderRegions::calculate(width, height, true, true);

            assert_eq!(summary.hud, Some(Rect::new(0, 0, width, 1)));
            assert_eq!(summary.scene.y, 1);
            assert_eq!(details.hud, Some(Rect::new(0, 0, width, 2)));
            assert_eq!(details.scene.y, 2);
        }
    }

    #[test]
    fn hud_and_scene_are_disjoint() {
        for (width, height) in [(70, 20), (90, 23), (120, 28), (180, 50)] {
            for details in [false, true] {
                let regions = RenderRegions::calculate(width, height, true, details);
                let hud = regions.hud.expect("HUD must fit supported sizes");
                assert!(
                    regions.scene.right() <= hud.x
                        || regions.scene.bottom() <= hud.y
                        || hud.right() <= regions.scene.x
                        || hud.bottom() <= regions.scene.y,
                    "scene and HUD overlap for {width}x{height} ({details})"
                );
                assert!(!regions.scene.is_empty());
                assert!(!regions.status.is_empty());
            }
        }
    }

    #[test]
    fn hidden_hud_gives_the_scene_the_whole_body() {
        let regions = RenderRegions::calculate(90, 23, false, true);
        assert_eq!(regions.hud, None);
        assert_eq!(regions.scene, Rect::new(0, 0, 90, 22));
    }

    #[test]
    fn all_regions_stay_inside_the_screen() {
        for (width, height) in [(70, 20), (90, 23), (120, 28), (240, 60)] {
            for details in [false, true] {
                let regions = RenderRegions::calculate(width, height, true, details);

                assert!(regions.scene.right() <= regions.screen.right());
                assert!(regions.scene.bottom() <= regions.screen.bottom());
                assert!(regions.status.right() <= regions.screen.right());
                assert!(regions.status.bottom() <= regions.screen.bottom());
                if let Some(hud) = regions.hud {
                    assert!(hud.right() <= regions.screen.right());
                    assert!(hud.bottom() <= regions.screen.bottom());
                }
            }
        }
    }
}
