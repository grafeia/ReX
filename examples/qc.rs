use std::fs;
use rex::{
    render::{BoundsBackend, Renderer, SvgBackend},
    layout::{Grid, Layout, engine, LayoutSettings, Style},
    parser::parse,
    font::FontContext
};

const SAMPLES: &[&str] = &[
    r"\iint \sqrt{1 + f^2(x,t,t)}\,\mathrm{d}x\mathrm{d}y\mathrm{d}t = \sum \xi(t)",
    r"\Vert f \Vert_2 = \sqrt{\int f^2(x)\,\mathrm{d}x}",
    r"\left.x^{x^{x^x_x}_{x^x_x}}_{x^{x^x_x}_{x^x_x}}\right\} \mathrm{wat?}",
    r"\hat A\grave A\bar A\tilde A\hat x \grave x\bar x\tilde x\hat y\grave y\bar y\tilde y",
    r"\mathop{\overbrace{1+2+3+\unicodecdots+n}}\limits^{\mathrm{Arithmatic}} = \frac{n(n+1)}{2}",
    r"\sigma = \left(\int f^2(x)\,\mathrm{d}x\right)^{1/2}",
    r"\left\vert\sum_k a_k b_k\right\vert \leq \left(\sum_k a_k^2\right)^{\frac12}\left(\sum_k b_k^2\right)^{\frac12}",
    r"f^{(n)}(z) = \frac{n!}{2\pi i} \oint \frac{f(\xi)}{(\xi - z)^{n+1}}\,\mathrm{d}\xi",
    r"\frac{1}{\left(\sqrt{\phi\sqrt5} - \phi\right) e^{\frac{2}{5}\pi}} = 1 + \frac{e^{-2\pi}}{1 + \frac{e^{-4\pi}}{1 + \frac{e^{-6\pi}}{1 + \frac{e^{-8\pi}}{1 + \unicodecdots}}}}",
    r"\mathop{\mathrm{lim\,sup}}\limits_{x\rightarrow\infty}\ \mathop{\mathrm{sin}}(x)\mathrel{\mathop{=}\limits^?}1"
];


fn main() {
    env_logger::init();

    let samples: Vec<_> = SAMPLES.iter().cloned().map(|tex| parse(dbg!(tex)).unwrap()).collect();

    // Fonts: the paths given on the command line, else everything in `fonts/`.
    let mut paths: Vec<std::path::PathBuf> = std::env::args().skip(1).map(Into::into).collect();
    if paths.is_empty() {
        paths = fs::read_dir("fonts")
            .map(|dir| dir.filter_map(|e| e.ok()).map(|e| e.path()).collect())
            .unwrap_or_default();
    }
    if paths.is_empty() {
        eprintln!("no fonts: pass .otf files as arguments, or put them in ./fonts");
        std::process::exit(1);
    }
    let fonts: Vec<_> = paths.into_iter()
        .filter_map(|path| {
            fs::read(&path).ok()
            .and_then(|data| font::parse(&data).ok())
            .map(|font| (font, path))
        })
        .filter(|(font, _)| font.math.is_some())
        .collect();

    let mut grid = Grid::new();
    for (row, (font, path)) in fonts.iter().enumerate() {
        let ctx = FontContext::new(&font);
        let layout_settings = LayoutSettings::new(&ctx, 10.0, Style::Display);

        let name = format!("\\mathtt{{{}}}", path.file_name().unwrap().to_str().unwrap());
        if let Ok(node) = engine::layout(&parse(&name).unwrap(), layout_settings).map(|l| l.as_node()) {
            grid.insert(row, 0, node);
        }
        for (column, sample) in samples.iter().enumerate() {
            if let Ok(node) = engine::layout(sample, layout_settings).map(|l| l.as_node()) {
                grid.insert(row, column+1, node);
            }
        }
    }

    let mut layout = Layout::new();
    layout.add_node(grid.build());

    let mut renderer = Renderer::new();
    let mut backend = SvgBackend::new();
    renderer.render(&layout, &mut backend);

    let svg = backend.finish();
    fs::write("qc.svg", &svg).unwrap();
    println!("qc.svg: {} bytes, {} paths", svg.len(), svg.matches("<path").count());

    // With the CPU backend the same grid also comes out as a PNG, which is the only way
    // to look at the result without an SVG rasteriser.
    #[cfg(feature = "vello_cpu")]
    {
        use rex::render::CpuBackend;
        use vello_cpu::kurbo::Affine;

        let mut probe = BoundsBackend::new();
        renderer.render(&layout, &mut probe);
        let bb = probe.bounds().unwrap();
        let scale = 2.0; // ~20 px per em at the 10 Px layout em
        let pad = 8.0;
        let w_px = (bb.width() as f64 * scale + 2.0 * pad).ceil() as u16;
        let h_px = (bb.height() as f64 * scale + 2.0 * pad).ceil() as u16;
        let transform = Affine::translate((
            pad - bb.min_x() as f64 * scale,
            pad - bb.min_y() as f64 * scale,
        )) * Affine::scale(scale);

        let mut backend = CpuBackend::new(w_px, h_px, transform);
        backend.fill_background(255, 255, 255);
        renderer.render(&layout, &mut backend);
        let png = backend.finish().into_png().expect("cannot encode png");
        fs::write("qc.png", &png).unwrap();
        println!("qc.png: {}x{} px, {} bytes", w_px, h_px, png.len());
    }
}