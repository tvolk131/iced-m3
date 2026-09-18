use super::{harness::Harness, reference, theme};
use crate::{Element, Theme, TypeScale, typography};
use iced::{
    Event, Length, Size,
    keyboard::{self, key::Named},
    widget, window,
};
#[derive(Clone, Debug, PartialEq)]
enum Message {
    Toggle(bool),
    Action,
    Close,
    Range((f32, f32)),
}
fn host(content: impl Into<Element<'static, Message>>) -> Element<'static, Message> {
    crate::focus::scope(widget::container(content).padding(24))
}
fn ui(
    content: impl Into<Element<'static, Message>>,
    size: Size,
    theme: Theme,
) -> Harness<'static, Message> {
    let mut ui = Harness::with_backend(
        content,
        size,
        theme,
        &std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into()),
    );
    ui.at(0);
    ui.frame();
    ui
}
fn tab(ui: &mut Harness<'_, Message>) {
    ui.event(Event::Keyboard(keyboard::Event::KeyPressed {
        key: keyboard::Key::Named(Named::Tab),
        modified_key: keyboard::Key::Named(Named::Tab),
        physical_key: keyboard::key::Physical::Unidentified(
            keyboard::key::NativeCode::Unidentified,
        ),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::empty(),
        text: None,
        repeat: false,
    }));
}
fn progress(circular: bool, loading: bool, value: f32) -> Element<'static, Message> {
    let p = if circular {
        crate::circular_progress(value)
    } else {
        crate::linear_progress(value)
    };
    host(p.indeterminate(loading))
}
#[test]
fn progress_handoff_preserves_the_first_frame_and_is_frame_rate_independent() {
    for circular in [false, true] {
        let run = |stepped: bool| {
            let mut ui = ui(
                progress(circular, true, 0.0),
                Size::new(360.0, 110.0),
                theme(false),
            );
            ui.at(450);
            let before = ui.frame();
            ui.rebuild(progress(circular, false, 0.75));
            ui.at(450);
            assert!(
                before == ui.frame(),
                "Changing mode must not replace the current arc/segments immediately"
            );
            if stepped {
                for ms in (460..=2450).step_by(10) {
                    ui.at(ms);
                }
            } else {
                ui.at(2450);
            }
            let in_spring = ui.frame();
            ui.at(5000);
            let final_frame = ui.frame();
            assert_eq!(ui.at(5001), window::RedrawRequest::Wait);
            let expected = self::ui(
                progress(circular, false, 0.75),
                Size::new(360.0, 110.0),
                theme(false),
            )
            .frame();
            assert!(
                final_frame == expected,
                "The final measured value must be exact"
            );
            (in_spring, final_frame)
        };
        let a = run(false);
        let b = run(true);
        assert!(
            a.0 == b.0 && a.1 == b.1,
            "Sparse redraws must not delay the handoff"
        );
    }
}
#[test]
fn progress_can_resume_loading_during_handoff_and_reduced_motion_skips_it() {
    for circular in [false, true] {
        let mut ui = ui(
            progress(circular, true, 0.0),
            Size::new(360.0, 110.0),
            theme(false),
        );
        ui.at(400);
        ui.rebuild(progress(circular, false, 0.8));
        ui.at(400);
        ui.at(450);
        let finishing = ui.frame();
        ui.rebuild(progress(circular, true, 0.0));
        ui.at(450);
        assert!(
            finishing == ui.frame(),
            "Retargeting a closing arc must preserve its current shape"
        );
        assert_eq!(ui.at(3000), window::RedrawRequest::NextFrame);
        ui.at(3100);
        assert!(ui.frame() != finishing);
        ui.rebuild(progress(circular, false, 0.6));
        ui.at(3100);
        ui.at(6000);
        ui.at(9000);
        assert_eq!(ui.at(9001), window::RedrawRequest::Wait);
        let mut t = theme(false);
        t.motion = crate::tokens::Motion::reduced();
        let mut reduced = self::ui(
            progress(circular, true, 0.0),
            Size::new(360.0, 110.0),
            t.clone(),
        );
        reduced.rebuild(progress(circular, false, 0.6));
        reduced.at(0);
        let expected = self::ui(progress(circular, false, 0.6), Size::new(360.0, 110.0), t).frame();
        assert!(reduced.frame() == expected);
        assert_eq!(reduced.at(1), window::RedrawRequest::Wait);
    }
}
fn check(selected: bool, mixed: bool) -> Element<'static, Message> {
    host(
        crate::checkbox(selected)
            .indeterminate(mixed)
            .label("Selection")
            .on_toggle(Message::Toggle),
    )
}
#[test]
fn checkbox_fill_fades_in_fifty_ms_and_deselection_keeps_the_hover_circle() {
    for dark in [false, true] {
        let mut ui = ui(check(false, false), Size::new(280.0, 100.0), theme(dark));
        ui.move_to((48.0, 48.0));
        ui.at(200);
        let hover = ui.frame();
        ui.rebuild(check(true, false));
        ui.at(200);
        ui.at(225);
        let early = ui.frame();
        ui.at(550);
        let selected = ui.frame();
        assert!(early != selected);
        ui.rebuild(check(false, false));
        ui.at(550);
        ui.at(600);
        let cleared = ui.frame();
        // The mark/fill are transparent after 50 ms while scale finishes over 150.
        assert!(
            cleared == hover,
            "Deselection must reveal the original outline and hover layer at 50 ms"
        );
        ui.at(700);
        assert_eq!(ui.at(701), window::RedrawRequest::Wait);
        ui.down();
        ui.at(850);
        let held = ui.frame();
        assert!(
            held.crop(56, 88, 4, 4) == hover.crop(56, 88, 4, 4),
            "The outer hover rim persists under press feedback"
        );
        assert!(ui.messages.is_empty());
        ui.up();
        assert_eq!(ui.messages, [Message::Toggle(true)]);
    }
}
fn chips(selected: bool) -> Element<'static, Message> {
    host(
        widget::column![
            crate::filter_chip("Filter", selected)
                .leading(Element::new(crate::glyph::Glyph::Close))
                .on_press(Message::Action),
            crate::input_chip("Avatar")
                .avatar(
                    widget::container(widget::space())
                        .width(24)
                        .height(24)
                        .style(|_| widget::container::Style {
                            background: Some(iced::Color::from_rgb(1.0, 0.0, 0.0).into()),
                            border: iced::Border {
                                radius: 12.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                )
                .selected(selected)
                .on_press(Message::Action)
                .on_remove(Message::Close)
        ]
        .spacing(16),
    )
}
#[test]
fn chip_selection_keeps_layout_and_avatar_and_has_a_finite_icon_transition() {
    let mut ui = ui(chips(false), Size::new(320.0, 170.0), theme(false));
    let label = ui.find("Filter");
    let avatar = ui.frame().crop(72, 168, 16, 16);
    ui.rebuild(chips(true));
    ui.at(0);
    let first = ui.frame();
    ui.at(75);
    let middle = ui.frame();
    ui.at(200);
    let last = ui.frame();
    assert_eq!(ui.find("Filter"), label);
    assert!(first != middle && middle != last);
    assert!(
        avatar == last.crop(72, 168, 16, 16),
        "Selection must not replace an input chip's avatar"
    );
    assert_eq!(ui.at(201), window::RedrawRequest::Wait);
    ui.click("Filter");
    assert_eq!(ui.messages, [Message::Action]);
}
fn range(pair: (f32, f32), long: bool) -> Element<'static, Message> {
    host(
        crate::range_slider(0.0..=100.0, pair)
            .value_labels(
                if long { "Lower limit: 48" } else { "48" },
                if long { "Upper limit: 52" } else { "52" },
            )
            .on_change(Message::Range),
    )
}
#[test]
fn range_labels_separate_while_both_handles_are_focused() {
    let mut ui = ui(
        range((48.0, 52.0), false),
        Size::new(360.0, 140.0),
        theme(false),
    );
    tab(&mut ui);
    ui.at(0);
    ui.at(100);
    let frame = ui.frame();
    let color = self::ui(
        host(
            widget::container(widget::space())
                .width(40)
                .height(40)
                .style(|t: &Theme| widget::container::Style {
                    background: Some(t.colors.primary.into()),
                    ..Default::default()
                }),
        ),
        Size::new(100.0, 100.0),
        theme(false),
    )
    .frame();
    let ink = &color.pixels
        [((60 * color.width + 60) * 4) as usize..((60 * color.width + 60) * 4 + 3) as usize];
    // Count separated capsule runs on a horizontal row above their text.
    let mut runs = 0;
    let mut previous = false;
    for x in 48..frame.width - 48 {
        let i = ((70 * frame.width + x) * 4) as usize;
        let filled = frame.pixels[i..i + 3] == *ink;
        if filled && !previous {
            runs += 1;
        }
        previous = filled;
    }
    assert_eq!(runs, 2, "Close values need two distinct readable capsules");
    assert!(ui.messages.is_empty());
}
fn modal(open: bool) -> Element<'static, Message> {
    crate::dialog::modal(
        widget::container(typography(
            "Background text stays covered",
            TypeScale::BodyLarge,
        ))
        .center_x(Length::Fill)
        .center_y(Length::Fill),
        crate::dialog(
            widget::column![
                typography("Review changes", TypeScale::HeadlineSmall),
                typography(
                    "Dialog contents appear as the surface grows.",
                    TypeScale::BodyMedium
                ),
                crate::button("Confirm").on_press(Message::Action)
            ]
            .spacing(16),
        )
        .on_dismiss(Message::Close),
        open,
    )
}
#[test]
fn dialog_reversal_keeps_its_position_and_exit_blocks_actions() {
    let mut ui = ui(modal(false), Size::new(520.0, 380.0), theme(false));
    ui.rebuild(modal(true));
    ui.at(0);
    ui.at(120);
    let before = ui.frame();
    ui.rebuild(modal(false));
    ui.at(120);
    assert!(before == ui.frame());
    ui.at(150);
    let closing = ui.frame();
    ui.rebuild(modal(true));
    ui.at(150);
    assert!(closing == ui.frame());
    ui.at(650);
    let point = ui.find("Confirm").center();
    ui.rebuild(modal(false));
    ui.at(650);
    ui.move_to(point);
    ui.down();
    ui.up();
    assert!(ui.messages.is_empty());
    ui.at(800);
    assert_eq!(ui.at(801), window::RedrawRequest::Wait);
}
#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_baseline_completion() {
    for dark in [false, true] {
        let mode = if dark { "dark" } else { "light" };
        let mut checkbox = ui(check(false, false), Size::new(280.0, 100.0), theme(dark));
        checkbox.move_to((48.0, 48.0));
        checkbox.at(200);
        checkbox.rebuild(check(true, false));
        for ms in [0, 25, 50, 100, 175, 350] {
            checkbox.at(200 + ms);
            reference::check(
                &format!("baseline-checkbox/{mode}/select-{ms:03}ms"),
                &checkbox.frame(),
            );
        }
        checkbox.rebuild(check(false, true));
        for ms in [0, 75, 175, 350] {
            checkbox.at(600 + ms);
            reference::check(
                &format!("baseline-checkbox/{mode}/mixed-{ms:03}ms"),
                &checkbox.frame(),
            );
        }
        checkbox.rebuild(check(false, false));
        for ms in [0, 25, 50, 100, 150] {
            checkbox.at(1000 + ms);
            reference::check(
                &format!("baseline-checkbox/{mode}/clear-{ms:03}ms"),
                &checkbox.frame(),
            );
        }
        let mut chips = ui(chips(false), Size::new(320.0, 170.0), theme(dark));
        chips.rebuild(self::chips(true));
        for ms in [0, 25, 75, 150, 200] {
            chips.at(ms);
            reference::check(
                &format!("baseline-chips/{mode}/select-{ms:03}ms"),
                &chips.frame(),
            );
        }
        for (name, width, pair, long) in [
            ("close", 360.0, (48.0, 52.0), false),
            ("long", 360.0, (48.0, 52.0), true),
            ("left", 360.0, (0.0, 1.0), false),
            ("right", 360.0, (99.0, 100.0), false),
            ("narrow", 160.0, (48.0, 52.0), true),
        ] {
            let mut range = ui(range(pair, long), Size::new(width, 140.0), theme(dark));
            tab(&mut range);
            for ms in [0, 50, 100] {
                range.at(ms);
                reference::check(
                    &format!("baseline-range-{name}/{mode}/focus-{ms:03}ms"),
                    &range.frame(),
                );
            }
        }
        for circular in [false, true] {
            let name = if circular { "circular" } else { "linear" };
            let mut progress = ui(
                progress(circular, true, 0.0),
                Size::new(360.0, 110.0),
                theme(dark),
            );
            progress.at(450);
            progress.rebuild(self::progress(circular, false, 0.75));
            for ms in [
                0, 100, 200, 333, 500, 1000, 1550, 1650, 1750, 2050, 3000, 4000,
            ] {
                progress.at(450 + ms);
                reference::check(
                    &format!("baseline-progress-{name}/{mode}/handoff-{ms:04}ms"),
                    &progress.frame(),
                );
            }
        }
        let mut modal = ui(modal(false), Size::new(520.0, 380.0), theme(dark));
        modal.rebuild(self::modal(true));
        for ms in [0, 25, 50, 100, 150, 250, 500] {
            modal.at(ms);
            reference::check(
                &format!("baseline-dialog/{mode}/open-{ms:03}ms"),
                &modal.frame(),
            );
        }
        modal.rebuild(self::modal(false));
        for ms in [0, 50, 100, 149, 150] {
            modal.at(600 + ms);
            reference::check(
                &format!("baseline-dialog/{mode}/close-{ms:03}ms"),
                &modal.frame(),
            );
        }
        let make_menu = || {
            host(crate::menu(
                "Commands",
                [
                    crate::MenuItem::new("Open workspace", Message::Action),
                    crate::MenuItem::new("Duplicate", Message::Action),
                    crate::MenuItem::new("Archive", Message::Action),
                ],
            ))
        };
        let mut menu = ui(make_menu(), Size::new(360.0, 320.0), theme(dark));
        menu.click("Commands");
        for ms in [0, 25, 50, 100, 250, 500] {
            menu.at(ms);
            reference::check(
                &format!("baseline-menu/{mode}/open-{ms:03}ms"),
                &menu.frame(),
            );
        }
        menu.click("Duplicate");
        for ms in [0, 50, 100, 149, 150] {
            menu.at(600 + ms);
            reference::check(
                &format!("baseline-menu/{mode}/close-{ms:03}ms"),
                &menu.frame(),
            );
        }
    }
}

#[test]
fn picker_headers_match_the_baseline_spacings_and_divider_extent() {
    let date = crate::Date::new(2026, 9, 14).unwrap();
    for (selection, height) in [
        (crate::DateSelection::Single(Some(date)), 120u32),
        (
            crate::DateSelection::Range {
                start: Some(date),
                end: Some(date),
            },
            128u32,
        ),
    ] {
        let mut ui = ui(
            host(crate::date_picker(date, selection)),
            Size::new(420.0, 660.0),
            theme(false),
        );
        assert_eq!(ui.find("Select date").y, 40.0);
        let frame = ui.frame();
        let t = theme(false);
        let c = t.colors.outline_variant;
        let expected = [
            (c.r * 255.0).round() as u8,
            (c.g * 255.0).round() as u8,
            (c.b * 255.0).round() as u8,
        ];
        for x in [25, 200, 382] {
            let index = (((24 + height) * 2 * frame.width + x * 2) * 4) as usize;
            assert_eq!(
                frame.pixels[index..index + 3],
                expected,
                "Full-width divider at {height}px"
            );
        }
    }
    let mut time = ui(
        host(crate::time_picker(
            crate::Time::new(3, 15).unwrap(),
            crate::TimePart::Hour,
        )),
        Size::new(420.0, 600.0),
        theme(false),
    );
    assert_eq!(time.find("Select time").y, 40.0);
    assert_eq!(
        time.find("AM").center_y(),
        88.0,
        "The 80px time display begins 44px below the surface top"
    );
}
