use super::{harness::Harness, reference, theme, themed_name};
use crate::{
    Date, DateSelection, Element, Time, TimePart, date_input, date_picker, time_input, time_picker,
};
use iced::{Size, widget};
#[derive(Clone, Debug, PartialEq)]
enum Message {
    Date(DateSelection),
    Month(Date),
    Time(Time),
    Part(TimePart),
    Input(String),
    Years,
}
fn date(day: u8) -> Date {
    Date::new(2026, 9, day).unwrap()
}
fn make_ui(
    element: impl Into<Element<'static, Message>>,
    size: Size,
    dark: bool,
) -> Harness<'static, Message> {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    Harness::with_backend(element, size, theme(dark), &backend)
}
fn calendar(selection: DateSelection, years: bool) -> Element<'static, Message> {
    date_picker(date(1), selection)
        .today(date(13))
        .bounds(date(10), Date::new(2027, 12, 31).unwrap())
        .date_enabled(|d| d.weekday() != 0)
        .on_select(Message::Date)
        .on_month(Message::Month)
        .years(years)
        .on_toggle_years(Message::Years)
        .into()
}
fn clock(format24: bool, part: TimePart) -> Element<'static, Message> {
    time_picker(Time::new(10, 30).unwrap(), part)
        .format24(format24)
        .on_change(Message::Time)
        .on_part(Message::Part)
        .into()
}
#[test]
fn calendar_constraints_and_navigation_emit_only_valid_endpoints() {
    let mut ui = make_ui(
        calendar(DateSelection::default(), false),
        Size::new(360.0, 540.0),
        false,
    );
    ui.click("9");
    ui.click("13");
    assert!(ui.messages.is_empty());
    ui.click("18");
    assert_eq!(
        ui.messages,
        [Message::Date(DateSelection::Single(Some(date(18))))]
    );
    let nav_y = ui.find("September 2026").center_y();
    ui.move_to((276.0, nav_y));
    ui.down();
    ui.up();
    assert_eq!(ui.messages.len(), 1);
    ui.move_to((316.0, nav_y));
    ui.down();
    ui.up();
    assert_eq!(
        ui.messages.last(),
        Some(&Message::Month(Date::new(2026, 10, 1).unwrap()))
    );
}
#[test]
fn clock_distinguishes_inner_and_outer_hours_and_advances_after_release() {
    for (label, expected) in [("3", 3), ("15", 15)] {
        let mut ui = make_ui(clock(true, TimePart::Hour), Size::new(360.0, 500.0), false);
        let point = ui.find(label).center();
        ui.move_to(point);
        ui.down();
        assert_eq!(
            ui.messages,
            [Message::Time(Time::new(expected, 30).unwrap())]
        );
        ui.up();
        assert_eq!(ui.messages.last(), Some(&Message::Part(TimePart::Minute)));
    }
}
#[test]
fn clock_drag_supports_each_minute_and_cancel_does_not_advance() {
    let mut ui = make_ui(
        clock(false, TimePart::Minute),
        Size::new(360.0, 500.0),
        false,
    );
    let point = ui.find("15").center();
    ui.move_to(point);
    ui.down();
    assert_eq!(ui.messages, [Message::Time(Time::new(10, 15).unwrap())]);
    let right = ui.find("15").center();
    let top = ui.find("00").center();
    ui.move_to((top.x, right.y + right.x - top.x));
    assert_eq!(
        ui.messages.last(),
        Some(&Message::Time(Time::new(10, 30).unwrap()))
    );
    ui.leave();
    ui.up();
    assert!(ui.messages.iter().all(|m| !matches!(m, Message::Part(_))));
}
#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_date_and_time_pickers() {
    for dark in [false, true] {
        for (name, selection, years) in [
            ("single", DateSelection::Single(Some(date(18))), false),
            (
                "range",
                DateSelection::Range {
                    start: Some(date(15)),
                    end: Some(date(22)),
                },
                false,
            ),
            (
                "pending-range",
                DateSelection::Range {
                    start: Some(date(15)),
                    end: None,
                },
                false,
            ),
            ("years", DateSelection::Single(Some(date(18))), true),
        ] {
            let case = themed_name(&format!("picker-date-{name}"), dark);
            let mut ui = make_ui(calendar(selection, years), Size::new(360.0, 540.0), dark);
            ui.at(0);
            reference::check(&format!("{case}/00-default"), &ui.frame());
            if !years {
                let p = ui.find("18").center();
                ui.move_to(p);
                ui.down();
                for t in [0, 75, 150] {
                    ui.at(t);
                    reference::check(&format!("{case}/01-held-{t:03}ms"), &ui.frame());
                }
            }
        }
        for format24 in [false, true] {
            for part in [TimePart::Hour, TimePart::Minute] {
                let case = themed_name(
                    &format!(
                        "picker-time-{}-{part:?}",
                        if format24 { "24h" } else { "12h" }
                    ),
                    dark,
                );
                let mut ui = make_ui(clock(format24, part), Size::new(360.0, 470.0), dark);
                ui.at(0);
                reference::check(&format!("{case}/00-default"), &ui.frame());
            }
        }
        let fields = widget::column![
            date_input("Date", "2026-09-18").on_input(Message::Input),
            date_input("Invalid date", "2026-02-30").on_input(Message::Input),
            time_input("Time", "15:30").on_input(Message::Input),
            time_input("Invalid time", "25:90").on_input(Message::Input)
        ]
        .spacing(16);
        let mut ui = make_ui(
            widget::container(fields).padding(16),
            Size::new(360.0, 430.0),
            dark,
        );
        ui.at(0);
        reference::check(
            &format!("{}/00-input", themed_name("picker-input", dark)),
            &ui.frame(),
        );
    }
}

#[test]
fn year_pages_reach_the_final_partial_page() {
    let mut ui = make_ui(
        date_picker(date(1), DateSelection::default())
            .years(true)
            .bounds(date(1), Date::new(2030, 12, 31).unwrap())
            .on_month(Message::Month),
        Size::new(360.0, 540.0),
        false,
    );
    let nav_y = ui.find("2017–2028").center_y();
    ui.move_to((316.0, nav_y));
    ui.down();
    ui.up();
    assert_eq!(
        ui.messages,
        [Message::Month(Date::new(2029, 9, 1).unwrap())]
    );
}

#[test]
fn clock_hand_paints_in_the_dial_and_leaves_the_header_clear() {
    let theme = crate::Theme::light();
    let mut ui = make_ui(clock(true, TimePart::Hour), Size::new(360.0, 470.0), false);
    ui.at(0);
    // Locate the dial independently of the header's typography and spacing.
    let center = iced::Point::new(ui.find("12").center_x(), ui.find("3").center_y());
    let hand_midpoint = center + iced::Vector::new(-45.03332, -26.0);
    let header = ui.find("10");
    let frame = ui.frame();
    let pixel = |x: u32, y: u32| {
        let i = ((y * frame.width + x) * 4) as usize;
        [frame.pixels[i], frame.pixels[i + 1], frame.pixels[i + 2]]
    };
    let rgb = |c: iced::Color| {
        [
            (c.r * 255.0).round() as u8,
            (c.g * 255.0).round() as u8,
            (c.b * 255.0).round() as u8,
        ]
    };
    assert_eq!(
        pixel(
            (header.center_x() * 2.0) as u32,
            ((header.center_y() + 32.0) * 2.0) as u32
        ),
        rgb(theme.colors.primary_container)
    );
    let x = (hand_midpoint.x * 2.0).round() as u32;
    let y = (hand_midpoint.y * 2.0).round() as u32;
    assert!(
        (x - 2..=x + 2).any(|x| (y - 2..=y + 2).any(|y| pixel(x, y) == rgb(theme.colors.primary)))
    );
}
