//! Independent state-role expectations, rather than self-approved screenshots.
//! Toggle reference: AndroidX 8fd64ac21a546397597caa1527fd2e589ea2915e,
//! ToggleButton.kt and Filled/Tonal/Elevated/OutlinedButtonTokens.kt.
use super::{harness::Harness, reference::Image};
use crate::{ButtonVariant as Variant, Element, Theme, button, button_group, focus};
use iced::{Color, Event, Size, keyboard, widget};

const VARIANTS: [Variant; 5] = [
    Variant::Tonal,
    Variant::Filled,
    Variant::Elevated,
    Variant::Outlined,
    Variant::Text,
];

fn distinct_theme() -> Theme {
    let mut t = Theme::light();
    let c = &mut t.colors;
    c.primary = Color::from_rgb8(181, 21, 41);
    c.on_primary = Color::from_rgb8(21, 181, 41);
    c.secondary = Color::from_rgb8(31, 51, 191);
    c.on_secondary = Color::from_rgb8(191, 171, 31);
    c.secondary_container = Color::from_rgb8(111, 31, 151);
    c.on_secondary_container = Color::from_rgb8(31, 151, 111);
    c.surface_container = Color::from_rgb8(151, 111, 71);
    c.surface_container_low = Color::from_rgb8(71, 111, 151);
    c.surface_container_highest = Color::from_rgb8(161, 71, 111);
    c.on_surface = Color::from_rgb8(91, 121, 21);
    c.on_surface_variant = Color::from_rgb8(121, 21, 91);
    c.inverse_surface = Color::from_rgb8(41, 101, 161);
    c.inverse_on_surface = Color::from_rgb8(161, 101, 41);
    c.outline = Color::from_rgb8(11, 131, 201);
    t
}

fn ui(content: impl Into<Element<'static, u8>>, theme: Theme) -> Harness<'static, u8> {
    Harness::new(host(content), Size::new(600., 120.), theme)
}
fn host(content: impl Into<Element<'static, u8>>) -> Element<'static, u8> {
    widget::container(content).padding(12).into()
}
fn rgba(c: Color) -> [u8; 4] {
    [c.r, c.g, c.b, c.a].map(|v| (v * 255.).round() as u8)
}
fn pixel(image: &Image, x: f32, y: f32) -> [u8; 4] {
    let offset = ((y * 2.) as usize * image.width as usize + (x * 2.) as usize) * 4;
    image.pixels[offset..offset + 4].try_into().unwrap()
}
fn contains(image: &Image, c: Color) -> bool {
    image
        .pixels
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| **p == rgba(c))
        .count()
        >= 3
}
fn pair(t: &Theme, v: Variant, selected: bool) -> (Color, Color) {
    let c = t.colors;
    match (v, selected) {
        (Variant::Filled, false) => (c.surface_container, c.on_surface_variant),
        (Variant::Filled | Variant::Elevated, true) => (c.primary, c.on_primary),
        (Variant::Tonal, false) => (c.secondary_container, c.on_secondary_container),
        (Variant::Tonal, true) => (c.secondary, c.on_secondary),
        (Variant::Elevated, false) => (c.surface_container_low, c.primary),
        (Variant::Outlined, false) => (c.surface, c.on_surface_variant),
        (Variant::Outlined, true) => (c.inverse_surface, c.inverse_on_surface),
        // No reference Text toggle: retain the documented library extension.
        (Variant::Text, false) => (c.surface, c.primary),
        (Variant::Text, true) => (c.secondary_container, c.on_secondary_container),
    }
}

#[test]
fn common_toggle_selection_uses_reference_roles_in_every_variant() {
    for base in [Theme::light(), Theme::dark(), distinct_theme()] {
        for expressive in [false, true] {
            let t = if expressive {
                base.clone().expressive()
            } else {
                base.clone()
            };
            for variant in VARIANTS {
                for selected in [false, true] {
                    let mut h = ui(
                        button("MMMM")
                            .width(180)
                            .variant(variant)
                            .selected(selected)
                            .on_press(1),
                        t.clone(),
                    );
                    let image = h.frame();
                    let (fill, ink) = pair(&t, variant, selected);
                    assert_eq!(
                        pixel(&image, 40., 32.),
                        rgba(fill),
                        "{variant:?}, selected={selected}, expressive={expressive}"
                    );
                    assert!(
                        contains(&image, ink),
                        "foreground: {variant:?}, selected={selected}"
                    );
                }
            }
        }
    }
}

#[test]
fn outlined_toggle_selection_removes_the_outline() {
    let t = distinct_theme();
    for selected in [false, true] {
        let mut h = ui(
            button("MMMM")
                .width(180)
                .variant(Variant::Outlined)
                .selected(selected)
                .on_press(1),
            t.clone(),
        );
        assert_eq!(
            pixel(&h.frame(), 100., 12.5),
            rgba(if selected {
                t.colors.inverse_surface
            } else {
                t.colors.outline
            })
        );
    }
}

fn group(selected: u8, connected: bool) -> Element<'static, u8> {
    focus::scope(
        button_group(
            ["First", "Second", "Third"]
                .into_iter()
                .enumerate()
                .map(|(i, label)| {
                    button(label)
                        .variant(Variant::Tonal)
                        .selected(selected == i as u8)
                        .on_press(i as u8)
                }),
        )
        .connected(connected),
    )
}
fn key(h: &mut Harness<'_, u8>, name: keyboard::key::Named, release: bool) {
    let key = keyboard::Key::Named(name);
    h.event(Event::Keyboard(if release {
        keyboard::Event::KeyReleased {
            key: key.clone(),
            modified_key: key,
            physical_key: keyboard::key::Physical::Unidentified(
                keyboard::key::NativeCode::Unidentified,
            ),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::empty(),
        }
    } else {
        keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: keyboard::key::Physical::Unidentified(
                keyboard::key::NativeCode::Unidentified,
            ),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::empty(),
            text: None,
            repeat: false,
        }
    }));
}

#[test]
fn group_selection_survives_release_pointer_exit_and_rebuild() {
    for base in [Theme::light(), Theme::dark()] {
        for expressive in [false, true] {
            for connected in [false, true] {
                for keyboard in [false, true] {
                    let t = if expressive {
                        base.clone().expressive()
                    } else {
                        base.clone()
                    };
                    let mut h = ui(group(0, connected), t.clone());
                    h.at(0);
                    h.frame();
                    if keyboard {
                        key(&mut h, keyboard::key::Named::Tab, false);
                        key(&mut h, keyboard::key::Named::Tab, true);
                        key(&mut h, keyboard::key::Named::ArrowRight, false);
                        key(&mut h, keyboard::key::Named::ArrowRight, true);
                        key(&mut h, keyboard::key::Named::Space, false);
                        key(&mut h, keyboard::key::Named::Space, true);
                    } else {
                        h.click("Second");
                    }
                    assert_eq!(h.messages, [1]);
                    h.rebuild(host(group(h.messages[0], connected)));
                    h.operate(&mut iced::advanced::widget::operation::focusable::unfocus::<()>());
                    h.leave();
                    h.at(0);
                    h.at(3000);
                    let image = h.frame();
                    for (label, expected) in [
                        ("First", t.colors.secondary_container),
                        ("Second", t.colors.secondary),
                        ("Third", t.colors.secondary_container),
                    ] {
                        let bounds = h.find(label);
                        assert_eq!(
                            pixel(&image, bounds.center_x(), 20.),
                            rgba(expected),
                            "{label}, connected={connected}, keyboard={keyboard}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn ordinary_actions_text_extension_and_palette_overrides_keep_their_contract() {
    let t = distinct_theme();
    for variant in VARIANTS {
        let expected = match variant {
            Variant::Filled => (t.colors.primary, t.colors.on_primary),
            Variant::Tonal => (
                t.colors.secondary_container,
                t.colors.on_secondary_container,
            ),
            Variant::Elevated => (t.colors.surface_container_low, t.colors.primary),
            Variant::Outlined | Variant::Text => (t.colors.surface, t.colors.primary),
        };
        let image = ui(
            button("MMMM").width(180).variant(variant).on_press(1),
            t.clone(),
        )
        .frame();
        assert_eq!(pixel(&image, 40., 32.), rgba(expected.0));
        assert!(contains(&image, expected.1));
        for selected in [false, true] {
            let make = |override_palette: bool, disabled: bool| {
                let b = button("MMMM")
                    .width(180)
                    .variant(variant)
                    .selected(selected)
                    .on_press(1)
                    .disabled(disabled);
                if override_palette {
                    b.palette(|t| (t.colors.error_container, t.colors.on_error_container))
                } else {
                    b
                }
            };
            let image = ui(make(true, false), t.clone()).frame();
            assert_eq!(pixel(&image, 40., 32.), rgba(t.colors.error_container));
            assert!(contains(&image, t.colors.on_error_container));
            assert!(
                ui(make(true, true), t.clone()).frame() == ui(make(false, true), t.clone()).frame()
            );
            if variant == Variant::Outlined {
                // Keep the library's baseline disabled outline treatment.
                assert_ne!(
                    pixel(&ui(make(false, true), t.clone()).frame(), 100., 12.5),
                    rgba(t.colors.surface)
                );
            }
        }
    }
}

#[test]
fn icon_toggles_keep_their_separate_variant_recipes() {
    for t in [Theme::light(), Theme::dark(), distinct_theme()] {
        let c = t.colors;
        for variant in [
            Variant::Filled,
            Variant::Tonal,
            Variant::Outlined,
            Variant::Text,
        ] {
            for selected in [false, true] {
                let (fill, ink) = match (variant, selected) {
                    (Variant::Filled, false) => (c.surface_container_highest, c.primary),
                    (Variant::Filled, true) => (c.primary, c.on_primary),
                    (Variant::Tonal, false) => (c.surface_container_highest, c.on_surface_variant),
                    (Variant::Tonal, true) => (c.secondary_container, c.on_secondary_container),
                    (Variant::Outlined, false) => (c.surface, c.on_surface_variant),
                    (Variant::Outlined, true) => (c.inverse_surface, c.inverse_on_surface),
                    (Variant::Text, false) => (c.surface, c.on_surface_variant),
                    (Variant::Text, true) => (c.surface, c.primary),
                    _ => unreachable!(),
                };
                let image = ui(
                    crate::icon_button(widget::text("M").font(crate::fonts::REGULAR).size(18))
                        .variant(variant)
                        .selected(selected)
                        .on_press(1),
                    t.clone(),
                )
                .frame();
                assert_eq!(
                    pixel(&image, 20., 32.),
                    rgba(fill),
                    "icon {variant:?}, selected={selected}"
                );
                assert!(
                    contains(&image, ink),
                    "icon foreground {variant:?}, selected={selected}"
                );
            }
        }
    }
}

#[test]
fn related_controls_keep_distinct_selection_roles() {
    use crate::{NavigationItem, Segment, SegmentSelection, Tab, TabVariant};
    for t in [Theme::light(), Theme::dark(), distinct_theme()] {
        let c = t.colors;
        for selected in [false, true] {
            let active = selected.then_some(0);
            let chip = ui(crate::filter_chip("MMMM", selected).on_press(1), t.clone()).frame();
            assert_eq!(
                contains(&chip, c.secondary_container),
                selected,
                "filter chip container"
            );
            assert!(contains(
                &chip,
                if selected {
                    c.on_secondary_container
                } else {
                    c.on_surface_variant
                }
            ));

            let segments = ui(
                crate::segmented_buttons(
                    [Segment::new(0, "MMMM")],
                    SegmentSelection::Single(active),
                )
                .on_change(|_| 1),
                t.clone(),
            )
            .frame();
            assert_eq!(
                contains(&segments, c.secondary_container),
                selected,
                "segment container"
            );
            assert!(contains(
                &segments,
                if selected {
                    c.on_secondary_container
                } else {
                    c.on_surface
                }
            ));

            for variant in [TabVariant::Primary, TabVariant::Secondary] {
                let tabs = ui(
                    crate::tabs([Tab::new(0, "MMMM")], active)
                        .variant(variant)
                        .on_select(|_| 1),
                    t.clone(),
                )
                .frame();
                assert_eq!(
                    contains(&tabs, c.primary),
                    selected,
                    "tab indicator {variant:?}"
                );
                assert!(contains(
                    &tabs,
                    if !selected {
                        c.on_surface_variant
                    } else if variant == TabVariant::Primary {
                        c.primary
                    } else {
                        c.on_surface
                    }
                ));
            }

            let items = || {
                [NavigationItem::new(
                    0,
                    "MMMM",
                    widget::text("M").font(crate::fonts::REGULAR).size(18),
                )]
            };
            for nav in [
                Element::from(crate::navigation_bar(items(), active).on_select(|_| 1)),
                Element::from(crate::navigation_rail(items(), active).on_select(|_| 1)),
            ] {
                let image = ui(nav, t.clone()).frame();
                assert_eq!(
                    contains(&image, c.secondary_container),
                    selected,
                    "navigation indicator"
                );
                assert!(contains(
                    &image,
                    if selected {
                        c.on_secondary_container
                    } else {
                        c.on_surface_variant
                    }
                ));
            }

            for (name, control) in [
                (
                    "checkbox",
                    Element::from(crate::checkbox(selected).on_toggle(|_| 1)),
                ),
                (
                    "radio",
                    Element::from(crate::radio("", 0, active).on_select(|_| 1)),
                ),
                (
                    "switch",
                    Element::from(crate::switch(selected).on_toggle(|_| 1)),
                ),
            ] {
                let image = ui(control, t.clone()).frame();
                assert_eq!(contains(&image, c.primary), selected, "{name} selection");
                assert!(
                    contains(
                        &image,
                        if selected {
                            if name == "radio" {
                                c.primary
                            } else {
                                c.on_primary
                            }
                        } else if name == "switch" {
                            c.outline
                        } else {
                            c.on_surface_variant
                        }
                    ),
                    "{name} foreground"
                );
            }
        }
    }
}
