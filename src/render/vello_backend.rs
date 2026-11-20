//! GPU output: a [`Backend`] that records the layout into a `vello::Scene`.
//!
//! This is the `pathfinder_renderer` replacement. It only *records*; turning the scene
//! into pixels needs a wgpu device and a `vello::Renderer`, which a caller has to set up
//! (`vello_cpu` does that work on the CPU if no GPU is available — see [`super::cpu`]).
//!
//! Every draw is a fill of a kurbo path with the current colour; the y flip lives in the
//! glyph transform, so a scene is in rex's y-down layout space, with `(0, 0)` at the
//! start of the first line and `1.0` one em of type size.

use super::bez::{outline_to_bez, stroked_rect, transform_to_affine};
use super::{Backend, Cursor, Role};
use crate::font::MathFont;
use crate::parser::color::RGBA;
use font::{pathfinder_impl::flatten, Font, GlyphId};
use pathfinder_geometry::{rect::RectF, transform2d::Transform2F, vector::Vector2F};
use vello::kurbo::{Affine, BezPath};
use vello::peniko::color::{AlphaColor, Srgb};
use vello::peniko::Fill;
use vello::Scene;

pub struct VelloBackend {
    scene: Scene,
    transform: Affine,
    color: AlphaColor<Srgb>,
    color_stack: Vec<AlphaColor<Srgb>>,
}

impl VelloBackend {
    pub fn new() -> Self {
        VelloBackend::with_transform(Transform2F::default())
    }

    pub fn with_transform(transform: Transform2F) -> Self {
        VelloBackend {
            scene: Scene::new(),
            transform: transform_to_affine(transform),
            color: AlphaColor::from_rgba8(0, 0, 0, 255),
            color_stack: Vec::new(),
        }
    }

    /// The recorded scene, ready for `vello::Renderer::render`.
    pub fn finish(self) -> Scene {
        self.scene
    }

    fn fill(&mut self, bez: BezPath, transform: Affine) {
        let transform = self.transform * transform;
        let color = self.color;
        self.scene
            .fill(Fill::NonZero, transform, color, None, &bez);
    }
}

impl Default for VelloBackend {
    fn default() -> Self {
        VelloBackend::new()
    }
}

impl Backend for VelloBackend {
    fn bbox(&mut self, pos: Cursor, width: f64, height: f64, role: Role) {
        let (r, g, b) = match role {
            Role::Glyph => (0, 200, 0),
            Role::HBox => (200, 0, 0),
            Role::VBox => (0, 0, 200),
        };
        let outline = stroked_rect(
            RectF::new(
                Vector2F::new(pos.x as f32, pos.y as f32),
                Vector2F::new(width as f32, height as f32),
            ),
            0.1,
        );
        let color_before = self.color;
        self.color = AlphaColor::from_rgba8(r, g, b, 255);
        self.fill(outline_to_bez(&outline), Affine::IDENTITY);
        self.color = color_before;
    }

    fn symbol(&mut self, pos: Cursor, gid: u16, scale: f64, ctx: &MathFont) {
        let outline = match flatten(ctx, GlyphId(gid as u32)) {
            Some(outline) => outline,
            None => return,
        };
        let tr = Transform2F::from_translation(Vector2F::new(pos.x as f32, pos.y as f32))
            * Transform2F::from_scale(Vector2F::new(scale as f32, -scale as f32))
            * ctx.font_matrix();
        self.fill(outline_to_bez(&outline), transform_to_affine(tr));
    }

    fn rule(&mut self, pos: Cursor, width: f64, height: f64) {
        let bez = vello::kurbo::Rect::new(
            pos.x,
            pos.y,
            pos.x + width,
            pos.y + height,
        );
        let color = self.color;
        self.scene
            .fill(Fill::NonZero, self.transform, color, None, &bez);
    }

    fn begin_color(&mut self, RGBA(r, g, b, a): RGBA) {
        self.color_stack.push(self.color);
        self.color = AlphaColor::from_rgba8(r, g, b, a);
    }

    fn end_color(&mut self) {
        self.color = self
            .color_stack
            .pop()
            .unwrap_or(AlphaColor::from_rgba8(0, 0, 0, 255));
    }
}
