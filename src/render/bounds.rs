//! Measuring: a [`Backend`] that draws nothing and only accumulates the box the layout
//! occupies.
//!
//! The GPU backends need the page size before they can be set up (a `wgpu` texture, a
//! `vello_cpu` pixmap), and `Renderer::size` only reports the layout's ascent/descent
//! hints — this walks the same draw calls the real backends see and unions the actual
//! ink. Render once into this, then render again into the real backend with a transform
//! that fits the box.

use super::bez::outline_to_bez;
use super::{Backend, Cursor, Role};
use crate::font::MathFont;
use crate::parser::color::RGBA;
use font::{pathfinder_impl::flatten, Font, GlyphId};
use pathfinder_geometry::{rect::RectF, transform2d::Transform2F, vector::Vector2F};

pub struct BoundsBackend {
    transform: Transform2F,
    bounds: Option<RectF>,
}

impl BoundsBackend {
    pub fn new() -> Self {
        BoundsBackend::with_transform(Transform2F::default())
    }

    pub fn with_transform(transform: Transform2F) -> Self {
        BoundsBackend {
            transform,
            bounds: None,
        }
    }

    /// The union of everything drawn, in layout space; `None` if nothing was drawn.
    pub fn bounds(&self) -> Option<RectF> {
        self.bounds
    }

    fn add(&mut self, rect: RectF) {
        if !(rect.width().is_finite() && rect.height().is_finite()) {
            return;
        }
        self.bounds = Some(match self.bounds {
            Some(b) => b.union_rect(rect),
            None => rect,
        });
    }
}

impl Default for BoundsBackend {
    fn default() -> Self {
        BoundsBackend::new()
    }
}

impl Backend for BoundsBackend {
    fn symbol(&mut self, pos: Cursor, gid: u16, scale: f64, ctx: &MathFont) {
        let outline = match flatten(ctx, GlyphId(gid as u32)) {
            Some(outline) => outline,
            None => return,
        };
        let tr = self.transform
            * Transform2F::from_translation(Vector2F::new(pos.x as f32, pos.y as f32))
            * Transform2F::from_scale(Vector2F::new(scale as f32, -scale as f32))
            * ctx.font_matrix();
        self.add(outline_to_bez(&outline).transformed_bounds(tr));
    }

    fn rule(&mut self, pos: Cursor, width: f64, height: f64) {
        let rect = pathfinder_content::outline::Outline::from_rect(RectF::new(
            Vector2F::new(pos.x as f32, pos.y as f32),
            Vector2F::new(width as f32, height as f32),
        ));
        self.add(rect.transformed(&self.transform).bounds());
    }

    fn bbox(&mut self, _pos: Cursor, _width: f64, _height: f64, _role: Role) {
        // Debug boxes are not content, and including them would make the page the size
        // of the outermost box.
    }

    fn begin_color(&mut self, _color: RGBA) {}

    fn end_color(&mut self) {}
}

/// The bounds of `outline` after `transform`, as a pathfinder rect (`RectF` is y-down,
/// so `min_y`/`max_y` are top/bottom).
trait TransformedBounds {
    fn transformed_bounds(&self, transform: Transform2F) -> RectF;
}

impl TransformedBounds for kurbo::BezPath {
    fn transformed_bounds(&self, transform: Transform2F) -> RectF {
        let mut path = self.clone();
        path.apply_affine(super::bez::transform_to_affine(transform));
        let bb = kurbo::Shape::bounding_box(&path);
        RectF::from_points(
            Vector2F::new(bb.x0 as f32, bb.y0 as f32),
            Vector2F::new(bb.x1 as f32, bb.y1 as f32),
        )
    }
}
