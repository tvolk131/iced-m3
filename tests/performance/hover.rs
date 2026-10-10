//! Exercise real card events; inspect the raster cache inputs, not wall-clock time.
use super::*;
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Raster {
    id: u64,
    width: u32,
    height: u32,
}

fn card(
    variant: crate::SurfaceVariant,
    size: Size,
    modal: Option<bool>,
) -> Element<'static, Message> {
    let background = widget::container(
        crate::card(widget::Space::new().height((size.height - 32.).max(0.)))
            .variant(variant)
            .on_press(Message::Input),
    )
    .padding(32)
    .width(size.width + 64.)
    .height(Length::Fill);
    if let Some(open) = modal {
        crate::dialog::modal(
            background,
            crate::dialog(widget::Space::new().height(100)).width(300.),
            open,
        )
    } else {
        background.into()
    }
}

fn rasters(renderer: &mut Renderer) -> HashSet<Raster> {
    software(renderer)
        .layers()
        .iter()
        // A tiled source can start above its visible portion; use the layer's
        // hard clip to exclude the centered dialog from background-card probes.
        .filter(|layer| layer.bounds.y < 300.)
        .flat_map(|layer| &layer.images)
        .filter_map(|image| match image {
            graphics::Image::Vector { svg, bounds, .. } if bounds.y < 300. => Some(Raster {
                id: svg.handle.id(),
                width: bounds.width.to_bits(),
                height: bounds.height.to_bits(),
            }),
            _ => None,
        })
        .collect()
}

fn frame(
    ui: &mut UserInterface<'_, Message, Theme, Renderer>,
    renderer: &mut Renderer,
    theme: &Theme,
    now: Instant,
    over: bool,
) {
    let cursor = if over {
        mouse::Cursor::Available(Point::new(60., 48.))
    } else {
        mouse::Cursor::Unavailable
    };
    let mut messages = Vec::new();
    crate::motion::with_time(now, || {
        ui.update(
            &[Event::Window(window::Event::RedrawRequested(now))],
            cursor,
            renderer,
            &mut iced::advanced::clipboard::Null,
            &mut messages,
        );
    });
    assert!(messages.is_empty(), "Hover must never activate the card");
    ui.draw(
        renderer,
        theme,
        &renderer::Style {
            text_color: theme.colors.on_surface,
        },
        cursor,
    );
}

#[test]
fn hover_shadow_reuses_endpoint_rasters_on_reversal_and_modal_cover() {
    for variant in [
        crate::SurfaceVariant::Elevated,
        crate::SurfaceVariant::Filled,
        crate::SurfaceVariant::Outlined,
    ] {
        let size = Size::new(1330., 250.);
        let viewport = Size::new(1394., 900.);
        let mut renderer = cpu_renderer();
        let theme = Theme::light();
        let origin = Instant::now();
        let mut ui = UserInterface::build(
            card(variant, size, Some(false)),
            viewport,
            Cache::default(),
            &mut renderer,
        );
        frame(&mut ui, &mut renderer, &theme, origin, false);
        let mut endpoints = rasters(&mut renderer);
        frame(
            &mut ui,
            &mut renderer,
            &theme,
            origin + Duration::from_millis(1),
            true,
        );
        frame(
            &mut ui,
            &mut renderer,
            &theme,
            origin + Duration::from_millis(400),
            true,
        );
        endpoints.extend(rasters(&mut renderer));
        assert_eq!(
            endpoints.len(),
            if variant == crate::SurfaceVariant::Elevated {
                2
            } else {
                1
            }
        );
        for (start, end, over, modal) in [
            (401, 449, false, false),
            (450, 650, true, false),
            (651, 900, true, true),
            (901, 1400, true, false),
        ] {
            ui = UserInterface::build(
                card(variant, size, Some(modal)),
                viewport,
                ui.into_cache(),
                &mut renderer,
            );
            for ms in (start..=end).step_by(16) {
                frame(
                    &mut ui,
                    &mut renderer,
                    &theme,
                    origin + Duration::from_millis(ms),
                    over,
                );
                assert!(
                    rasters(&mut renderer).is_subset(&endpoints),
                    "{variant:?} generated an intermediate shadow raster at {ms}ms"
                );
            }
        }
    }
}

#[test]
fn hover_shadow_raster_area_is_bounded_for_wide_and_shallow_cards() {
    for size in [
        Size::new(1330., 250.),
        Size::new(1330., 32.),
        Size::new(4000., 250.),
    ] {
        let mut renderer = cpu_renderer();
        let theme = Theme::light();
        let origin = Instant::now();
        let mut ui = UserInterface::build(
            card(crate::SurfaceVariant::Elevated, size, None),
            Size::new(size.width + 64., 900.),
            Cache::default(),
            &mut renderer,
        );
        for (ms, over) in [(0, false), (1, true), (80, true), (400, true)] {
            frame(
                &mut ui,
                &mut renderer,
                &theme,
                origin + Duration::from_millis(ms),
                over,
            );
            let images = rasters(&mut renderer);
            assert!(!images.is_empty());
            for image in images {
                assert!(
                    f32::from_bits(image.width) <= 160. && f32::from_bits(image.height) <= 160.,
                    "{size:?} generated a surface-sized shadow: {image:?}"
                );
            }
        }
    }
}

#[test]
#[ignore = "release-only diagnostic timings; run explicitly on an idle machine"]
fn profile_hover_rendering() {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    #[cfg(feature = "wgpu")]
    let gpu = (backend == "wgpu").then(|| iced::futures::executor::block_on(Gpu::new()));
    #[cfg(not(feature = "wgpu"))]
    let gpu: Option<Gpu> = None;
    for (variant, transparent) in [
        (crate::SurfaceVariant::Elevated, false),
        (crate::SurfaceVariant::Filled, false),
        (crate::SurfaceVariant::Outlined, false),
        (crate::SurfaceVariant::Elevated, true),
    ] {
        let mut theme = Theme::light();
        if transparent {
            theme.colors.shadow = iced::Color::TRANSPARENT;
        }
        let viewport = graphics::Viewport::with_physical_size(Size::new(2788, 628), 2.);
        let (mut renderer, mut target) = Target::new(gpu.as_ref(), viewport.physical_size());
        let mut ui = UserInterface::build(
            card(variant, Size::new(1330., 250.), None),
            viewport.logical_size(),
            Cache::default(),
            &mut renderer,
        );
        let origin = Instant::now();
        let mut changing = Vec::new();
        let mut settled = Vec::new();
        let mut recording = Vec::new();
        for sample in 0..35 {
            let start = Instant::now();
            frame(
                &mut ui,
                &mut renderer,
                &theme,
                origin + Duration::from_millis(sample * 16),
                (1..18).contains(&sample),
            );
            let ui_ms = start.elapsed().as_secs_f64() * 1000.;
            let start = Instant::now();
            target.render(
                &mut renderer,
                &viewport,
                theme.colors.surface,
                Damage::Tracked,
            );
            let repaint_ms = start.elapsed().as_secs_f64() * 1000.;
            if (2..=11).contains(&sample) || (19..=28).contains(&sample) {
                changing.push(repaint_ms);
                recording.push(ui_ms);
            } else if sample > 11 && sample != 18 {
                settled.push(repaint_ms);
            }
            println!(
                "{backend} {variant:?} transparent={transparent} frame={sample}: UI={ui_ms:.3}ms repaint={repaint_ms:.3}ms"
            );
        }
        let median = |v: &mut Vec<f64>| {
            v.sort_by(f64::total_cmp);
            (v[(v.len() - 1) / 2] + v[v.len() / 2]) / 2.
        };
        println!(
            "SUMMARY {backend} {variant:?} transparent={transparent}: UI={:.3}ms transition={:.3}ms settled={:.3}ms",
            median(&mut recording),
            median(&mut changing),
            median(&mut settled)
        );
    }
}
