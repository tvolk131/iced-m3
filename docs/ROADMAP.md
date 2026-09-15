# M3 completion sequence

Authorized sequence: adaptive navigation → date/time pickers → remaining variants
and carousel → visual/Expressive refinement, with keyboard and accessibility work
throughout. Keep released upstream iced 0.14; do not claim full conformance merely
because every family has an implementation.

- [x] Adaptive navigation: bottom navigation, expanded rail, responsive host,
  app-bar variants/collapse, keyboard/focus infrastructure and gallery.
- [x] Date/time: calendar and text input, date ranges, bounds/disabled dates,
  clock and text time entry, validation, gallery and tests.
- [x] Remaining variants: filled fields, elevated buttons, interactive cards,
  richer badges/dividers/tooltips/menus, switch icons, sheet variants/gestures,
  extended FAB motion; carousel with sizing and snapping.
- [x] Visual/Expressive: HCT palettes, contrast/reduced motion, fuller state and
  transition coverage, button groups/split buttons/toolbars/FAB menus/loading
  indicator, opt-in press shapes, shared motion preferences and gallery.
- [x] Final validation: meaningful behavioral tests, precise reference frames,
  review images, default/software builds, Metal checks, documentation and release
  examples. Explicitly record any platform limitations or unfinished subvariants.

Accessibility audit: the installed released iced 0.14 Widget/Operation APIs expose
focus traversal but no accessibility-tree hook or AccessKit dependency. Native
screen-reader integration therefore needs an upstream capability change or a
separately evaluated platform bridge. Do not silently fork iced or claim working
screen-reader support. Exact native text tracking is another upstream API gap.

These milestones deliver the desktop implementations described in [COMPLETION.md](COMPLETION.md). That document lists unimplemented subvariants and platform boundaries; checking a milestone does not imply complete M3 conformance.


## Desktop polish follow-up

User priority: desktop polish before touch/mobile.

- [x] Roving focus, selected-item entry, calendar rows and nested scroll reveal.
- [x] Cascading menus, keyboard/pointer interaction and focus restoration.
- [x] Full-screen dialogs with fixed actions and a transactional gallery editor.
- [x] Docked date selection with persistent month/year navigation.
- [x] Animated rail width and modal expanded rails with input isolation.
- [x] Final desktop visual review, default/software/GPU checks and release build.

Details and remaining limits: [DESKTOP.md](DESKTOP.md).

## Motion and desktop interactions

- [x] Bounded pointer-origin button ripples over preserved hover feedback.
- [x] Retained menu, dialog, tooltip and snackbar opening/closing lifecycles.
- [x] Menu type-ahead and bounded diagonal pointer intent.
- [x] Calendar month/year keyboard navigation with retained focus and bounds.
- [x] Gallery integration and timed interaction/reference coverage.
- [x] Final visual comparison, default/software/Metal checks, docs and release build.

Details and remaining limits: [MOTION_POLISH.md](MOTION_POLISH.md).


## Baseline fidelity audit

- [x] Pin the reference profile and compare component/state recipes across the catalog.
- [x] Correct field, button, selection, chip, card, navigation, picker and sheet gaps.
- [x] Add renderer assertions, keyboard-focus references and timed field-hover coverage.
- [x] Review representative light/dark images and preserve a before/after comparison.
- [x] Complete final software/Metal checks, documentation and release rebuild.

Audit matrix, source links and explicit remaining adaptations: [FIDELITY.md](FIDELITY.md).

## Desktop completion

- [x] Ellipsize long app-bar titles with shaped, Unicode-safe fitting.
- [x] Retained modal stack with top-only input, focus/caret restoration and nested popups.
- [x] Compact docked calendar and validated manual-entry toggle with Apply/Enter.
- [x] Gallery discard-confirmation and calendar integration.
- [x] Behavioral/reference review, software/Metal checks, documentation and release rebuild.

Details and remaining scope: [DESKTOP_COMPLETION.md](DESKTOP_COMPLETION.md).

## Desktop variants follow-up

- [x] Field leading/trailing icons, independent trailing actions, prefix/suffix text.
- [x] Filled/outlined dropdown fields with error/supporting text and keyboard menus.
- [x] Integrated clock/numeric time switching, controlled drafts and validated Apply/Enter.
- [x] Disabled/dragged cards and lowered FAB elevation/color variants.
- [x] Independently buffered linear progress and multicolor indicator cycles.
- [x] Runtime checks, timed visual references, gallery compositions and documentation.

Recipes and explicit remaining adaptations: [DESKTOP_VARIANTS.md](DESKTOP_VARIANTS.md).

## Desktop color and control fidelity

- [x] Complete baseline semantic color roles, including fixed accents and surfaces.
- [x] Shared key/ambient shadows with spread, clipping and fractional levels.
- [x] Picker Cancel/OK APIs, validation and continuous AM/PM outlines.
- [x] Slider thumb elevation, pointed labels and hover/focus scale motion.
- [x] Current progress drawing profile and reference circular advance timing.
- [x] Runtime/property tests, reference images, gallery, software/Metal review and docs.

Details and remaining boundaries: [DESKTOP_FIDELITY.md](DESKTOP_FIDELITY.md).


## Baseline desktop motion and layout

- [x] Separate colliding range labels while preserving controlled handle behavior.
- [x] Progress finish handoff and velocity-preserving spring updates.
- [x] Checkbox opacity/bar morph and retained chip icon/avatar feedback.
- [x] Rounded plain-menu/basic-dialog surface transitions and independent fades.
- [x] Baseline date/time header insets, dimensions and full-width date divider.
- [x] Gallery, runtime/property checks, precise reference frames and documentation.

Details and explicit remaining adaptations: [BASELINE_COMPLETION.md](BASELINE_COMPLETION.md).

## Overlay and carousel completion follow-up

- [x] Separate dialog action timing and directional menu-row staggering.
- [x] Rounded rich-popup/calendar surfaces and independent content fades.
- [x] Stable search layout, full-screen header height, result/clear timing.
- [x] Fitted carousel keylines, centered hero, recent-velocity snapping and lazy construction.
- [x] Dark-mode disabled foreground compositing regression correction.
- [x] Accessibility feasibility assessment against pinned iced and current AccessKit adapter requirements.

Details: [DESKTOP_FINISH.md](DESKTOP_FINISH.md), [ACCESSIBILITY.md](ACCESSIBILITY.md).
Remaining work is native accessibility integration, native text tracking/caret
styling, locale/RTL, remaining per-component/Expressive adaptations and native
Windows/Linux validation. Touch/mobile remains a lower priority than desktop.

## Composition foundations

- [x] Transparent tabs/lists and explicit color/gradient fills, including a fixed tab viewport fill.
- [x] Parent-composition, input, scrolling and timed reference regressions in light/dark themes.
- [x] Rounded clipping and arbitrary-content opacity prototypes, exercised on Tiny Skia and Metal.
- [x] Measured performance/color-space boundaries and a concrete renderer integration proposal.
- [ ] Native renderer groups for efficient antialiased child clipping and opacity.
- [ ] Migrate carousel masks and arbitrary-content fades once that integration passes correctness/performance checks.

The experiments are test-only. Current carousel masks and known-surface fades
remain unchanged; see [COMPOSITION.md](COMPOSITION.md) for measured tradeoffs and
the upstream-oriented integration contract.

## Loading/progress completion

- [x] Canonical seven-shape curves and AndroidX morph matching, with reproducible export and license notices.
- [x] Correct loading shape/container size ratio and contained foreground role.
- [x] Wavy linear/circular profiles, wave configuration, amplitude settling and completion handoffs.
- [x] Pause/reduced-motion/covered-window regressions, source-curve comparisons and timed references.
- [x] Interactive gallery controls, 30fps captured motion review and release rendering-cost checks.

Details and remaining numerical/platform adaptations: [LOADING_PROGRESS.md](LOADING_PROGRESS.md).

## Desktop beta readiness

- [x] Independent consumer package with its own manifest, lockfile, features and application state.
- [x] Public-API flows mixing Material/native widgets, a generic component, custom surfaces and nested dialogs.
- [x] Native-theme adapter documentation and runtime/keyboard/Metal integration checks.
- [x] Rust 1.88 checks for library and consumer, with default and software-only features.
- [x] Smaller distributable with complete build inputs/notices and tests against the extracted package.
- [x] Consumer/packaging/MSRV CI jobs, release build, native macOS flow and multi-scale visual review.

See [BETA_READINESS.md](BETA_READINESS.md). Publication identity, native
Windows/Linux validation and the documented accessibility/renderer/platform work
remain separate. No component-family expansion or runtime behavior change was
needed by the consumer app.
