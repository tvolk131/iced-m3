# Documentation

## Using the library

| Guide | What you will find |
| --- | --- |
| [Getting started](GETTING_STARTED.md) | Dependencies, a complete application, rendering and keyboard setup |
| [Component catalog](COMPONENTS.md) | Available builders, variants and interaction behavior |
| [Cookbook](COOKBOOK.md) | Composed controls, navigation, dialogs, sheets and feedback |
| [Theming and native widgets](THEMING.md) | Semantic colors, typography, surfaces and theme adapters |
| [Gallery and sample apps](GALLERY.md) | Run and explore the gallery or independent consumer |
| [Support and limitations](LIMITATIONS.md) | Platform, accessibility, localization and fidelity boundaries |

Build the API reference with `cargo doc --open --no-deps`. Its homepage has direct
component links; the `guide` module includes getting started, the cookbook and
theming. The examples remain compiled doctests. All three guides are included in
the distributable and work in locally generated rustdoc without a hosted repository.

## Development and release

- [Contributing and validation commands](CONTRIBUTING.md)
- [Visual regression tests](VISUAL_TESTS.md)
- [Executed validation results](VALIDATION.md)
- [Desktop beta integration checks](BETA_READINESS.md)
- [Release process](RELEASING.md)
- [Release notes and compatibility](../CHANGELOG.md)
- [Architecture](ARCHITECTURE.md), [roadmap](ROADMAP.md), and [references/licensing](REFERENCES.md)

## Detailed design notes

These record implementation decisions and historical milestones. Use the consumer
guides above for application setup; older milestone completion counts describe
the state at that time, not current total coverage.

| Topic | Notes |
| --- | --- |
| Fidelity and scope | [Component audit](FIDELITY.md), [MUI/M3 comparison](MATERIAL_AUDIT.md), [catalog completion](COMPLETION.md) |
| Input and workflows | [Values and progress](VALUES.md), [search and filtering](SEARCH.md), [workflow controls](WORKFLOWS.md) |
| Navigation and overlays | [Navigation](NAVIGATION.md), [sheets and FABs](SHEETS.md), [desktop behavior](DESKTOP.md) |
| Composition and accessibility | [Surface/renderer composition](COMPOSITION.md), [screen-reader feasibility](ACCESSIBILITY.md) |
| Motion | [Interaction motion](MOTION_POLISH.md), [loading and wavy progress](LOADING_PROGRESS.md) |
| Desktop milestones | [Completion](DESKTOP_COMPLETION.md), [variants](DESKTOP_VARIANTS.md), [fidelity](DESKTOP_FIDELITY.md), [baseline](BASELINE_COMPLETION.md), [finish](DESKTOP_FINISH.md) |
