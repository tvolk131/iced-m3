# Release process

The current desktop beta candidate is **`iced-m3` 0.1.0-beta.4**, imported as `iced_m3`.
The repository is [tvolk131/iced-m3](https://github.com/tvolk131/iced-m3), on `master`.
The manifest permits the crates.io registry. CI verifies publication using a
dry run; uploads and release tagging are explicit maintainer actions.

See the [release notes and compatibility policy](../CHANGELOG.md),
[support boundaries](LIMITATIONS.md), [validation record](VALIDATION.md), and
[beta.4 validation](VALIDATION.md#tooltip-and-callback-contracts--2026-09-29).
No new component family is required for this desktop beta. Native accessibility,
localization and remaining renderer/fidelity work remain documented limitations.

## Prepare a release PR

1. Open a PR with the desired version, changelog/release date, the library and
   consumer lockfiles, README installation example, getting-started guide, and
   documentation URL updated consistently. Keep `xtask/Cargo.lock` current if tooling dependencies
   change; the unpublished tool has no release version to bump. Generated
   documentation images are **not committed**.
2. Review the `component-documentation` CI artifact, including motion/variants.
   CI generates images twice, checks exact asset membership and freshness,
   compiles the shown snippets, verifies the actual `.crate` documentation, and
   builds that archive on Linux with the docs.rs settings.
3. Merge the release PR to `master`. Wait for **all checks on the merged commit**,
   including `doc-media` and `packaged-docs-linux`, to pass. Successful checks on
   an earlier PR revision are insufficient.

## Prepare the merged candidate

From a clean, up-to-date checkout of that exact `master` commit:

```sh
git switch master
git pull --ff-only
cargo xtask prepare-release
cargo xtask check-package --source target/doc-media/stage --require-doc-media
```

The command refuses a dirty checkout, another branch, or a commit different
from local `origin/master`. It does not fetch Git refs or attest GitHub status;
verify the fresh remote state and CI yourself. See [tool prerequisites](DOC_MEDIA.md).
It renders the component examples, assembles `target/doc-media/stage`, packages
it, verifies generated docs and doctests from the **extracted archive**, and
runs `cargo publish --dry-run`. Nothing is uploaded.

Review `target/doc/iced_m3/guide/components/index.html` and
`target/doc-media/release.json` (source commit, input hashes and archive checksum).
Do not edit the staged sources or generated assets. If source changes, repeat
preparation from the new merged, CI-approved commit. Retain the existing
independent consumer/platform checks in CI; headless checks do not certify
native accessibility, IME or platform scaling.

## Hosted documentation

The API destination for this beta is
[docs.rs/iced-m3/0.1.0-beta.4](https://docs.rs/iced-m3/0.1.0-beta.4/iced_m3/).
It becomes available only after publication and a successful docs.rs build.
The metadata uses `x86_64-unknown-linux-gnu` and disables default features to
document the same public API without the optional GPU backend. Linux CI checks
that target/feature combination with rustdoc warnings denied. The metadata also
enables `iced_m3_doc_media`, which includes the prepared, self-contained media
fragments. No rendering or downloads are needed by the docs.rs build. Docs.rs runs its
own service/toolchain, so this is a preflight check rather than a hosted-build
guarantee.

## Publish and verify

With release authorization, recheck the exact commit, registry state and dry run,
then upload the prepared staging crate using the maintainer's crates.io credentials.
Do **not** publish directly from the repository root: generated media is absent
there. We intentionally rely on this documented workflow rather than a build-script
guard or a publish hook:

```sh
cargo xtask doc-media verify --root target/doc-media/stage
cargo publish --manifest-path target/doc-media/stage/Cargo.toml --locked --registry crates-io
```

Verify the registry version, its archived source revision, an independent
application using the registry dependency, and the hosted documentation. Then
create the matching `v0.1.0-beta.4` Git tag and GitHub prerelease from that exact
commit. Keep later documentation changes separate from the released source.

## References

- [Cargo publication and dry runs](https://doc.rust-lang.org/cargo/commands/cargo-publish.html)
- [Cargo compatibility conventions](https://doc.rust-lang.org/cargo/reference/semver.html)
- [Docs.rs build metadata](https://docs.rs/about/metadata)
