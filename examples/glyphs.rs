//! Print every glyph and rule the renderer places for a TeX string: `gid x y size`
//! (or `rule x y width height`), in layout units, with the origin on the formula's
//! baseline. The metric harness for "did this change move anything?".
//!
//! ```text
//! cargo run --example glyphs -- tests/out/rex-xits.otf '\frac{1}{2}'
//! ```

use rex::font::FontContext;
use rex::layout::{LayoutSettings, Style};
use rex::render::{Backend, Cursor, Renderer};

struct Dump;

impl Backend for Dump {
    fn symbol(&mut self, pos: Cursor, gid: u16, scale: f64, _font: &rex::font::MathFont) {
        println!("glyph {} {:.5} {:.5} {:.5}", gid, pos.x, pos.y, scale);
    }

    fn rule(&mut self, pos: Cursor, width: f64, height: f64) {
        println!("rule {:.5} {:.5} {:.5} {:.5}", pos.x, pos.y, width, height);
    }

    fn begin_color(&mut self, _color: rex::render::RGBA) {}
    fn end_color(&mut self) {}
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!("usage: glyphs <font> <tex> [display|text]");
        std::process::exit(2);
    }
    let style = match args.get(2).map(|s| &s[..]) {
        Some("text") => Style::Text,
        _ => Style::Display,
    };

    let data = std::fs::read(&args[0]).unwrap();
    let font = rex::font::parse(&data).unwrap();
    let ctx = FontContext::new(&font);
    let renderer = Renderer::new();
    let layout = renderer
        .layout(&args[1], LayoutSettings::new(&ctx, 10.0, style))
        .unwrap();

    println!("size {:.5} {:.5} {:.5} {:.5}", 0.0, layout.depth / rex::dimensions::Px, layout.width / rex::dimensions::Px, layout.height / rex::dimensions::Px);
    renderer.render(&layout, &mut Dump);
}
