//! State comparisons against the pinned baseline recipes in docs/FIDELITY.md.
use super::{
    Message, capture, harness::Harness, icon, padded, reference::Image, theme, themed_name,
};
use crate::{
    ButtonVariant, Element, SurfaceVariant, TextFieldVariant, Theme, TypeScale, button, card,
    checkbox, focus, radio, surface, switch, text_field, typography,
};
use iced::{
    Color, Event, Length, Point, Size,
    keyboard::{self, key::Named},
    widget, window,
};

fn backend() -> String {
    std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into())
}
fn runtime(
    content: impl Into<Element<'static, Message>>,
    size: Size,
    theme: Theme,
) -> Harness<'static, Message> {
    Harness::with_backend(content, size, theme, &backend())
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
fn pixel(image: &Image, point: impl Into<Point>) -> [u8; 4] {
    let p = point.into();
    let offset = (((p.y * 2.0) as u32 * image.width + (p.x * 2.0) as u32) * 4) as usize;
    image.pixels[offset..offset + 4].try_into().unwrap()
}
// Render reference swatches with the same backend so alpha composition and
// color-space rounding are tested without assuming CPU/GPU byte equivalence.
fn swatch(theme: &Theme, layers: &[Color]) -> [u8; 4] {
    let stack = widget::Stack::with_children(layers.iter().copied().map(|color| {
        widget::container(widget::space())
            .width(20)
            .height(20)
            .style(move |_| widget::container::Style {
                background: Some(color.into()),
                ..Default::default()
            })
            .into()
    }));
    pixel(
        &runtime(stack, Size::new(20.0, 20.0), theme.clone()).frame(),
        (10.0, 10.0),
    )
}
fn field(variant: TextFieldVariant, error: bool, disabled: bool) -> Element<'static, Message> {
    let field = text_field("Workspace", "Studio")
        .id("fidelity-field")
        .variant(variant)
        .width(280)
        .supporting_text("Supporting text")
        .on_input(Message::Input)
        .disabled(disabled);
    padded(if error {
        field.error("Choose a different name")
    } else {
        field
    })
}
#[test]
fn field_hover_error_focus_and_disabled_use_distinct_material_roles() {
    for dark in [false, true] {
        let theme = theme(dark);
        let c = theme.colors;
        for variant in [TextFieldVariant::Outlined, TextFieldVariant::Filled] {
            let filled = variant == TextFieldVariant::Filled;
            for error in [false, true] {
                let mut ui = runtime(
                    field(variant, error, false),
                    Size::new(320.0, 112.0),
                    theme.clone(),
                );
                ui.at(0);
                ui.frame();
                let background = if filled {
                    c.surface_container_highest
                } else {
                    c.surface
                };
                let idle = ui.frame();
                assert_eq!(pixel(&idle, (200.0, 40.0)), swatch(&theme, &[background]));
                assert_eq!(
                    pixel(&idle, (150.0, 79.5)),
                    swatch(
                        &theme,
                        &[if error {
                            c.error
                        } else if filled {
                            c.on_surface_variant
                        } else {
                            c.outline
                        }]
                    )
                );
                ui.move_to((200.0, 40.0));
                ui.at(150);
                let hover = ui.frame();
                assert_eq!(
                    pixel(&hover, (150.0, 79.5)),
                    swatch(
                        &theme,
                        &[if error {
                            c.on_error_container
                        } else {
                            c.on_surface
                        }]
                    )
                );
                assert_eq!(
                    pixel(&hover, (200.0, 40.0)),
                    swatch(
                        &theme,
                        &if filled {
                            vec![background, crate::theme::alpha(c.on_surface, 0.08)]
                        } else {
                            vec![background]
                        }
                    )
                );
                // Supporting text retains its own role throughout hover and focus.
                assert!(idle.crop(64, 168, 500, 32) == hover.crop(64, 168, 500, 32));
                ui.down();
                ui.up();
                ui.at(350);
                assert_eq!(
                    pixel(&ui.frame(), (150.0, 78.5)),
                    swatch(&theme, &[if error { c.error } else { c.primary }])
                );
                ui.rebuild(field(variant, error, true));
                ui.at(550);
                let disabled = ui.frame();
                let backgrounds = if filled {
                    vec![crate::theme::alpha(c.on_surface, 0.04)]
                } else {
                    vec![c.surface]
                };
                assert_eq!(
                    pixel(&disabled, (200.0, 40.0)),
                    swatch(&theme, &backgrounds)
                );
                let mut edge = backgrounds;
                edge.push(crate::theme::alpha(
                    c.on_surface,
                    if filled { 0.38 } else { 0.12 },
                ));
                assert_eq!(pixel(&disabled, (150.0, 79.5)), swatch(&theme, &edge));
                ui.messages.clear();
                ui.down();
                ui.up();
                assert!(ui.messages.is_empty());
            }
        }
    }
}
#[test]
fn field_hover_excludes_supporting_text_and_reduced_motion_settles_immediately() {
    let theme = theme(false).reduced_motion(true);
    let mut ui = runtime(
        field(TextFieldVariant::Filled, false, false),
        Size::new(320.0, 112.0),
        theme,
    );
    ui.at(0);
    let idle = ui.frame();
    ui.move_to((200.0, 90.0));
    ui.at(0);
    assert!(idle == ui.frame());
    ui.move_to((200.0, 40.0));
    ui.at(0);
    assert!(idle != ui.frame());
    assert_eq!(ui.at(1), window::RedrawRequest::Wait);
    ui.leave();
    ui.at(1);
    assert!(idle == ui.frame());
}
#[test]
fn filled_and_tonal_hover_elevation_returns_to_rest_while_held() {
    for variant in [
        ButtonVariant::Filled,
        ButtonVariant::Tonal,
        ButtonVariant::Outlined,
        ButtonVariant::Text,
    ] {
        let mut ui = runtime(
            padded(
                button("Action")
                    .variant(variant)
                    .width(160)
                    .on_press(Message::Action),
            ),
            Size::new(200.0, 90.0),
            theme(false),
        );
        ui.at(0);
        let idle = ui.frame().crop(70, 114, 220, 12);
        ui.move_to((90.0, 36.0));
        ui.at(150);
        let hovered = ui.frame().crop(70, 114, 220, 12);
        assert_eq!(
            idle != hovered,
            matches!(variant, ButtonVariant::Filled | ButtonVariant::Tonal)
        );
        ui.down();
        ui.at(600);
        assert!(idle == ui.frame().crop(70, 114, 220, 12));
        assert!(ui.messages.is_empty());
        ui.up();
        ui.at(800);
        assert!(hovered == ui.frame().crop(70, 114, 220, 12));
    }
}
#[test]
fn passive_and_interactive_cards_share_variant_surface_roles() {
    for dark in [false, true] {
        let theme = theme(dark);
        for (variant, color) in [
            (
                SurfaceVariant::Filled,
                theme.colors.surface_container_highest,
            ),
            (SurfaceVariant::Outlined, theme.colors.surface),
            (SurfaceVariant::Elevated, theme.colors.surface_container_low),
        ] {
            for content in [
                Element::from(
                    surface(widget::space().width(100).height(50))
                        .variant(variant)
                        .padding(16),
                ),
                Element::from(card(widget::space().width(100).height(50)).variant(variant)),
                Element::from(
                    card(widget::space().width(100).height(50))
                        .variant(variant)
                        .on_press(Message::Action),
                ),
            ] {
                let mut ui = runtime(padded(content), Size::new(200.0, 120.0), theme.clone());
                assert_eq!(pixel(&ui.frame(), (80.0, 40.0)), swatch(&theme, &[color]));
            }
        }
    }
}
#[test]
fn switch_track_and_thumb_follow_interaction_and_disabled_roles() {
    for dark in [false, true] {
        let theme = theme(dark);
        let c = theme.colors;
        for selected in [false, true] {
            let make = |disabled| {
                padded(
                    switch(selected)
                        .on_toggle(Message::Toggle)
                        .disabled(disabled),
                )
            };
            let mut ui = runtime(make(false), Size::new(120.0, 90.0), theme.clone());
            ui.at(0);
            let idle = ui.frame();
            let thumb = if selected { (60.0, 40.0) } else { (40.0, 40.0) };
            assert_eq!(
                pixel(&idle, thumb),
                swatch(&theme, &[if selected { c.on_primary } else { c.outline }])
            );
            assert_eq!(
                pixel(&idle, (50.0, 28.0)),
                swatch(
                    &theme,
                    &[if selected {
                        c.primary
                    } else {
                        c.surface_container_highest
                    }]
                )
            );
            ui.move_to(thumb);
            ui.at(150);
            assert_eq!(
                pixel(&ui.frame(), thumb),
                swatch(
                    &theme,
                    &[if selected {
                        c.primary_container
                    } else {
                        c.on_surface_variant
                    }]
                )
            );
            ui.down();
            ui.at(650);
            assert!(ui.messages.is_empty());
            assert_eq!(
                pixel(&ui.frame(), thumb),
                swatch(
                    &theme,
                    &[if selected {
                        c.primary_container
                    } else {
                        c.on_surface_variant
                    }]
                )
            );
            ui.rebuild(make(true));
            ui.at(800);
            assert_eq!(
                pixel(&ui.frame(), (50.0, 28.0)),
                swatch(
                    &theme,
                    &[crate::theme::alpha(
                        if selected {
                            c.on_surface
                        } else {
                            c.surface_container_highest
                        },
                        0.12
                    )]
                )
            );
            ui.up();
            assert!(ui.messages.is_empty());
        }
    }
}

#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_fidelity_fields() {
    for dark in [false, true] {
        for (name, variant) in [
            ("outlined", TextFieldVariant::Outlined),
            ("filled", TextFieldVariant::Filled),
        ] {
            for error in [false, true] {
                let case = themed_name(
                    &format!(
                        "fidelity-field-{name}-{}",
                        if error { "error" } else { "normal" }
                    ),
                    dark,
                );
                let mut ui = Harness::new(
                    field(variant, error, false),
                    Size::new(320.0, 112.0),
                    theme(dark),
                );
                ui.at(0);
                capture(&mut ui, &case, "00-rest");
                ui.move_to((200.0, 40.0));
                ui.at(75);
                capture(&mut ui, &case, "01-hover-075ms");
                ui.at(150);
                capture(&mut ui, &case, "02-hover-150ms");
                ui.down();
                ui.up();
                // iced's native caret uses wall time, outside our motion clock.
                // A fixed text selection keeps focus references deterministic.
                ui.operate(
                    &mut iced::advanced::widget::operation::text_input::select_all::<()>(
                        widget::Id::new("fidelity-field"),
                    ),
                );
                ui.at(350);
                capture(&mut ui, &case, "03-focused-hover");
                ui.leave();
                ui.at(550);
                capture(&mut ui, &case, "04-focused");
                ui.rebuild(field(variant, error, true));
                ui.at(750);
                capture(&mut ui, &case, "05-disabled");
            }
        }
    }
}
fn actions() -> Element<'static, Message> {
    let mut rows = widget::Column::new().spacing(16);
    for (label, variant) in [
        ("Filled", ButtonVariant::Filled),
        ("Tonal", ButtonVariant::Tonal),
        ("Elevated", ButtonVariant::Elevated),
        ("Outlined", ButtonVariant::Outlined),
        ("Text", ButtonVariant::Text),
    ] {
        rows = rows.push(
            widget::row![
                button(label)
                    .variant(variant)
                    .width(160)
                    .on_press(Message::Action),
                button("Disabled").variant(variant).width(160)
            ]
            .spacing(24),
        );
    }
    focus::scope(widget::container(rows).padding(24))
}
#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_fidelity_buttons_and_focus() {
    for dark in [false, true] {
        let case = themed_name("fidelity-buttons", dark);
        let mut ui = Harness::new(actions(), Size::new(400.0, 340.0), theme(dark));
        ui.at(0);
        capture(&mut ui, &case, "00-rest-and-disabled");
        for (index, label) in ["Filled", "Tonal", "Elevated", "Outlined", "Text"]
            .into_iter()
            .enumerate()
        {
            tab(&mut ui);
            ui.at(index as u64);
            capture(&mut ui, &case, &format!("01-focus-{index}-{label}"));
        }
        ui.move_to((70.0, 40.0));
        ui.at(200);
        capture(&mut ui, &case, "02-hover");
        ui.down();
        ui.at(650);
        capture(&mut ui, &case, "03-held");
        ui.up();
        ui.at(900);
        capture(&mut ui, &case, "04-release");
        for kind in ["checkbox", "radio", "switch", "switch-icons"] {
            for selected in [false, true] {
                let control: Element<'static, Message> = match kind {
                    "checkbox" => checkbox(selected)
                        .label("Workspace updates")
                        .on_toggle(Message::Toggle)
                        .into(),
                    "radio" => radio("Workspace updates", 1, selected.then_some(1))
                        .on_select(Message::Select)
                        .into(),
                    _ => switch(selected)
                        .icons(kind == "switch-icons")
                        .label("Workspace updates")
                        .on_toggle(Message::Toggle)
                        .into(),
                };
                let mut ui = Harness::new(
                    focus::scope(padded(control)),
                    Size::new(300.0, 90.0),
                    theme(dark),
                );
                ui.at(0);
                ui.frame();
                tab(&mut ui);
                ui.at(200);
                capture(
                    &mut ui,
                    &themed_name(
                        &format!(
                            "fidelity-{kind}-{}",
                            if selected { "selected" } else { "unselected" }
                        ),
                        dark,
                    ),
                    "00-keyboard-focus",
                );
            }
        }
    }
}
#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_fidelity_cards_and_chips() {
    for dark in [false, true] {
        let mut items = widget::Column::new().spacing(16);
        for (name, variant) in [
            ("Filled", SurfaceVariant::Filled),
            ("Outlined", SurfaceVariant::Outlined),
            ("Elevated", SurfaceVariant::Elevated),
        ] {
            items = items.push(
                card(
                    widget::column![
                        typography(name, TypeScale::TitleMedium),
                        typography("Card supporting text", TypeScale::BodyMedium)
                    ]
                    .spacing(8),
                )
                .variant(variant)
                .on_press(Message::Action),
            );
        }
        items = items.push(
            widget::row![
                crate::assist_chip("Assist")
                    .leading(icon())
                    .on_press(Message::Action),
                crate::suggestion_chip("Suggestion").on_press(Message::Action),
                crate::filter_chip("Filter", false).on_press(Message::Action)
            ]
            .spacing(12),
        );
        items = items.push(
            widget::row![
                crate::input_chip("Input")
                    .on_press(Message::Action)
                    .on_remove(Message::Dismiss),
                crate::filter_chip("Selected", true).on_press(Message::Action),
                crate::suggestion_chip("Elevated")
                    .elevated(true)
                    .on_press(Message::Action)
            ]
            .spacing(12),
        );
        items = items.push(
            widget::row![
                crate::filter_chip("Disabled", false),
                crate::filter_chip("Selected", true),
                crate::suggestion_chip("Elevated").elevated(true)
            ]
            .spacing(12),
        );
        let mut ui = Harness::new(
            focus::scope(padded(items.width(Length::Fill))),
            Size::new(540.0, 460.0),
            theme(dark),
        );
        let case = themed_name("fidelity-cards-chips", dark);
        ui.at(0);
        capture(&mut ui, &case, "00-rest");
        ui.click("Filled");
        ui.at(500);
        capture(&mut ui, &case, "01-hover-card");
        ui.leave();
        ui.at(700);
        for i in 0..4 {
            tab(&mut ui);
            ui.at(701 + i);
        }
        capture(&mut ui, &case, "02-focused-chip");
    }
}

#[test]
fn keyboard_focus_has_a_state_layer_without_triggering_selection() {
    for dark in [false, true] {
        let theme = theme(dark);
        for selected in [false, true] {
            for radio_control in [false, true] {
                let control: Element<'static, Message> = if radio_control {
                    radio("Focus", 1, selected.then_some(1))
                        .on_select(Message::Select)
                        .into()
                } else {
                    checkbox(selected)
                        .label("Focus")
                        .on_toggle(Message::Toggle)
                        .into()
                };
                let mut ui = runtime(
                    focus::scope(padded(control)),
                    Size::new(180.0, 90.0),
                    theme.clone(),
                );
                ui.at(0);
                ui.frame();
                tab(&mut ui);
                ui.at(200);
                let color = if selected {
                    theme.colors.primary
                } else {
                    theme.colors.on_surface
                };
                assert_eq!(
                    pixel(&ui.frame(), (40.0, 25.0)),
                    swatch(&theme, &[crate::theme::alpha(color, 0.12)])
                );
                assert!(ui.messages.is_empty());
            }
        }
        let mut ui = runtime(
            focus::scope(padded(
                button("Focus")
                    .variant(ButtonVariant::Outlined)
                    .width(160)
                    .on_press(Message::Action),
            )),
            Size::new(200.0, 90.0),
            theme.clone(),
        );
        ui.at(0);
        ui.frame();
        tab(&mut ui);
        ui.at(200);
        assert_eq!(
            pixel(&ui.frame(), (60.0, 25.0)),
            swatch(&theme, &[crate::theme::alpha(theme.colors.primary, 0.12)])
        );
        assert!(ui.messages.is_empty());
    }
}
