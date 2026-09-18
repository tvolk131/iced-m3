use super::*;
use crate::{
    Segment, SegmentSelection, circular_progress, linear_progress, segmented_buttons, slider,
};

fn slider_view(value: f32, discrete: bool, disabled: bool) -> Element<'static, Message> {
    let slider = slider(0.0..=100.0, value)
        .labeled(true)
        .on_change(Message::Value)
        .on_release(Message::Action)
        .disabled(disabled);
    padded(if discrete {
        slider.step(10.0).ticks(true)
    } else {
        slider
    })
}
fn segments(selection: SegmentSelection<u8>, disabled: bool) -> Element<'static, Message> {
    padded(
        segmented_buttons(
            [
                Segment::new(0, "PNG"),
                Segment::new(1, "JPEG").icon(icon()),
                Segment::new(2, "WebP").disabled(true),
            ],
            selection,
        )
        .width(300)
        .on_change(Message::Segments)
        .disabled(disabled),
    )
}

#[test]
fn segmented_selection_cannot_cover_the_connected_outline() {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    for dark in [false, true] {
        let frames: Vec<_> = [0, 1]
            .into_iter()
            .map(|selected| {
                let mut ui = Harness::with_backend(
                    segments(SegmentSelection::Single(Some(selected)), false),
                    Size::new(340.0, 90.0),
                    theme(dark),
                    &backend,
                );
                ui.frame().crop(104, 32, 12, 2)
            })
            .collect();
        assert!(
            frames[0] == frames[1],
            "selection fill must stay beneath the outline"
        );
    }
}
fn progress_view(
    circular: bool,
    value: f32,
    indeterminate: bool,
    paused: bool,
) -> Element<'static, Message> {
    padded(
        if circular {
            circular_progress(value)
        } else {
            linear_progress(value)
        }
        .indeterminate(indeterminate)
        .paused(paused),
    )
}

#[test]
fn slider_drag_clamps_snaps_and_commits_after_the_last_value() {
    let mut ui = Harness::new(
        slider_view(50.0, true, false),
        Size::new(320.0, 120.0),
        Theme::light(),
    );
    ui.at(0);
    ui.frame();
    ui.move_to((160.0, 72.0));
    ui.down();
    assert!(
        ui.messages.is_empty(),
        "pressing the current value is not a change"
    );
    ui.move_to((221.0, 72.0));
    assert_eq!(ui.messages, [Message::Value(80.0)]);
    ui.rebuild(slider_view(80.0, true, false));
    ui.move_to((222.0, 72.0));
    assert_eq!(
        ui.messages.len(),
        1,
        "a rebuild preserves drag and deduplicates steps"
    );
    ui.move_to((500.0, 72.0));
    ui.up();
    assert_eq!(
        ui.messages,
        [Message::Value(80.0), Message::Value(100.0), Message::Action]
    );
    ui.move_to((160.0, 72.0));
    ui.down();
    ui.move_to((-20.0, 72.0));
    ui.up();
    assert_eq!(
        &ui.messages[ui.messages.len() - 2..],
        [Message::Value(0.0), Message::Action]
    );
}

#[test]
fn slider_cancel_and_disabled_rebuild_do_not_commit() {
    for cancel in 0..3 {
        let mut ui = Harness::new(
            slider_view(50.0, true, false),
            Size::new(320.0, 120.0),
            Theme::light(),
        );
        ui.at(0);
        ui.frame();
        ui.move_to((160.0, 72.0));
        ui.down();
        match cancel {
            0 => {
                ui.event(Event::Window(window::Event::Unfocused));
            }
            1 => ui.leave(),
            _ => ui.rebuild(slider_view(50.0, true, true)),
        }
        ui.move_to((200.0, 72.0));
        ui.up();
        assert!(ui.messages.is_empty());
        ui.leave();
        ui.at(500);
        assert_eq!(ui.at(1000), window::RedrawRequest::Wait);
    }
}

#[test]
fn slider_right_click_label_space_and_invalid_ranges_are_inert() {
    let mut ui = Harness::new(
        slider_view(50.0, false, false),
        Size::new(320.0, 120.0),
        Theme::light(),
    );
    ui.move_to((160.0, 30.0));
    ui.down();
    ui.up();
    ui.move_to((160.0, 72.0));
    ui.event(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Right,
    )));
    ui.event(Event::Mouse(mouse::Event::ButtonReleased(
        mouse::Button::Right,
    )));
    assert!(ui.messages.is_empty());
    for (min, max) in [
        (1.0, 1.0),
        (10.0, 0.0),
        (f32::NAN, 100.0),
        (0.0, f32::INFINITY),
    ] {
        let mut ui = Harness::new(
            padded(
                slider(min..=max, f32::NAN)
                    .on_change(Message::Value)
                    .ticks(true)
                    .step(0.0),
            ),
            Size::new(320.0, 100.0),
            Theme::light(),
        );
        ui.at(0);
        ui.frame();
        ui.move_to((60.0, 40.0));
        ui.down();
        ui.up();
        assert!(ui.messages.is_empty());
    }
}

#[test]
fn segmented_single_multiple_and_disabled_items_emit_controlled_values() {
    let mut ui = Harness::new(
        segments(SegmentSelection::Single(Some(0)), false),
        Size::new(340.0, 90.0),
        Theme::light(),
    );
    ui.click("JPEG");
    ui.click("WebP");
    assert_eq!(
        ui.messages,
        [Message::Segments(SegmentSelection::Single(Some(1)))]
    );
    ui.rebuild(segments(SegmentSelection::Multiple(vec![0, 1]), false));
    ui.click("PNG");
    assert_eq!(
        ui.messages.last(),
        Some(&Message::Segments(SegmentSelection::Multiple(vec![1])))
    );
    ui.rebuild(segments(SegmentSelection::Multiple(vec![1]), false));
    ui.click("JPEG");
    assert_eq!(
        ui.messages.last(),
        Some(&Message::Segments(SegmentSelection::Multiple(vec![])))
    );
    let point = ui.find("PNG").center();
    ui.move_to(point);
    ui.down();
    ui.rebuild(segments(SegmentSelection::Single(Some(1)), true));
    ui.up();
    assert_eq!(ui.messages.len(), 3, "disabling cancels an in-flight press");
}

#[test]
fn progress_pauses_resumes_and_finishes_without_idle_redraws() {
    for circular in [false, true] {
        let mut ui = Harness::new(
            progress_view(circular, 0.0, true, false),
            Size::new(320.0, 90.0),
            Theme::light(),
        );
        ui.at(0);
        let initial = ui.frame();
        ui.at(250);
        let moving = ui.frame();
        assert!(moving != initial);
        ui.rebuild(progress_view(circular, 0.0, true, true));
        assert_eq!(ui.at(1000), window::RedrawRequest::Wait);
        assert!(ui.frame() == moving);
        ui.rebuild(progress_view(circular, 0.0, true, false));
        ui.at(1100);
        assert!(ui.frame() == moving, "resume does not include paused time");
        ui.at(1400);
        assert!(ui.frame() != moving);
        ui.event(Event::Window(window::Event::Unfocused));
        let unfocused = ui.frame();
        assert_eq!(ui.at(2000), window::RedrawRequest::Wait);
        assert!(ui.frame() == unfocused);
        ui.event(Event::Window(window::Event::Focused));
        ui.at(2200);
        ui.rebuild(progress_view(circular, 0.5, false, false));
        ui.at(2300);
        ui.frame();
        ui.rebuild(progress_view(circular, 1.0, false, false));
        ui.at(2400);
        ui.at(2600);
        ui.at(6000);
        let complete = ui.frame();
        assert_eq!(ui.at(6001), window::RedrawRequest::Wait);
        assert!(ui.frame() == complete);
        assert!(ui.messages.is_empty());
    }
}

#[test]
fn hidden_progress_and_zero_motion_do_not_schedule_animation() {
    for circular in [false, true] {
        let mut ui = Harness::new(
            widget::scrollable(widget::column![
                widget::space().height(400),
                progress_view(circular, 0.0, true, false)
            ]),
            Size::new(320.0, 90.0),
            Theme::light(),
        );
        ui.at(0);
        ui.frame();
        assert_eq!(ui.at(1000), window::RedrawRequest::Wait);
        let mut theme = Theme::light();
        theme.motion.medium = std::time::Duration::ZERO;
        let mut ui = Harness::new(
            progress_view(circular, 0.0, true, false),
            Size::new(320.0, 90.0),
            theme,
        );
        ui.frame();
        assert_eq!(ui.at(0), window::RedrawRequest::Wait);
        let still = ui.frame();
        ui.at(5000);
        assert!(ui.frame() == still);
    }
}

#[test]
fn indeterminate_progress_replays_identical_frames_with_different_clock_origins() {
    for circular in [false, true] {
        let render = || {
            let mut ui = Harness::new(
                progress_view(circular, 0.0, true, false),
                Size::new(320.0, 90.0),
                Theme::light(),
            );
            [0, 167, 333, 667, 1333, 2000]
                .into_iter()
                .map(|ms| {
                    ui.at(ms);
                    ui.frame()
                })
                .collect::<Vec<_>>()
        };
        assert!(render() == render());
    }
}

#[test]
fn linear_indeterminate_pixels_grow_contract_and_handoff_between_segments() {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    for dark in [false, true] {
        let mut full = Harness::with_backend(
            progress_view(false, 1.0, false, false),
            Size::new(320.0, 90.0),
            theme(dark),
            &backend,
        );
        let solid = full.frame();
        let index = ((36 * solid.width + 40) * 4) as usize;
        let ink = &solid.pixels[index..index + 4];
        let runs = |frame: &reference::Image| {
            let mut widths = Vec::new();
            let mut width = 0;
            for x in 32..=608 {
                let index = ((36 * frame.width + x) * 4) as usize;
                if &frame.pixels[index..index + 4] == ink {
                    width += 1;
                } else if width > 0 {
                    widths.push(width);
                    width = 0;
                }
            }
            widths
        };
        let mut ui = Harness::with_backend(
            progress_view(false, 0.0, true, false),
            Size::new(320.0, 90.0),
            theme(dark),
            &backend,
        );
        ui.at(0);
        ui.frame();
        ui.at(200);
        let growing = runs(&ui.frame());
        assert_eq!(
            growing.len(),
            1,
            "only the leading segment has entered this early"
        );
        ui.at(750);
        let expanded = runs(&ui.frame());
        assert_eq!(expanded.len(), 1);
        assert!(
            expanded[0] > 3 * growing[0],
            "the visible segment must stretch substantially"
        );
        ui.at(1100);
        let contracting = runs(&ui.frame());
        assert_eq!(contracting.len(), 1);
        assert!(
            contracting[0] < expanded[0] / 2,
            "its tail must catch up before it exits"
        );
        ui.at(1200);
        let handoff = runs(&ui.frame());
        assert_eq!(
            handoff.len(),
            2,
            "the outgoing and incoming segments briefly coexist"
        );
        ui.at(1400);
        assert_eq!(runs(&ui.frame()).len(), 1);
        ui.at(1950);
        assert!(
            runs(&ui.frame()).is_empty(),
            "both segments exit before the cycle restarts"
        );
        ui.at(2200);
        assert_eq!(runs(&ui.frame()), growing);
    }

    // Optional motion review artifact; the images come from the real renderer.
    if std::env::var_os("WRITE_PROGRESS_PREVIEW").is_some() {
        let root = std::path::Path::new("target/visual-report/progress-motion");
        for dark in [false, true] {
            let mut ui = Harness::new(
                progress_view(false, 0.0, true, false),
                Size::new(320.0, 90.0),
                theme(dark),
            );
            ui.at(0);
            ui.frame();
            for index in 0..60 {
                ui.at(index * 2000 / 60);
                ui.frame().crop(0, 16, 640, 40).write(&root.join(format!(
                    "{}/{index:02}.png",
                    if dark { "dark" } else { "light" }
                )));
            }
        }
        std::fs::write(
            root.join("index.html"),
            include_str!("progress-preview.html"),
        )
        .unwrap();
    }
}

#[test]
fn value_controls_render_correctly_under_scroll_translation() {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    let scene = || {
        widget::column![
            slider_view(50.0, true, false),
            segments(SegmentSelection::Single(Some(0)), false),
            progress_view(true, 0.75, false, false)
        ]
        .width(340)
    };
    let mut original =
        Harness::with_backend(scene(), Size::new(360.0, 300.0), Theme::dark(), &backend);
    original.at(0);
    let expected = original.frame();
    let mut scrolled = Harness::with_backend(
        widget::scrollable(widget::column![widget::space().height(120), scene()]),
        Size::new(360.0, 300.0),
        Theme::dark(),
        &backend,
    );
    scrolled.at(0);
    scrolled.frame();
    scrolled.move_to((300.0, 100.0));
    scrolled.event(Event::Mouse(mouse::Event::WheelScrolled {
        delta: mouse::ScrollDelta::Pixels { x: 0.0, y: -60.0 },
    }));
    scrolled.leave();
    scrolled.at(300);
    // The 120px spacer minus the 60px scroll leaves the controls at y=60.
    // Exclude the native scrollbar and bottom clip from the comparison.
    assert!(expected.crop(0, 0, 650, 450) == scrolled.frame().crop(0, 120, 650, 450));
}

#[test]
fn root_dialog_blocks_slider_and_segmented_actions() {
    let background = || {
        widget::column![
            slider_view(50.0, true, false),
            segments(SegmentSelection::Single(Some(0)), false)
        ]
    };
    let mut ui = Harness::new(
        crate::dialog::modal(
            background(),
            crate::dialog(typography("Modal", TypeScale::TitleLarge)),
            true,
        ),
        Size::new(400.0, 500.0),
        Theme::light(),
    );
    ui.move_to((200.0, 72.0));
    ui.down();
    ui.move_to((240.0, 72.0));
    ui.up();
    ui.move_to((170.0, 132.0));
    ui.down();
    ui.up();
    assert!(ui.messages.is_empty());
}

#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_value_controls() {
    for dark in [false, true] {
        for discrete in [false, true] {
            let case = themed_name(
                if discrete {
                    "slider-stepped"
                } else {
                    "slider-continuous"
                },
                dark,
            );
            let mut ui = Harness::new(
                slider_view(50.0, discrete, false),
                Size::new(320.0, 120.0),
                theme(dark),
            );
            ui.at(0);
            capture(&mut ui, &case, "00-idle");
            ui.move_to((160.0, 72.0));
            ui.at(150);
            capture(&mut ui, &case, "01-hover");
            ui.down();
            for ms in [0, 75, 150] {
                ui.at(150 + ms);
                capture(&mut ui, &case, &format!("02-press-{ms:03}ms"));
            }
            let held = ui.frame();
            assert_eq!(ui.at(1000), window::RedrawRequest::Wait);
            assert!(ui.frame() == held);
            ui.move_to((232.0, 72.0));
            assert_eq!(ui.messages, [Message::Value(80.0)]);
            ui.rebuild(slider_view(80.0, discrete, false));
            ui.at(1050);
            capture(&mut ui, &case, "03-held-mid-drag");
            ui.up();
            ui.at(1125);
            capture(&mut ui, &case, "04-release-75ms");
            ui.leave();
            ui.at(1500);
            capture(&mut ui, &case, "05-settled");
            for value in [0, 100] {
                ui.rebuild(slider_view(value as f32, discrete, false));
                ui.at(1600 + value as u64);
                capture(&mut ui, &case, &format!("06-endpoint-{value:03}"));
            }
            ui.rebuild(slider_view(50.0, discrete, true));
            ui.at(2000);
            capture(&mut ui, &case, "07-disabled");
        }
        for multiple in [false, true] {
            let case = themed_name(
                if multiple {
                    "segments-multiple"
                } else {
                    "segments-single"
                },
                dark,
            );
            let initial = if multiple {
                SegmentSelection::Multiple(vec![0])
            } else {
                SegmentSelection::Single(Some(0))
            };
            let mut ui = Harness::new(
                segments(initial, false),
                Size::new(340.0, 90.0),
                theme(dark),
            );
            ui.at(0);
            capture(&mut ui, &case, "00-idle");
            let p = ui.find("JPEG").center();
            ui.move_to(p);
            ui.at(150);
            capture(&mut ui, &case, "01-hover");
            ui.down();
            ui.at(300);
            capture(&mut ui, &case, "02-held");
            ui.up();
            let next = if multiple {
                SegmentSelection::Multiple(vec![0, 1])
            } else {
                SegmentSelection::Single(Some(1))
            };
            assert_eq!(ui.messages, [Message::Segments(next.clone())]);
            ui.rebuild(segments(next.clone(), false));
            for ms in [0, 75, 200] {
                ui.at(300 + ms);
                capture(&mut ui, &case, &format!("03-selection-{ms:03}ms"));
            }
            ui.leave();
            ui.at(800);
            capture(&mut ui, &case, "04-settled");
            assert_eq!(ui.at(1000), window::RedrawRequest::Wait);
            ui.rebuild(segments(next, true));
            ui.at(1200);
            capture(&mut ui, &case, "05-disabled");
        }
    }
}

#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_progress() {
    for dark in [false, true] {
        for circular in [false, true] {
            let case = themed_name(
                if circular {
                    "progress-circular"
                } else {
                    "progress-linear"
                },
                dark,
            );
            let mut ui = Harness::new(
                progress_view(circular, 0.0, false, false),
                Size::new(320.0, 90.0),
                theme(dark),
            );
            ui.at(0);
            capture(&mut ui, &case, "00-empty");
            ui.rebuild(progress_view(circular, 0.75, false, false));
            for ms in [0, 50, 100, 200] {
                ui.at(ms);
                capture(&mut ui, &case, &format!("01-determinate-{ms:03}ms"));
            }
            assert_eq!(ui.at(2000), window::RedrawRequest::Wait);
            ui.rebuild(progress_view(circular, 1.0, false, false));
            ui.at(2100);
            ui.at(4000);
            capture(&mut ui, &case, "02-complete");
            ui.rebuild(progress_view(circular, 0.0, true, false));
            let times: &[u64] = if circular {
                &[0, 167, 333, 667, 1000, 1333, 1667, 2000, 2666]
            } else {
                &[
                    0, 167, 333, 667, 1000, 1100, 1150, 1200, 1250, 1333, 1500, 1667, 1750, 1950,
                    2000, 2666,
                ]
            };
            for &ms in times {
                ui.at(4100 + ms);
                capture(&mut ui, &case, &format!("03-indeterminate-{ms:04}ms"));
            }
            ui.rebuild(progress_view(circular, 0.0, true, true));
            assert_eq!(ui.at(7000), window::RedrawRequest::Wait);
            capture(&mut ui, &case, "04-paused");
        }
    }
}
