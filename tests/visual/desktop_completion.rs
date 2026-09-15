use super::{harness::Harness, reference, theme};
use crate::{Date, DateSelection, Element, TypeScale, button, dialog, focus, typography};
use iced::{
    Event, Length, Size,
    keyboard::{self, key::Named},
    widget,
};

#[derive(Clone, Debug, PartialEq)]
enum Message {
    Action(u8),
    Close(u8),
    Input(String),
    End(String),
    Toggle,
    Select(DateSelection),
    Month(Date),
}
fn ui(content: impl Into<Element<'static, Message>>, size: Size) -> Harness<'static, Message> {
    Harness::with_backend(
        content,
        size,
        theme(false),
        &std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into()),
    )
}
fn key(name: Named, release: bool, shift: bool) -> Event {
    let key = keyboard::Key::Named(name);
    let physical_key =
        keyboard::key::Physical::Unidentified(keyboard::key::NativeCode::Unidentified);
    let modifiers = if shift {
        keyboard::Modifiers::SHIFT
    } else {
        keyboard::Modifiers::empty()
    };
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
    ui.event(key(name, false, false));
}
fn activate(ui: &mut Harness<'_, Message>) {
    press(ui, Named::Enter);
    ui.event(key(Named::Enter, true, false));
}
fn stack(depth: usize) -> Element<'static, Message> {
    focus::scope(dialog::stack(
        widget::column![
            button("Background first").on_press(Message::Action(0)),
            button("Open parent").on_press(Message::Action(1))
        ]
        .width(Length::Fill)
        .height(Length::Fill),
        (0..3).map(|i| {
            (
                dialog::dialog(
                    widget::column![
                        typography(format!("Dialog {}", i + 1), TypeScale::HeadlineSmall),
                        button(format!("First {}", i + 1)).on_press(Message::Action(10 + i as u8)),
                        button(format!("Next {}", i + 1)).on_press(Message::Action(20 + i as u8)),
                        crate::menu(
                            format!("Menu {}", i + 1),
                            [crate::MenuItem::new("Nested command", Message::Action(90))]
                        ),
                    ]
                    .spacing(16),
                )
                .on_dismiss(Message::Close(i as u8 + 1))
                .width(480.0 - i as f32 * 80.0),
                i < depth,
            )
        }),
    ))
}
#[test]
fn stacked_dialogs_restore_focus_at_each_level_and_trap_tab() {
    let mut ui = ui(stack(0), Size::new(720.0, 560.0));
    press(&mut ui, Named::Tab);
    press(&mut ui, Named::Tab);
    activate(&mut ui);
    for depth in 1..=3 {
        ui.rebuild(stack(depth));
        ui.at(depth as u64 * 1000);
        ui.at(depth as u64 * 1000 + 600);
        activate(&mut ui);
        press(&mut ui, Named::Tab);
        activate(&mut ui);
        // Next -> menu -> first: traversal stays within the top dialog.
        press(&mut ui, Named::Tab);
        press(&mut ui, Named::Tab);
        activate(&mut ui);
        press(&mut ui, Named::Tab);
    }
    assert_eq!(
        ui.messages,
        [
            Message::Action(1),
            Message::Action(10),
            Message::Action(20),
            Message::Action(10),
            Message::Action(11),
            Message::Action(21),
            Message::Action(11),
            Message::Action(12),
            Message::Action(22),
            Message::Action(12)
        ]
    );
    ui.messages.clear();
    for (step, depth) in (0..3).rev().enumerate() {
        press(&mut ui, Named::Escape);
        assert_eq!(ui.messages.pop(), Some(Message::Close(depth as u8 + 1)));
        ui.rebuild(stack(depth));
        ui.at(4000 + step as u64 * 1000);
        activate(&mut ui);
        assert!(ui.messages.is_empty(), "exit still blocks input");
        ui.at(4600 + step as u64 * 1000);
        ui.at(4601 + step as u64 * 1000);
        activate(&mut ui);
        assert_eq!(
            ui.messages.pop(),
            Some(Message::Action(if depth == 0 {
                1
            } else {
                19 + depth as u8
            }))
        );
    }
}
#[test]
fn stacked_dialog_top_scrim_and_nested_popup_consume_dismissal() {
    let mut ui = ui(stack(2), Size::new(720.0, 560.0));
    ui.at(0);
    ui.click("Menu 2");
    ui.at(0);
    ui.at(600);
    press(&mut ui, Named::Escape);
    assert!(ui.messages.is_empty(), "popup receives Escape first");
    ui.at(1000);
    press(&mut ui, Named::Escape);
    assert_eq!(ui.messages, [Message::Close(2)]);
    ui.messages.clear();
    ui.move_to((5.0, 5.0));
    ui.down();
    ui.up();
    assert_eq!(
        ui.messages,
        [Message::Close(2)],
        "backdrop only dismisses the top dialog"
    );
}
#[test]
fn stacked_dialog_cancels_held_parent_gesture() {
    let mut ui = ui(stack(1), Size::new(720.0, 560.0));
    ui.at(0);
    let parent = ui.find("Next 1").center();
    ui.move_to(parent);
    ui.down();
    ui.rebuild(stack(2));
    ui.at(0);
    ui.at(600);
    ui.up();
    assert!(ui.messages.is_empty());
    ui.rebuild(stack(1));
    ui.at(1000);
    ui.at(1600);
    ui.at(1601);
    ui.move_to(parent);
    ui.up();
    assert!(ui.messages.is_empty());
}

#[test]
fn stacked_dialog_restores_native_caret_and_survives_window_deactivation() {
    let make = |depth| {
        focus::scope(dialog::stack(
            button("Open").on_press(Message::Action(1)),
            [
                (
                    dialog::dialog(
                        crate::text_field("Draft", "Draft")
                            .id("draft")
                            .on_input(Message::Input),
                    ),
                    depth > 0,
                ),
                (
                    dialog::dialog(button("Keep editing").on_press(Message::Close(2))),
                    depth > 1,
                ),
            ],
        ))
    };
    let mut ui = ui(make(0), Size::new(600.0, 450.0));
    press(&mut ui, Named::Tab);
    ui.rebuild(make(1));
    ui.at(0);
    ui.at(600);
    use iced::advanced::widget::{
        Operation,
        operation::{Outcome, black_box},
    };
    let mut query = iced_test::Selector::find(widget::Id::new("draft"));
    ui.operate(&mut black_box(&mut query));
    let Outcome::Some(Some(found)) = query.finish() else {
        panic!("missing draft")
    };
    let field = found.visible_bounds().unwrap();
    ui.move_to(field.center());
    ui.down(); // Cover a native editor during an active selection drag.
    ui.operate(
        &mut iced::advanced::widget::operation::text_input::move_cursor_to::<()>(
            widget::Id::new("draft"),
            2,
        ),
    );
    ui.rebuild(make(2));
    ui.at(1000);
    ui.at(1600);
    ui.rebuild(make(1));
    ui.at(2000);
    ui.at(2600);
    ui.at(2601);
    ui.move_to((field.x + field.width - 8.0, field.center_y()));
    press(&mut ui, Named::Backspace);
    assert_eq!(ui.messages, [Message::Input("Daft".into())]);
    ui.messages.clear();
    ui.event(Event::Window(iced::window::Event::Unfocused));
    ui.rebuild(make(0));
    ui.at(3000);
    ui.at(3600);
    ui.event(Event::Window(iced::window::Event::Focused));
    activate(&mut ui);
    assert_eq!(ui.messages, [Message::Action(1)]);
}

#[test]
fn stacked_dialog_surface_occludes_lower_text() {
    let make = |text| {
        dialog::stack(
            widget::space().width(Length::Fill).height(Length::Fill),
            [
                (
                    dialog::dialog(typography(text, TypeScale::HeadlineLarge)).width(520.0),
                    true,
                ),
                (
                    dialog::dialog(typography("Opaque upper", TypeScale::HeadlineSmall))
                        .width(320.0),
                    true,
                ),
            ],
        )
    };
    let mut covered = ui(
        make("Lower text crossing behind the upper dialog"),
        Size::new(600.0, 450.0),
    );
    let mut blank = ui(make(""), Size::new(600.0, 450.0));
    covered.at(0);
    blank.at(0);
    let center = covered.find("Opaque upper").center();
    let crop = |image: super::reference::Image| {
        image.crop(
            (center.x * 2.0) as u32 - 100,
            (center.y * 2.0) as u32 - 16,
            200,
            32,
        )
    };
    let covered = covered.frame();
    let blank = blank.frame();
    covered.write(std::path::Path::new(
        "target/visual-report/stack-occlusion-covered.png",
    ));
    blank.write(std::path::Path::new(
        "target/visual-report/stack-occlusion-blank.png",
    ));
    assert!(crop(covered) == crop(blank));
}
fn picker(
    mode: bool,
    compact: bool,
    value: &str,
    end: &str,
    range: bool,
) -> crate::DatePicker<'static, Message> {
    let date = Date::new(2026, 9, 13).unwrap();
    crate::date_picker(
        date,
        if range {
            DateSelection::Range {
                start: Some(date),
                end: Some(Date::new(2026, 9, 20).unwrap()),
            }
        } else {
            DateSelection::Single(Some(date))
        },
    )
    .today(date)
    .bounds(
        Date::new(2026, 1, 1).unwrap(),
        Date::new(2030, 12, 31).unwrap(),
    )
    .date_enabled(|d| d.day() != 17)
    .on_select(Message::Select)
    .on_month(Message::Month)
    .compact(compact)
    .input_mode(mode)
    .on_toggle_input(Message::Toggle)
    .input(value, Message::Input)
    .end_input(end, Message::End)
}
#[test]
fn picker_manual_entry_enforces_calendar_restrictions_and_range_order() {
    for value in ["2026-02-30", "2025-12-31", "2026-09-17", "2026-09-"] {
        assert!(
            picker(true, false, value, "", false)
                .input_selection()
                .is_err()
        );
    }
    assert!(
        picker(true, false, "2026-09-20", "2026-09-13", true)
            .input_selection()
            .is_err()
    );
    assert_eq!(
        picker(true, false, "2026-09-13", "2026-09-20", true)
            .input_selection()
            .unwrap(),
        DateSelection::Range {
            start: Some(Date::new(2026, 9, 13).unwrap()),
            end: Some(Date::new(2026, 9, 20).unwrap())
        }
    );
    let mut ui = ui(
        picker(true, false, "2026-09-17", "", false),
        Size::new(400.0, 450.0),
    );
    let invalid = ui.frame();
    ui.rebuild(picker(true, false, "2026-09-16", "", false));
    assert!(
        invalid != ui.frame(),
        "unavailable date renders an error state"
    );
    assert!(ui.messages.is_empty());
}
#[test]
fn picker_toggle_keeps_raw_edit_and_docked_navigation_open() {
    let make = |mode| {
        focus::scope(crate::docked_date_picker(
            typography("Calendar", TypeScale::LabelLarge),
            picker(mode, true, "2026-09-", "", false),
        ))
    };
    let mut ui = ui(make(false), Size::new(390.0, 600.0));
    ui.click("Calendar");
    ui.at(0);
    ui.at(600);
    // Pointer-opened popups acquire keyboard focus on Tab.
    press(&mut ui, Named::Tab);
    activate(&mut ui);
    assert_eq!(ui.messages, [Message::Toggle]);
    ui.rebuild(make(true));
    ui.at(601);
    #[derive(Default)]
    struct Inputs(Vec<String>);
    impl iced::advanced::widget::Operation for Inputs {
        fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn iced::advanced::widget::Operation)) {
            visit(self);
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
    let mut inputs = Inputs::default();
    ui.operate(&mut inputs);
    assert_eq!(inputs.0, ["2026-09-"]);
    assert_eq!(
        ui.messages.len(),
        1,
        "switching modes does not commit an incomplete date"
    );
    ui.rebuild(make(false));
    ui.at(602);
    ui.find("September 2026");
    // First focus slot (toggle) survives this rebuild; next is previous month.
    press(&mut ui, Named::Tab);
    activate(&mut ui);
    assert_eq!(
        ui.messages.last(),
        Some(&Message::Month(Date::new(2026, 8, 1).unwrap()))
    );
}

#[test]
fn picker_manual_apply_and_enter_commit_valid_input_and_close_popup() {
    for enter in [false, true] {
        let make = |value: &str| {
            focus::scope(crate::docked_date_picker(
                typography("Calendar", TypeScale::LabelLarge),
                picker(true, true, value, "", false),
            ))
        };
        let mut ui = ui(make("2026-09-"), Size::new(390.0, 600.0));
        ui.click("Calendar");
        ui.at(0);
        ui.at(600);
        ui.click("Apply");
        assert!(ui.messages.is_empty(), "incomplete input cannot be applied");
        ui.rebuild(make("2026-09-16"));
        ui.at(601);
        if enter {
            press(&mut ui, Named::Tab); // toggle
            press(&mut ui, Named::Tab); // native field
            press(&mut ui, Named::Enter);
        } else {
            ui.click("Apply");
        }
        assert_eq!(
            ui.messages,
            [Message::Select(DateSelection::Single(Some(
                Date::new(2026, 9, 16).unwrap()
            )))]
        );
        ui.at(1200);
        ui.click("Calendar");
        ui.at(1201);
        ui.at(1800);
        ui.find("Apply"); // applying closed it; the trigger reopens it
    }
}
#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_desktop_completion() {
    for dark in [false, true] {
        let color = if dark { "dark" } else { "light" };
        for width in [320, 720] {
            let mut ui = Harness::new(stack(0), Size::new(width as f32, 560.0), theme(dark));
            ui.at(0);
            ui.frame();
            for depth in 1..=3 {
                ui.rebuild(stack(depth));
                for ms in [0, 100, 300, 600] {
                    ui.at(depth as u64 * 1000 + ms);
                    reference::check(
                        &format!("desktop-completion-stack/{color}/{width}/open-{depth}-{ms:03}ms"),
                        &ui.frame(),
                    );
                }
            }
            ui.rebuild(stack(1));
            for ms in [0, 100, 200, 600] {
                ui.at(4000 + ms);
                reference::check(
                    &format!("desktop-completion-stack/{color}/{width}/close-two-{ms:03}ms"),
                    &ui.frame(),
                );
            }
            let bars = [
                crate::AppBarVariant::Small,
                crate::AppBarVariant::CenterAligned,
                crate::AppBarVariant::Medium,
                crate::AppBarVariant::Large,
            ];
            let make = |collapse| {
                widget::Column::with_children(bars.map(|variant| {
                    crate::app_bar(
                        "Workspace planning · September release and follow-up milestones",
                    )
                    .variant(variant)
                    .collapse(collapse)
                    .leading(
                        button("←")
                            .on_press(Message::Action(1))
                            .width(40)
                            .padding(0),
                    )
                    .action(button("Save").on_press(Message::Action(2)))
                    .into()
                }))
            };
            for (name, collapse) in [("expanded", 0.0), ("half", 0.5), ("collapsed", 1.0)] {
                let mut ui =
                    Harness::new(make(collapse), Size::new(width as f32, 480.0), theme(dark));
                ui.at(0);
                reference::check(
                    &format!("desktop-completion-titles/{color}/{width}/{name}"),
                    &ui.frame(),
                );
            }
        }
        for compact in [false, true] {
            for range in [false, true] {
                for (name, mode, value, end) in [
                    ("calendar", false, "2026-09-13", "2026-09-20"),
                    ("input", true, "2026-09-13", "2026-09-20"),
                    ("incomplete", true, "2026-09-", ""),
                    ("invalid", true, "2026-09-17", "2026-09-12"),
                ] {
                    let mut ui = Harness::new(
                        focus::scope(picker(mode, compact, value, end, range)),
                        Size::new(390.0, 560.0),
                        theme(dark),
                    );
                    ui.at(0);
                    reference::check(
                        &format!(
                            "desktop-completion-picker/{color}/{}-{}/{name}",
                            if compact { "compact" } else { "modal" },
                            if range { "range" } else { "single" }
                        ),
                        &ui.frame(),
                    );
                }
            }
        }
    }
}
