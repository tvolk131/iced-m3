# Release notes

## 0.1.0-beta.1 — first release candidate

Prepared for crates.io; publication is pending. The package is `iced-m3`, imported
as `iced_m3` in Rust. Earlier development used the local name `iced-material`.

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
