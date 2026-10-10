# Rendering performance

## Hover elevation — 2026-10-09

Large clickable cards previously generated a new full-surface, two-blur SVG for
each fractional hover elevation. Both iced renderers rasterize SVG filters on
the CPU, including the Metal backend. The changing handle defeated raster reuse.

Interactive elevation now crossfades separate resting/hovered shadow templates.
Only their drawing opacity changes during hover; the handle and raster dimensions
remain stable. Large surfaces tile straight edges around the exact curved corners.
A shallow surface retains its full short axis, preserving overlapping corner
blurs while still bounding the long axis. The shared tiler supports asymmetric
corners and uses layer clipping at tile joins to avoid fractional-scale seams.
Custom fractional `elevated` shadows retain their existing geometry semantics,
using bounded tiles too (see the follow-up audit below).

Crossfading is an approximation of the previous intermediate blur geometry;
resting and hovered appearances are checked against the original Gaussian
outlines. Widget APIs, hit testing and animation timing are unchanged. Renderer
cache eviction, scale/color changes and changing corner shapes can still cause
small templates to be rasterized again; retaining an SVG handle does not pin its
renderer allocation forever.

Run the diagnostic benchmark on an idle machine:

```sh
ICED_TEST_BACKEND=tiny-skia cargo test --locked --release --lib profile_hover_rendering -- --ignored --nocapture
ICED_TEST_BACKEND=wgpu cargo test --locked --release --lib profile_hover_rendering -- --ignored --nocapture
```

The fixture renders a real 1330×250 card at 2×, advances hover-in/out at explicit
16ms timestamps, and measures 20 transition frames separately from idle frames.
It includes the first use of the hovered endpoint. GPU repaint includes submission
and completion, but not window presentation, vsync or screenshot readback.
These are renderer measurements, not whole-application frame rates.

Local Apple M4 Pro / Metal, Rust 1.98.1, release profile, default features:

| Card | Software before | Software after | Metal before | Metal after |
| --- | ---: | ---: | ---: | ---: |
| Elevated | 78.51ms | 19.19ms | 77.11ms | 1.45ms |
| Filled | 320.68ms | 9.81ms | 321.31ms | 1.38ms |
| Outlined | 323.63ms | 9.45ms | 321.89ms | 1.41ms |
| Elevated, transparent shadow | 2.57ms | 2.41ms | 1.30ms | 1.32ms |

The before measurement used an isolated beta.4 source copy with instrumentation
to record shadow handle IDs; the after benchmark is retained here. Both use the
same viewport, card dimensions, transition samples and repaint machinery. Raw
local logs are under `target/hover-shadow-review/performance/`.
Software rendering still pays for clipped image composition and damaged-region
painting; this fix does not establish a sub-millisecond or 60fps software budget.
Metal repaint is now close to the shadow-free control. Recording commands takes
about 0.01ms during the fixed transition. Timings are diagnostic, never CI gates.

Ordinary tests in `tests/performance/hover.rs` assert bounded raster dimensions
and reuse of endpoint handle/size pairs during interrupted hover, reversal and
modal covering/uncovering. Both contracts failed against the original code.
`tests/visual/shadow_motion.rs` checks endpoint fidelity at 1×, 1.25× and 2×,
light/dark themes, narrow/shallow/asymmetric shapes, visible intermediate changes,
reversal and unmodified surface interiors. Run it with `ICED_TEST_BACKEND=wgpu`
to exercise hardware clipping as well. Canonical references cover actual button,
FAB, dialog and gallery transitions, including the changed intermediate shadows.

Consumers can temporarily disable Material shadows without disabling hover
feedback by setting `theme.colors.shadow = iced::Color::TRANSPARENT`.

## Animation audit — 2026-10-09

The follow-up audit found two more expensive paths: bounded press ripples and
custom animated elevation. A ripple previously rendered a circle through an SVG
clipping mask, creating extra full-surface intermediates. Its fading opacity was
also embedded in the SVG, invalidating the raster on every fade frame.

Ripples now paint the radial gradient directly into the rounded outline, with a
transparent outer stop. Fading uses renderer opacity and reuses the raster.
Expansion still rasterizes changing geometry; it is cheaper, not free. Custom
fractional elevation and sheet shadows now use the same perimeter tiler as hover
shadows. Fractional blur geometry is preserved rather than rounded to integer
levels or approximated with a crossfade.

Run the diagnostic fixtures on an idle machine:

```sh
ICED_TEST_BACKEND=tiny-skia cargo test --locked --release --lib profile_animation_rendering -- --ignored --nocapture
ICED_TEST_BACKEND=wgpu cargo test --locked --release --lib profile_animation_rendering -- --ignored --nocapture
```

`ICED_PERF_SCENE` selects `RippleExpand`, `RippleFade`, `CustomShadow`,
`LinearWave`, `Loaders`, `Rings`, `StaggeredLoaders`, or `StaggeredRings`.
The benchmark reports 1×/2×, first-render cost, median command-recording cost,
and median repaint cost, with frozen controls. First-render cost includes renderer
initialization and is not directly comparable across scenarios. Ripple fixtures
isolate the actual drawing helper; the others build real widget trees. These
measurements exclude presentation/vsync and screenshot readback. Ordinary tests
also exercise a real card's press/hold/release and verify message delivery.

Matched M4 Pro / Metal, Rust 1.98.1 release medians at 2× (30 samples):

| Changing scene | Software before | Software after | Metal before | Metal after |
| --- | ---: | ---: | ---: | ---: |
| 1330×250 ripple expansion | 29.14ms | 10.59ms | 26.92ms | 7.82ms |
| Same ripple fading | 48.55ms | 4.35ms | 45.10ms | 1.36ms |
| 1330×250 fractional elevation | 80.91ms | 15.60ms | 74.59ms | 6.71ms |

The initial audit already included the hover optimization above. Raw local logs
are under `target/animation-audit/`. Timings are diagnostic, never CI gates.

A 1330px traveling wave cost 0.71ms software / 1.71ms Metal repaint. Sixteen
48px loading indicators with staggered clocks cost 0.47ms / 1.35ms; sixteen wavy
loading rings cost 1.39ms / 1.53ms. Staggered fixtures sample across complete
morph/loading cycles so identical poses cannot hide cache misses. Command
recording was under 0.25ms for these fixtures. This does not establish a general
performance guarantee for arbitrary sizes, populations or surrounding content.
No comparable rendering cliff was found in these indicator workloads.

CI contracts require opacity-only ripple fading to reuse its raster and custom
elevation to keep raster dimensions bounded. Pixel tests compare the optimized
ripple against the former circle/clip construction at multiple progress values,
opacities, scales, themes and corner shapes. Moving opacity out of the 8-bit SVG
intermediate changes rounding by at most two channel units in software and five
on Metal in this matrix. Fractional-shadow comparisons retain a one-pixel raster
rounding allowance; integer levels allow two channel units and fractional levels
four. Existing interaction references cover the resulting control appearances.

Determinate waves now travel by default independently of progress updates, at
one wavelength per second. Explicit zero speed remains stationary. Tests cover
constant values, custom wavelengths, pause/resume, speed changes, completion,
viewport clipping and reduced motion. See [LOADING_PROGRESS.md](LOADING_PROGRESS.md)
for the distinction between Material Compose and Android Views defaults.

Remaining costs: expanding ripples still scale with rendered area; fractional
custom shadows rebuild small blur templates as their geometry changes; software
painting and layered damage retain the upstream costs described below. This is
a targeted audit, not an exhaustive proof of all component performance.

## Settled dialog repainting

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
