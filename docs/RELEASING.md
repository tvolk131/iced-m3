# Publication preparation

The library remains unpublished with `publish = false`. Documentation preparation
does not authorize a registry upload or a repository push.

The packaged crate has been built and tested through an independent application.
This is a candidate for a first desktop beta, not a claim of full M3 conformance
or a stable 1.0 API. See [beta readiness](BETA_READINESS.md) and the latest
[validation report](VALIDATION.md) for evidence and platform boundaries.

## Before the first release

1. **Confirm and apply the package name.** `iced-material` conflicts with the
   existing `iced_material` crate. `iced-m3` was proposed and appeared unused when
   checked on September 15, 2026; recheck before release. Renaming must include
   manifests, Rust imports, examples, consumer dependencies, lockfiles and tooling.
   The documentation cleanup does not change the current package/import names.
2. **Establish the repository.** At this review the checkout has no commits or
   configured remote. Choose its owner/location, commit the reviewed source and
   add the real repository URL to package metadata. Do not invent placeholder URLs.
3. **Choose the version and compatibility policy.** A first `0.1.0-beta.1` release
   is a reasonable option. Record API stability expectations, supported iced/Rust
   versions and known limitations in release notes before creating a release tag.
4. **Verify the final documentation destinations.** Once the name and repository
   exist, update installation instructions and public links. Make source-document
   and screenshot links resolve on crates.io as well as the repository. Verify the
   docs.rs target/features and warning-free documentation build; local macOS docs
   are not evidence of a successful docs.rs Linux build.
5. **Run the configured CI.** Obtain successful macOS/Windows/Linux, minimum-Rust,
   packaged-consumer and canonical visual jobs. Native Windows/Linux interaction,
   IME and display-scaling checks remain separate from headless CI; describe
   support conservatively until they have been performed.
6. **Verify the final release archive.** Run the package/consumer checks after
   naming, metadata and documentation changes, then review the archive contents
   and perform Cargo's publication dry run. Keep publication disabled until an
   actual release is explicitly authorized.

No new component family is required for this first beta. Screen readers,
localization and remaining renderer/fidelity work stay explicit limitations.
Subsequent API improvements should be informed by actual consuming applications.

## References

- [Cargo publishing guide](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [Crates.io naming restrictions](https://doc.rust-lang.org/cargo/reference/registry-index.html#name-restrictions)
- [Docs.rs build metadata](https://docs.rs/about/metadata)
