use super::harness::Harness;
use crate::{Element, Theme, button, button_group, focus};
use iced::{Length, Rectangle, Size, advanced::widget::Operation, widget};

fn group() -> Element<'static, u8> {
    focus::scope(
        widget::container(button_group([
            button("One").on_press(1),
            button("Two").on_press(2),
            button("Three").on_press(3),
        ]))
        .width(Length::Fill)
        .padding(16),
    )
}

fn bounds(ui: &mut Harness<'_, u8>) -> Vec<Rectangle> {
    #[derive(Default)]
    struct Targets(Vec<Rectangle>);
    impl Operation for Targets {
        fn traverse(&mut self, f: &mut dyn FnMut(&mut dyn Operation)) {
            f(self);
        }
        fn focusable(
            &mut self,
            _: Option<&widget::Id>,
            bounds: Rectangle,
            _: &mut dyn iced::advanced::widget::operation::Focusable,
        ) {
            self.0.push(bounds);
        }
    }
    let mut targets = Targets::default();
    ui.operate(&mut targets);
    targets.0
}

#[test]
fn reduced_motion_round_trip_preserves_application_timings() {
    let mut theme = Theme::dark();
    theme.motion.short = std::time::Duration::from_millis(271);
    theme.motion.dialog_enter = std::time::Duration::from_millis(613);
    let original = theme.clone();
    assert_eq!(theme.reduced_motion(true).reduced_motion(false), original);
}

#[test]
fn pressed_group_button_expands_and_only_its_neighbors_compress() {
    let mut ui = Harness::new(group(), Size::new(500., 100.), Theme::light());
    ui.frame();
    let before = bounds(&mut ui);
    ui.move_to(before[0].center());
    ui.down();
    ui.at(1000);
    ui.frame();
    let held = bounds(&mut ui);
    assert!(
        held[0].width > before[0].width + 1.,
        "pressed button must expand"
    );
    assert!(
        held[1].width < before[1].width - 1.,
        "its neighbor must compress"
    );
    assert!((held[2].width - before[2].width).abs() < 0.01);
    assert!(
        (held.iter().map(|b| b.width).sum::<f32>() - before.iter().map(|b| b.width).sum::<f32>())
            .abs()
            < 0.01
    );
    ui.up();
    ui.at(2000);
    assert_eq!(ui.messages, [1]);
    for (a, b) in bounds(&mut ui).iter().zip(before) {
        assert!((a.width - b.width).abs() < 0.01);
    }
}

fn host(content: impl Into<Element<'static, u8>>) -> Element<'static, u8> {
    focus::scope(widget::container(content).padding(16))
}
fn action(shape: Option<bool>, size: crate::ButtonSize, selected: bool) -> Element<'static, u8> {
    let mut action = button("Action").size(size).selected(selected).on_press(1);
    if let Some(shape) = shape {
        action = action.expressive(shape);
    }
    host(action)
}

#[test]
fn expressive_theme_matches_explicit_feedback_and_respects_opt_out() {
    let size = Size::new(250., 120.);
    let theme = Theme::light().expressive();
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    let mut inherited = Harness::with_backend(
        action(None, crate::ButtonSize::Medium, false),
        size,
        theme.clone(),
        &backend,
    );
    let mut explicit = Harness::with_backend(
        action(Some(true), crate::ButtonSize::Medium, false),
        size,
        theme.clone(),
        &backend,
    );
    let mut disabled = Harness::with_backend(
        action(Some(false), crate::ButtonSize::Medium, false),
        size,
        theme,
        &backend,
    );
    for ui in [&mut inherited, &mut explicit, &mut disabled] {
        ui.frame();
        let center = bounds(ui)[0].center();
        ui.move_to(center);
        ui.down();
        ui.at(1000);
    }
    let inherited = inherited.frame();
    assert!(inherited == explicit.frame());
    assert!(inherited != disabled.frame());
}

#[test]
fn size_recipes_preserve_label_roles_and_icon_container_dimensions() {
    use crate::{ButtonSize::*, IconButtonWidth::*};
    for (size, height, widths) in [
        (ExtraSmall, 32., [28., 32., 40.]),
        (Small, 40., [32., 40., 52.]),
        (Medium, 56., [48., 56., 72.]),
        (Large, 96., [64., 96., 128.]),
        (ExtraLarge, 136., [104., 136., 184.]),
    ] {
        let mut ui = Harness::new(
            action(None, size, false),
            Size::new(600., 200.),
            Theme::light(),
        );
        assert_eq!(bounds(&mut ui)[0].height, height);
        assert_eq!(ui.find("Action").height, size.label().metrics().1);
        for (width, expected) in [Narrow, Default, Wide].into_iter().zip(widths) {
            for first in [true, false] {
                let icon = crate::icon_button(
                    widget::space()
                        .width(size.icon_button_icon_size())
                        .height(size.icon_button_icon_size()),
                )
                .on_press(1);
                let icon = if first {
                    icon.icon_width(width).size(size)
                } else {
                    icon.size(size).icon_width(width)
                };
                let mut ui = Harness::new(host(icon), Size::new(300., 200.), Theme::light());
                let b = bounds(&mut ui)[0];
                assert_eq!((b.width, b.height), (expected, height));
            }
        }
    }
}

#[test]
fn reduced_motion_wins_over_local_springs_and_group_expansion_is_immediate() {
    let mut theme = Theme::dark().expressive();
    theme.motion.short = std::time::Duration::from_millis(437);
    let theme = theme
        .reduced_motion(true)
        .motion_scheme(crate::MotionScheme::expressive());
    assert!(theme.effective_motion().short.is_zero());
    assert_eq!(
        theme.clone().reduced_motion(false).motion.short.as_millis(),
        437
    );
    let mut ui = Harness::new(group(), Size::new(500., 100.), theme);
    ui.frame();
    let before = bounds(&mut ui);
    ui.move_to(before[1].center());
    ui.down();
    let held = bounds(&mut ui);
    assert!(held[1].width > before[1].width);
    assert!(held[0].width < before[0].width && held[2].width < before[2].width);
    let still = ui.frame();
    ui.at(500);
    assert!(still == ui.frame());
    ui.up();
    assert_eq!(bounds(&mut ui), before);
}

#[test]
fn reversing_group_motion_keeps_focus_and_disabled_press_never_activates() {
    let mut ui = Harness::new(group(), Size::new(500., 100.), Theme::light().expressive());
    ui.frame();
    let original = bounds(&mut ui);
    ui.move_to(original[0].center());
    ui.down();
    ui.at(45);
    let held = bounds(&mut ui);
    ui.up();
    assert_eq!(held, bounds(&mut ui), "release must not jump geometry");
    ui.messages.clear();
    ui.down();
    ui.at(60);
    ui.rebuild(focus::scope(
        widget::container(button_group([
            button("One").disabled(true).on_press(1),
            button("Two").on_press(2),
            button("Three").on_press(3),
        ]))
        .width(Length::Fill)
        .padding(16),
    ));
    ui.up();
    ui.at(2000);
    assert!(ui.messages.is_empty());
    assert_eq!(bounds(&mut ui).len(), 2);
}

#[test]
fn empty_single_and_narrow_groups_have_finite_nonnegative_geometry() {
    for count in 0..4 {
        for width in [0., 10., 50., 500.] {
            let content: Element<'_, u8> =
                button_group((0..count).map(|i| button("A").on_press(i))).into();
            let mut ui = Harness::new(content, Size::new(width, 100.), Theme::light().expressive());
            if width > 0. {
                ui.frame();
            }
            for b in bounds(&mut ui) {
                assert!(b.width.is_finite() && b.width >= 0.);
                assert!(b.x + b.width <= width + 0.01);
            }
        }
    }
}

fn key(named: iced::keyboard::key::Named, release: bool) -> iced::Event {
    use iced::keyboard::{
        self,
        key::{NativeCode, Physical},
    };
    let key = keyboard::Key::Named(named);
    if release {
        iced::Event::Keyboard(keyboard::Event::KeyReleased {
            key: key.clone(),
            modified_key: key,
            physical_key: Physical::Unidentified(NativeCode::Unidentified),
            modifiers: keyboard::Modifiers::empty(),
            location: keyboard::Location::Standard,
        })
    } else {
        iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: Physical::Unidentified(NativeCode::Unidentified),
            modifiers: keyboard::Modifiers::empty(),
            location: keyboard::Location::Standard,
            text: None,
            repeat: false,
        })
    }
}

#[test]
fn group_keyboard_press_expands_and_rebuild_retains_active_focus() {
    use iced::keyboard::key::Named;
    let mut ui = Harness::new(group(), Size::new(500., 100.), Theme::light().expressive());
    ui.frame();
    let original = bounds(&mut ui);
    for k in [Named::Tab, Named::ArrowRight] {
        ui.event(key(k, false));
        ui.event(key(k, true));
    }
    ui.event(key(Named::Space, false));
    ui.at(140);
    assert!(bounds(&mut ui)[1].width > original[1].width);
    ui.rebuild(group());
    ui.event(key(Named::Space, true));
    ui.at(1500);
    assert_eq!(ui.messages, [2]);
    assert_eq!(bounds(&mut ui), original);
}

#[test]
fn local_spring_override_has_precedence_over_theme_and_reduced_motion_has_final_say() {
    let render = |theme: Theme, ms| {
        let content = host(
            button("Action")
                .size(crate::ButtonSize::Medium)
                .on_press(1)
                .motion_scheme(crate::MotionScheme::standard()),
        );
        let mut ui = Harness::new(content, Size::new(250., 120.), theme);
        ui.frame();
        let point = bounds(&mut ui)[0].center();
        ui.move_to(point);
        ui.down();
        ui.at(ms);
        ui.frame()
    };
    assert!(
        render(Theme::light().expressive(), 70)
            == render(
                Theme::light()
                    .expressive()
                    .motion_scheme(crate::MotionScheme::standard()),
                70
            )
    );
    assert!(
        render(Theme::light().expressive().reduced_motion(true), 0)
            == render(Theme::light().expressive().reduced_motion(true), 1000)
    );
}

#[test]
fn expressive_dialog_presence_reverses_without_losing_the_host() {
    let content = |open| {
        crate::dialog::modal(
            host(button("Background").on_press(1)),
            crate::dialog(crate::typography(
                "Spring dialog",
                crate::TypeScale::TitleLarge,
            ))
            .actions(button("Close").on_press(2))
            .width(360.),
            open,
        )
    };
    let mut ui = Harness::new(
        content(false),
        Size::new(600., 320.),
        Theme::light().expressive(),
    );
    let closed = ui.frame();
    ui.rebuild(content(true));
    ui.at(0);
    ui.at(70);
    let opening = ui.frame();
    assert!(opening != closed);
    ui.rebuild(content(false));
    ui.at(70);
    assert!(
        ui.frame() == opening,
        "reversal at the same instant must preserve rendered position"
    );
    ui.at(100);
    ui.rebuild(content(true));
    ui.at(100);
    ui.at(2000);
    assert!(ui.frame() != closed);
    ui.rebuild(content(false));
    ui.at(2000);
    ui.at(4000);
    assert!(ui.frame() == closed);
}

#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_expressive_actions() {
    use super::reference;
    use crate::{ButtonShape, ButtonSize, ButtonVariant, IconButtonWidth};
    for dark in [false, true] {
        for standard in [false, true] {
            let theme = if dark { Theme::dark() } else { Theme::light() }
                .expressive()
                .motion_scheme(if standard {
                    crate::MotionScheme::standard()
                } else {
                    crate::MotionScheme::expressive()
                });
            let mode = format!(
                "{}-{}",
                if dark { "dark" } else { "light" },
                if standard { "standard" } else { "expressive" }
            );
            let mut ui = Harness::new(group(), Size::new(500., 100.), theme.clone());
            reference::check(&format!("expressive/{mode}-group-rest"), &ui.frame());
            let center = bounds(&mut ui)[1].center();
            ui.move_to(center);
            ui.down();
            for ms in [0, 45, 100, 180, 400, 1000] {
                ui.at(ms);
                reference::check(&format!("expressive/{mode}-group-held-{ms}"), &ui.frame());
            }
            ui.up();
            for ms in [1045, 1100, 1180, 1400, 2000] {
                ui.at(ms);
                reference::check(
                    &format!("expressive/{mode}-group-release-{ms}"),
                    &ui.frame(),
                );
            }
            if standard {
                continue;
            }
            for size in [
                ButtonSize::ExtraSmall,
                ButtonSize::Small,
                ButtonSize::Medium,
                ButtonSize::Large,
                ButtonSize::ExtraLarge,
            ] {
                let name = format!("{:?}", size).to_lowercase();
                for shape in [ButtonShape::Round, ButtonShape::Square] {
                    let shape_name = format!("{:?}", shape).to_lowercase();
                    let content = || {
                        host(widget::column![
                        widget::row![button("Action").size(size).shape(shape).on_press(1), button("Toggle").size(size).shape(shape).selected(true).on_press(2).variant(ButtonVariant::Outlined)].spacing(20),
                        widget::Row::with_children([IconButtonWidth::Narrow, IconButtonWidth::Default, IconButtonWidth::Wide].map(|width| {
                            crate::icon_button(crate::icon(iced::widget::svg::Handle::from_memory(br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M12 4v16M4 12h16" fill="none" stroke="black" stroke-width="2"/></svg>"#.as_slice())).size(size.icon_button_icon_size()))
                                .size(size).icon_width(width).shape(shape).variant(ButtonVariant::Tonal).on_press(3).into()
                        })).spacing(20),
                    ].spacing(24))
                    };
                    let mut ui = Harness::new(content(), Size::new(950., 350.), theme.clone());
                    reference::check(
                        &format!("expressive/{mode}-{name}-{shape_name}-rest"),
                        &ui.frame(),
                    );
                    let point = bounds(&mut ui)[0].center();
                    ui.move_to(point);
                    ui.down();
                    ui.at(180);
                    reference::check(
                        &format!("expressive/{mode}-{name}-{shape_name}-press"),
                        &ui.frame(),
                    );
                }
            }
            let connected = || {
                host(
                    button_group([
                        button("First").on_press(1),
                        button("Selected").selected(true).on_press(2),
                        button("Last").on_press(3),
                    ])
                    .connected(true),
                )
            };
            let mut ui = Harness::new(connected(), Size::new(500., 100.), theme);
            reference::check(&format!("expressive/{mode}-connected"), &ui.frame());
            let point = bounds(&mut ui)[1].center();
            ui.move_to(point);
            ui.down();
            ui.at(180);
            reference::check(&format!("expressive/{mode}-connected-press"), &ui.frame());
        }
    }
}

fn controls(selected: bool) -> Element<'static, u8> {
    host(
        widget::column![
            crate::switch(selected).on_toggle(|_| 1),
            crate::radio("Radio", true, Some(selected)).on_select(|_| 2),
            crate::text_field("Floating label", if selected { "Value" } else { "" })
                .on_input(|_| 3),
            crate::tabs(
                [
                    crate::Tab::new(false, "First"),
                    crate::Tab::new(true, "Second")
                ],
                Some(selected)
            )
            .on_select(|_| 4),
        ]
        .spacing(24)
        .width(400),
    )
}

#[test]
fn shared_spring_controls_render_through_overshoot_and_reversal() {
    for dark in [false, true] {
        let theme = if dark { Theme::dark() } else { Theme::light() }.expressive();
        let mut ui = Harness::new(controls(false), Size::new(450., 330.), theme);
        let start = ui.frame();
        ui.rebuild(controls(true));
        ui.at(0);
        for ms in [40, 100, 140, 180, 240] {
            ui.at(ms);
            ui.frame();
        }
        ui.rebuild(controls(false));
        ui.at(240);
        for ms in [260, 300, 400, 600, 2000] {
            ui.at(ms);
            ui.frame();
        }
        assert!(ui.frame() == start);
        assert!(ui.messages.is_empty());
    }
}

#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_expressive_controls() {
    for dark in [false, true] {
        let theme = if dark { Theme::dark() } else { Theme::light() }.expressive();
        let name = if dark { "dark" } else { "light" };
        let mut ui = Harness::new(controls(false), Size::new(450., 330.), theme);
        super::reference::check(
            &format!("expressive-controls/{name}/00-default"),
            &ui.frame(),
        );
        ui.rebuild(controls(true));
        ui.at(0);
        for ms in [40, 100, 140, 180, 240, 1000] {
            ui.at(ms);
            super::reference::check(
                &format!("expressive-controls/{name}/01-select-{ms:04}"),
                &ui.frame(),
            );
        }
        ui.rebuild(controls(false));
        ui.at(1000);
        for ms in [1040, 1100, 1180, 1400, 2000] {
            ui.at(ms);
            super::reference::check(
                &format!("expressive-controls/{name}/02-deselect-{ms:04}"),
                &ui.frame(),
            );
        }
    }
}
