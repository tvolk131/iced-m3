# Official Material Symbols

The complete **4,157-name** catalog from Google's
[`material-design-icons`](https://github.com/google/material-design-icons/tree/10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7/symbols/web)
repository, pinned to **`10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7`**.
The original SVG bytes are preserved, including their paths and metadata.

Coordinates: **Outlined and Rounded**, **FILL 0 and 1**, **optical sizes
20/24/40/48**, **weight 400**, **grade 0**: **66,512 selectable variants**.
Identical byte sequences share storage. Sharp and other weights/grades are not
bundled. `LICENSE` and `NOTICE` cover all the bundled Google artwork.

## Storage format

`symbols.bin` is generated, checked in, and published directly. Consumer builds
perform no SVG generation, extraction or downloads. `catalog.json`
records the source revision, upstream archive SHA-256, ordered names, coordinates
and the SHA-256 of all original SVG bytes concatenated in variant order.
`SHA256SUMS` covers the JSON inventory, compressed bundle and upstream license.

`catalog.json` is the single committed name inventory. The companion's `build.rs`
uses the shared `codegen.rs` helper to generate typed Rust names into Cargo's
`OUT_DIR`; generated `catalog.rs` is neither committed nor packaged. Only the
small JSON-to-Rust step runs during consumer builds, using a JSON-parsing build
dependency. It writes only to `OUT_DIR` and Cargo reruns it when its inputs change.

All integers are little-endian u32. The bundle consists of:

1. An 8-byte `M3SYM001` magic, variant count, chunk count and payload offset.
2. One 12-byte `(chunk, decoded offset, SVG length)` record per variant.
3. One 12-byte `(payload offset, compressed length, decoded length)` record per chunk.
4. Independently gzip-compressed chunks, each at most **65,536 decoded bytes**.

Variant order is name (alphabetical), family (Outlined/Rounded), fill (0/1), then
optical size (20/24/40/48). Payload offsets are relative to the payload start.
The runtime copies each requested SVG into a cached iced handle. A separate
16-entry LRU cache bounds retained decoded chunks to 1 MiB; evicting a chunk does
not invalidate handles. Handle memory grows with the unique artwork requested.

## Maintenance

Verify every recovered variant and validate the name inventory offline:

```sh
cargo xtask material-symbols verify
```

To regenerate, download the pinned upstream archive, then run:

```sh
curl --fail --location --output target/material-design-icons.tar.gz \
  https://codeload.github.com/google/material-design-icons/tar.gz/10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7
cargo xtask material-symbols generate --archive target/material-design-icons.tar.gz
```

These commands use the repository's unpublished Rust `xtask`; archive processing
and compression do not run in consumer builds. Use `cargo xtask material-symbols verify --root
PATH` to verify an extracted companion crate from the matching repository checkout.
Generation reads the archive as a stream and keeps only the chosen assets; no
upstream file tree is extracted. It rejects missing variants, duplicate names,
Rust-name collisions and changes to the pinned original SVG corpus checksum.
Changes to the pin require reviewing membership and updating the revision,
expected name count and original-corpus checksum in `xtask/src/material_symbols.rs`.
