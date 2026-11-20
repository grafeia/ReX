//! Print the layout tree rex produces for a TeX string — the debugging aid for metric
//! questions (glyph ids, kern sizes, box extents), independent of any renderer.
//!
//! ```text
//! cargo run --example layout_dump -- fonts/xits.otf '\frac{1}{2}'
//! ```

use rex::font::FontContext;
use rex::layout::{LayoutSettings, Style};
use rex::render::Renderer;

fn main() {
    let path = std::env::args().nth(1).unwrap();
    let tex = std::env::args().nth(2).unwrap();
    let data = std::fs::read(path).unwrap();
    let font = rex::font::parse(&data).unwrap();
    let ctx = FontContext::new(&font);
    let renderer = Renderer::new();
    let layout = renderer
        .layout(&tex, LayoutSettings::new(&ctx, 10.0, Style::Display))
        .unwrap();
    println!("{:#?}", layout);
    println!("size {:?}", renderer.size(&layout));
}
