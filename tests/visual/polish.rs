use super::{harness::Harness, reference, theme, themed_name};
use crate::{Date, DateSelection, Element, MenuItem, TypeScale, button, focus, menu, typography};
use iced::{
    Event, Length, Point, Size,
    keyboard::{self, key::Named},
    widget, window,
};

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Action(u8),
    Month(Date),
    Date(DateSelection),
    Close,
}
fn ui(content: impl Into<Element<'static, Message>>, size: Size) -> Harness<'static, Message> {
    Harness::with_backend(
        content,
        size,
        theme(false),
        &std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into()),
    )
}
fn key(key: keyboard::Key, release: bool, modifiers: keyboard::Modifiers) -> Event {
    let physical_key =
        keyboard::key::Physical::Unidentified(keyboard::key::NativeCode::Unidentified);
    if release {
        Event::Keyboard(keyboard::Event::KeyReleased {
            key: key.clone(),
            modified_key: key,
            physical_key,
            location: keyboard::Location::Standard,
            modifiers,
        })
    } else {
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key,
            location: keyboard::Location::Standard,
            modifiers,
            text: None,
            repeat: false,
        })
    }
}
fn press(ui: &mut Harness<'_, Message>, name: Named) {
    ui.event(key(
        keyboard::Key::Named(name),
        false,
        keyboard::Modifiers::empty(),
    ));
}
fn activate(ui: &mut Harness<'_, Message>) {
    press(ui, Named::Enter);
    ui.event(key(
        keyboard::Key::Named(Named::Enter),
        true,
        keyboard::Modifiers::empty(),
    ));
}
fn type_key(ui: &mut Harness<'_, Message>, text: &str) {
    ui.event(key(
        keyboard::Key::Character(text.into()),
        false,
        keyboard::Modifiers::empty(),
    ));
}
fn actions() -> Element<'static, Message> {
    focus::scope(
        widget::container(menu(
            "Commands",
            [
                MenuItem::new("Alpha", Message::Action(0)),
                MenuItem::new("Alpine", Message::Action(1)),
                MenuItem::new("Beta", Message::Action(2)).disabled(true),
                MenuItem::new("Bravo", Message::Action(3)),
                MenuItem::new("Zürich", Message::Action(4)),
            ],
        ))
        .padding(24),
    )
}
fn open(ui: &mut Harness<'_, Message>) {
    press(ui, Named::Tab);
    press(ui, Named::ArrowDown);
    ui.at(0);
    ui.at(500);
}
#[test]
fn menu_typeahead_prefix_repeat_timeout_unicode_and_disabled_rows() {
    let mut ui = ui(actions(), Size::new(540.0, 460.0));
    open(&mut ui);
    for text in ["A", "l", "p", "i"] {
        type_key(&mut ui, text);
    }
    assert!(ui.messages.is_empty());
    activate(&mut ui);
    assert_eq!(ui.messages, [Message::Action(1)]);
    for (text, expected) in [("a", 0), ("b", 3), ("ZÜ", 4)] {
        let mut ui = self::ui(actions(), Size::new(540.0, 460.0));
        open(&mut ui);
        type_key(&mut ui, text);
        if text == "a" {
            type_key(&mut ui, "a");
        }
        activate(&mut ui);
        assert_eq!(ui.messages, [Message::Action(expected)]);
    }
    let mut ui = self::ui(actions(), Size::new(540.0, 460.0));
    open(&mut ui);
    type_key(&mut ui, "z");
    ui.at(750);
    type_key(&mut ui, "b");
    ui.event(key(
        keyboard::Key::Character("a".into()),
        false,
        keyboard::Modifiers::CTRL,
    ));
    activate(&mut ui);
    assert_eq!(ui.messages, [Message::Action(3)]);
}
fn calendar(month: Date, selected: Date, docked: bool) -> Element<'static, Message> {
    let picker = crate::date_picker(month, DateSelection::Single(Some(selected)))
        .bounds(
            Date::new(2025, 1, 1).unwrap(),
            Date::new(2028, 12, 31).unwrap(),
        )
        .on_month(Message::Month)
        .on_select(Message::Date);
    focus::scope(if docked {
        crate::docked_date_picker(typography("Calendar", TypeScale::LabelLarge), picker)
    } else {
        picker.into()
    })
}
#[test]
fn calendar_cross_month_keys_keep_focus_and_do_not_select_until_activation() {
    for docked in [false, true] {
        let jan = Date::new(2026, 1, 31).unwrap();
        let feb = Date::new(2026, 2, 1).unwrap();
        let mut ui = ui(calendar(jan, jan, docked), Size::new(500.0, 640.0));
        ui.at(0);
        if docked {
            ui.click("Calendar");
            ui.at(0);
            ui.at(500);
        }
        ui.click("31");
        if docked {
            ui.click("Calendar");
            ui.at(500);
            ui.at(1000);
        }
        ui.messages.clear();
        if docked {
            press(&mut ui, Named::Tab);
            press(&mut ui, Named::Tab);
            press(&mut ui, Named::Tab);
        }
        press(&mut ui, Named::PageDown);
        assert_eq!(ui.messages, [Message::Month(feb)]);
        ui.rebuild(calendar(feb, jan, docked));
        activate(&mut ui);
        assert_eq!(
            ui.messages.last(),
            Some(&Message::Date(DateSelection::Single(Some(
                Date::new(2026, 2, 28).unwrap()
            ))))
        );
    }
}
#[test]
fn calendar_arrow_crosses_year_and_shift_page_moves_one_year() {
    let dec = Date::new(2026, 12, 31).unwrap();
    let jan = Date::new(2027, 1, 1).unwrap();
    let mut ui = ui(calendar(dec, dec, false), Size::new(400.0, 560.0));
    ui.click("31");
    ui.messages.clear();
    press(&mut ui, Named::ArrowRight);
    assert_eq!(ui.messages, [Message::Month(jan)]);
    ui.rebuild(calendar(jan, dec, false));
    activate(&mut ui);
    assert_eq!(
        ui.messages.last(),
        Some(&Message::Date(DateSelection::Single(Some(jan))))
    );
    ui.event(key(
        keyboard::Key::Named(Named::PageDown),
        false,
        keyboard::Modifiers::SHIFT,
    ));
    assert_eq!(
        ui.messages.last(),
        Some(&Message::Month(Date::new(2028, 1, 1).unwrap()))
    );
}
fn cascade() -> Element<'static, Message> {
    widget::container(menu(
        "Menu",
        [
            MenuItem::submenu(
                "First branch",
                [
                    MenuItem::new("One", Message::Action(1)),
                    MenuItem::new("Two", Message::Action(2)),
                    MenuItem::new("Three", Message::Action(3)),
                ],
            ),
            MenuItem::submenu(
                "Second branch",
                [MenuItem::new("Other", Message::Action(4))],
            ),
        ],
    ))
    .padding(24)
    .into()
}
#[test]
fn diagonal_submenu_movement_has_a_bounded_grace_period() {
    let mut ui = ui(cascade(), Size::new(760.0, 420.0));
    ui.at(0);
    ui.click("Menu");
    ui.at(0);
    ui.at(500);
    let first = ui.find("First branch");
    ui.move_to(Point::new(first.x + 12.0, first.center_y()));
    ui.at(700);
    let second = ui.find("Second branch");
    ui.move_to(Point::new(first.x + first.width - 8.0, second.center_y()));
    ui.at(850);
    ui.find("One");
    ui.at(1001);
    ui.at(1201);
    ui.find("Other");
}
fn dialog(open: bool, full: bool) -> Element<'static, Message> {
    let content = widget::column![
        typography("Review changes", TypeScale::HeadlineSmall),
        button("Apply").on_press(Message::Action(1))
    ]
    .spacing(24);
    let dialog = if full {
        crate::full_screen_dialog("Review", content)
            .on_dismiss(Message::Close)
            .into()
    } else {
        crate::dialog(content).on_dismiss(Message::Close)
    };
    focus::scope(crate::dialog::modal(
        widget::container(button("Background").on_press(Message::Action(9)))
            .width(Length::Fill)
            .height(Length::Fill),
        dialog,
        open,
    ))
}
#[test]
fn dialog_exit_blocks_input_cancels_held_actions_and_restores_invoker() {
    for full in [false, true] {
        let mut ui = ui(dialog(false, full), Size::new(600.0, 500.0));
        ui.at(0);
        ui.frame();
        press(&mut ui, Named::Tab);
        ui.rebuild(dialog(true, full));
        ui.at(0);
        ui.at(300);
        let action = ui.find("Apply").center();
        ui.move_to(action);
        ui.down();
        ui.rebuild(dialog(false, full));
        ui.at(300);
        ui.up();
        ui.move_to((35.0, 20.0));
        ui.down();
        ui.up();
        activate(&mut ui);
        assert!(ui.messages.is_empty());
        ui.at(800);
        activate(&mut ui);
        assert_eq!(ui.messages, [Message::Action(9)]);
        ui.at(1300);
        assert_eq!(ui.at(1301), window::RedrawRequest::Wait);
    }
}
#[test]
fn dialog_reverses_without_jumping_and_reduced_motion_settles_immediately() {
    let mut ui = ui(dialog(false, false), Size::new(600.0, 500.0));
    ui.at(0);
    ui.frame();
    ui.rebuild(dialog(true, false));
    ui.at(0);
    ui.at(75);
    let opening = ui.find("Apply").y;
    ui.rebuild(dialog(false, false));
    ui.at(75);
    let closing = ui.frame();
    ui.rebuild(dialog(true, false));
    ui.at(75);
    assert!((ui.find("Apply").y - opening).abs() < 0.1);
    assert!(ui.frame() == closing);
    ui.at(500);
    let mut ui = Harness::new(
        dialog(false, false),
        Size::new(600.0, 500.0),
        theme(false).reduced_motion(true),
    );
    ui.at(0);
    ui.frame();
    ui.rebuild(dialog(true, false));
    ui.at(0);
    let mut settled = Harness::new(
        dialog(true, false),
        Size::new(600.0, 500.0),
        theme(false).reduced_motion(true),
    );
    assert!((ui.find("Apply").y - settled.find("Apply").y).abs() < 0.1);
}
fn notice(visible: bool, id: u64) -> Element<'static, Message> {
    crate::snackbar::host(
        widget::container(button("Background").on_press(Message::Action(9)))
            .align_bottom(Length::Fill)
            .width(Length::Fill),
        Some(
            crate::snackbar("Workspace unpinned")
                .id(id)
                .visible(visible)
                .action("Undo", Message::Action(1))
                .on_dismiss(Message::Close)
                .duration(std::time::Duration::from_secs(1)),
        ),
    )
}
#[test]
fn snackbar_exit_cancels_held_action_and_new_notice_reverses_exit() {
    let mut ui = ui(notice(true, 1), Size::new(600.0, 300.0));
    ui.at(0);
    ui.frame();
    let undo = ui.find("Undo").center();
    ui.move_to(undo);
    ui.down();
    ui.rebuild(notice(false, 1));
    ui.at(0);
    ui.up();
    assert!(ui.messages.is_empty());
    ui.at(50);
    let closing = ui.frame();
    ui.rebuild(notice(true, 2));
    ui.at(50);
    // The new identity clears the old action's hover/press state, while the
    // surface and message reverse from the same painted position.
    assert!(ui.frame().crop(0, 0, 800, 600) == closing.crop(0, 0, 800, 600));
    ui.at(400);
    ui.click("Undo");
    assert_eq!(ui.messages, [Message::Action(1)]);
    ui.at(400);
    ui.at(700);
    assert_eq!(ui.at(701), window::RedrawRequest::Wait);
}
fn ripple() -> Element<'static, Message> {
    widget::container(
        button("")
            .width(220)
            .height(48)
            .on_press(Message::Action(1)),
    )
    .padding(24)
    .into()
}
#[test]
fn button_ripple_starts_at_pointer_preserves_hover_and_stays_inside_rounding() {
    // Isolate ripple clipping: the hover elevation intentionally drops on press,
    // and its ambient shadow now reaches into the outside rounded-corner crop.
    let mut theme = theme(false);
    theme.shadows = false;
    let mut ui = Harness::with_backend(
        ripple(),
        Size::new(290.0, 110.0),
        theme,
        &std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into()),
    );
    ui.at(0);
    ui.frame();
    ui.move_to((52.0, 48.0));
    ui.at(150);
    let hover = ui.frame();
    ui.down();
    let initial = ui.frame();
    assert!(hover.crop(96, 88, 16, 16) != initial.crop(96, 88, 16, 16));
    assert!(
        hover
            .crop(96, 88, 16, 16)
            .pixels
            .iter()
            .zip(initial.crop(96, 88, 16, 16).pixels)
            .all(|(before, after)| before.abs_diff(after) <= 32),
        "The 12% ripple must remain translucent"
    );
    assert!(hover.crop(400, 88, 16, 16) == initial.crop(400, 88, 16, 16));
    assert!(hover.crop(48, 48, 8, 8) == initial.crop(48, 48, 8, 8));
    ui.at(300);
    assert!(ui.frame() != initial);
    assert!(ui.messages.is_empty());
    ui.at(700);
    assert_eq!(ui.at(701), window::RedrawRequest::Wait);
    ui.up();
    assert_eq!(ui.messages, [Message::Action(1)]);
    ui.at(1000);
    assert!(ui.frame() == hover);
    assert_eq!(ui.at(1001), window::RedrawRequest::Wait);
}
#[test]
fn quick_button_click_publishes_immediately_and_finishes_its_feedback() {
    let mut ui = ui(ripple(), Size::new(290.0, 110.0));
    ui.at(0);
    ui.frame();
    ui.move_to((52.0, 48.0));
    ui.at(150);
    let hover = ui.frame();
    ui.down();
    ui.up();
    assert_eq!(ui.messages, [Message::Action(1)]);
    ui.at(250);
    assert!(ui.frame() != hover);
    ui.at(600);
    assert!(ui.frame() == hover);
    assert_eq!(ui.at(601), window::RedrawRequest::Wait);
}

#[test]
fn calendar_skips_disabled_days_in_direction_and_respects_bounds() {
    let day = Date::new(2026, 9, 10).unwrap();
    let picker = crate::date_picker(day, DateSelection::Single(Some(day)))
        .bounds(
            Date::new(2026, 9, 8).unwrap(),
            Date::new(2026, 9, 12).unwrap(),
        )
        .date_enabled(|d| d.day() != 9 && d.day() != 11)
        .on_month(Message::Month)
        .on_select(Message::Date);
    let mut ui = ui(focus::scope(picker), Size::new(400.0, 540.0));
    ui.click("10");
    ui.messages.clear();
    press(&mut ui, Named::ArrowLeft);
    activate(&mut ui);
    assert_eq!(
        ui.messages,
        [Message::Date(DateSelection::Single(Some(
            Date::new(2026, 9, 8).unwrap()
        )))]
    );
    press(&mut ui, Named::PageUp);
    activate(&mut ui);
    assert_eq!(ui.messages.last(), ui.messages.first());
    assert!(!ui.messages.iter().any(|m| matches!(m, Message::Month(_))));
}

#[test]
fn reduced_motion_preserves_tooltip_delay_and_settles_popup_lifetimes() {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    let reduced = theme(false).reduced_motion(true);
    let mut ui = Harness::with_backend(
        actions(),
        Size::new(540.0, 460.0),
        reduced.clone(),
        &backend,
    );
    ui.at(0);
    ui.frame();
    ui.click("Commands");
    ui.at(0);
    ui.find("Bravo");
    press(&mut ui, Named::Escape);
    ui.at(0);
    assert_eq!(ui.at(1), window::RedrawRequest::Wait);
    let mut ui = Harness::with_backend(
        crate::tooltip(button("Help").on_press(Message::Action(1)), "Hint"),
        Size::new(320.0, 180.0),
        reduced.clone(),
        &backend,
    );
    ui.at(0);
    ui.frame();
    let p = ui.find("Help").center();
    ui.move_to(p);
    assert!(matches!(ui.at(499), window::RedrawRequest::At(_)));
    ui.at(500);
    ui.find("Hint");
    assert_eq!(ui.at(501), window::RedrawRequest::Wait);
    let mut ui =
        Harness::with_backend(notice(false, 1), Size::new(600.0, 300.0), reduced, &backend);
    ui.at(0);
    ui.frame();
    ui.rebuild(notice(true, 1));
    ui.at(0);
    ui.find("Undo");
    ui.rebuild(notice(false, 1));
    ui.at(0);
    assert_eq!(ui.at(1), window::RedrawRequest::Wait);
}
#[test]
fn menus_finish_exit_without_leaking_clicks_or_scheduling_idle_frames() {
    let mut ui = ui(actions(), Size::new(540.0, 460.0));
    open(&mut ui);
    let open = ui.frame();
    ui.click("Alpha");
    assert_eq!(ui.messages, [Message::Action(0)]);
    ui.at(500);
    let exit = ui.frame();
    assert!(exit != open);
    ui.down();
    ui.up();
    assert_eq!(ui.messages, [Message::Action(0)]);
    ui.at(700);
    assert!(ui.frame() != exit);
    assert_eq!(ui.at(701), window::RedrawRequest::Wait);
}

#[test]
fn opening_menu_clips_nested_layers_to_its_animated_edge() {
    let mut ui = ui(actions(), Size::new(540.0, 460.0));
    ui.at(0);
    ui.frame();
    let background = ui.frame();
    ui.click("Commands");
    ui.at(100);
    let partial = ui.frame();
    partial.write(std::path::Path::new(
        "target/visual-report/polish-menu-clipping.png",
    ));
    assert!(
        partial.crop(48, 440, 450, 140) == background.crop(48, 440, 450, 140),
        "Rows beyond the revealed edge plus its 14px ambient shadow extent must stay hidden"
    );
    ui.at(500);
    assert!(ui.frame().crop(48, 440, 450, 140) != background.crop(48, 440, 450, 140));
}

#[test]
fn menu_invoker_finishes_its_ripple_while_the_panel_stays_open() {
    let mut ui = ui(actions(), Size::new(540.0, 460.0));
    ui.at(0);
    let original = ui.frame();
    ui.click("Commands");
    ui.at(0);
    ui.at(1000);
    ui.find("Bravo");
    assert!(ui.frame().crop(0, 0, 800, 128) == original.crop(0, 0, 800, 128));
    assert_eq!(ui.at(1001), window::RedrawRequest::Wait);
}
#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_motion_polish() {
    for dark in [false, true] {
        let mut ui = Harness::new(ripple(), Size::new(290.0, 110.0), theme(dark));
        ui.at(0);
        ui.frame();
        ui.move_to((52.0, 48.0));
        ui.at(150);
        ui.down();
        let case = themed_name("polish-button-ripple", dark);
        for t in [0, 50, 100, 225, 350, 450, 600] {
            ui.at(150 + t);
            reference::check(&format!("{case}/01-held-{t:03}ms"), &ui.frame());
        }
        ui.up();
        for t in [0, 50, 100, 150, 300] {
            ui.at(750 + t);
            reference::check(&format!("{case}/02-release-{t:03}ms"), &ui.frame());
        }
        let mut ui = Harness::new(actions(), Size::new(540.0, 460.0), theme(dark));
        ui.at(0);
        ui.frame();
        ui.click("Commands");
        let case = themed_name("polish-menu", dark);
        for t in [0, 50, 100, 200, 350, 500] {
            ui.at(t);
            reference::check(&format!("{case}/01-open-{t:03}ms"), &ui.frame());
        }
        type_key(&mut ui, "b");
        reference::check(&format!("{case}/02-typeahead"), &ui.frame());
        press(&mut ui, Named::Escape);
        for t in [0, 50, 100, 150, 250] {
            ui.at(500 + t);
            reference::check(&format!("{case}/03-close-{t:03}ms"), &ui.frame());
        }
        for full in [false, true] {
            let mut ui = Harness::new(dialog(false, full), Size::new(600.0, 500.0), theme(dark));
            ui.at(0);
            ui.frame();
            ui.rebuild(dialog(true, full));
            let case = themed_name(
                if full {
                    "polish-full-screen-dialog"
                } else {
                    "polish-dialog"
                },
                dark,
            );
            for t in [0, 50, 100, 200, 300] {
                ui.at(t);
                reference::check(&format!("{case}/01-open-{t:03}ms"), &ui.frame());
            }
            ui.rebuild(dialog(false, full));
            for t in [0, 50, 100, 200, 300] {
                ui.at(300 + t);
                reference::check(&format!("{case}/02-close-{t:03}ms"), &ui.frame());
            }
        }
        let mut ui = Harness::new(notice(false, 1), Size::new(600.0, 300.0), theme(dark));
        ui.at(0);
        ui.frame();
        ui.rebuild(notice(true, 1));
        let case = themed_name("polish-snackbar", dark);
        for t in [0, 50, 100, 150, 250] {
            ui.at(t);
            reference::check(&format!("{case}/01-open-{t:03}ms"), &ui.frame());
        }
        ui.rebuild(notice(false, 1));
        for t in [0, 50, 100, 200, 300] {
            ui.at(250 + t);
            reference::check(&format!("{case}/02-close-{t:03}ms"), &ui.frame());
        }
        let hint = widget::container(crate::tooltip(
            button("Hover for help").on_press(Message::Action(1)),
            "A helpful hint",
        ))
        .padding(48);
        let mut ui = Harness::new(hint, Size::new(340.0, 180.0), theme(dark));
        ui.at(0);
        ui.frame();
        let p = ui.find("Hover for help").center();
        ui.move_to(p);
        let case = themed_name("polish-tooltip", dark);
        for t in [500, 550, 600, 650, 800] {
            ui.at(t);
            reference::check(&format!("{case}/01-open-{:03}ms", t - 500), &ui.frame());
        }
        ui.leave();
        for t in [0, 25, 50, 100, 200] {
            ui.at(800 + t);
            reference::check(&format!("{case}/02-close-{t:03}ms"), &ui.frame());
        }
    }
}
