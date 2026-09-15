# Baseline desktop motion and layout follow-up

Completed 2026-09-14. This pass improves the existing desktop components while
retaining released iced 0.14, the public controlled-value APIs and the established
baseline Material profile. Expressive remains opt-in.

Try **Workspace → Components → Feedback in motion** in the gallery. The local
[review page](http://127.0.0.1:8766/baseline-completion/) compares library versions;
[timed frames](http://127.0.0.1:8766/?filter=baseline-) use the actual iced renderer.
Neither is a set of screenshots from Google's implementation.

## Dialog regression follow-up

The resizing shadow originally changed its SVG handle and raster dimensions on
every frame. Both iced renderers rasterize filtered SVGs on the CPU, so the two
large Gaussian blurs caused severe animation stalls even in a release GPU build.
Basic dialogs now reuse a small shadow template, tiling only its straight edges
and retaining unscaled corners. Patch boundaries are shared and snapped to avoid
GPU clipping seams. Small surfaces whose corner blurs interact retain the exact
outline path. The 500ms entrance and 150ms exit durations are unchanged.

`Select::background` and `Select::background_with` now mirror the text-field API.
The gallery's workspace-access selector uses `surface_container_high` inside its
dialog, matching both the trigger fill and the floating-label cutout to the host.
Previously it incorrectly retained the page's `surface` color there. The popup
menu retains its own elevated surface styling.

Regression coverage checks cache reuse, a bounded shadow raster, Gaussian outline
equivalence (allowing one device pixel at fractional SVG translations), viewport
clipping and opacity, both dialog hosts, light/dark field surfaces, selection, and
the actual gallery composition. Forty timed opening/closing reference frames
cover the fixed selector inside both hosts. Performance measurements and executed
checks are recorded in [VALIDATION.md](VALIDATION.md).

### Feedback underneath dialogs and sheets

Opening a modal previously stopped background redraws, freezing released FAB
ripples and progress indicators. Shared covered-content updates now finish finite
feedback and advance Material progress/loading indicators behind dialogs, stacked
parent dialogs, and modal side/bottom sheets. Input, application messages, event
capture and IME requests remain isolated. Existing gesture cancellation, native
caret suspension, tooltip suppression and snackbar timeout pauses are preserved.

Private synchronous update scopes distinguish interaction-only cancellation from
actual window deactivation; they do not change iced's event API. Only modal hosts
and Material progress/loading clocks opt into that distinction. Covered native
controls never receive a real focus-restoration event just to restart a spinner.
Nested hosts inherit the outer host's window activity. Explicit pause, viewport
clipping and reduced motion remain respected; deactivating the app pauses progress
without counting inactive wall time when it resumes.

Tests cover quick taps, held gesture cancellation, finite ripple scheduling with
an instant dialog, all three indicator types behind seven overlay compositions,
real window focus, explicit pause, reduced motion, and snackbar actions surviving
modal coverage. Light/dark timed references cover both ripple completion and
ongoing progress. Ripple and dialog durations are unchanged. Validation and image
counts are recorded in [VALIDATION.md](VALIDATION.md).

## Range labels

Hover shows the nearest handle's value. Keyboard focus and dragging show both.
Overlapping capsules separate with a 4px gap and stay inside the control's bounds.
They retain their pointed shape and 100ms scale transition. Very narrow hosts
clip long text within each capsule. Handle movement, ordering, snapping, callbacks
and the outline on overlapping handles retain their existing behavior.

Outward label separation and a reserved label slot are deliberate desktop
adaptations. Material Web instead marks the upper overlapping handle/label with
an outline; this pass does not claim its exact label choreography.
[Reference slider](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/slider/internal/slider.ts).

## Progress

Measured values use an analytic critically damped spring: stiffness 50, damping
ratio 1. Retargeting preserves velocity. `motion.medium / 200ms` scales the spring
in the same direction as Android's animator-duration setting. Zero settles
immediately. Settling uses a one-logical-pixel minimum visible change based on
track width or circular perimeter, with the reference's 0.75 position and 62.5
velocity multipliers. It is not a fixed 200ms transition.

Turning off `.indeterminate(true)` preserves the current frame and completes
loading before applying the latest measured value:

- Circular: the arc closes over 333ms with fast-out-slow-in easing.
- Linear: the current disjoint sequence finishes at its next cycle boundary.
  This library retains the reviewed Material Web 2000ms cycle, so the wait is
  at most one cycle.
- The measured spring then starts from zero. A delayed frame integrates the
  elapsed time after completion too; sparse and dense redraws agree.
- Restarting loading during completion retains continuity. Paused, hidden,
  unfocused and reduced-motion indicators skip a requested handoff and show the
  measured value. Settled indicators stop scheduling redraws.

References: [Android determinate drawable](https://github.com/material-components/material-components-android/blob/d12048664f383e88148afb18e971aa6dd24ed42e/lib/java/com/google/android/material/progressindicator/DeterminateDrawable.java),
[circular advance completion](https://github.com/material-components/material-components-android/blob/d12048664f383e88148afb18e971aa6dd24ed42e/lib/java/com/google/android/material/progressindicator/CircularIndeterminateAdvanceAnimatorDelegate.java),
[linear completion](https://github.com/material-components/material-components-android/blob/d12048664f383e88148afb18e971aa6dd24ed42e/lib/java/com/google/android/material/progressindicator/LinearIndeterminateDisjointAnimatorDelegate.java).
Android's integer drawable bounds are physical pixels; the library's geometry
and threshold are logical pixels. Buffered rendering, the Web linear cycle,
other Android delegates and Expressive/wavy variants remain distinct choices.

## Selection and chips

Checkbox selection uses 350ms emphasized-decelerate scale motion; deselection
uses 150ms emphasized-accelerate motion. Fill, outline and mark opacity have a
separate 50ms linear transition. The mark uses two perpendicular bars: the long
bar grows from their shared vertex; mixed selection rotates/shortens the bars.
Rapid reversals retain an already visible stroke. Selection/mixed durations use
`checkbox_select`; deselection uses `short`, and opacity uses `short / 3`.
The existing 450ms checkbox/radio press ripple and persistent hover circle are
unchanged. Radio and switch selection retain their own existing motion.
[Reference checkbox](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/checkbox/internal/_checkbox.scss).

Filter chips retain both icon trees in a fixed 18px slot and exchange them over
150ms, so selection does not move adjacent chips. This scale exchange is a
library animation, not an exact Material Web icon animation: the pinned Web
filter implementation swaps its selected slot directly. Input chips preserve
their supplied avatar/icon when selected. Avatar input chips use 16px container
corners and a 4px leading inset; caller content still owns the avatar artwork.
[Filter reference](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/chips/internal/filter-chip.ts),
[input-chip reference](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/chips/internal/_input-chip.scss).

## Menus and dialogs

Plain menus and basic dialogs paint their rounded surface separately from their
contents. The moving bottom edge retains its corners and shadow. Contents keep
their complete layout, wrapping and editor state while the available drawing
area grows or contracts. Closing surfaces retain input isolation.

| Property | Open | Close |
| --- | --- | --- |
| Plain menu height | 0 → 100%, 500ms standard/emphasized | Current height → 35%, 150ms emphasized-accelerate |
| Basic dialog height | 35 → 100%, 500ms standard/emphasized | Current height → 35%, 150ms emphasized-accelerate |
| Basic dialog translation | −50px → 0, 500ms | Current position → −50px, 150ms |
| Surface opacity | 0 → 1, 50ms | 1 → 0, final 50ms of exit |
| Dialog scrim | 0 → 32%, linear over 500ms | Current opacity → 0, linear over 150ms |
| Basic dialog content | 50ms delay, 200ms linear fade | 100ms linear fade |
| Plain menu content | 50ms delay, 250ms linear fade | 100ms linear fade |

Custom menu/dialog duration settings scale these intervals. Reversals retarget
height, translation, surface, content and scrim independently without jumping;
reduced motion settles all of them. Both single and stacked basic-dialog hosts
share the same renderer. Full-screen task dialogs retain their whole-window
slide and fixed action bar, using the shared dialog durations.

The subsequent [overlay/carousel pass](DESKTOP_FINISH.md) adds `dialog::actions`,
per-row menu staggering, rich-popup surface/body separation and search staging.
The table above records this earlier pass; the linked document contains the
current timing details. Fades still use the known surface color because iced
0.14 does not expose arbitrary subtree group opacity.
[Dialog motion reference](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/dialog/internal/animations.ts),
[menu reference](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/menu/internal/menu.ts).

## Picker headers

Regular date headers reserve 120px for single selection and 128px for ranges,
with 24px horizontal and 16px top/bottom insets. The divider spans the entire
surface. Long text may increase the header height. Compact calendars retain
16px padding and their existing compact header. Clock and numeric time titles
start 16px below the surface; the time display begins at 44px. Existing picker
confirmation, validation and cancellation APIs are unchanged.
[Date tokens](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/tokens/versions/v0_192/_md-comp-date-picker-modal.scss),
[time-picker layout](https://github.com/material-components/material-components-android/blob/d12048664f383e88148afb18e971aa6dd24ed42e/lib/java/com/google/android/material/timepicker/res/layout/material_timepicker_dialog.xml).
Locale/RTL, every native picker layout and mobile gestures are outside this pass.

## Verification and remaining work

The new checks cover independent overlay timelines/reversals, sparse-frame
progress completion, loading restart/reduced motion, checkbox opacity and hover
preservation, chip layout/avatar retention, range-label separation, exact header
spacing and the gallery's controlled preview values. Existing interaction,
scroll, nested-menu, focus, occlusion and caret tests run alongside them. Both
themes have new fixed-time reference sequences. See [VALIDATION.md](VALIDATION.md)
for final counts and executed checks.

This milestone does not certify full M3 conformance. Remaining work includes
native accessibility, native text tracking/caret styling, locale/RTL,
Windows/Linux native validation and the documented Expressive variants.
The subsequent overlay/carousel pass closes semantic staggering, fitted
keylines and lazy construction, with explicitly documented motion adaptations. The current component matrix
is in [FIDELITY.md](FIDELITY.md).
