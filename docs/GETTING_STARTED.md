# Getting started

This library targets released upstream iced 0.14.0 and Rust 1.88 or newer.
Use an ordinary iced application: your state owns values and callbacks produce
messages; the widgets retain focus, editing and animation state between views.

## Install and run

Install from crates.io. Pin the beta exactly
so API changes arrive only when you deliberately upgrade:

```toml
[dependencies]
iced-m3 = "=0.1.0-beta.1"
iced = { version = "=0.14.0", default-features = false, features = ["tiny-skia", "thread-pool"] }
```

Use `iced_m3::Element<'_, Message>` so iced infers the Material theme. `iced::Element<'_, Message>` without its theme parameter defaults to iced's own theme.

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

The complete application is also in `examples/minimal.rs` in the source checkout.
Omitting a button/input/toggle callback makes that control disabled. Values and
business actions belong to your application's state and messages. No animation
messages or subscriptions are needed.

## Rendering and development builds

The default renderer is iced's **wgpu GPU renderer**, with Tiny Skia as a fallback.
Software rendering can become slow around large shadows and layered surfaces on
high-resolution displays, even in release mode; GPU rendering is recommended for
the gallery. Development builds also optimize dependencies. When consuming this
crate, profiles belong to your application's root manifest; add this there for a
faster development preview:

```toml
[profile.dev.package."*"]
opt-level = 2
```

The `wgpu` feature is enabled by default. Cargo combines features across dependencies: `--no-default-features` cannot disable GPU rendering if another dependency enables `iced/wgpu`. This repository uses Rust edition 2024 and supports Rust 1.88 minimum (iced's minimum). CI checks the library and independent consumer on Rust 1.88 and tests on current stable Rust across macOS, Windows and Linux. Native macOS interaction was also checked; native Windows/Linux interaction remains unverified. Linux builds may require `libxkbcommon-dev` and `libwayland-dev`. See [desktop beta readiness](https://github.com/tvolk131/iced-m3/blob/master/docs/BETA_READINESS.md) for executed checks and platform boundaries.

For a software-only application, disable this dependency's default features in
your application's manifest as well:

```toml
[dependencies]
iced-m3 = { version = "=0.1.0-beta.1", default-features = false }
iced = { version = "=0.14.0", default-features = false, features = ["tiny-skia", "thread-pool"] }
```

## Source installation

To work on the library, clone
[the repository](https://github.com/tvolk131/iced-m3) and replace only the
`iced-m3` dependency above with a path to your checkout:

```toml
iced-m3 = { path = "../iced-m3" }
```

For a software-only build, add `default-features = false` to that path dependency.
The package is named `iced-m3`; Rust imports use `iced_m3`. There is no dependency
on the older, unrelated `iced_material` crate. Beta API changes and supported
versions are recorded in the [release notes and compatibility policy](https://github.com/tvolk131/iced-m3/blob/master/CHANGELOG.md).

## Keyboard focus and retained overlays

Wrap the view in `focus::scope` for Tab/Shift+Tab traversal, focus rings and
Enter/Space activation. Navigation groups also handle arrow keys. This supports
keyboard operation; it does not provide a native screen-reader bridge.

Keep dialog, sheet and snackbar hosts mounted across view rebuilds. Use their
open/visible flags when you want exit animations; removing an element immediately
also removes its retained animation state. Keep durable form values in your app.
The cookbook includes complete overlay recipes.
