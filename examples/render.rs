//! Render one TeX string with one font.
//!
//! ```text
//! cargo run --example render -- fonts/xits.otf '\frac{a}{b}' out.svg
//! cargo run --example render --features vello_cpu -- fonts/xits.otf '\frac{a}{b}' out.png
//! ```
//!
//! `size` is the type size in pixels: the layout is measured in units of `FONT_SIZE`
//! pixels per em (that is the number in `LayoutSettings`), so the transform scales by
//! `size / FONT_SIZE`. The page is the measured extent of the layout plus `PAD` pixels
//! on every side. `vello` (a recorded `vello::Scene`) is the
//! GPU path and has no way to write a file on its own, so it is not an output format
//! here — `vello_cpu` is the one that makes pixels.

use std::fs;
use std::process::ExitCode;

use rex::font::FontContext;
use rex::layout::{LayoutSettings, Style};
use rex::render::{BoundsBackend, Renderer, SvgBackend};

const PAD: f64 = 8.0;
/// The type size the layout is computed at, in layout units per em.
const FONT_SIZE: f64 = 10.0;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 3 {
        eprintln!("usage: render <font> <tex> <out.svg|out.png> [size=32] [debug=1]");
        return ExitCode::FAILURE;
    }
    let (font_path, tex, out) = (&args[0], &args[1], &args[2]);
    let size: f64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(32.0);
    let debug = args.get(4).map(|s| s != "0").unwrap_or(true);

    let data = match fs::read(font_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("cannot read {}: {}", font_path, e);
            return ExitCode::FAILURE;
        }
    };

    // Layout: one renderer for both output paths. `Layout` borrows the font, so both
    // live in this scope.
    let font = match rex::font::parse(&data) {
        Ok(font) => font,
        Err(e) => {
            eprintln!("cannot parse {}: {:?}", font_path, e);
            return ExitCode::FAILURE;
        }
    };
    let ctx = FontContext::new(&font);
    let mut renderer = Renderer::new();
    renderer.debug = debug;
    let layout = match renderer.layout(tex, LayoutSettings::new(&ctx, FONT_SIZE, Style::Display)) {
        Ok(layout) => layout,
        Err(e) => {
            eprintln!("cannot lay out {:?}: {:?}", tex, e);
            return ExitCode::FAILURE;
        }
    };

    // The page box is the ink box of the layout, not the ascent/descent hints `size`
    // reports: a fraction's numerator sits above the ascent and rex records its depth
    // with the sign of the outline bounds.
    let mut probe = BoundsBackend::new();
    renderer.render(&layout, &mut probe);
    let bb = probe
        .bounds()
        .unwrap_or(pathfinder_geometry::rect::RectF::new(
            pathfinder_geometry::vector::Vector2F::zero(),
            pathfinder_geometry::vector::Vector2F::new(1.0, 1.0),
        ));
    let (x0, y0) = (bb.min_x() as f64, bb.min_y() as f64);
    let (w_units, h_units) = (bb.width() as f64, bb.height() as f64);

    if out.ends_with(".svg") {
        let mut backend = SvgBackend::new();
        renderer.render(&layout, &mut backend);
        if let Err(e) = fs::write(out, backend.finish()) {
            eprintln!("cannot write {}: {}", out, e);
            return ExitCode::FAILURE;
        }
        println!("{}: {:.1}x{:.1} units", out, w_units, h_units);
        return ExitCode::SUCCESS;
    }

    if out.ends_with(".png") {
        #[cfg(feature = "vello_cpu")]
        {
            use rex::render::CpuBackend;
            use vello_cpu::kurbo::Affine;

            let scale = size / FONT_SIZE;
            let w_px = (w_units * scale + 2.0 * PAD).ceil() as u16;
            let h_px = (h_units * scale + 2.0 * PAD).ceil() as u16;
            // layout units -> pixels, with (x0, y0) landing on the padding corner.
            let transform = Affine::translate((PAD - x0 * scale, PAD - y0 * scale))
                * Affine::scale(scale);

            let mut backend = CpuBackend::new(w_px, h_px, transform);
            backend.fill_background(255, 255, 255);
            renderer.render(&layout, &mut backend);
            let pixmap = backend.finish();
            let png = match pixmap.into_png() {
                Ok(png) => png,
                Err(e) => {
                    eprintln!("cannot encode png: {}", e);
                    return ExitCode::FAILURE;
                }
            };
            if let Err(e) = fs::write(out, &png) {
                eprintln!("cannot write {}: {}", out, e);
                return ExitCode::FAILURE;
            }
            println!("{}: {}x{} px, {} bytes", out, w_px, h_px, png.len());
            return ExitCode::SUCCESS;
        }
        #[cfg(not(feature = "vello_cpu"))]
        {
            eprintln!("png output needs --features vello_cpu");
            return ExitCode::FAILURE;
        }
    }

    eprintln!("unknown output format: {} (want .svg or .png)", out);
    ExitCode::FAILURE
}
