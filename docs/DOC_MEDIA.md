# Executable component documentation

The catalog has 40 component showcase pages, covering every family in
[COMPONENTS.md](COMPONENTS.md). Constructors show a primary preview and its actual
view code; `guide::components` adds light/dark appearance grids. Switches and
button groups also compare Standard and Expressive springs using synchronized
input. Static components use PNG previews; motion examples use APNG.

Related constructors share family pages (for example the four chip types,
side/bottom sheets, and regular/extended FABs). Layout helpers such as
`adaptive_navigation` link to their component family rather than duplicating
media. The grids are curated examples, not an exhaustive matrix of every builder
option, theme, input method, or state. Existing interaction and reference-image
tests remain the broader behavior checks.

Media is generated from real iced components, not hand-authored pictures. It is
not committed to Git. The generated documentation is embedded in the release
archive and works without remote image hosting. No renderer runs in consumer
builds, build scripts, or procedural macro expansion.

## Build and review

Requirements: Rust 1.92.0 and rustfmt. No Python is needed for this workflow.
CI uses macOS 26 / Apple Silicon / TinySkia / bundled Roboto for canonical generation. A second Linux job
builds the exact resulting archive using the docs.rs configuration.

```sh
cargo xtask doc-media build
```

Open `target/doc/iced_m3/guide/components/index.html`. Constructor and showcase
pages share the generated examples. The command builds a staging crate under
`target/doc-media/stage`; it never writes generated files into the working source
tree or uploads anything. Plain `cargo doc` still builds the text-only reference
from a fresh checkout. To regenerate and check repeatability, packaging and
publication without uploading:

```sh
cargo test --locked --manifest-path xtask/Cargo.toml
cargo xtask doc-media check
```

The publication dry run requires network access to crates.io, even when build
dependencies are cached. `CARGO_NET_OFFLINE=true` can be used for `build` with
cached dependencies; it is not sufficient for the full publication check.

## Author one example

The `doc_examples!` invocation in `tests/visual/doc_media/catalog.rs` is the
single rendering registry. The renderer and the development tool compile the
same tokens using their own macro definitions: the renderer builds the views,
and xtask reads metadata without depending on iced. The renderer also emits JSON
metadata, which must match xtask exactly. No tool parses Rust source with regexes.
Each entry provides:

- A normal Rust `view` function, compiled for rendering and stringified from the
  same tokens for the displayed code. Generated snippets are also run as doctests.
- A `make` closure mapping scenario state to that view.
- A `variants` composition that reuses the view with relevant settings.
- Primary and appearance-grid dimensions, scale, caption, whether the primary
  preview animates, comparison selection, and a byte budget for its media.

The renderer in `tests/visual/doc_media/mod.rs` supplies deterministic input and
state updates. New interactions need an explicit timeline there. It asserts
message emission, actual pixel changes, and matching first/last frames for the
finite interaction loops. Selection controls round-trip through their actual
messages; the slider drag checks snapping, both endpoints of the demonstrated
movement, one commit per release, and outside-click focus clearing. Menus,
selects, tabs, chips, navigation, and pickers assert their selection/action
messages. Picker captures check that OK and Cancel remain visible.
Loading and indeterminate progress are explicitly labeled as three-second
excerpts; the capture boundary need not match their natural cycles.

All animations use 60 fps. Compact controls generally use 2×; larger
compositions, popups, and the wide button-group comparison use 1×. Each registry
entry declares its scale, and its static variants use the same scale. Stateless
actions perform one press/release per loop; controlled selections round-trip.
Static examples render once instead of encoding artificial motion. The encoder combines identical consecutive frames and stores only
changed pixels without reducing color precision; it decodes every APNG and
checks all pixels and durations against the original captures.

Add the constructor's conditional `#[doc]` inclusion and a showcase page in
`src/guide/components.rs`. Missing pages, snippets, or images fail the generated
rustdoc check. Struct docs link to the showcase instead of duplicating large
animations. This macro is internal development tooling, not public crate API.

The `page!` macro's `@layout` arm in `src/guide/components.rs` defines the shared
showcase layout: primary preview and code, appearance variants and their recipe,
then an optional motion comparison. Change section ordering there once to update
every showcase. Page declarations supply only an ID, title, and optional `motion`
flag (matching `compare: true` in the rendering registry). Section contents and
formatting are shared by the generator in `xtask/src/generate.rs`.

## Validation and artifact ownership

Generation starts in an empty owned directory. The manifest records source,
font/asset, dependency, harness, generator, and output hashes. The unpublished
`xtask` crate has its own lockfile and stays outside the published crate and
consumer dependency graph. Its compiled source/dependency fingerprint is stored
in the manifest. Verify a downloaded archive using xtask from the matching
source revision; newer tooling deliberately rejects older generator fingerprints.
The published archive needs no xtask or generation dependencies to build rustdoc.
Validation checks exact output membership against the source registry, PNG integrity, animation
frame counts/durations, and the actual embedded images in generated rustdoc.
CI renders twice and requires byte-identical output before checking the extracted
`.crate`. It also compiles all generated examples. Tests deliberately exercise
missing files, orphans, deleted examples, changed sources/fonts/lockfiles,
corrupted images and edited snippets.

These checks establish provenance and repeatability. Visual regression baselines
and motion assertions remain separate; generated docs do not approve their own
appearance as correct.

Do not commit `docs/generated` or anything under `target`. The full catalog
package has a 9.8 MB compressed budget, enforced
by both package validators through `MAX_PACKAGE_BYTES` in `xtask/src/media.rs`.
Each example also has a primary/comparison media-byte budget. Recheck the archive size when extending
coverage; the budget is bounded rather than increasing with the registry.
The package contains
only Markdown fragments with embedded PNG/APNG data plus their manifest; it does
not also ship duplicate standalone images. Readers requesting reduced motion
receive static images through HTML `picture` media selection. Preview pages
have descriptive alternative text.

## Release

Use the [release procedure](RELEASING.md): release PR, merge to master, passing
checks for the merged commit, clean preparation, review, then explicit publish
from staging. Plain `cargo publish` is not intercepted; it is the maintainer's
responsibility to publish the prepared package.
