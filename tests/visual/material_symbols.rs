use super::harness::Harness;
use crate::material_symbols::{Fill, IconName, OpticalSize, Style, Symbol};
use crate::{Element, Theme};
use iced::advanced::svg::{Data, Handle};
use iced::{Color, Size, widget};

#[test]
fn complete_catalog_has_typed_names_and_every_variant_renders_offline() {
    assert_eq!(IconName::ALL.len(), 4157);
    let theme = Theme::light();
    let background = theme.colors.surface;
    let bg = [background.r, background.g, background.b, 1.0].map(|v| (v * 255.).round() as u8);
    let mut h: Harness<'_, ()> = Harness::new(widget::space(), Size::new(24., 24.), theme);
    for &name in IconName::ALL {
        assert_eq!(IconName::from_name(name.name()), Some(name));
        let default: Handle = name.into();
        assert_eq!(
            default,
            Handle::from(Symbol::new(name)),
            "default coordinates for {name:?}"
        );
        for style in [Style::Outlined, Style::Rounded] {
            for fill in [Fill::Unfilled, Fill::Filled] {
                for optical in [
                    OpticalSize::Size20,
                    OpticalSize::Size24,
                    OpticalSize::Size40,
                    OpticalSize::Size48,
                ] {
                    let symbol = Symbol::new(name)
                        .style(style)
                        .fill(fill)
                        .optical_size(optical);
                    let handle: Handle = symbol.into();
                    let Data::Bytes(bytes) = handle.data() else {
                        panic!("Bundled symbols must never depend on filesystem paths");
                    };
                    let svg = std::str::from_utf8(bytes.as_ref()).unwrap();
                    assert!(svg.contains(&format!("width=\"{}\"", optical.pixels())));
                    assert!(
                        std::ptr::eq(handle.data(), Handle::from(symbol).data()),
                        "Handles should reuse the cached allocation"
                    );
                    h.rebuild(symbol);
                    let image = h.frame();
                    let ink = image
                        .pixels
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .filter(|p| **p != bg)
                        .count();
                    assert!(
                        ink > 0,
                        "Missing SVG ink: {name:?}/{style:?}/{fill:?}/{optical:?}"
                    );
                }
            }
        }
    }
    assert_eq!(IconName::from_name("not_a_material_symbol"), None);
    assert_eq!(IconName::from_name("Home"), None);
    assert_eq!(IconName::Icon10k.name(), "10k");
}

#[test]
fn optical_size_changes_artwork_without_changing_layout_size() {
    let large = Symbol::new(IconName::Home).optical_size(OpticalSize::Size48);
    let default: Handle = IconName::Home.into();
    assert_ne!(default, Handle::from(large));
    let size = |icon: &crate::Icon| {
        <crate::Icon as iced::advanced::Widget<(), Theme, iced::Renderer>>::size(icon)
    };
    assert_eq!(size(&large.icon()), Size::new(24.0.into(), 24.0.into()));
    assert_eq!(
        size(&large.icon().size(48)),
        Size::new(48.0.into(), 48.0.into())
    );
}

#[test]
fn material_symbols_inherit_foreground_alpha_in_both_themes() {
    for theme in [Theme::light(), Theme::dark()] {
        for alpha in [1.0, 0.38] {
            let foreground = Color {
                a: alpha,
                ..Color::from_rgb8(20, 160, 60)
            };
            let background = Color::from_rgb8(80, 40, 120);
            let content: Element<'_, ()> =
                widget::container(Symbol::new(IconName::Home).fill(Fill::Filled))
                    .style(move |_| widget::container::Style {
                        text_color: Some(foreground),
                        background: Some(background.into()),
                        ..Default::default()
                    })
                    .into();
            let mut h = Harness::new(content, Size::new(24., 24.), theme.clone());
            let image = h.frame();
            // A solid interior roof pixel of the official filled Home asset.
            let offset = (20 * image.width as usize + 24) * 4;
            let actual = &image.pixels[offset..offset + 4];
            let expected = [
                foreground.r * alpha + background.r * (1.0 - alpha),
                foreground.g * alpha + background.g * (1.0 - alpha),
                foreground.b * alpha + background.b * (1.0 - alpha),
                1.0,
            ]
            .map(|v| (v * 255.).round() as u8);
            for (a, e) in actual.iter().zip(expected) {
                assert!(
                    a.abs_diff(e) <= 1,
                    "Foreground/opacity mismatch: {actual:?} vs {expected:?}"
                );
            }
        }
    }
}

/// Run this test alone to measure a fresh process (the global handle cache starts empty).
#[test]
#[ignore = "manual first-use measurement: run alone with --exact --ignored --nocapture"]
fn navigation_rail_first_use_measurement() {
    use crate::{NavigationItem, navigation_rail};
    use std::time::Instant;
    let mut h: Harness<'_, ()> =
        Harness::new(widget::space(), Size::new(96., 320.), Theme::light());
    let started = Instant::now();
    let items: Vec<_> = [IconName::Home, IconName::Inbox, IconName::Settings]
        .into_iter()
        .enumerate()
        .map(|(index, name)| {
            let symbol = Symbol::new(name).style(Style::Rounded);
            NavigationItem::new(index, name.name(), symbol).selected_icon(symbol.fill(Fill::Filled))
        })
        .collect();
    let six_handles = started.elapsed();
    h.rebuild(navigation_rail(items, Some(0)));
    let handles_and_layout = started.elapsed();
    let first = h.frame();
    let first_frame = started.elapsed();
    let warm = Instant::now();
    for _ in 0..100 {
        assert!(h.frame() == first);
    }
    println!(
        "Material Symbols navigation rail: six handles {:?}, handles + layout {:?}, first frame {:?}, warm frame average {:?}; TinySkia, 2x, renderer initialization excluded; layout includes cold font setup",
        six_handles,
        handles_and_layout,
        first_frame,
        warm.elapsed() / 100
    );
}
