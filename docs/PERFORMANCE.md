# Rendering performance

The dialog investigation found avoidable shadow repainting in iced-m3 and a
separate cost in iced 0.14's software renderer. The library fix reduces the
matched dialog benchmark's software repaint median by about 30%. Software
rendering remains substantially slower than GPU rendering for this composition.

## Run the benchmark

The ignored `performance::profile_dialog_rendering` library test is an explicit
release benchmark. It is not a timing gate in CI. Run it on an otherwise idle
machine, with the same compiler, feature configuration, scale and power settings
for comparisons:

```sh
ICED_TEST_BACKEND=tiny-skia cargo test --locked --release --lib profile_dialog_rendering -- --ignored --nocapture
ICED_TEST_BACKEND=wgpu cargo test --locked --release --lib profile_dialog_rendering -- --ignored --nocapture
```

The software command also works with `--no-default-features`. GPU measurements
require a hardware adapter; they do not silently substitute software rendering.
The environment variable `ICED_PERF_SCENE` optionally selects `Form`, `Panel`,
`Scrim`, `DialogNoShadow`, `Dialog`, `NativeForm` or `NativeScrim`.

The fixture uses 1280×800 logical pixels at 2× scale, 8 warmup frames and 30
measured scrolling frames. Form, panel and dialog variants have matching body
positions, dimensions and content, checked by an ordinary test. The body is
600px high; the dialog's total height including padding is 648px. The dialog is
mounted already open. This measures settled repainting, not its entrance.

The output separates:

- **UI:** event handling, layout invalidation and recording draw commands.
- **Repaint:** software damage comparison/grouping and painting, or GPU submission
  and waiting for completion. No screenshot readback, PNG encoding, window
  presentation or vsync is timed.
- **Cold repaint:** the first sample, before warmup. Treat this as an order-dependent
  diagnostic: GPU scenes share an engine, so the first scene also pays shared
  pipeline startup costs. It is not an application startup measurement.
- **Damage:** the last frame's layer count, region count and summed logical area.
  Regions may overlap, so summed area is not unique damaged area.

## Measured result — 2026-09-18

Local Apple M4 Pro / Metal, Rust 1.98.1, release profile and default crate features.
The baseline was the code at `1466716`, with the same matched-size benchmark.
These are local medians in milliseconds, not portable latency guarantees:

| Scene | Software before | Software after | GPU before | GPU after |
| --- | ---: | ---: | ---: | ---: |
| Form | 18.27 | 18.14 | 1.39 | 1.37 |
| Form with panel fill | 33.05 | 31.57 | 1.39 | 1.38 |
| Panel plus scrim | 54.74 | 51.67 | 1.39 | 1.40 |
| Dialog, shadows disabled | 55.59 | 51.90 | 1.38 | 1.40 |
| Dialog | 77.00 | 53.99 | 1.46 | 1.47 |

Repeated baseline runs put the dialog at 76.71–77.00ms. UI processing was around
0.04ms. The larger software costs come from painting; the small changes between
control scenes should not be attributed to the shadow fix. The GPU difference
is within the variation of these runs. P95 and cold samples are emitted by the
benchmark but are not used as acceptance thresholds. Local raw logs are retained
in `target/performance/{before,after}-{cpu,gpu}.log`.

These measurements differ from Comet's original 115.14ms report: this fixture
matches the plain form and dialog geometry, runs the current source and uses a
simpler scene. It does not claim a measured improvement in the Comet application.

## Library fix

Resizing dialogs use small cached SVG shadow templates, tiled around the panel.
Previously, the straight-edge tiles' clipping rectangles included a broad
transparent strip over the dialog body. Scrolling damaged that strip, and the
software renderer repainted the shadow image despite there being no shadow ink
in the intersecting area.

The straight-edge clips now end near the panel edge, with a raster-rounding
guard. Corner tiles keep their full extent to preserve the curved blur and its
transition to straight edges. The cached raster and animation geometry remain
unchanged. Disabled or fully transparent shadows also return before recording
any clipping layers or allocating a cache entry.

This removes most shadow-related work from body-only repaints. It does not remove
shadows, change the dialog's colors, shorten animations, or require an iced fork.
All 2,679 canonical visual references compare exactly without baseline updates.

## Remaining renderer cost and upstream reproduction

Adding a panel and scrim also raises repaint cost without any shadow. A native
iced control experiment isolates this further: `NativeForm` and `NativeScrim`
use iced's own text inputs and theme, scrollable and containers, with no Material
text-field drawing or dialog host. The outer container styles only supply solid
colors and a corner radius.

The benchmark can override the software damage regions to explore this cost:

```sh
ICED_TEST_BACKEND=tiny-skia ICED_PERF_SCENE=NativeScrim ICED_PERF_DAMAGE=tracked cargo test --locked --release --lib profile_dialog_rendering -- --ignored --nocapture
ICED_TEST_BACKEND=tiny-skia ICED_PERF_SCENE=NativeScrim ICED_PERF_DAMAGE=union cargo test --locked --release --lib profile_dialog_rendering -- --ignored --nocapture
ICED_TEST_BACKEND=tiny-skia ICED_PERF_SCENE=NativeScrim ICED_PERF_DAMAGE=full cargo test --locked --release --lib profile_dialog_rendering -- --ignored --nocapture
```

`tracked` uses iced's normal damage calculation. `union` replaces nonempty damage
with its single bounding rectangle. `full` repaints the whole window whenever
there is damage. These are benchmark-only controls, not application settings.
GPU rendering ignores these software experiments.

| Scene after the library fix | Tracked regions | One union | Whole window |
| --- | ---: | ---: | ---: |
| Form | 18.14ms | 9.48ms | 9.63ms |
| Dialog | 53.99ms | 19.65ms | 28.59ms |
| Native form | 5.09ms | 1.61ms | 1.99ms |
| Native form, panel and scrim | 42.04ms | 11.03ms | 8.72ms |

The final Material frame had 8 regions with a summed area of 520,916 logical
pixels, versus one union covering 357,588. The native frame had 11 regions.
Redrawing even the whole window can outperform these smaller regions.

The released source explains where to investigate:

- [`iced_graphics::damage::group`](https://github.com/iced-rs/iced/blob/3997291f318a8bc06fa522f5579836fb3feb94df/graphics/src/damage.rs)
  greedily groups rectangles; the output can still overlap.
- [`iced_tiny_skia::Renderer::draw`](https://github.com/iced-rs/iced/blob/3997291f318a8bc06fa522f5579836fb3feb94df/tiny_skia/src/lib.rs)
  visits intersecting layers for every damage region.
- [Its drawing engine](https://github.com/iced-rs/iced/blob/3997291f318a8bc06fa522f5579836fb3feb94df/tiny_skia/src/engine.rs)
  resets a full-target clip mask and paints intersecting quad paths through that
  mask. Large backgrounds can therefore incur repeated work for small changes.

The experiments establish expensive fragmented repainting, but they do not
isolate every millisecond between mask clearing, path rasterization and blending.
A suitable upstream report is: **“Tiny Skia partial repainting can be slower than
full repainting for scrolling forms over large backgrounds.”** Include the native
fixture, the three damage-policy commands and these measurements. This report is
prepared here and has not been submitted.

Potential upstream fixes are better merging of overlapping damage, a cost-based
fallback to a larger repaint region, and clipping raster work more tightly to the
physical damage bounds. Always taking a union would be premature: sparse changes
far apart could make it slower. Any upstream fix needs additional workloads and
pixel-correctness checks. Until then, the default GPU path avoids the observed
software bottleneck.

## Regression coverage

Ordinary CI runs deterministic tests in `tests/performance/checks.rs`:

- The benchmark's Material scenes have identical scroll viewport/content geometry.
- Scrolling the inset body touches at most the four corner shadow tiles, independent
  of the number of straight-edge tiles along a large dialog.
- Disabled and transparent shadows record no image or clipping work.
- Incremental dialog painting matches a complete repaint in light/dark themes at
  1×, 1.25× and 2× scale; unchanged frames produce zero damage. The incremental
  comparison allows at most one byte per channel for Tiny Skia's partially
  clipped glyph blend rounding. Canonical full-frame golden comparisons stay exact.

Existing checks also verify bounded cached shadow dimensions, handle reuse across
resizing and opacity changes, Gaussian outline fidelity, and animation scheduling.
Timing results remain diagnostic. Add further benchmark scenes when they cover a
distinct rendering path or a reported regression, rather than assigning a fixed
millisecond budget to every widget on shared CI runners.
