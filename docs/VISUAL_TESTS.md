# Visual regression tests

Every current component family has reference-image coverage. The suite stores
2,665 PNGs across component states, timed animation frames and gallery
compositions. These complement the ordinary interaction tests. A passing comparison means the rendered pixels match the
reviewed reference; it does not establish complete Material 3 compliance.

## Run and review

```sh
cargo test --locked --no-default-features --all-targets visual_references -- --ignored
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
UPDATE_VISUAL_REFERENCES=1 cargo test --locked --no-default-features --all-targets visual_references -- --ignored
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
clock; redraw events carry explicit timestamps. Taking a screenshot does not
advance time, dispatch a redraw event or sleep. Rebuilding a view preserves its
widget cache, so tests exercise application-controlled values and ongoing gestures.

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

| Family | Reference cases |
| --- | --- |
| Typography | All 15 roles, including font weight and size, in both themes |
| Surfaces, divider, badges | Three surface variants, divider, zero/small/overflow counts |
| Buttons | All four variants; idle, hover, held, release and disabled states |
| Icon buttons and legacy chips | Variants, selection and disabled combinations; shared button feedback |
| Switches, checkboxes, radios | Timed hover/press/release, both initial values, disabled during press; mixed/error checkbox examples |
| Fields | Floating label moves up and back at fixed times; filled, error, wrapped supporting text, disabled and secure states |
| Tabs | Primary/secondary, icons, badges and disabled items; moving indicator at 0/50/100/200 ms |
| Rail, app bar, lists | Selection, hover, long title ellipsis, supporting text, child controls and a trailing menu |
| Menus, context menus, selects | Edge placement, selected/disabled/destructive items, shortcuts and a select above a dialog |
| Dialogs | Narrow composition and layering above a snackbar, with a nested popup |
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
| Gallery | Workspace, Activity, Settings, Export; file details, modal side/bottom sheets, scrolled notes/actions, FAB overview and New workspace; search suggestions/results, combined filters and empty results at 320/390/1160px |

The existing interaction tests cover additional cancellation, focus, editing,
scrolling and message semantics. There is no attempt to snapshot every possible
combination. Shared rendering paths receive focused animation coverage instead
of duplicating every frame for each helper built on them.

## Value and progress regression checks

`tests/visual/values.rs` adds 158 reference images and checks dragging across
rebuilds, step deduplication, endpoint clamps, cancellation, disabled/invalid
input, root-dialog blocking, controlled single/multiple selection and idle frame
scheduling. Indeterminate frames must be identical under two independent clock
origins. Paused, unfocused, invisible and zero-motion indicators stop requesting
animation frames, and resuming excludes the paused interval.

A selected-segment border regression failed before the foreground outline layer
was added and passes afterward. A scroll comparison verifies unchanged pixels
for the slider, check/icon segments and circular progress under a 60px native
scroll translation. Both rendering properties also pass on wgpu/Metal. The
reference PNGs remain canonical Tiny Skia output, not cross-renderer goldens.

The linear motion regression checks actual colored pixel runs: early single
segment, substantial growth, contraction, a brief two-segment handoff, exit and
repeat. It rejects the old symmetric animation. Reference captures now include
the 1100–1250ms handoff and later contraction/exit frames. To regenerate a looping
preview of 60 real renderer frames in both themes:

```sh
WRITE_PROGRESS_PREVIEW=1 cargo test --locked --no-default-features --lib linear_indeterminate_pixels
```

Open `target/visual-report/progress-motion/index.html` for playback and frame
scrubbing. These extra preview images are review artifacts, not golden references.

## Reproducibility and limits

The sheets/FAB milestone adds 358 references. Ordinary tests separately verify
continuous reversal, no idle redraws, modal blocking through exit, cancelled
covered presses, outside-click origins, nested-menu Escape, native typing,
scroll retention, standard/modal changes, dimension validation, root-dialog
precedence and suspended snackbar timers. Sheet occlusion, reversal and held FAB
rendering are also checked on Metal through `ICED_TEST_BACKEND=wgpu`; the golden
suite always selects Tiny Skia. See [SHEETS.md](SHEETS.md) for the component scope.

References were captured on Apple Silicon, macOS 26.6.2, Rust 1.92.0, with the
checked-in Cargo.lock, bundled Roboto, Tiny Skia and a fixed 2× scale. The visual
CI job uses the macOS 26 Arm64 runner and the same compiler, lockfile, fonts,
renderer and scale. It is configured but has not yet executed on a remote runner.
OS image updates can still affect rendering; inspect such differences before
refreshing references. Exact cross-platform or cross-renderer equality is not
promised. GPU blending is deliberately not compared to software golden pixels.

The clock controls this library's finite transitions, progress loops and popup timers. It does
not replace every upstream iced/system clock: native caret blinking, IME and
platform font fallback need separate checks. Floating-label snapshots change the
controlled value without focusing the native input, avoiding caret variability.
These snapshots verify sampled frames and state transitions, not real-time frame
rate or perceived animation smoothness.

To add coverage, put a fixture in `tests/visual/mod.rs`, drive real input through
`Harness`, capture meaningful milestones with `capture`, and assert messages and
settling where relevant. Generate and visually review its references explicitly.
Do not replace interaction assertions with snapshots alone.

## Snackbar overlap regression

The reported **Workspace unpinned** corruption was caused by text and rectangle
batching in a shared rendering layer. Drawing the snackbar after its background
was insufficient: background text could still paint over the snackbar surface.
`snackbar::Host::draw` now draws the complete notice in a separate foreground
layer, following iced's own stack layering pattern.

`snackbar_surface_occludes_background_text` compares the snackbar interior with
and without text behind it. It failed before the fix on both Tiny Skia and
wgpu/Metal and passes afterward in light and dark themes. Run its GPU check on a
machine with an available graphics device:

```sh
ICED_TEST_BACKEND=wgpu cargo test --locked --lib snackbar_surface_occludes_background_text
```

The [fixed Settings composition](../tests/visual/references/gallery/settings-dark-snackbar.png)
also captures the original scrolled-gallery situation.

## Search and filtering regression checks

`tests/visual/search.rs` adds 136 timed component references, with another 30
gallery compositions. Search expands at 0/50/100/200/300ms and collapses at
0/50/125/200/250ms. Chip selection uses 0/50/100/200ms and range-slider held
feedback uses 0/50/100/150ms. Ordinary tests cover native query editing, explicit
clear/submit, modal input capture, nested menus, scrolling/reopening, reversible
and zero-duration motion, independent input-chip removal, overlapping handles,
invalid/extreme ranges and gesture cancellation.

Search inputs are explicitly unfocused by the reference harness before pixel
capture so native caret blinking cannot make goldens nondeterministic. Autofocus
and typing have separate behavioral tests. See [SEARCH.md](SEARCH.md) for scope.
The 19 updated older references are gallery compositions affected by the new file
workflow; every pre-existing component reference remains byte-for-byte unchanged.

Run the new behavioral/rendering-property cases with Metal using
`ICED_TEST_BACKEND=wgpu cargo test --lib visual_tests::search`. All 14 cases passed
on this machine. Golden comparisons remain pinned to Tiny Skia.


## Adaptive navigation and catalog completion references

This pass adds 204 frames for adaptive navigation, keyboard focus and held keys,
app-bar collapse, date/range/year and time pickers, filled fields, extended FABs,
Expressive previews, carousel masks/snapping, FAB-menu entrance/exit and sheet
resizing. The complete suite has 1,287 references, including 101 gallery
compositions, in 195 review groups. All 22 reference test functions pass with
both feature configurations; golden rendering remains pinned to Tiny Skia.

All 1,083 preceding references were intentionally updated after replacing the
approximate accent scheme with HCT tonal-spot colors. New gallery compositions
also include adaptive navigation and the Schedule workflow. No preceding
reference was removed. Light/dark and wide/narrow results, picker hand placement,
carousel cropping, app-bar collapse and menu/sheet motion were visually reviewed.

Twenty new interaction/rendering cases also pass on wgpu/Metal. In particular,
clock-hand pixels must remain inside the dial, carousel children cannot receive
clicks beyond their visible mask, and closing FAB menus cancel held child actions.
These assertions complement exact reference comparisons. They do not measure
real-time smoothness or establish full Material/Expressive conformance. See
[implemented behavior and remaining boundaries](COMPLETION.md).

## Desktop polish references

This pass adds 72 component frames and 16 gallery compositions in light and dark
themes. Cascading menus cover keyboard entry, three levels, held actions and edge
flipping. Rail width changes are captured at 0/50/100/200/300ms in each direction;
modal rails include intermediate entrance and exit frames. Full-screen dialogs
cover fixed headers and scrolled bodies at 320 and 840px. Docked calendars are
captured above and below their triggers. The gallery adds 390 and 1280px views of
the editor, navigation panel, submenu and docked calendar.

All 1,186 preceding component references remain byte-for-byte unchanged. Twenty-two
older gallery frames were updated for the new calendar control and automatic
scrolling to the selected Schedule/Export tab. No preceding reference was removed.
The complete suite has 24 reference functions; reference pixels use Tiny Skia in
both build configurations.

Eighteen new ordinary component tests cover one Tab stop per group, remembered
entry, nested scroll reveal, native ID-based focus, calendar rows/week navigation,
menu hover timing and dismissal, dialog actions, docked date/range selection,
reversible/reduced-motion rails and modal isolation through exit. Two gallery tests
exercise editor Save/Cancel validation and closing navigation after selection.
See [desktop behavior and remaining limits](DESKTOP.md).

## Motion and interaction polish references

The motion pass adds 128 precise frames for button ripples, menu opening/closing,
basic/full-screen dialogs, plain tooltips and snackbars in both themes. Four more
frames extend existing tooltip and flipped-submenu sequences through their new
settled times. Every one of the preceding 1,375 reference files is retained.
There are now 25 reference test functions.

Button captures include held 0/50/100/225/350/450/600ms and released
0/50/100/150/300ms. Menus include 0/50/100/200/350/500ms opening, type-ahead focus,
and 0/50/100/150/250ms closing. Dialogs cover opening/closing and full-screen
translation. Tooltip timing begins after its 500ms delay. Snackbar sequences
include both visibility directions with stable background content.

Ordinary pixel tests independently assert ripple opacity, pointer origin,
preserved hover/rounding, clipping of nested layers and shadows at a menu's
animated edge, and an invoker finishing its ripple while a menu remains open.
Keyboard tests exercise menu prefixes/repeated letters/Unicode, disabled rows,
bounded diagonal pointer intent, calendar month/year changes and restored focus.
The retained surface tests cover reversal, cancellation, input blocking and idle
redraws. See [motion implementation and limits](MOTION_POLISH.md).


## Baseline fidelity state references

`fidelity-*` adds 88 frames in 28 light/dark groups: normal/error outlined and
filled field hover/focus/disabled states; five button variants and keyboard focus;
checkbox/radio/switch focus (including switch icons); and cards/chips. Existing
selection, picker, sheet, navigation and gallery references were reviewed and
updated for the corrected recipes. Six ordinary renderer tests compare actual
state colors with reference swatches and verify spatial bounds, focus without
activation, reduced motion and hover elevation returning to rest during a hold.
These assertions run on both Tiny Skia and Metal; exact PNG comparisons continue
to use Tiny Skia. See [the source audit](FIDELITY.md) for token versions and limits.

## Desktop completion references

The `desktop-completion-*` groups add 120 images for shaped title ellipsis,
three stacked dialogs with timed opening/closing, compact calendars, valid and
invalid manual entry, and gallery discard confirmations. The focused fidelity
field frames now use a fixed text selection: native iced caret blinking uses
wall time, while Material motion uses the harness clock. See
[desktop completion](DESKTOP_COMPLETION.md) for scope and behavior.

## Desktop variants follow-up

The [desktop variants pass](DESKTOP_VARIANTS.md) adds 206 references: 190 component
frames and 16 gallery compositions. Field groups cover filled/outlined icons,
affixes, floating labels, errors, disabled/narrow states and a held/released
trailing action. Dropdown groups cover default/focus/open/disabled/empty-error
states. Numeric time entry covers 12/24-hour valid, focused and invalid drafts.
Card groups cover all three surfaces in resting/disabled/dragged states; FAB
groups compare regular/lowered elevation and the lowered surface color. Buffer
frames exercise an independent finite transition, and four-color indicators
cover multiple cycles through 7800ms.

Both feature configurations pass all 32 reference functions. The 29 existing
images that changed are dropdown and gallery compositions affected by the new
field trigger or the clock toggle. The remaining 1,686 earlier images are
byte-identical. No references were removed. Native focused inputs use a selected
value in reference frames so their wall-clock caret cannot destabilize captures.

## Desktop fidelity references

The `finish-*` groups add 118 component frames; `gallery-finish` adds four
400/1100px light/dark compositions. They cover elevation levels, picker actions
and errors, slider hover scaling/held feedback, and progress extrema and motion.
The full suite now has 34 strict reference functions. See
[DESKTOP_FIDELITY.md](DESKTOP_FIDELITY.md) for sources and renderer assertions.
