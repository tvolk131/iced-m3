# Value controls and progress

Current follow-up: [baseline desktop motion/layout](BASELINE_COMPLETION.md) supersedes the historical range-label, progress timing/completion, selection animation, basic-overlay and picker-header limitations below.

Current follow-up: [desktop fidelity](DESKTOP_FIDELITY.md) supersedes the historical shadow, color-role, picker-action, slider-label and progress limitations below.


Current update: [adaptive navigation and catalog completion](COMPLETION.md) supersedes earlier milestone notes about keyboard support, HCT palettes, pickers, sheet dragging, FAB expansion and Expressive previews.

This milestone adds single-thumb sliders, connected outlined segmented buttons,
and linear/circular progress. The gallery's Workspace → Export page combines
them in a cancellable simulated export. Values and work belong to the application;
animation never requires an application message or subscription.

## Visual references and scope

The library continues its baseline Material 3 visual system. References checked
on 2026-09-13:

- [M3 sliders](https://m3.material.io/components/sliders/specs) and
  [Material Web slider tokens](https://github.com/material-components/material-web/blob/main/tokens/versions/v0_192/_md-comp-slider.scss):
  4px track, 20px round thumb, 40px state layer, 28px value bubble and Label Medium.
- [M3 segmented buttons](https://m3.material.io/components/segmented-buttons/specs)
  and [baseline tokens](https://github.com/material-components/material-web/blob/main/tokens/versions/v0_192/_md-comp-outlined-segmented-button.scss):
  40px height, 1px connected outline, rounded outer ends, 18px icons, Label Large,
  secondary-container selection and semantic disabled colors.
- [Material Web progress](https://material-web.dev/components/progress/): linear
  and circular shapes, each with determinate and indeterminate operation. The
  default linear track is 4px; the circular container is 48px.

These are baseline controls, not the newer Expressive slider/progress treatments.
The implementation uses original Rust code and the existing iced dependency.
No upstream source was copied or new runtime dependency added.

## Sliders

`slider(range, value)` accepts `f32` values and a finite increasing inclusive
range. Missing `on_change` or `.disabled(true)` makes it inert. Out-of-range values
are clamped and NaN becomes the minimum. Empty, reversed or non-finite ranges
render an inert control. Invalid steps fall back to continuous input.

`.step(amount)` snaps from the minimum; both endpoints remain reachable even when
the range is not evenly divisible. `.ticks(true)` shows discrete stops when they
fit at least 4px apart; very dense stops are omitted while snapping still works.
`.labeled(true)` shows the numeric bubble while held, and `.value_label(text)`
provides units or custom formatting. Bubble space is reserved even when hidden.
Long labels are clipped inside the bubble instead of painting over neighbors.

A left press updates immediately, dragging can continue outside the track with
endpoint clamping, and release sends the final change before optional
`on_release`. Repeated movement within one step does not duplicate value messages.
Window deactivation, leaving the window or disabling cancels the gesture without
a release callback; values already delivered remain application-owned. Wheel
events remain available to surrounding scrollables. Hover/press fades use
`motion.short`; the hover circle stays beneath press feedback.

The first version is horizontal and mouse-first. Range sliders, vertical/RTL
layout, touch input, keyboard traversal and focus rings are future work. Incoming
values are also visually snapped when a step is configured; provide matching
values when supplying custom bubble text.

## Segmented buttons

`SegmentSelection::Single(Some(value))` keeps exactly one chosen item on click;
`Single(None)` allows an initially empty selection. `Multiple(Vec<Value>)`
toggles items independently and permits an empty result. `on_change` receives
the whole new selection; the application owns it. Use unique item values.

The default width fits content. `Fill` or a fixed width distributes equal segment
widths. Keep labels concise, or compose with a horizontal scrollable. Passive
icons occupy an 18px slot and are replaced by a check when selected. That slot
remains reserved to avoid label movement. Selection colors animate using
`motion.medium`; the check changes with the application value. Hover and press
reuse the existing button interaction implementation. Disabled groups/items
cancel gestures and do not emit changes.

The shared outline is drawn in a foreground renderer layer after the child
buttons. This prevents selected fills from painting over the connected border.

## Progress and scheduling

`linear_progress(value)` and `circular_progress(value)` accept a fraction in
0..=1, clamp infinities/out-of-range input, and treat NaN as zero. The initial
value renders immediately; subsequent values use `motion.medium`. A complete or
settled determinate indicator requests no further frames.

`.indeterminate(true)` enables autonomous animation. Linear motion now evaluates
Material Web's independent position and width keyframes over a 2000ms cycle.
One segment grows/contracts, then hands off to the other; two segments are briefly
visible together. The internal widths grow from 8% to about 66% and 73% before
contracting and leaving the track. The original identical, half-cycle-offset
sweeps were too repetitive and have been replaced. See the
[Material Web motion source](https://github.com/material-components/material-web/blob/main/progress/internal/_linear-progress.scss).

Baseline circular motion now follows the pinned Android advance delegate: four
grow/shrink phases over 5400ms. Measured updates use a critically damped spring
scaled by the shared motion setting. Circular drawing uses cached SVG geometry
to preserve correct scroll transforms on both renderers.

`.paused(true)` freezes indeterminate motion. Invisible or unfocused indicators
stop requesting frames and discard the inactive interval when resumed. Modal
input isolation preserves background animation; only actual window inactivity
pauses it. Setting
`motion.medium` to zero holds a visible static indeterminate frame (linear uses a mid-cycle pose). Linear `.width` and
`.size` control width and thickness; circular `.size` controls its square size.
Buffer tracks and custom color cycles are available. `.wavy(true)` opts into
linear disjoint/circular retreat motion, amplitude settling and visible circular
tracks; wave travel is optional. See [LOADING_PROGRESS.md](LOADING_PROGRESS.md).

The gallery simulates work on a background thread. Cancellation stops its worker;
generation IDs also reject stale messages after a restart. Components do not
implement the simulation. No files are exported.

## Validation

Behavioral tests exercise controlled updates, step deduplication, endpoint clamps,
cancellation, disabled input, root-dialog blocking, multiple selection, progress
pause/resume and invisible/zero-motion scheduling. Pixel checks cover the shared
outline and scroll translation. Two independent virtual clock origins produce
identical indeterminate frames.

Reference sequences cover both themes, slider press/held/drag/release/endpoints,
segment selection transitions, determinate and indeterminate progress frames,
and narrow/wide export compositions. See [visual tests](VISUAL_TESTS.md) and
[validation results](VALIDATION.md) for the executed checks and platform limits.
