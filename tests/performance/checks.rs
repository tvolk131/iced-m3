//! CI-safe performance contracts: render work and pixels, never elapsed time.
use super::*;
use iced::advanced::Renderer as _;

#[test]
fn scrolling_body_damage_only_touches_corner_shadow_tiles() {
    let mut renderer = cpu_renderer();
    let viewport = Rectangle::with_size(Size::new(1920., 1080.));
    let cache = crate::elevation::Cache::default();
    for size in [
        Size::new(280., 120.),
        Size::new(640., 648.),
        Size::new(1280., 800.),
    ] {
        let bounds = Rectangle::new(Point::new(80.25, 64.75), size);
        renderer.reset(viewport);
        crate::elevation::draw_resizing(
            &mut renderer,
            &Theme::light(),
            bounds,
            800.,
            28.,
            3.,
            viewport,
            &cache,
        );
        let layers = software(&mut renderer).layers();
        let shadow_layers: Vec<_> = layers
            .iter()
            .filter(|layer| !layer.images.is_empty())
            .collect();
        assert!(shadow_layers.len() > 4, "Fixture must exercise tiled edges");
        let body_damage = bounds.shrink(24.);
        let touched = shadow_layers
            .iter()
            .filter(|layer| layer.bounds.intersects(&body_damage))
            .count();
        assert!(
            touched <= 4,
            "Scrolling a {size:?} body touched {touched} shadow tiles; transparent edge interiors must not participate in repainting"
        );
    }
}

#[test]
fn disabled_and_invisible_shadows_record_no_paint_or_clip_work() {
    let mut renderer = cpu_renderer();
    let viewport = Rectangle::with_size(Size::new(1280., 800.));
    for invisible in [false, true] {
        let mut theme = Theme::light();
        theme.shadows = invisible;
        if invisible {
            theme.colors.shadow.a = 0.;
        }
        renderer.reset(viewport);
        let cache = crate::elevation::Cache::default();
        crate::elevation::draw_resizing(
            &mut renderer,
            &theme,
            Rectangle::new(Point::new(40., 40.), Size::new(640., 648.)),
            648.,
            28.,
            3.,
            viewport,
            &cache,
        );
        assert!(
            cache.borrow().is_none(),
            "Invisible shadows must not allocate a raster"
        );
        let layers = software(&mut renderer).layers();
        assert_eq!(
            layers.len(),
            1,
            "Invisible shadows must not create empty clipping layers"
        );
        assert!(layers[0].images.is_empty() && layers[0].quads.is_empty());
    }
}

#[test]
fn partial_dialog_repaints_match_full_rendering_and_idle_frames_have_no_damage() {
    crate::fonts::ensure_loaded();
    for dark in [false, true] {
        for scale in [1., 1.25, 2.] {
            let viewport = graphics::Viewport::with_physical_size(
                Size::new((720. * scale) as u32, (550. * scale) as u32),
                scale,
            );
            let theme = if dark { Theme::dark() } else { Theme::light() }.reduced_motion(true);
            let (mut renderer, mut target) = Target::new(None, viewport.physical_size());
            let mut ui = UserInterface::build(
                view(Scene::Dialog),
                viewport.logical_size(),
                Cache::default(),
                &mut renderer,
            );
            let mut full = tiny_skia::Pixmap::new(
                viewport.physical_size().width,
                viewport.physical_size().height,
            )
            .unwrap();
            let mut mask = tiny_skia::Mask::new(
                viewport.physical_size().width,
                viewport.physical_size().height,
            )
            .unwrap();
            let origin = Instant::now();
            for frame in 0..8 {
                let now = origin + Duration::from_millis(frame * 16);
                let cursor = mouse::Cursor::Available(Point::new(360., 250.));
                let mut messages = Vec::new();
                crate::motion::with_time(now, || {
                    ui.update(
                        &[
                            Event::Mouse(mouse::Event::WheelScrolled {
                                delta: mouse::ScrollDelta::Pixels {
                                    x: 0.,
                                    y: if frame % 4 < 2 { -31.5 } else { 31.5 },
                                },
                            }),
                            Event::Window(window::Event::RedrawRequested(now)),
                        ],
                        cursor,
                        &mut renderer,
                        &mut iced::advanced::clipboard::Null,
                        &mut messages,
                    );
                });
                ui.draw(
                    &mut renderer,
                    &theme,
                    &renderer::Style {
                        text_color: theme.colors.on_surface,
                    },
                    cursor,
                );
                target.render(
                    &mut renderer,
                    &viewport,
                    theme.colors.surface,
                    Damage::Tracked,
                );
                software(&mut renderer).draw(
                    &mut full.as_mut(),
                    &mut mask,
                    &viewport,
                    &[Rectangle::with_size(viewport.logical_size())],
                    theme.colors.surface,
                );
                let pixmap = match &target {
                    Target::Cpu { pixmap, .. } => pixmap,
                    #[cfg(feature = "wgpu")]
                    _ => unreachable!("This test uses the software renderer"),
                };
                // Tiny Skia can round a partially clipped glyph's blend one
                // byte differently. Permit only that per-channel rounding;
                // the strict full-frame visual references remain exact.
                assert!(
                    pixmap
                        .data()
                        .iter()
                        .zip(full.data())
                        .all(|(a, b)| a.abs_diff(*b) <= 1),
                    "Partial repaint left stale pixels (dark={dark}, scale={scale}, frame={frame})"
                );
                let idle = target
                    .render(
                        &mut renderer,
                        &viewport,
                        theme.colors.surface,
                        Damage::Tracked,
                    )
                    .unwrap();
                assert_eq!(
                    idle.regions, 0,
                    "Unchanged dialog must not cause another repaint"
                );
            }
        }
    }
}

#[test]
fn benchmark_scenes_use_matching_scroll_viewports_and_content_sizes() {
    use iced::advanced::widget::{Operation, operation::scrollable::Scrollable};
    #[derive(Default)]
    struct ScrollBounds(Vec<(Rectangle, Rectangle)>);
    impl Operation for ScrollBounds {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
            operate(self);
        }
        fn scrollable(
            &mut self,
            _: Option<&widget::Id>,
            bounds: Rectangle,
            content: Rectangle,
            _: iced::Vector,
            _: &mut dyn Scrollable,
        ) {
            self.0.push((bounds, content));
        }
    }
    let mut expected = None;
    for scene in [
        Scene::Form,
        Scene::Panel,
        Scene::Scrim,
        Scene::DialogNoShadow,
        Scene::Dialog,
    ] {
        let mut renderer = cpu_renderer();
        let mut ui = UserInterface::build(
            view(scene),
            Size::new(WIDTH as f32, HEIGHT as f32),
            Cache::default(),
            &mut renderer,
        );
        ui.update(
            &[Event::Window(
                window::Event::RedrawRequested(Instant::now()),
            )],
            mouse::Cursor::Unavailable,
            &mut renderer,
            &mut iced::advanced::clipboard::Null,
            &mut Vec::new(),
        );
        let mut query = ScrollBounds::default();
        ui.operate(&renderer, &mut query);
        assert_eq!(query.0.len(), 1, "{scene:?} must have one body scroller");
        let geometry = query.0[0];
        assert_eq!(
            geometry.0.height, 600.,
            "The same body height is measured in every scene"
        );
        assert_eq!(
            *expected.get_or_insert(geometry),
            geometry,
            "{scene:?} geometry must match the plain form"
        );
    }
}
