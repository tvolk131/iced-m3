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

`Theme::expressive()` opts into shared spring motion and action shape feedback.
Component sizes/variants stay explicit. Read `theme.effective_motion()` in custom
widgets so reduced motion wins without erasing your saved timings. See
[Expressive actions and motion](https://github.com/tvolk131/iced-m3/blob/master/docs/EXPRESSIVE.md)
for precedence, coverage and reference pins.

## Typography and surfaces

Material helpers register bundled Roboto automatically. Set the application's
default font to `fonts::REGULAR` so native iced text matches. `Theme::typography`
owns all 15 baseline roles and 15 independently customizable emphasized roles.
Each `TypeStyle` contains a font (including family, weight, stretch and style),
size, absolute line height and recorded tracking. Existing `TypeScale` metrics,
token constants and free constructors retain their baseline recipes.

### Construct themed labels and editors

Use `theme.widgets()` when building a view. The factory owns a cheap,
copy-on-write typography snapshot; returned widgets do not borrow the theme.
It applies the same snapshot to built-in labels, supporting text, menu/submenu
rows, navigation, picker content, full-screen dialog titles and native editors.

```rust
use iced::{font::Weight, widget};
use iced_m3::{Element, Theme, TypeScale};

let mut theme = Theme::light();
theme.typography[TypeScale::TitleLarge].font.weight = Weight::Bold;
theme.typography[TypeScale::BodyLarge].size = 18.0;
theme.typography[TypeScale::BodyLarge].line_height = 28.0;
theme.typography.emphasized_mut(TypeScale::TitleLarge).size = 26.0;

let material = theme.widgets();
let content: Element<'_, ()> = widget::column![
    material.emphasized("Workspace", TypeScale::TitleLarge),
    material.text_field("Name", "Ada").on_input(|_| ()),
    material.button("Save").on_press(()),
].spacing(16).into();
```

Return the same current theme from the application's `.theme(...)` callback.
**That callback alone does not change typography in already constructed text.**
iced 0.14 supplies the drawing theme after text layout; an explicit factory
avoids a first-frame fallback, delayed relayout or hidden ambient state.
Rebuild the view with a new factory when changing schemes. Native input trees,
focus, selection and animation state are retained by the ordinary iced diff.
Cached/deferred content must also be rebuilt: `Typography` implements `Hash` so
its clone can be included in a lazy view's dependency key. Capture the current
factory in lazy-carousel or responsive builders. Prebuilt elements and arbitrary
custom content keep their own typography; the factory does not rewrite them.

The factory's `typography` and `emphasized` methods return native iced `Text`.
Explicit `.font(...)`, `.size(...)` and `.line_height(...)` builders win. A typed
common-button size chooses the corresponding role from its captured scheme;
`Button::new(custom_content)` preserves that content. Text fields grow for larger
input lines and reserve supporting-text space; chips, tabs, segmented rows and
slider value bubbles accommodate their label's line height. Typography scaling
does not implement a new density or target-size
policy, and large text can still require application layout changes in bounded
navigation, clock/calendar grids and fixed-width compositions.

`Typography::family(...)` changes every role's family while retaining weights;
`Typography::scaled(...)` scales text metrics only. Applications load custom font
bytes through iced. Invalid nonpositive/nonfinite sizes and line heights fall
back to baseline metrics; negative/nonfinite shape sizes fall back to their
default role. Tracking remains metadata: iced's public text API cannot apply it.
`Theme::expressive()` preserves custom typography and shapes.

### Shape roles

`theme.shapes` is a `ShapeScheme` with ExtraSmall (4), Small (8), Medium (12),
Large (16), LargeIncreased (20), ExtraLarge (28), ExtraLargeIncreased (32), and
ExtraExtraLarge (48) roles. `ShapeScale::None` and `Full` retain their structural
meanings. Role names follow the current Material shape scale; the older
`tokens::shape` constants keep their original names and values for compatibility.

```rust
use iced::widget;
use iced_m3::{Element, ShapeScale, Theme, surface};

let mut theme = Theme::light();
theme.shapes.medium = 20.0;
theme.shapes.extra_large = 32.0;
let material = theme.widgets();
let content: Element<'_, ()> = surface(widget::column![
    material.typography("Themed surface", iced_m3::TypeScale::TitleLarge),
    material.button("Themed corners").shape_role(ShapeScale::Medium).on_press(()),
    material.button("Explicit corners").radius(4.0).on_press(()),
]).into();
```

Built-in surfaces, fields, cards, FABs, popup menus, dialogs, sheets, picker
surfaces, tooltips, snackbars and Expressive action corners resolve their roles
from the drawing theme. Their fills, elevation and interaction feedback use the
same resolved corners. `ShapeScheme::{top,left,right}` derive asymmetric edges;
left/right are physical directions, not an RTL implementation. Explicit button
radius/corner overrides and `elevated(content, level, radius)` remain explicit.
Clock faces, checkbox artwork, slider thumbs, badges and other structural circles
are not replaced with arbitrary theme shapes. This is rounded-corner theming,
not cut-corner/custom-path clipping or group-opacity support.

Run `cargo run --example theming` to compare light/dark, custom weights, 125%
text and alternate corners while retaining live editor state.

### Reference profile

The existing baseline is preserved, including recorded legacy tracking values.
Emphasized weights and additional shape roles are checked against AndroidX
Material 3 commit `8fd64ac21a546397597caa1527fd2e589ea2915e`:
`tokens/TypeScaleTokens.kt` (v0_103), `tokens/TypographyTokens.kt` and
`tokens/ShapeTokens.kt` (14_1_0). Emphasized display/headline/body and TitleLarge
use weight 500; emphasized labels and TitleMedium/TitleSmall use 700. Metrics
remain independently customizable. Numerical/default, first-layout, retained
editing, font-cache and renderer contracts live in `tests/visual/theming.rs`;
canonical reference comparisons separately protect existing appearances.

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
