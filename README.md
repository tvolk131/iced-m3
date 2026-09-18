# iced-m3

Material 3 components for desktop applications built with **upstream iced 0.14**.
Compose typed controls with ordinary iced layouts, semantic light/dark themes,
bundled Roboto, keyboard navigation, and animated interaction feedback.

**Status:** desktop beta; Rust **1.88+**. The API is still evolving; see the
[compatibility policy](https://github.com/tvolk131/iced-m3/blob/master/CHANGELOG.md#compatibility).

![Northstar Studio sample application with tabs, cards, and wavy progress](https://raw.githubusercontent.com/tvolk131/iced-m3/master/docs/desktop-beta.png)

*Northstar Studio combines Material controls with native iced widgets.*

## Quick start

Install from crates.io:

```toml
[dependencies]
iced-m3 = "=0.1.0-beta.2"
iced = { version = "=0.14.0", default-features = false, features = ["tiny-skia", "thread-pool"] }
```

For a local checkout, see [source installation](https://github.com/tvolk131/iced-m3/blob/master/docs/GETTING_STARTED.md#source-installation).

```rust,no_run
use iced::widget::{column, container};
use iced_m3::{button, fonts, text_field, Element, Theme};

#[derive(Default)]
struct App { name: String, saved: bool }

#[derive(Debug, Clone)]
enum Message { Name(String), Save }

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::Name(name) => { self.name = name; self.saved = false; }
            Message::Save => self.saved = true,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        iced_m3::focus::scope(container(column![
            text_field("Your name", &self.name).on_input(Message::Name),
            button(if self.saved { "Saved" } else { "Save" })
                .on_press(Message::Save).disabled(self.name.trim().is_empty()),
         ].spacing(24)).padding(32))
    }
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .default_font(fonts::REGULAR)
        .theme(|_: &App| Theme::light())
        .title("Material example")
        .run()
}
```

This complete example is also in [examples/minimal.rs](https://github.com/tvolk131/iced-m3/blob/master/examples/minimal.rs).
Your app owns values and messages; widgets handle their own animation scheduling.
Omitting a control's callback disables it.

## Components

| Area | Includes |
| --- | --- |
| Actions | Buttons, icon buttons, FABs, chips, segmented buttons |
| Input | Text fields, checkboxes, switches, radios, selects, sliders, date/time pickers |
| Navigation | App bars, tabs, navigation bars/rails, search, menus |
| Content and feedback | Cards, lists, carousels, dialogs, sheets, snackbars, tooltips, badges, progress and loading indicators |
| Foundations | Typography, surfaces, dividers, accent colors, elevation, reduced motion |

See the [component catalog](https://github.com/tvolk131/iced-m3/blob/master/docs/COMPONENTS.md) for variants and the
[cookbook](https://github.com/tvolk131/iced-m3/blob/master/docs/COOKBOOK.md) for composed examples.

## Integrating with iced

Use `iced_m3::Element` and return a `Theme` from your application's theme
callback. Set `fonts::REGULAR` as the default font to match native iced text.
Wrap the view in `focus::scope` for keyboard traversal and activation.

The default `wgpu` feature enables GPU rendering with a software fallback.
See [getting started](https://github.com/tvolk131/iced-m3/blob/master/docs/GETTING_STARTED.md) for software-only configuration
and faster development builds. Native and third-party iced widgets can share the
view; the [theming guide](https://github.com/tvolk131/iced-m3/blob/master/docs/THEMING.md) explains the supported catalogs and
`Theme::iced()` adapter.

## Try it and learn more

From the source checkout:

```sh
cargo run --locked --example gallery
```

- [Getting started](https://github.com/tvolk131/iced-m3/blob/master/docs/GETTING_STARTED.md) — application setup and rendering.
- [Cookbook](https://github.com/tvolk131/iced-m3/blob/master/docs/COOKBOOK.md) — controls, overlays, navigation and feedback.
- [Theming](https://github.com/tvolk131/iced-m3/blob/master/docs/THEMING.md) — colors, typography and native widget integration.
- [Gallery guide](https://github.com/tvolk131/iced-m3/blob/master/docs/GALLERY.md) — walkthrough and independent sample app.
- [Documentation index](https://github.com/tvolk131/iced-m3/blob/master/docs/README.md) — API docs, development and reference guides.

Run `cargo doc --open --no-deps` for the API reference and embedded consumer guides.

## Support and license

Desktop keyboard and pointer interaction are the focus. CI builds and tests on
macOS, Windows and Linux; native interaction has been checked on macOS. Native
screen-reader integration, full localization/RTL and some Material variants remain
unfinished. See [support and limitations](https://github.com/tvolk131/iced-m3/blob/master/docs/LIMITATIONS.md) before adopting.

Rust code is [MIT licensed](https://github.com/tvolk131/iced-m3/blob/master/LICENSE). Bundled fonts and loading assets have their
own licenses; [NOTICE](https://github.com/tvolk131/iced-m3/blob/master/NOTICE) identifies them and the notices to retain.
