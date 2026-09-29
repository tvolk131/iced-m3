# Release notes

## Unreleased

## 0.1.0-beta.4 — 2026-09-29

This beta corrects toggle selection colors, keyboard tooltip behavior and callback
timing. There are no public signature changes; iced 0.14 and the Rust 1.88 minimum
are unchanged. Code that relied on select/time-picker callbacks running while
building a view should move that work into application initialization or updates.

- Plain tooltips now reveal on focus as well as hover, with independent dismissal
  state. Rich tooltips allow Tab to continue to the next page control and
  Shift+Tab to return to their trigger; menus retain contained traversal.
- Select and time-picker callbacks now construct messages at activation rather
  than during view construction. Numeric time submission prefers `on_confirm`,
  falling back to `on_change`, and invalid drafts cannot submit. Dial part and
  period changes follow the same deferred callback contract.
- Add a focused `desktop_contracts` native example for keyboard hints, callbacks,
  toggle selection, themes, reduced motion, dialogs and continuous progress.
- Correct common toggle-button selected/unselected palettes by variant. Tonal
  selection now changes color, Filled toggles have a neutral unselected state,
  and enabled selected Outlined toggles lose their outline. Ordinary action,
  icon-button, chip, segmented-button, custom-palette and disabled recipes remain
  separate. Text toggles retain their documented library-specific treatment.
- Add independent rendered selection-role checks and pointer/keyboard selection
  round trips, including related controls and a theme with distinct role colors.
  Document review expectations and show action/toggle state comparisons in the
  generated button examples.

## 0.1.0-beta.3 — 2026-09-28

This beta adds opt-in Expressive spring motion and action recipes, fixes desktop
input contracts, and ships executable illustrated component documentation.
Expressive coverage remains partial; the iced 0.14 dependency and Rust 1.88
minimum are unchanged.

### Compatibility

`tokens::Motion` gains an optional spring scheme and no longer implements `Eq`.
Code constructing it with struct literals must supply the new field or use
`..Default::default()`. Custom animation code should use
`Theme::effective_motion()` to honor reduced motion; `.reduced_motion(false)`
now restores customized timings. See [Expressive support](docs/EXPRESSIVE.md)
for the opt-in behavior and remaining scope.

### Executable component documentation

- Added 40 component showcase pages with code compiled from the same registry
  used to render their previews. Motion examples use lossless APNGs, with static
  reduced-motion fallbacks, light/dark variants and selected motion comparisons.
- Added an unpublished Rust `xtask` for generation, freshness/orphan validation,
  extracted-package checks and release preparation. Media is generated during
  release preparation and embedded in the archive; it is not committed to Git.
- Retired the preliminary APNG size-spike runner while retaining its findings
  and the production encoder's pixel/timing round-trip tests.

### Motion and desktop interaction

- Fixed switch spring motion losing visible overshoot. Thumb position and size
  now retain their spatial spring response, while colors use bounded effects
  motion. Spring-enabled switches snap to their held shape and spring back on
  release; baseline timing and reduced motion remain supported.

- Made the Expressive example's button-group selections independent. Added tab,
  navigation-rail, extended-FAB, switch and slider motion demos, with persistent
  comparison controls and interaction tests for independent demo state.

- Fixed Expressive button-group motion hitting its expansion limit prematurely
  and losing release rebound. Padding limits now constrain the target before
  applying the spring; overlapping animations preserve child content space.
  Added motion-property tests for groups, navigation rails and tab indicators.
- Fixed settled springs restarting at rest, which could keep rail/reveal widgets
  requesting animation frames after their visible movement had finished.

- Fixed a rectangular flash at the end of spring-animated dialog dismissal.
  Content now finishes fading before the rounded panel becomes translucent,
  including when closing is interrupted and reversed.

- Added opt-in `Theme::expressive()` and customizable Standard/Expressive spring
  schemes, with component motion/shape overrides and interruption continuity.
- Added five common/icon button sizes, round/square toggle shapes and three
  icon-button widths. SVG artwork sizing remains explicit. Button groups retain
  focus/state while expanding pressed items and compressing adjacent padding.
- Reduced motion now preserves custom motion settings when toggled off again.
  Custom animations should use `Theme::effective_motion()`. `tokens::Motion` now
  contains an optional spring scheme and implements `PartialEq`, no longer `Eq`.
- Added an interactive `expressive` example and a [reference/coverage guide](docs/EXPRESSIVE.md).

- Focus groups let focused children consume editing keys before arrow/Home/End
  traversal. Text fields and sliders work inside toolbars; Tab still leaves the
  group. Context clicks no longer clear focus in the root focus scope.
- Disabled builders retain callbacks and apply an independent override across
  buttons, FABs, fields, selection controls, selects and sliders. Builder order
  no longer changes disabled state; `.disabled(false)` restores supplied handlers.
  Omitting the handler still leaves a control disabled.
- Visible progress and loading indicators keep animating when the window loses
  keyboard focus. Explicit pause, viewport clipping and reduced motion still
  stop continuous redraws. Input cancellation and snackbar timeout pauses remain
  tied to interaction focus; OS window occlusion is not exposed by iced 0.14.

## 0.1.0-beta.2 — 2026-09-18

This beta simplifies the dialog lifecycle, adds fixed dialog actions and bounded
body scrolling, improves software repainting and clarifies FAB icon usage.
**Upgrading from beta.1 requires migrating single-dialog construction below.**
The iced 0.14.0 dependency and Rust 1.88 minimum are unchanged.

### Breaking: one animated single-dialog API

`dialog::modal(background, dialog, open)` replaces both previous single-dialog
hosts. It retains the dialog's widget state and animates opening and closing.

- Replace `dialog::host(background, dialog, open)` with
  `dialog::modal(background, dialog, open)`.
- Replace the old two-argument `dialog::modal(background, Some(dialog))` with
  `dialog::modal(background, dialog, true)`. To close, keep constructing the
  dialog and pass `false` instead of removing it with `None`.
- Keep the host mounted, including while closed. A host first mounted open starts
  fully visible; changes to `open` animate. Keep any data used by the closing
  dialog available through its exit animation. Reduced motion remains supported.
- `dialog::stack` is unchanged for multiple dialogs. There is no compatibility
  alias or separate immediate-removal dialog API.

### Software dialog repainting

Tightened cached shadow-edge clipping so scrolling a dialog body does not repaint
transparent shadow interiors. Disabled or transparent shadows record no shadow
layers. The matched local software benchmark improved from about 77ms to 54ms
median repaint, with unchanged visual references and roughly 1.5ms GPU repaint.

Added a release CPU/GPU benchmark, deterministic rendering-work tests and a
[performance investigation](docs/PERFORMANCE.md). A separate upstream software
cost remains around fragmented damage and large overlapping backgrounds.

### Fixed dialog actions

Basic dialogs accept `.actions(content)` for a fixed, trailing-aligned footer and
`.max_height(pixels)` to cap the whole panel. The body scrolls within the remaining
height, with a scrollbar gutter and keyboard focus reveal. Short content stays
compact and the panel also fits the window. Footer actions receive the existing
Material entrance animation automatically.

The cookbook, gallery preferences dialog and separate desktop consumer demonstrate
the layout without nested scrollables or manual body-height calculations.
`dialog(content)` still scrolls all content together; the standalone
`dialog::actions(content)` remains an animation marker for inline content.

### FAB icon guidance

FAB constructor docs and the cookbook now demonstrate `icon(svg_handle)` and
explain icon dimensions, layout-box centering and text-glyph alignment pitfalls.
The sizing documentation is attached to `.size()`, and `.extended()` documents
label expansion/collapse. Text-plus FAB test fixtures use SVGs, with rendered
alignment checks through expansion and collapse. The public API and layout are
unchanged.

## 0.1.0-beta.1 — 2026-09-15

The first desktop beta uses package name `iced-m3`, imported as `iced_m3` in Rust.
Earlier development used the local name `iced-material`.

The first desktop beta includes:

- Material 3 actions, selection and text controls, navigation, cards, lists,
  dialogs, sheets, menus, date/time pickers, feedback and loading indicators.
- Semantic light/dark themes, accent colors, bundled Roboto, elevation,
  animated interaction states and reduced motion.
- Keyboard traversal and activation, retained overlay focus, and composition
  with native iced widgets through the public theme adapter.
- A gallery, a separate desktop consumer, executable documentation examples,
  behavioral tests and 2,665 deterministic visual references in the repository.

This beta targets desktop applications using released upstream iced **0.14.0**
and Rust **1.88+**, without a framework fork. GPU rendering is enabled by default;
`default-features = false` selects software rendering unless another dependency
enables iced's GPU feature. CI builds and tests on macOS, Windows and Linux;
native interaction has been checked on macOS.

Native screen-reader integration, full localization/RTL, complete touch support,
arbitrary-child rounded clipping/group opacity and some Expressive variants
remain unfinished. Native Windows/Linux interaction, IME and multi-monitor
scaling still need real-platform testing. See
[support and limitations](https://github.com/tvolk131/iced-m3/blob/master/docs/LIMITATIONS.md).
This release does not claim complete Material 3 conformance or a stable 1.0 API.

## Compatibility

- **During beta:** public APIs may change between prereleases. Pin
  `iced-m3 = "=0.1.0-beta.4"` and upgrade deliberately; subsequent release notes
  will identify breaking changes and migration steps.
- **After beta:** compatible fixes stay within a `0.x` minor series. Breaking
  public API changes advance the minor version while the crate remains below
  1.0. A future 1.0 release will use major versions for breaking changes.
- **Toolchains and iced:** this beta declares Rust 1.88 and pins iced 0.14.0.
  Changes to those requirements will be explicit in release notes. The MSRV
  checks use the included lockfiles; Cargo consumers resolve their own transitive
  dependencies and may need compatible versions when using older Rust.
- **Rendering:** tests cover the documented canonical renderer/environment.
  Pixel identity across platforms, backends, drivers and font rasterizers is not
  promised. Visual corrections may change pixels without changing the Rust API.

Rust code is MIT licensed. Bundled Roboto and loading data retain their OFL-1.1
and Apache-2.0 notices; see [NOTICE](https://github.com/tvolk131/iced-m3/blob/master/NOTICE).
