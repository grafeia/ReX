//! CPU output: a [`Backend`] that rasterises straight into a `vello_cpu` pixmap.
//!
//! `vello_cpu` is immediate mode — every draw call is rasterised into strips at once and
//! [`RenderContext::render`] composites them — so there is no scene to hand around. That
//! makes this the backend for tests and for the command line: no GPU, no wgpu, a PNG out.
//!
//! Enable with `--features vello_cpu`.

use super::bez::{outline_to_bez, stroked_rect, transform_to_affine};
use super::{Backend, Cursor, Role};
use crate::font::MathFont;
use crate::parser::color::RGBA;
use font::{pathfinder_impl::flatten, Font, GlyphId};
use pathfinder_geometry::{rect::RectF, transform2d::Transform2F, vector::Vector2F};
use vello_cpu::kurbo::{Affine, Rect};
use vello_cpu::peniko::color::{AlphaColor, Srgb};
use vello_cpu::{Pixmap, RenderContext, Resources};

pub struct CpuBackend {
    ctx: RenderContext,
    /// Maps layout space (1 unit = 1 em of type size) to pixels.
    transform: Affine,
    color: AlphaColor<Srgb>,
    color_stack: Vec<AlphaColor<Srgb>>,
}

impl CpuBackend {
    /// `width` x `height` pixels; `transform` maps layout space into that pixmap.
    pub fn new(width: u16, height: u16, transform: Affine) -> Self {
        CpuBackend {
            ctx: RenderContext::new(width, height),
            transform,
            color: AlphaColor::from_rgba8(0, 0, 0, 255),
            color_stack: Vec::new(),
        }
    }

    pub fn width(&self) -> u16 {
        self.ctx.width()
    }

    pub fn height(&self) -> u16 {
        self.ctx.height()
    }

    /// Paint the whole pixmap one colour first — a page background. Without it the
    /// untouched pixels stay transparent, which reads as black in most viewers.
    pub fn fill_background(&mut self, r: u8, g: u8, b: u8) {
        let (width, height) = (self.ctx.width(), self.ctx.height());
        self.ctx.set_transform(Affine::IDENTITY);
        self.ctx.set_paint(AlphaColor::<Srgb>::from_rgba8(r, g, b, 255));
        self.ctx
            .fill_rect(&Rect::new(0.0, 0.0, width as f64, height as f64));
    }

    /// Rasterise everything drawn so far.
    pub fn finish(mut self) -> Pixmap {
        let mut resources = Resources::new();
        let mut pixmap = Pixmap::new(self.ctx.width(), self.ctx.height());
        self.ctx.flush();
        self.ctx.render(&mut pixmap, &mut resources);
        pixmap
    }

    fn fill(&mut self, bez: vello_cpu::kurbo::BezPath, transform: Affine) {
        self.ctx.set_transform(self.transform * transform);
        self.ctx.set_paint(self.color);
        self.ctx.fill_path(&bez);
    }
}

impl Backend for CpuBackend {
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
        let rect = Rect::new(pos.x, pos.y, pos.x + width, pos.y + height);
        self.ctx.set_transform(self.transform);
        self.ctx.set_paint(self.color);
        self.ctx.fill_rect(&rect);
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
