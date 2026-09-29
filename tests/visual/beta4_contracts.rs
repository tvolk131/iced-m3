//! User-interaction contracts for the next desktop beta.
use super::harness::Harness;
use crate::{Element, Theme, Time, TimePart, button, focus};
use iced::{
    Event, Size,
    keyboard::{self, key::Named},
    widget, window,
};
use std::{cell::RefCell, rc::Rc, time::Duration};

#[derive(Clone, Debug, PartialEq)]
enum Message {
    Before,
    Trigger,
    First,
    Second,
    After,
    Selected(u8),
    Change(Time),
    Confirm(Time),
    Part(TimePart),
    Period(bool),
    Input(String),
}
fn key(h: &mut Harness<'_, Message>, name: Named, shift: bool) {
    let modifiers = if shift {
        keyboard::Modifiers::SHIFT
    } else {
        keyboard::Modifiers::empty()
    };
    let physical_key =
        keyboard::key::Physical::Unidentified(keyboard::key::NativeCode::Unidentified);
    h.event(Event::Keyboard(keyboard::Event::KeyPressed {
        key: keyboard::Key::Named(name),
        modified_key: keyboard::Key::Named(name),
        physical_key,
        location: keyboard::Location::Standard,
        modifiers,
        text: None,
        repeat: false,
    }));
    h.event(Event::Keyboard(keyboard::Event::KeyReleased {
        key: keyboard::Key::Named(name),
        modified_key: keyboard::Key::Named(name),
        physical_key,
        location: keyboard::Location::Standard,
        modifiers,
    }));
}
fn ui(e: impl Into<Element<'static, Message>>) -> Harness<'static, Message> {
    Harness::configured(
        focus::scope(widget::container(e).padding(24)),
        Size::new(700., 650.),
        Theme::light().reduced_motion(true),
    )
}
fn visible(h: &mut Harness<'_, Message>, label: &str) -> bool {
    use iced::advanced::widget::{Operation, operation::Outcome};
    let mut op = iced_test::Selector::find(label);
    h.operate(&mut iced::advanced::widget::operation::black_box(&mut op));
    matches!(op.finish(), Outcome::Some(Some(_)))
}
fn plain() -> Element<'static, Message> {
    widget::column![
        crate::tooltip(
            button("Trigger").on_press(Message::Trigger),
            "Keyboard hint"
        )
        .delay(Duration::from_millis(500)),
        button("After").on_press(Message::After)
    ]
    .spacing(16)
    .into()
}
#[test]
fn plain_tooltip_keyboard_focus_delay_escape_and_exit() {
    let mut h = ui(plain());
    h.frame();
    key(&mut h, Named::Tab, false);
    h.at(0);
    h.at(499);
    assert!(!visible(&mut h, "Keyboard hint"));
    h.at(500);
    assert!(
        visible(&mut h, "Keyboard hint"),
        "keyboard focus must reveal the hint"
    );
    h.leave();
    h.at(600); // Pointer leaving must not hide a keyboard hint.
    assert!(visible(&mut h, "Keyboard hint"));
    key(&mut h, Named::Escape, false);
    h.at(1500);
    assert!(!visible(&mut h, "Keyboard hint"));
    key(&mut h, Named::Tab, false);
    h.at(1501);
    key(&mut h, Named::Tab, true);
    h.at(1502);
    h.at(2002);
    assert!(visible(&mut h, "Keyboard hint"));
    key(&mut h, Named::Tab, false);
    h.at(2003);
    assert!(!visible(&mut h, "Keyboard hint"));
    key(&mut h, Named::Enter, false);
    assert_eq!(h.messages, [Message::After]);
}
fn rich(actions: bool) -> Element<'static, Message> {
    let body: Element<'static, Message> = if actions {
        widget::column![
            button("First").on_press(Message::First),
            button("Second").on_press(Message::Second)
        ]
        .into()
    } else {
        widget::text("Details").into()
    };
    widget::column![
        button("Before").on_press(Message::Before),
        crate::rich_tooltip(widget::text("Explain"), "Rich hint", body),
        button("After").on_press(Message::After)
    ]
    .spacing(20)
    .into()
}
fn open_rich(h: &mut Harness<'_, Message>) {
    h.frame();
    key(h, Named::Tab, false);
    key(h, Named::Tab, false);
    key(h, Named::Enter, false);
    h.at(0);
    assert!(visible(h, "Rich hint"));
}
#[test]
fn rich_tooltip_tab_exits_to_following_page_control() {
    let mut h = ui(rich(true));
    open_rich(&mut h);
    key(&mut h, Named::Tab, false); // First -> Second
    key(&mut h, Named::Tab, false); // Second -> After
    key(&mut h, Named::Enter, false);
    assert_eq!(h.messages, [Message::After]);
}
#[test]
fn rich_tooltip_shift_tab_returns_to_trigger_then_previous_control() {
    let mut h = ui(rich(true));
    open_rich(&mut h);
    key(&mut h, Named::Tab, true); // First -> trigger
    h.at(1);
    assert!(!visible(&mut h, "Rich hint"));
    key(&mut h, Named::Tab, true);
    key(&mut h, Named::Enter, false);
    assert_eq!(h.messages, [Message::Before]);
}
#[test]
fn rich_tooltip_without_actions_does_not_swallow_tab() {
    let mut h = ui(rich(false));
    open_rich(&mut h);
    key(&mut h, Named::Tab, false);
    key(&mut h, Named::Enter, false);
    assert_eq!(h.messages, [Message::After]);
}
#[test]
fn rich_tooltip_escape_restores_invoker_and_menu_tab_stays_contained() {
    let mut h = ui(rich(true));
    open_rich(&mut h);
    key(&mut h, Named::Escape, false);
    key(&mut h, Named::Enter, false);
    h.at(1);
    assert!(visible(&mut h, "Rich hint"));
    key(&mut h, Named::Enter, false);
    assert_eq!(h.messages, [Message::First]);
    let mut h = ui(widget::column![
        crate::menu(
            "Menu",
            [
                crate::MenuItem::new("First", Message::First),
                crate::MenuItem::new("Second", Message::Second)
            ]
        ),
        button("After").on_press(Message::After)
    ]);
    h.frame();
    key(&mut h, Named::Tab, false);
    key(&mut h, Named::Enter, false);
    h.at(0);
    key(&mut h, Named::Tab, false);
    key(&mut h, Named::Tab, false);
    key(&mut h, Named::Enter, false);
    assert_eq!(h.messages, [Message::First]);
}
type Calls = Rc<RefCell<Vec<Message>>>;
fn record<T>(calls: &Calls, f: impl Fn(T) -> Message + 'static) -> impl Fn(T) -> Message + 'static {
    let calls = calls.clone();
    move |v| {
        let message = f(v);
        calls.borrow_mut().push(message.clone());
        message
    }
}
#[test]
fn select_callbacks_are_deferred_until_actual_selection() {
    for keyboard in [false, true] {
        let calls = Calls::default();
        let mut h = ui(crate::select(
            "Role",
            [
                crate::SelectOption::new(0, "Viewer"),
                crate::SelectOption::new(1, "Editor"),
                crate::SelectOption::new(2, "Locked").disabled(true),
            ],
            Some(0),
        )
        .on_select(record(&calls, Message::Selected)));
        h.frame();
        h.at(100);
        assert!(
            calls.borrow().is_empty(),
            "building/rendering must not invoke callbacks"
        );
        if keyboard {
            key(&mut h, Named::Tab, false);
            key(&mut h, Named::ArrowDown, false);
        } else {
            h.click("Viewer");
        }
        h.at(101);
        assert!(calls.borrow().is_empty());
        if keyboard {
            key(&mut h, Named::ArrowDown, false);
            key(&mut h, Named::Enter, false);
        } else {
            h.click("Locked");
            assert!(calls.borrow().is_empty());
            h.click("Editor");
        }
        assert_eq!(*calls.borrow(), [Message::Selected(1)]);
        assert_eq!(h.messages, *calls.borrow());
    }
}
fn picker(calls: &Calls, numeric: bool, valid: bool, confirm: bool) -> Element<'static, Message> {
    let p = crate::time_picker(Time::new(10, 30).unwrap(), TimePart::Hour)
        .input_mode(numeric)
        .on_change(record(calls, Message::Change))
        .on_part(record(calls, Message::Part))
        .hour_input(if valid { "10" } else { "" }, Message::Input)
        .minute_input("30", Message::Input)
        .input_period(false, record(calls, Message::Period));
    if confirm {
        p.on_confirm(record(calls, Message::Confirm)).into()
    } else {
        p.into()
    }
}
#[test]
fn time_picker_callbacks_are_deferred_and_submission_prefers_confirm() {
    for numeric in [false, true] {
        for confirm in [false, true] {
            for enter in [false, true] {
                if !numeric && (!confirm || enter) {
                    continue;
                }
                let calls = Calls::default();
                let mut h = ui(picker(&calls, numeric, true, confirm));
                h.frame();
                h.at(0);
                assert!(
                    calls.borrow().is_empty(),
                    "building picker must not invoke callbacks"
                );
                if enter {
                    key(&mut h, Named::Tab, false);
                    key(&mut h, Named::Enter, false);
                } else {
                    h.click(if confirm { "OK" } else { "Apply" });
                }
                let expected = if confirm {
                    Message::Confirm(Time::new(10, 30).unwrap())
                } else {
                    Message::Change(Time::new(10, 30).unwrap())
                };
                assert_eq!(*calls.borrow(), [expected]);
                assert_eq!(h.messages, *calls.borrow());
            }
        }
    }
}
#[test]
fn invalid_time_drafts_cannot_submit_and_period_part_callbacks_wait() {
    let calls = Calls::default();
    let mut h = ui(picker(&calls, true, false, true));
    h.frame();
    assert!(calls.borrow().is_empty());
    h.click("OK");
    key(&mut h, Named::Tab, false);
    key(&mut h, Named::Enter, false);
    assert!(calls.borrow().is_empty());
    h.click("PM");
    assert_eq!(*calls.borrow(), [Message::Period(true)]);
    calls.borrow_mut().clear();
    let mut h = ui(picker(&calls, false, true, true));
    h.frame();
    assert!(calls.borrow().is_empty());
    h.click("30");
    assert_eq!(*calls.borrow(), [Message::Part(TimePart::Minute)]);
    h.click("PM");
    assert_eq!(
        calls.borrow().last(),
        Some(&Message::Change(Time::new(22, 30).unwrap()))
    );
}
#[test]
fn keyboard_tooltip_closes_on_window_unfocus() {
    let mut h = ui(plain());
    h.frame();
    key(&mut h, Named::Tab, false);
    h.at(0);
    h.at(500);
    assert!(visible(&mut h, "Keyboard hint"));
    h.event(Event::Window(window::Event::Unfocused));
    h.at(600);
    assert!(!visible(&mut h, "Keyboard hint"));
}

#[test]
fn callback_rebuilds_and_disabled_controls_do_not_invoke_handlers() {
    let calls = Calls::default();
    let make = |disabled| {
        crate::select(
            "Role",
            [
                crate::SelectOption::new(0, "Viewer"),
                crate::SelectOption::new(1, "Editor"),
            ],
            Some(0),
        )
        .on_select(record(&calls, Message::Selected))
        .disabled(disabled)
    };
    let mut h = ui(make(false));
    // Match the host structure so iced retains the widget tree on rebuild.
    h.rebuild(focus::scope(widget::container(make(true)).padding(24)));
    h.frame();
    h.click("Viewer");
    key(&mut h, Named::Tab, false);
    key(&mut h, Named::Enter, false);
    assert!(calls.borrow().is_empty());
    assert!(h.messages.is_empty());
    h.rebuild(focus::scope(widget::container(make(false)).padding(24)));
    h.frame();
    assert!(calls.borrow().is_empty());
    h.click("Viewer");
    h.at(1);
    h.click("Editor");
    assert_eq!(*calls.borrow(), [Message::Selected(1)]);
    calls.borrow_mut().clear();
    let mut h = ui(picker(&calls, true, true, true));
    h.rebuild(focus::scope(
        widget::container(picker(&calls, true, true, true)).padding(24),
    ));
    h.frame();
    assert!(calls.borrow().is_empty());
    h.click("OK");
    assert_eq!(
        *calls.borrow(),
        [Message::Confirm(Time::new(10, 30).unwrap())]
    );
}

#[test]
fn rich_tooltip_pointer_open_tab_entry_and_action_dismissal() {
    let mut h = ui(rich(true));
    h.frame();
    h.click("Explain");
    h.at(0);
    key(&mut h, Named::Tab, false); // Pointer opening does not preselect an action.
    key(&mut h, Named::Enter, false);
    assert_eq!(h.messages, [Message::First]);
    h.at(1);
    assert!(!visible(&mut h, "Rich hint"));
    key(&mut h, Named::Tab, false);
    key(&mut h, Named::Enter, false);
    assert_eq!(h.messages, [Message::First, Message::After]);
}

#[test]
fn disabled_plain_hint_does_not_disable_its_trigger() {
    let mut h = ui(crate::tooltip(
        button("Trigger").on_press(Message::Trigger),
        "Disabled hint",
    )
    .disabled(true)
    .delay(Duration::ZERO));
    h.frame();
    key(&mut h, Named::Tab, false);
    h.at(0);
    h.at(500);
    assert!(!visible(&mut h, "Disabled hint"));
    key(&mut h, Named::Space, false);
    assert_eq!(h.messages, [Message::Trigger]);
}
