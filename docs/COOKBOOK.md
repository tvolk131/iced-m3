# Component cookbook

Recipes for application-owned values, messages, and composed Material controls.
Every Rust example on this page is compiled by `cargo test --doc`; the page is
also included under `guide::cookbook` in the generated API documentation.

- [Value controls and progress](#value-controls-and-progress)
- [Sheets and primary actions](#sheets-and-primary-actions)
- [Navigation and lists](#navigation-and-list-composition)
- [Dialogs](#a-dialog-with-actions)
- [Typography and checkboxes](#typography-and-checkbox-states)
- [Menus, choices and hints](#menus-choices-and-hints)
- [Snackbar feedback](#snackbar-feedback)
- [Component composition](#component-composition)
- [Desktop overlays](#desktop-overlays)

## Value controls and progress

See Search and filtering (`docs/SEARCH.md` in the source checkout) for the search/chip/range APIs,
interaction details, animation timings and current scope.

```rust
use iced_m3::{Element, search_bar, list, list_item, filter_chip, range_slider};
use iced::widget::column;

#[derive(Clone)]
enum Message { Open, Close, Query(String), Result, Shared, Age((f32, f32)) }

fn filters(query: &str, open: bool, shared: bool, age: (f32, f32)) -> Element<'static, Message> {
    column![
        search_bar("Search files", query).open(open)
            .on_open(Message::Open).on_close(Message::Close).on_input(Message::Query)
            .results(list([list_item("Design notes").on_press(Message::Result).into()])),
        filter_chip("Shared", shared).on_press(Message::Shared),
        range_slider(0.0..=30.0, age).step(1.0).labeled(true).on_change(Message::Age),
    ].spacing(16).into()
}
```

```rust
use iced::{Length, widget::column};
use iced_m3::{Element, Segment, SegmentSelection, slider,
    segmented_buttons, linear_progress, circular_progress};

#[derive(Clone)]
enum Message { Format(SegmentSelection<u8>), Quality(f32), QualityCommitted }

fn controls(format: u8, quality: f32, progress: Option<f32>) -> Element<'static, Message> {
    column![
        segmented_buttons([
            Segment::new(0, "PNG"), Segment::new(1, "JPEG"),
        ], SegmentSelection::Single(Some(format)))
            .width(Length::Fill).on_change(Message::Format),
        slider(0.0..=100.0, quality).step(5.0).ticks(true).labeled(true)
            .on_change(Message::Quality).on_release(Message::QualityCommitted),
        linear_progress(progress.unwrap_or(0.0)).indeterminate(progress.is_none()),
        circular_progress(progress.unwrap_or(0.0)).indeterminate(progress.is_none()),
    ].spacing(16).into()
}
```

Store callback values in your application and rebuild the view. Use
`SegmentSelection::Multiple(vec![...])` for independently toggled segments; its
callback reports the new complete selection. Item values should be unique.
Labeled sliders reserve 32px above their 48px interaction area. Range labels
separate when close together; focus or drag shows both values without moving
nearby controls. Measured progress uses a critically damped spring and finishes
the current loading sequence when switching from indeterminate mode. See the
value controls milestone (`docs/VALUES.md` in the source checkout) for timing, validation and scope.

Use `.wavy(true)` on either progress builder for the Expressive profile. Linear
waves reserve 10px by default (4px stroke plus wave height); circular waves keep
their allocated diameter and show the inactive track while loading. Optional
`.wave_amplitude(...)`, `.wavelength(...)` and `.wave_speed(...)` customize the
effect. `loading_indicator().contained(true)` uses the canonical seven-shape
morph sequence. Pause, reduced motion and modal-background animation are covered
by timed tests. Details and examples (`docs/LOADING_PROGRESS.md` in the source checkout).

## Sheets and primary actions

```rust
use iced::{Length, widget::{column, svg::Handle}};
use iced_m3::{Element, Theme, TypeScale, button, extended_fab, icon,
    sheet, side_sheet, text_field, typography};

#[derive(Clone)]
enum Message { OpenDetails, CloseDetails, Note(String), NewWorkspace }

fn workspace<'a>(open: bool, note: &'a str, add_svg: Handle)
    -> Element<'a, Message>
{
    let background = column![
        button("File details").on_press(Message::OpenDetails),
        extended_fab(icon(add_svg), "New workspace").on_press(Message::NewWorkspace),
    ].spacing(24).width(Length::Fill).height(Length::Fill);
    sheet::host(background, side_sheet(column![
        typography("File details", TypeScale::HeadlineSmall),
        text_field("Note", note).on_input(Message::Note),
        button("Close").on_press(Message::CloseDetails),
    ].spacing(24)).open(open).on_dismiss(Message::CloseDetails))
}
```

Keep `sheet::host` and its content present while closed; set `.open(false)` so
the exit can finish. Standard side sheets reserve space for the panel and keep
the background interactive. `.modal()` adds a scrim and blocks background input
until the exit ends; `bottom_sheet` is always modal. Put an enclosing dialog host
outside the sheet host, and snackbars inside it to pause their timers while modal.
Use `icon(svg_handle)` with centered artwork in a square SVG view box. Supply
24×24 logical-pixel icons for small, regular and extended FABs; use
`fab(icon(svg_handle).size(36)).size(FabSize::Large)` for a large FAB. Extended
FABs keep their 24×24 icon and 56px height when collapsed. Custom widgets remain
supported, but the FAB centers the supplied layout box without scaling the artwork
or adjusting its optical alignment. Text glyphs such as `text("+")` can look
off-center because of their font baseline and line height. The app owns placement,
including any floating stack.
See sheet and FAB behavior, tokens, and limits (`docs/SHEETS.md` in the source checkout).

## Navigation and list composition

```rust
use iced::{Length, widget::{column, row}};
use iced_m3::{Element, NavigationItem, Tab, TabVariant, app_bar, button,
    checkbox, list, list_item, navigation_rail, tabs};

#[derive(Clone)]
enum Message { Page(u8), Tab(u8), Read(bool), More }

fn screen(page: u8, tab: u8, read: bool, icons: [Element<'static, Message>; 2])
    -> Element<'static, Message>
{
    let [workspace, activity] = icons;
    row![
        navigation_rail([
            NavigationItem::new(1, "Workspace", workspace),
            NavigationItem::new(2, "Activity", activity).badge(3),
        ], Some(page)).on_select(Message::Page),
        column![
            app_bar("Workspace").action(button("More").on_press(Message::More)),
            tabs([Tab::new(1, "Overview"), Tab::new(2, "Files")], Some(tab))
                .variant(TabVariant::Secondary).on_select(Message::Tab),
            list([
                list_item("Design notes").supporting_text("Updated today")
                    .trailing(checkbox(read).on_toggle(Message::Read))
                    .on_press(Message::Read(!read)).into(),
            ]),
        ].width(Length::Fill),
    ].height(Length::Fill).into()
}
```

Tab and rail callbacks report the selected value, including a click on the current
value. The app chooses the displayed page; these components do not own a router.
Callbacks should construct messages without side effects. Use concise labels for
equal-width tabs, or `.scrollable(true)` for natural widths and horizontal overflow.
Programmatic selection does not automatically scroll a tab into view. Rail header
content scrolls with destinations; footer actions stay at the bottom.

Tabs and lists are transparent by default and preserve their enclosing surface.
For an intentional tab fill, use `.background(color_or_gradient)` or
`.background_with(|theme| theme.colors.surface)`; the fill stays fixed while
labels scroll. Lists return a native container with its usual `.style(...)`
background override. See surface composition and examples (`docs/COMPOSITION.md` in the source checkout).

A list row without `on_press` is static and its children still work. Leading and
trailing controls receive input before the row; clicking a checkbox, menu or button
emits its action once. `.disabled(true)` blocks the entire row and cancels pending
child gestures. Also disable custom child widgets in your view for their own
disabled appearance. Size icons, avatars and images before passing them in. Lists
grow to fit wrapped text and compose with native scrollables; they are not virtualized.
See navigation design and scope (`docs/NAVIGATION.md` in the source checkout).

## A dialog with actions

Keep `modal` and its dialog content at the **root** of your view on every rebuild,
including when closed. Change the `open` flag to animate opening and closing while
preserving widget state. The overlay covers the whole window and blocks background
input until the exit animation finishes. A dialog first mounted open starts fully
visible; subsequent changes animate. Closing is an application message.

```rust
use iced::{Length, widget::{column, container, row}};
use iced_m3::{button, ButtonVariant, dialog::{actions, dialog, modal}, typography, TypeScale, Element};

#[derive(Clone)]
enum Message { Cancel, Confirm }

fn view(open: bool) -> Element<'static, Message> {
    let background = typography("Workspace", TypeScale::Headline);
    modal(background,
        dialog(column![
            typography("Save changes?", TypeScale::HeadlineSmall),
            typography("Your preferences will be updated.", TypeScale::BodyMedium),
            actions(container(row![
                button("Cancel").variant(ButtonVariant::Text).on_press(Message::Cancel),
                button("Save").variant(ButtonVariant::Text).on_press(Message::Confirm),
            ].spacing(8)).align_right(Length::Fill)),
        ].spacing(24))
        .on_dismiss(Message::Cancel)
        .dismiss_on_outside(false),
        open,
    )
}
```

`actions(...)` marks the action row for its later entrance fade. Use
`dialog::stack` for multiple dialogs, keeping each entry mounted with its own
open flag. If the dialog displays optional application data, retain that data
through closing so its content remains available for the exit animation.

`dismiss_on_escape(false)` disables Escape independently. Missing `on_dismiss` means no implicit dismissal. Scrim dismissal requires a left press **and release** outside, so dragging from the dialog does not dismiss it. Input events are captured even when dismissal is disabled. Background raw-event subscriptions are still application code: if you use `iced::event::listen_raw`, respect captured events and your dialog-open state.

## Typography and checkbox states

Material helpers automatically register the bundled Roboto font once. Body and
headings use Regular (400); action labels and smaller titles use Medium (500).
Set `.default_font(iced_m3::fonts::REGULAR)` on your iced application to
match ordinary iced text widgets too. Override individual fonts with `.font(...)`;
text fields apply that override to their value, label and supporting text.

`TypeScale` provides Large, Medium and Small roles for Display, Headline, Title,
Body and Label. The original `Display`, `Headline`, `Title`, `Body`, `Label` and
`Supporting` names remain aliases. See the full scale and visual comparison (`docs/MATERIAL_AUDIT.md` in the source checkout).
M3 tracking values are available through `TypeScale::tracking()`, but iced 0.14
cannot apply letter spacing through its native text/input APIs.

For group selection, use `.indeterminate(some_selected && !all_selected)` on a
checkbox. Use `.error(invalid)` for error styling. The application owns both the
checked and mixed values; `.on_toggle(...)` reports the new Boolean value on
release. Its label and padded target are clickable. Disabling cancels pending
presses and removes hover feedback. Selection motion uses
`theme.motion.checkbox_select`; hover/exit and ripple release fading use `short`,
and mixed-mark motion also uses `checkbox_select`. Fill/mark opacity fades over
`short / 3` (50ms by default). Checkbox and radio ripples expand from the center
over `ripple_expand` (450 ms by default), remain visible while held, then fade over
150 ms. Quick clicks retain feedback for at least half the expansion duration
before fading; their action message still arrives immediately on release.
The hover circle remains visible beneath the ripple while the pointer is over
the control; release fades the ripple back to that same hover appearance.

The helper returns this library's `Checkbox`, with label, sizing, font and callback
builders. Native iced `.style`, `.class` and `.icon` methods are not part of this
builder; a native iced checkbox can still use the Material theme's catalog.

## Menus, choices and hints

```rust
use iced::widget::column;
use iced_m3::{button, menu, radio_group, select, tooltip, Element,
    MenuItem, RadioOption, SelectOption};

#[derive(Clone)]
enum Message { Duplicate, Save, Delivery(u8), Access(u8) }

fn controls(delivery: u8, access: u8) -> Element<'static, Message> {
    column![
        menu("Workspace actions", [
            MenuItem::new("Duplicate", Message::Duplicate),
            MenuItem::separator(),
            MenuItem::new("Save", Message::Save),
        ]),
        radio_group([
            RadioOption::new(1, "Daily"),
            RadioOption::new(2, "Weekly"),
        ], Some(delivery)).on_select(Message::Delivery),
        select("Workspace access", [
            SelectOption::new(1, "Viewer"),
            SelectOption::new(2, "Editor"),
            SelectOption::new(3, "Owner (managed)").disabled(true),
        ], Some(access)).on_select(Message::Access),
        tooltip(button("Save").on_press(Message::Save), "Save these preferences"),
    ].spacing(16).into()
}
```

Radio and select messages report a value; update your model to display the new
selection. Clicking an already selected radio does nothing. Missing callbacks
disable these controls. Groups lay out vertically; individual radios compose in
ordinary rows. Selection callbacks should construct messages without side effects.

Menus open on release, scroll when tall, and flip/clamp near window edges. A
matching outside press/release, outside wheel, Escape or window deactivation
closes the menu. Outside dismissal captures the event. Use `.placement(...)`,
`.width(...)`, and `.max_height(...)` to adjust a menu; `Placement` is shared with
tooltips. `MenuItem` supports `.leading(passive_icon)`, `.shortcut("⌘D")`,
`.selected(...)`, `.destructive(...)` and `.disabled(...)`. Shortcut text is a
visual hint; the app owns the actual binding. `context_menu` preserves normal
interaction with its content and opens on right-click. `menu_button` takes
passive trigger content. `MenuItem::submenu(label, items)` creates cascading menus. Up/Down and Home/End move focus; Right/Enter/Space open a branch, and Left/Escape return to its parent. Selecting a leaf closes the whole chain. See desktop behavior (`docs/DESKTOP.md` in the source checkout).

Tooltips accept `.delay(Duration)`, `.placement(Placement)` and `.disabled(bool)`.
They disappear on press, Escape or pointer exit; pressing still reaches the
wrapped control. A dismissed hint stays hidden until the pointer leaves. No
application subscription is needed for their delay.

## Snackbar feedback

```rust
use iced::{Length, widget::container};
use iced_m3::{Element, snackbar};

#[derive(Clone)]
enum Message { Undo, Dismiss(u64) }

fn view(notice_id: Option<u64>) -> Element<'static, Message> {
    let page = container("Workspace").width(Length::Fill).height(Length::Fill);
    let notice = notice_id.map(|id| snackbar("Workspace duplicated")
        .id(id)
        .action("Undo", Message::Undo)
        .on_dismiss(Message::Dismiss(id)));
    snackbar::host(page, notice)
}
```

Keep `snackbar::host` mounted. If the page also has dialogs, put it **inside**
`dialog::modal(background, dialog, open)` or `dialog::stack`. The app
owns the current `Option<Snackbar>`; clear it on the action or dismissal message.
Increment `.id(...)` for each notification, including repeated identical text.
The gallery checks that a dismissal ID matches the current notice before clearing
it. Rebuilds preserve the remaining time. Hovering, deactivating the window or
opening a root dialog pauses the timer. Use `.duration(...)` to adjust the
four-second default, or `.persistent()` to opt out. Without `on_dismiss`, the
notice persists. An action hides the notice and emits its action message once;
it does not also emit the timeout message. No application timer or animation
subscription is needed. Queueing/replacing notices belongs to the application.

## Component composition

```rust
use iced::widget::row;
use iced_m3::{Button, ButtonVariant, Element, TypeScale, badge, checkbox, chip,
    divider, icon_button, surface, switch, typography, SurfaceVariant};

#[derive(Clone)]
enum Message { Add, Toggle(bool), Filter }

fn controls(add_icon: Element<'_, Message>) -> Element<'_, Message> {
surface(iced::widget::column![
    typography("Notifications", TypeScale::Title),
    switch(true).label("Show notifications").on_toggle(Message::Toggle),
    checkbox(false).label("Email me too").on_toggle(Message::Toggle),
    divider(),
    row![chip("Important", true).on_press(Message::Filter), badge(5)].spacing(8),
    icon_button(add_icon).on_press(Message::Add),
    Button::new(row![typography("+", TypeScale::Title), typography("Add item", TypeScale::Label)].spacing(8))
        .variant(ButtonVariant::Tonal).on_press(Message::Add),
].spacing(16)).variant(SurfaceVariant::Outlined).into()
}
```

Supply a square icon widget as `add_icon`. `icon(svg_handle)` is a passive 24px monochrome SVG widget that inherits its enclosing component's foreground, including disabled alpha; `.size(18)` or `.size(36)` changes its size. The gallery's vector example (`examples/gallery/icons.rs` in the source checkout) uses this shared renderer with cached SVG handles. Custom widgets can still be supplied; they should honor the inherited foreground's alpha. In iced 0.14, SVG tint affects RGB only, so custom SVG drawing must pass foreground alpha through SVG opacity as well.

## Desktop overlays

Cascading menus and full-screen dialogs share the keyboard and focus layer:

```rust
use iced_m3::{button, dialog, focus, full_screen_dialog, menu, MenuItem, Element, typography, TypeScale};
#[derive(Clone)]
enum Message { Pdf, Save, Close }

let actions = menu("Actions", [MenuItem::submenu("Export", [
    MenuItem::new("PDF", Message::Pdf),
])]);
let editor = full_screen_dialog("Workspace details", typography("Review before saving.", TypeScale::BodyLarge))
    .on_dismiss(Message::Close)
    .action(button("Save").on_press(Message::Save));
let open = true; // Application state; set false to animate closing.
let _: Element<'_, Message> = focus::scope(dialog::modal(actions, editor.into(), open));
```

A docked calendar retains month navigation across app updates and closes after
selection is complete. Keep this widget mounted and rebuild with the new values:

```rust
use iced_m3::{Date, DateSelection, date_picker, docked_date_picker, typography, TypeScale, Element};
#[derive(Clone)]
enum Message { Month(Date), Selection(DateSelection) }

let calendar = date_picker(Date::new(2026, 9, 1).unwrap(), DateSelection::Single(None))
    .on_month(Message::Month)
    .on_select(Message::Selection);
let _: Element<'_, Message> = docked_date_picker(typography("Choose date", TypeScale::LabelLarge), calendar);
```

`modal_navigation_rail(background, rail, open, on_dismiss)` provides a leading
modal rail. Keep the host mounted through closing, and handle destination
selection and dismissal in application state. Desktop behavior and limits (`docs/DESKTOP.md` in the source checkout)
includes keyboard commands, initial focus and the gallery walkthrough.

For animated dialog dismissal, retain the dialog in `dialog::modal(background,
dialog, open)` and set `open` to false. Snackbar hosts similarly retain an outgoing
notice with `.visible(false)`; passing `None` removes it immediately. See
motion and desktop interactions (`docs/MOTION_POLISH.md` in the source checkout) for timings and scope.
