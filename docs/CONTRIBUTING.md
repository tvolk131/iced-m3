# Development and validation

Run commands from the source checkout. Rust 1.88 is the supported minimum;
see [integration and platform boundaries](BETA_READINESS.md). The workflow is
in [`.github/workflows/ci.yml`](../.github/workflows/ci.yml); check the exact
candidate's run in
[GitHub Actions](https://github.com/tvolk131/iced-m3/actions).

## Routine checks

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
ICED_TEST_BACKEND=tiny-skia cargo test --locked --all-targets
cargo test --locked --doc
cargo build --locked --examples
ICED_TEST_BACKEND=tiny-skia cargo test --locked --all-targets --no-default-features
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps
```

These tests use iced_test 0.14 and real headless rendering. The ordinary suite
checks interaction, messages, focus/editing, animation scheduling and rendering
properties. Exact reference images and diagnostic generators are intentionally
separate; see [visual coverage, commands and baseline review](VISUAL_TESTS.md).

CI also checks the unpublished `xtask` and independent `consumers/desktop`
workspaces, both feature configurations, Rust 1.88, the docs.rs target and the
extracted package. Keep their separate manifests and lockfiles: they exercise
tooling and consumer boundaries that root-package tests cannot replace.
[VALIDATION.md](VALIDATION.md) records historical findings and executed native
checks; [LIMITATIONS.md](LIMITATIONS.md) records current support boundaries.

## Performance

Normal tests check bounded shadow work, cache reuse, idle damage and incremental
repaint correctness. For release CPU/GPU timing comparisons and the dialog
investigation, see [rendering performance](PERFORMANCE.md). Run the ignored
benchmark explicitly on a consistent machine; shared CI has no wall-clock gates.

## Selection-state checks

Run `cargo test --locked --no-default-features --lib selection_contracts` when
changing selection styling or a shared button rendering branch. These ordinary
CI tests render explicit background/foreground/border expectations independently
of golden images. The common-toggle expectations follow the pinned AndroidX
reference in [EXPRESSIVE.md](EXPRESSIVE.md); Text toggles and disabled treatments
retain the library's existing recipes. Related-control checks protect the
baseline role contracts for icon buttons, chips, segmented buttons, tabs,
navigation, checkboxes, radios and switches; they are not a full conformance audit.

Use both default themes and a diagnostic theme with distinct role colors. Do not
require every custom palette to make selected/unselected colors different. Check
specific roles and regions: a whole-image inequality can pass just because a
corner or shadow changed. Exercise actual activation, update application state,
rebuild, clear focus/hover feedback, and check the settled result. A missing
selection fill must fail even when focus, ripples and shape animation work.

For review, show actions next to unselected/selected toggles, and include selected
first/middle/last items in connected groups. Ask which item remains selected
after the pointer leaves. Keep assertions independent of rendering helpers that
resolve the production palette. Establish the failure before fixing it, then
temporarily restore the faulty branch to verify that the new test rejects it.

## Documentation

For executable component previews, run `cargo xtask doc-media build`.
See [generated visual documentation](DOC_MEDIA.md) for the macro registry,
variant policy, CI checks, and preview workflow. Generated images stay out of Git.

The README is the short consumer introduction. Getting-started, cookbook and
theming pages also appear under `guide` in rustdoc, with compiled examples.
Use absolute public links in the README so it renders on crates.io; source-only
guides can use relative links. The embedded
guides should be self-contained: use external URLs or describe source paths as
code, since source Markdown and images are not copied into generated rustdoc.
The crate homepage in `src/lib.rs` uses native Rust API links.

`cargo test --doc` also compiles the README example independently. After moving
examples, verify that the same snippets are still tested and run the strict
rustdoc command above. Generated component snippets require the separate
[doc-media checks](DOC_MEDIA.md), beyond plain checkout doctests.

## Verify the distributable

```sh
cargo xtask check-package
cargo publish --dry-run --locked
```

`check-package` packages and extracts the crate, checks its files/notices, then
tests it and a separate consuming app. Add `--offline` to use cached dependencies.
Neither command uploads. These are source-package development checks; a release
must use the staging crate containing generated documentation media, following
[RELEASING.md](RELEASING.md).
Reference PNGs and the independent consumer are intentionally
excluded from the distributable; ordinary tests and the consumer guides are included.
