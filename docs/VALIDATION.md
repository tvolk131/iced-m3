# Validation record

Use [CONTRIBUTING.md](CONTRIBUTING.md) for current commands,
[VISUAL_TESTS.md](VISUAL_TESTS.md) for coverage and the canonical reference
procedure, and [RELEASING.md](RELEASING.md) for release acceptance. Check the exact
candidate in [GitHub Actions](https://github.com/tvolk131/iced-m3/actions);
historical results below do not certify a later commit.

These entries retain findings, measurements and verification boundaries from
individual investigations. Counts and implementation descriptions belong to the
recorded revision. Later entries supersede earlier behavior where noted. Local
`target/` paths identify historical development artifacts, not files available
in a clean checkout. Reproducing older measurements requires the corresponding
source revision and recorded environment. Current limitations live in
[LIMITATIONS.md](LIMITATIONS.md); exact pixels never establish full M3 conformance.

## Rendering performance and wave defaults — 2026-10-09

Regressions first reproduced unbounded shadow rasters, ripple fade cache
invalidation and stationary default waves. Tests now require bounded raster
sizes, endpoint/fade reuse, actual press/release messages, unchanged-value wave
travel, phase continuity, pause/resume and idle scheduling. Independent pixel
oracles retain the former full-outline shadow and clipped-circle ripple across
themes, scales and shapes. See [measurements and remaining costs](PERFORMANCE.md)
and [wave defaults and pinned references](LOADING_PROGRESS.md).

Default/software all-target suites passed, including 269 library tests with 48
diagnostic/reference tests ignored by the ordinary run. Separately, 51 canonical
reference suites passed on Rust 1.92.0. Strict Clippy, formatting and the Rust 1.88
all-target check passed. Updated shadow and ripple images were reviewed. Metal
passed the ripple oracle, integer and fractional shadow comparisons, and seven
loading/progress checks. Generated docs
passed 140 doctests; the staged archive passed media, extracted-package and
independent consumer checks in both feature configurations. Timings remain
opt-in diagnostics, not CI thresholds or native-window frame-rate guarantees.

## Tooltip and callback contracts — 2026-09-29

The first nine-test run had eight failures; the existing menu/Escape control
passed. Twelve final checks passed on Tiny Skia and Metal for plain-tooltip
focus/delay/dismissal, rich-tooltip traversal and pointer entry, Escape/menu
containment, deferred callbacks, disabled controls, rebuilds and invalid time
submission. Existing hover tests also caught overlapping focus/hover dismissal;
the two paths now retain independent suppression state.

The software suite passed, along with Metal Expressive, switch, dialog-exit and
modal-activity regressions covering interruption, reduced motion and simulated
window-focus loss. A native macOS wgpu smoke check of `desktop_contracts` verified
keyboard hints/Escape, traversal out of rich hints, arrow/Space toggles, Enter
submission, invalid drafts, disabled select options, both themes, repeat
selection, dialog dismissal and reduced motion. The spinner continued behind
an open dialog. This was not an accessibility or cross-platform certification.

An initial run on local stable Rust 1.98.1 differed by 60 pixels at a circular
progress zero-length handoff; the reference passed unchanged on canonical Rust
1.92.0. Toolchain differences must be investigated separately from image approval.
Focused reproduction: `cargo test --locked --lib beta4_contracts` with
`ICED_TEST_BACKEND=tiny-skia` or `wgpu`; the native example uses `ICED_BACKEND=wgpu`.

## Switch spatial spring response — 2026-09-27

Regressions reproduced hidden selection overshoot and delayed pressed sizing.
Separate Fast spatial transitions now drive position/diameter; Fast effects drive
colors. Pressing snaps to held geometry, release/cancellation retains spring
response, and only physical track containment bounds extreme custom geometry.
Baseline timed motion and public APIs were unchanged.

Six pixel/property checks passed on Tiny Skia and Metal for both directions,
press/cancel/release, reversal momentum, keyboard activation, reduced motion,
disable, bounded colors, cadence independence and eventual idle scheduling.
The canonical suite gained 128 switch frames across schemes/themes/values/icons;
16 Expressive frames changed while 2,903 prior references stayed identical.
Default/software suites, doctests, lint and the release example passed.
Press/overshoot/settled filmstrips were inspected and the native demo reopened.
Artifacts: `target/switch-motion/`. No Compose pixel parity or new timing
benchmark was claimed.

## Button-group spring response and proactive motion checks — 2026-09-27

The first regressions failed for clipped press overshoot and delayed release.
Targets now account for neighbor padding before applying springs; edge transfers
preserve rebound and share constrained padding without order bias or loss of
content space. Existing APIs and spring presets were retained.

Eight tests covered compact/desktop layouts, every position, connected/spaced
groups, quick reversals and neighbor presses, frame cadence, custom springs,
narrow/asymmetric padding, zero expansion, rail width and tab-indicator pixels.
The rail probe exposed springs restarting at rest and scheduling endless redraws.
Both widget and transition tests failed before correcting unchanged-target
sampling, which also stopped layout consumers from missing changed samples.

Group peak and tab-indicator checks passed on Tiny Skia and Metal; the tab fixture
initializes the retained indicator before changing selection. Default/software
suites and 50 reference functions passed; 76 group frames were added and 26
Expressive frames updated. Representative motion was inspected. Artifacts:
`target/group-motion/`. See [motion contracts](VISUAL_TESTS.md#motion-contracts-beyond-reference-images)
for the continuing audit, not a claim of component-wide motion certification.

## Spring dialog exit compositing — 2026-09-27

At 200ms after closing, a nominally faded white dialog still painted a fully
opaque rectangular cover (channel value 255 on black). Paper and content now
use sequential ranges of one effects spring, keeping paper opaque until content
has disappeared and preserving lifetime/reversal continuity.

The failing pixel regression passed after correction on Tiny Skia and Metal.
Checks exercise actual Close-button input, single/stacked hosts, both schemes,
reversals and reduced motion. The ordinary suite passed; 48 new light/dark
references cover exit through 400ms and interrupted reopening, with earlier
references unchanged. Corrected frames were inspected. Artifacts:
`target/dialog-exit/`, including the original failing image.

## Expressive motion and action recipes — 2026-09-27

The initial red run reproduced missing group width deformation and reduced-motion
round trips erasing custom durations. Opt-in shared springs, retargetable action
shapes and retained equal-width groups were added. [EXPRESSIVE.md](EXPRESSIVE.md)
records reference pins, precedence and remaining catalog gaps.

Fifteen new tests covered analytic cadence independence, interruption, bounded
effects, theme/local/reduced-motion precedence, 30 icon size/width/order
combinations, neighbor width conservation, focus through held-press rebuilds,
disable/cancellation, narrow/empty groups, reversible dialogs and control motion.
Default/software suites, the independent consumer and software-only Rust 1.88
checks passed. The reference suite added 116 frames and updated 12 connected-group
images; representative actions and transitions were inspected.

Theme/explicit/opt-out rendering passed on Metal with native GPU access after
sandboxed adapter creation failed. The native release example's dark layout was
inspected; precise held/interrupted gestures used the headless dispatcher.
Artifacts: `target/expressive/`. Native Windows/Linux and platform-wide spring
parity were not measured.

## Interaction contracts and window activity — 2026-09-26

Tests first reproduced toolbar slider/caret key interception, right-click focus
loss, order-dependent disabled builders and paused unfocused animations; ordinary
toolbar button navigation already passed. Eight regressions covered editing,
context-clicks, 110 disabled-builder/pointer/keyboard combinations across 11
controls, held-press cancellation, restored field actions and viewport clipping.
They also exposed disabled native fields accepting focus and Enter submission;
fields now exclude disabled focus targets and suppress editing while retaining
callbacks.

Default/software suites, six consumer tests and all 46 reference functions
passed without golden changes. Unfocused animation also passed on Metal across
seven modal compositions. Visible loading/progress clocks now ignore keyboard
focus; widget viewport clipping, explicit pause and reduced motion remain
separate controls. Widget clipping is not OS occlusion, which iced 0.14 does not
expose to widgets. Native window-manager interaction was not exercised.
Artifacts: `target/interaction-contracts/`.

## Software dialog repainting — 2026-09-18

A matched release benchmark measured software dialog repaint medians of 77.00ms
before tighter shadow clipping and 53.99ms after; GPU medians were 1.46/1.47ms.
Native iced scenes reproduced the remaining fragmented-repaint cost.
[PERFORMANCE.md](PERFORMANCE.md) retains the matched geometry, damage-policy
experiments, upstream reproduction and limitations; these are not measured
improvements in the downstream Comet application.

Default/software suites, the software consumer and all 2,679 references passed
without baseline changes. New tests checked viewport parity, bounded repaint
work, invisible-shadow skips, incremental correctness at three scales and zero
idle damage. The resizing-shadow pixel/cache check also passed on Metal.
Native input latency and Windows/Linux GPU performance were not measured.

## First release preparation — 2026-09-15

The beta.1 rename established `iced-m3` / `iced_m3` throughout manifests, lockfiles,
imports and examples without changing dependency versions or behavior. The
archive passed 261 ordinary tests, 14 doctests and five independent consumer
flows in each renderer configuration. It retained licenses/release notes while
excluding reference PNGs: 161 files, approximately 1.82 MB compressed.

Library/consumer checks passed on Rust 1.88 in both feature configurations;
local stable 1.98.1 lint and software rustdoc passed. Link checks covered renamed
API/source destinations and anchors. A publication dry run succeeded but did
not upload or tag anything in this preparation pass. The earlier remote
[run 34933165692](https://github.com/tvolk131/iced-m3/actions/runs/34933165692)
passed all five jobs at `4332143`, including platforms, MSRV, packaging and 2,665
visual references. Artifacts: `target/release-preparation/`.
Use the current [staged release process](RELEASING.md), not the historical root
package preparation, for releases with generated documentation media.

## Consumer documentation — 2026-09-15

The README shrank from 648 to 112 lines while all 13 Rust examples were preserved
in consumer guides. Getting started, cookbook and theming also became compiled
rustdoc guides; the README example was compiled separately. Link checks included
Markdown and generated API pages/anchors. Extracted-package tests and independent
consumer flows passed in both configurations. No component or PNG changed; the
2,665-reference suite was deliberately not rerun for this documentation-only pass.
Artifacts: `target/documentation-review/`.

## Desktop beta readiness — 2026-09-14

[Northstar Studio](BETA_READINESS.md) established an independent package/lockfile
using only public APIs. Five flows passed against the extracted archive in both
feature configurations and against Metal from source: validation/exactly-once
save, foreign-themed native popups, modal input blocking, nested discard/focus
restoration, navigation and theme changes. Rust 1.88 checks included the
independently resolved consumer dependencies.

An explicit package include list reduced roughly 72 MB uncompressed to 3.45 MB
(1.75 MB compressed), retaining source/assets/notices and excluding 2,665 golden
PNGs. The dependency tree confirmed that package checks consumed the extraction,
not the checkout. The report was `target/beta-package-report.json`.

Twenty-four reduced-motion review captures covered light/dark, 420/1000px and
1×/1.25×/2× scales, including patterned preferences and nested dialogs. The native
macOS app verified keyboard activation, text replacement, nested discard,
editor focus restoration, Save traversal and the saved-value snackbar. This did
not certify OS scaling, IME, Windows/Linux interaction or screen readers.
Artifacts: `target/beta-*.log`; the local app bundle was unsigned.

## Canonical loading and wavy progress — 2026-09-14

Seven canonical Material shapes and AndroidX-matched cubic morphs replaced the
approximate loader. Source pins, wave geometry and numerical adaptations are in
[LOADING_PROGRESS.md](LOADING_PROGRESS.md); the later 2026-10-09 pass changed the
default wave travel policy.

All seven shapes and seven morph midpoints were compared against independently
exported AndroidX SVGs at 6× scale: at least 99.5% of channels matched exactly,
and remaining differences had to lie within one physical pixel of the contour.
Interior pixels and canonical timed references remained exact. Five new
renderer/behavior checks passed on Tiny Skia and Metal for transparency/gaps,
pause/resume, reduced motion, handoffs, idle scheduling, clock origins and covered
motion. Four unit tests covered loader springs and wave geometry/timing.

Default/software suites and 44 reference functions passed. Eighty-six frames
were added and 66 earlier loader images updated; none were removed. Motion review
used 362 actual captures (six seconds at 30fps in each theme). See the loading
guide for regeneration. Artifacts: `target/loading-progress-*`.

Release measurements below used 60 changing frames at 320×140 logical pixels and
2× scale, **including screenshot readback**. Values are local mean / p95 ms,
not native frame-rate guarantees or directly comparable to repaint-only timings.

| Fixture | Tiny Skia | Metal |
| --- | --- | --- |
| Flat linear | 0.34 / 0.57 | 1.40 / 1.45 |
| Wavy linear | 0.53 / 0.65 | 1.56 / 2.71 |
| Flat circular | 0.42 / 0.46 | 1.54 / 2.76 |
| Wavy circular | 0.67 / 0.74 | 1.59 / 1.66 |
| Canonical loader | 0.43 / 0.49 | 1.57 / 2.72 |

## Tab/list composition and renderer experiments — 2026-09-14

Tabs/lists began preserving parent surfaces by default, with explicit tab
color/gradient fills fixed to the viewport during scrolling. Six tests passed
on software and Metal for parent pixels, actions/disabled rows, explicit fills
and experimental offscreen-group overlap, editing/progress and ancestor clipping.
Default/software suites and exact references passed; 28 frames were added and
eight dark-gallery scrollbar-edge images changed.

At 560×520 logical pixels / 2× scale, changing-frame means for direct drawing,
57-band replay and a CPU offscreen group were respectively 3.85/91.83/25.22ms on
Tiny Skia and 3.95/8.31/37.44ms on Metal, including readback. Translucent overlap
also exposed a CPU/Metal blending mismatch. Neither experimental path shipped;
[COMPOSITION.md](COMPOSITION.md) retains methodology and the proposed native
renderer groups. Artifacts: `target/composition-foundations-*` and
`target/compositing-probe-*`.

## Surface composition and inherited icon opacity — 2026-09-14

Outlined fields/selects changed to transparent fills with a clipped border gap
for moving labels. Explicit fills stayed bounded; filled variants kept their
recipe. This superseded the parent-color workarounds below. Disabled list text,
chip/navigation icons, menu triggers and field adornments retained foreground
alpha. `icon(handle)` passes it through SVG opacity because the pinned renderers
ignore alpha in SVG tint itself; foreign widgets must honor their own alpha.

Five new regressions passed on software and Metal for patterned parents/label
gaps, explicit fills, rebuilds, disabled actions and narrow interrupted labels.
Restoring the old code made three fail. The icon-versus-quad property allowed two
8-bit channel levels for rounding; zero-opacity icons had to equal the parent.
Canonical PNG comparisons stayed exact: 24 frames were added and 447 updated.
Default/software suites passed and representative/native disabled states were
inspected. A paired warm release/Metal dialog profile measured 4.71ms before and
4.82ms after (p95 6.40/6.46ms), including readback. Artifacts:
`target/surface-composition-*`.

## Settings field surface correction — 2026-09-14

The settings fields used `surface_container` instead of their card's
`surface_container_highest`, producing dark fill/label patches. Corrected
parent-color overrides passed gallery behavior/reference checks and light/dark
inspection. The general transparent-fill/border-gap solution above subsequently
removed the need for those overrides. Artifacts: `target/settings-field-*.log`.

## Overlay/carousel completion and dark disabled buttons — 2026-09-14

[DESKTOP_FINISH.md](DESKTOP_FINISH.md) records action staging, directional menu
fades, rich-popup/calendar surface separation, stable search layout, 72px
full-screen headers, fitted carousel keylines, bounded lazy construction and
recent-velocity snapping. Seven new regressions passed on software and Metal,
including interrupted/reduced motion, closing input suppression, 10,000-item
construction, retained overlapping editors, global callbacks/count shrink,
flick-versus-stopped drags and fractional offsets.

The dark disabled-button pixel test failed before fixing shared foreground
compositing (label/fill samples ranged from 96 to 100). It passed with the real
translucent foreground; native output was inspected. Default/software suites
and exact references passed, with 202 added frames and 433 intentional changes.
Timed overlays/search/calendar and all five carousel layouts were reviewed.
Artifacts: `target/desktop-finish-*`. Screen-reader work remained a feasibility
assessment, not an implemented bridge.

## Progress beneath modal overlays — 2026-09-14

Modal input suspension was separated from recurring Material animation clocks,
superseding the pause retained by the earlier invoker fix. Three initially failing
regressions passed on software and Metal across single/stacked/nested dialogs,
side/bottom sheets and mixed hosts. They kept held-gesture cancellation, blocked
typing/caret timers and paused snackbar deadlines separate from indicator motion.
At this revision OS window deactivation still paused indicators; the 2026-09-26
change above removed that policy for visible indicators.

Default/software suites and exact references passed. Forty-two modal-progress
frames were added and 24 ripple frames updated to show advancing background
progress. Nested compositions were inspected. Artifacts: `target/modal-activity-*`.

## Covered invoker feedback — 2026-09-14

A quick FAB click left non-idle pixels after its dialog had fully opened.
Redraw-only updates allowed finite invoker feedback to finish while retaining
input isolation; the same regression then passed for single, stacked and nested
dialogs in both themes. Five dialog regressions passed on Metal, including held
cancellation and scheduling with zero-duration dialog motion. Recurring indicators
still paused at this revision; the subsequent modal-activity pass corrected that.

Default/software suites and exact references passed. Thirty quick-tap frames
were added; four nested-editor gallery frames were corrected to initialize covered
native fields with their enabled foreground. Artifacts: `target/modal-ripple-*`.

## Dialog lag and label-surface correction — 2026-09-14

A release GPU profile at 1160×844 logical pixels / 2× captured 33 opening frames,
including offscreen rendering and screenshot readback. Cached shadow work cut
mean/median/max from 79.17/87.95/94.27ms to 7.05/6.49/15.90ms; the 500ms entrance
was not shortened. Reproduce with
`ICED_TEST_BACKEND=wgpu cargo test --locked --release --example gallery dialog_animation_profile -- --ignored --nocapture`.

Software/Metal regressions checked shadow-raster size/reuse, fractional edges,
clipping, opacity and dropdown host/label colors and selection. Default/software
suites and references passed; 40 frames were added and 50 shadow-edge images
updated. Corrected light/dark renders were inspected. Native automation lost its
window before opening the dialog, so this pass did not establish uninterrupted
native animation or typing. See [implementation details](BASELINE_COMPLETION.md#dialog-regression-follow-up).
Artifacts: `target/dialog-profile-{before,after}.log` and `target/dialog-fix-before/`.

Subsequent release GPU captures checked that the speedup remained: all include
readback and are headless observations, not native FPS guarantees.

| Follow-up | Mean | Median | p95 | Maximum |
| --- | ---: | ---: | ---: | ---: |
| Covered invoker feedback | 5.34ms | — | — | 15.43ms |
| Progress beneath modals | 6.31ms | 5.61ms | 10.74ms | 17.98ms |
| Overlay/carousel completion | 6.04ms | 5.27ms | 8.13ms | 15.63ms |

## Earlier validation

The following milestones were checked locally on 2026-09-12–14 with macOS /
Apple Silicon, Rust 1.92.0 and released iced 0.14.0 (`iced_widget` 0.14.2,
`iced_test` 0.14.0). They precede remote CI/MSRV validation and later behavior
changes documented above.

### Executed checks

Default/software ordinary suites, doctests, formatting, strict Clippy,
documentation and release example builds passed during these milestones.
Canonical references used Tiny Skia even in default-feature builds; targeted
properties also ran on Metal. Manually generated inspection images were never
portable golden assertions. Current commands and suite boundaries are in
[CONTRIBUTING.md](CONTRIBUTING.md) and [VISUAL_TESTS.md](VISUAL_TESTS.md).

### Behavioral coverage

Early tests established press/release matching and exactly-once messages,
rebuild/disable cancellation, finite transition settling/retargeting, viewport
clipping, Unicode field editing/submission/focus retention, and modal pointer,
wheel and keyboard isolation. Additional checks verified actual Roboto font
registration was idempotent, generated semantic color pairs reached 4.5:1 for
black/white/saturated RGB accents in both themes, and empty floating labels
settled without perpetual redraws. These specific probes do not guarantee every
custom palette's contrast or every platform's text behavior.

An oversized-dialog test kept its bottom action reachable by scrolling at
320×480. After floating-label settling, typing scheduled only future caret
redraws rather than continuous immediate frames.

### Visual and native checks

Native macOS observations included text entry/floating labels, light/dark and
teal-accent changes, narrow resizing, scrolling, switches/checkboxes, dialog
rendering, Escape and typing inside dialogs. Wide/narrow headless cases included
320×560 scrolling dialogs and wrapped errors, long labels and custom children.
The overview remains in [gallery.png](gallery.png).

Two findings changed implementation:

- **Typing cost:** at 1160×844 / 2×, rebuilding/layout/recording each populated-note
  change cost about 0.09–0.14ms, versus 14.7–16.2ms for Tiny Skia damaged-region
  painting. Dependency optimization alone did not fix release-mode lag. wgpu
  became the default with software fallback/opt-out; native Metal typing was
  verified, a process sample mostly waited for events, and a spot CPU reading was
  0.8%. Reproduce the software probe with
  `cargo test --release --no-default-features --example gallery software_dialog_redraw_profile -- --ignored --nocapture`.
  Later shadow/painting investigations above refine this diagnosis.
- **Scrollbar clearance:** a 320px dialog's overlay scrollbar covered the content
  edge. Reserving a slim gutter corrected the minimum-size render.

Cached vector plus/star/close icons replaced font glyphs with baseline offsets.
A pixel test subtracts an empty outlined button and measures visible ink at 2×,
including after scrolling. Native GPU and software output were inspected.
Intermittent native automation failures limited some later gesture checks;
headless tests remained reproducible. No long-duration latency or accessibility
certification was claimed.

### Material comparison pass

[MATERIAL_AUDIT.md](MATERIAL_AUDIT.md) records the official M3/MUI comparison of
font roles/weights, checkbox feedback, tonal colors, dialogs, fields and badges.
Inspection caught disappearing checked/mixed/disabled marks after Tiny Skia
scrolling. Cached SVG handles fixed it; the regression failed before and passed
afterward. Both renderer outputs were inspected after bundling Roboto and updating
tokens. Native automation rejected new checkbox gestures, so those interactions
were verified headlessly. The [prior overview](gallery-before-material-audit.png)
is retained; CPU/GPU disabled-state blending was not treated as pixel-identical.

### Everyday workflow milestone

[WORKFLOWS.md](WORKFLOWS.md) records menu/select/tooltip/snackbar/radio recipes.
Tests and inspection exposed unscrolled popup visibility bounds, covered snackbar
releases leaving gestures pending, and snackbar clocks staying paused after a
root dialog closed. New notices also needed to preserve window-deactivation
pause across rebuilds; disabled menus had to close and clear pending presses.
Gallery Duplicate/Undo checked that stale dismissals could not clear a newer
notice. Software/Metal renders covered edge flipping, narrow wrapping and a
select above a dialog. Native automation lost its window before the first menu
click; no native gestures were claimed for this milestone.

### Navigation and structured content milestone

Tests covered tabs/rail overflow, selected values, indicators under scroll
translation, row actions versus nested controls/popups, app-bar actions, modal
blocking, switch feedback and gallery state across widths/filters. Visual review
caught a hidden switch label, wrapped rail label, app-bar text over actions,
squeezed tab badge and inherited parent minimum width in composite tab content.
Corrected light/dark and 320/390px software/Metal renders were inspected. The
native app rendered, but its first page-switch click failed with
`noWindowsAvailable`; final centering was verified headlessly.

### Visual regression milestone

The first 323 references introduced a private clock for real input/redraws,
independent-origin switch comparisons, message assertions and idle scheduling.
The **Workspace unpinned** corruption reproduced on software and Metal: renderer
text/quad batching let background text cross the snackbar. Giving the notice its
own foreground layer fixed it; the scrolled dark Settings reference covers the
original composition.

The report's filters, frame slider and diff links were checked. Deliberately
altered/missing references and a CI update attempt all failed, then restored
references passed. The initial visual job pinned macOS 26 Arm64, Rust 1.92.0,
lockfile, Roboto, Tiny Skia and 2× scale. Remote execution was first verified
later, in the release-preparation entry above.

### Checkbox/radio ripple refinement

Expansion changed to 450ms Standard easing, with a 150ms release fade and a 225ms
minimum visible interval for quick clicks without delaying messages. Tests covered
long holds, growth, quick/repeated/interrupted presses, disabling, zero-duration
motion and idle scheduling on software and Metal. Sixty-four late/quick-click
frames were added; other families' references stayed unchanged.

### Hover beneath the press ripple

The timing change unintentionally faded the existing hover circle. A separate
press layer corrected it. `selection_ripple_preserves_hover_circle` passed on
Tiny Skia and Metal across both controls, values and themes, requiring unchanged
outer hover pixels, added inner feedback, return to hover on release, clearing
on exit and eventual idle scheduling.

### Value controls and progress milestone

Slider/segment/progress checks covered drag rebuilds, deduplication/clamping,
cancellation, controlled selection, modal blocking, scroll translation and idle
scheduling; Export also guarded against stale worker messages. The selected
segment's outline failed a pixel property until moved to a foreground layer.
That property and 60px native-scroll comparisons passed on software and Metal.
Light/dark held/drag/release and timed progress references were inspected; native
gestures and perceived real-time smoothness were not rechecked. See [VALUES.md](VALUES.md).

### Indeterminate linear progress correction

Identical half-cycle sweeps were replaced with Material Web's 2000ms independent
position/width keyframes and cubic-Bezier easing. Segments stretch/contract and
briefly coexist at handoff; circular progress kept its single changing arc.
Pixel tests rejected the old motion on software and Metal, while numerical tests
checked width extrema, contraction, valid geometry and period. Fourteen handoff/
exit references were added; circular and unrelated components were unchanged.
A 60-frame preview used real renderer output, not interpolated screenshots.

### Sheets and primary actions milestone

Tests caught held sheet actions surviving an enclosing dialog. Further checks
covered exit-time blocking, nested-menu Escape, matching outside gestures, typing,
notes/scroll retention, standard/modal changes, invalid dimensions and paused
snackbar deadlines. Thirteen cases passed on Metal; the canonical suite retained
all 559 previous files and added 358 sheet/FAB frames. Review corrected cramped
320px file-row text and a details-list background mismatch on modal sheets.
New native gestures were not rechecked. See [SHEETS.md](SHEETS.md).

### Unverified or deliberately deferred

At these early milestones remote CI, Rust 1.88 and the package identity were
unverified; the later beta-readiness/release entries supersede those statuses.
Canonical loading, waves and width deformation were also later implemented.
Native Windows/Linux interaction, IME/font fallback/clipboard coverage,
screen-reader integration, full touch/RTL/localization and remaining Expressive
fidelity still require their own evidence. Consult [LIMITATIONS.md](LIMITATIONS.md)
and [EXPRESSIVE.md](EXPRESSIVE.md) for current scope.

### Search and filtering milestone

[SEARCH.md](SEARCH.md) records query/filter/range composition. Fourteen new cases
passed on software and Metal for native editing/autofocus/clear/submit, nested
Escape, outside gestures, exit-time input blocking, scrolling, reversible/zero
motion, opacity, separate chip removal, coincident handles, extreme/snapped ranges
and cancellation. Reference inputs were unfocused before capture to avoid native
caret wall time; autofocus/typing retained behavioral coverage. The pass added
166 images and updated 19 gallery compositions; preceding component images stayed
identical. Native OS gestures were not rechecked.

### Adaptive navigation and catalog completion pass

Twenty targeted cases passed on software and Metal, catching dial hands painting
outside bounds, clicks escaping carousel masks and held actions surviving FAB-menu
exit. Other checks covered partial year pages, focus/activation and gallery draft
validation. All 1,083 preceding references were intentionally updated for HCT
colors and the gallery; 204 new frames were added and none removed. Light/dark,
narrow/wide and intermediate renders were reviewed. Family coverage did not
establish full conformance; see [COMPLETION.md](COMPLETION.md).

### Desktop polish pass

Eighteen component checks passed on software and Metal for remembered group
focus, nested scroll reveal, ID-based native focus, calendar row/week navigation,
submenus, matching outside gestures, fixed full-screen actions, date/range popup
lifetimes and rail reversal/reduced motion/modal exit blocking. Gallery tests
covered Save/Cancel and navigation. Review fixed rail header alignment, duplicate
disabled-menu dimming and ancestor focus outlines. Eighty-eight references were
added and 22 gallery images updated; previous component images stayed identical.
See [DESKTOP.md](DESKTOP.md); no new native gesture session was claimed.

### Motion and desktop interaction pass

Fourteen runtime/pixel checks passed on software and Metal for menu prefixes,
repeat letters/Unicode/modifiers, pointer intent, calendar month/year bounds,
overlay reversal/isolation, snackbar replacement and quick/held feedback.
Inspection corrected opaque SVG ripples and full-height shadows escaping partial
menu reveals. Repeated gallery comparisons caught an invoker ripple freezing
while its menu owned input; redraw forwarding corrected it. The suite added 132
frames and intentionally updated 226 for ripple/popup lifecycles and settled
capture times. See [MOTION_POLISH.md](MOTION_POLISH.md).

### Baseline fidelity pass — 2026-09-13

[FIDELITY.md](FIDELITY.md) pins the source audit. Six new renderer properties and
five existing ripple/clock/sheet/snackbar regressions passed on Metal after GPU
access was granted; this was not a native gesture test. Eighty-eight state frames
were added and 839 intentionally changed for corrected recipes. Reviewed fields,
focus states, cards, pickers and galleries compared library revisions, not Google
screenshots. Artifacts: `target/fidelity-*.log`.

### Desktop completion pass — 2026-09-14

Eight new runtime checks and the long-title regression passed on Metal for
native caret/drag preservation, three-level focus return, deactivation, popup-first
Escape, backdrop isolation, opaque layering and manual dates. The pass added
120 frames, changed 28 layout/composition images, and replaced blinking carets
with deterministic text selections in 16 focused-field frames. Native typing
still had behavioral coverage. Light/dark title/picker/stack renders were
inspected; no native-window session was claimed. See [DESKTOP_COMPLETION.md](DESKTOP_COMPLETION.md).
Artifacts: `target/desktop-*.log`.

### Desktop variants pass — 2026-09-14

Seven runtime/rendering checks passed on Metal; scalar tests covered buffer
normalization and color continuity, and a gallery test covered time drafts.
Reviewed cases included field adornments/moving labels, disabled icon contrast,
dropdowns, 12/24-hour drafts, card states, lowered FABs and narrow/wide galleries.
The pass added 206 references and changed 29 dropdown/gallery images. Focused
inputs used selected text to exclude wall-clock caret blinking. Source pins and
remaining adaptations are in [DESKTOP_VARIANTS.md](DESKTOP_VARIANTS.md).
Artifacts: `target/variants-*.log`.

### Desktop color and control fidelity (2026-09-14)

Five new renderer checks and three ripple/menu/scroll regressions passed on Metal.
The ripple-corner probe was isolated from intentional hover-shadow changes;
menu clipping allowed measured ambient shadow extent while still rejecting
escaped content. The pass added 122 references and reviewed elevation levels,
picker footers, slider hover/hold and progress extrema. Comparisons were between
library revisions, not Google screenshots; [DESKTOP_FIDELITY.md](DESKTOP_FIDELITY.md)
preserves primary-source pins and limits. Artifacts: `target/finish-*.log`.

### Baseline desktop motion and layout (2026-09-14)

Seven runtime checks and existing hover/ripple, stack-occlusion and menu-clipping
regressions passed on Metal. Picker tests began locating rendered labels instead
of stale coordinates; progress tests waited for loading completion and spring
settling instead of assuming a 200ms update. The pass added 168 references and
updated 349. Review covered checkbox clearing, overlapping range labels, chip
avatars, loading handoffs, rounded overlays, picker headers and narrow wrapping.

The report's theme/timestamp playback was verified; optional before/after panels
require a local prior-reference backup and are absent in a clean CI checkout.
No new native-window/Windows/Linux GPU verification was claimed. See
[BASELINE_COMPLETION.md](BASELINE_COMPLETION.md). Artifacts: `target/baseline-*.log`.
