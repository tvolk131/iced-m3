use super::{harness::Harness, reference, theme, themed_name};
use crate::{
    Element, Theme, TypeScale, assist_chip, button, filter_chip, input_chip, list, list_item,
    range_slider, search_bar, suggestion_chip, typography,
};
use iced::{Event, Length, Size, keyboard, widget, window};

#[derive(Clone, Debug, PartialEq)]
enum Message {
    Open,
    Close,
    Input(String),
    Submit,
    Result,
    Background,
    Chip,
    Remove,
    Range((f32, f32)),
    Release,
}
fn make_ui<'a>(
    element: impl Into<Element<'a, Message>>,
    size: Size,
    theme: Theme,
) -> Harness<'a, Message> {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    Harness::with_backend(element, size, theme, &backend)
}
fn key(key: keyboard::Key, text: Option<&str>) -> Event {
    Event::Keyboard(keyboard::Event::KeyPressed {
        modified_key: key.clone(),
        key,
        physical_key: keyboard::key::Physical::Unidentified(
            keyboard::key::NativeCode::Unidentified,
        ),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::empty(),
        text: text.map(Into::into),
        repeat: false,
    })
}
fn escape() -> Event {
    key(keyboard::Key::Named(keyboard::key::Named::Escape), None)
}
fn unfocus(ui: &mut Harness<'_, Message>) {
    ui.operate(&mut iced::advanced::widget::operation::focusable::unfocus::<()>());
}
fn fixture(open: bool, value: &str, background: bool) -> Element<'static, Message> {
    widget::container(
        widget::column![
            search_bar("Search files", value)
                .open(open)
                .on_open(Message::Open)
                .on_close(Message::Close)
                .on_input(Message::Input)
                .on_submit(Message::Submit)
                .results(
                    widget::column![
                        typography("Recent searches", TypeScale::LabelLarge),
                        suggestion_chip("Design").on_press(Message::Input("Design".into())),
                        list([list_item("Design notes")
                            .supporting_text("Working notes and decisions")
                            .on_press(Message::Result)
                            .into()])
                        .style(|_| widget::container::Style::default()),
                    ]
                    .spacing(16)
                )
                .width(420),
            typography(
                if background {
                    "BACKGROUND TEXT THAT MUST BE COVERED BY THE SEARCH VIEW"
                } else {
                    ""
                },
                TypeScale::BodyLarge
            ),
            button("Background").on_press(Message::Background),
        ]
        .spacing(24),
    )
    .padding(16)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
fn search_ui(open: bool) -> Harness<'static, Message> {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    let mut ui = Harness::with_backend(
        fixture(open, "", true),
        Size::new(720.0, 560.0),
        Theme::light(),
        &backend,
    );
    ui.at(0);
    ui.frame();
    ui
}
#[test]
fn search_open_focus_query_clear_submit_and_result_are_controlled() {
    let mut ui = search_ui(false);
    ui.click("Search files");
    assert_eq!(ui.messages, [Message::Open]);
    ui.messages.clear();
    ui.rebuild(fixture(true, "", true));
    ui.at(0);
    ui.at(300);
    ui.event(key(keyboard::Key::Character("d".into()), Some("d")));
    assert_eq!(ui.messages, [Message::Input("d".into())]);
    ui.messages.clear();
    ui.rebuild(fixture(true, "d", true));
    ui.at(300);
    ui.event(key(keyboard::Key::Named(keyboard::key::Named::Enter), None));
    assert_eq!(ui.messages, [Message::Submit]);
    ui.messages.clear();
    ui.move_to((408.0, 44.0));
    ui.down();
    ui.up();
    assert_eq!(ui.messages, [Message::Input(String::new())]);
    ui.messages.clear();
    ui.click("Design notes");
    assert_eq!(ui.messages, [Message::Result]);
}
#[test]
fn search_escape_and_outside_click_do_not_activate_background() {
    let mut ui = search_ui(true);
    ui.move_to((650.0, 500.0));
    ui.down();
    ui.up();
    assert_eq!(ui.messages, [Message::Close]);
    ui.messages.clear();
    ui.event(escape());
    assert_eq!(ui.messages, [Message::Close]);
    ui.messages.clear();
    ui.rebuild(fixture(false, "", true));
    ui.at(0);
    ui.move_to((80.0, 160.0));
    ui.down();
    ui.up();
    assert!(ui.messages.is_empty());
    ui.at(250);
    ui.click("Background");
    assert_eq!(ui.messages, [Message::Background]);
}
#[test]
fn search_reopen_retargets_continuously_and_finite_motion_stops() {
    let mut ui = search_ui(false);
    ui.rebuild(fixture(true, "", true));
    ui.at(0);
    unfocus(&mut ui);
    ui.at(75);
    let before = ui.frame();
    ui.rebuild(fixture(false, "", true));
    ui.at(75);
    assert!(before == ui.frame());
    ui.at(150);
    let before = ui.frame();
    ui.rebuild(fixture(true, "", true));
    ui.at(150);
    unfocus(&mut ui);
    assert!(before == ui.frame());
    ui.at(450);
    assert_eq!(ui.at(1000), window::RedrawRequest::Wait);
}
#[test]
fn search_surface_occludes_background_on_both_renderers() {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    for dark in [false, true] {
        let frame = |background| {
            let mut ui = Harness::with_backend(
                fixture(true, "", background),
                Size::new(720.0, 560.0),
                theme(dark),
                &backend,
            );
            ui.at(0);
            unfocus(&mut ui);
            ui.frame().crop(40, 170, 800, 90)
        };
        assert!(frame(true) == frame(false));
    }
}
#[test]
fn search_full_window_results_remain_reachable_in_narrow_view() {
    let mut ui = make_ui(
        fixture(true, "", true),
        Size::new(280.0, 400.0),
        Theme::light(),
    );
    ui.at(0);
    ui.click("Design notes");
    assert_eq!(ui.messages, [Message::Result]);
    ui.move_to((28.0, 28.0));
    ui.down();
    ui.up();
    assert_eq!(ui.messages.last(), Some(&Message::Close));
}
#[test]
fn search_nested_menu_escape_and_inside_to_outside_drag_do_not_dismiss() {
    let mut ui = make_ui(
        search_bar("Search", "")
            .open(true)
            .on_close(Message::Close)
            .results(crate::menu(
                "Options",
                [crate::MenuItem::new("Result action", Message::Result)],
            )),
        Size::new(720.0, 560.0),
        Theme::light(),
    );
    ui.at(0);
    ui.click("Options");
    ui.event(escape());
    assert!(ui.messages.is_empty());
    ui.move_to((300.0, 200.0));
    ui.down();
    ui.move_to((710.0, 550.0));
    ui.up();
    assert!(ui.messages.is_empty());
    ui.event(escape());
    assert_eq!(ui.messages, [Message::Close]);
}
#[test]
fn search_scrolls_long_results_and_reopens_with_query_preserved() {
    let make = |open| {
        widget::container(
            search_bar("Search", "Saved query")
                .open(open)
                .on_close(Message::Close)
                .on_input(Message::Input)
                .results(widget::Column::with_children((0..30).map(|i| {
                    list_item(format!("Result {i}"))
                        .on_press(Message::Result)
                        .into()
                }))),
        )
        .padding(16)
        .align_bottom(Length::Fill)
    };
    let mut ui = make_ui(make(true), Size::new(640.0, 480.0), Theme::light());
    ui.at(0);
    ui.move_to((200.0, 300.0));
    ui.event(Event::Mouse(iced::mouse::Event::WheelScrolled {
        delta: iced::mouse::ScrollDelta::Lines { x: 0.0, y: -100.0 },
    }));
    ui.click("Result 29");
    assert_eq!(ui.messages, [Message::Result]);
    ui.messages.clear();
    ui.rebuild(make(false));
    ui.at(0);
    ui.at(250);
    ui.rebuild(make(true));
    ui.at(250);
    ui.at(550);
    ui.click("Result 29");
    assert_eq!(ui.messages, [Message::Result]);
}
#[test]
fn zero_duration_search_motion_settles_immediately() {
    let mut theme = Theme::light();
    theme.motion.search_enter = std::time::Duration::ZERO;
    theme.motion.search_exit = std::time::Duration::ZERO;
    let mut ui = make_ui(fixture(false, "", true), Size::new(720.0, 560.0), theme);
    ui.at(0);
    ui.frame();
    ui.rebuild(fixture(true, "", true));
    ui.at(0);
    unfocus(&mut ui);
    assert_eq!(ui.at(1), window::RedrawRequest::Wait);
    ui.rebuild(fixture(false, "", true));
    ui.at(1);
    assert_eq!(ui.at(2), window::RedrawRequest::Wait);
}
fn chips(selected: bool, disabled: bool) -> Element<'static, Message> {
    widget::container(
        widget::column![
            assist_chip("Add to calendar")
                .on_press(Message::Chip)
                .disabled(disabled),
            suggestion_chip("Design")
                .on_press(Message::Chip)
                .disabled(disabled),
            filter_chip("Shared", selected)
                .on_press(Message::Chip)
                .disabled(disabled),
            input_chip("Team: Studio")
                .selected(selected)
                .on_press(Message::Chip)
                .on_remove(Message::Remove)
                .disabled(disabled),
        ]
        .spacing(16),
    )
    .padding(16)
    .into()
}
#[test]
fn input_chip_remove_and_body_are_independent_and_disabling_cancels() {
    let mut ui = make_ui(chips(false, false), Size::new(320.0, 240.0), Theme::light());
    ui.at(0);
    ui.click("Team: Studio");
    assert_eq!(ui.messages, [Message::Chip]);
    ui.messages.clear();
    let label = ui.find("Team: Studio");
    ui.move_to((label.x + label.width + 24.0, label.center_y()));
    ui.down();
    ui.up();
    assert_eq!(ui.messages, [Message::Remove]);
    ui.messages.clear();
    ui.down();
    ui.rebuild(chips(false, true));
    ui.up();
    assert!(ui.messages.is_empty());
    ui.click("Shared");
    assert!(ui.messages.is_empty());
}
#[test]
fn removal_only_chip_works_without_body_action() {
    let mut ui = make_ui(
        input_chip("Query").on_remove(Message::Remove),
        Size::new(240.0, 50.0),
        Theme::light(),
    );
    ui.at(0);
    let label = ui.find("Query");
    ui.click("Query");
    assert!(ui.messages.is_empty());
    ui.move_to((label.x + label.width + 24.0, 16.0));
    ui.down();
    ui.up();
    assert_eq!(ui.messages, [Message::Remove]);
}
fn range(values: (f32, f32), enabled: bool, step: f32) -> Element<'static, Message> {
    widget::container(
        range_slider(0.0..=100.0, values)
            .step(step)
            .ticks(true)
            .labeled(true)
            .on_change(Message::Range)
            .on_release(Message::Release)
            .disabled(!enabled),
    )
    .padding(16)
    .into()
}
#[test]
fn range_slider_moves_nearest_handle_and_clamps_at_its_neighbor() {
    let mut ui = make_ui(
        range((20.0, 80.0), true, 10.0),
        Size::new(280.0, 110.0),
        Theme::light(),
    );
    ui.at(0);
    // Track is x=40..240 and y=72, including reserved label space.
    ui.move_to((60.0, 68.0));
    ui.down();
    assert_eq!(ui.messages, [Message::Range((10.0, 80.0))]);
    ui.messages.clear();
    ui.rebuild(range((10.0, 80.0), true, 10.0));
    ui.move_to((260.0, 68.0));
    ui.up();
    assert_eq!(
        ui.messages,
        [Message::Range((80.0, 80.0)), Message::Release]
    );
}
#[test]
fn coincident_range_handles_can_separate_in_both_directions() {
    for (x, expected) in [(80.0, (20.0, 50.0)), (220.0, (50.0, 90.0))] {
        let mut ui = make_ui(
            range((50.0, 50.0), true, 10.0),
            Size::new(280.0, 110.0),
            Theme::light(),
        );
        ui.at(0);
        ui.move_to((140.0, 68.0));
        ui.down();
        ui.move_to((x, 68.0));
        ui.up();
        assert_eq!(ui.messages, [Message::Range(expected), Message::Release]);
    }
}
#[test]
fn range_slider_cancels_on_disable_focus_loss_and_window_exit() {
    for cancel in [
        Event::Window(window::Event::Unfocused),
        Event::Mouse(iced::mouse::Event::CursorLeft),
    ] {
        let mut ui = make_ui(
            range((20.0, 80.0), true, 10.0),
            Size::new(280.0, 110.0),
            Theme::light(),
        );
        ui.at(0);
        ui.move_to((80.0, 68.0));
        ui.down();
        ui.event(cancel);
        ui.move_to((180.0, 68.0));
        ui.up();
        assert!(ui.messages.is_empty());
    }
    let mut ui = make_ui(
        range((20.0, 80.0), true, 10.0),
        Size::new(280.0, 110.0),
        Theme::light(),
    );
    ui.at(0);
    ui.move_to((80.0, 68.0));
    ui.down();
    ui.rebuild(range((20.0, 80.0), false, 10.0));
    ui.move_to((180.0, 68.0));
    ui.up();
    assert!(ui.messages.is_empty());
}
#[test]
fn invalid_range_is_inert_and_extreme_domains_stay_finite() {
    for domain in [1.0..=1.0, 2.0..=1.0, f32::NAN..=100.0] {
        let mut ui = make_ui(
            range_slider(domain, (10.0, 90.0)).on_change(Message::Range),
            Size::new(200.0, 48.0),
            Theme::light(),
        );
        ui.at(0);
        ui.move_to((100.0, 24.0));
        ui.down();
        ui.up();
        ui.frame();
        assert!(ui.messages.is_empty());
    }
    let mut ui = make_ui(
        range_slider(-f32::MAX..=f32::MAX, (f32::NAN, f32::INFINITY)).on_change(Message::Range),
        Size::new(200.0, 48.0),
        Theme::light(),
    );
    ui.at(0);
    ui.move_to((100.0, 24.0));
    ui.down();
    ui.up();
    ui.frame();
    assert!(
        matches!(ui.messages.as_slice(),[Message::Range((a,b))] if a.is_finite() && b.is_finite() && a <= b)
    );
}
fn capture(ui: &mut Harness<'_, Message>, case: &str, frame: &str) {
    reference::check(&format!("{case}/{frame}"), &ui.frame());
}
#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_search() {
    for dark in [false, true] {
        for (name, size) in [
            ("docked", Size::new(720.0, 560.0)),
            ("full", Size::new(320.0, 480.0)),
        ] {
            let case = themed_name(&format!("search-{name}"), dark);
            let mut ui = make_ui(fixture(false, "", true), size, theme(dark));
            ui.at(0);
            capture(&mut ui, &case, "00-closed");
            ui.rebuild(fixture(true, "", true));
            ui.at(0);
            unfocus(&mut ui);
            for time in [0, 50, 100, 200, 300] {
                ui.at(time);
                capture(&mut ui, &case, &format!("01-expand-{time:03}ms"));
            }
            ui.rebuild(fixture(true, "Design", true));
            ui.at(300);
            capture(&mut ui, &case, "02-query");
            ui.rebuild(fixture(false, "Design", true));
            for time in [300, 350, 425, 500, 550] {
                ui.at(time);
                capture(&mut ui, &case, &format!("03-collapse-{:03}ms", time - 300));
            }
            assert_eq!(ui.at(1000), window::RedrawRequest::Wait);
        }
    }
}
#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_chip_variants() {
    for dark in [false, true] {
        let case = themed_name("chip-variants", dark);
        let mut ui = make_ui(chips(false, false), Size::new(320.0, 240.0), theme(dark));
        ui.at(0);
        capture(&mut ui, &case, "00-default");
        let point = ui.find("Shared").center();
        ui.move_to(point);
        for time in [0, 75, 150] {
            ui.at(time);
            capture(&mut ui, &case, &format!("01-hover-{time:03}ms"));
        }
        ui.down();
        ui.at(200);
        capture(&mut ui, &case, "02-held");
        ui.up();
        ui.rebuild(chips(true, false));
        for time in [200, 250, 300, 400] {
            ui.at(time);
            capture(&mut ui, &case, &format!("03-selected-{:03}ms", time - 200));
        }
        let label = ui.find("Team: Studio");
        ui.move_to((label.x + label.width + 24.0, label.center_y()));
        ui.down();
        ui.at(450);
        capture(&mut ui, &case, "04-remove-held");
        ui.rebuild(chips(true, true));
        ui.at(500);
        capture(&mut ui, &case, "05-disabled");
    }
}
#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_range_slider() {
    for dark in [false, true] {
        for (label, values) in [
            ("separated", (20.0, 80.0)),
            ("coincident", (50.0, 50.0)),
            ("endpoints", (0.0, 100.0)),
        ] {
            let case = themed_name(&format!("range-slider-{label}"), dark);
            let mut ui = make_ui(
                range(values, true, 10.0),
                Size::new(280.0, 110.0),
                theme(dark),
            );
            ui.at(0);
            capture(&mut ui, &case, "00-default");
            ui.move_to((40.0 + values.0 * 2.0, 68.0));
            ui.at(150);
            capture(&mut ui, &case, "01-hover");
            ui.down();
            for time in [150, 200, 250, 300] {
                ui.at(time);
                capture(&mut ui, &case, &format!("02-held-{:03}ms", time - 150));
            }
            ui.move_to((60.0, 68.0));
            ui.rebuild(range((10.0, values.1), true, 10.0));
            ui.at(350);
            capture(&mut ui, &case, "03-drag");
            ui.up();
            for time in [350, 425, 500] {
                ui.at(time);
                capture(&mut ui, &case, &format!("04-release-{:03}ms", time - 350));
            }
            ui.rebuild(range(values, false, 10.0));
            ui.at(600);
            capture(&mut ui, &case, "05-disabled");
            ui.leave();
            assert_eq!(ui.at(1000), window::RedrawRequest::Wait);
        }
    }
}
