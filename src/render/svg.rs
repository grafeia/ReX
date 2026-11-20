//! SVG output: a [`Backend`] that writes the layout as filled paths.
//!
//! This replaces the old `scene.rs`, which recorded a `pathfinder_renderer::scene::Scene`
//! and let `pathfinder_export` write the SVG — both crates are unused elsewhere and no
//! longer part of the tree we build against. Every draw call becomes one `<path>` with
//! an absolute `d` (the transform is applied to the outline first, exactly like the old
//! `outline.transformed(&tr)`), so the file needs no nested groups and no font.
//!
//! The view box is measured from what was actually drawn, so the output does not depend
//! on a size hint being passed in correctly.

use super::bez::{outline_to_bez, stroked_rect};
use super::{Backend, Cursor, Role};
use crate::font::{FontContext, MathFont};
use crate::layout::{LayoutSettings, Style};
use crate::parser::color::RGBA;
use crate::render::Renderer;
use font::{pathfinder_impl::flatten, Font, GlyphId};
use pathfinder_content::outline::Outline;
use pathfinder_geometry::{rect::RectF, transform2d::Transform2F, vector::Vector2F};

pub struct SvgBackend {
    body: String,
    transform: Transform2F,
    color: [u8; 4],
    color_stack: Vec<[u8; 4]>,
    bounds: Option<kurbo::Rect>,
}

impl SvgBackend {
    pub fn new() -> Self {
        SvgBackend::with_transform(Transform2F::default())
    }

    pub fn with_transform(transform: Transform2F) -> Self {
        SvgBackend {
            body: String::new(),
            transform,
            color: [0, 0, 0, 255],
            color_stack: Vec::new(),
            bounds: None,
        }
    }

    /// Emit one outline, transformed by `transform`, as a filled `<path>`.
    fn fill(&mut self, outline: &Outline, transform: Transform2F) {
        use super::bez::transform_to_affine;
        use kurbo::Shape;

        let mut bez = outline_to_bez(outline);
        bez.apply_affine(transform_to_affine(self.transform * transform));
        let bb = bez.bounding_box();
        if bb.width() > 0.0 || bb.height() > 0.0 {
            self.bounds = Some(match self.bounds {
                Some(b) => b.union(bb),
                None => bb,
            });
        }
        let (r, g, b, a) = (self.color[0], self.color[1], self.color[2], self.color[3]);
        self.body.push_str(&format!(
            "  <path d=\"{}\" fill=\"#{:02x}{:02x}{:02x}\"",
            bez.to_svg(),
            r,
            g,
            b
        ));
        if a != 255 {
            self.body.push_str(&format!(" fill-opacity=\"{}\"", a as f64 / 255.0));
        }
        self.body.push_str("/>\n");
    }

    /// The document. `width`/`height` are CSS pixels; the view box keeps the drawing
    /// coordinates 1:1 so nothing is resampled.
    pub fn finish(self) -> String {
        let bb = self.bounds.unwrap_or(kurbo::Rect::new(0.0, 0.0, 1.0, 1.0));
        let mut out = String::with_capacity(self.body.len() + 256);
        out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>\n");
        out.push_str(&format!(
            "<svg width=\"{w}\" height=\"{h}\" viewBox=\"{x} {y} {w} {h}\" \
             xmlns=\"http://www.w3.org/2000/svg\">\n",
            x = bb.x0,
            y = bb.y0,
            w = bb.width(),
            h = bb.height()
        ));
        out.push_str(&self.body);
        out.push_str("</svg>\n");
        out
    }
}

impl Backend for SvgBackend {
    fn bbox(&mut self, pos: Cursor, width: f64, height: f64, role: Role) {
        let color = match role {
            Role::Glyph => [0, 200, 0, 255],
            Role::HBox => [200, 0, 0, 255],
            Role::VBox => [0, 0, 200, 255],
        };
        let outline = stroked_rect(
            RectF::new(
                Vector2F::new(pos.x as f32, pos.y as f32),
                Vector2F::new(width as f32, height as f32),
            ),
            0.1,
        );
        let color_before = self.color;
        self.color = color;
        self.fill(&outline, Transform2F::default());
        self.color = color_before;
    }

    fn symbol(&mut self, pos: Cursor, gid: u16, scale: f64, ctx: &MathFont) {
        let outline = match flatten(ctx, GlyphId(gid as u32)) {
            Some(outline) => outline,
            None => return,
        };
        // The same chain the pathfinder scene used: the look-at transform, then the
        // glyph's position, then the y flip into rex's y-down space, then the font size.
        let tr = Transform2F::from_translation(Vector2F::new(pos.x as f32, pos.y as f32))
            * Transform2F::from_scale(Vector2F::new(scale as f32, -scale as f32))
            * ctx.font_matrix();
        self.fill(&outline, tr);
    }

    fn rule(&mut self, pos: Cursor, width: f64, height: f64) {
        let outline = Outline::from_rect(RectF::new(
            Vector2F::new(pos.x as f32, pos.y as f32),
            Vector2F::new(width as f32, height as f32),
        ));
        self.fill(&outline, Transform2F::default());
    }

    fn begin_color(&mut self, RGBA(r, g, b, a): RGBA) {
        self.color_stack.push(self.color);
        self.color = [r, g, b, a];
    }

    fn end_color(&mut self) {
        self.color = self.color_stack.pop().unwrap_or([0, 0, 0, 255]);
    }
}

impl Default for SvgBackend {
    fn default() -> Self {
        SvgBackend::new()
    }
}

/// Lay out `tex` with the font in `font_data` and return it as an SVG document.
pub fn render_to_string(font_data: &[u8], tex: &str, debug: bool) -> Result<String, String> {
    let font = crate::font::parse(font_data).map_err(|e| format!("font: {:?}", e))?;
    let ctx = FontContext::new(&font);
    let mut renderer = Renderer::new();
    renderer.debug = debug;
    let layout = renderer
        .layout(tex, LayoutSettings::new(&ctx, 10.0, Style::Display))
        .map_err(|e| format!("layout: {:?}", e))?;
    let mut backend = SvgBackend::new();
    renderer.render(&layout, &mut backend);
    Ok(backend.finish())
}

/// Kept for callers of the old `SceneWrapper` API: SVG bytes for `tex`.
pub fn svg(font_data: &[u8], tex: &str) -> Vec<u8> {
    render_to_string(font_data, tex, true)
        .expect("rex::render::svg: layout failed")
        .into_bytes()
}
