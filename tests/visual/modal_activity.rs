use super::{harness::Harness, reference, theme};
use crate::{Element, Theme, TypeScale, typography};
use iced::{Event, Length, Size, widget, window};

const HOSTS: [&str; 7] = [
    "dialog",
    "stack",
    "nested",
    "side",
    "bottom",
    "mixed",
    "dialog-over-sheet",
];

#[derive(Clone, Debug, PartialEq)]
enum Message {
    Action,
    Expired,
    Input(String),
}

fn fixture(host: &str, open: bool, paused: bool) -> Element<'static, Message> {
    let indicators = widget::column![
        crate::circular_progress(0.)
            .indeterminate(true)
            .paused(paused),
        crate::linear_progress(0.)
            .indeterminate(true)
            .paused(paused)
            .width(140),
        crate::loading_indicator().paused(paused),
        crate::button("Background action").on_press(Message::Action),
        crate::text_field("Background input", "")
            .id("background-entry")
            .on_input(Message::Input),
    ]
    .spacing(16);
    let background = widget::container(indicators)
        .padding(24)
        .width(Length::Fill)
        .height(Length::Fill);
    let background = crate::snackbar::host(
        background,
        Some(
            crate::snackbar("Pending action")
                .duration(std::time::Duration::from_secs(2))
                .action("Undo", Message::Action)
                .on_dismiss(Message::Expired),
        ),
    );
    let content = || typography("Modal content", TypeScale::HeadlineSmall);
    let dialog = || crate::dialog(content()).width(280.);
    match host {
        "dialog" => crate::dialog::modal(background, dialog(), open),
        "stack" => crate::dialog::stack(background, [(dialog(), open)]),
        "nested" => crate::dialog::stack(
            widget::space().width(Length::Fill).height(Length::Fill),
            [
                (
                    crate::dialog(widget::container(background).height(420)).width(680.),
                    true,
                ),
                (dialog(), open),
            ],
        ),
        "side" => crate::sheet::host(
            background,
            crate::side_sheet(content()).modal().width(280.).open(open),
        ),
        "bottom" => crate::sheet::host(
            background,
            crate::bottom_sheet(content()).height(160.).open(open),
        ),
        // A closed inner modal host must inherit the outer host's real window
        // activity instead of mistaking cancellation for app deactivation.
        "mixed" => crate::sheet::host(
            crate::dialog::modal(background, dialog(), false),
            crate::side_sheet(content()).modal().width(280.).open(open),
        ),
        "dialog-over-sheet" => crate::dialog::modal(
            crate::sheet::host(
                background,
                crate::side_sheet(content()).width(200.).open(true),
            ),
            dialog(),
            open,
        ),
        _ => unreachable!(),
    }
}

fn ui(host: &str, open: bool, paused: bool, theme: Theme) -> Harness<'static, Message> {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    let mut ui = Harness::with_backend(
        fixture(host, open, paused),
        Size::new(800., 600.),
        theme,
        &backend,
    );
    ui.frame();
    ui.at(0);
    ui
}

fn indicators(host: &str, image: reference::Image) -> [reference::Image; 3] {
    let (x, y) = if host == "nested" {
        (108, 114)
    } else {
        (24, 24)
    };
    [
        image.crop(x * 2, y * 2, 96, 96),
        image.crop(x * 2, (y + 64) * 2, 280, 8),
        image.crop(x * 2, (y + 84) * 2, 96, 96),
    ]
}

#[test]
fn visible_progress_keeps_moving_behind_modals_without_background_actions() {
    for host in HOSTS {
        for dark in [false, true] {
            let mut ui = ui(host, false, false, theme(dark));
            let target = ui.find("Background action").center();
            ui.move_to(target);
            ui.down();
            ui.at(100);
            ui.rebuild(fixture(host, true, false));
            ui.at(100);
            ui.at(800);
            let before = indicators(host, ui.frame());
            assert_eq!(ui.at(1037), window::RedrawRequest::NextFrame, "{host}");
            let after = indicators(host, ui.frame());
            for i in 0..3 {
                assert!(
                    before[i] != after[i],
                    "Indicator {i} must move behind {host}"
                );
            }
            ui.up();
            ui.down();
            ui.up();
            ui.at(5000);
            assert!(
                ui.messages.is_empty(),
                "Covered actions/timeouts stay blocked: {host}"
            );
            ui.rebuild(fixture(host, false, false));
            ui.at(5000);
            ui.at(5600);
            ui.up();
            assert!(
                ui.messages.is_empty(),
                "Covered gestures must not replay: {host}"
            );
            ui.click("Background action");
            assert_eq!(ui.messages, [Message::Action]);
            ui.click("Undo");
            assert_eq!(
                ui.messages,
                [Message::Action, Message::Action],
                "Snackbar action must still be available after uncovering {host}"
            );
        }
    }
}

#[test]
fn closing_a_modal_resumes_the_snackbar_timer_without_pointer_movement() {
    // Use the runtime harness's scoped clock so synthetic focus events and
    // redraws share the same timeline throughout the retained dialog's exit.
    let mut ui = ui("dialog", true, true, theme(false));
    ui.at(30000);
    assert!(ui.messages.is_empty());
    ui.rebuild(fixture("dialog", false, true));
    ui.at(30000);
    ui.at(30075);
    assert!(ui.messages.is_empty(), "The timer stays paused during exit");
    ui.at(31000);
    assert!(
        ui.messages.is_empty(),
        "Exit resumes the remaining duration"
    );
    ui.at(32999);
    assert!(ui.messages.is_empty());
    ui.at(33001);
    assert_eq!(ui.messages, [Message::Expired]);
    ui.at(40000);
    assert_eq!(
        ui.messages,
        [Message::Expired],
        "Expiration is emitted once"
    );
}

#[test]
fn covered_fields_stay_suspended_after_the_real_window_regains_focus() {
    for host in HOSTS {
        let mut ui = ui(host, false, true, theme(false));
        ui.operate(
            &mut iced::advanced::widget::operation::focusable::focus::<()>(widget::Id::new(
                "background-entry",
            )),
        );
        assert_ne!(
            ui.at(100),
            window::RedrawRequest::Wait,
            "The background caret starts focused: {host}"
        );
        ui.rebuild(fixture(host, true, true));
        ui.at(100);
        ui.at(800);
        ui.event(Event::Window(window::Event::Unfocused));
        ui.event(Event::Window(window::Event::Focused));
        assert_eq!(
            ui.at(1500),
            window::RedrawRequest::Wait,
            "Regaining app focus must not restart a covered caret: {host}"
        );
        let before = ui.frame();
        ui.event(Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Character("x".into()),
            modified_key: iced::keyboard::Key::Character("x".into()),
            physical_key: iced::keyboard::key::Physical::Unidentified(
                iced::keyboard::key::NativeCode::Unidentified,
            ),
            location: iced::keyboard::Location::Standard,
            modifiers: iced::keyboard::Modifiers::empty(),
            text: Some("x".into()),
            repeat: false,
        }));
        assert_eq!(ui.at(1800), window::RedrawRequest::Wait);
        assert!(
            before == ui.frame(),
            "Covered text must remain unchanged: {host}"
        );
        assert!(ui.messages.is_empty());
    }
}

#[test]
fn covered_progress_obeys_window_focus_pause_and_reduced_motion() {
    for host in HOSTS {
        let mut ui = ui(host, true, false, theme(false));
        ui.at(700);
        let moving = indicators(host, ui.frame());
        ui.event(Event::Window(window::Event::Unfocused));
        assert_eq!(ui.at(4000), window::RedrawRequest::Wait, "{host}");
        assert!(
            moving == indicators(host, ui.frame()),
            "App deactivation pauses {host}"
        );
        ui.event(Event::Window(window::Event::Focused));
        ui.at(4000);
        assert!(
            moving == indicators(host, ui.frame()),
            "Inactive time must not advance {host}"
        );
        ui.at(4237);
        assert!(
            moving != indicators(host, ui.frame()),
            "Real focus resumes {host}"
        );
        ui.rebuild(fixture(host, true, true));
        ui.at(4237);
        let paused = indicators(host, ui.frame());
        assert_eq!(ui.at(8000), window::RedrawRequest::Wait, "{host}");
        assert!(paused == indicators(host, ui.frame()));

        let mut reduced = self::ui(host, true, false, theme(false).reduced_motion(true));
        reduced.at(700);
        let still = reduced.frame();
        assert_eq!(reduced.at(1000), window::RedrawRequest::Wait, "{host}");
        assert!(still == reduced.frame());
    }
}

#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_modal_activity() {
    for host in HOSTS {
        for dark in [false, true] {
            let mut ui = ui(host, false, false, theme(dark));
            ui.at(100);
            ui.rebuild(fixture(host, true, false));
            ui.at(100);
            for ms in [800, 1037, 1274] {
                ui.at(ms);
                reference::check(
                    &format!(
                        "modal-activity/{}/{host}/{ms:04}ms",
                        if dark { "dark" } else { "light" }
                    ),
                    &ui.frame(),
                );
            }
        }
    }
}
