//! Sequence contracts measured through iced events and rendered output.
//! These compare geometry, continuity and independently reached endpoints; they
//! do not regenerate goldens or inspect the widgets' private animation state.
use super::{harness::Harness, input::key, reference::Image};
use crate::{Element, MotionScheme, TextFieldVariant, Theme};
use iced::{Color, Event, Length, Size, keyboard::key::Named, widget, window};

fn themes() -> [Theme; 3] {
    [
        Theme::light(),
        Theme::light().motion_scheme(MotionScheme::standard()),
        Theme::light().expressive(),
    ]
}

fn ui<Message>(
    content: Element<'static, Message>,
    size: Size,
    theme: Theme,
) -> Harness<'static, Message> {
    let mut ui = Harness::configured(content, size, theme);
    ui.frame(); // Resolve the theme before the first event.
    ui.at(0);
    ui
}

fn field(value: &str, variant: TextFieldVariant, width: f32) -> Element<'static, ()> {
    widget::container(
        crate::text_field("Name", value)
            .variant(variant)
            .width(width)
            .leading(widget::space().width(24).height(24))
            .on_input(|_| ()),
    )
    .padding(16)
    .into()
}

fn label_theme(mut theme: Theme) -> Theme {
    // Isolate the label's ink from the value, outline and background. The test
    // measures actual glyph bounds, not the transition's internal scalar.
    theme.colors.on_surface_variant = Color::from_rgb8(255, 0, 255);
    theme.colors.outline = Color::BLACK;
    theme.colors.on_surface = Color::BLACK;
    theme
}

fn label_bounds(image: &Image) -> (u32, u32, u32, u32) {
    let (mut left, mut top, mut right, mut bottom) = (image.width, image.height, 0, 0);
    for (i, pixel) in image.pixels.as_chunks::<4>().0.iter().enumerate() {
        // Exclude the filled field's underline; the label stays above 72px.
        if i as u32 / image.width < 144 && pixel[0] > 220 && pixel[1] < 80 && pixel[2] > 220 {
            let (x, y) = (i as u32 % image.width, i as u32 / image.width);
            left = left.min(x);
            top = top.min(y);
            right = right.max(x);
            bottom = bottom.max(y);
        }
    }
    assert!(
        right > left && bottom > top,
        "label ink must remain visible"
    );
    (left, top, right - left, bottom - top)
}

#[test]
fn floating_labels_reverse_from_visible_geometry_and_reach_exact_endpoints() {
    for theme in themes().map(label_theme) {
        for variant in [TextFieldVariant::Outlined, TextFieldVariant::Filled] {
            for width in [140., 320.] {
                let size = Size::new(width + 32., 104.);
                let mut view = ui(field("", variant, width), size, theme.clone());
                let resting = view.frame();
                let rest = label_bounds(&resting);
                view.rebuild(field("x", variant, width));
                view.at(0);
                assert_eq!(label_bounds(&view.frame()), rest);
                view.at(40);
                let moving = label_bounds(&view.frame());
                assert!(moving.1 < rest.1, "floating label must move upward");
                assert!(moving.2 < rest.2, "floating label must shrink");
                view.rebuild(field("", variant, width));
                view.at(40);
                assert_eq!(label_bounds(&view.frame()), moving, "reversal jumped");
                if theme.motion.scheme == Some(MotionScheme::expressive()) {
                    view.at(50);
                    assert!(
                        label_bounds(&view.frame()).1 < moving.1,
                        "the outgoing spring velocity must survive reversal"
                    );
                }
                view.at(3000);
                assert!(
                    view.frame() == resting,
                    "return must match a fresh empty field"
                );
                assert_eq!(view.at(3016), window::RedrawRequest::Wait);
                view.rebuild(field("x", variant, width));
                view.at(3016);
                view.at(6000);
                let mut fresh = ui(field("x", variant, width), size, theme.clone());
                assert!(
                    view.frame() == fresh.frame(),
                    "settled label layout must be exact"
                );
                assert_eq!(view.at(6016), window::RedrawRequest::Wait);
                assert!(
                    view.messages.is_empty(),
                    "controlled values do not emit edits"
                );
            }
        }
    }
}

#[test]
fn floating_label_motion_is_independent_of_redraw_cadence_and_rebuilds() {
    for theme in themes().map(label_theme) {
        for variant in [TextFieldVariant::Outlined, TextFieldVariant::Filled] {
            let make = |value| field(value, variant, 240.);
            let mut sparse = ui(make(""), Size::new(272., 104.), theme.clone());
            let mut frequent = ui(make(""), Size::new(272., 104.), theme.clone());
            for (start, value) in [(0, "x"), (48, ""), (80, "x")] {
                for view in [&mut sparse, &mut frequent] {
                    view.rebuild(make(value));
                    view.at(start);
                }
                for offset in [8, 16, 24] {
                    frequent.at(start + offset);
                    frequent.frame();
                    frequent.rebuild(make(value));
                }
                sparse.at(start + 32);
                frequent.at(start + 32);
                assert!(
                    sparse.frame() == frequent.frame(),
                    "cadence/rebuild changed label pixels"
                );
            }
        }
    }
}

fn selection(kind: &str, value: u8, disabled: bool) -> Element<'static, ()> {
    let content: Element<'static, ()> = match kind {
        "checkbox" => crate::checkbox(value == 1)
            .indeterminate(value == 2)
            .on_toggle(|_| ())
            .disabled(disabled)
            .into(),
        "radio" => crate::radio("", 1, (value == 1).then_some(1))
            .on_select(|_| ())
            .disabled(disabled)
            .into(),
        "switch" => crate::switch(value == 1)
            .on_toggle(|_| ())
            .disabled(disabled)
            .into(),
        _ => unreachable!(),
    };
    widget::container(content).padding(16).into()
}

#[test]
fn selection_reversals_and_mixed_handoffs_preserve_pixels_and_settle() {
    for theme in themes() {
        for kind in ["checkbox", "radio", "switch"] {
            let mut view = ui(
                selection(kind, 0, false),
                Size::new(112., 88.),
                theme.clone(),
            );
            for (start, value) in [
                (0, 1),
                (40, 0),
                (60, 1),
                (90, if kind == "checkbox" { 2 } else { 0 }),
                (120, 1),
            ] {
                view.at(start);
                let before = view.frame();
                view.rebuild(selection(kind, value, false));
                view.at(start);
                assert!(view.frame() == before, "{kind}: retarget at {start} jumped");
            }
            view.at(3000);
            let mut fresh = ui(
                selection(kind, 1, false),
                Size::new(112., 88.),
                theme.clone(),
            );
            assert!(
                view.frame() == fresh.frame(),
                "{kind}: final selected pixels"
            );
            assert_eq!(view.at(3016), window::RedrawRequest::Wait);
            assert!(view.messages.is_empty());
        }
    }
}

#[test]
fn selection_disable_during_pointer_or_keyboard_press_cancels_until_a_new_press() {
    for theme in themes() {
        for kind in ["checkbox", "radio", "switch"] {
            for keyboard in [false, true] {
                let make = |disabled| crate::focus::scope(selection(kind, 0, disabled));
                let mut view = ui(make(false), Size::new(112., 88.), theme.clone());
                if keyboard {
                    view.tap_key(key(Named::Tab));
                    view.event(key(Named::Space).press());
                } else {
                    view.move_to((40., 40.));
                    view.down();
                }
                view.at(40);
                view.rebuild(make(true));
                view.at(40);
                view.rebuild(make(false));
                view.at(40);
                if keyboard {
                    view.event(key(Named::Space).release());
                } else {
                    view.up();
                }
                assert!(
                    view.messages.is_empty(),
                    "{kind}: cancelled press must not activate"
                );
                view.leave();
                view.at(3000);
                let mut fresh = ui(make(false), Size::new(112., 88.), theme.clone());
                assert!(
                    view.frame() == fresh.frame(),
                    "{kind}: no stranded feedback"
                );
                assert_eq!(view.at(3016), window::RedrawRequest::Wait);
                view.move_to((40., 40.));
                view.down();
                view.up();
                assert_eq!(
                    view.messages,
                    [()],
                    "{kind}: a new click must activate exactly once"
                );
            }
        }
    }
}

#[test]
fn enabling_reduced_motion_mid_transition_settles_and_disabling_it_restores_motion() {
    for theme in themes() {
        for kind in ["field", "checkbox", "radio", "switch"] {
            let make = |selected| {
                if kind == "field" {
                    field(
                        if selected { "x" } else { "" },
                        TextFieldVariant::Outlined,
                        180.,
                    )
                } else {
                    selection(kind, u8::from(selected), false)
                }
            };
            let size = Size::new(212., 104.);
            let mut view = ui(make(false), size, theme.clone());
            view.rebuild(make(true));
            view.at(0);
            view.at(40);
            let reduced = theme.clone().reduced_motion(true);
            view.set_theme(reduced.clone());
            view.frame();
            view.at(40);
            let mut fresh = ui(make(true), size, reduced);
            assert!(
                view.frame() == fresh.frame(),
                "{kind}: preference must settle existing motion"
            );
            assert_eq!(view.at(56), window::RedrawRequest::Wait);
            view.set_theme(theme.clone());
            view.frame();
            view.rebuild(make(false));
            view.at(56);
            let start = view.frame();
            view.at(96);
            assert!(
                view.frame() != start,
                "{kind}: motion must resume after preference is cleared"
            );
            view.at(3000);
            let mut fresh = ui(make(false), size, theme.clone());
            assert!(view.frame() == fresh.frame());
            assert_eq!(view.at(3016), window::RedrawRequest::Wait);
            assert!(view.messages.is_empty());
        }
    }
}

fn overlay(kind: &str, open: bool) -> Element<'static, u8> {
    let background = widget::container(crate::button("Background").on_press(0))
        .padding(8)
        .width(Length::Fill)
        .height(Length::Fill);
    let content = widget::column![
        widget::text("Panel"),
        crate::button("Action").on_press(1),
        widget::space().height(100),
    ]
    .spacing(16);
    let host = match kind {
        "dialog" => crate::dialog::modal(background, crate::dialog(content).width(280.), open),
        "stack" => crate::dialog::stack(background, [(crate::dialog(content).width(280.), open)]),
        "side" => crate::sheet::host(background, crate::side_sheet(content).modal().open(open)),
        "bottom" => crate::sheet::host(
            background,
            crate::bottom_sheet(content).height(260.).open(open),
        ),
        _ => unreachable!(),
    };
    crate::focus::scope(host)
}

#[test]
fn overlay_close_reopen_between_redraws_cancels_pointer_and_keyboard_presses() {
    for theme in themes() {
        for kind in ["dialog", "stack", "side", "bottom"] {
            for keyboard in [false, true] {
                let mut view = ui(overlay(kind, true), Size::new(520., 420.), theme.clone());
                view.at(3000);
                if keyboard {
                    view.tap_key(key(Named::Tab));
                    view.event(key(Named::Space).press());
                } else {
                    let action = view.find("Action").center();
                    view.move_to(action);
                    view.down();
                }
                view.rebuild(overlay(kind, false));
                view.rebuild(overlay(kind, true));
                // No redraw between these rebuilds: the close must still cancel
                // the old gesture, even though the surface never visibly left.
                if keyboard {
                    view.event(key(Named::Space).release());
                } else {
                    view.up();
                }
                assert!(
                    view.messages.is_empty(),
                    "{kind}: stale release after reopen, keyboard={keyboard}"
                );
                view.at(6000);
                view.click("Action");
                assert_eq!(view.messages, [1], "{kind}: fresh click delivered once");
            }
        }
    }
}

#[test]
fn overlay_motion_continues_after_window_unfocus_and_reversal() {
    for theme in themes() {
        for kind in ["dialog", "stack", "side", "bottom"] {
            let size = Size::new(520., 420.);
            let mut active = ui(overlay(kind, false), size, theme.clone());
            let mut inactive = ui(overlay(kind, false), size, theme.clone());
            for view in [&mut active, &mut inactive] {
                view.rebuild(overlay(kind, true));
                view.at(0);
                view.at(40);
            }
            let before = inactive.frame();
            inactive.event(Event::Window(window::Event::Unfocused));
            active.at(80);
            inactive.at(80);
            assert!(
                inactive.frame() != before,
                "{kind}: window unfocus froze entrance"
            );
            assert!(
                active.frame() == inactive.frame(),
                "{kind}: keyboard focus changed animation clock"
            );
            for view in [&mut active, &mut inactive] {
                view.rebuild(overlay(kind, false));
                view.at(80);
                view.at(120);
            }
            assert!(
                active.frame() == inactive.frame(),
                "{kind}: unfocused exit differs"
            );
            inactive.at(3000);
            let mut fresh = ui(overlay(kind, false), size, theme.clone());
            assert!(
                inactive.frame() == fresh.frame(),
                "{kind}: closed overlay left ink"
            );
            assert_eq!(inactive.at(3016), window::RedrawRequest::Wait);
            assert!(inactive.messages.is_empty());
        }
    }
}

#[test]
fn overlay_open_close_between_redraws_still_cancels_the_background_press() {
    for kind in ["dialog", "stack", "side", "bottom"] {
        for keyboard in [false, true] {
            let mut view = ui(overlay(kind, false), Size::new(520., 420.), Theme::light());
            if keyboard {
                view.tap_key(key(Named::Tab));
                view.event(key(Named::Space).press());
            } else {
                let target = view.find("Background").center();
                view.move_to(target);
                view.down();
            }
            view.rebuild(overlay(kind, true));
            view.rebuild(overlay(kind, false));
            if keyboard {
                view.event(key(Named::Space).release());
            } else {
                view.up();
            }
            assert!(
                view.messages.is_empty(),
                "{kind}: covered press replayed, keyboard={keyboard}"
            );
            view.click("Background");
            assert_eq!(view.messages, [0]);
        }
    }
}

#[test]
fn nested_dialog_open_close_between_redraws_cancels_the_lower_dialog_press() {
    let make = |open| {
        crate::dialog::stack(
            widget::space().width(Length::Fill).height(Length::Fill),
            [
                (
                    crate::dialog(crate::button("Lower action").on_press(1)).width(320.),
                    true,
                ),
                (
                    crate::dialog(crate::button("Upper action").on_press(2)).width(240.),
                    open,
                ),
            ],
        )
    };
    for keyboard in [false, true] {
        let mut view = ui(make(false), Size::new(520., 420.), Theme::light());
        if keyboard {
            view.event(key(Named::Space).press());
        } else {
            let target = view.find("Lower action").center();
            view.move_to(target);
            view.down();
        }
        view.rebuild(make(true));
        view.rebuild(make(false));
        if keyboard {
            view.event(key(Named::Space).release());
        } else {
            view.up();
        }
        assert!(
            view.messages.is_empty(),
            "covered lower dialog press replayed, keyboard={keyboard}"
        );
        view.click("Lower action");
        assert_eq!(view.messages, [1]);
    }
}

#[test]
fn reduced_motion_during_overlay_exit_removes_pixels_and_restores_input() {
    for theme in themes() {
        for kind in ["dialog", "stack", "side", "bottom"] {
            for size in [Size::new(320., 360.), Size::new(720., 560.)] {
                let mut view = ui(overlay(kind, true), size, theme.clone());
                view.at(3000);
                view.rebuild(overlay(kind, false));
                view.at(3000);
                view.at(3040);
                let reduced = theme.clone().reduced_motion(true);
                view.set_theme(reduced.clone());
                view.frame();
                view.at(3040);
                let mut fresh = ui(overlay(kind, false), size, reduced);
                assert!(
                    view.frame() == fresh.frame(),
                    "{kind}: reduced exit left ink"
                );
                assert_eq!(view.at(3056), window::RedrawRequest::Wait);
                view.click("Background");
                assert_eq!(
                    view.messages,
                    [0],
                    "{kind}: finished overlay still blocks input"
                );
            }
        }
    }
}

fn reveal(kind: &str, open: bool) -> Element<'static, ()> {
    let icon = || widget::space().width(24).height(24);
    let content = match kind {
        "fab" => crate::extended_fab(icon(), "Create")
            .extended(open)
            .on_press(())
            .into(),
        "menu" => crate::fab_menu(
            icon(),
            open,
            (),
            [
                crate::FabMenuItem::new("First action", icon(), ()),
                crate::FabMenuItem::new("Second action", icon(), ()),
            ],
        ),
        _ => unreachable!(),
    };
    let layout: Element<'static, ()> = if kind == "fab" {
        widget::row![content, widget::text("After")].into()
    } else {
        widget::column![content, widget::text("After")].into()
    };
    widget::container(layout).padding(16).into()
}

#[test]
fn reveal_reversal_and_reduced_motion_restore_clipped_pixels_and_sibling_layout() {
    for theme in themes() {
        for kind in ["fab", "menu"] {
            for size in [Size::new(200., 280.), Size::new(480., 320.)] {
                let mut view = ui(reveal(kind, false), size, theme.clone());
                let closed = view.frame();
                let after = view.find("After");
                let mut opened = ui(reveal(kind, true), size, theme.clone());
                let open_after = opened.find("After");
                let extent = |r: iced::Rectangle| if kind == "fab" { r.x } else { r.y };
                view.rebuild(reveal(kind, true));
                view.at(0);
                view.at(60);
                let partial = view.frame();
                let moving = view.find("After");
                assert!(
                    extent(after) < extent(moving) && extent(moving) < extent(open_after),
                    "{kind}: reveal must move sibling layout through intermediate sizes"
                );
                assert!(partial != closed, "{kind}: reveal must animate");
                view.rebuild(reveal(kind, false));
                view.at(60);
                assert_eq!(view.find("After"), moving, "{kind}: reversal jumped layout");
                // The menu launcher intentionally swaps its close icon immediately.
                // Compare only the revealed items above that 56px launcher.
                let height = if kind == "menu" {
                    ((view.find("After").y - 56.) * 2.) as u32
                } else {
                    partial.height
                };
                assert!(
                    view.frame().crop(0, 0, partial.width, height)
                        == partial.crop(0, 0, partial.width, height),
                    "{kind}: reversal changed revealed pixels"
                );
                view.at(90);
                view.rebuild(reveal(kind, true));
                view.at(90);
                let reduced = theme.clone().reduced_motion(true);
                view.set_theme(reduced.clone());
                view.frame();
                view.at(90);
                let mut fresh = ui(reveal(kind, true), size, reduced.clone());
                assert!(
                    view.frame() == fresh.frame(),
                    "{kind}: final open clipping/layout"
                );
                view.rebuild(reveal(kind, false));
                view.at(90);
                assert_eq!(
                    view.find("After"),
                    after,
                    "{kind}: sibling layout must return"
                );
                assert!(view.frame() == closed, "{kind}: closed reveal left pixels");
                assert_eq!(view.at(106), window::RedrawRequest::Wait);
                assert!(view.messages.is_empty());
            }
        }
    }
}
