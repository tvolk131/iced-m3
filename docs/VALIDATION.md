# Validation report

## Software dialog repainting — 2026-09-18

The matched release benchmark measured a software dialog median of 77.00ms
before tighter shadow clipping and 53.99ms after; GPU medians were 1.46ms and
1.47ms. Native iced scenes also reproduce the remaining fragmented-repaint cost.
See [the methodology, measurements and upstream reproduction](PERFORMANCE.md).

- All 272 ordinary repository tests pass in default and software-only builds.
- The independent desktop consumer passes all 6 tests in its software-only build.
- All 18 doctests, strict rustdoc, both Clippy feature configurations, formatting
  and example builds pass.
- All 46 canonical reference functions pass: 2,679 PNGs with no baseline changes.
- The resizing-shadow pixel/cache check also passes on wgpu/Metal.
- New tests check viewport parity, bounded shadow repaint work, invisible-shadow
  skips, incremental pixel correctness at three scales and zero idle damage.
- Timing benchmarks are explicit release diagnostics and introduce no CI timing
  thresholds. Native-window input latency and Windows/Linux GPU performance were
  not measured in this investigation.

## First release preparation — 2026-09-15

The release candidate is `iced-m3` **0.1.0-beta.1** (`iced_m3` in Rust).
Manifests, both lockfiles, imports, examples and package tooling use that identity;
no dependency version or component behavior changed. Public metadata, registry
installation examples, a compatibility policy and release notes are present.
The README remains concise at 115 lines and uses public source/image links.

- The renamed archive passes all **261 ordinary tests** and **14 doctests**.
  The independent consumer passes its **five flows in each renderer configuration**
  against the extracted archive. It contains 161 files, approximately **1.82 MB
  compressed**, retaining licenses and release notes while excluding reference PNGs.
- Current-stable Rust **1.98.1** Clippy passes for the library and consumer with
  warnings denied. All library/consumer targets check on Rust **1.88.0**, with
  default and software-only features; transitive versions in both lockfiles are
  unchanged. Software-only rustdoc builds with warnings denied locally.
- A link audit checks **399 links** across consumer Markdown and generated API
  pages, including the renamed crate module, public-source destinations and anchors.
- `cargo publish --dry-run --locked --allow-dirty` succeeds, including registry
  checks, packaging and verification. Cargo explicitly aborts the upload for the
  dry run. **Nothing was published or tagged.** The manifest now permits crates.io
  so this verification works; the independent consumer remains non-publishable.
- Before the rename, all five remote jobs passed at `4332143` in
  [run 34933165692](https://github.com/tvolk131/iced-m3/actions/runs/34933165692),
  including Windows/Linux/macOS, MSRV, packaged consumer and all 2,665 visual
  references. The release candidate's CI additionally checks the docs.rs Linux
  target/features and a clean-checkout publication dry run. Require a successful
  run for the exact release commit before uploading; see
  [current CI runs](https://github.com/tvolk131/iced-m3/actions).

Local logs are in `target/release-preparation/`. Hosted docs.rs remains pending
publication; native Windows/Linux interaction remains outside the headless CI
evidence. See [release preparation](RELEASING.md) for the remaining upload steps.
Older entries below are historical snapshots, including their earlier package
name and publication/CI status; this entry supersedes those readiness statements.

## Consumer documentation — 2026-09-15

The README is now 112 lines, down from 648, with one screenshot, a complete
example and links into the [documentation index](README.md). Getting started,
the cookbook and theming also appear in rustdoc's `guide` module. The crate API
homepage has its own introduction and direct component links. Gallery walkthroughs,
development commands, limitations and references have dedicated source guides.

- All 13 original Rust examples were preserved verbatim in the three guides.
  The README example is compiled separately, for **14 passing doctests**.
- Formatting and documentation generation with warnings denied pass. A local
  link audit checked **391 links** across 11 Markdown pages and five generated
  API pages, including section anchors and rustdoc source ranges.
- The package verification script now requires the guide inputs and screenshot
  and runs doctests from the extracted archive. All **261 ordinary tests**, all
  **14 doctests**, and the independent consumer's **five flows in each renderer
  configuration** pass against that archive. The distributable contains 159 files
  and is approximately **1.82 MB compressed**.
- No component behavior or golden PNG changed; the 2,665-reference suite was not
  rerun for this documentation-only change. No public name/version, repository
  identity or publishing setting changed. Remote CI and docs.rs remain unverified.

Logs: `target/documentation-review/rustdoc.log`, `links.log`, and `package.log`.
The remaining first-release work is tracked in [RELEASING.md](RELEASING.md).

## Desktop beta readiness — 2026-09-14

Northstar Studio is a separate consumer package with its own manifest, lockfile,
features, profiles and message loop. It imports only public APIs and combines
Material widgets with a generic application component, native iced text entry,
buttons and a pick-list popup. See [BETA_READINESS.md](BETA_READINESS.md).

- All 261 existing ordinary tests pass from the extracted crate in the default
  configuration and from the source checkout in the software-only configuration.
  Five new consumer flows pass against the extracted dependency with both feature
  configurations and against Metal from the source checkout. They cover validation,
  exactly-once save, themed native popups, background input blocking, nested discard,
  focus restoration, keyboard traversal, navigation and theme changes.
- All library and consumer targets compile on Rust **1.88.0**, with default and
  software-only features. The consumer's independently resolved lockfile also
  passes. The toolchain default was not changed. Logs: `target/beta-msrv-final.log`.
- Formatting, library/consumer Clippy with warnings denied, all **13 doctests**,
  documentation generation with warnings denied and the release consumer build
  pass. Logs: `target/beta-final-checks.log`, `target/beta-docs-final.log`,
  `target/beta-consumer-metal.log` and `target/beta-package-check.log`.
- The package is approximately **3.45 MB uncompressed / 1.75 MB compressed**,
  versus approximately 72 MB uncompressed before the include list. It retains
  147 files including the gallery's separate icon module, ordinary test sources,
  docs, font/shape inputs and notices. The 2,665 source-checkout golden PNGs are
  excluded from the archive and remain unchanged in the repository. No component
  drawing code changed and no golden images were regenerated in this milestone.
- `tools/check_package.py` verifies the archive and runs the library plus a copied
  independent app against its extracted contents. The dependency tree confirms
  that the app resolves iced-material from the temporary extraction, not the source
  checkout. The machine-readable report is `target/beta-package-report.json`.
  Publication remains disabled; the package/repository identity is not yet verified.
- All 24 consumer review captures were generated at 420/1000px widths and
  1x/1.25x/2x scale in light/dark themes. Representative overview, custom-gradient
  preferences, editor and nested-dialog images were visually inspected. The
  [consumer review](http://127.0.0.1:8766/desktop-beta/) switches theme/size and was
  checked in the browser. These are stable reduced-motion review captures rather
  than new timed goldens or OS display-scaling tests.
- The separate release app launched in a local macOS window. Keyboard activation,
  text replacement, nested discard confirmation, return to the focused editor,
  traversal to Save and the final saved value/snackbar were visually verified.
  The temporary app bundle is under `target/Northstar Studio.app`; it is an
  unsigned local testing artifact, not a distribution installer.

The top-level CI now explicitly checks the consumer on each desktop OS, minimum
Rust in a dedicated job, and the packaged consumer on Linux. Those workflow changes
have not been run remotely here. Native Windows/Linux interaction and IME/display
behavior, screen-reader integration, locale/RTL, arbitrary-child clipping/group
opacity and remaining Expressive coverage remain the documented boundaries.

## Canonical loading and wavy progress — 2026-09-14

The loader now uses the seven canonical Material shapes and AndroidX-matched
cubic morphs. Linear and circular progress expose opt-in waves with the pinned
reference geometry, endpoint motion and completion profiles. See
[LOADING_PROGRESS.md](LOADING_PROGRESS.md) for API defaults, source attribution
and the numerical adaptations used by the Rust renderer.

- All 261 ordinary tests pass in default and software-only builds: 187 library,
  56 integration and 18 gallery tests. Five new renderer/behavior regressions
  also pass on Metal. They cover source curves, transparent parents and gaps,
  pause/resume, reduced motion, completion handoffs, idle scheduling, clock-origin
  independence and continued motion beneath overlays. Four additional unit tests
  exercise the loader spring and wave geometry/timing.
- All 44 strict reference functions pass with updates disabled: 2,665 PNGs.
  There are 86 new light/dark frames covering timed motion, measured endpoints,
  custom waves, loading-to-measured handoffs and narrow/wide gallery layouts.
  The canonical loader changes 66 earlier images; no references were removed.
  Before/after images are retained under `target/loading-progress-before/references`;
  the inventory is `target/loading-progress-reference-diff.txt`.
- All seven source shapes and seven morph midpoints are compared with independently
  exported AndroidX SVGs at 6x scale. At least 99.5% of channels must match exactly;
  remaining differences must lie within one physical pixel of the expected contour.
  This accounts for SVG rasterization of subdivided versus original cubics. Interior
  pixels still match, and the normal timed golden suite remains exact.
- New states, completion sequences, narrow/wide gallery layouts and representative
  changed loader frames were visually inspected. The local
  [motion review](http://127.0.0.1:8766/loading-progress-motion/) replays 362 actual
  renderer captures: six seconds at 30fps in each theme. It includes play/pause,
  scrubbing and links to the strict golden frames. Regenerate it with
  `python3 tests/visual/loading-preview.py` after running the capture helper.
- Formatting, Clippy with warnings denied, all 12 doctests, documentation generation
  and release example builds pass. Final logs are `target/loading-progress-final.log`
  and `target/loading-progress-metal-final.log`.

Release rendering measurements use 60 changing frames at 320×140 logical pixels
and 2x scale, including screenshot readback. Values below are mean / p95 ms.
These compare rendering paths on this host; they are not native-window frame-rate
guarantees. Logs are `target/loading-progress-profile-{software,metal}.log`.

| Fixture | Tiny Skia | Metal |
| --- | --- | --- |
| Flat linear | 0.34 / 0.57 | 1.40 / 1.45 |
| Wavy linear | 0.53 / 0.65 | 1.56 / 2.71 |
| Flat circular | 0.42 / 0.46 | 1.54 / 2.76 |
| Wavy circular | 0.67 / 0.74 | 1.59 / 1.66 |
| Canonical loader | 0.43 / 0.49 | 1.57 / 2.72 |

Production uses cached vector paths on upstream iced 0.14, with no added runtime
dependency or offscreen readback. Restart an existing gallery process and open
**Workspace → Components → Loading with expression** to try the new controls.
Broader Expressive size/shape systems and native renderer-group integration remain
separate work; this milestone does not claim full Material 3 coverage.

## Tab/list composition and renderer experiments — 2026-09-14

Tabs and lists now preserve the enclosing surface by default. Tabs expose color
and gradient background builders; their optional fill covers the viewport and
stays fixed during horizontal scrolling. Lists retain native container styling.
Two redundant gallery overrides were removed. See [COMPOSITION.md](COMPOSITION.md)
for usage, measured renderer limits and the proposed native group integration.

- All 252 ordinary tests pass in default and software-only builds: 178 library,
  56 integration and 18 gallery tests. Six added tests cover parent pixels,
  controlled actions/disabled rows, explicit fills/scrolling, and the experimental
  offscreen group's overlap, native editing/progress and ancestor clipping.
  All six also pass against Metal.
- All 42 strict reference functions pass with updates disabled: 2,579 PNGs.
  There are 28 new light/dark tab/list frames, including held presses and moving
  indicators; eight existing dark gallery frames change at tab scrollbar edges.
  No references were removed. Before/after images are retained under
  `target/composition-foundations-before/references`; the inventory is
  `target/composition-foundations-reference-diff.txt`.
- New references, the changed gallery regions and Tiny Skia/Metal experiment
  captures were visually inspected. The local [composition review](http://127.0.0.1:8766/composition-review/)
  separates shipped behavior from prototypes. Regenerate it with
  `python3 tests/visual/composition-preview.py` after capturing the fixtures.
- The release experiment compares direct drawing, 57-band child replay and a
  CPU offscreen group. At 560×520 logical pixels / 2x, changing frames averaged
  3.85 / 91.83 / 25.22ms on Tiny Skia and 3.95 / 8.31 / 37.44ms on Metal.
  Timings include screenshot readback and vary with host scheduling; they are
  not native-window frame-rate guarantees. A translucent overlap also exposes
  a CPU/Metal blending mismatch. See the detailed methodology and limitations
  in COMPOSITION.md and `target/compositing-probe-*-profile.log`.
- Formatting, Clippy with warnings denied, all 12 doctests, documentation
  generation and release example builds pass. Ordinary/reference logs use
  `target/composition-foundations-*`; Metal probe results are in
  `target/compositing-probe-gpu-tests.log`.

Neither experimental rendering path is enabled in production. Carousel color
patches and known-surface content fades remain unchanged; native renderer groups
are the proposed follow-up. Restart an existing gallery process for the tab/list
background changes.

## Surface composition and inherited icon opacity — 2026-09-14

Outlined fields and selects now default to a transparent fill and clip the border
around the moving label. Explicit fills remain bounded by the field; filled
variants keep their existing container recipe. Gallery and date-picker fields no
longer need parent-color overrides. Disabled list text, chip/navigation icons,
menu triggers and field adornments retain actual foreground alpha. The new
`icon(handle)` widget and internal marks pass that alpha through SVG opacity,
since the pinned iced renderers ignore alpha in the SVG tint itself.

- All 246 ordinary tests pass in default and software-only builds: 172 library,
  56 integration and 18 gallery tests. Five new regressions also pass on Metal:
  patterned-parent transparency and label gaps, bounded explicit fills, icon
  opacity across rebuilds, disabled foreground/action behavior, and narrow-label
  clipping during interrupted focus animation.
- Restoring the previous field/menu/list/chip/navigation code makes three of
  those regressions fail; the corrected code is restored and passes. See
  `target/surface-composition-before-regressions.log`.
- All 41 strict reference functions pass with updates disabled: 2,551 PNGs,
  including 24 new light/dark field, select, disabled and timed floating-label
  frames. 447 earlier references change; none were removed. Before/after images
  are retained under `target/surface-composition-before/references`, with the
  inventory in `target/surface-composition-reference-diff.txt`.
- Patterned fields, gallery settings/dialogs, representative changed states and
  native Metal disabled components were visually inspected. Image-reference
  comparisons remain exact. The separate icon-versus-quad compositing assertion
  allows two 8-bit channel levels for the renderer's different opacity rounding
  paths; zero-opacity icons must match the parent exactly.
- Formatting, Clippy with warnings denied, all 12 doctests and release example
  builds pass. A paired warm release/Metal dialog profile measured mean 4.71ms
  with the previous field renderer and 4.82ms after restoring the new renderer
  (p95 6.40ms and 6.46ms). These measurements include screenshot readback and are
  not native-window frame-rate guarantees. Logs use `target/surface-composition-*`.

[Review the new references](http://127.0.0.1:8766/?filter=surface-composition).
Restart an existing gallery process to use the rebuilt executable. Carousel
corner masking, configurable host backgrounds for tabs/lists, and arbitrary
subtree opacity remain separate follow-ups; this change does not claim to solve
them. Custom third-party widgets must still honor the foreground alpha they
receive; `icon(handle)` provides that behavior for monochrome SVG content.

## Settings field surface correction — 2026-09-14

The settings form's two outlined fields now resolve their fill and floating-label
cutout from `surface_container_highest`, matching their filled parent card.
The old overrides used `surface_container` and produced darker patches.

All 18 gallery behavior tests and the focused five-frame gallery reference
sequence pass. Two new light/dark settings references were visually inspected;
the suite now contains 2,527 images. Formatting, gallery Clippy and the release
gallery build pass. Logs: `target/settings-field-*.log`.

## Overlay/carousel completion and dark disabled buttons — 2026-09-14

Completed changes: semantic dialog action staging, directional menu row fades,
rounded rich-popup/calendar surface/body separation, stable search layout with
72px full-screen headers, fitted carousel keylines/centered hero, bounded lazy
construction and recent-velocity snapping. The dark disabled-button report is
fixed in the shared foreground compositing path. See [DESKTOP_FINISH.md](DESKTOP_FINISH.md).

- All 241 ordinary tests pass in default and software-only builds: 167 library,
  56 integration and 18 gallery tests. New coverage includes interrupted row
  fades, reduced-motion action timing, editor layout through search expansion,
  closing input suppression, 10,000-item lazy construction and overlapping native
  editor state, global callbacks/count shrink, quick flick versus stopped drag,
  fractional end offsets, rich-popup/calendar actions and disabled foregrounds.
- All seven new renderer/interaction regressions pass with wgpu/Metal. The three
  earlier modal-activity tests also pass on Metal, preserving progress behind
  dialogs/sheets and real window-inactive behavior.
- The GPU disabled-button test failed before the correction: the label/fill
  sample ranged from 96 to 100 in the dark theme. It passes after using the real
  translucent foreground, and the corrected native render was visually inspected.
- All 40 strict reference functions pass with updates disabled in the canonical
  software build: 2,525 PNGs, including 202 new light/dark fixed-time frames.
  433 earlier images change with disabled foreground compositing, overlay/search
  timings, popup shadows and fitted carousel/gallery geometry; none were removed.
  Prior images are retained in `target/desktop-finish-before/references`; the
  categorized comparison is `target/desktop-finish-reference-diff.txt`.
- Timed overlay, search, calendar and all five carousel layout contact sheets
  were visually inspected. The review page has been regenerated.
- Clippy with warnings denied, formatting, documentation checks and release
  example builds pass. Release GPU dialog capture averages 6.04ms, median 5.27ms,
  p95 8.13ms and maximum 15.63ms including readback. This is a headless measurement,
  not a native-window frame-rate guarantee. Restart an existing gallery process
  to use the rebuilt executable.

[Review the new frames](http://127.0.0.1:8766/?filter=desktop-finish).
Logs and review images are under `target/desktop-finish-*`. Native screen-reader
work is an assessment only; no AccessKit bridge or OS accessibility test is
claimed. See [ACCESSIBILITY.md](ACCESSIBILITY.md) for the evaluated runtime paths.

## Progress beneath modal overlays — 2026-09-14

Modal interaction suspension now leaves Material progress/loading clocks running
while the app window is active. This supersedes the recurring-indicator pause
noted in the earlier ripple fix below. Pointer/keyboard input, held-gesture
cancellation, native caret suspension and snackbar timeout pauses remain isolated.
Actual app deactivation, explicit pause and reduced motion still stop indicators.

- Three new regression tests exercise circular/linear progress and the Expressive
  loading indicator across single and stacked dialogs, covered parent dialogs,
  modal side/bottom sheets, and mixed dialog/sheet hosts. They also verify window
  focus, inactive-time exclusion, explicit pause, reduced motion, cancelled held
  presses, blocked typing/caret timers and continued availability of snackbar Undo.
- All three pass on software and wgpu/Metal. The initial tests reproduced the
  original freeze before the implementation change.
- All 233 ordinary tests pass in default and software-only builds (159 library,
  56 integration, 18 gallery). Existing ripple/focus and interaction regressions pass.
- All 39 strict reference functions pass with updates disabled in the canonical
  software build: 2,323 PNGs, including 42 new light/dark modal-progress frames.
  Twenty-four existing ripple frames now show advancing background progress;
  no references were removed. Representative nested-dialog and sheet images
  were visually inspected.
- Clippy with warnings denied, formatting and release example builds pass.
  The release GPU capture profile averages 6.31ms (median 5.61ms, p95 10.74ms,
  maximum 17.98ms, including screenshot readback). The earlier dialog shadow
  optimization is preserved; this is a headless rendering measurement, not a
  native-window frame-rate guarantee. Restart an already running gallery to
  load the rebuilt release executable.

Logs: `target/modal-activity-*.log`. Previous references are retained in
`target/modal-activity-before/references`.
[Review the timed progress frames](http://127.0.0.1:8766/?filter=modal-activity).

## Covered invoker feedback — 2026-09-14

A new regression reproduced the reported quick-FAB-click freeze before the fix:
the invoker's pixels still differed from idle after the dialog had fully opened.
Modal hosts now finish finite feedback with redraw-only updates while retaining
input isolation and suspending recurring indicators. The same test passes after
the fix for single, stacked and nested dialogs in both themes.

- 230 ordinary tests pass in default and software-only builds (156 library,
  56 integration, 18 gallery). Existing focus/caret restoration, modal isolation,
  held-gesture cancellation and idle redraw checks also pass.
- Five dialog regression tests pass on wgpu/Metal, including all three new tests:
  quick-tap completion, held-gesture cancellation, and continued scheduling when
  the dialog itself has zero-duration motion.
- All 38 strict reference functions pass in the canonical software-only build.
  There are 2,281 PNGs, including 30 new quick-tap frames across six host/theme
  combinations. Four nested-editor gallery references now initialize their
  covered native text values with the enabled foreground instead of leaving the
  initial disabled tint. These changes were visually reviewed; no images were removed.
- Clippy with warnings denied, formatting and release example builds pass.
- A release GPU capture profile still averages 5.34ms per frame (15.43ms maximum,
  including screenshot readback); the earlier shadow performance fix is preserved.

Logs are in `target/modal-ripple-*.log`; prior images remain in
`target/modal-ripple-before/references`. [Review the timed ripple frames](http://127.0.0.1:8766/?filter=modal-ripple).

## Dialog lag and label-surface correction — 2026-09-14

The release GPU profile at 1160×844 logical pixels and 2× scale captures 33
opening frames, including offscreen rendering and screenshot readback. Before
the fix it measured 79.17ms mean, 87.95ms median, and 94.27ms maximum per frame.
After the fix: 7.05ms mean, 6.49ms median, and 15.90ms maximum. This is a local
rendering comparison, not a native-window FPS or input-latency guarantee.
The entrance remains 500ms; no motion duration was shortened.

Reproduce with `ICED_TEST_BACKEND=wgpu cargo test --locked --release --example gallery dialog_animation_profile -- --ignored --nocapture`.
Logs: `target/dialog-profile-{before,after}.log`.

- 227 ordinary tests pass in default and software-only configurations
  (153 library, 56 integration, 18 gallery); 12 doctests also pass.
- Both new rendering/interaction checks pass on software and wgpu/Metal. They
  test shadow-raster reuse and size, fractional-position edges, clipping,
  opacity, the dropdown's host/label colors in both themes and modal hosts,
  and actual dropdown selection. A gallery test verifies the reported composition.
- 37 strict reference functions pass in both build configurations, with updates
  disabled: 2,251 PNGs, including 40 new timed dialog frames. Fifty existing
  dialog-motion references changed along their shadow edges; no references were
  removed. The previous references are retained in `target/dialog-fix-before`.
- Clippy with warnings denied, formatting, and release example builds pass.
- The corrected light/dark gallery and timed renders were visually inspected.
  A rebuilt native QA app launched successfully, but native UI automation lost
  its actionable window before opening the dialog. No new uninterrupted native
  animation or typing session is claimed; rendering and interaction verification
  use iced's real headless runtime and Metal checks above.

Review [the fixed dialog frames](http://127.0.0.1:8766/?filter=dialog-fix).
Implementation details: [BASELINE_COMPLETION.md](BASELINE_COMPLETION.md#dialog-regression-follow-up).

## Earlier validation

Current update: [desktop fidelity](DESKTOP_FIDELITY.md) completes baseline color roles and shared shadows, picker actions/borders, slider labels and progress refinement. Earlier milestone counts and platform checks below are historical.

Validated locally on 2026-09-12–14 on macOS / Apple Silicon with Rust 1.92.0 and released iced 0.14.0 (`iced_widget` 0.14.2, `iced_test` 0.14.0, as recorded in Cargo.lock).

## Executed checks

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --locked --all-targets -- -D warnings` | Passed, no warnings |
| `cargo test --locked --all-targets` | 213 tests passed; 34 reference test functions, 4 visual generators and 1 performance probe intentionally ignored by the normal suite |
| `cargo test --locked --doc` | All 12 README/API code examples passed |
| `cargo build --locked --release --examples` | Gallery and minimal application built with the default GPU renderer |
| `cargo test --locked --all-targets --no-default-features` | 213 tests passed with the software renderer |
| Visual reference comparisons | All 34 reference test functions passed in default and software-only builds; 2,043 images across 319 review groups |
| Earlier desktop wgpu/Metal checks | All 18 new component interaction/rendering tests passed |
| Earlier motion polish wgpu/Metal checks | All 14 new component interaction/rendering tests passed |
| Fidelity wgpu/Metal checks | All 6 new renderer/property tests and 5 targeted existing regressions passed |
| Desktop completion wgpu/Metal checks | All 8 new runtime tests and the existing long-title rendering regression passed |
| Desktop variants wgpu/Metal checks | All 7 new runtime/rendering tests passed |
| Desktop fidelity wgpu/Metal checks | All 5 new runtime/rendering tests and 3 ripple/menu/scroll regressions passed |
| Reference failure safeguards | Altered reference and missing reference correctly failed; CI baseline update was rejected; restored references passed again |
| Snackbar occlusion regression | Reproduced before the fix and passed afterward on Tiny Skia and wgpu/Metal, in both themes |
| `cargo doc --locked --no-deps` | Passed |
| Navigation visual generator | Executed separately on Tiny Skia and wgpu/Metal; 4 cases per renderer covering tabs, badges, long app-bar titles, lists, switches and icon-button variants |
| Workflow visual generator | Executed separately on Tiny Skia and wgpu/Metal; 10 cases per renderer, including light/dark menus, tooltips, snackbars, radios, edge placement and a dropdown inside a dialog |
| Milestone-one visual generator | Executed separately; light, dark, narrow field and dialog images inspected |
| Final gallery visual generator | Executed separately on Tiny Skia and wgpu/Metal; 16 cases per renderer, including Workspace/Activity/Settings, wide/dark/narrow layouts, minimum-dialog, checkbox-state and scrolled long-content cases |

Ordinary tests comprise 132 library tests, 56 integration interaction/rendering tests, and 16 gallery tests. The 12 doctests and 32 visual reference test functions are additional. The earlier visual generators create manual inspection artifacts. The reference suite asserts exact software pixels in its canonical environment, with a dedicated CI job; see [VISUAL_TESTS.md](VISUAL_TESTS.md). CI explicitly selects Tiny Skia for headless rendering while compiling both feature configurations.

## Behavioral coverage

- Finite transitions settle, handle zero duration, and retarget without a value discontinuity.
- Generated semantic color pairs maintain at least 4.5:1 contrast for black, white and saturated RGB accents in light and dark themes.
- Buttons emit once after a matching press/release. Release outside, orphan release and disabled buttons produce no action. View rebuilds preserve an active press; disabling cancels it.
- Button animations stop asking for redraws after settling. A real renderer pixel test verifies rounded corners and viewport clipping while pressed.
- Roboto is present in the actual renderer font database; repeated registration does not change its version.
- Checkbox pixel tests verify a circular hover halo in both selection states, intermediate selection/mixed frames, and no continued redraw requests after settling. Label/padded-target clicks, orphan releases, dragging away, focus loss and disabling during a press are covered. A scrolling regression checks checked, mixed and disabled marks at 2× scale.
- Checkboxes and switches emit the new application-owned Boolean value; disabled controls do not. Chip and icon-button messages are exercised too.
- The gallery's vector icon pixels are centered within their circular button outlines at 2× display scale. The test subtracts an empty outlined button and measures the painted icon bounds, independently of the widget layout box. The same pixels must remain visible and centered after scrolling.
- Native field typing includes Unicode, Backspace and Enter submission. Focus/selection survive a validation-message rebuild; disabling clears focus. An empty floating label settles after focus leaves.
- Dialogs capture background pointer, wheel and keyboard input even when dismissal is disabled. Actions remain functional. Escape/outside dismissal and drag-origin semantics are covered.
- Oversized dialogs scroll and their bottom action remains clickable at 320×480.
- Typing inside a dialog can relayout and render. After its label animation settles, a focused dialog field requests only future native caret blinks, not immediate animation frames.

## Visual and native checks

The gallery was launched as a native macOS application. Successful native observations included normal text entry with a floating label, light/dark switching, teal accent changes across components, narrow resize, scrolling, switches and checkboxes, dialog rendering, Escape dismissal, and text entry inside the dialog. Disabled and selected appearances were inspected. Native screen-reader semantics were not tested or claimed.

Final headless gallery cases include 1160px-wide light/dark layouts; 390px-wide light/dark layouts; wide and narrow dialogs; a 320×560 dialog with scrolling; and the bottom of the gallery at 320px width showing deliberately long labels, wrapped error text, disabled controls and custom button children. Dedicated light/dark checkbox panels cover hovered, checked, mixed, error and disabled appearances. A screenshot of the overview is checked in as [gallery.png](gallery.png). Full renders remain in `target/visuals/` and can be regenerated using the commands in [CONTRIBUTING.md](CONTRIBUTING.md).

Two findings changed the implementation during validation:

1. **Dialog typing performance:** dependency optimization alone did not resolve native lag; the user also reproduced it in release mode. A release probe at 1160×844 logical pixels and 2× scale measured roughly 0.09–0.14 ms for rebuilding/layout/recording each populated-note change, versus 14.7–16.2 ms for Tiny Skia painting only its damaged region. Upstream Tiny Skia 0.14 redraws intersecting surfaces and regenerates the dialog's large shadow even for a small text change. The crate now enables iced's wgpu renderer by default, retaining software fallback and a `--no-default-features` option. Native typing was verified with wgpu/Metal; a process sample confirmed that renderer and spent most of its time waiting for events. A spot CPU reading with the note focused was 0.8%. These are local observations, not a cross-platform latency guarantee. Reproduce the software measurement with `cargo test --release --no-default-features --example gallery software_dialog_redraw_profile -- --ignored --nocapture`.
2. **Small dialog scrollbar:** at 320px width, the default overlaid scrollbar covered the content's right edge. The dialog now reserves room for a slim scrollbar. The final minimum-size render was re-inspected after the change.

The gallery's plus, star and close symbols now use cached vector paths instead of system-font glyphs. Their square bounds avoid font baseline and line-height offsets, and their foreground follows the enclosing button's state. Both the native GPU preview and the software pixel regression test were inspected during this fix.

The earlier software preview sometimes lost its native UI automation connection. The updated GPU preview allowed opening the dialog, focusing the note, typing a full sentence and inspecting the centered icons. This is not an uninterrupted long-duration performance test. The independent headless tests remain reproducible without UI automation.

## Material comparison pass

The official M3 checkbox specs and live MUI checkbox demo were visually inspected,
including MUI’s circular hover feedback. Reference tokens were checked for type
roles, font weights, checkbox geometry/motion, tonal colors, dialog typography,
field supporting text and badges. The resulting audit is [MATERIAL_AUDIT.md](MATERIAL_AUDIT.md).

The full suite and both renderer configurations were rerun after bundling Roboto
and replacing the checkbox helper. The new checkbox tests use real 2× software
rendering and timed input events. The gallery render generator was rerun after the
font, badge and dialog-action changes; wide and narrow layouts, light/dark
checkboxes, and minimum-size dialogs were inspected. Inspection found and fixed disappearing checkmarks after scrolling with Tiny Skia.
Checkbox marks and gallery icons now use cached SVG handles; a regression first
reproduced the failure and then passed with the updated rendering path.
The previous overview is
preserved as [gallery-before-material-audit.png](gallery-before-material-audit.png).

A separate native macOS GPU preview rendered the new typography and controls.
Its automation connection rejected click attempts even after refreshing its
state, so new checkbox gestures were verified with the independent headless
interaction tests, not claimed as successfully automated native interactions.
Headless wgpu/Metal rendering was also run with device access after the SVG fix;
checkbox marks, icons, narrow scrolling and dark dialog output were inspected.
GPU and software alpha blending differ, especially in disabled states; these
artifacts are not expected to be pixel-identical. Earlier native typing/renderer
observations above remain historical results from the preceding performance fix.

## Everyday workflow milestone

The menus/selects, tooltips, snackbars and radios were checked against baseline
Material Web tokens; see [WORKFLOWS.md](WORKFLOWS.md). Eighteen new interaction tests
cover action/disabled/outside-click behavior; drag cancellation; Escape precedence
inside a dialog; long-menu scrolling and placement after ancestor scrolling;
context-menu left/right clicks; controlled radio values, intermediate animation
frames and disable-during-press behavior; tooltip delay, passive overlays,
suppression and deactivation; snackbar timeouts, IDs, pause/resume, covered drags
and background typing. A gallery test drives Duplicate and Undo through real
component messages and checks that an old dismissal cannot clear the new notice.
A placement unit test covers all sides and very small window bounds.

Validation found and fixed menu/tooltip visibility checks using unscrolled bounds,
covered snackbar releases leaving a background gesture pending, and snackbar
timers staying paused after a root dialog closed. New notices also retain a
window-deactivation pause across rebuilds. Popups retain their tree state across
ordinary view rebuilds; disabled menus close and clear a pending press.

Ten workflow renders and all eleven updated gallery renders were generated for
each backend. Inspection covered light/dark colors, radio hover/selection,
selected and disabled menu items, narrow snackbar wrapping, bottom-edge popup
flipping and dropdown layering above a dialog. The updated overview and two
workflow examples are retained under `docs/`.

The release gallery rendered in a separate native macOS preview. The automation
connection then reported no available window when attempting the first menu
click. Native gestures for this milestone are therefore unverified; the headless
interaction tests and GPU renders passed. The historical native observations
above are from earlier work. No Windows/Linux native run or screen-reader test
was performed.

## Navigation and structured content milestone

Nineteen component tests and two gallery flow tests were added. They cover
primary/secondary tab values, centered fixed-tab labels, narrow badges, horizontal
overflow, animated indicator frames/settling and translated rendering; rail
destination/disabled behavior, overflow scrolling and footer reachability; row
actions vs trailing buttons/checkboxes/menus, interrupted drags, wrapped content,
disable-during-press and child overlays; app-bar action reachability and title
clipping; root modal input blocking; switch label rendering, thumb-centered hover,
pressed thumb growth, idle scheduling and clipping; and icon-button toggle colors.
Gallery flows drive navigation, preserve preferences across widths and remove read
items from the Unread filter after application updates.

Sixteen gallery renders and four focused component renders were generated for
each backend. Inspected cases include light/dark views, 320/390px layouts, long
titles and supporting text, selected/disabled tabs and rows, switch hover/press,
icon variants, and the new rail. Visual inspection caught a switch label hidden by
an early return, a wrapped rail label, long app-bar text drawing over actions, a
squeezed tab badge, and composite tab content inheriting its parent's minimum
width. These were corrected and affected renders regenerated. The current
overview and narrow dark Activity page are retained under `docs/`.

A separate release app rendered the new gallery natively on macOS. Connecting to
that preview took unusually long; its screenshot was inspected, but the first
page-switch click returned `noWindowsAvailable`. Native gestures for this milestone
are therefore unverified. The final centering adjustment was verified through the
headless interaction tests and both renderer outputs. Earlier native typing and
theme-switching observations above belong to earlier milestones.

## Visual regression milestone

Added 323 reference frames covering every current component family in light/dark
themes and three gallery compositions. A private clock controls input and redraw
times for component fixtures, including switch thumb growth while held, checkbox
and radio selection, button release, tab indicators, floating labels and popup
deadlines. Independent switch sequences produce identical frames despite different
wall-clock origins. Assertions also check message emission and settled redraw
scheduling. All reference comparisons passed in separate default-feature and
software-only runs.

The reported **Workspace unpinned** corruption reproduced with background footer
text crossing the snackbar. Drawing the notice in its own foreground layer fixes
the renderer's text/quad batching order. A pixel test failed with the previous
implementation and passed after the fix on both Tiny Skia and wgpu/Metal. The
scrolled dark Settings reference now covers the original gallery situation.

Inspected typography, action variants, switch press and release frames, floating
labels, navigation/list menus, dialog layering and the fixed gallery composition.
The offline HTML report provides filtering, frame selection, filmstrips and
expected/actual/difference views. Its filter, held-switch slider and comparison
link were verified in the browser. Deliberately altered/missing references and a
CI update attempt all failed as intended; original references were restored and
compared successfully afterward.

The new visual CI job pins macOS 26 Arm64, Rust 1.92.0, the lockfile, bundled
Roboto, Tiny Skia and 2× scale, and uploads reports on success or failure. Remote
CI execution remains unverified. These checks use the actual headless runtime;
they do not add a new native gesture or cross-platform GPU verification claim.

## Checkbox/radio ripple refinement

The centered press ripple now expands over 450 ms with Standard easing, remains
visible during a long hold, and fades separately over 150 ms on release. Quick
clicks preserve visual feedback for a minimum 225 ms before fading without
delaying the action message. Three new normal tests cover sustained growth,
held/released pixels and idle scheduling; quick clicks, interrupted/repeated
gestures and disabling; and zero-duration motion. The hold/release test also
passed on wgpu/Metal for both controls in light and dark themes.

Added 64 reference frames for later expansion milestones and quick-click release,
bringing the suite to 387. Updated checkbox/radio frames were inspected against
the previous renders, and every reference group passed again in default and
software-only configurations. Other families' existing reference pixels matched
without updates. The full regular suites, doctests, formatting, Clippy and release
examples passed after the change. Native gesture automation was not repeated.

## Hover beneath the press ripple

Corrected the timing refinement's unintended hover fade: checkboxes and radios
now keep their full hover circle beneath the separate press ripple. Expansion,
minimum visible duration and release fade retain the refined timings. Updated
references show the persistent outer circle and additional inner press feedback.

The new `selection_ripple_preserves_hover_circle` pixel regression covers both
controls, both selection values and both themes. It verifies unchanged hover
pixels outside the growing ripple, added feedback inside it, return to hover on
release, clearing on pointer exit, and idle redraw scheduling. It passes with
Tiny Skia and wgpu/Metal. All 72 normal tests, 387 reference images, doctests,
formatting, Clippy and release example builds pass.

## Value controls and progress milestone

Added sliders, segmented buttons and both progress shapes, with the simulated
Export page and a continuous-slider component demo. Ten new library tests and
two gallery tests cover input, cancellation, selection, modal blocking, scroll
rendering, progress scheduling and the worker's stale-message protection. All
84 normal tests pass in default and software-only configurations; 8 doctests,
Clippy, formatting, documentation and release example builds also pass.

Added 158 reviewed reference images, bringing the total to 545. Both themes cover
slider held/drag/release states, connected single/multiple selection, determinate
transitions and indeterminate progress frames. Export is rendered idle, preparing
and running at widths 320, 390 and 1160. Other component reference pixels match
without updates; the existing Workspace reference adds its Export tab.

The selected-segment outline property failed before the explicit foreground
layer and passes after it. That property and the new controls' native scroll
translation comparison pass on Tiny Skia and wgpu/Metal. All 10 reference test
groups pass in both feature configurations. Visual review used real headless
renders; native window gestures and perceived real-time smoothness were not
rechecked for this milestone. Baseline M3 and the documented progress-motion
approximations are described in [VALUES.md](VALUES.md).

## Indeterminate linear progress correction

Replaced the identical half-cycle-offset sweeps with Material Web's actual
2000ms position/width keyframes and cubic-Bezier easing curves. The two segments
now stretch and contract independently and briefly coexist during their handoff.
The circular indicator retains its single growing/shrinking arc.

A pixel regression rejects the previous animation and passes with the correction
in both themes on Tiny Skia and wgpu/Metal. A separate numerical check verifies the reference
width extrema, contraction, valid geometry and the full two-second period.
Added 14 reference captures around handoff/exit, bringing the total to 559.
Only linear indeterminate frames and the six preparing-export compositions
changed; circular and other component references are unchanged. A 60-frame
playback preview uses actual software-rendered images in both themes.

After the correction, all 86 ordinary tests passed in default and software-only
builds, all 559 references passed in both configurations, and all 8 doctests
passed. Formatting, Clippy with warnings denied, and the release example build
also passed.

## Sheets and primary actions milestone

Added 13 library tests and two gallery workflow tests, with 358 new references
covering sheet opening/closing/reversal, narrow scrolling, FAB sizes/colors and
held/release states, and actual gallery compositions. All previous 559 baseline
files remain byte-for-byte unchanged. The whole reference suite now has 917 PNGs.

The tests caught and fixed cancellation of a held sheet button when an enclosing
dialog opens. Additional checks cover modal blocking throughout exit, nested-menu
Escape, matching outside gestures, typing, per-file notes, preserved scrollbar
position, standard/modal changes, invalid dimensions and paused snackbar timers.
The sheet tests were also run with `ICED_TEST_BACKEND=wgpu`, selecting Metal for
the occlusion, reversal, modal/standard interaction and held-FAB checks; other
fixtures deliberately keep canonical software rendering. All 13 tests passed.

Reviewed actual rendered light/dark gallery frames at 320, 390 and 1160px, including
scrolled sheet actions, the new-workspace dialog, FAB variants/held states and
intermediate sheet entrance/exit frames. Review found and fixed cramped file-row
text at 320px and the details list's mismatched background on modal sheets.
The release gallery was rebuilt. The new native gestures were not manually
rechecked in a live OS window; validation used iced's actual headless UI runtime
and the tested software/Metal rendering paths. See [SHEETS.md](SHEETS.md).

## Unverified or deliberately deferred

- Windows and Linux native execution, and GPU behavior outside the tested macOS/Metal preview. A cross-platform CI workflow is supplied but has not run on remote runners in this task.
- Rust 1.88 MSRV execution: that toolchain was not installed locally. The declared minimum matches iced; all local checks used 1.92.
- Exhaustive IME/composition, script/font fallback and clipboard behavior across platforms. Editing uses upstream iced; Unicode entry and native keyboard behavior were exercised locally.
- Native screen-reader integration, full touch and RTL handling, and localized calendar input. Automatic focus scrolling, roving groups and calendar row/week/month/year traversal are now implemented as described in DESKTOP.md and MOTION_POLISH.md, alongside the keyboard activation, modal focus trapping, contrast adjustment and reduced motion from COMPLETION.md.
- Complete M3/Expressive fidelity, including canonical loading shapes, width deformation, wavy progress and every new size token. HCT tonal-spot palettes now replace the earlier sRGB approximation.

There are no known failing automated checks in the delivered source. The project is unpublished and the working package name has not been checked for availability.

## Search and filtering milestone

Added search bars/views, four baseline chip families and range sliders, documented
in [SEARCH.md](SEARCH.md). The Files gallery now combines queries, Shared/Pinned
filters and an age range, shows local recent queries and empty results, and opens
result details. Component input chips demonstrate actual removal/restoration.

Fourteen new library behavior/rendering tests and three gallery tests cover the
new interaction paths. The library tests include native autofocus/editing/clear/
submit, nested Escape priority, outside gesture matching, input blocking through
search exit, long-result scrolling, interrupted/zero-duration motion, opaque
search surfaces, independent chip actions, coincident handle separation, snapped
range clamping and cancellation.

There are 166 new reference images: 48 search, 22 chip, 66 range-slider and 30
gallery frames. All 864 older component reference files remain byte-for-byte
unchanged. Nineteen older gallery references were updated for the changed Files
workflow and overview list. The suite now contains 1,083 PNGs (23.3 MiB) in 137
review groups. Light/dark, wide/narrow, selected/disabled and intermediate motion
frames were visually inspected.

The default and software-only ordinary/reference suites, documentation examples,
formatting, strict Clippy, warning-free docs and release example builds pass.
All 14 new library tests also passed with the wgpu/Metal renderer, including
search occlusion, query editing, chip removal and range dragging. Metal needed
execution outside the restricted sandbox to initialize its adapter.
This milestone was checked in iced's actual headless UI runtime; the native OS
window was not manually rechecked. Full accessibility and Expressive parity remain
outside the scope documented in SEARCH.md.


## Adaptive navigation and catalog completion pass

The full default and software-only ordinary suites pass: 144 tests, plus 10
passing doctests. All 22 reference functions pass in both configurations with
1,287 PNGs (30.5 MiB), 195 review groups and 101 gallery compositions. This pass
adds 204 references and intentionally updates all 1,083 preceding references
for the HCT color scheme and changed gallery. No preceding baseline is removed.

Twenty targeted tests also pass with wgpu/Metal: six adaptive navigation/focus
cases, five picker cases, and nine variant/carousel/Expressive cases. Regression
assertions cover the clock hand painting outside its dial, clicks escaping
carousel masks, held actions during FAB-menu exit, partial date-year pages,
keyboard focus/activation, and gallery draft validation.

Formatting, strict Clippy, documentation with warnings denied and optimized
example builds pass. Rendered light/dark, narrow/wide and intermediate animation
frames were inspected in the actual headless iced runtime. The new native OS
window interactions were not manually rechecked; Windows/Linux, the declared
Rust 1.88 minimum and remote CI remain unverified locally. Native screen-reader
support and unfinished subvariants are explicitly listed in [COMPLETION.md](COMPLETION.md).

## Desktop polish pass

Added 18 component tests and two gallery workflow tests. Both default and
software-only builds pass all 164 ordinary tests and all 24 reference functions;
the 12 README/API doctests pass as well. The new component tests also pass with
`ICED_TEST_BACKEND=wgpu`, using Metal with GPU access outside the restricted
sandbox. Formatting, strict Clippy, documentation with warnings denied and release
example builds pass.

The 88 new references comprise 72 component frames and 16 gallery compositions,
bringing the total to 1,375 PNGs (34.5 MiB), 215 review groups and 117 gallery
compositions. All 1,186 previous component references remain byte-for-byte
unchanged. Twenty-two older gallery frames changed intentionally for the added
calendar control and scrolling the selected Schedule/Export tab into view. No
previous reference was removed.

The tests cover remembered group focus, nested scrolling, targeted native field
focus, calendar row/week navigation, nested menu keyboard and hover behavior,
matching outside gestures, fixed full-screen dialog actions, date/range popup
lifetimes, rail reversal/reduced motion and modal blocking through exit. Gallery
tests exercise editor Save/Cancel and navigation selection. Visual review caught
and corrected rail header alignment, duplicate disabled-menu dimming and ancestor
focus outlines. Light/dark, wide/narrow, popup placement and intermediate motion
frames were inspected.

This pass used iced's actual headless UI runtime and software/Metal renderers.
The new gestures were not manually rechecked in a native OS window, and the
earlier native observations remain historical. Current scope and remaining
desktop/platform limits are documented in [DESKTOP.md](DESKTOP.md).

## Motion and desktop interaction pass

The default and software-only builds pass 179 ordinary tests, 12 doctests and all
25 reference functions. Fourteen new runtime/pixel tests also pass with Metal;
one additional unit test covers reducing motion during an active transition.
Formatting, strict Clippy, documentation with warnings denied and release example
builds pass. The local visual review page was regenerated and checked.

Added 132 frames: 128 new motion captures plus four settled frames extending
earlier tooltip/submenu sequences. The full set contains 1,507 PNGs (36.5 MiB),
227 groups and 117 gallery compositions. Of the preceding 1,375 references,
1,149 remain byte-for-byte unchanged and 226 changed for button ripple feedback,
popup lifecycles and adjusted settled capture times. No reference was removed.
The final default/software comparisons ran without baseline updates.

Behavioral checks include menu prefixes, repeated letters, Unicode, disabled rows,
modifier shortcuts, diagonal pointer intent, calendar month/year transitions,
disabled days and bounds, dialog reversal/cancellation/background isolation,
snackbar replacement, quick/held button feedback and reduced motion. Pixel checks
verify ripple opacity/origin/rounding, reveal clipping including shadows, and a
menu invoker's animation continuing while its panel is open.

Visual review found and corrected opaque SVG ripples and full-height shadows
escaping a partial menu reveal on Tiny Skia. A repeated gallery comparison also
caught the invoker's frozen ripple; it now receives redraw events while its popup
owns input. Light/dark motion frames, wide/narrow gallery popups and settled views
were inspected. The comparison now repeats exactly in both feature configurations.

Validation used iced's actual headless runtime with software and Metal rendering.
New native OS window gestures were not manually rechecked. Windows/Linux, native
screen-reader support and complete Material opacity/shape choreography remain
outside the verified scope. APIs, timing choices and remaining work are described
in [MOTION_POLISH.md](MOTION_POLISH.md).


## Baseline fidelity pass — 2026-09-13

The catalog audit and specific corrections are documented in [FIDELITY.md](FIDELITY.md).
Ordinary default and software-only suites each pass 185 tests; 12 doctests pass.
All 28 reference functions pass with both feature configurations, covering 1,595
images in 255 groups (117 gallery compositions). The pass adds 88 frames; of the
previous 1,507 images, 839 intentionally changed and 668 stayed byte-identical.
No reference was removed. A previous-reference copy and curated before/after page
were preserved locally before updating. Representative fields, focused buttons,
selection controls, cards, calendars, clocks and wide/narrow gallery renders were
visually inspected. These compare library revisions, not external screenshots.

Strict Clippy, formatting, warning-denied documentation and release builds of both
examples pass. Six new renderer assertions passed on Metal, plus five existing
ripple/clock/sheet/snackbar regressions. GPU creation failed inside the sandbox;
the checks passed when rerun with access to the Mac's graphics device. This is not
a native-window gesture test. Windows/Linux native behavior, remote CI and the
declared Rust 1.88 MSRV were not newly executed.

Logs: `target/fidelity-{all-tests,software-tests,doctests,clippy,doc-build,reference-comparison,default-reference-comparison,metal-tests,metal-regressions,release-build}.log`.
Review: [before/after](http://127.0.0.1:8766/fidelity-audit/) and
[timed state frames](http://127.0.0.1:8766/?filter=fidelity).

## Desktop completion pass — 2026-09-14

Default and software-only ordinary suites pass 194 tests (123 library, 56
integration and 15 gallery), plus 12 doctests. Strict Clippy, formatting,
warning-denied documentation and the default-renderer release examples pass.
Both feature configurations pass all 30 reference functions: 1,715 PNGs in 275
review groups, including 129 gallery compositions.

This pass adds 9 ordinary tests and 120 reference images. It updates 28 existing
layout/composition references for app-bar ellipsis, compact calendars and the
gallery stack, plus 16 focused-field references to replace wall-clock blinking
carets with deterministic text selections. No previous reference is removed.

Eight new runtime tests and the existing long-title regression also pass on
wgpu/Metal, including native caret/drag preservation, three-level focus return,
window deactivation, popup-first Escape, backdrop isolation, opaque layering and
manual date submission. The GPU checks run outside the filesystem sandbox, as
in earlier passes. No upstream iced fork or screen-reader bridge is introduced.

Representative light/dark title, picker and nested-dialog renders were inspected.
The [report](http://127.0.0.1:8766/?filter=desktop-completion) has 320/720px component
layouts, timed stack motion, and 390/1280px gallery compositions. These are local
headless runtime/rendering checks; this pass does not claim a new native-window
manual session, remote CI execution or Windows/Linux GPU validation.

Logs: `target/desktop-{all-tests,software-tests,doctests,clippy,doc-build,reference-comparison,default-reference-comparison,metal-tests,release-build}.log`.

## Desktop variants pass — 2026-09-14

Default and software-only ordinary suites pass **204 tests** (132 library, 56
integration, 16 gallery). All 12 doctests pass, as do formatting, warning-free
Clippy, documentation generation and both release examples. The seven new
runtime/rendering checks also pass on Metal; two scalar tests cover buffer
normalization and color continuity, and a gallery test covers time drafts.

Both feature configurations pass the 32 exact reference functions with updates
disabled. There are **1,921 PNGs**, 299 review groups and 145 gallery compositions
(48.3 MiB). This pass adds 206 images and changes 29 existing dropdown/gallery
compositions; 1,686 earlier references remain byte-identical and none were removed.
Reviewed light/dark adorned fields, field labels during motion, disabled icon
contrast, dropdown placement, 12/24-hour input, card states, lowered surface FABs
and the 390/1280px gallery compositions. The local report was rebuilt and its
HTTP response verified.

Logs: `target/variants-{all-tests,software-tests,doctests,clippy,doc-build,release-build,reference-comparison,default-reference-comparison,metal-tests}.log`.
The report is at [gallery variants](http://127.0.0.1:8766/?filter=gallery-variants).
The pinned baseline and remaining recipe/platform limits are documented in
[DESKTOP_VARIANTS.md](DESKTOP_VARIANTS.md); passing snapshots do not establish
complete Material 3 conformance.

## Desktop color and control fidelity (2026-09-14)

Nine new ordinary tests bring the total to **213**: 141 library, 56 integration,
16 gallery. Default and software-only configurations pass. The 12 doctests,
formatting, warnings-as-errors lint, public documentation and release examples
also pass. Five new renderer/runtime checks and three existing ripple, animated
menu and scroll-translation regressions pass on wgpu/Metal.

The 34 strict reference functions (25 component and 9 gallery) pass in both
feature configurations with updating disabled. There are **2,043 PNGs**, 319
review groups and 149 gallery compositions (50.7 MiB). This pass adds 122 images;
changed existing frames reflect the new shadows, picker borders, labels and
progress. Independent property checks and visual review accompany the updates.
The ripple corner test now isolates ripple rendering from the intentional
hover-shadow drop on press; the menu test allows the measured ambient extent
while continuing to reject content beyond the reveal.

Inspected light/dark wide/narrow gallery compositions, all elevation levels,
picker footers, slider hover/held frames, and circular determinate/growth states.
The before/after review is a comparison of library versions, not screenshots of
Google components; pinned primary sources are in DESKTOP_FIDELITY.md.

Run logs are under `target/finish-*.log`. The native gallery is rebuilt but no
new native-window interaction, screen-reader, Windows/Linux, MSRV or remote CI
verification is claimed.

## Baseline desktop motion and layout (2026-09-14)

Both default and software-only configurations pass **224 ordinary tests**:
151 library, 56 integration and 17 gallery. The 11 added checks comprise seven
renderer/runtime cases, three scalar motion/layout cases and one gallery case.
All 12 doctests pass, as do formatting, warnings-as-errors Clippy, documentation
with warnings denied, and optimized builds of both examples.

The seven new runtime checks also pass on Metal, together with the existing
checkbox/radio hover-ripple, stacked-dialog occlusion and animated-menu clipping
regressions. Picker gesture tests now locate the rendered labels instead of
assuming the old header coordinates. The regular calendar confirmation test has
enough room for the new header and footer. Progress tests wait for actual loading
completion and spring settling rather than assuming a 200ms measured update.

Both feature configurations pass all **36 strict reference functions** (26
component and 10 gallery), with reference updating disabled. There are **2,211
PNGs**, 345 review groups and 153 gallery compositions (52.6 MiB). This pass adds
168 images, updates 349 prior images and leaves 1,694 prior images byte-identical;
none were removed. These snapshots detect regressions in the reviewed library
profile; they do not certify full Material conformance.

Inspected light/dark checkbox selection and clearing, overlapping range labels,
chip avatars, loading handoff, rounded opening/closing menus and dialogs, date
headers and narrow gallery wrapping. The focused browser player was opened and
its theme switch and timestamp selection verified. The local report server was
restarted on its existing loopback port after finding it stopped.

The [review page](http://127.0.0.1:8766/baseline-completion/) includes saved-frame
playback and before/after comparisons. `tests/visual/report.py` rebuilds the
player through `baseline-preview.py`; before/after panels are optional when the
local pre-pass backup is unavailable, as in a clean CI checkout.

Logs: `target/baseline-{tests,software-tests,metal-tests,clippy,doctests,doc-build,release-build,software-reference-comparison,default-reference-comparison}.log`.
No new native-window manual session, Windows/Linux GPU run, screen-reader bridge,
MSRV validation or remote CI execution is claimed. Remaining recipe differences
are recorded in [BASELINE_COMPLETION.md](BASELINE_COMPLETION.md).
