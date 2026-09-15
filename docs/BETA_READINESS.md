# Desktop beta readiness

This milestone tests the crate as a dependency of an independent application and
prepares a reproducible distributable. It does not certify full M3 conformance,
native accessibility, Windows/Linux runtime behavior or a stable 1.0 API. The
first `iced-m3` beta is prepared for publication but has not been uploaded.
See [release preparation](RELEASING.md) for current distribution details.

## Separate application boundary

`consumers/desktop` is Northstar Studio, a separate Cargo workspace with its own
manifest, lockfile, profiles and renderer feature. It has no source includes,
private library imports or access to the private animation clock. The application
contains its own state/update/view code and keeps all data in memory.

Five integration flows exercise that real application:

- Field validation, native text entry, Material select and exactly-once saving.
- Native iced button and pick-list actions/popups through `widget::themer`,
  including blocked background actions under a modal.
- Retained nested discard confirmation, native focus restoration and draft retention.
- Navigation, theme changes, fractional-scale rendering and outlined fields on
  a custom gradient while preserving application values.
- Keyboard entry, traversal and activation across the consumer/library boundary.

The generic metrics component uses only iced text/container catalogs. It is an
application-owned reusable widget, not a claim to have validated all third-party
widget libraries. The native popup deliberately tests a different theme type in
the same widget tree. Capture fixtures cover light/dark themes, 420/1000px windows,
1x/1.25x/2x scale and both dialog levels; they are review artifacts, not additional
goldens or an OS scaling certification.

## Integration and API findings

The existing public APIs support these flows without adding a wrapper framework
or changing the library's runtime behavior. Application profiles are intentionally
declared in the consumer manifest; dependency profiles do not configure an app.
Its `wgpu` feature forwards to `iced-m3/wgpu`, and both configurations are
checked independently of the root package's feature graph.

The main documentation gap was the foreign-theme boundary. Native text/input,
container, SVG, scrollable, checkbox and rule catalogs work directly. Native
buttons/pick lists and other widgets can be scoped with
`widget::themer(Some(material_theme.iced()), content)`. Messages, operations and
popups are forwarded by iced. This preserves native appearance and behavior; it
does not turn those widgets into Material controls or make every native widget
keyboard-focusable. Consumers cannot implement a foreign iced catalog on the
library's foreign Theme type; the [theming guide](THEMING.md) shows the adapter concretely.

The review kept the existing callback conventions (no callback means disabled),
application-owned values and retained dialog hosts. Geometry measurements such as
loader diameter are `f32`; layout widths accept `Length`; enum-valued variants
remain typed. No broad rename or speculative API expansion was needed by this app.

## Package contents and reuse

The default Cargo package previously included all reference PNGs, around 72 MB
uncompressed. An explicit include list now keeps build inputs, docs, examples,
ordinary test sources, the shape generator and licensed assets while excluding
the 2,665 pixel baselines. The source repository retains every baseline unchanged.
The normal tests compile and run from the extracted archive; the ignored strict
reference suite requires a source checkout containing those PNGs.

The package metadata now records `MIT AND Apache-2.0 AND OFL-1.1` for the jointly
distributed Rust code, generated shape data and font. The Rust code remains MIT;
this is not a license change for individual files. Root and asset notices stay in
the archive, with runtime notice constants available for application acknowledgments.
The package is now `iced-m3`, with public repository and documentation metadata.
Both manifests/lockfiles and all executable examples use the release name.

`python3 tools/check_package.py` runs Cargo's package verification, checks required
assets/notices and the absence of heavyweight development files, executes ordinary
tests from the extracted archive, then tests a copied consumer against that archive
in both renderer configurations. Its temporary dependency path cannot fall back to
the source checkout. `--offline` uses cached dependencies; omit it on a clean host.
The report is `target/beta-package-report.json`. Python 3.9+ is needed only for
this development check; it is not a library/application dependency.

The procedure follows Cargo's documented
[package verification](https://doc.rust-lang.org/cargo/commands/cargo-package.html),
[include/license metadata](https://doc.rust-lang.org/cargo/reference/manifest.html)
and [minimum-version policy](https://doc.rust-lang.org/cargo/reference/rust-version.html).

## Support and verification boundaries

| Area | Scope |
| --- | --- |
| Rust | Edition 2024; minimum 1.88.0 checked for library and consumer targets in both feature configurations; CI also tests current stable; canonical visual tests use 1.92.0 |
| iced | Released upstream 0.14.0; default GPU backend with software fallback, optional software-only build |
| macOS | Local software and Metal checks; native release-window keyboard editing, nested confirmation, focus restoration and saving; captures at multiple logical sizes/scales |
| Windows/Linux | Remote builds and headless tests pass; native interaction, IME, multiple-monitor scaling and GPU behavior still need real-platform checks |
| Accessibility | Material keyboard traversal/reduced motion included; native screen-reader integration remains missing (see ACCESSIBILITY.md) |
| Localization | RTL and localized calendar/date entry remain unfinished |
| Fidelity | Known arbitrary-child clipping/group-opacity and remaining Expressive variants remain documented in FIDELITY.md |

CI now explicitly enters the consumer workspace, checks Rust 1.88 in a dedicated
job, and verifies the packaged consumer. The existing strict image suite remains
in its canonical macOS job. All five jobs passed in
[the first successful remote run](https://github.com/tvolk131/iced-m3/actions/runs/34933165692).
This establishes automated coverage, not a native Windows/Linux session. Later
release-candidate checks and artifact sizes are recorded in [VALIDATION.md](VALIDATION.md).

## Next use

Run `cargo run --locked --manifest-path consumers/desktop/Cargo.toml` from the source
checkout, then try the library in a real project. Before publishing, complete the
[release checklist](RELEASING.md). The recommended next development
work should come from actual application integration; screen-reader support and
renderer-group integration remain separate, larger milestones.
