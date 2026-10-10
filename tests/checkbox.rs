#[path = "support/pointer.rs"]
mod pointer;
#[path = "support/widget.rs"]
mod widget_harness;
use iced::{Event, Point, Size, mouse, window};
use iced_m3::{Element, checkbox};
use pointer::{press, redraw, release};
use std::time::Duration;
use widget_harness::Harness;

fn element(checked: bool, mixed: bool, enabled: bool) -> Element<'static, bool> {
    checkbox(checked)
        .indeterminate(mixed)
        .label("Workspace updates")
        .on_toggle_maybe(enabled.then_some(std::convert::identity))
        .into()
}

fn ui(checked: bool, mixed: bool, enabled: bool) -> Harness<bool> {
    Harness::new(
        element(checked, mixed, enabled),
        Size::new(200.0, 80.0),
        2.0,
    )
}

#[test]
fn checkbox_marks_follow_scroll_translation_at_retina_scale() {
    for (checked, mixed, enabled) in [
        (true, false, true),
        (false, true, true),
        (true, false, false),
    ] {
        let mut ui = ui(checked, mixed, enabled);
        let at_origin = ui.frame(0.0);
        assert!(
            at_origin == ui.frame(500.0),
            "scrolling must keep the mark inside its box"
        );
    }
}

#[test]
fn checkbox_has_a_real_hover_halo_in_both_selection_states_but_not_when_disabled() {
    for checked in [false, true] {
        let mut ui = ui(checked, false, true);
        let before = ui.frame(0.0);
        let point = Point::new(110.0, 24.0); // hover on the label, outside the box
        ui.event(
            Event::Mouse(mouse::Event::CursorMoved { position: point }),
            Some(point),
        );
        let (_, request) = ui.event(
            redraw(iced::time::Instant::now() + Duration::from_secs(1)),
            Some(point),
        );
        assert_eq!(
            request,
            window::RedrawRequest::Wait,
            "hover animation must settle"
        );
        let after = ui.frame(0.0);
        let pixel = |image: &[u8], x: usize, y: usize| {
            image[(y * 400 + x) * 4..(y * 400 + x) * 4 + 4].to_vec()
        };
        assert_ne!(
            pixel(&before, 48, 14),
            pixel(&after, 48, 14),
            "halo extends beyond 18px box"
        );
        assert_eq!(
            pixel(&before, 2, 2),
            pixel(&after, 2, 2),
            "halo stays circular and within target"
        );

        ui.rebuild(element(checked, false, false));
        let disabled = ui.frame(0.0);
        ui.event(
            Event::Mouse(mouse::Event::CursorMoved { position: point }),
            Some(point),
        );
        assert_eq!(
            disabled,
            ui.frame(0.0),
            "disabled controls have no hover effect"
        );
    }
}

#[test]
fn checkbox_selection_and_mixed_mark_animate_across_rebuilds_and_then_stop() {
    let mut ui = ui(false, false, true);
    let unchecked = ui.frame(0.0);
    let now = iced::time::Instant::now();
    ui.rebuild(element(true, false, true));
    assert_ne!(ui.event(redraw(now), None).1, window::RedrawRequest::Wait);
    assert_ne!(
        ui.event(redraw(now + Duration::from_millis(80)), None).1,
        window::RedrawRequest::Wait
    );
    let during = ui.frame(0.0);
    assert_eq!(
        ui.event(redraw(now + Duration::from_millis(400)), None).1,
        window::RedrawRequest::Wait
    );
    let checked = ui.frame(0.0);
    assert_ne!(unchecked, during);
    assert_ne!(
        during, checked,
        "selection must have an intermediate painted state"
    );
    ui.rebuild(element(false, true, true));
    ui.event(redraw(now + Duration::from_millis(500)), None);
    assert_eq!(
        ui.event(redraw(now + Duration::from_secs(1)), None).1,
        window::RedrawRequest::Wait
    );
    assert_ne!(checked, ui.frame(0.0), "mixed state uses a different mark");
    ui.rebuild(element(false, false, true));
    ui.event(redraw(now + Duration::from_millis(1100)), None);
    assert_eq!(
        ui.event(redraw(now + Duration::from_secs(2)), None).1,
        window::RedrawRequest::Wait
    );
    assert_eq!(unchecked, ui.frame(0.0));
}

#[test]
fn checkbox_padded_target_and_label_activate_once_and_cancel_interrupted_gestures() {
    let mut ui = ui(false, false, true);
    for point in [Point::new(3.0, 24.0), Point::new(110.0, 24.0)] {
        assert!(ui.event(press(), Some(point)).0.is_empty());
        assert_eq!(ui.event(release(), Some(point)).0, [true]);
        assert!(ui.event(release(), Some(point)).0.is_empty());
    }
    let point = Some(Point::new(24.0, 24.0));
    ui.event(press(), point);
    assert!(ui.event(release(), None).0.is_empty());
    ui.event(press(), point);
    ui.event(Event::Window(window::Event::Unfocused), point);
    assert!(ui.event(release(), point).0.is_empty());
    ui.event(press(), point); // works again after a modal's synthetic Unfocused
    assert_eq!(ui.event(release(), point).0, [true]);
    ui.event(press(), point);
    ui.rebuild(element(false, false, false));
    assert!(ui.event(release(), point).0.is_empty());
    assert!(ui.event(press(), point).0.is_empty());
}
