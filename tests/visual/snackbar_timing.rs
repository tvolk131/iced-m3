use super::harness::Harness;
use crate::{Element, Theme, snackbar};
use iced::{Event, Length, Size, widget, window};

#[derive(Clone, Debug, PartialEq)]
enum Message {
    Expired,
}

fn scene(id: Option<u64>, pause: Option<bool>) -> Element<'static, Message> {
    snackbar::host(
        widget::space().width(Length::Fill).height(Length::Fill),
        id.map(|id| {
            let notice = snackbar("Saved").id(id).on_dismiss(Message::Expired);
            match pause {
                Some(pause) => notice.pause_on_window_blur(pause),
                None => notice,
            }
        }),
    )
}

fn ui(id: Option<u64>, pause: Option<bool>) -> Harness<'static, Message> {
    let mut ui = Harness::configured(scene(id, pause), Size::new(600., 400.), Theme::light());
    ui.frame();
    ui.at(0);
    ui
}

fn focus(ui: &mut Harness<'_, Message>, focused: bool) {
    ui.event(Event::Window(if focused {
        window::Event::Focused
    } else {
        window::Event::Unfocused
    }));
}

#[test]
fn snackbar_timing_default_expires_unfocused_including_new_and_replaced_notices() {
    for initial in [None, Some(1)] {
        for replace in [false, true] {
            if initial.is_none() && !replace {
                continue;
            }
            for skip_redraws in [false, true] {
                let mut ui = ui(initial, None);
                ui.at(1000);
                focus(&mut ui, false);
                if replace {
                    ui.rebuild(scene(Some(2), None));
                    ui.at(1000);
                }
                let deadline = if replace { 5000 } else { 4000 };
                if !skip_redraws {
                    ui.at(deadline - 1);
                    assert!(ui.messages.is_empty());
                }
                ui.at(deadline + if skip_redraws { 60000 } else { 0 });
                assert_eq!(ui.messages, [Message::Expired]);
                focus(&mut ui, true);
                ui.at(70000);
                assert_eq!(ui.messages, [Message::Expired], "Dismiss exactly once");
                assert_eq!(ui.at(71000), window::RedrawRequest::Wait);
            }
        }
    }
}

#[test]
fn snackbar_timing_opt_in_preserves_remaining_time_across_blur_and_rebuild() {
    for replace in [false, true] {
        let mut ui = ui(Some(1), Some(true));
        ui.at(1000);
        focus(&mut ui, false);
        ui.rebuild(scene(Some(if replace { 2 } else { 1 }), Some(true)));
        ui.at(1000);
        assert_eq!(ui.at(61000), window::RedrawRequest::Wait);
        assert!(ui.messages.is_empty());
        focus(&mut ui, true);
        let deadline = if replace { 65000 } else { 64000 };
        ui.at(deadline - 1);
        assert!(ui.messages.is_empty(), "Resume the remaining duration");
        ui.at(deadline);
        assert_eq!(ui.messages, [Message::Expired]);
    }
}

#[test]
fn snackbar_timing_hover_pauses_but_stale_hover_cannot_pause_an_unfocused_window() {
    let mut ui = ui(Some(1), None);
    ui.at(1000);
    let label = ui.find("Saved").center();
    ui.move_to(label);
    ui.at(11000);
    assert!(ui.messages.is_empty());
    // Alt-Tab need not move the pointer or produce CursorLeft.
    focus(&mut ui, false);
    ui.at(13999);
    assert!(ui.messages.is_empty());
    ui.at(14000);
    assert_eq!(ui.messages, [Message::Expired]);
}

#[test]
fn snackbar_timing_policy_changes_do_not_restart_the_countdown() {
    let mut ui = ui(Some(1), Some(true));
    ui.at(1000);
    focus(&mut ui, false);
    ui.at(11000);
    ui.rebuild(scene(Some(1), Some(false)));
    ui.at(11000);
    ui.at(12000);
    ui.rebuild(scene(Some(1), Some(true)));
    ui.at(12000);
    ui.at(22000);
    assert!(ui.messages.is_empty());
    focus(&mut ui, true);
    ui.at(23999);
    assert!(ui.messages.is_empty());
    ui.at(24000);
    assert_eq!(ui.messages, [Message::Expired]);
}
