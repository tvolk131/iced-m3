# Visual regression tests

Every current component family has reference-image coverage. The suite stores
3,047 PNGs across component states, timed animation frames and gallery
compositions. These complement the ordinary interaction tests. A passing
comparison means the rendered pixels match the reviewed reference; it does not
establish complete Material 3 compliance. This guide owns the current reference
coverage and procedure; [VALIDATION.md](VALIDATION.md) retains historical findings.

## Run and review

Use the [canonical environment](#reproducibility-and-limits), including Rust
1.92.0, for both comparisons and intentional updates:

```sh
cargo +1.92.0 test --locked --no-default-features --all-targets visual_references -- --ignored
python3 tests/visual/report.py
```

Open `target/visual-report/index.html` in a browser. Filter by component, move the
slider through saved frames, or expand **All frames** for a filmstrip. Each frame
links to expected, actual and difference images. Pink pixels identify differences.
The slider selects captured frames; it does not invent intermediate images.

Reference tests are explicitly ignored by the normal test command because exact
pixels require the canonical environment below. CI runs them in a dedicated job
and uploads the complete report, including when comparison fails. Ordinary
behavioral and rendering-property tests continue to run across the CI OS matrix.

Missing references and any changed RGBA pixel or image dimension fail the test.
Generating artifacts never accepts a new baseline. To intentionally update after
a design change, run locally:

```sh
UPDATE_VISUAL_REFERENCES=1 cargo +1.92.0 test --locked --no-default-features --all-targets visual_references -- --ignored
python3 tests/visual/report.py
```

Review the changed PNGs before including them with the source change. Run the
comparison command again without the update variable. Updates are rejected when
the `CI` environment variable is present. To limit an update, use a test-name
filter, for example `--lib visual_references_selection_animations -- --ignored`.
Preserve a previous report before updating if you want its old/new comparison;
an update run compares against the newly written baseline.

## Precisely timed animation frames

The harness in `tests/visual/harness.rs` drives iced's real `UserInterface`, widget
tree, layout, overlays and renderer. Mouse events use a scoped, thread-local test
clock; redraw events carry explicit timestamps. `frame()` and `capture(name)` do not
advance time, dispatch a redraw event or sleep. `capture_at(milliseconds, name)`
explicitly advances to an absolute test timestamp with one redraw before capture.
Rebuilding a view preserves its widget cache, so tests exercise
application-controlled values and ongoing gestures.

The clock override exists only under `cfg(test)` in the library's unit suite.
Production continues to use the system clock, with no public testing API,
application subscription or runtime dependency. Gallery references use the same
renderer harness for settled compositions; precise motion cases live in the unit
suite where that private clock is available.

For example, switch, checkbox and radio sequences use this schedule in both
themes, starting in both selection states:

| Phase | Captures and assertions |
| --- | --- |
| Hover | 75 ms and settled at 150 ms after pointer entry |
| Mouse held down | 0, 50, 100 and 150 ms after pressing; checkbox/radio expansion also at 250/350/450 ms; still held at 500 ms |
| Release and application update | 0, 50, 100, 200 and 350 ms after release |
| Pointer leaves | Settled pixels stay unchanged; no animation redraw remains scheduled |
| Disable during a press | Feedback clears and the subsequent release emits no action |
| Quick checkbox/radio click | 0/100/225/300/375 ms; minimum visible interval and fade while the application updates immediately |

Holding the switch verifies its enlarged thumb without toggling it. Separate
assertions check that holding does not emit a message and settled frames do not
keep requesting redraws. A normal test repeats a complete switch press sequence
with two independent clock origins and requires identical pixels.

Checkbox/radio ripple expansion uses Standard easing over 450 ms. Opacity stays
visible while held and fades over 150 ms after release; a quick click starts
fading no earlier than 225 ms after pressing. Normal rendering-property tests also
check a long hold, intermediate growth beyond 200 ms, immediate action delivery,
repeated presses, cancellation during the minimum interval, disabling, and zero
motion durations. Their settled state schedules no further animation frames.
An additional pixel regression checks both selection states and themes for
checkboxes and radios: the outer hover circle must stay unchanged while the
ripple expands inside it, release must return to the same hover appearance, and
pointer exit must clear both effects. It can also run against wgpu/Metal via
`ICED_TEST_BACKEND=wgpu cargo test --locked --lib selection_ripple_preserves_hover_circle`.

## Coverage

### Motion contracts beyond reference images

Approved screenshots can preserve a faulty animation. The spring solver can also
pass its tests while a widget's layout or painter clamps away the result. Test
the observable geometry or pixels of each distinct motion path, in addition to
reviewing its reference frames. Do not require every component to bounce: opacity
must stay bounded, clipping reveals have physical endpoints, and ripples and
continuous indicators have their own timelines.

The button-group regression in `tests/visual/expressive.rs` checks the actual
retained iced layout at 40/100/140ms and after settling, across 500px and 1100px
groups, all three positions, and connected/spaced variants. Independently defined
response ranges reject the early plateau even if someone approves new PNGs.
Release must move promptly and retain its small rebound; quick release/repress
must preserve position and velocity. Other checks cover 8/16/33ms and skipped
frames, reduced motion, narrow hosts, asymmetric padding, stronger custom springs,
width conservation, content space, disabled presses and idle redraws.

Proactive checks now also measure navigation-rail width and the tab indicator's
painted pixels through overshoot, reversal, reduced motion and settling. The tab
fixture sends its initial redraw before changing the selected value, so it tests
a retained transition instead of initializing directly at the new destination.
The rail check caught a shared transition bug that restarted settled springs;
the regression also exercises the consumer's short-circuit redraw pattern.

`tests/visual/switch_motion.rs` measures the painted thumb through selection
overshoot in both directions, immediate pressed sizing, cancellation/release
rebound, position/velocity continuity on reversal, keyboard activation, reduced
motion and disable. Separate checks require monotonic Fast effects colors,
identical frames at different redraw cadences and eventual idle scheduling.
Its 128 reference frames cover both schemes, themes, initial states and optional
icons. These checks fail if a draw-time clamp hides the spatial overshoot even
when the spring solver itself remains correct.

`tests/visual/interruption_contracts.rs` extends these checks across baseline,
Standard and Expressive motion. It measures floating-label ink and checks
reversal velocity, redraw cadence and retained rebuilds; exercises checkbox
mixed-state handoffs and selection cancellation; and changes reduced motion
during field, control, overlay and FAB-reveal transitions. Final frames must
match independently constructed resting widgets, with no further redraw work.
Dialog/sheet sequences include multiple open/close rebuilds before the next
event, nested coverage, pointer/key release and window unfocus. These caught
lost dialog cancellation flags and a remaining focus-gated sheet clock.
The checks use actual iced dispatch and rendered pixels without new goldens.
Run `cargo test --lib interruption_contracts`; use `ICED_TEST_BACKEND=wgpu`
with the default features for the same assertions on an available GPU.

For new motion or changes to existing motion, use this checklist:

- Define the expected response from design/reference evidence, independently of
  the implementation. Spatial overshoot, bounded effects, reveal clipping and
  continuous cycles need different assertions.
- Measure early motion, the expected peak, settling, and motion immediately after
  reversal. Check position continuity and continued momentum on early release.
- Exercise a compact layout, a realistic desktop layout, and a constrained layout;
  include endpoints, long content and overlapping transitions where relevant.
- Check reduced motion, cancellation/disable, retained view rebuilds, cadence
  independence and eventual idle scheduling. Continuous indicators instead need
  explicit pause, viewport and window-focus policy checks.
- Add a small light/dark filmstrip around the important transition times and
  review it in motion. Golden updates alone are not evidence of correctness.

The interruption suite covers the first selection/label and overlay/reveal
sequences. Further property coverage can extend slider gestures, continuous
cycles and combinations such as nested scrolling during a reveal. Targeted
regressions below remain in place; this is not yet a uniform contract suite for
every component or motion scheme.

### Reference families

| Family | Reference cases |
| --- | --- |
| Typography | All 15 roles, including font weight and size, in both themes |
| Surfaces, divider, badges | Three surface variants, divider, zero/small/overflow counts |
| Buttons | Five baseline variants at rest/disabled and with keyboard focus; shared hover, held and release motion |
| Icon buttons and legacy chips | Variants, selection and disabled combinations; shared button feedback |
| Switches, checkboxes, radios | Timed hover/press/release, both initial values, disabled during press; mixed/error checkbox examples |
| Fields | Floating label moves up and back at fixed times; filled, error, wrapped supporting text, disabled and secure states |
| Tabs | Primary/secondary, icons, badges and disabled items; moving indicator at 0/50/100/200 ms |
| Rail, app bar, lists | Selection, hover, long title ellipsis, supporting text, child controls and a trailing menu |
| Menus, context menus, selects | Edge placement, selected/disabled/destructive items, shortcuts and a select above a dialog |
| Dialogs | Narrow composition, nested popups and snackbar layering; fixed actions during entrance, scrolling, resizing and action-row wrapping |
| Tooltips | Just before the 500 ms delay, visible at 500 ms, dismissed by pressing |
| Snackbars | Opaque surface over background text, narrow action/wrapping, 3999 ms before timeout and 4000 ms dismissal |
| Sliders | Continuous/stepped, hover/press, held mid-drag, release, endpoints and disabled states in both themes |
| Segmented buttons | Single/multiple selection, icon/check replacement, hover/held/selection-color transitions, disabled items/groups |
| Progress | Linear/circular, empty/complete, 0/50/100/200ms determinate transitions; indeterminate frames through 2666ms and pause |
| Sheets | Standard/modal side and modal bottom; timed entrance, exit, interrupted/reversed motion; 320/390px long-content layouts in both themes |
| FABs | Small/regular/large/extended in all four colors; idle, intermediate hover, hover, 500ms held, intermediate release, released and disabled |
| Search | Docked/full-window expansion and collapse at fixed times, empty and populated queries in both themes |
| Chip families | Assist/suggestion/filter/input; hover, held, selection transition, remove held and disabled |
| Range slider | Separated/coincident/endpoints; hover, held bubble, drag, release and disabled |
| Adaptive navigation and pickers | Breakpoint layouts, keyboard focus/held keys, app-bar collapse/ellipsis, date/range/year/manual entry, 12/24-hour time drafts and picker actions |
| Expressive and carousel | Action sizes/shapes, group press/rebound, switch springs, rail width, FAB-menu lifecycles, all carousel layouts/masks/snapping and sheet resizing |
| Loading and waves | Canonical morphs, linear/circular waves, extrema, pause/resume, measured/loading handoffs, buffers and multicolor cycles |
| Gallery | Workspace, Activity, Settings, Export; file details, modal side/bottom sheets, scrolled notes/actions, FAB overview and New workspace; search suggestions/results, combined filters and empty results at 320/390/1160px |

The existing interaction tests cover additional cancellation, focus, editing,
scrolling and message semantics. There is no attempt to snapshot every possible
combination. Shared rendering paths receive focused animation coverage instead
of duplicating every frame for each helper built on them.

### Finding the detailed captures

Use these prefixes in the visual report to inspect the scenarios condensed in
the family table. The linked guides retain their design sources and adaptations.

| Report prefix | Distinct scenarios and guide |
| --- | --- |
| `sheet-*`, `fab-*`, `gallery-sheets` | Sheet interruption, nested menus, scrolled content and FAB sizes/colors; [sheets and FABs](SHEETS.md) |
| `search-*`, `chip-variants`, `range-slider-*` | Docked/full search, empty/populated queries, independent chip removal and overlapping range handles; [search and filtering](SEARCH.md) |
| `adaptive-*`, `picker-*`, `completion-*` | Breakpoint layouts, held keyboard focus, picker modes, carousel masks and FAB-menu exit; [catalog completion](COMPLETION.md) |
| `desktop-*` | Three-level/edge-flipped menus, full-screen fixed headers and scrolled bodies at 320/840px, calendars above/below their triggers, modal rail entry/exit; [desktop behavior](DESKTOP.md) |
| `polish-*` | Basic/full-screen dialog translation, menu type-ahead, button feedback, tooltip delay and both snackbar visibility directions; [motion](MOTION_POLISH.md) |
| `fidelity-*` | Normal/error filled/outlined field hover/focus/disable, selection-control keyboard focus with optional switch icons, cards and chips; [source audit](FIDELITY.md) |
| `desktop-completion-*`, `gallery-desktop-completion` | Shaped title ellipsis, three-level dialog stacks, manual-date validity and discard confirmation; [desktop completion](DESKTOP_COMPLETION.md) |
| `variants-*` | Narrow/error/disabled adorned fields, held/released trailing actions, empty/error dropdowns, 12/24-hour drafts, card drag states, lowered FABs, independent buffer and color transitions; [variants](DESKTOP_VARIANTS.md) |
| `finish-*`, `gallery-finish` | Elevation levels, picker footers/errors, slider hover/hold, progress extrema and 400/1100px compositions; [desktop fidelity](DESKTOP_FIDELITY.md) |

The following are **capture offsets**, in milliseconds from each phase's start,
not prescribed component durations. Fixtures live under `tests/visual/` and run
in both themes; the source arrays remain authoritative when adding samples.

| Fixture | Sampled phases |
| --- | --- |
| `search.rs`: search | Expand 0/50/100/200/300; collapse 0/50/125/200/250 |
| `search.rs`: chips/range | Chip selection 0/50/100/200; range held 0/50/100/150 |
| `desktop.rs`: rail width | Expand and collapse 0/50/100/200/300 |
| `polish.rs`: button ripple | Held 0/50/100/225/350/450/600; release 0/50/100/150/300 |
| `polish.rs`: menu | Open 0/50/100/200/350/500; close 0/50/100/150/250 |
| `polish.rs`: tooltip | Open 0/50/100/150/300 after the 500ms hover delay |
| `variants.rs`: multicolor progress | Multiple cycles sampled through 7800ms |

## Targeted rendering and interaction checks

In addition to the reference families above, ordinary tests independently check
properties that an accidentally approved image could otherwise hide:

- **Values/progress** (`values.rs`, `variants.rs`, `loading_progress.rs`): drag
  rebuilds, step deduplication, clamps/cancellation, invalid ranges, controlled
  selection, modal blocking and idle scheduling. Independent clock origins must
  produce identical frames; paused/clipped/reduced-motion indicators stop
  scheduling, and resuming excludes paused time. Visible indicators keep moving
  after keyboard window-focus loss, as checked in `interaction_contracts.rs`.
- **Paint order and scroll translation:** the selected-segment outline remains
  visible; sliders, check/icon segments and circular progress retain pixels after
  a 60px native scroll. `snackbar_surface_occludes_background_text` compares the
  notice interior with and without text behind it in both themes. Its
  [scrolled gallery reference](../tests/visual/references/gallery/settings-dark-snackbar.png)
  reproduces the reported Workspace unpinned composition.
- **Sheets and FABs** (`sheets.rs`): continuous reversal, no idle redraws, blocking
  through exit, cancellation of covered presses, matching outside-click origins,
  nested-menu Escape, native typing, scroll retention, standard/modal changes,
  invalid dimensions, root-dialog precedence and paused snackbar deadlines.
- **Search/chips/ranges** (`search.rs`): native autofocus/edit/clear/submit,
  exit-time input capture, nested menus, scroll/reopen, reversible/zero motion,
  opaque surfaces, independent chip removal, coincident handles, extreme/snapped
  ranges and gesture cancellation. Reference inputs are unfocused before capture
  to exclude native caret blinking; autofocus/typing retain behavioral checks.
- **Navigation/pickers/carousels** (`adaptive.rs`, `pickers.rs`, `desktop.rs`,
  `polish.rs`, `completion.rs`): group entry/memory, nested focus scrolling, native
  ID-based focus, calendar row/week/month/year traversal, bounds/disabled days, menu
  keyboard/hover behavior and edge placement. Dial hands must stay in the dial,
  carousel children must reject masked clicks, and closing FAB menus cancel holds.
- **Popup motion** (`polish.rs`): menu prefixes/repeated letters/Unicode,
  disabled rows, bounded diagonal pointer intent, calendar focus restoration,
  reversal/cancellation and idle redraws. Pixel probes measure ripple opacity,
  pointer origin and preserved hover/rounding; nested content and shadows must
  stay inside an animated reveal, and a covered invoker's ripple must finish.
- **State fidelity** (`fidelity.rs`, `desktop_completion.rs`, `variants.rs`,
  `desktop_fidelity.rs`): independent color swatches, spatial bounds, focus without
  activation, reduced motion, hover elevation returning to rest during a hold,
  stacked-dialog caret/focus return, picker validation/actions, adorned fields,
  dropdowns, card states, lowered FABs, buffer transitions and color continuity.

The paths above are under `tests/visual/`. Focused properties have also been run
on Metal; [VALIDATION.md](VALIDATION.md) records which investigations did so and
their limits. For example, on a machine with an available GPU:

```sh
ICED_TEST_BACKEND=wgpu cargo test --locked --lib snackbar_surface_occludes_background_text
ICED_TEST_BACKEND=wgpu cargo test --locked --lib visual_tests::search
```

### Linear progress motion

`linear_indeterminate_pixels_grow_contract_and_handoff_between_segments` checks
actual colored runs: early single segment, growth, contraction, brief two-segment
handoff, exit and repeat. It rejects the old symmetric sweeps. References include
1100–1250ms handoff frames and later contraction/exit. Generate a 60-frame looping
preview in both themes with:

```sh
WRITE_PROGRESS_PREVIEW=1 cargo test --locked --no-default-features --lib linear_indeterminate_pixels
```

Open `target/visual-report/progress-motion/index.html` for playback/scrubbing.
These extra captures are review artifacts, not additional golden references.

### Fixed dialog footers

`dialog-fixed-actions-{light,dark}` captures action entrance at 149/225/275ms,
a settled 400px panel, scrolling, a short window and wrapped narrow footer actions.
Behavioral checks require body-first traversal, offscreen focus reveal, scrollbar
gutter clearance, nested select dismissal, fixed actions after resizing and
blocked exit-time activation in both `modal` and `stack`. The independent desktop
consumer also scrolls, resizes and saves a settings form through this API.

## Reproducibility and limits

References were captured on Apple Silicon, macOS 26.6.2, Rust 1.92.0, with the
checked-in Cargo.lock, bundled Roboto, Tiny Skia and a fixed 2× scale. Visual CI
uses macOS 26 Arm64 with the same compiler, lockfile, fonts, renderer and scale.
OS image updates can still affect rendering; inspect differences before refreshing
references. Exact cross-platform or cross-renderer equality is not promised.
GPU blending is deliberately not compared to software golden pixels.

The private clock controls Material transitions, progress loops and popup timers,
not every upstream/system clock. Native caret blinking, IME and platform font
fallback need separate checks. Floating-label references change controlled values
without input focus; focused-field fixtures select text to suppress the blinking
caret. These captures verify sampled states, not real-time frame rate or perceived
smoothness. Native gesture, accessibility and platform limits are recorded in
[VALIDATION.md](VALIDATION.md) and [LIMITATIONS.md](LIMITATIONS.md).

Add fixtures under `tests/visual/`, drive real input through `Harness`, and keep
messages, controlled-value rebuilds and settling assertions explicit. Use
`capture` for the current frame or `capture_at` to advance time before capture;
retain explicit rendering when inspecting/cropping pixels. Generate and visually
review references deliberately. Snapshots do not replace interaction assertions.

## Manual inspection generators

Earlier generators remain useful for broad scene inspection:

```sh
cargo test --locked --no-default-features --test interactions milestone_one_snapshots -- --ignored
cargo test --locked --no-default-features --example gallery gallery_snapshots -- --ignored
cargo test --locked --no-default-features --test workflows workflow_snapshots -- --ignored
cargo test --locked --no-default-features --test navigation navigation_snapshots -- --ignored
ICED_TEST_BACKEND=wgpu cargo test --locked --example gallery gallery_snapshots -- --ignored
```

They write to `target/visuals/`; their outputs are inspection artifacts, not
portable pixel assertions. For canonical loader/wave playback and its capture
commands, see [LOADING_PROGRESS.md](LOADING_PROGRESS.md#review-and-tests).
For benchmarks with matched conditions and explicit timing boundaries, see
[PERFORMANCE.md](PERFORMANCE.md). For generated rustdoc previews, see
[DOC_MEDIA.md](DOC_MEDIA.md); those images are generated separately and stay out
of Git.
