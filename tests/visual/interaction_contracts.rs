//! Real iced event dispatch: keyboard editing, focus and builder contracts.
use super::harness::Harness;
use crate::{Element, Theme, button, focus, text_field, toolbar};
use iced::{
    Event, Length, Size,
    keyboard::{self, key::Named},
    mouse, widget,
};

#[derive(Clone, Debug, PartialEq)]
enum Message {
    Action,
    Outside,
    Input(String),
    Value(f32),
}

fn ui(content: impl Into<Element<'static, Message>>) -> Harness<'static, Message> {
    Harness::new(focus::scope(content), Size::new(640., 240.), Theme::light())
}
fn key(key: keyboard::Key, released: bool) -> Event {
    let physical_key =
        keyboard::key::Physical::Unidentified(keyboard::key::NativeCode::Unidentified);
    let modifiers = keyboard::Modifiers::empty();
    let location = keyboard::Location::Standard;
    if released {
        Event::Keyboard(keyboard::Event::KeyReleased {
            modified_key: key.clone(),
            key,
            physical_key,
            modifiers,
            location,
        })
    } else {
        let text = match &key {
            keyboard::Key::Character(s) => Some(s.clone()),
            _ => None,
        };
        Event::Keyboard(keyboard::Event::KeyPressed {
            modified_key: key.clone(),
            key,
            physical_key,
            modifiers,
            location,
            text,
            repeat: false,
        })
    }
}
fn press(ui: &mut Harness<'_, Message>, named: Named) {
    ui.event(key(keyboard::Key::Named(named), false));
    ui.event(key(keyboard::Key::Named(named), true));
}
fn type_x(ui: &mut Harness<'_, Message>) {
    ui.event(key(keyboard::Key::Character("x".into()), false));
}

#[test]
fn toolbar_preserves_slider_keyboard_editing() {
    for named in [Named::ArrowRight, Named::ArrowLeft, Named::Home, Named::End] {
        let mut view = ui(toolbar([
            crate::slider(0.0..=100.0, 50.0)
                .width(180)
                .on_change(Message::Value)
                .into(),
            button("Next").on_press(Message::Outside).into(),
        ]));
        press(&mut view, Named::Tab);
        press(&mut view, named);
        let expected = match named {
            Named::ArrowRight => 51.,
            Named::ArrowLeft => 49.,
            Named::Home => 0.,
            _ => 100.,
        };
        assert_eq!(view.messages, [Message::Value(expected)], "{named:?}");
    }
}

#[test]
fn toolbar_preserves_text_caret_navigation() {
    for (named, expected) in [
        (Named::ArrowLeft, "abxc"),
        (Named::Home, "xabc"),
        (Named::End, "abcx"),
    ] {
        let mut view = ui(toolbar([
            text_field("Editor", "abc")
                .width(200)
                .on_input(Message::Input)
                .into(),
            button("Next").on_press(Message::Outside).into(),
        ]));
        press(&mut view, Named::Tab);
        press(&mut view, Named::End);
        press(&mut view, named);
        type_x(&mut view);
        assert_eq!(
            view.messages,
            [Message::Input(expected.into())],
            "{named:?}"
        );
    }
}

#[test]
fn toolbar_buttons_still_roam_and_tab_can_leave_editors() {
    let mut view = ui(widget::column![
        toolbar([
            button("One").on_press(Message::Action).into(),
            button("Two").on_press(Message::Outside).into()
        ]),
        text_field("After", "").on_input(Message::Input),
    ]);
    press(&mut view, Named::Tab);
    press(&mut view, Named::ArrowRight);
    press(&mut view, Named::Enter);
    press(&mut view, Named::Home);
    press(&mut view, Named::Enter);
    press(&mut view, Named::Tab);
    type_x(&mut view);
    assert_eq!(
        view.messages,
        [
            Message::Outside,
            Message::Action,
            Message::Input("x".into())
        ]
    );

    let mut view = ui(widget::column![
        toolbar([text_field("Editor", "abc").on_input(Message::Input).into()]),
        button("After").on_press(Message::Outside),
    ]);
    press(&mut view, Named::Tab);
    press(&mut view, Named::Tab);
    press(&mut view, Named::Enter);
    assert_eq!(view.messages, [Message::Outside]);
}

#[test]
fn context_click_preserves_editing_but_primary_click_outside_blurs() {
    let mut view = ui(text_field("Editor", "abc")
        .width(200)
        .on_input(Message::Input));
    press(&mut view, Named::Tab);
    press(&mut view, Named::End);
    press(&mut view, Named::ArrowLeft);
    view.move_to((80., 28.));
    view.event(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Right,
    )));
    view.event(Event::Mouse(mouse::Event::ButtonReleased(
        mouse::Button::Right,
    )));
    type_x(&mut view);
    assert_eq!(view.messages, [Message::Input("abxc".into())]);
    view.messages.clear();
    view.move_to((500., 200.));
    view.down();
    view.up();
    type_x(&mut view);
    assert!(
        view.messages.is_empty(),
        "An outside primary click must still blur"
    );
}

#[derive(Clone, Copy, Debug)]
enum Configuration {
    Enabled,
    DisabledFirst,
    DisabledLast,
    Restored,
    NoHandler,
}
fn control(kind: &str, mode: Configuration) -> Element<'static, Message> {
    macro_rules! configure {
        ($control:expr, $method:ident, $handler:expr) => {
            match mode {
                Configuration::Enabled => $control.$method($handler),
                Configuration::DisabledFirst => $control.disabled(true).$method($handler),
                Configuration::DisabledLast => $control.$method($handler).disabled(true),
                Configuration::Restored => {
                    $control.$method($handler).disabled(true).disabled(false)
                }
                Configuration::NoHandler => $control.disabled(false),
            }
            .into()
        };
    }
    match kind {
        "button" => configure!(button("Target").width(160), on_press, Message::Action),
        "chip" => configure!(
            crate::assist_chip("Target").width(160),
            on_press,
            Message::Action
        ),
        "checkbox" => configure!(crate::checkbox(false).width(160), on_toggle, |_| {
            Message::Action
        }),
        "radio" => configure!(
            crate::radio("Target", 1, Some(0)).width(160),
            on_select,
            |_| Message::Action
        ),
        "switch" => configure!(crate::switch(false).width(160), on_toggle, |_| {
            Message::Action
        }),
        "slider" => configure!(
            crate::slider(0.0..=100.0, 20.0).width(160),
            on_change,
            |_| Message::Action
        ),
        "range" => configure!(
            crate::range_slider(0.0..=100.0, (20., 80.)).width(160),
            on_change,
            |_| Message::Action
        ),
        "field" => configure!(text_field("Target", "abc").width(160), on_input, |_| {
            Message::Action
        }),
        "select" => configure!(
            crate::select("Target", [crate::SelectOption::new(1, "Choice")], None).width(160.),
            on_select,
            |_| Message::Action
        ),
        "fab" => configure!(
            crate::extended_fab(widget::space().width(24).height(24), "Target"),
            on_press,
            Message::Action
        ),
        "card" => configure!(
            crate::card(widget::space().width(160).height(56)),
            on_press,
            Message::Action
        ),
        _ => unreachable!(),
    }
}

#[test]
fn disabled_is_independent_of_handler_order_for_pointer_and_keyboard() {
    let mut failures = Vec::new();
    for kind in [
        "button", "chip", "checkbox", "radio", "switch", "slider", "range", "field", "select",
        "fab", "card",
    ] {
        for mode in [
            Configuration::Enabled,
            Configuration::DisabledFirst,
            Configuration::DisabledLast,
            Configuration::Restored,
            Configuration::NoHandler,
        ] {
            for pointer in [false, true] {
                let mut view = ui(widget::column![
                    control(kind, mode),
                    button("Outside").on_press(Message::Outside)
                ]
                .spacing(20)
                .width(Length::Fill));
                if pointer {
                    view.move_to((30., 28.));
                    view.down();
                    view.up();
                } else {
                    press(&mut view, Named::Tab);
                }
                match kind {
                    "field" => type_x(&mut view),
                    "slider" | "range" => press(&mut view, Named::ArrowRight),
                    "select" => {
                        if !pointer {
                            press(&mut view, Named::Enter);
                        }
                        press(&mut view, Named::ArrowDown);
                        press(&mut view, Named::Enter);
                    }
                    _ => {
                        if !pointer {
                            press(&mut view, Named::Space);
                        }
                    }
                }
                let expected = matches!(mode, Configuration::Enabled | Configuration::Restored);
                if view.messages.contains(&Message::Action) != expected {
                    failures.push(format!(
                        "{kind} {mode:?} pointer={pointer}: {:?}",
                        view.messages
                    ));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn disabling_during_a_press_cancels_it_even_after_reenabling() {
    for kind in [
        "button", "checkbox", "radio", "switch", "slider", "range", "fab", "chip", "card",
    ] {
        let mut view = ui(control(kind, Configuration::Enabled));
        view.move_to((30., 28.));
        view.down();
        view.messages.clear(); // Sliders legitimately publish on pointer-down.
        view.rebuild(focus::scope(control(kind, Configuration::DisabledFirst)));
        view.up();
        assert!(view.messages.is_empty(), "Disabled release: {kind}");
        view.rebuild(focus::scope(control(kind, Configuration::Enabled)));
        view.up();
        assert!(
            view.messages.is_empty(),
            "A cancelled press cannot replay: {kind}"
        );
        press(&mut view, Named::Tab);
        press(
            &mut view,
            if matches!(kind, "slider" | "range") {
                Named::ArrowRight
            } else {
                Named::Space
            },
        );
        assert!(
            view.messages.contains(&Message::Action),
            "New interaction after re-enabling: {kind}"
        );
    }
}

#[test]
fn disabling_a_field_preserves_submit_and_trailing_action_configuration() {
    for disabled in [true, false] {
        let mut view = ui(text_field("Editor", "abc")
            .width(240)
            .on_input(Message::Input)
            .on_submit(Message::Action)
            .trailing_action(widget::text("Clear"), Message::Outside)
            .disabled(true)
            .disabled(disabled));
        press(&mut view, Named::Tab);
        press(&mut view, Named::Enter);
        // Disabled trailing actions are deliberately absent from focus operations.
        view.move_to((212., 36.));
        view.down();
        view.up();
        assert_eq!(
            view.messages,
            if disabled {
                vec![]
            } else {
                vec![Message::Action, Message::Outside]
            }
        );
    }
}

#[test]
fn unfocused_indicators_animate_only_inside_the_widget_viewport() {
    for kind in ["linear", "circular", "loading"] {
        let indicator = || -> Element<'static, Message> {
            match kind {
                "linear" => crate::linear_progress(0.)
                    .indeterminate(true)
                    .width(160)
                    .into(),
                "circular" => crate::circular_progress(0.).indeterminate(true).into(),
                _ => crate::loading_indicator().into(),
            }
        };
        let mut view = ui(indicator());
        view.frame();
        view.at(0);
        view.at(137);
        let before = view.frame();
        view.event(Event::Window(iced::window::Event::Unfocused));
        assert_eq!(
            view.at(374),
            iced::window::RedrawRequest::NextFrame,
            "{kind}"
        );
        assert!(before != view.frame(), "Visible and unfocused: {kind}");
        let mut hidden =
            ui(
                widget::scrollable(widget::column![widget::space().height(400), indicator()])
                    .height(100),
            );
        hidden.frame();
        hidden.at(0);
        hidden.event(Event::Window(iced::window::Event::Unfocused));
        assert_eq!(
            hidden.at(374),
            iced::window::RedrawRequest::Wait,
            "Clipped: {kind}"
        );
    }
}
