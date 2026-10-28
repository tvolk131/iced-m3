# iced-m3-material-symbols

Optional typed Material Symbols catalog and official asset bundle for
[iced-m3](https://github.com/tvolk131/iced-m3).
Enable iced-m3's `material-symbols` feature to use the typed icon API and shared
renderer. Applications do not need a direct dependency on this crate.

Contains all 4,157 names at the pinned Google revision, Outlined and Rounded,
both fills, optical sizes 20/24/40/48, fixed weight 400 and grade 0.
The 66,512 selectable variants retain the original SVG bytes, deduplicated and
packed into independently compressed chunks of at most 64 KiB decoded data.
The crate exposes typed names, style/fill/optical-size enums and cached iced SVG
handles. iced-m3 adds the theme-aware Icon wrapper. A small local build script
generates the typed API from bundled JSON into Cargo's `OUT_DIR`. Everything
works offline, without downloading or extracting SVGs during builds or runtime.

See [asset provenance and storage format](assets/README.md),
[Google's license](assets/LICENSE), and [attribution](assets/NOTICE).

Rust code is [MIT licensed](LICENSE-MIT); Google artwork retains its Apache 2.0
license and attribution.
