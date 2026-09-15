use super::{Message, harness::Harness, reference, theme};
use crate::{
    Element, Theme, TypeScale, circular_progress, linear_progress, loading_indicator, typography,
};
use iced::{Color, Event, Length, Size, widget, window};

fn host(content: impl Into<Element<'static, Message>>) -> Element<'static, Message> {
    widget::stack![
        widget::row((0..20).map(|i| {
            widget::container(widget::space())
                .width(40)
                .height(Length::Fill)
                .style(move |t: &Theme| widget::container::Style {
                    background: Some(
                        if i % 2 == 0 {
                            t.colors.surface_container_highest
                        } else {
                            t.colors.secondary_container
                        }
                        .into(),
                    ),
                    ..Default::default()
                })
                .into()
        })),
        widget::container(content).padding(20),
    ]
    .into()
}
fn ui(
    content: impl Into<Element<'static, Message>>,
    size: Size,
    dark: bool,
) -> Harness<'static, Message> {
    Harness::with_backend(
        host(content),
        size,
        theme(dark),
        &std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into()),
    )
}
fn progress(
    circular: bool,
    value: f32,
    loading: bool,
    paused: bool,
    speed: f32,
) -> crate::Progress {
    (if circular {
        circular_progress(value).size(80.)
    } else {
        linear_progress(value).width(280)
    })
    .wavy(true)
    .indeterminate(loading)
    .paused(paused)
    .wave_speed(speed)
}
fn gallery(value: f32, loading: bool, paused: bool) -> Element<'static, Message> {
    widget::column![
        typography("Loading and progress", TypeScale::TitleLarge),
        widget::row![
            loading_indicator().paused(paused),
            loading_indicator().contained(true).paused(paused)
        ]
        .spacing(24),
        typography(
            "Measured · wave amplitude settles near the ends",
            TypeScale::BodyMedium
        ),
        progress(false, value, false, paused, 0.).buffer((value + 0.2).min(1.)),
        widget::row![
            progress(true, value, false, paused, 0.),
            progress(true, value, false, paused, 12.)
        ]
        .spacing(24),
        typography(
            "Loading · moving endpoints and a visible track",
            TypeScale::BodyMedium
        ),
        progress(false, value, loading, paused, 0.),
        progress(true, value, loading, paused, 0.),
    ]
    .spacing(20)
    .into()
}

#[test]
fn canonical_curves_match_independent_androidx_shapes_and_morph_midpoints() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/loading/reference");
    for (i, name) in [
        "soft-burst",
        "cookie-9",
        "pentagon",
        "pill",
        "sunny",
        "cookie-4",
        "oval",
    ]
    .iter()
    .enumerate()
    {
        for (amount, suffix) in [(0., ""), (0.5, "-midpoint")] {
            let rendered =
                |handle| ui(crate::icon(handle).size(144), Size::new(184., 184.), false).frame();
            let actual = rendered(crate::loading::reference_shape(i, amount));
            let expected = rendered(widget::svg::Handle::from_memory(
                std::fs::read(root.join(format!("{name}{suffix}.svg"))).unwrap(),
            ));
            let errors: Vec<_> = actual
                .pixels
                .iter()
                .zip(&expected.pixels)
                .map(|(a, b)| a.abs_diff(*b))
                .filter(|e| *e > 0)
                .collect();
            let artifacts = std::path::PathBuf::from("target/loading-progress-source-check");
            actual.write(&artifacts.join(format!("{name}{suffix}-actual.png")));
            expected.write(&artifacts.join(format!("{name}{suffix}-expected.png")));
            // Morph splits the source curves before matching. The software SVG
            // flattener can rasterize split cubics differently at the boundary.
            // Require the same interior and a <=1 physical-pixel contour shift,
            // with at least 99.5% of channels unchanged. This tolerance is only
            // for independently segmented source paths, never golden updates.
            assert!(
                errors.len() < actual.pixels.len() / 200,
                "canonical coverage {name}{suffix}: {} channels, max {:?}, total {}",
                errors.len(),
                errors.iter().max(),
                errors.iter().map(|e| *e as usize).sum::<usize>()
            );
            for (pixel, (a, b)) in actual
                .pixels
                .as_chunks::<4>()
                .0
                .iter()
                .zip(expected.pixels.as_chunks::<4>().0.iter())
                .enumerate()
            {
                if a == b {
                    continue;
                }
                let x = pixel as u32 % actual.width;
                let y = pixel as u32 / actual.width;
                for (channel, value) in a.iter().enumerate().take(3) {
                    let mut low = 255u8;
                    let mut high = 0u8;
                    for ny in y.saturating_sub(1)..=(y + 1).min(actual.height - 1) {
                        for nx in x.saturating_sub(1)..=(x + 1).min(actual.width - 1) {
                            let value =
                                expected.pixels[((ny * actual.width + nx) * 4) as usize + channel];
                            low = low.min(value);
                            high = high.max(value);
                        }
                    }
                    assert!(
                        *value >= low.saturating_sub(2) && *value <= high.saturating_add(2),
                        "canonical contour {name}{suffix}"
                    );
                }
            }
        }
    }
}

#[test]
fn wavy_geometry_reserves_space_preserves_parent_and_has_visible_gaps() {
    for dark in [false, true] {
        let size = Size::new(320., 160.);
        let empty = ui(widget::space(), size, dark).frame();
        let mut view = ui(
            widget::column![
                progress(false, 0.5, false, false, 0.),
                progress(true, 0.5, false, false, 0.)
            ]
            .spacing(20),
            size,
            dark,
        );
        view.at(0);
        let frame = view.frame();
        assert!(
            frame.crop(40, 32, 560, 8) == empty.crop(40, 32, 560, 8),
            "no pixels above allocated wave height"
        );
        assert!(
            frame.crop(40, 40, 275, 4) != empty.crop(40, 40, 275, 4),
            "active peaks use extra height"
        );
        assert!(
            frame.crop(40, 57, 275, 3) != empty.crop(40, 57, 275, 3),
            "troughs stay inside extra height"
        );
        assert!(
            frame.crop(325, 40, 2, 20) == empty.crop(325, 40, 2, 20),
            "gap after active half"
        );
        assert!(
            frame.crop(370, 40, 80, 4) == empty.crop(370, 40, 80, 4),
            "inactive track stays flat"
        );
        assert!(view.messages.is_empty());
    }
}

#[test]
fn loading_and_waves_pause_resume_and_obey_reduced_motion() {
    let mut flat = ui(
        linear_progress(0.5)
            .wavy(true)
            .wave_amplitude(0.)
            .wave_speed(20.),
        Size::new(320., 80.),
        false,
    );
    flat.frame();
    assert_eq!(flat.at(1000), window::RedrawRequest::Wait);
    for circular in [false, true] {
        for loading in [false, true] {
            let mut view = ui(
                progress(circular, 0.5, loading, false, 20.),
                Size::new(320., 140.),
                true,
            );
            view.at(0);
            view.frame();
            view.at(480);
            let moving = view.frame();
            view.rebuild(host(progress(circular, 0.5, loading, true, 20.)));
            assert_eq!(view.at(500), window::RedrawRequest::Wait);
            assert!(moving == view.frame());
            view.at(8000);
            assert!(moving == view.frame());
            view.rebuild(host(progress(circular, 0.5, loading, false, 20.)));
            view.at(8000);
            assert!(moving == view.frame());
            view.at(8300);
            assert!(moving != view.frame());
            view.event(Event::Window(window::Event::Unfocused));
            let frozen = view.frame();
            assert_eq!(view.at(12000), window::RedrawRequest::Wait);
            assert!(frozen == view.frame());
        }
    }
    let mut loader = ui(loading_indicator(), Size::new(100., 100.), false);
    loader.at(0);
    let initial = loader.frame();
    loader.at(420);
    assert!(initial != loader.frame());
    loader.rebuild(host(loading_indicator().paused(true)));
    loader.at(440);
    let frozen = loader.frame();
    assert_eq!(loader.at(6000), window::RedrawRequest::Wait);
    assert!(frozen == loader.frame());
    let mut reduced = Harness::new(
        host(gallery(0.5, true, false)),
        Size::new(400., 520.),
        Theme::dark().reduced_motion(true),
    );
    reduced.frame();
    let initial = reduced.frame();
    reduced.at(0);
    assert_eq!(reduced.at(6000), window::RedrawRequest::Wait);
    assert!(initial == reduced.frame());
}

#[test]
fn wavy_handoff_keeps_the_first_frame_and_finishes_without_idle_redraws() {
    for circular in [false, true] {
        let mut view = ui(
            progress(circular, 0., true, false, 0.),
            Size::new(320., 140.),
            false,
        );
        view.at(0);
        view.frame();
        view.at(450);
        let before = view.frame();
        view.rebuild(host(progress(circular, 0.75, false, false, 0.)));
        view.at(450);
        assert!(before == view.frame());
        for ms in (466..5000).step_by(16) {
            view.at(ms);
            view.frame();
        }
        assert_eq!(view.at(5000), window::RedrawRequest::Wait);
        let expected = ui(
            progress(circular, 0.75, false, false, 0.),
            Size::new(320., 140.),
            false,
        )
        .frame();
        assert!(expected == view.frame());
        view.rebuild(host(progress(circular, 1., false, false, 0.)));
        for ms in (5016..7500).step_by(16) {
            view.at(ms);
        }
        assert_eq!(view.at(7500), window::RedrawRequest::Wait);
        assert!(
            ui(
                progress(circular, 1., false, false, 0.),
                Size::new(320., 140.),
                false
            )
            .frame()
                == view.frame()
        );
    }
}

#[test]
fn wavy_and_loader_frames_ignore_clock_origin_and_survive_overlay_coverage() {
    let content =
        || widget::row![loading_indicator(), progress(true, 0., true, false, 0.)].spacing(24);
    let background =
        || widget::column![content(), progress(false, 0., true, false, 0.)].spacing(20);
    let dialog = || crate::dialog(widget::space().width(80).height(40));
    let mut a = ui(
        crate::dialog::host(background(), dialog(), false),
        Size::new(320., 260.),
        false,
    );
    let mut b = ui(
        crate::dialog::host(background(), dialog(), false),
        Size::new(320., 260.),
        false,
    );
    a.at(0);
    a.frame();
    b.at(5000);
    b.frame();
    a.at(400);
    b.at(5400);
    assert!(a.frame() == b.frame());
    a.rebuild(host(crate::dialog::host(background(), dialog(), true)));
    a.at(1000);
    let first = a.frame();
    a.at(1300);
    let second = a.frame();
    assert!(
        first.crop(40, 40, 280, 120) != second.crop(40, 40, 280, 120),
        "visible covered progress keeps advancing"
    );
    assert!(a.messages.is_empty());
}

#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_loading_progress() {
    for dark in [false, true] {
        let mode = if dark { "dark" } else { "light" };
        let mut view = Harness::new(
            host(gallery(0.5, true, false)),
            Size::new(400., 520.),
            theme(dark),
        );
        view.at(0);
        view.frame();
        for ms in [
            0, 100, 325, 649, 650, 975, 1300, 1800, 1950, 2600, 3000, 3250, 3900, 4550, 5200, 5999,
            6000,
        ] {
            view.at(ms);
            reference::check(
                &format!("loading-progress/{mode}/motion-{ms:04}"),
                &view.frame(),
            );
        }
        for value in [0., 0.05, 0.1, 0.5, 0.9, 0.95, 1.] {
            let mut view = Harness::new(
                host(gallery(value, false, true)),
                Size::new(400., 520.),
                theme(dark),
            );
            view.at(0);
            reference::check(
                &format!("loading-progress/{mode}/value-{:03}", (value * 100.) as u8),
                &view.frame(),
            );
        }
        for circular in [false, true] {
            let view = |loading| host(progress(circular, 0.75, loading, false, 0.));
            let mut ui = Harness::new(view(true), Size::new(320., 140.), theme(dark));
            ui.at(0);
            ui.frame();
            ui.at(450);
            ui.rebuild(view(false));
            for ms in [0, 125, 250, 500, 750, 1350, 1800, 2500] {
                ui.at(450 + ms);
                reference::check(
                    &format!("loading-progress-handoff/{mode}/circular-{circular}-{ms:04}"),
                    &ui.frame(),
                );
            }
        }
        let mut view = Harness::new(
            host(
                widget::column![
                    linear_progress(0.65)
                        .wavy(true)
                        .size(8.)
                        .wave_amplitude(6.)
                        .wavelength(32.)
                        .buffer(0.9)
                        .width(280),
                    circular_progress(0.6)
                        .wavy(true)
                        .size(96.)
                        .wave_amplitude(4.)
                        .wavelength(30.)
                        .colors([Color::from_rgb8(40, 145, 130)]),
                ]
                .spacing(24),
            ),
            Size::new(320., 190.),
            theme(dark),
        );
        view.at(0);
        reference::check(
            &format!("loading-progress/{mode}/custom-wave"),
            &view.frame(),
        );
    }
}

#[test]
#[ignore = "release rendering cost comparison; includes screenshot readback"]
fn profile_loading_progress() {
    for name in [
        "baseline-linear",
        "wavy-linear",
        "baseline-circular",
        "wavy-circular",
        "canonical-loading",
    ] {
        let content: Element<'static, Message> = match name {
            "baseline-linear" => linear_progress(0.).width(280).indeterminate(true).into(),
            "wavy-linear" => progress(false, 0., true, false, 0.).into(),
            "baseline-circular" => circular_progress(0.).size(80.).indeterminate(true).into(),
            "wavy-circular" => progress(true, 0., true, false, 0.).into(),
            _ => loading_indicator().size(80.).into(),
        };
        let mut view = ui(content, Size::new(320., 140.), true);
        view.at(0);
        view.frame();
        let mut times = Vec::new();
        for ms in (16..=960).step_by(16) {
            let start = std::time::Instant::now();
            view.at(ms);
            view.frame();
            times.push(start.elapsed().as_secs_f64() * 1000.);
        }
        times.sort_by(f64::total_cmp);
        eprintln!(
            "{name}: mean {:.2}ms p95 {:.2}ms (60 changing frames, 2x, includes screenshot readback)",
            times.iter().sum::<f64>() / times.len() as f64,
            times[times.len() * 95 / 100]
        );
    }
}

#[test]
#[ignore = "generates the 30fps local motion review; not a golden comparison"]
fn capture_loading_progress_preview() {
    let root = std::path::PathBuf::from("target/visual-report/loading-progress-motion/frames");
    for dark in [false, true] {
        let mode = if dark { "dark" } else { "light" };
        let mut view = Harness::new(
            host(gallery(0.5, true, false)),
            Size::new(400., 520.),
            theme(dark),
        );
        view.at(0);
        view.frame();
        for frame in 0..=180 {
            view.at(frame * 1000 / 30);
            view.frame()
                .write(&root.join(format!("{mode}-{frame:03}.png")));
        }
    }
}
