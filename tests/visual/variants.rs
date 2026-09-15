use super::{harness::Harness, reference, theme};
use crate::{Element, TextFieldVariant, Time, TimePart, TypeScale, focus, typography};
use iced::{
    Event, Size,
    keyboard::{self, key::Named},
    widget, window,
};

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Input(String),
    Hour(String),
    Minute(String),
    Period(bool),
    Time(Time),
    Toggle,
    Action,
    Select(u8),
}
fn key(name: Named, up: bool) -> Event {
    let key = keyboard::Key::Named(name);
    let physical_key =
        keyboard::key::Physical::Unidentified(keyboard::key::NativeCode::Unidentified);
    if up {
        Event::Keyboard(keyboard::Event::KeyReleased {
            key: key.clone(),
            modified_key: key,
            physical_key,
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::empty(),
        })
    } else {
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key,
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::empty(),
            text: None,
            repeat: false,
        })
    }
}
fn press(ui: &mut Harness<'_, Message>, name: Named) {
    ui.event(key(name, false));
    ui.event(key(name, true));
}
fn host(e: impl Into<Element<'static, Message>>) -> Element<'static, Message> {
    focus::scope(widget::container(e).padding(16))
}
fn ui(
    e: impl Into<Element<'static, Message>>,
    size: Size,
    dark: bool,
) -> Harness<'static, Message> {
    Harness::with_backend(
        host(e),
        size,
        theme(dark),
        &std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into()),
    )
}
fn mark() -> Element<'static, Message> {
    Element::new(crate::glyph::Glyph::Search)
}
fn field(
    variant: TextFieldVariant,
    value: &str,
    disabled: bool,
    error: bool,
) -> Element<'static, Message> {
    let field = crate::text_field("Budget", value)
        .id("amount")
        .variant(variant)
        .leading(mark())
        .prefix("$")
        .suffix("USD")
        .trailing_action(mark(), Message::Action)
        .on_input(Message::Input)
        .supporting_text("A budget for the workspace")
        .disabled(disabled)
        .width(320);
    if error {
        field.error("Enter a positive amount").into()
    } else {
        field.into()
    }
}
#[derive(Default)]
struct Inputs(Vec<String>);
impl iced::advanced::widget::Operation for Inputs {
    fn traverse(&mut self, f: &mut dyn FnMut(&mut dyn iced::advanced::widget::Operation)) {
        f(self);
    }
    fn text_input(
        &mut self,
        _: Option<&widget::Id>,
        _: iced::Rectangle,
        state: &mut dyn iced::advanced::widget::operation::TextInput,
    ) {
        self.0.push(state.text().into());
    }
}
fn select_text(ui: &mut Harness<'_, Message>) {
    struct Select;
    impl iced::advanced::widget::Operation for Select {
        fn traverse(&mut self, f: &mut dyn FnMut(&mut dyn iced::advanced::widget::Operation)) {
            f(self);
        }
        fn text_input(
            &mut self,
            _: Option<&widget::Id>,
            _: iced::Rectangle,
            state: &mut dyn iced::advanced::widget::operation::TextInput,
        ) {
            state.select_all();
        }
    }
    ui.operate(&mut Select);
}
#[test]
fn adornments_do_not_change_native_text_and_trailing_action_has_its_own_focus() {
    let mut ui = ui(
        field(TextFieldVariant::Filled, "125.00", false, false),
        Size::new(352.0, 140.0),
        false,
    );
    let mut inputs = Inputs::default();
    ui.operate(&mut inputs);
    assert_eq!(inputs.0, ["125.00"]);
    press(&mut ui, Named::Tab);
    ui.operate(
        &mut iced::advanced::widget::operation::text_input::move_cursor_to::<()>(
            widget::Id::new("amount"),
            3,
        ),
    );
    ui.rebuild(host(field(
        TextFieldVariant::Outlined,
        "125.00",
        false,
        false,
    )));
    press(&mut ui, Named::Backspace);
    assert_eq!(ui.messages, [Message::Input("12.00".into())]);
    press(&mut ui, Named::Tab);
    press(&mut ui, Named::Enter);
    assert_eq!(ui.messages.last(), Some(&Message::Action));
    ui.messages.clear();
    ui.move_to((308.0, 52.0));
    ui.down();
    ui.rebuild(host(field(
        TextFieldVariant::Outlined,
        "125.00",
        true,
        false,
    )));
    ui.up();
    press(&mut ui, Named::Tab);
    press(&mut ui, Named::Enter);
    assert!(
        ui.messages.is_empty(),
        "disabling a held trailing action must cancel it"
    );
}
fn select(variant: TextFieldVariant, disabled: bool) -> Element<'static, Message> {
    crate::select(
        "Role",
        [
            crate::SelectOption::new(0, "Viewer"),
            crate::SelectOption::new(1, "Editor"),
            crate::SelectOption::new(2, "Managed").disabled(true),
        ],
        Some(0),
    )
    .on_select(Message::Select)
    .variant(variant)
    .leading(mark())
    .supporting_text("Choose workspace access")
    .disabled(disabled)
    .into()
}
#[test]
fn dropdown_fields_are_selection_only_and_keep_popup_keyboard_navigation() {
    for variant in [TextFieldVariant::Outlined, TextFieldVariant::Filled] {
        let mut ui = ui(select(variant, false), Size::new(360.0, 380.0), false);
        let mut inputs = Inputs::default();
        ui.operate(&mut inputs);
        assert!(
            inputs.0.is_empty(),
            "a select must not expose a native editor or IME target"
        );
        press(&mut ui, Named::Tab);
        press(&mut ui, Named::ArrowDown);
        ui.at(600);
        press(&mut ui, Named::ArrowDown);
        press(&mut ui, Named::Enter);
        assert_eq!(ui.messages, [Message::Select(1)]);
        ui.rebuild(host(select(variant, true)));
        ui.messages.clear();
        ui.move_to((80.0, 52.0));
        ui.down();
        ui.up();
        ui.at(1200);
        press(&mut ui, Named::Enter);
        assert!(ui.messages.is_empty());
    }
}
fn time(
    input: bool,
    format24: bool,
    hour: &str,
    minute: &str,
    pm: bool,
) -> crate::TimePicker<'static, Message> {
    crate::time_picker(Time::new(10, 30).unwrap(), TimePart::Hour)
        .format24(format24)
        .input_mode(input)
        .on_toggle_input(Message::Toggle)
        .on_change(Message::Time)
        .hour_input(hour, Message::Hour)
        .minute_input(minute, Message::Minute)
        .input_period(pm, Message::Period)
}
#[test]
fn manual_time_validates_partial_values_and_converts_noon_and_midnight() {
    for (h, m, format24) in [
        ("", "30", true),
        ("24", "30", true),
        ("0", "30", false),
        ("13", "30", false),
        ("12", "60", false),
        ("١٢", "30", false),
        ("+1", "30", true),
    ] {
        assert!(time(true, format24, h, m, false).input_time().is_err());
    }
    for (h, pm, expected) in [
        ("12", false, 0),
        ("12", true, 12),
        ("1", true, 13),
        ("1", false, 1),
    ] {
        assert_eq!(
            time(true, false, h, "5", pm).input_time(),
            Time::new(expected, 5)
        );
    }
    assert_eq!(
        time(true, true, "23", "59", false).input_time(),
        Time::new(23, 59)
    );
}
#[test]
fn manual_time_commits_only_on_valid_apply_or_enter_and_period_stays_a_draft() {
    let mut ui = ui(
        time(true, false, "12", "05", true),
        Size::new(400.0, 320.0),
        false,
    );
    ui.click("AM");
    assert_eq!(ui.messages, [Message::Period(false)]);
    ui.messages.clear();
    ui.click("Apply");
    assert_eq!(ui.messages, [Message::Time(Time::new(12, 5).unwrap())]);
    ui.rebuild(host(time(true, true, "", "05", false)));
    ui.messages.clear();
    ui.click("Apply");
    press(&mut ui, Named::Tab);
    press(&mut ui, Named::Enter);
    assert!(ui.messages.is_empty());
    ui.rebuild(host(time(true, true, "23", "05", false)));
    ui.move_to((84.0, 100.0));
    ui.down();
    ui.up();
    press(&mut ui, Named::Enter);
    assert_eq!(ui.messages, [Message::Time(Time::new(23, 5).unwrap())]);
    ui.rebuild(host(time(false, true, "", "05", false)));
    ui.messages.clear();
    ui.at(600);
    assert!(
        ui.messages.is_empty(),
        "switching to the dial must not commit a draft"
    );
}
fn card(disabled: bool, dragged: bool) -> Element<'static, Message> {
    crate::card(
        widget::column![
            typography("Collection", TypeScale::TitleMedium),
            crate::button("Child action").on_press(Message::Action)
        ]
        .spacing(16),
    )
    .on_press(Message::Toggle)
    .disabled(disabled)
    .dragged(dragged)
    .width(280)
    .into()
}
#[test]
fn disabled_and_dragged_cards_cancel_actions_and_remove_child_focus_targets() {
    for (disabled, dragged) in [(true, false), (false, true), (true, true)] {
        let mut ui = ui(card(false, false), Size::new(330.0, 200.0), false);
        let point = ui.find("Child action").center();
        ui.move_to(point);
        ui.down();
        ui.rebuild(host(card(disabled, dragged)));
        ui.up();
        press(&mut ui, Named::Tab);
        press(&mut ui, Named::Enter);
        ui.click("Collection");
        assert!(ui.messages.is_empty());
        ui.rebuild(host(card(false, false)));
        ui.click("Child action");
        assert_eq!(ui.messages, [Message::Action]);
    }
}
const COLORS: [iced::Color; 4] = [
    iced::Color::from_rgb(0.1, 0.3, 0.9),
    iced::Color::from_rgb(0.9, 0.2, 0.1),
    iced::Color::from_rgb(0.9, 0.7, 0.1),
    iced::Color::from_rgb(0.1, 0.7, 0.3),
];
fn progress(circular: bool, paused: bool) -> Element<'static, Message> {
    (if circular {
        crate::circular_progress(0.0)
    } else {
        crate::linear_progress(0.0)
    })
    .indeterminate(true)
    .colors(COLORS)
    .paused(paused)
    .into()
}
#[test]
fn multicolor_progress_freezes_both_color_and_geometry_on_pause_and_reduced_motion() {
    for circular in [false, true] {
        let mut ui = ui(progress(circular, false), Size::new(360.0, 100.0), false);
        ui.at(0);
        ui.frame();
        ui.at(1900);
        let before = ui.frame();
        ui.rebuild(host(progress(circular, true)));
        ui.at(1900);
        assert_eq!(ui.at(8000), window::RedrawRequest::Wait);
        assert!(before == ui.frame());
        let mut theme = theme(false);
        theme.motion = crate::tokens::Motion::reduced();
        let mut ui = Harness::new(
            host(progress(circular, false)),
            Size::new(360.0, 100.0),
            theme,
        );
        ui.at(0);
        let before = ui.frame();
        ui.at(0);
        assert_eq!(ui.at(8000), window::RedrawRequest::Wait);
        assert!(before == ui.frame());
    }
}
#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_desktop_variants() {
    for dark in [false, true] {
        let mode = if dark { "dark" } else { "light" };
        for variant in [TextFieldVariant::Outlined, TextFieldVariant::Filled] {
            let v = if variant == TextFieldVariant::Outlined {
                "outlined"
            } else {
                "filled"
            };
            for (name, value, disabled, error, width) in [
                ("default", "125.00", false, false, 352.0),
                ("empty", "", false, false, 352.0),
                ("disabled", "125.00", true, false, 352.0),
                ("error", "-125.00", false, true, 352.0),
                ("narrow", "125.00", false, false, 200.0),
            ] {
                let mut ui = ui(
                    field(variant, value, disabled, error),
                    Size::new(width, 160.0),
                    dark,
                );
                ui.at(0);
                reference::check(&format!("variants-field-{v}-{mode}/{name}"), &ui.frame());
            }
            let mut ui = ui(
                field(variant, "", false, false),
                Size::new(352.0, 160.0),
                dark,
            );
            ui.at(0);
            ui.frame();
            press(&mut ui, Named::Tab);
            // Empty editor caret would blink: populate and select it before capture.
            ui.rebuild(host(field(variant, "125.00", false, false)));
            select_text(&mut ui);
            for ms in [0, 75, 150, 300] {
                ui.at(ms);
                reference::check(
                    &format!("variants-field-{v}-{mode}/focus-{ms:03}"),
                    &ui.frame(),
                );
            }
            ui.move_to((308.0, 52.0));
            ui.down();
            for ms in [300, 375, 450, 650] {
                ui.at(ms);
                reference::check(
                    &format!("variants-field-{v}-{mode}/action-held-{ms:03}"),
                    &ui.frame(),
                );
            }
            ui.up();
            ui.at(950);
            reference::check(
                &format!("variants-field-{v}-{mode}/action-released"),
                &ui.frame(),
            );
            let mut ui = self::ui(select(variant, false), Size::new(360.0, 380.0), dark);
            ui.at(0);
            reference::check(&format!("variants-select-{v}-{mode}/default"), &ui.frame());
            press(&mut ui, Named::Tab);
            ui.at(300);
            reference::check(&format!("variants-select-{v}-{mode}/focused"), &ui.frame());
            press(&mut ui, Named::ArrowDown);
            ui.at(600);
            reference::check(&format!("variants-select-{v}-{mode}/open"), &ui.frame());
            let mut ui = self::ui(select(variant, true), Size::new(360.0, 180.0), dark);
            ui.at(0);
            reference::check(&format!("variants-select-{v}-{mode}/disabled"), &ui.frame());
            let mut ui = self::ui(
                crate::select("Role", [crate::SelectOption::new(0, "Viewer")], None)
                    .on_select(Message::Select)
                    .variant(variant)
                    .error("Choose a role"),
                Size::new(360.0, 180.0),
                dark,
            );
            ui.at(0);
            reference::check(
                &format!("variants-select-{v}-{mode}/empty-error"),
                &ui.frame(),
            );
        }
        for format24 in [false, true] {
            for (name, h, m) in [("valid", "12", "05"), ("invalid", "", "75")] {
                let mut ui = ui(
                    time(true, format24, h, m, true),
                    Size::new(400.0, 340.0),
                    dark,
                );
                ui.at(0);
                reference::check(
                    &format!(
                        "variants-time-{mode}/{name}-{}",
                        if format24 { "24h" } else { "12h" }
                    ),
                    &ui.frame(),
                );
                press(&mut ui, Named::Tab);
                select_text(&mut ui);
                ui.at(300);
                // Do not capture an empty field's blinking caret in invalid cases.
                if !h.is_empty() {
                    reference::check(
                        &format!(
                            "variants-time-{mode}/{name}-focused-{}",
                            if format24 { "24h" } else { "12h" }
                        ),
                        &ui.frame(),
                    );
                }
            }
        }
        for variant in [
            crate::SurfaceVariant::Filled,
            crate::SurfaceVariant::Outlined,
            crate::SurfaceVariant::Elevated,
        ] {
            for (name, disabled, dragged) in [
                ("rest", false, false),
                ("disabled", true, false),
                ("dragged", false, true),
            ] {
                let card = crate::card(
                    widget::column![
                        typography("Collection", TypeScale::TitleMedium),
                        typography("Workspace resources", TypeScale::BodyMedium)
                    ]
                    .spacing(8),
                )
                .variant(variant)
                .on_press(Message::Action)
                .disabled(disabled)
                .dragged(dragged)
                .width(280);
                let mut ui = ui(card, Size::new(330.0, 180.0), dark);
                ui.at(0);
                reference::check(
                    &format!("variants-card-{mode}/{variant:?}-{name}"),
                    &ui.frame(),
                );
            }
        }
        for lowered in [false, true] {
            let mut ui = ui(
                crate::fab(mark())
                    .lowered(lowered)
                    .on_press(Message::Action),
                Size::new(130.0, 120.0),
                dark,
            );
            ui.at(0);
            reference::check(&format!("variants-fab-{mode}/{lowered}-rest"), &ui.frame());
            ui.move_to((44.0, 44.0));
            ui.at(200);
            reference::check(&format!("variants-fab-{mode}/{lowered}-hover"), &ui.frame());
            ui.down();
            ui.at(350);
            reference::check(&format!("variants-fab-{mode}/{lowered}-held"), &ui.frame());
        }
        for lowered in [false, true] {
            let mut ui = ui(
                crate::fab(mark())
                    .color(crate::FabColor::Surface)
                    .lowered(lowered)
                    .on_press(Message::Action),
                Size::new(130.0, 120.0),
                dark,
            );
            ui.at(0);
            reference::check(
                &format!("variants-fab-{mode}/surface-{lowered}-rest"),
                &ui.frame(),
            );
            ui.move_to((44.0, 44.0));
            ui.at(200);
            reference::check(
                &format!("variants-fab-{mode}/surface-{lowered}-hover"),
                &ui.frame(),
            );
        }
        let mut ui = ui(
            crate::linear_progress(0.25).buffer(0.6),
            Size::new(360.0, 80.0),
            dark,
        );
        ui.at(0);
        reference::check(&format!("variants-buffer-{mode}/rest"), &ui.frame());
        ui.rebuild(host(crate::linear_progress(0.7).buffer(0.9)));
        for ms in [0, 75, 150, 300, 500] {
            ui.at(ms);
            reference::check(
                &format!("variants-buffer-{mode}/transition-{ms:03}"),
                &ui.frame(),
            );
        }
        for circular in [false, true] {
            let mut ui = self::ui(progress(circular, false), Size::new(360.0, 100.0), dark);
            ui.at(0);
            ui.frame();
            for ms in [
                0, 300, 700, 1000, 1250, 1333, 1750, 1900, 2000, 2400, 3800, 5800, 7800,
            ] {
                ui.at(ms);
                reference::check(
                    &format!("variants-color-{circular}-{mode}/{ms:04}"),
                    &ui.frame(),
                );
            }
        }
    }
}

#[test]
fn buffer_changes_animate_independently_and_ignored_buffers_do_not_schedule_work() {
    let mut ui = ui(
        crate::linear_progress(0.25).buffer(0.4),
        Size::new(360.0, 80.0),
        false,
    );
    ui.at(0);
    let before = ui.frame();
    ui.rebuild(host(crate::linear_progress(0.25).buffer(0.9)));
    ui.at(0);
    ui.at(100);
    let middle = ui.frame();
    ui.at(500);
    let after = ui.frame();
    assert!(before != middle && middle != after);
    assert_eq!(ui.at(1000), window::RedrawRequest::Wait);
    ui.rebuild(host(crate::circular_progress(0.25).buffer(0.4)));
    ui.at(1100);
    ui.frame();
    ui.at(1600);
    let before = ui.frame();
    ui.rebuild(host(crate::circular_progress(0.25).buffer(0.9)));
    assert_eq!(ui.at(1700), window::RedrawRequest::Wait);
    assert!(before == ui.frame());
}
