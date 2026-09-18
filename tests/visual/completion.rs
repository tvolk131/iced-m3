use super::{harness::Harness, reference, theme, themed_name};
use crate::{
    ButtonVariant, CarouselVariant, Element, TextFieldVariant, Theme, TypeScale, button, card,
    carousel, focus, typography,
};
use iced::{Event, Length, Size, widget, window};
#[derive(Clone, Debug, PartialEq)]
enum Message {
    Action,
    Other,
    Input(String),
    Toggle(bool),
    Index(usize),
    Height(f32),
    Dismiss,
}
fn make_ui(
    element: impl Into<Element<'static, Message>>,
    size: Size,
    theme: Theme,
) -> Harness<'static, Message> {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    Harness::with_backend(element, size, theme, &backend)
}
fn cards(index: usize, variant: CarouselVariant) -> Element<'static, Message> {
    focus::scope(
        carousel(
            (0..5).map(|i| {
                card(widget::column![
                    typography(format!("Item {i}"), TypeScale::HeadlineSmall),
                    typography("Collection preview", TypeScale::BodyLarge)
                ])
                .on_press(Message::Action)
                .into()
            }),
            index,
        )
        .variant(variant)
        .on_select(Message::Index),
    )
}
#[test]
fn carousel_drag_cancels_card_activation_and_emits_a_bounded_snap() {
    let mut ui = make_ui(
        cards(0, CarouselVariant::Uncontained),
        Size::new(600.0, 200.0),
        Theme::light(),
    );
    ui.at(0);
    ui.move_to((220.0, 100.0));
    ui.down();
    ui.move_to((20.0, 100.0));
    ui.up();
    assert_eq!(ui.messages, [Message::Index(1)]);
    ui.rebuild(cards(1, CarouselVariant::Uncontained));
    for t in [0, 100, 200, 400] {
        ui.at(t);
    }
    assert_eq!(ui.at(600), window::RedrawRequest::Wait);
}
#[test]
fn card_children_keep_independent_actions() {
    let mut ui = make_ui(
        focus::scope(
            card(widget::column![
                typography("Open card", TypeScale::BodyLarge),
                button("Child action").on_press(Message::Other)
            ])
            .on_press(Message::Action),
        ),
        Size::new(300.0, 150.0),
        Theme::light(),
    );
    ui.click("Child action");
    assert_eq!(ui.messages, [Message::Other]);
    ui.click("Open card");
    assert_eq!(ui.messages.last(), Some(&Message::Action));
}
fn sheet(height: f32, modal: bool) -> Element<'static, Message> {
    let panel = crate::bottom_sheet(button("Sheet action").on_press(Message::Other))
        .height(height)
        .width(600.0)
        .open(true)
        .on_height(Message::Height)
        .snap_points([200.0, 400.0])
        .on_dismiss(Message::Dismiss);
    crate::sheet::host(
        button("Background").on_press(Message::Action),
        if modal { panel } else { panel.standard() },
    )
}
#[test]
fn bottom_sheet_handle_resizes_snaps_and_cancels_without_dismissing() {
    let mut ui = make_ui(sheet(200.0, true), Size::new(600.0, 600.0), Theme::light());
    ui.at(0);
    ui.frame();
    ui.at(300);
    ui.move_to((300.0, 414.0));
    ui.down();
    ui.move_to((300.0, 270.0));
    assert_eq!(ui.messages.last(), Some(&Message::Height(344.0)));
    ui.up();
    assert_eq!(ui.messages.last(), Some(&Message::Height(400.0)));
    ui.rebuild(sheet(400.0, true));
    ui.at(400);
    ui.move_to((300.0, 214.0));
    ui.down();
    ui.move_to((300.0, 260.0));
    ui.event(Event::Window(window::Event::Unfocused));
    assert_eq!(ui.messages.last(), Some(&Message::Height(400.0)));
    assert!(!ui.messages.contains(&Message::Dismiss));
}
#[test]
fn standard_bottom_sheet_reserves_vertical_space() {
    let mut ui = make_ui(sheet(200.0, false), Size::new(600.0, 600.0), Theme::light());
    ui.at(0);
    ui.frame();
    ui.at(300);
    let background = ui.find("Background");
    assert!(background.y + background.height < 400.0);
    ui.click("Background");
    assert_eq!(ui.messages, [Message::Action]);
    ui.click("Sheet action");
    assert_eq!(ui.messages.last(), Some(&Message::Other));
}
#[test]
fn rich_tooltip_action_is_reachable_and_dismisses_panel() {
    let mut ui = make_ui(
        crate::rich_tooltip(
            typography("Help", TypeScale::LabelLarge),
            "Rich hint",
            widget::column![
                typography("Useful details", TypeScale::BodyMedium),
                button("Got it").on_press(Message::Action)
            ],
        ),
        Size::new(450.0, 300.0),
        Theme::light(),
    );
    ui.click("Help");
    ui.click("Got it");
    assert_eq!(ui.messages, [Message::Action]);
}
#[test]
fn reduced_motion_stops_indeterminate_redraws() {
    let mut ui = make_ui(
        crate::loading_indicator(),
        Size::new(48.0, 48.0),
        Theme::light().reduced_motion(true),
    );
    ui.at(0);
    ui.frame();
    assert_eq!(ui.at(100), window::RedrawRequest::Wait);
    let a = ui.frame();
    ui.at(500);
    assert_eq!(a.pixels, ui.frame().pixels);
}
fn variants() -> Element<'static, Message> {
    widget::container(
        widget::column![
            crate::text_field("Filled field", "Material")
                .variant(TextFieldVariant::Filled)
                .on_input(Message::Input),
            crate::text_field("Error field", "Bad value")
                .variant(TextFieldVariant::Filled)
                .error("Check this value")
                .on_input(Message::Input),
            button("Elevated button")
                .variant(ButtonVariant::Elevated)
                .on_press(Message::Action),
            widget::row![
                crate::switch(false).icons(true).on_toggle(Message::Toggle),
                crate::switch(true).icons(true).on_toggle(Message::Toggle),
                crate::switch(true).icons(true)
            ]
            .spacing(12),
            crate::assist_chip("Elevated chip")
                .elevated(true)
                .on_press(Message::Action),
            crate::input_chip("Studio")
                .avatar(typography("S", TypeScale::LabelLarge))
                .dropdown(true)
                .on_press(Message::Action),
            crate::button_group([
                button("Copy").on_press(Message::Action),
                button("Move").on_press(Message::Other),
                button("Share").on_press(Message::Action)
            ])
            .connected(true),
            crate::split_button(
                "Save",
                Message::Action,
                [crate::MenuItem::new("Save a copy", Message::Other)]
            ),
            crate::toolbar([
                crate::icon_button(typography("+", TypeScale::TitleLarge))
                    .on_press(Message::Action)
                    .into(),
                crate::icon_button(typography("×", TypeScale::TitleLarge))
                    .on_press(Message::Other)
                    .into()
            ])
            .floating(true),
            crate::badged(
                crate::icon_button(typography("A", TypeScale::TitleLarge))
                    .on_press(Message::Action),
                None
            ),
        ]
        .spacing(18),
    )
    .padding(24)
    .into()
}
#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_completion_variants() {
    for dark in [false, true] {
        let mut ui = make_ui(variants(), Size::new(500.0, 760.0), theme(dark));
        ui.at(0);
        let case = themed_name("completion-variants", dark);
        reference::check(&format!("{case}/00-default"), &ui.frame());
        let p = ui.find("Move").center();
        ui.move_to(p);
        ui.down();
        for t in [0, 50, 100, 150] {
            ui.at(t);
            reference::check(&format!("{case}/01-held-{t:03}ms"), &ui.frame());
        }
        for variant in [
            CarouselVariant::MultiBrowse,
            CarouselVariant::Hero,
            CarouselVariant::Uncontained,
            CarouselVariant::FullScreen,
        ] {
            let mut ui = make_ui(cards(0, variant), Size::new(600.0, 200.0), theme(dark));
            ui.at(0);
            let case = themed_name(&format!("completion-carousel-{variant:?}"), dark);
            reference::check(&format!("{case}/00-default"), &ui.frame());
            ui.rebuild(cards(1, variant));
            for t in [0, 50, 100, 200] {
                ui.at(t);
                reference::check(&format!("{case}/01-snap-{t:03}ms"), &ui.frame());
            }
        }
        let mut ui = make_ui(
            crate::loading_indicator().contained(true).size(96.0),
            Size::new(96.0, 96.0),
            theme(dark),
        );
        ui.at(0);
        let case = themed_name("completion-loading", dark);
        for t in [0, 100, 325, 650, 975, 1300, 1950, 2600, 3250, 3900, 4550] {
            ui.at(t);
            reference::check(&format!("{case}/frame-{t:04}ms"), &ui.frame());
        }
    }
}

fn extended_action(open: bool, blank: bool) -> Element<'static, Message> {
    let icon: Element<'static, Message> = if blank {
        widget::space().width(24).height(24).into()
    } else {
        super::icon::add().into()
    };
    widget::container(
        crate::extended_fab(icon, "Create collection")
            .extended(open)
            .on_press(Message::Action),
    )
    .padding(20)
    .width(Length::Fill)
    .into()
}

#[test]
fn extended_fab_svg_ink_stays_centered_through_collapse_and_expansion() {
    for dark in [false, true] {
        let mut with_icon = make_ui(
            extended_action(true, false),
            Size::new(340., 96.),
            theme(dark),
        );
        let mut without_icon = make_ui(
            extended_action(true, true),
            Size::new(340., 96.),
            theme(dark),
        );
        for ui in [&mut with_icon, &mut without_icon] {
            ui.at(0);
            ui.frame();
        }
        for (base, expanded) in [(0, false), (400, true)] {
            with_icon.rebuild(extended_action(expanded, false));
            without_icon.rebuild(extended_action(expanded, true));
            for ms in [0, 50, 100, 200, 400] {
                with_icon.at(base + ms);
                without_icon.at(base + ms);
                let image = with_icon.frame();
                let blank = without_icon.frame();
                // Subtract the identical button/label to measure actual SVG ink,
                // including antialiasing, independently of its widget layout box.
                let mut extent = (image.width, image.height, 0, 0);
                let mut painted = 0;
                for (i, (pixel, background)) in image
                    .pixels
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .zip(blank.pixels.as_chunks::<4>().0.iter())
                    .enumerate()
                {
                    if pixel != background {
                        let x = i as u32 % image.width;
                        let y = i as u32 / image.width;
                        extent = (
                            extent.0.min(x),
                            extent.1.min(y),
                            extent.2.max(x),
                            extent.3.max(y),
                        );
                        painted += 1;
                    }
                }
                assert!(painted > 0, "The SVG must paint visible ink");
                // 20px outer padding + 16px leading inset + half the 24px slot;
                // vertically: 20px padding + half the 56px FAB. Captured at 2x.
                let x = (extent.0 + extent.2 + 1) as f32 / 2.;
                let y = (extent.1 + extent.3 + 1) as f32 / 2.;
                assert!((x - 96.).abs() <= 1., "Horizontal ink center: {x}");
                assert!((y - 96.).abs() <= 1., "Vertical ink center: {y}");
                assert!(
                    (31..=33).contains(&(extent.2 - extent.0 + 1)),
                    "SVG width must stay 16 logical pixels"
                );
                assert!(
                    (31..=33).contains(&(extent.3 - extent.1 + 1)),
                    "SVG height must stay 16 logical pixels"
                );
            }
        }
    }
}

#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_extended_fab() {
    for dark in [false, true] {
        let mut ui = make_ui(
            extended_action(true, false),
            Size::new(340., 96.),
            theme(dark),
        );
        ui.at(0);
        ui.frame();
        ui.rebuild(extended_action(false, false));
        let case = themed_name("completion-fab", dark);
        for t in [0, 50, 100, 200, 400] {
            ui.at(t);
            reference::check(&format!("{case}/collapse-{t:03}ms"), &ui.frame());
        }
    }
}

#[test]
fn cropped_carousel_items_cannot_intercept_an_adjacent_card() {
    let mut ui = make_ui(
        cards(0, CarouselVariant::MultiBrowse),
        Size::new(600.0, 200.0),
        Theme::light(),
    );
    ui.move_to((260.0, 100.0));
    ui.down();
    ui.up();
    assert_eq!(ui.messages, [Message::Action]);
}
#[test]
fn carousel_mask_covers_square_content_corners() {
    let mut ui = make_ui(
        carousel(
            [widget::container(widget::space())
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_: &Theme| widget::container::Style {
                    background: Some(iced::Color::from_rgb(1.0, 0.0, 0.0).into()),
                    ..Default::default()
                })
                .into()],
            0,
        ),
        Size::new(280.0, 200.0),
        Theme::light(),
    );
    ui.at(0);
    let frame = ui.frame();
    let pixel = |x: u32, y: u32| {
        let i = ((y * frame.width + x) * 4) as usize;
        &frame.pixels[i..i + 4]
    };
    assert_ne!(pixel(30, 4), [255, 0, 0, 255]);
    assert_eq!(pixel(280, 200), [255, 0, 0, 255]);
}

fn fab_actions(open: bool) -> Element<'static, Message> {
    widget::container(crate::fab_menu(
        typography("+", TypeScale::TitleLarge),
        open,
        Message::Action,
        [
            crate::FabMenuItem::new(
                "New document",
                typography("D", TypeScale::TitleLarge),
                Message::Other,
            ),
            crate::FabMenuItem::new(
                "New folder",
                typography("F", TypeScale::TitleLarge),
                Message::Other,
            ),
        ],
    ))
    .padding(20)
    .align_right(Length::Fill)
    .align_bottom(Length::Fill)
    .into()
}
#[test]
fn closing_fab_menu_blocks_actions_during_exit() {
    let mut ui = make_ui(fab_actions(true), Size::new(340.0, 300.0), Theme::light());
    ui.at(0);
    ui.frame();
    let p = ui.find("New document").center();
    ui.click("New document");
    assert_eq!(ui.messages, [Message::Other]);
    ui.messages.clear();
    ui.rebuild(fab_actions(false));
    ui.at(0);
    ui.move_to(p);
    ui.down();
    ui.up();
    assert!(ui.messages.is_empty());
}
#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_completion_menus_and_sheets() {
    for dark in [false, true] {
        let mut ui = make_ui(fab_actions(false), Size::new(340.0, 300.0), theme(dark));
        ui.at(0);
        ui.frame();
        ui.rebuild(fab_actions(true));
        let case = themed_name("completion-fab-menu", dark);
        for t in [0, 50, 100, 200] {
            ui.at(t);
            reference::check(&format!("{case}/01-open-{t:03}ms"), &ui.frame());
        }
        ui.rebuild(fab_actions(false));
        for t in [200, 250, 300, 400] {
            ui.at(t);
            reference::check(&format!("{case}/02-close-{:03}ms", t - 200), &ui.frame());
        }
        for modal in [false, true] {
            let mut ui = make_ui(sheet(200.0, modal), Size::new(600.0, 600.0), theme(dark));
            ui.at(0);
            ui.frame();
            ui.at(300);
            let case = themed_name(
                &format!(
                    "completion-sheet-{}",
                    if modal { "modal" } else { "standard" }
                ),
                dark,
            );
            reference::check(&format!("{case}/00-open"), &ui.frame());
            ui.move_to((300.0, 414.0));
            ui.down();
            ui.move_to((300.0, 270.0));
            ui.rebuild(sheet(344.0, modal));
            ui.at(350);
            reference::check(&format!("{case}/01-drag"), &ui.frame());
            ui.up();
            ui.rebuild(sheet(400.0, modal));
            ui.at(400);
            reference::check(&format!("{case}/02-snap"), &ui.frame());
        }
    }
}
