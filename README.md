# ReX &nbsp; [![](https://tokei.rs/b1/github/cbreeden/rex)](https://github.com/cbreeden/rex)

<p align="center"><img src="rex.png" alt="ReX logo" width="300px"/></p>
<h3 align="center">Typesetting Mathematics</h3>

# Samples

You can try ReX live [here](https://retex.github.io/iReX/editor.html)!  Simply type in a formula in the editor and click the display on the top to update the rendering.

Note: ReX rendered all of these examples in SVG, but due to limitations in SVG rendering on GitHub, we need to convert them to PNG.
See the `samples/` folder for the original SVG source.

### The Quadratic Fromula
`x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}`

![Example](samples/The_Quadratic_Fromula.png)

### Double angle formula for Sine
`\sin(\theta + \phi) = \sin(\theta)\cos(\phi) + \sin(\phi)\cos(\theta)`

![Example](samples/Double_angle_formula_for_Sine.png)

### Divergence Theorem
`\int_D (\nabla \cdot F)\,\mathrm{d}V = \int_{\partial D} F \cdot n\,\mathrm{d}S`

![Example](samples/Divergence_Theorem.png)

### Standard Deviation
`\sigma = \sqrt{ \frac{1}{N} \sum_{i=1}^N (x_i - \mu)^2 }`

![Example](samples/Standard_Deviation.png)

### Fourier Inverse
`f(x) = \int_{-\infty}^{\infty} \hat f(\xi) e^{2\pi i \xi x}\,\mathrm{d}\xi`

![Example](samples/Fourier_Inverse.png)

### Cauchy-Schwarz Inequality
`\left\vert \sum_k a_kb_k \right\vert \leq \left(\sum_k a_k^2\right)^{\frac12}\left(\sum_k b_k^2\right)^{\frac12}`

![Example](samples/Cauchy-Schwarz_Inequality.png)

### Exponent
`e = \lim_{n \to \infty} \left(1 + \frac{1}{n}\right)^n`

![Example](samples/Exponent.png)

### Ramanujan's Identity
`\frac{1}{\pi} = \frac{2\sqrt{2}}{9801} \sum_{k=0}^\infty \frac{ (4k)! (1103+26390k) }{ (k!)^4 396^{4k} }`

![Example](samples/Ramanujan's_Identity.png)

### A surprising identity
`\int_{-\infty}^{\infty} \frac{\sin(x)}{x}\,\mathrm{d}x = \int_{-\infty}^{\infty}\frac{\sin^2(x)}{x^2}\,\mathrm{d}x`

![Example](samples/A_surprising_identity.png)

### Another gem from Ramanujan
`\frac{1}{\left(\sqrt{\phi\sqrt5} - \phi\right) e^{\frac{2}{5}\pi}} = 1 + \frac{e^{-2\pi}}{1 + \frac{e^{-4\pi}}{1 + \frac{e^{-6\pi}}{1 + \frac{e^{-8\pi}}{1 + \cdots}}}}`

![Example](samples/Another_gem_from_Ramanujan.png)

### Another gem from Cauchy
`f^{(n)}(z) = \frac{n!}{2\pi i} \oint \frac{f(\xi)}{(\xi - z)^{n+1}}\,\mathrm{d}\xi`

![Example](samples/Another_gem_from_Cauchy.png)

### An unneccesary number of scripts
`x^{x^{x^x_x}_{x^x_x}}_{x^{x^x_x}_{x^x_x}}`

![Example](samples/An_unneccesary_number_of_scripts.png)

### Quartic Function
`\mathop{\overbrace{c_4x^4 + c_3x^3 + c_2x^2 + c_1x + c_0}}\limits^{\gray{\mathrm{Quartic}}}`

![Example](samples/Quartic_Function.png)

### Another fun identity
`3^3 + 4^4 + 3^3 + 5^5 = 3435`

![Example](samples/Another_fun_identity.png)

# Install

First note that ReX is currently in heavy development and is not intended to be used in any way other than for testing and debugging.
That being said, you can install ReX using a Rust compiler.  Instructions are found [here](https://www.rustup.rs/).

ReX builds as a library plus examples; there is no binary. To render one formula:

```
cargo run --example render -- fonts/xits.otf 'x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}' out.svg
cargo run --example render --features vello_cpu -- fonts/xits.otf 'x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}' out.png 32
```

`--example layout_dump` prints the layout tree for a formula, which is where metric
questions are answered; `--example qc` renders a grid of test formulas (SVG, and also a PNG
when `vello_cpu` is enabled).

# Renderers

Layout is renderer independent: `Renderer::render` walks the layout and calls a `Backend`
(`symbol`, `rule`, `bbox`, `begin_color`/`end_color`). Three backends ship:

* `SvgBackend` — always available. Writes filled `<path>`s, so the file needs no font.
* `VelloBackend` (feature `vello`) — records into a `vello::Scene`. Turning that into
  pixels needs a wgpu device, so it is the backend for GPU hosts.
* `CpuBackend` (feature `vello_cpu`) — rasterises on the CPU into a `vello_cpu::Pixmap`
  and PNG, without a GPU. That is the one for tests and the command line.

`BoundsBackend` draws nothing and reports the ink box, which a backend needs before it can
be sized (a pixmap, a texture). Glyphs stay `pathfinder_content::Outline`s internally and
are converted to kurbo paths in `render::bez`; `pathfinder_renderer`, `pathfinder_export`
and `pathfinder_color` are no longer used.

# License

ReX is primarily distributed under the terms of both the MIT license and
the Apache License (Version 2.0), with portions covered by various BSD-like
licenses.

See LICENSE-APACHE, and LICENSE-MIT for details.
