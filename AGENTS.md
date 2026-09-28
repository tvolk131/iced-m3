# Working in this repository

- Component visual documentation is generated from the `doc_examples!` registry
  in `tests/visual/doc_media/catalog.rs`. See `docs/DOC_MEDIA.md` before changing
  that pipeline. Generated media stays out of Git.
- Releases follow `docs/RELEASING.md`: version PR, merge to master, green checks
  on the merged commit, prepare the staging crate, then explicit publication.
  Do not publish the repository root; it lacks generated documentation media.
- Generating documentation, packaging, and `cargo publish --dry-run` do not upload.
  Actual publication and release tagging require the user's authorization.
