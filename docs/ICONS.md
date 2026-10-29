# Icons and Material Symbols

Enable the optional `material-symbols` feature to use the complete pinned, typed catalog of
official Google SVGs with iced-m3's existing `Icon` renderer:

```toml
iced-m3 = { version = "=0.1.0-beta.6", features = ["material-symbols"] }
```

From the source checkout, run the small navigation example:

```sh
cargo run --locked --example material_symbols --features material-symbols
# Software renderer only:
cargo run --locked --no-default-features --example material_symbols --features material-symbols
```

The assets are embedded and work offline. There are no network requests,
icon fonts, font-registration steps, or ligatures. The optional `flate2`
dependency uses its pure Rust backend for in-memory decoding. The feature is off by default, so ordinary applications do not compile
or embed this catalog. The assets live in the optional `iced-m3-material-symbols`
companion crate, so ordinary builds with the feature off do not download that
bundle. Enabling the integration embeds the complete compressed bundle when
symbols are used; the linker cannot remove individual icons from it.

The companion's small `build.rs` reads its bundled `catalog.json` and generates
the typed Rust names into Cargo's `OUT_DIR`. This deterministic step uses a
JSON-parsing build dependency, writes no source files and performs no downloads
or SVG generation/extraction. The application never parses JSON at runtime.

Docs.rs enables this feature so its API pages include `material_symbols`.

## Defaults and supported variants

| Property | Default | Supported values |
| --- | --- | --- |
| Style | Outlined | `Style::Outlined`, `Style::Rounded` |
| Weight (`wght`) | 400 | Fixed at 400 |
| Optical size (`opsz`) | 24 | `OpticalSize::Size20`, `Size24`, `Size40`, `Size48` |
| Grade (`GRAD`) | 0 | Fixed at 0 |
| Fill (`FILL`) | 0, unfilled | `Fill::Unfilled` (0), `Fill::Filled` (1) |
| Rendered size | 24 logical pixels | `Icon::size(...)` |

Rendering at 18 or 36 pixels scales the same optical-size-24 artwork; it does not
select or synthesize another optical size or weight. Use `Symbol::optical_size`
to select the official artwork separately from `Icon::size`. Optical size refers
to logical viewing size, independent of display density. Each coordinate uses
the corresponding official SVG. Some symbols have identical artwork for both
fills. Style names describe the design family; unfilled does not mean every
shape is hollow.

```rust
# #[cfg(feature = "material-symbols")]
# {
use iced_m3::{icon, Icon};
use iced_m3::material_symbols::{Fill, IconName, OpticalSize, Style, Symbol};

let default_home: Icon = icon(IconName::Home);
let rounded_filled: Icon = Symbol::new(IconName::Home)
    .style(Style::Rounded)
    .fill(Fill::Filled)
    .optical_size(OpticalSize::Size40)
    .icon()
    .size(40);
# }
```

`IconName::ALL` lists all **4,157 names** in the pinned upstream catalog.
`IconName::name()` gives Google's snake_case name, and `IconName::from_name(...)`
looks up that exact spelling. Rust variants use PascalCase; names starting with a
digit have an `Icon` prefix (`10k` becomes `IconName::Icon10k`).

The catalog has **66,512 selectable variants**: 4,157 names × two families × two
fills × four optical sizes. Byte-identical SVGs share storage and cached handles.
The original SVG bytes are packed into independently gzip-compressed chunks of
at most 64 KiB decoded data. First use decodes only the needed chunk in memory.
Handles retain their own SVG bytes for the process lifetime, so repeated requests
reuse the renderer cache key. A separate 16-entry LRU cache retains at most
1 MiB of decoded chunks; handle memory grows with the unique artwork requested.
No SVG files are extracted during builds or runtime.

## Selected navigation icons

`NavigationItem::new` supplies the unselected icon. `.selected_icon(...)` overrides
the selected artwork; without it, both states use the original icon. This applies
to bottom navigation, collapsed/expanded rails, and modal rails. Selection and
routing remain application-owned. Selected icons also apply to selected disabled
items, using the usual disabled foreground.

```rust
# #[cfg(feature = "material-symbols")]
# {
use iced_m3::{Element, NavigationItem, navigation_rail};
use iced_m3::material_symbols::{Fill, IconName, Symbol};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page { Home, Inbox, Settings }
#[derive(Clone)]
enum Message { Select(Page) }

fn rail(selected: Page) -> Element<'static, Message> {
    let items = [
        (Page::Home, "Home", IconName::Home),
        (Page::Inbox, "Inbox", IconName::Inbox),
        (Page::Settings, "Settings", IconName::Settings),
    ].map(|(page, label, name)| {
        let symbol = Symbol::new(name);
        NavigationItem::new(page, label, symbol)
            .selected_icon(symbol.fill(Fill::Filled))
    });
    navigation_rail(items, Some(selected)).on_select(Message::Select).into()
}
# }
```

`Symbol` converts to the shared `Icon` widget. Both variants inherit the enclosing
component's semantic foreground and disabled alpha, and SVG handles are cached.
No custom renderer or hardcoded color is needed. The selected-icon API accepts
any passive element and is available even with the feature disabled.

## Custom icons and licensing

For Sharp, another weight/grade, or artwork outside the pinned catalog, download the desired
official SVG into your own application, then use
`icon(iced::advanced::svg::Handle::from_memory(include_bytes!("my_icon.svg").as_slice()))`.
The same helper accepts custom square monochrome SVGs. Keep the upstream license
and attribution with downloaded assets. Runtime asset discovery and arbitrary
font-axis interpolation are outside this integration.

Bundled SVGs are unmodified files from `google/material-design-icons` at commit
`10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7`, licensed under Apache 2.0. The
`crates/material-symbols/assets/` directory in the source checkout contains the license, provenance and SHA-256
checksums and a manifest authenticating the recovered original SVG bytes. Use `material_symbols::LICENSE` and `material_symbols::NOTICE` for
application acknowledgments. From the repository checkout, maintainers can verify
the bundle without network access using `cargo xtask material-symbols verify`.
