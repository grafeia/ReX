//! Geometry shared by the renderers: a pathfinder [`Outline`] as a kurbo [`BezPath`].
//!
//! `pathfinder_content` stays the crate's geometry type (glyphs are encoded into
//! `Outline`s at parse time, see [`crate::font`]); the rendering backends want kurbo
//! paths, so this is the one place that converts. The mapping is the same one
//! `pdf_render`'s vello backend uses: a contour's first position is a `move_to`, each
//! segment becomes a line/quad/cubic, and a closed contour gets a `close_path` —
//! `pathfinder` only ever stores lines, quadratics and cubics, so nothing is
//! approximated.


use pathfinder_content::outline::{ContourIterFlags, Outline};
use pathfinder_content::segment::SegmentKind;
use pathfinder_geometry::rect::RectF;
use pathfinder_geometry::transform2d::Transform2F;
use pathfinder_geometry::vector::Vector2F;

/// One pathfinder outline as a kurbo path, untransformed.
pub fn outline_to_bez(outline: &Outline) -> kurbo::BezPath {
    fn point(v: Vector2F) -> kurbo::Point {
        kurbo::Point::new(v.x() as f64, v.y() as f64)
    }

    let mut bez = kurbo::BezPath::new();
    for contour in outline.contours() {
        if let Some(p) = contour.first_position() {
            bez.move_to(point(p));
        }
        for elem in contour.iter(ContourIterFlags::empty()) {
            match elem.kind {
                SegmentKind::None => {}
                SegmentKind::Line => {
                    bez.line_to(point(elem.baseline.to()));
                }
                SegmentKind::Quadratic => {
                    bez.quad_to(point(elem.ctrl.from()), point(elem.baseline.to()));
                }
                SegmentKind::Cubic => {
                    bez.curve_to(
                        point(elem.ctrl.from()),
                        point(elem.ctrl.to()),
                        point(elem.baseline.to()),
                    );
                }
            }
        }
        if contour.is_closed() {
            bez.close_path();
        }
    }
    bez
}

/// A pathfinder transform as a kurbo affine.
pub fn transform_to_affine(transform: Transform2F) -> kurbo::Affine {
    kurbo::Affine::new([
        transform.m11() as f64,
        transform.m21() as f64,
        transform.m12() as f64,
        transform.m22() as f64,
        transform.m13() as f64,
        transform.m23() as f64,
    ])
}

/// `rect` drawn as a stroke of `line_width`, expanded into a fillable outline — what the
/// debug bounding boxes of every backend draw.
///
/// `pathfinder_content`'s stroke expander does the work, so the bboxes do not need the
/// backend's own stroke support and the SVG, vello and CPU output agree exactly.
pub fn stroked_rect(rect: RectF, line_width: f32) -> Outline {
    use pathfinder_content::stroke::{LineCap, LineJoin, OutlineStrokeToFill, StrokeStyle};

    let style = StrokeStyle {
        line_cap: LineCap::Square,
        line_join: LineJoin::Bevel,
        line_width,
    };
    let rect = Outline::from_rect(rect);
    let mut stroke = OutlineStrokeToFill::new(&rect, style);
    stroke.offset();
    stroke.into_outline()
}
