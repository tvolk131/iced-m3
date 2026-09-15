# Release preparation

The first release candidate is **`iced-m3` 0.1.0-beta.1**, imported as `iced_m3`.
The repository is [tvolk131/iced-m3](https://github.com/tvolk131/iced-m3), on `master`.
The crate has **not been uploaded**. Its manifest allows the crates.io registry
so Cargo can perform a real publication dry run; this setting is not evidence of
publication. Actual upload and release tagging remain separate release actions.

See the [release notes and compatibility policy](../CHANGELOG.md),
[support boundaries](LIMITATIONS.md) and [validation record](VALIDATION.md).
No new component family is required for this desktop beta. Native accessibility,
localization and remaining renderer/fidelity work remain documented limitations.

## Verify a candidate

1. Keep the package name/version, both lockfiles, README installation example,
   getting-started guide, release notes and documentation URL consistent.
2. Require successful CI for the exact commit to release: macOS/Windows/Linux
   checks, minimum Rust, extracted-package consumer and canonical visual tests.
   Headless CI does not certify native Windows/Linux interaction or IME/scaling.
3. Run the archive and consumer checks from the repository:

   ```sh
   python3 tools/check_package.py
   cargo publish --dry-run --locked
   ```

   The first command builds the archive, verifies its inputs and notices, and
   tests an independent application against its extracted contents. It writes
   `target/beta-package-report.json`. The second performs Cargo's publication
   checks without uploading. Both also run in Linux CI. Neither needs a registry
   upload token. Use a clean checkout for the final release verification.
4. Review the archive and public links. README links use public GitHub/raw URLs
   so the screenshot and guides resolve on crates.io. Its registry snippet is
   explicitly marked as available after publication; source installation works
   before then. Check name availability again immediately before a first upload;
   the registry returned “does not exist” for `iced-m3` on September 15, 2026,
   which does not reserve the name.

## Hosted documentation

The planned API destination is
[docs.rs/iced-m3/0.1.0-beta.1](https://docs.rs/iced-m3/0.1.0-beta.1/iced_m3/).
It becomes available only after publication and a successful docs.rs build.
The metadata uses `x86_64-unknown-linux-gnu` and disables default features to
document the same public API without the optional GPU backend. Linux CI checks
that target/feature combination with rustdoc warnings denied. Docs.rs runs its
own service/toolchain, so this is a preflight check rather than a hosted-build
guarantee.

## When publication is authorized

Recheck the exact commit, name availability and dry run, then upload with Cargo
using the maintainer's own crates.io credentials. Verify the registry version and
hosted documentation before creating the matching `v0.1.0-beta.1` Git tag and
GitHub release. Record the publication date in the release notes and remove the
temporary prepublication wording from the source installation instructions.
Do not claim a live registry or docs.rs page before checking it.

## References

- [Cargo publication and dry runs](https://doc.rust-lang.org/cargo/commands/cargo-publish.html)
- [Cargo compatibility conventions](https://doc.rust-lang.org/cargo/reference/semver.html)
- [Docs.rs build metadata](https://docs.rs/about/metadata)
