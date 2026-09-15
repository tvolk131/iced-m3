# Northstar Studio

A standalone desktop client of `iced-m3`, kept in this repository for
repeatable integration testing. It has its own manifest, lockfile, features and
profiles; it does not import the gallery, library internals or private test clock.
The two runtime dependencies are public `iced` and `iced-m3` packages.

From the repository root:

```sh
cargo run --locked --manifest-path consumers/desktop/Cargo.toml
cargo run --locked --manifest-path consumers/desktop/Cargo.toml --no-default-features
cargo test --locked --manifest-path consumers/desktop/Cargo.toml --all-targets
```

Use Overview's **Edit workspace** action. Change fields, select access, choose a
native digest option, try team defaults and save. Cancel a changed draft to open
the second dialog; Escape returns to editing with native input focus restored.
Preferences contains an outlined field on a custom gradient plus appearance and
reduced-motion controls. Activity refresh is an ordinary iced button. All data is
in memory, and the example email check is intentionally a small demo validation.

`src/metrics.rs` is a generic application component using only iced text/container
catalogs. It is not a downloaded third-party widget. `widget::themer` scopes an
iced theme around native buttons and the pick-list popup. The Material theme's
native text-input catalog lets the team-note input compose without that adapter.

The test client rebuilds the real view after each event and delivers messages to
the real update function, preserving iced's widget cache. It uses only public
iced runtime, renderer, operation and selector APIs. Keyboard events include
modifier-state changes, just like a window runtime. Native PickList paints labels
without selector text operations, so its test uses an ID on the surrounding
container and the visible menu's position. No library-specific selector escape
hatch or new runtime dependency is needed.

For source-checkout builds, Cargo puts output under this package's `target/` by
default. Set `CARGO_TARGET_DIR` to share the repository's build directory when
desired. The top-level CI explicitly checks this package, since root
`cargo test --all-targets` does not enter its independent workspace.

```sh
ICED_TEST_BACKEND=wgpu cargo test --locked --manifest-path consumers/desktop/Cargo.toml --test flows
CONSUMER_CAPTURES=/absolute/output/path cargo test --locked --manifest-path consumers/desktop/Cargo.toml capture_consumer_review -- --ignored
python3 tools/check_package.py --offline
```

The last command runs from the repository root and checks a copy of this client
against the extracted distributable, including GPU-enabled and software-only
dependency configurations. It never publishes. See `docs/BETA_READINESS.md` in
the repository for the complete scope and remaining platform work.
