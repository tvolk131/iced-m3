use super::{Harness, reference};
use crate::{Element, MotionScheme, Theme, switch};
use iced::{Color, Size, keyboard, widget, window};

fn control(checked: bool, icons: bool, disabled: bool) -> Element<'static, bool> {
    widget::container(
        switch(checked)
            .icons(icons)
            .on_toggle(|v| v)
            .disabled(disabled),
    )
    .padding(16)
    .into()
}

fn probe_theme(dark: bool, reduced: bool) -> Theme {
    let mut theme = if dark { Theme::dark() } else { Theme::light() }
        .expressive()
        .reduced_motion(reduced);
    // A constant black thumb isolates geometry from changing theme colors.
    // The thin unchecked outline is also black; the longest scanline run
    // identifies the thumb rather than the separate track border.
    theme.colors.outline = Color::BLACK;
    theme.colors.on_surface_variant = Color::BLACK;
    theme.colors.on_primary = Color::BLACK;
    theme.colors.primary_container = Color::BLACK;
    theme.colors.surface = Color::WHITE;
    theme.colors.surface_container_highest = Color::from_rgb(0.5, 0.5, 0.5);
    theme.colors.primary = Color::WHITE;
    theme
}

fn harness(checked: bool, theme: Theme) -> Harness<'static, bool> {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    let mut ui = Harness::with_backend(
        control(checked, false, false),
        Size::new(100., 80.),
        theme,
        &backend,
    );
    ui.frame();
    ui.at(0);
    ui
}

fn thumb(ui: &mut Harness<'_, bool>) -> (f32, f32) {
    let image = ui.frame();
    let (mut longest, mut run) = ((0, 0), (0, 0));
    for x in 0..image.width {
        let i = ((80 * image.width + x) * 4) as usize;
        if image.pixels[i..i + 3].iter().all(|&v| v < 64) {
            if run.1 == 0 {
                run.0 = x;
            }
            run.1 += 1;
            if run.1 > longest.1 {
                longest = run;
            }
        } else {
            run.1 = 0;
        }
    }
    // A held unchecked thumb touches its 2px outline at the left edge. Exclude
    // that border if it joins the run; selected overshoot has no such outline.
    if longest.0 == 48 {
        longest.0 += 4;
        longest.1 -= 4;
    }
    assert!(longest.1 >= 20, "thumb must remain visible");
    (
        (2 * longest.0 + longest.1) as f32 / 4.,
        longest.1 as f32 / 2.,
    )
}

#[test]
fn switch_spring_painted_thumb_overshoots_in_both_directions() {
    for dark in [false, true] {
        for initial in [false, true] {
            let mut ui = harness(initial, probe_theme(dark, false));
            let target = if initial { 40. } else { 60. };
            let direction = if initial { -1. } else { 1. };
            ui.rebuild(control(!initial, false, false));
            ui.at(0);
            ui.at(140);
            let (center, diameter) = thumb(&mut ui);
            assert!(
                (center - target) * direction > 1.,
                "painted overshoot missing: {center} vs {target}"
            );
            assert!((center - target) * direction < 2.5);
            if initial {
                assert!(diameter < 16.);
            } else {
                assert!(diameter > 24.);
            }
            ui.at(2000);
            assert!((thumb(&mut ui).0 - target).abs() <= 0.5);
            assert_eq!(ui.at(2016), window::RedrawRequest::Wait);
            assert!(ui.messages.is_empty());
        }
    }
}

#[test]
fn switch_spring_press_snaps_and_cancel_release_rebounds() {
    for initial in [false, true] {
        let mut ui = harness(initial, probe_theme(false, false));
        ui.move_to((50., 40.));
        ui.down();
        let pressed = thumb(&mut ui);
        assert!(
            (pressed.1 - 28.).abs() <= 0.5,
            "pressed size must be immediate: {pressed:?}"
        );
        ui.at(500);
        assert!(ui.messages.is_empty(), "holding must not toggle");
        ui.leave(); // Cancel, without changing the controlled selection.
        ui.up();
        ui.at(640);
        let idle = if initial { 24. } else { 16. };
        assert!(
            thumb(&mut ui).1 < idle - 0.25,
            "release should undershoot resting size"
        );
        ui.at(2500);
        assert!((thumb(&mut ui).1 - idle).abs() <= 0.5);
        assert_eq!(ui.at(2516), window::RedrawRequest::Wait);
        assert!(ui.messages.is_empty());
    }
}

#[test]
fn switch_spring_reversal_preserves_position_and_momentum() {
    let mut ui = harness(false, probe_theme(false, false));
    ui.rebuild(control(true, false, false));
    ui.at(0);
    ui.at(40);
    let before = thumb(&mut ui);
    ui.rebuild(control(false, false, false));
    ui.at(40);
    assert_eq!(thumb(&mut ui), before, "retargeting must not jump");
    ui.at(50);
    assert!(
        thumb(&mut ui).0 > before.0,
        "outgoing velocity must survive reversal"
    );
    ui.at(2000);
    assert!((thumb(&mut ui).0 - 40.).abs() <= 0.5);
    assert_eq!(ui.at(2016), window::RedrawRequest::Wait);
}

#[test]
fn switch_spring_reduced_motion_and_keyboard_release_respect_state() {
    for reduced in [false, true] {
        let mut ui = harness(false, probe_theme(false, reduced));
        // Focus with pointer input, then activate through the keyboard.
        ui.move_to((50., 40.));
        ui.down();
        ui.leave();
        ui.up();
        for event in iced_test::simulator::tap_key(keyboard::key::Named::Space, None) {
            ui.event(event);
        }
        assert_eq!(ui.messages, [true]);
        ui.rebuild(control(true, false, false));
        ui.at(0);
        if reduced {
            assert!((thumb(&mut ui).0 - 60.).abs() <= 0.5);
        }
        ui.at(140);
        if reduced {
            assert!((thumb(&mut ui).0 - 60.).abs() <= 0.5);
        } else {
            assert!(thumb(&mut ui).0 > 61.);
        }
        ui.rebuild(control(true, false, true));
        ui.at(140);
        ui.at(2000);
        assert_eq!(ui.at(2016), window::RedrawRequest::Wait);
    }
}

#[test]
fn switch_spring_colors_use_bounded_effects_timing() {
    let mut ui = harness(false, probe_theme(false, false));
    ui.rebuild(control(true, false, false));
    ui.at(0);
    let mut previous = 0;
    for ms in [0, 20, 40, 80, 140, 200, 500] {
        ui.at(ms);
        let image = ui.frame();
        // Inside the left end of the track, beyond its outline and outside
        // the unpressed thumb's complete path, including its size overshoot.
        let i = ((80 * image.width + 56) * 4) as usize;
        let color = image.pixels[i];
        assert!(
            color >= previous,
            "effects must not bounce back with geometry"
        );
        assert_eq!(&image.pixels[i..i + 4], &[color, color, color, 255]);
        if ms == 40 {
            assert!(
                (212..=222).contains(&color),
                "FastEffects response, got {color}"
            );
        }
        previous = color;
    }
    assert_eq!(previous, 255);
}

#[test]
fn switch_spring_painted_response_is_independent_of_frame_cadence() {
    let mut reference = None;
    for cadence in [8, 16, 33, 140] {
        let mut ui = harness(false, probe_theme(false, false));
        ui.rebuild(control(true, false, false));
        ui.at(0);
        for ms in (cadence..140).step_by(cadence as usize) {
            ui.at(ms);
        }
        ui.at(140);
        let frame = ui.frame();
        if let Some(expected) = &reference {
            assert!(expected == &frame);
        } else {
            reference = Some(frame);
        }
    }
}

#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_switch_spring_motion() {
    for dark in [false, true] {
        for icons in [false, true] {
            for initial in [false, true] {
                for standard in [false, true] {
                    let theme = if dark { Theme::dark() } else { Theme::light() }
                        .expressive()
                        .motion_scheme(if standard {
                            MotionScheme::standard()
                        } else {
                            MotionScheme::expressive()
                        });
                    let name = format!(
                        "switch-spring/{}-{}-{}-{}",
                        if dark { "dark" } else { "light" },
                        if icons { "icons" } else { "plain" },
                        if initial { "on" } else { "off" },
                        if standard { "standard" } else { "expressive" }
                    );
                    let mut ui =
                        Harness::new(control(initial, icons, false), Size::new(100., 80.), theme);
                    reference::check(&format!("{name}/00-idle"), &ui.frame());
                    ui.at(0);
                    ui.move_to((50., 40.));
                    ui.down();
                    reference::check(&format!("{name}/01-pressed"), &ui.frame());
                    ui.at(500);
                    reference::check(&format!("{name}/02-held"), &ui.frame());
                    assert!(ui.messages.is_empty());
                    ui.up();
                    assert_eq!(ui.messages, [!initial]);
                    ui.rebuild(control(!initial, icons, false));
                    ui.at(500);
                    for ms in [540, 600, 640, 740, 1500] {
                        ui.at(ms);
                        reference::check(
                            &format!("{name}/03-release-{:04}", ms - 500),
                            &ui.frame(),
                        );
                    }
                    ui.leave();
                    ui.at(2500);
                    assert_eq!(ui.at(2516), window::RedrawRequest::Wait);
                }
            }
        }
    }
}
