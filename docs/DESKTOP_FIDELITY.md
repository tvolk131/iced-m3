# Desktop color, elevation and control fidelity

Current follow-up: [baseline desktop motion/layout](BASELINE_COMPLETION.md) supersedes the historical range-label, progress timing/completion, selection animation, basic-overlay and picker-header limitations below.

Completed 2026-09-14. This closes the scoped follow-up on semantic colors,
shadows, picker actions and borders, slider feedback, and progress. Baseline
Material 3 remains the default; Expressive remains opt-in.

Review [the gallery](http://127.0.0.1:8766/?filter=gallery-finish),
[timed component frames](http://127.0.0.1:8766/?filter=finish-), and
[before/after examples](http://127.0.0.1:8766/desktop-fidelity/).

## Semantic colors and elevation

`ColorScheme` exposes all 49 roles in this library's baseline HCT scheme,
including secondary/tertiary foreground pairs, surface dim/bright/lowest,
background aliases, surface tint, shadow, and all twelve fixed accent roles.
The same cached tonal-spot generator supplies every role. Fixed colors retain
their standard-contrast tones across light/dark mode; contrast settings still
participate in palette generation.

`scrim` is now an opaque semantic color. Modal hosts apply the 32% scrim opacity
when drawing it. Custom scrim alpha multiplies that opacity. Code constructing a
complete `ColorScheme` literal must provide the new fields; applications can
continue customizing a generated `Theme` instead.

`elevated(content, level, radius)` wraps a matching surface without changing its
layout, input, focus or overlays. It draws Material's key and ambient shadows,
including spread, at levels 0–5 and interpolates fractional levels. Built-in
buttons, cards, FABs, surfaces, app bars, dialogs, menus, tooltips, snackbars,
sheets, search, docked calendars and floating toolbars use that renderer.
Slider thumbs use level 1; disabled thumbs have no shadow.

| Level | Key: y / blur / spread | Ambient: y / blur / spread |
| --- | --- | --- |
| 0 | 0 / 0 / 0 | 0 / 0 / 0 |
| 1 | 1 / 2 / 0 | 1 / 3 / 1 |
| 2 | 1 / 2 / 0 | 2 / 6 / 2 |
| 3 | 1 / 3 / 0 | 4 / 8 / 3 |
| 4 | 2 / 3 / 0 | 6 / 10 / 4 |
| 5 | 4 / 4 / 0 | 8 / 12 / 6 |

Key opacity is 30%; ambient opacity is 15%. Measurements are CSS logical pixels;
blur maps to Gaussian sigma at half that value. Cached SVG masks exclude the
surface interior and honor clipping on both renderers. Hover changes elevation;
held buttons return to their prescribed resting level. Menu opening shadows
follow the revealed surface instead of its eventual full height.

`Theme::elevation` remains an iced single-shadow compatibility helper. Use
`elevated` for the complete recipe; native iced's `Shadow` cannot represent both
layers and spread. Rasterization and finite blur kernels can differ from browsers.

Source: [Material Web elevation renderer](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/elevation/internal/_elevation.scss),
[color roles](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/tokens/versions/v0_192/_md-sys-color.scss).

## Pickers and sliders

Date and time pickers offer `.on_confirm(callback)` and `.on_cancel(message)`.
They add trailing Cancel/OK actions. Calendar/dial edits still go to `on_select`
or `on_change`; the application keeps a draft when confirmation must be
transactional. Cancel only sends its message. It does not mutate or commit the
controlled value. Numeric OK/Enter validates and sends the confirmation callback
instead of the ordinary change callback. Without confirmation configured,
manual input retains its existing Apply/Enter behavior.

Calendar OK is disabled for incomplete, reversed, out-of-bounds or unavailable
endpoints. Numeric time OK rejects invalid hours/minutes and converts AM/PM.
A docked calendar with confirmation stays open after selecting a date and closes
on OK/Cancel. The default docked calendar still closes on complete selection.

Both clock and numeric AM/PM selectors share one continuous rounded outline and
one separator. The selected tertiary fill stays inside that outline.

Slider value labels now have the capsule-and-pointer shape and scale from their
bottom center over 100ms (`motion.slider_label`, standard/emphasized curve).
They appear on hover, focus and hold without changing layout, and reduced motion
settles them immediately. Single and range sliders share label drawing. Range
handles have the reference's 1px on-primary outline when overlapping.

Sources: [slider implementation](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/slider/internal/_slider.scss),
[time-picker tokens](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/tokens/versions/v0_192/_md-comp-time-picker.scss).
Exact modal header spacing, locale/RTL, colliding range-label choreography and
newer Expressive slider sizes remain outside this pass.

## Progress reference profile

Progress uses the current Android baseline drawing profile, pinned to revision
`d12048664f383e88148afb18e971aa6dd24ed42e`: rounded ends, primary-container track,
4px gaps and a determinate linear stop marker. Circular determinate progress has
an inactive track; its indeterminate form hides that track. Expressive retreat
motion and wavy geometry are separate variants and are not enabled here.

Circular advance motion evaluates the reference's four-cycle, 5400ms sequence:
667ms growth and contraction, 1350ms starts, a 20–270° arc, and the independent
rotation. Color changes use the 1000ms delay/333ms fade with gamma-correct RGB
interpolation. Scalar tests check extrema and continuity at the wrap. Linear
motion retains the already-reviewed Material Web 2000ms disjoint keyframes;
linear multicolor now holds for 60% and blends over 40% of each color interval.

Sources: [Android styles](https://github.com/material-components/material-components-android/blob/d12048664f383e88148afb18e971aa6dd24ed42e/lib/java/com/google/android/material/progressindicator/res/values/styles.xml),
[circular advance delegate](https://github.com/material-components/material-components-android/blob/d12048664f383e88148afb18e971aa6dd24ed42e/lib/java/com/google/android/material/progressindicator/CircularIndeterminateAdvanceAnimatorDelegate.java),
[linear keyframes](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/progress/internal/_linear-progress.scss).

`.track_color(color)` overrides the inactive color. The translucent buffered bar
remains a documented library adaptation. Determinate updates keep the shared
200ms transition; Android spring completion, indeterminate-to-determinate finish
choreography, every size delegate and legacy dotted buffering are not claimed.

## Validation and remaining boundaries

New scalar checks exercise fixed roles, contrast pairs, elevation measurements,
and circular timing/color. Runtime checks cover shadow interior/extent/tint,
AM/PM outlines, validated confirmation/cancellation, animated slider labels,
reduced motion, progress gaps and stop markers. Five new runtime checks and three
existing ripple/overlay/scroll regressions pass with Metal as well as software.
122 additional reference images cover both themes, all shadow levels, picker
footers, held sliders, small progress fractions and precise circular frames.

See [VALIDATION.md](VALIDATION.md) for complete counts and executed commands.
The broader remaining work includes native accessibility integration, native
text tracking/caret styling, locale/RTL, cross-platform validation, and full
Expressive shape/size/motion fidelity. None is implied complete by this pass.
