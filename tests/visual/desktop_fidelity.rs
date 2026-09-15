use super::{harness::Harness, reference, theme};
use crate::{Date, DateSelection, Element, Theme, Time, TimePart, TypeScale, typography};
use iced::{Color, Size, widget};

#[derive(Clone, Debug, PartialEq)]
enum Message {
    Change,
    Cancel,
    ConfirmTime(Time),
    ConfirmDate(DateSelection),
    Input(String),
}
fn backend() -> String {
    std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into())
}
fn host(e: impl Into<Element<'static, Message>>) -> Element<'static, Message> {
    crate::focus::scope(widget::container(e).padding(24))
}
fn ui(
    e: impl Into<Element<'static, Message>>,
    size: Size,
    theme: Theme,
) -> Harness<'static, Message> {
    let mut ui = Harness::with_backend(host(e), size, theme, &backend());
    // Native text inputs derive their enabled style during the first event.
    ui.at(0);
    ui
}
fn pixel(image: &reference::Image, x: f32, y: f32) -> [u8; 4] {
    let offset = (((y * 2.0) as u32 * image.width + (x * 2.0) as u32) * 4) as usize;
    image.pixels[offset..offset + 4].try_into().unwrap()
}
fn swatch(color: Color, theme: Theme) -> [u8; 4] {
    pixel(
        &ui(
            widget::container(widget::space())
                .width(50)
                .height(50)
                .style(move |_| widget::container::Style {
                    background: Some(color.into()),
                    ..Default::default()
                }),
            Size::new(100.0, 100.0),
            theme,
        )
        .frame(),
        40.0,
        40.0,
    )
}
fn raised(level: f32) -> Element<'static, Message> {
    crate::elevated(
        widget::container(widget::space())
            .width(180)
            .height(72)
            .style(|theme: &Theme| widget::container::Style {
                background: Some(theme.colors.surface_container_low.into()),
                border: iced::Border {
                    radius: 12.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        level,
        12.0,
    )
}

#[test]
fn material_shadows_have_an_ambient_extent_without_darkening_the_surface() {
    for dark in [false, true] {
        let t = theme(dark);
        let flat = ui(raised(0.0), Size::new(260.0, 150.0), t.clone()).frame();
        let raised = ui(raised(3.0), Size::new(260.0, 150.0), t.clone()).frame();
        assert!(
            flat.crop(80, 64, 260, 100) == raised.crop(80, 64, 260, 100),
            "Outset shadows must not tint the surface"
        );
        assert_ne!(
            pixel(&flat, 100.0, 102.0),
            pixel(&raised, 100.0, 102.0),
            "Ambient blur extends beyond the old key shadow"
        );
        assert_eq!(
            pixel(&flat, 100.0, 125.0),
            pixel(&raised, 100.0, 125.0),
            "Finite blur bounds must not leak"
        );
        let mut tinted = t.clone();
        tinted.colors.shadow = Color::from_rgb(1.0, 0.0, 0.0);
        let colored = ui(self::raised(3.0), Size::new(260.0, 150.0), tinted).frame();
        assert_ne!(pixel(&raised, 100.0, 102.0), pixel(&colored, 100.0, 102.0));
        let mut hidden = t;
        hidden.shadows = false;
        assert!(flat == ui(self::raised(3.0), Size::new(260.0, 150.0), hidden).frame());
    }
}
fn clock(pm: bool, numeric: bool, h: &str, m: &str) -> Element<'static, Message> {
    crate::time_picker(
        Time::new(if pm { 15 } else { 3 }, 15).unwrap(),
        TimePart::Hour,
    )
    .on_change(|_| Message::Change)
    .on_part(|_| Message::Change)
    .input_mode(numeric)
    .hour_input(h, Message::Input)
    .minute_input(m, Message::Input)
    .input_period(pm, |_| Message::Change)
    .on_cancel(Message::Cancel)
    .on_confirm(Message::ConfirmTime)
    .into()
}
#[test]
fn picker_period_selection_preserves_the_entire_group_outline() {
    for dark in [false, true] {
        for numeric in [false, true] {
            let mut a = ui(
                clock(false, numeric, "03", "15"),
                Size::new(420.0, 580.0),
                theme(dark),
            );
            let am = a.find("AM");
            let pm = a.find("PM");
            let left = am.center_x() - 26.0;
            let top = am.center_y() - 20.0;
            let a = a.frame();
            let mut b = ui(
                clock(true, numeric, "03", "15"),
                Size::new(420.0, 580.0),
                theme(dark),
            );
            let b = b.frame();
            for (x, y) in [
                (left + 20.0, top + 0.25),
                (left + 20.0, top + 40.25),
                (left + 20.0, top + 79.75),
                (left + 0.25, top + 12.0),
                (left + 51.25, top + 65.0),
            ] {
                assert_eq!(
                    pixel(&a, x, y),
                    pixel(&b, x, y),
                    "Selected fill must stay beneath the outline at {x},{y}; AM {am:?}, PM {pm:?}, numeric {numeric}"
                );
            }
            assert_ne!(
                pixel(&a, left + 10.0, am.center_y()),
                pixel(&b, left + 10.0, am.center_y())
            );
            assert_eq!(pm.center_y() - am.center_y(), 40.0);
        }
    }
}
#[test]
fn picker_confirmation_is_explicit_validated_and_cancel_never_commits() {
    for valid in [false, true] {
        let mut time = ui(
            clock(true, true, if valid { "12" } else { "25" }, "05"),
            Size::new(420.0, 390.0),
            theme(false),
        );
        time.click("OK");
        assert_eq!(
            time.messages,
            if valid {
                vec![Message::ConfirmTime(Time::new(12, 5).unwrap())]
            } else {
                vec![]
            }
        );
        time.messages.clear();
        time.click("Cancel");
        assert_eq!(time.messages, [Message::Cancel]);
        let date = Date::new(2026, 9, 14).unwrap();
        let mut calendar = ui(
            crate::date_picker(date, DateSelection::Single(None))
                .bounds(date, date)
                .input_mode(true)
                .input(
                    if valid { "2026-09-14" } else { "2026-09-15" },
                    Message::Input,
                )
                .on_select(|_| Message::Change)
                .on_confirm(Message::ConfirmDate)
                .on_cancel(Message::Cancel),
            Size::new(420.0, 480.0),
            theme(false),
        );
        calendar.click("OK");
        assert_eq!(
            calendar.messages,
            if valid {
                vec![Message::ConfirmDate(DateSelection::Single(Some(date)))]
            } else {
                vec![]
            }
        );
        calendar.messages.clear();
        calendar.click("Cancel");
        assert_eq!(calendar.messages, [Message::Cancel]);
    }
    let mut calendar = ui(
        crate::date_picker(Date::new(2026, 9, 1).unwrap(), DateSelection::Single(None))
            .on_confirm(Message::ConfirmDate),
        Size::new(420.0, 640.0),
        theme(false),
    );
    calendar.click("OK");
    assert!(
        calendar.messages.is_empty(),
        "Incomplete calendar selection cannot confirm"
    );
}
#[test]
fn slider_label_scales_on_hover_and_focus_and_obeys_reduced_motion() {
    for range in [false, true] {
        let build = || -> Element<'static, Message> {
            if range {
                crate::range_slider(0.0..=100.0, (25.0, 75.0))
                    .labeled(true)
                    .on_change(|_| Message::Change)
                    .into()
            } else {
                crate::slider(0.0..=100.0, 50.0)
                    .labeled(true)
                    .on_change(|_| Message::Change)
                    .into()
            }
        };
        let mut ui = ui(build(), Size::new(320.0, 140.0), theme(false));
        ui.at(0);
        let idle = ui.frame();
        let x = if range { 104.0 } else { 160.0 };
        ui.move_to((x, 80.0));
        ui.at(50);
        let growing = ui.frame();
        ui.at(100);
        let held = ui.frame();
        assert!(idle.crop(48, 48, 540, 70) != growing.crop(48, 48, 540, 70));
        assert!(growing.crop(48, 48, 540, 70) != held.crop(48, 48, 540, 70));
        ui.leave();
        ui.at(250);
        assert!(ui.frame() == idle);
        ui.event(iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Tab),
            modified_key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Tab),
            physical_key: iced::keyboard::key::Physical::Unidentified(
                iced::keyboard::key::NativeCode::Unidentified,
            ),
            location: iced::keyboard::Location::Standard,
            modifiers: iced::keyboard::Modifiers::empty(),
            text: None,
            repeat: false,
        }));
        ui.at(250);
        ui.at(350);
        assert!(ui.frame().crop(48, 48, 540, 70) != idle.crop(48, 48, 540, 70));
        let mut reduced = theme(false);
        reduced.motion = crate::tokens::Motion::reduced();
        let mut ui = self::ui(build(), Size::new(320.0, 140.0), reduced);
        ui.at(0);
        ui.frame();
        ui.move_to((x, 80.0));
        ui.at(0);
        assert_eq!(ui.at(1), iced::window::RedrawRequest::Wait);
    }
}
#[test]
fn progress_tracks_have_clear_gaps_rounded_ends_and_a_determinate_stop_marker() {
    for dark in [false, true] {
        let t = theme(dark);
        let mut ui = ui(
            crate::linear_progress(0.5).width(200),
            Size::new(250.0, 64.0),
            t.clone(),
        );
        let frame = ui.frame();
        assert_eq!(
            pixel(&frame, 100.0, 26.0),
            swatch(t.colors.primary, t.clone())
        );
        assert_eq!(
            pixel(&frame, 126.0, 26.0),
            swatch(t.colors.surface, t.clone())
        );
        assert_eq!(
            pixel(&frame, 150.0, 26.0),
            swatch(t.colors.primary_container, t.clone())
        );
        assert_eq!(
            pixel(&frame, 222.0, 26.0),
            swatch(t.colors.primary, t.clone())
        );
        assert_ne!(pixel(&frame, 24.0, 24.0), pixel(&frame, 30.0, 24.0));
        ui.rebuild(host(crate::linear_progress(1.0).width(200)));
        ui.at(0);
        ui.at(250);
        let full = ui.frame();
        assert_eq!(pixel(&full, 126.0, 26.0), swatch(t.colors.primary, t));
    }
}

#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_desktop_fidelity() {
    for dark in [false, true] {
        let mode = if dark { "dark" } else { "light" };
        let mut row = widget::row![].spacing(40);
        for level in 0..=5 {
            row = row.push(
                widget::column![
                    typography(format!("Level {level}"), TypeScale::LabelLarge),
                    raised(level as f32)
                ]
                .spacing(24),
            );
        }
        reference::check(
            &format!("finish-elevation-{mode}/all-levels"),
            &ui(row, Size::new(1390.0, 210.0), theme(dark)).frame(),
        );
        for (kind, element, size) in [
            (
                "clock",
                clock(false, false, "03", "15"),
                Size::new(420.0, 580.0),
            ),
            (
                "numeric",
                clock(true, true, "12", "05"),
                Size::new(420.0, 380.0),
            ),
            (
                "invalid",
                clock(true, true, "25", "05"),
                Size::new(420.0, 480.0),
            ),
            (
                "calendar",
                crate::date_picker(
                    Date::new(2026, 9, 1).unwrap(),
                    DateSelection::Single(Some(Date::new(2026, 9, 14).unwrap())),
                )
                .on_select(|_| Message::Change)
                .on_confirm(Message::ConfirmDate)
                .on_cancel(Message::Cancel)
                .into(),
                Size::new(420.0, 610.0),
            ),
        ] {
            reference::check(
                &format!("finish-picker-{mode}/{kind}"),
                &ui(element, size, theme(dark)).frame(),
            );
        }
        for range in [false, true] {
            let element: Element<'static, Message> = if range {
                crate::range_slider(0.0..=100.0, (40.0, 45.0))
                    .labeled(true)
                    .on_change(|_| Message::Change)
                    .into()
            } else {
                crate::slider(0.0..=100.0, 50.0)
                    .labeled(true)
                    .on_change(|_| Message::Change)
                    .into()
            };
            let mut ui = ui(element, Size::new(320.0, 140.0), theme(dark));
            ui.at(0);
            let group = format!("finish-slider-{range}-{mode}");
            reference::check(&format!("{group}/idle"), &ui.frame());
            ui.move_to((if range { 148.8 } else { 160.0 }, 80.0));
            for ms in [0, 25, 50, 75, 100, 150] {
                ui.at(ms);
                reference::check(&format!("{group}/hover-{ms:03}"), &ui.frame());
            }
            ui.down();
            ui.at(350);
            reference::check(&format!("{group}/held"), &ui.frame());
            ui.up();
            ui.leave();
            ui.at(550);
            reference::check(&format!("{group}/released"), &ui.frame());
        }
        for circular in [false, true] {
            for indeterminate in [false, true] {
                let make = |value| {
                    if circular {
                        crate::circular_progress(value)
                    } else {
                        crate::linear_progress(value).width(260)
                    }
                    .indeterminate(indeterminate)
                };
                let mut ui = ui(make(0.5), Size::new(320.0, 110.0), theme(dark));
                ui.at(0);
                ui.frame();
                let group = format!("finish-progress-{circular}-{indeterminate}-{mode}");
                if indeterminate {
                    for ms in [0, 167, 333, 667, 1000, 1166, 1334, 1350, 2017, 5334, 5400] {
                        ui.at(ms);
                        reference::check(&format!("{group}/{ms:04}"), &ui.frame());
                    }
                } else {
                    for (i, value) in [0.0, 0.005, 0.01, 0.25, 0.5, 0.99, 1.0]
                        .into_iter()
                        .enumerate()
                    {
                        ui.rebuild(host(make(value)));
                        ui.at(i as u64 * 300);
                        ui.at(i as u64 * 300 + 250);
                        reference::check(&format!("{group}/value-{i}"), &ui.frame());
                    }
                }
            }
        }
    }
}
