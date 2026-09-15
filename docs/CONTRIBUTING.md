# Development and validation

Run commands from the source checkout. Rust 1.88 is the supported minimum;
see [executed checks and platform boundaries](BETA_READINESS.md). The workflow is
in `.github/workflows/ci.yml`; check the exact candidate's run in
[GitHub Actions](https://github.com/tvolk131/iced-m3/actions).

## Routine checks

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo test --locked --doc
cargo build --locked --examples
cargo test --locked --all-targets --no-default-features
cargo doc --locked --no-deps
```

The tests use **iced_test 0.14** and real headless rendering. They cover transitions, disabled behavior, message emission, drag cancellation, text entry/selection/focus persistence, dialog event capture, long-dialog scrolling, popup placement after scrolling, nested dialog menus, snackbar timeouts/pause/Undo, tooltip delay, radio selection, tab/rail overflow, nested list actions, navigation across gallery rebuilds, and pixel clipping.

The visual regression suite compares **2,665 reference images** across every current
component family, including precisely timed hover, held-mouse, drag, release and progress frames.
A private test clock makes animation captures independent of rendering speed.
Run it separately on the canonical macOS 26 / Apple Silicon environment:

```sh
cargo test --locked --no-default-features --all-targets visual_references -- --ignored
python3 tests/visual/report.py
```

Open `target/visual-report/index.html` to filter components, scrub captured frames
and inspect expected/actual/difference images. Changed or missing references fail;
CI runs a dedicated visual job and uploads the report. See
[visual test coverage, timing and reference updates](VISUAL_TESTS.md).

The earlier manual visual generators remain available separately:

```sh
cargo test --locked --no-default-features --test interactions milestone_one_snapshots -- --ignored
cargo test --locked --no-default-features --example gallery gallery_snapshots -- --ignored
cargo test --locked --no-default-features --test workflows workflow_snapshots -- --ignored
cargo test --locked --no-default-features --test navigation navigation_snapshots -- --ignored
# Optional GPU comparison on a machine with an available graphics device:
ICED_TEST_BACKEND=wgpu cargo test --locked --example gallery gallery_snapshots -- --ignored
```

Outputs are in `target/visuals/`. These are visual inspection artifacts, not portable pixel-golden assertions. See [docs/VALIDATION.md](VALIDATION.md) for the executed checks, native inspection, and remaining platform limits.

## Documentation

The README is the short consumer introduction. Getting-started, cookbook and
theming pages also appear under `guide` in rustdoc, with compiled examples.
Use absolute public links in the README so it renders on crates.io; source-only
guides can use relative links. The three embedded
guides should be self-contained: use external URLs or describe source paths as
code, since source Markdown and images are not copied into generated rustdoc.
The crate homepage in `src/lib.rs` uses native Rust API links.

`cargo test --doc` also compiles the README example independently. After moving
examples, verify the doctest count and build docs with warnings treated as errors:

```sh
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps
```

## Verify the distributable

```sh
python3 tools/check_package.py
cargo publish --dry-run --locked
```

This packages and extracts the crate, checks its files and notices, then tests the
package and a separate consuming app. Add `--offline` to use cached dependencies.
Neither command uploads. Linux CI also checks the docs.rs target and feature
configuration before verifying the archive and publication dry run.
Reference PNGs and the independent consumer are intentionally
excluded from the distributable; ordinary tests and the consumer guides are included.
See [publication preparation](RELEASING.md) for the remaining release steps.
