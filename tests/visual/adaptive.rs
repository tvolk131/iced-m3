use super::{harness::Harness, reference, theme, themed_name};
use crate::{
    AppBarVariant, Element, NavigationItem, NavigationLayout, TypeScale, app_bar, button, checkbox,
    focus, navigation_bar, navigation_rail, typography,
};
use iced::{
    Event, Length, Size,
    keyboard::{self, key::Named},
    widget,
};
#[derive(Clone, Debug, PartialEq)]
enum Message {
    Action(u8),
    Check(bool),
    Value(f32),
    Range((f32, f32)),
    Close,
}
fn make_ui(
    element: impl Into<Element<'static, Message>>,
    size: Size,
    dark: bool,
) -> Harness<'static, Message> {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    Harness::with_backend(element, size, theme(dark), &backend)
}
fn key(key: Named, release: bool, shift: bool) -> Event {
    let modifiers = if shift {
        keyboard::Modifiers::SHIFT
    } else {
        keyboard::Modifiers::empty()
    };
    let physical_key =
        keyboard::key::Physical::Unidentified(keyboard::key::NativeCode::Unidentified);
    if release {
        Event::Keyboard(keyboard::Event::KeyReleased {
            key: keyboard::Key::Named(key),
            modified_key: keyboard::Key::Named(key),
            physical_key,
            location: keyboard::Location::Standard,
            modifiers,
        })
    } else {
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(key),
            modified_key: keyboard::Key::Named(key),
            physical_key,
            location: keyboard::Location::Standard,
            modifiers,
            text: None,
            repeat: false,
        })
    }
}
fn press(ui: &mut Harness<'_, Message>, k: Named) {
    ui.event(key(k, false, false));
}
fn activate(ui: &mut Harness<'_, Message>) {
    press(ui, Named::Space);
    ui.event(key(Named::Space, true, false));
}
fn controls() -> Element<'static, Message> {
    focus::scope(
        widget::column![
            button("One").on_press(Message::Action(1)),
            button("Disabled"),
            checkbox(false).label("Check").on_toggle(Message::Check),
            button("Two").on_press(Message::Action(2))
        ]
        .spacing(12),
    )
}
#[test]
fn tab_skips_disabled_wraps_and_shift_reverses() {
    let mut ui = make_ui(controls(), Size::new(400.0, 320.0), false);
    press(&mut ui, Named::Tab);
    activate(&mut ui);
    press(&mut ui, Named::Tab);
    activate(&mut ui);
    press(&mut ui, Named::Tab);
    activate(&mut ui);
    press(&mut ui, Named::Tab);
    activate(&mut ui);
    ui.event(key(Named::Tab, false, true));
    activate(&mut ui);
    assert_eq!(
        ui.messages,
        [
            Message::Action(1),
            Message::Check(true),
            Message::Action(2),
            Message::Action(1),
            Message::Action(2)
        ]
    );
}
#[test]
fn tab_during_held_space_cancels_activation() {
    let mut ui = make_ui(controls(), Size::new(400.0, 320.0), false);
    press(&mut ui, Named::Tab);
    press(&mut ui, Named::Space);
    press(&mut ui, Named::Tab);
    ui.event(key(Named::Space, true, false));
    assert!(ui.messages.is_empty());
    activate(&mut ui);
    assert_eq!(ui.messages, [Message::Check(true)]);
}
fn items() -> [NavigationItem<'static, u8, Message>; 3] {
    [
        NavigationItem::new(1, "Home", typography("H", TypeScale::TitleLarge)),
        NavigationItem::new(2, "Library", typography("L", TypeScale::TitleLarge)).badge(3),
        NavigationItem::new(3, "Settings", typography("S", TypeScale::TitleLarge)).disabled(true),
    ]
}
#[test]
fn navigation_arrows_skip_disabled_and_do_not_change_selection_until_activated() {
    let mut ui = make_ui(
        focus::scope(navigation_bar(items(), Some(1)).on_select(Message::Action)),
        Size::new(390.0, 80.0),
        false,
    );
    press(&mut ui, Named::Tab);
    press(&mut ui, Named::ArrowRight);
    assert!(ui.messages.is_empty());
    activate(&mut ui);
    press(&mut ui, Named::ArrowRight);
    activate(&mut ui);
    assert_eq!(ui.messages, [Message::Action(2), Message::Action(1)]);
}
#[test]
fn modal_tab_is_trapped_and_escape_restores_invoker() {
    fn fixture(open: bool) -> Element<'static, Message> {
        focus::scope(crate::dialog::modal(
            button("Outside").on_press(Message::Action(1)),
            open.then(|| {
                crate::dialog(widget::row![
                    button("Cancel").on_press(Message::Close),
                    button("Save").on_press(Message::Action(2))
                ])
                .on_dismiss(Message::Close)
            }),
        ))
    }
    let mut ui = make_ui(fixture(false), Size::new(500.0, 300.0), false);
    press(&mut ui, Named::Tab);
    ui.rebuild(fixture(true));
    ui.at(0);
    for _ in 0..3 {
        press(&mut ui, Named::Tab);
    }
    activate(&mut ui);
    assert_eq!(ui.messages, [Message::Close]);
    ui.messages.clear();
    ui.rebuild(fixture(false));
    activate(&mut ui);
    assert_eq!(ui.messages, [Message::Action(1)]);
}
#[test]
fn keyboard_slider_endpoints_and_range_handles_are_independent() {
    let mut ui = make_ui(
        focus::scope(widget::column![
            crate::slider(0.0..=100.0, 50.0)
                .step(10.0)
                .on_change(Message::Value),
            crate::range_slider(0.0..=100.0, (20.0, 80.0))
                .step(10.0)
                .on_change(Message::Range)
        ]),
        Size::new(320.0, 120.0),
        false,
    );
    press(&mut ui, Named::Tab);
    press(&mut ui, Named::ArrowRight);
    press(&mut ui, Named::Home);
    press(&mut ui, Named::End);
    press(&mut ui, Named::Tab);
    press(&mut ui, Named::End);
    press(&mut ui, Named::Tab);
    press(&mut ui, Named::Home);
    assert_eq!(
        ui.messages,
        [
            Message::Value(60.0),
            Message::Value(0.0),
            Message::Value(100.0),
            Message::Range((80.0, 80.0)),
            Message::Range((20.0, 20.0))
        ]
    );
}
#[test]
fn navigation_breakpoints_are_logical_widths() {
    assert_eq!(NavigationLayout::for_width(599.0), NavigationLayout::Bar);
    assert_eq!(NavigationLayout::for_width(600.0), NavigationLayout::Rail);
    assert_eq!(
        NavigationLayout::for_width(1200.0),
        NavigationLayout::ExpandedRail
    );
}
#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_adaptive_navigation_and_focus() {
    for dark in [false, true] {
        let variants: [(&str, Size, Element<'static, Message>); 3] = [
            (
                "bar",
                Size::new(390.0, 80.0),
                navigation_bar(items(), Some(1))
                    .on_select(Message::Action)
                    .into(),
            ),
            (
                "rail",
                Size::new(80.0, 320.0),
                navigation_rail(items(), Some(1))
                    .on_select(Message::Action)
                    .into(),
            ),
            (
                "expanded",
                Size::new(280.0, 320.0),
                navigation_rail(items(), Some(1))
                    .expanded(true)
                    .on_select(Message::Action)
                    .into(),
            ),
        ];
        for (label, size, element) in variants {
            let case = themed_name(&format!("adaptive-{label}"), dark);
            let mut ui = make_ui(focus::scope(element), size, dark);
            ui.at(0);
            reference::check(&format!("{case}/00-default"), &ui.frame());
            press(&mut ui, Named::Tab);
            reference::check(&format!("{case}/01-focus"), &ui.frame());
            press(&mut ui, Named::Space);
            for t in [0, 75, 150] {
                ui.at(t);
                reference::check(&format!("{case}/02-held-{t:03}ms"), &ui.frame());
            }
        }
        let case = themed_name("adaptive-app-bars", dark);
        let mut column = widget::Column::new().spacing(8);
        for variant in [
            AppBarVariant::Small,
            AppBarVariant::CenterAligned,
            AppBarVariant::Medium,
            AppBarVariant::Large,
        ] {
            for collapse in [0.0, 0.5, 1.0] {
                column = column.push(
                    app_bar("Workspace")
                        .variant(variant)
                        .collapse(collapse)
                        .leading(button("‹").on_press(Message::Close))
                        .action(button("+").on_press(Message::Action(1))),
                );
            }
        }
        let mut ui = make_ui(
            widget::container(column).width(Length::Fill),
            Size::new(480.0, 1200.0),
            dark,
        );
        ui.at(0);
        reference::check(&format!("{case}/variants"), &ui.frame());
        let mut ui = make_ui(controls(), Size::new(400.0, 320.0), dark);
        ui.at(0);
        press(&mut ui, Named::Tab);
        press(&mut ui, Named::Tab);
        press(&mut ui, Named::Space);
        for t in [0, 75, 150, 225] {
            ui.at(t);
            reference::check(
                &format!(
                    "{}/held-{t:03}ms",
                    themed_name("adaptive-checkbox-focus", dark)
                ),
                &ui.frame(),
            );
        }
    }
}
