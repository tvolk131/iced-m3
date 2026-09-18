# Release notes

## Unreleased

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
  `iced-m3 = "=0.1.0-beta.1"` and upgrade deliberately; subsequent release notes
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
