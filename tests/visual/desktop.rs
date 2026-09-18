use super::{harness::Harness, theme};
use crate::{
    Element, NavigationItem, TypeScale, button, focus, navigation_bar, navigation_rail, typography,
};
use iced::{
    Event, Length, Size,
    keyboard::{self, key::Named},
    widget,
};
#[derive(Clone, Debug, PartialEq)]
enum Message {
    Action(u8),
    Close,
    Date(crate::DateSelection),
    Month(crate::Date),
    Input(String),
}
fn ui(content: impl Into<Element<'static, Message>>, size: Size) -> Harness<'static, Message> {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    Harness::with_backend(content, size, theme(false), &backend)
}
fn key(k: Named, release: bool, shift: bool) -> Event {
    let key = keyboard::Key::Named(k);
    let modifiers = if shift {
        keyboard::Modifiers::SHIFT
    } else {
        keyboard::Modifiers::empty()
    };
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
fn press(ui: &mut Harness<'_, Message>, k: Named) {
    ui.event(key(k, false, false));
}
fn activate(ui: &mut Harness<'_, Message>) {
    press(ui, Named::Space);
    ui.event(key(Named::Space, true, false));
}
fn items() -> Vec<NavigationItem<'static, u8, Message>> {
    (0..12)
        .map(|i| {
            NavigationItem::new(
                i,
                format!("Page {i}"),
                typography(i.to_string(), TypeScale::TitleLarge),
            )
            .disabled(i == 2)
        })
        .collect()
}
#[test]
fn group_has_one_tab_stop_and_remembers_arrow_focus() {
    let mut ui = ui(
        focus::scope(widget::column![
            button("Before").on_press(Message::Action(20)),
            navigation_bar(items().into_iter().take(4), Some(1)).on_select(Message::Action),
            button("After").on_press(Message::Action(21))
        ]),
        Size::new(500.0, 220.0),
    );
    press(&mut ui, Named::Tab);
    press(&mut ui, Named::Tab);
    activate(&mut ui);
    press(&mut ui, Named::ArrowRight);
    activate(&mut ui);
    press(&mut ui, Named::Tab);
    activate(&mut ui);
    ui.event(key(Named::Tab, false, true));
    activate(&mut ui);
    assert_eq!(
        ui.messages,
        [
            Message::Action(1),
            Message::Action(3),
            Message::Action(21),
            Message::Action(3)
        ]
    );
}
#[test]
fn focused_rail_item_scrolls_into_view_and_home_returns_to_top() {
    let mut ui = ui(
        focus::scope(navigation_rail(items(), Some(0)).on_select(Message::Action)),
        Size::new(80.0, 250.0),
    );
    press(&mut ui, Named::Tab);
    press(&mut ui, Named::End);
    let bottom = ui.find("Page 11");
    assert!(bottom.y >= 0.0 && bottom.y + bottom.height <= 250.0);
    activate(&mut ui);
    press(&mut ui, Named::Home);
    let top = ui.find("Page 0");
    assert!(top.y >= 0.0 && top.y + top.height <= 250.0);
    activate(&mut ui);
    assert_eq!(ui.messages, [Message::Action(11), Message::Action(0)]);
}
#[test]
fn tab_reveals_native_input_inside_nested_scrollables() {
    let field = crate::text_field("Last field", "")
        .id("last-field")
        .on_input(Message::Input);
    let inner = widget::scrollable(widget::column![widget::space().height(300), field]).height(160);
    let content = widget::scrollable(widget::column![
        button("First").on_press(Message::Action(1)),
        widget::space().height(400),
        inner
    ])
    .height(Length::Fill);
    let mut ui = ui(focus::scope(content), Size::new(360.0, 240.0));
    press(&mut ui, Named::Tab);
    press(&mut ui, Named::Tab);
    use iced::advanced::widget::{
        Operation,
        operation::{Outcome, black_box},
    };
    let mut query = iced_test::Selector::find(widget::Id::new("last-field"));
    ui.operate(&mut black_box(&mut query));
    let Outcome::Some(Some(found)) = query.finish() else {
        panic!("missing field")
    };
    let bounds = found.visible_bounds().expect("focused input visible");
    assert!(bounds.y >= 0.0 && bounds.y + bounds.height <= 240.0);
}
#[test]
fn date_grid_arrows_move_by_calendar_rows() {
    let date = crate::Date::new(2026, 9, 9).unwrap();
    let picker =
        crate::date_picker(date, crate::DateSelection::Single(Some(date))).on_select(|s| match s {
            crate::DateSelection::Single(Some(d)) => Message::Action(d.day()),
            _ => unreachable!(),
        });
    let mut ui = ui(focus::scope(picker), Size::new(400.0, 500.0));
    press(&mut ui, Named::Tab);
    press(&mut ui, Named::ArrowDown);
    activate(&mut ui);
    assert_eq!(ui.messages, [Message::Action(16)]);
}

fn visible(ui: &mut Harness<'_, Message>, label: &str) -> Option<iced::Rectangle> {
    use iced::advanced::widget::{
        Operation,
        operation::{Outcome, black_box},
    };
    let mut query = iced_test::Selector::find(label);
    ui.operate(&mut black_box(&mut query));
    match query.finish() {
        Outcome::Some(Some(found)) => found.visible_bounds(),
        _ => None,
    }
}
fn cascade() -> crate::Menu<'static, Message> {
    crate::menu(
        "Actions",
        [
            crate::MenuItem::new("Open", Message::Action(1)),
            crate::MenuItem::submenu(
                "Export",
                [
                    crate::MenuItem::new("PDF", Message::Action(2)),
                    crate::MenuItem::submenu(
                        "Image",
                        [crate::MenuItem::new("PNG", Message::Action(3))],
                    ),
                ],
            ),
            crate::MenuItem::submenu(
                "Unavailable",
                [crate::MenuItem::new("Hidden", Message::Action(4))],
            )
            .disabled(true),
            crate::MenuItem::new("Last action", Message::Action(5)),
        ],
    )
}
fn open_cascade(ui: &mut Harness<'_, Message>) {
    press(ui, Named::Tab);
    activate(ui);
    ui.at(0);
    press(ui, Named::ArrowDown);
    press(ui, Named::ArrowRight);
    ui.at(0);
}
#[test]
fn cascades_open_by_keyboard_and_leaf_action_closes_every_panel() {
    let mut ui = ui(focus::scope(cascade()), Size::new(720.0, 400.0));
    open_cascade(&mut ui);
    assert!(visible(&mut ui, "PDF").is_some());
    press(&mut ui, Named::ArrowDown);
    press(&mut ui, Named::ArrowRight);
    ui.at(0);
    assert!(visible(&mut ui, "PNG").is_some());
    activate(&mut ui);
    ui.at(1);
    assert_eq!(ui.messages, [Message::Action(3)]);
    assert!(visible(&mut ui, "Export").is_none());
    assert!(visible(&mut ui, "PNG").is_none());
    activate(&mut ui);
    ui.at(2);
    assert!(visible(&mut ui, "Export").is_some());
}
#[test]
fn cascade_left_and_escape_return_focus_one_level_at_a_time() {
    let mut ui = ui(focus::scope(cascade()), Size::new(720.0, 400.0));
    open_cascade(&mut ui);
    press(&mut ui, Named::ArrowLeft);
    ui.at(1);
    assert!(visible(&mut ui, "PDF").is_none());
    press(&mut ui, Named::ArrowRight);
    ui.at(2);
    assert!(visible(&mut ui, "PDF").is_some());
    press(&mut ui, Named::Escape);
    ui.at(3);
    assert!(visible(&mut ui, "Export").is_some());
    press(&mut ui, Named::Escape);
    ui.at(4);
    assert!(visible(&mut ui, "Export").is_none());
    assert!(ui.messages.is_empty());
}
#[test]
fn cascade_hover_delay_siblings_and_window_edge_flip() {
    let content = widget::container(cascade()).align_right(Length::Fill);
    let mut ui = ui(focus::scope(content), Size::new(560.0, 350.0));
    ui.click("Actions");
    ui.at(0);
    let anchor = ui.find("Export");
    ui.move_to(anchor.center());
    ui.at(199);
    assert!(visible(&mut ui, "PDF").is_none());
    ui.at(200);
    let child = ui.find("PDF");
    assert!(child.x < anchor.x, "submenu flips left at right edge");
    let last = ui.find("Last action");
    ui.move_to(last.center());
    ui.at(201);
    assert!(visible(&mut ui, "PDF").is_none());
    ui.click("Unavailable");
    ui.at(202);
    assert!(visible(&mut ui, "Hidden").is_none());
    assert!(ui.messages.is_empty());
}
#[test]
fn submenu_escape_does_not_dismiss_containing_dialog() {
    let dialog = crate::dialog(cascade()).on_dismiss(Message::Close);
    let mut ui = ui(
        focus::scope(crate::dialog::modal(button("Background"), dialog, true)),
        Size::new(700.0, 400.0),
    );
    ui.click("Actions");
    ui.click("Export");
    ui.at(0);
    press(&mut ui, Named::Escape);
    ui.at(1);
    assert!(ui.messages.is_empty());
    press(&mut ui, Named::Escape);
    ui.at(2);
    assert!(ui.messages.is_empty());
    press(&mut ui, Named::Escape);
    assert_eq!(ui.messages, [Message::Close]);
}
#[test]
fn full_screen_dialog_keeps_header_visible_while_focusing_scrolled_body() {
    let dialog = crate::full_screen_dialog(
        "Edit workspace",
        widget::column![
            widget::space().height(1200),
            button("Last field").on_press(Message::Action(2))
        ],
    )
    .on_dismiss(Message::Close)
    .action(button("Save").on_press(Message::Action(1)))
    .into();
    let mut ui = ui(
        focus::scope(crate::dialog::modal(
            button("Background").on_press(Message::Action(9)),
            dialog,
            true,
        )),
        Size::new(390.0, 500.0),
    );
    ui.at(0);
    press(&mut ui, Named::Tab);
    activate(&mut ui);
    press(&mut ui, Named::Tab);
    let last = ui.find("Last field");
    assert!(last.y >= 64.0 && last.y + last.height <= 500.0);
    let save = ui.find("Save");
    assert!(save.y < 64.0);
    activate(&mut ui);
    press(&mut ui, Named::Escape);
    assert_eq!(
        ui.messages,
        [Message::Action(1), Message::Action(2), Message::Close]
    );
    assert!(visible(&mut ui, "Background").is_none());
}
fn date_popup(month: crate::Date, selection: crate::DateSelection) -> Element<'static, Message> {
    crate::docked_date_picker(
        typography("Choose date", TypeScale::LabelLarge),
        crate::date_picker(month, selection)
            .on_month(Message::Month)
            .on_select(Message::Date),
    )
}
#[test]
fn docked_picker_month_navigation_stays_open_and_selection_closes() {
    let september = crate::Date::new(2026, 9, 1).unwrap();
    let october = september.add_months(1).unwrap();
    let mut ui = ui(
        focus::scope(date_popup(september, crate::DateSelection::Single(None))),
        Size::new(500.0, 640.0),
    );
    ui.click("Choose date");
    ui.at(0);
    let nine = ui.find("9");
    ui.click("9");
    ui.at(1);
    assert!(nine.y > 40.0);
    assert_eq!(
        ui.messages,
        [Message::Date(crate::DateSelection::Single(Some(
            crate::Date::new(2026, 9, 9).unwrap()
        )))]
    );
    assert!(visible(&mut ui, "September 2026").is_none());
    ui.click("Choose date");
    ui.at(2);
    // Keyboard starts at the first enabled heading control: previous month.
    press(&mut ui, Named::Tab);
    press(&mut ui, Named::Tab);
    activate(&mut ui);
    ui.at(3);
    assert_eq!(ui.messages.last(), Some(&Message::Month(october)));
    ui.rebuild(focus::scope(date_popup(
        october,
        crate::DateSelection::Single(None),
    )));
    ui.at(4);
    assert!(visible(&mut ui, "October 2026").is_some());
    press(&mut ui, Named::Escape);
    ui.at(5);
    assert!(visible(&mut ui, "October 2026").is_none());
}

fn rail_shell(expanded: bool) -> Element<'static, Message> {
    focus::scope(
        widget::row![
            navigation_rail(items().into_iter().take(4), Some(1))
                .expanded(expanded)
                .on_select(Message::Action),
            button("Content").on_press(Message::Action(20))
        ]
        .height(Length::Fill),
    )
}
#[test]
fn rail_width_animates_reverses_and_settles_without_idle_redraws() {
    let mut ui = ui(rail_shell(false), Size::new(700.0, 350.0));
    ui.at(0);
    ui.frame();
    let start = ui.find("Content").x;
    ui.rebuild(rail_shell(true));
    ui.at(0);
    ui.at(75);
    let middle = ui.find("Content").x;
    assert!(middle > start && middle < start + 200.0);
    ui.rebuild(rail_shell(false));
    ui.at(75);
    assert!(
        (ui.find("Content").x - middle).abs() < 0.1,
        "reversal starts at current width"
    );
    ui.at(500);
    assert!((ui.find("Content").x - start).abs() < 0.1);
    assert_eq!(ui.at(1000), iced::window::RedrawRequest::Wait);
}
fn modal_rail(open: bool) -> Element<'static, Message> {
    focus::scope(crate::modal_navigation_rail(
        widget::container(button("Background").on_press(Message::Action(20)))
            .align_right(Length::Fill)
            .height(Length::Fill),
        navigation_rail(items(), Some(1)).on_select(Message::Action),
        open,
        Message::Close,
    ))
}
#[test]
fn modal_rail_blocks_through_exit_and_restores_keyboard_invoker() {
    let mut ui = ui(modal_rail(false), Size::new(700.0, 400.0));
    ui.at(0);
    ui.frame();
    press(&mut ui, Named::Tab);
    ui.rebuild(modal_rail(true));
    ui.at(0);
    ui.at(500);
    press(&mut ui, Named::Tab);
    activate(&mut ui);
    assert_eq!(ui.messages, [Message::Action(1)]);
    let background = iced::Point::new(650.0, 20.0);
    ui.move_to(background);
    ui.down();
    ui.up();
    assert_eq!(ui.messages.last(), Some(&Message::Close));
    ui.messages.clear();
    ui.rebuild(modal_rail(false));
    ui.at(500);
    ui.down();
    ui.up();
    assert!(ui.messages.is_empty());
    ui.at(1000);
    activate(&mut ui);
    assert_eq!(ui.messages, [Message::Action(20)]);
}
#[test]
fn docked_range_picker_keeps_first_endpoint_open() {
    let month = crate::Date::new(2026, 9, 1).unwrap();
    let start = crate::Date::new(2026, 9, 9).unwrap();
    let initial = crate::DateSelection::Range {
        start: None,
        end: None,
    };
    let mut ui = ui(
        focus::scope(date_popup(month, initial)),
        Size::new(500.0, 640.0),
    );
    ui.click("Choose date");
    ui.click("9");
    ui.rebuild(focus::scope(date_popup(
        month,
        crate::DateSelection::Range {
            start: Some(start),
            end: None,
        },
    )));
    ui.at(0);
    assert!(visible(&mut ui, "September 2026").is_some());
    ui.click("16");
    ui.at(1);
    assert!(visible(&mut ui, "September 2026").is_none());
    assert_eq!(
        ui.messages.last(),
        Some(&Message::Date(crate::DateSelection::Range {
            start: Some(start),
            end: Some(crate::Date::new(2026, 9, 16).unwrap())
        }))
    );
}

#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_desktop_variants_and_motion() {
    use super::{reference, themed_name};
    for dark in [false, true] {
        let mut ui = Harness::new(
            focus::scope(cascade()),
            Size::new(760.0, 400.0),
            theme(dark),
        );
        ui.at(0);
        let case = themed_name("desktop-cascade", dark);
        open_cascade(&mut ui);
        ui.at(500);
        reference::check(&format!("{case}/01-keyboard-submenu"), &ui.frame());
        press(&mut ui, Named::ArrowDown);
        press(&mut ui, Named::ArrowRight);
        ui.at(1000);
        reference::check(&format!("{case}/02-three-levels"), &ui.frame());
        press(&mut ui, Named::Space);
        for t in [0, 75, 150] {
            ui.at(1000 + t);
            reference::check(&format!("{case}/03-held-{t:03}ms"), &ui.frame());
        }
        ui.event(key(Named::Space, true, false));
        ui.at(1600);
        reference::check(&format!("{case}/04-closed"), &ui.frame());
        let mut ui = Harness::new(rail_shell(false), Size::new(600.0, 340.0), theme(dark));
        ui.at(0);
        ui.frame();
        let case = themed_name("desktop-rail-width", dark);
        ui.rebuild(rail_shell(true));
        for t in [0, 50, 100, 200, 300] {
            ui.at(t);
            reference::check(&format!("{case}/01-expand-{t:03}ms"), &ui.frame());
        }
        ui.rebuild(rail_shell(false));
        for t in [0, 50, 100, 200, 300] {
            ui.at(300 + t);
            reference::check(&format!("{case}/02-collapse-{t:03}ms"), &ui.frame());
        }
        let mut ui = Harness::new(modal_rail(false), Size::new(600.0, 400.0), theme(dark));
        ui.at(0);
        ui.frame();
        ui.rebuild(modal_rail(true));
        let case = themed_name("desktop-modal-rail", dark);
        for t in [0, 75, 150, 300, 500] {
            ui.at(t);
            reference::check(&format!("{case}/01-open-{t:03}ms"), &ui.frame());
        }
        ui.rebuild(modal_rail(false));
        for t in [0, 75, 150, 250, 400] {
            ui.at(500 + t);
            reference::check(&format!("{case}/02-close-{t:03}ms"), &ui.frame());
        }
        for width in [320.0, 840.0] {
            let dialog = crate::full_screen_dialog(
                "Edit workspace",
                widget::column![
                    typography("Workspace details", TypeScale::HeadlineSmall),
                    crate::text_field("Name", "Design systems").on_input(Message::Input),
                    typography(
                        "Keep your team informed about the next review.",
                        TypeScale::BodyLarge
                    ),
                    widget::space().height(800),
                    button("Last action").on_press(Message::Action(2))
                ]
                .spacing(24),
            )
            .on_dismiss(Message::Close)
            .action(
                button("Save")
                    .variant(crate::ButtonVariant::Text)
                    .on_press(Message::Action(1)),
            )
            .into();
            let mut ui = Harness::new(
                focus::scope(crate::dialog::modal(button("Background"), dialog, true)),
                Size::new(width, 500.0),
                theme(dark),
            );
            ui.at(0);
            let case = themed_name(&format!("desktop-full-screen-{}", width as u32), dark);
            reference::check(&format!("{case}/00-open"), &ui.frame());
            for _ in 0..3 {
                press(&mut ui, Named::Tab);
            }
            reference::check(&format!("{case}/01-last-focused"), &ui.frame());
        }
        for bottom in [false, true] {
            let picker = date_popup(
                crate::Date::new(2026, 9, 1).unwrap(),
                crate::DateSelection::Single(Some(crate::Date::new(2026, 9, 9).unwrap())),
            );
            let content = widget::container(picker)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_y(if bottom {
                    iced::alignment::Vertical::Bottom
                } else {
                    iced::alignment::Vertical::Top
                })
                .padding(16);
            let mut ui = Harness::new(focus::scope(content), Size::new(390.0, 620.0), theme(dark));
            ui.click("Choose date");
            ui.at(500);
            reference::check(
                &format!(
                    "{}/00-open",
                    themed_name(
                        if bottom {
                            "desktop-calendar-above"
                        } else {
                            "desktop-calendar-below"
                        },
                        dark
                    )
                ),
                &ui.frame(),
            );
        }
        let mut ui = Harness::new(
            focus::scope(widget::container(cascade()).align_right(Length::Fill)),
            Size::new(560.0, 350.0),
            theme(dark),
        );
        ui.click("Actions");
        let row = ui.find("Export");
        ui.move_to(row.center());
        for t in [0, 100, 200, 350, 700] {
            ui.at(t);
            reference::check(
                &format!(
                    "{}/hover-{t:03}ms",
                    themed_name("desktop-cascade-flipped", dark)
                ),
                &ui.frame(),
            );
        }
    }
}

#[test]
fn selected_tab_is_revealed_without_stealing_focus_from_other_controls() {
    let tabs = crate::tabs(
        (0..10).map(|i| crate::Tab::new(i, format!("Section {i}"))),
        Some(9),
    )
    .on_select(Message::Action)
    .scrollable(true);
    let mut ui = ui(
        focus::scope(widget::column![
            button("Before").on_press(Message::Action(20)),
            tabs
        ]),
        Size::new(300.0, 180.0),
    );
    assert!(visible(&mut ui, "Section 9").is_some());
    press(&mut ui, Named::Tab);
    activate(&mut ui);
    assert_eq!(ui.messages, [Message::Action(20)]);
}
#[test]
fn calendar_home_and_end_follow_the_current_week() {
    let date = crate::Date::new(2026, 9, 9).unwrap();
    let picker =
        crate::date_picker(date, crate::DateSelection::Single(Some(date))).on_select(|s| match s {
            crate::DateSelection::Single(Some(d)) => Message::Action(d.day()),
            _ => unreachable!(),
        });
    let mut ui = ui(focus::scope(picker), Size::new(400.0, 520.0));
    press(&mut ui, Named::Tab);
    press(&mut ui, Named::Home);
    activate(&mut ui);
    press(&mut ui, Named::End);
    activate(&mut ui);
    assert_eq!(ui.messages, [Message::Action(6), Message::Action(12)]);
}
#[test]
fn reduced_motion_rail_reaches_its_final_width_immediately() {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    let mut ui = Harness::with_backend(
        rail_shell(false),
        Size::new(600.0, 350.0),
        theme(false).reduced_motion(true),
        &backend,
    );
    ui.at(0);
    ui.frame();
    ui.rebuild(rail_shell(true));
    ui.at(0);
    let expanded = ui.find("Content").x;
    let mut reference = Harness::new(rail_shell(true), Size::new(600.0, 350.0), theme(false));
    assert_eq!(expanded, reference.find("Content").x);
    assert_eq!(ui.at(1000), iced::window::RedrawRequest::Wait);
}

#[test]
fn roving_group_preserves_native_targeted_focus_operations() {
    use iced::advanced::widget::{
        Operation,
        operation::{self, Outcome, focusable},
    };
    let mut ui = ui(
        focus::scope(focus::group(widget::row![
            crate::text_field("First", "")
                .id("first-field")
                .on_input(Message::Input),
            crate::text_field("Second", "")
                .id("second-field")
                .on_input(Message::Input)
        ])),
        Size::new(500.0, 120.0),
    );
    ui.operate(&mut focusable::focus::<()>(widget::Id::new("second-field")));
    let mut query = focusable::find_focused();
    ui.operate(&mut operation::black_box(&mut query));
    assert!(matches!(query.finish(),Outcome::Some(id) if id==widget::Id::new("second-field")));
}

#[test]
fn returning_to_parent_cancels_a_submenu_outside_press() {
    let mut ui = ui(focus::scope(cascade()), Size::new(760.0, 400.0));
    open_cascade(&mut ui);
    let parent = ui.find("Export").center();
    ui.move_to((730.0, 380.0));
    ui.down();
    ui.move_to(parent);
    ui.up();
    ui.move_to((730.0, 380.0));
    ui.up();
    ui.at(1);
    assert!(visible(&mut ui, "Export").is_some());
    assert!(visible(&mut ui, "PDF").is_some());
    assert!(ui.messages.is_empty());
}
