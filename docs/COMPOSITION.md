# Surface composition and renderer feasibility

This pass makes tabs and lists transparent by default, with explicit background
overrides. It also evaluates rounded clipping and opacity for arbitrary child
widgets. The clipping/opacity experiments remain test-only: neither is enabled
in the library or gallery. The dependency remains upstream iced 0.14.0.

## Shipped background behavior

Tabs previously painted `theme.colors.surface` behind their children, and the list
container did the same. On elevated cards, custom colors or patterns, that could
leave an unexpected rectangular patch. Both now preserve the enclosing surface.
Text colors, row selection, hover/ripple layers, dividers and indicators retain
their existing recipes.

`Tabs::background` accepts a color or gradient. `Tabs::background_with` resolves
that fill from the current theme. The fill covers the entire viewport, including
the scrollbar, and stays fixed when labels scroll. `list` continues to return a
native iced container; use its existing `style` builder for an explicit fill.

```rust
use iced::{widget, Background, Color};
use iced_material::{list, list_item, tabs, Tab, Theme};

#[derive(Clone)]
enum Message { Page(u8) }

let navigation = tabs([Tab::new(0, "Overview"), Tab::new(1, "Files")], Some(0))
    .on_select(Message::Page)
    .background_with(|theme: &Theme| theme.colors.surface);

let files = list::<Message>([list_item("Design notes").into()])
    .style(|theme: &Theme| widget::container::Style {
        background: Some(Background::Color(theme.colors.surface)),
        text_color: Some(theme.colors.on_surface),
        ..Default::default()
    });

// Omit a background entirely to preserve the parent, or request it explicitly:
let transparent = tabs([Tab::new(0, "Overview")], Some(0))
    .on_select(Message::Page)
    .background(Color::TRANSPARENT);
```

These explicit surface fills restore the previous appearance where it was
intentional. Filled fields, cards and other components with their own semantic
container colors still paint their containers. Transparency is not a blanket
replacement for component surface tokens.

## What the pinned renderer can do

The installed `iced_core` 0.14.0 `renderer::Renderer` API exposes rectangular
`start_layer`/`end_layer`, transformations and quads. Its layer is a clipping and
batching boundary, not an isolated group with adjustable opacity. Quad radii
round the quad; they do not clip its descendants. Image radii/opacity likewise
operate on an image, not an arbitrary widget subtree.

The wgpu renderer owns private command layers and its engine. Shader primitives
can draw their own GPU content, but cannot capture the existing child widget
commands as a texture. They also do not supply a Tiny Skia implementation.
The public headless path can capture ordinary widgets, but adds a rasterization
and transfer boundary rather than an efficient native layer.

Inspected pinned source anchors: `iced_core/src/renderer.rs`,
`iced_core/src/image.rs`, `iced_renderer/src/lib.rs`,
`iced_wgpu/src/{lib,engine,primitive}.rs` and `iced_tiny_skia/src/lib.rs`.
The conclusion concerns the versions in this repository's `Cargo.lock`.

## Reproducible experiments

`tests/visual/compositing_probe.rs` wraps ordinary elements while preserving their
widget tree, layout, operations and input. It compares three paths:

| Path | Result | Limitation |
| --- | --- | --- |
| Direct drawing | Native rectangular drawing; benchmark baseline | No group fade or rounded child clip |
| Horizontal clipping bands | Replays the same child through narrow rectangular clips | Stepped edges; 57 child draw calls per frame at a 28px radius; no group opacity |
| CPU offscreen group | Draws children on transparent Tiny Skia, unpremultiplies RGBA, encodes PNG, wraps it in a rounded SVG clip and draws with SVG opacity | Changing content repeats CPU rendering, encoding, upload and rasterization; fixed 2x capture scale; GPU-only primitives are not preserved |

The offscreen tests pass on Tiny Skia and Metal for opaque overlapping children
faded as one group, transparent interiors and corners over stripes, ancestor
scrolling/clipping, an SVG with raster content, a button, native text entry and a
live progress indicator. Unchanged captures can be reused; events conservatively
invalidate the capture, including a transition's final redraw. This is a
feasibility fixture, not a complete invalidation or resource-management system.
Child overlays are forwarded separately and do not participate in the group.

There is also a visible color-space mismatch: half-transparent blue over red
produced RGB **(163, 91, 166)** through native Metal drawing, versus
**(130, 90, 140)** after CPU isolation. The latter matches Tiny Skia's direct
result. A production implementation must preserve the active renderer's blending
behavior within a group; simply uploading a CPU screenshot cannot guarantee it.

### Measured rendering cost

Release captures on this Apple Silicon host, 2026-09-14. Each row measures 30
changing frames at 2x with native text, input, SVG/raster content and indeterminate
progress. Values are mean / p95 milliseconds and include final headless screenshot
readback. They are local feasibility measurements, not native-window frame-rate
guarantees; GPU scheduling and other host activity affect the results.

| Renderer / logical size | Direct | Bands | CPU offscreen |
| --- | ---: | ---: | ---: |
| Tiny Skia, 280 × 320 | 1.74 / 2.07 | 42.88 / 44.09 | 9.44 / 9.90 |
| Tiny Skia, 560 × 520 | 3.85 / 4.08 | 91.83 / 94.85 | 25.22 / 26.10 |
| Metal, 280 × 320 | 3.83 / 6.71 | 7.79 / 14.07 | 23.38 / 54.42 |
| Metal, 560 × 520 | 3.95 / 7.33 | 8.31 / 15.23 | 37.44 / 51.72 |

With unchanged content, the larger cached offscreen case averaged 6.52ms on
Tiny Skia and 4.85ms on Metal. That saving does not apply when a progress
indicator, caret or child transition changes each frame. An earlier Metal run
with the same offscreen path measured about 22ms versus 2ms direct; the direction
of the cost is consistent despite host timing variation.

Reproduce the tests and generate the local review page:

```sh
cargo test --locked --lib compositing_probe
ICED_TEST_BACKEND=wgpu cargo test --locked --lib compositing_probe
cargo test --locked --release --lib profile_compositing_approaches -- --ignored --nocapture
ICED_TEST_BACKEND=wgpu cargo test --locked --release --lib profile_compositing_approaches -- --ignored --nocapture
python3 tests/visual/report.py
python3 tests/visual/composition-preview.py
```

Profile logs are `target/compositing-probe-{software,gpu}-profile.log`; captures
are under `target/compositing-probe/`. The preview uses those files when available
and labels experiments separately from shipped components. Run timing profiles
sequentially, without other test workloads, for a useful comparison.

## Production decision and proposed integration

Keep the fast existing carousel corner patches and known-surface fades until
renderer integration is available. Carousel `.background(...)` still needs a
matching opaque parent color. Gradients/patterns cannot be matched by a single
corner-patch color. Existing content fades still use a known surface overlay and
do not supply mathematically correct arbitrary-subtree opacity. This pass does
not claim to close those two gaps.

The next implementation should provide an isolated drawing group inside iced's
renderer layer, preferably upstream, with this contract:

1. Collect normal child draw commands into a group with bounds, a rounded clip
   and opacity. Composite the completed group once, so overlapping children fade
   correctly. Preserve command order, transforms, ancestor clips and nested groups.
2. Use pooled native offscreen targets on wgpu, an antialiased rounded mask and
   one final composition. Avoid CPU readback, PNG/SVG conversion and texture
   upload per animation frame. Provide equivalent native Tiny Skia groups.
3. Resolve display scale and blending/color space in the compositor. Bound and
   reuse allocations, handle resize/device loss, and retain sensible fast paths
   for transparent and fully visible groups. A zero-opacity draw must not change
   the widget's event or focus lifecycle.
4. Keep the existing widget trees and editor state. Redraw scheduling must preserve
   progress, caret and ripple updates behind overlays. Decide explicitly whether
   a popup overlay is part of the group or escapes it.

Acceptance coverage should include patterned parents, opaque and translucent
overlaps, nested clips, text and raster/SVG content, native editing, scrolling,
resize and fractional display scale on both renderers. Custom GPU primitives need
an explicit supported contract. Compare release frame cost against direct drawing
for moving carousels and dialogs before enabling the path in components.

This follows the project's preference for upstream iced without a framework fork:
the experiments establish the boundary and a concrete compromise. No patched
renderer, alternate public `Element` type or runtime dependency was introduced.
The PNG/base64 capture machinery is development-only.
