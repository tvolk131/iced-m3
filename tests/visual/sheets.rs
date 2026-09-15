use super::{harness::Harness, reference, theme, themed_name};
use crate::{
    Element, FabColor, FabSize, MenuItem, Theme, TypeScale, bottom_sheet, button, extended_fab,
    fab, menu, sheet, side_sheet, text_field, typography,
};
use iced::{Event, Length, Size, keyboard, mouse, widget, window};

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Background,
    Action,
    Dismiss,
    Input(String),
    BackgroundInput(String),
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
fn content(long: bool) -> Element<'static, Message> {
    let mut body = widget::column![
        typography("File details", TypeScale::HeadlineSmall),
        typography("Design notes", TypeScale::TitleLarge),
        button("Sheet action").on_press(Message::Action),
        text_field("Sheet input", "").on_input(Message::Input),
        menu(
            "Sheet menu",
            [MenuItem::new("Nested action", Message::Action)]
        ),
    ]
    .spacing(16);
    if long {
        for i in 0..18 {
            body = body.push(typography(
                format!("Revision {i}: a long line that wraps within a narrow sheet."),
                TypeScale::BodyMedium,
            ));
        }
    }
    body.push(button("Close sheet").on_press(Message::Dismiss))
        .into()
}
fn fixture(kind: &str, open: bool, long: bool, dismiss: bool) -> Element<'static, Message> {
    let background = widget::container(
        widget::column![
            button("Background").on_press(Message::Background),
            text_field("Background input", "").on_input(Message::BackgroundInput),
            typography(
                "Workspace content remains available beside a standard sheet.",
                TypeScale::BodyLarge
            ),
        ]
        .spacing(24),
    )
    .padding(16)
    .width(Length::Fill)
    .height(Length::Fill);
    let panel = match kind {
        "standard" => side_sheet(content(long)),
        "side" => side_sheet(content(long)).modal(),
        "bottom" => bottom_sheet(content(long)).height(420.0),
        _ => unreachable!(),
    }
    .open(open)
    .on_dismiss(Message::Dismiss)
    .dismiss_on_outside(dismiss)
    .dismiss_on_escape(dismiss);
    sheet::host(
        widget::mouse_area(background).on_scroll(|_| Message::Background),
        panel,
    )
}
fn ready(kind: &str, open: bool, long: bool, dismiss: bool) -> Harness<'static, Message> {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    let mut ui = Harness::with_backend(
        fixture(kind, open, long, dismiss),
        Size::new(720.0, 560.0),
        Theme::light(),
        &backend,
    );
    ui.frame();
    ui.at(0);
    ui.at(400);
    ui
}

#[test]
fn sheets_reverse_without_jumps_and_stop_redrawing_when_settled() {
    for kind in ["standard", "side", "bottom"] {
        let mut ui = ready(kind, false, false, true);
        let closed = ui.frame();
        ui.rebuild(fixture(kind, true, false, true));
        ui.at(400);
        assert!(closed == ui.frame(), "opening starts from closed geometry");
        ui.at(475);
        let partial = ui.frame();
        assert!(partial != closed);
        ui.rebuild(fixture(kind, false, false, true));
        ui.at(475);
        assert!(
            partial == ui.frame(),
            "closing starts where opening was interrupted"
        );
        ui.at(525);
        let closing = ui.frame();
        assert!(closing != partial);
        ui.rebuild(fixture(kind, true, false, true));
        ui.at(525);
        assert!(closing == ui.frame(), "reopening does not jump");
        ui.at(900);
        let opened = ui.frame();
        assert_eq!(ui.at(2000), window::RedrawRequest::Wait);
        assert!(opened == ui.frame());
        ui.rebuild(fixture(kind, false, false, true));
        ui.at(2000);
        ui.at(2300);
        assert!(closed == ui.frame());
        assert_eq!(ui.at(4000), window::RedrawRequest::Wait);
    }
}

#[test]
fn modal_sheets_block_background_through_exit_and_cancel_covered_presses() {
    for kind in ["side", "bottom"] {
        let mut ui = ready(kind, false, false, true);
        let target = ui.find("Background").center();
        ui.move_to(target);
        ui.down();
        ui.rebuild(fixture(kind, true, false, false));
        ui.at(400);
        ui.at(800);
        ui.up();
        assert!(ui.messages.is_empty());
        ui.move_to(target);
        ui.down();
        ui.up();
        ui.event(escape());
        ui.event(Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: -3.0 },
        }));
        assert!(
            ui.messages.is_empty(),
            "disabling dismissal must not enable click-through"
        );
        ui.click("Sheet action");
        assert_eq!(ui.messages, [Message::Action]);
        ui.messages.clear();
        ui.rebuild(fixture(kind, false, false, true));
        ui.at(800);
        ui.at(850);
        ui.move_to(target);
        ui.down();
        ui.up();
        assert!(ui.messages.is_empty(), "exit animation remains modal");
        ui.at(1100);
        ui.click("Background");
        assert_eq!(ui.messages, [Message::Background]);
    }
}

#[test]
fn sheet_dismissal_requires_matching_outside_press_and_nested_escape_has_priority() {
    for kind in ["side", "bottom"] {
        let mut ui = ready(kind, true, false, true);
        ui.move_to((8.0, 8.0));
        ui.up();
        assert!(ui.messages.is_empty());
        let inside = ui.find("File details").center();
        ui.move_to(inside);
        ui.down();
        ui.move_to((8.0, 8.0));
        ui.up();
        assert!(ui.messages.is_empty());
        ui.down();
        ui.event(Event::Window(window::Event::Unfocused));
        ui.up();
        assert!(ui.messages.is_empty());
        ui.event(Event::Window(window::Event::Focused));
        ui.down();
        ui.up();
        assert_eq!(ui.messages, [Message::Dismiss]);
        ui.messages.clear();
        ui.click("Sheet menu");
        ui.event(escape());
        assert!(ui.messages.is_empty());
        ui.event(escape());
        assert_eq!(ui.messages, [Message::Dismiss]);
    }
}

#[test]
fn standard_sheet_keeps_background_actions_live_and_closed_content_cannot_activate() {
    let mut ui = ready("standard", true, false, true);
    ui.click("Background");
    ui.click("Sheet action");
    assert_eq!(ui.messages, [Message::Background, Message::Action]);
    ui.messages.clear();
    let target = ui.find("Sheet action").center();
    ui.move_to(target);
    ui.down();
    ui.rebuild(fixture("standard", false, false, true));
    ui.at(400);
    ui.up();
    assert!(ui.messages.is_empty());
    ui.at(700);
    ui.rebuild(fixture("standard", true, false, true));
    ui.at(700);
    ui.at(1100);
    ui.up();
    assert!(ui.messages.is_empty());
    ui.click("Sheet menu");
    ui.click("Nested action");
    assert_eq!(ui.messages, [Message::Action]);
}

#[test]
fn modal_sheet_preserves_native_typing_and_blocks_the_background_field() {
    for kind in ["side", "bottom"] {
        let mut ui = ready(kind, false, false, true);
        ui.move_to((80.0, 112.0));
        ui.down();
        ui.up();
        ui.rebuild(fixture(kind, true, false, true));
        ui.at(400);
        ui.at(800);
        ui.event(key(keyboard::Key::Character("a".into()), Some("a")));
        assert!(ui.messages.is_empty());
        let action = ui.find("Sheet action");
        ui.move_to((action.center_x(), action.y + action.height + 44.0));
        ui.down();
        ui.up();
        ui.event(key(keyboard::Key::Character("é".into()), Some("é")));
        assert_eq!(ui.messages, [Message::Input("é".into())]);
    }
}

#[test]
fn long_sheets_scroll_to_actions_and_retain_scroll_position_after_reopening() {
    for kind in ["standard", "side", "bottom"] {
        let mut ui = ready(kind, true, true, true);
        ui.move_to((600.0, 400.0));
        ui.event(Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: -100.0 },
        }));
        ui.click("Close sheet");
        assert_eq!(ui.messages, [Message::Dismiss]);
        ui.rebuild(fixture(kind, false, true, true));
        ui.at(400);
        ui.at(700);
        ui.rebuild(fixture(kind, true, true, true));
        ui.at(700);
        ui.at(1100);
        ui.click("Close sheet");
        assert_eq!(ui.messages.len(), 2);
    }
}

#[test]
fn sheet_motion_honors_zero_duration_and_window_deactivation() {
    let mut theme = Theme::light();
    theme.motion.sheet_enter = std::time::Duration::ZERO;
    theme.motion.sheet_exit = std::time::Duration::ZERO;
    let mut ui = Harness::new(
        fixture("bottom", true, false, true),
        Size::new(720.0, 560.0),
        theme,
    );
    ui.frame();
    ui.at(0);
    let frame = ui.frame();
    ui.at(1);
    assert!(frame == ui.frame());
    ui.click("Sheet action");
    assert_eq!(ui.messages, [Message::Action]);
    let mut ui = ready("side", false, false, true);
    ui.rebuild(fixture("side", true, false, true));
    ui.at(400);
    ui.at(450);
    ui.event(Event::Window(window::Event::Unfocused));
    assert_eq!(ui.at(600), window::RedrawRequest::Wait);
    ui.event(Event::Window(window::Event::Focused));
    ui.at(900);
    ui.click("Sheet action");
    assert_eq!(ui.messages, [Message::Action]);
}

#[test]
fn switching_between_standard_and_modal_sheets_updates_input_isolation() {
    let mut ui = ready("standard", true, false, true);
    let background = ui.find("Background").center();
    ui.rebuild(fixture("bottom", true, false, false));
    ui.at(400);
    ui.move_to(background);
    ui.down();
    ui.up();
    assert!(ui.messages.is_empty());
    ui.click("Sheet action");
    assert_eq!(ui.messages, [Message::Action]);
    ui.messages.clear();
    ui.rebuild(fixture("standard", true, false, true));
    ui.at(500);
    ui.click("Background");
    ui.click("Sheet action");
    assert_eq!(ui.messages, [Message::Background, Message::Action]);
}

#[test]
fn invalid_sheet_dimensions_use_defaults_and_tiny_hosts_remain_finite() {
    let render = |invalid, size| {
        let mut panel = bottom_sheet(button("Action").on_press(Message::Action)).open(true);
        if invalid {
            panel = panel.width(f32::NAN).height(f32::INFINITY);
        }
        let mut ui = Harness::new(sheet::host(widget::space(), panel), size, Theme::light());
        ui.frame();
        ui.at(0);
        ui.at(400);
        ui.frame()
    };
    for size in [Size::new(320.0, 480.0), Size::new(48.0, 48.0)] {
        assert!(render(true, size) == render(false, size));
    }
}

#[test]
fn modal_sheet_suspends_and_resumes_background_snackbar_timer() {
    let scene = |open: bool| {
        sheet::host(
            crate::snackbar::host(
                widget::space().width(Length::Fill).height(Length::Fill),
                Some(crate::snackbar("Saved").on_dismiss(Message::Background)),
            ),
            bottom_sheet(content(false))
                .open(open)
                .on_dismiss(Message::Dismiss),
        )
    };
    let mut ui = Harness::new(scene(false), Size::new(720.0, 560.0), Theme::light());
    ui.frame();
    ui.at(0);
    ui.at(1000);
    ui.rebuild(scene(true));
    ui.at(1000);
    ui.at(1400);
    ui.at(10000);
    assert!(
        ui.messages.is_empty(),
        "modal time must not consume the snackbar timeout"
    );
    ui.rebuild(scene(false));
    ui.at(10000);
    ui.at(10300);
    ui.at(13299);
    assert!(ui.messages.is_empty());
    ui.at(13301);
    assert_eq!(ui.messages, [Message::Background]);
}

#[test]
fn a_root_dialog_cancels_sheet_gestures_and_hides_nested_sheet_menus() {
    let scene = |open: bool| {
        crate::dialog::modal(
            fixture("side", true, false, true),
            open.then(|| {
                crate::dialog::dialog(button("Dialog action").on_press(Message::Background))
                    .on_dismiss(Message::Dismiss)
            }),
        )
    };
    let mut ui = Harness::new(scene(false), Size::new(720.0, 560.0), Theme::light());
    ui.frame();
    ui.at(0);
    ui.at(400);
    let point = ui.find("Sheet action").center();
    ui.move_to(point);
    ui.down();
    ui.rebuild(scene(true));
    ui.at(400);
    ui.up();
    assert!(ui.messages.is_empty());
    ui.click("Dialog action");
    assert_eq!(ui.messages, [Message::Background]);
    ui.messages.clear();
    ui.rebuild(scene(false));
    ui.at(800);
    ui.move_to(point);
    ui.up();
    assert!(ui.messages.is_empty());
    ui.click("Sheet menu");
    ui.rebuild(scene(true));
    ui.at(800);
    ui.event(escape());
    assert_eq!(ui.messages, [Message::Dismiss]);
}

#[test]
fn sheet_surfaces_occlude_background_text_on_both_themes() {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    for dark in [false, true] {
        for kind in ["side", "bottom"] {
            let render = |text| {
                let bg = widget::container(typography(text, TypeScale::HeadlineLarge))
                    .center_x(Length::Fill)
                    .center_y(Length::Fill);
                let content = widget::space().height(Length::Fill);
                let panel: crate::Sheet<'_, Message> = if kind == "side" {
                    side_sheet(content).modal()
                } else {
                    bottom_sheet(content)
                }
                .open(true);
                let mut ui = Harness::with_backend(
                    sheet::host(bg, panel),
                    Size::new(720.0, 560.0),
                    theme(dark),
                    &backend,
                );
                ui.frame();
                ui.at(0);
                ui.at(400);
                ui.frame().crop(760, 480, 520, 220)
            };
            assert!(
                render("BACKGROUND TEXT UNDER THE SHEET") == render(""),
                "opaque sheet occlusion: {backend}/{kind}/{dark}"
            );
        }
    }
}

fn fab_fixture(
    size: FabSize,
    color: FabColor,
    extended: bool,
    disabled: bool,
) -> Element<'static, Message> {
    let button = if extended {
        extended_fab(super::icon::Icon, "New workspace")
    } else if size == FabSize::Large {
        fab(super::icon::LargeIcon).size(size)
    } else {
        fab(super::icon::Icon).size(size)
    };
    widget::container(
        button
            .color(color)
            .on_press(Message::Action)
            .disabled(disabled),
    )
    .padding(24)
    .into()
}
#[test]
fn fab_held_release_cancel_and_disabled_interactions() {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    for size in [FabSize::Small, FabSize::Regular, FabSize::Large] {
        let mut ui = Harness::with_backend(
            fab_fixture(size, FabColor::Primary, false, false),
            Size::new(200.0, 160.0),
            Theme::light(),
            &backend,
        );
        ui.frame();
        ui.at(0);
        let idle = ui.frame();
        ui.move_to((44.0, 44.0));
        ui.at(150);
        let hover = ui.frame();
        assert!(hover != idle);
        ui.down();
        ui.at(650);
        let held = ui.frame();
        assert!(held != hover);
        assert!(ui.messages.is_empty());
        assert_eq!(ui.at(800), window::RedrawRequest::Wait);
        assert!(held == ui.frame());
        ui.up();
        assert_eq!(ui.messages, [Message::Action]);
        ui.messages.clear();
        ui.down();
        ui.move_to((190.0, 150.0));
        ui.up();
        assert!(ui.messages.is_empty());
        ui.move_to((44.0, 44.0));
        ui.down();
        ui.rebuild(fab_fixture(size, FabColor::Primary, false, true));
        ui.up();
        assert!(ui.messages.is_empty());
    }
}

#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_sheets() {
    for kind in ["standard", "side", "bottom"] {
        for dark in [false, true] {
            let case = themed_name(&format!("sheet-{kind}"), dark);
            let mut ui = Harness::new(
                fixture(kind, false, false, true),
                Size::new(720.0, 560.0),
                theme(dark),
            );
            ui.frame();
            ui.at(0);
            reference::check(&format!("{case}/00-closed"), &ui.frame());
            ui.rebuild(fixture(kind, true, false, true));
            for ms in [0, 50, 100, 150, 225, 300, 500] {
                ui.at(ms);
                reference::check(&format!("{case}/01-open-{ms:03}ms"), &ui.frame());
            }
            ui.rebuild(fixture(kind, false, false, true));
            for ms in [0, 50, 100, 150, 200] {
                ui.at(500 + ms);
                reference::check(&format!("{case}/02-close-{ms:03}ms"), &ui.frame());
            }
            ui.rebuild(fixture(kind, true, false, true));
            ui.at(700);
            ui.at(775);
            reference::check(&format!("{case}/03-interrupted-opening"), &ui.frame());
            ui.rebuild(fixture(kind, false, false, true));
            ui.at(775);
            ui.at(825);
            reference::check(&format!("{case}/04-reversed"), &ui.frame());
            for width in [320.0, 390.0] {
                let mut narrow = Harness::new(
                    fixture(kind, true, true, true),
                    Size::new(width, 480.0),
                    theme(dark),
                );
                narrow.frame();
                narrow.at(0);
                narrow.at(400);
                reference::check(&format!("{case}/05-narrow-{width}"), &narrow.frame());
            }
        }
    }
}

#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_fabs() {
    for dark in [false, true] {
        for (name, size, extended) in [
            ("small", FabSize::Small, false),
            ("regular", FabSize::Regular, false),
            ("large", FabSize::Large, false),
            ("extended", FabSize::Regular, true),
        ] {
            for (color_name, color) in [
                ("primary", FabColor::Primary),
                ("secondary", FabColor::Secondary),
                ("tertiary", FabColor::Tertiary),
                ("surface", FabColor::Surface),
            ] {
                let case = themed_name(&format!("fab-{name}-{color_name}"), dark);
                let mut ui = Harness::new(
                    fab_fixture(size, color, extended, false),
                    Size::new(240.0, 150.0),
                    theme(dark),
                );
                ui.frame();
                ui.at(0);
                reference::check(&format!("{case}/00-idle"), &ui.frame());
                ui.move_to((44.0, 44.0));
                ui.at(75);
                reference::check(&format!("{case}/01-hover-75ms"), &ui.frame());
                ui.at(150);
                reference::check(&format!("{case}/02-hover"), &ui.frame());
                ui.down();
                ui.at(650);
                reference::check(&format!("{case}/03-held"), &ui.frame());
                ui.up();
                ui.at(725);
                reference::check(&format!("{case}/04-release-75ms"), &ui.frame());
                ui.at(800);
                reference::check(&format!("{case}/05-released"), &ui.frame());
                ui.rebuild(fab_fixture(size, color, extended, true));
                reference::check(&format!("{case}/06-disabled"), &ui.frame());
            }
        }
    }
}
