# Theming and native widgets

## Colors and motion

```rust
use iced::{Color, Font};
use iced_m3::{Theme, TypeScale, typography};
use std::time::Duration;

let mut theme = Theme::from_accent(Color::from_rgb8(0, 106, 106), true);
theme.motion.short = Duration::from_millis(120);
theme.colors.error = Color::from_rgb8(255, 180, 171);
let heading = typography("A new workspace", TypeScale::Headline).font(Font::DEFAULT);
```

Return your theme from the application's `.theme(...)` callback. All custom components share semantic roles. Accent generation uses HCT tonal-spot colors from the pinned Rust port of Material color utilities. `Theme::from_accent_with_contrast(accent, dark, level)` sets contrast in -1..1; `.reduced_motion(true)` disables shared animation. At standard contrast, generated foreground/background role pairs are tested for at least 4.5:1 contrast. Custom role overrides are the application's responsibility.

## Typography and surfaces

Material helpers register bundled Roboto automatically. Set the application's
default font to `fonts::REGULAR` so native iced text matches. `TypeScale` contains
all 15 baseline Material roles; individual builders accept font overrides.

Layout tokens live in `tokens::{spacing, shape, size, TypeScale}`; elevation comes from `Theme::elevation`, and animation timing from `Theme::motion`. This release has a fixed shared layout scale rather than a runtime density system. Use typed variant, width, height, padding, radius, font, and native style builders where available. Outlined fields and selects are transparent by default. Their floating label uses a real gap in the border, so they compose on cards, dialogs, custom colors and patterned backgrounds without matching a parent color. Filled fields retain their own container fill. Use `.background(...)` or `.background_with(...)` only for an intentional field fill; the override does not paint outside the field behind its label.

## Native and third-party widgets

The Material theme implements iced catalogs for text, containers, text inputs, SVGs, scrollables, checkboxes and rules. Layout widgets such as rows, columns, wrapping rows, containers, scrollables and stacks compose directly. For native widgets requiring additional catalogs, use `iced::widget::themer` with `Theme::iced()`; that method produces an iced theme from the semantic palette. A consumer cannot implement iced's foreign catalog trait on this crate's foreign `Theme` type. The library uses iced's default renderer type, including when Cargo feature unification selects its GPU renderer.

```rust
use iced::widget;
use iced_m3::{Element, Theme, text_field};

#[derive(Clone)]
enum Message { Note(String), Refresh }

let theme = Theme::dark();
let native: iced::Element<'_, Message> = widget::button("Refresh")
    .on_press(Message::Refresh).into();
let mixed: Element<'_, Message> = widget::column![
    text_field("Note", "").on_input(Message::Note),
    widget::themer(Some(theme.iced()), native),
].into();
```

Build the adapter from the current theme when rebuilding the view so accent and
light/dark changes propagate. It forwards messages, operations and native popups;
native widgets retain their own appearance and interaction behavior. Focus
traversal depends on whether each widget exposes iced focus operations. A
third-party widget can compose directly if it accepts the Material theme's
existing catalogs; otherwise scope its supported theme in the same way.
