//! Rendering costs of changing geometry versus compositing cached geometry.
use super::*;
use iced::advanced::Renderer as _;

fn vectors(renderer: &mut Renderer) -> Vec<(u64, Size)> {
    software(renderer)
        .layers()
        .iter()
        .flat_map(|l| &l.images)
        .filter_map(|image| {
            if let graphics::Image::Vector { svg, bounds, .. } = image {
                Some((svg.handle.id(), bounds.size()))
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn fading_ripple_reuses_geometry() {
    let mut renderer = cpu_renderer();
    let bounds = Rectangle::new(Point::new(32., 32.), Size::new(1330., 250.));
    let viewport = Rectangle::with_size(Size::new(1394., 314.));
    let cache = Default::default();
    let mut first = None;
    for alpha in [0.12, 0.09, 0.04, 0.01] {
        renderer.reset(viewport);
        crate::ripple::draw_bounded(
            &mut renderer,
            bounds,
            viewport,
            12.into(),
            Point::new(0.2, 0.6),
            1.,
            iced::Color::BLACK.scale_alpha(alpha),
            &cache,
        );
        let rasters = vectors(&mut renderer);
        assert_eq!(rasters.len(), 1);
        assert_eq!(
            *first.get_or_insert(rasters[0]),
            rasters[0],
            "Opacity-only fading must not rerasterize the ripple"
        );
    }
}

#[test]
fn held_card_release_keeps_the_ripple_raster_and_event_delivery() {
    let mut renderer = cpu_renderer();
    let mut theme = Theme::light();
    theme.shadows = false;
    let mut ui = UserInterface::build(
        widget::container(crate::card(widget::Space::new().height(100)).on_press(Message::Input))
            .padding(32),
        Size::new(700., 250.),
        Cache::default(),
        &mut renderer,
    );
    let cursor = mouse::Cursor::Available(Point::new(80., 70.));
    let origin = Instant::now();
    let style = renderer::Style {
        text_color: theme.colors.on_surface,
    };
    ui.draw(&mut renderer, &theme, &style, cursor);
    let mut messages = Vec::new();
    let mut held = None;
    for (ms, event) in [
        (
            0,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        ),
        (
            700,
            Event::Window(window::Event::RedrawRequested(
                origin + Duration::from_millis(700),
            )),
        ),
        (
            700,
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
        ),
        (
            725,
            Event::Window(window::Event::RedrawRequested(
                origin + Duration::from_millis(725),
            )),
        ),
        (
            750,
            Event::Window(window::Event::RedrawRequested(
                origin + Duration::from_millis(750),
            )),
        ),
    ] {
        crate::motion::with_time(origin + Duration::from_millis(ms), || {
            ui.update(
                &[event],
                cursor,
                &mut renderer,
                &mut iced::advanced::clipboard::Null,
                &mut messages,
            );
        });
        ui.draw(&mut renderer, &theme, &style, cursor);
        if ms >= 700 {
            let images = vectors(&mut renderer);
            assert_eq!(images.len(), 1);
            assert_eq!(*held.get_or_insert(images[0]), images[0]);
        }
    }
    assert!(
        matches!(messages.as_slice(), [Message::Input]),
        "Release must still activate exactly once"
    );
}

#[test]
fn custom_elevation_keeps_fractional_blur_rasters_small() {
    let mut renderer = cpu_renderer();
    let theme = Theme::light();
    let size = Size::new(1394., 414.);
    let mut cache = Cache::default();
    for (level, height) in [(1., 250.), (1.25, 270.), (2.75, 300.), (4.5, 340.)] {
        let mut ui = UserInterface::build(custom_shadow(level, height), size, cache, &mut renderer);
        ui.draw(
            &mut renderer,
            &theme,
            &renderer::Style {
                text_color: theme.colors.on_surface,
            },
            mouse::Cursor::Unavailable,
        );
        let images = vectors(&mut renderer);
        assert!(!images.is_empty());
        assert!(
            images
                .iter()
                .all(|(_, size)| size.width <= 240. && size.height <= 240.),
            "Fractional level {level} produced full-surface raster(s): {images:?}"
        );
        cache = ui.into_cache();
    }
}

fn custom_shadow(level: f32, height: f32) -> Element<'static, Message> {
    widget::container(crate::elevated(
        widget::Space::new().width(1330.).height(height),
        level,
        12.,
    ))
    .padding(32)
    .into()
}

#[derive(Clone, Copy, Debug)]
enum AnimatedScene {
    RippleExpand,
    RippleFade,
    CustomShadow,
    LinearWave,
    Loaders,
    Rings,
    StaggeredLoaders,
    StaggeredRings,
}

impl AnimatedScene {
    fn size(self) -> Size<u32> {
        match self {
            Self::Loaders | Self::Rings | Self::StaggeredLoaders | Self::StaggeredRings => {
                Size::new(320, 320)
            }
            _ => Size::new(1394, 414),
        }
    }
    fn view(self, frame: u64, paused: bool) -> Element<'static, Message> {
        match self {
            Self::CustomShadow => {
                custom_shadow(if paused { 2.5 } else { 1. + frame as f32 / 12. }, 250.)
            }
            Self::LinearWave => widget::container(
                crate::linear_progress(0.6)
                    .wavy(true)
                    .wave_speed(40.)
                    .paused(paused)
                    .width(1330),
            )
            .padding(32)
            .into(),
            Self::Loaders | Self::Rings | Self::StaggeredLoaders | Self::StaggeredRings => {
                widget::container(
                    widget::column((0..4).map(|row| {
                        widget::row((0..4).map(|col| -> Element<'static, Message> {
                            let paused = paused
                                || (matches!(self, Self::StaggeredLoaders | Self::StaggeredRings)
                                    && frame < row * 4 + col);
                            match self {
                                Self::Loaders | Self::StaggeredLoaders => {
                                    crate::loading_indicator().paused(paused).into()
                                }
                                _ => crate::circular_progress(0.)
                                    .wavy(true)
                                    .indeterminate(true)
                                    .paused(paused)
                                    .into(),
                            }
                        }))
                        .spacing(16)
                        .into()
                    }))
                    .spacing(16),
                )
                .padding(32)
                .into()
            }
            _ => widget::Space::new().into(),
        }
    }
}

#[test]
#[ignore = "release-only diagnostic timings; run explicitly on an idle machine"]
fn profile_animation_rendering() {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    #[cfg(feature = "wgpu")]
    let gpu = (backend == "wgpu").then(|| iced::futures::executor::block_on(Gpu::new()));
    #[cfg(not(feature = "wgpu"))]
    let gpu: Option<Gpu> = None;
    let filter = std::env::var("ICED_PERF_SCENE").ok();
    for scene in [
        AnimatedScene::RippleExpand,
        AnimatedScene::RippleFade,
        AnimatedScene::CustomShadow,
        AnimatedScene::LinearWave,
        AnimatedScene::Loaders,
        AnimatedScene::Rings,
        AnimatedScene::StaggeredLoaders,
        AnimatedScene::StaggeredRings,
    ] {
        if filter.as_ref().is_some_and(|f| f != &format!("{scene:?}")) {
            continue;
        }
        for scale in [1., 2.] {
            for paused in [false, true] {
                let logical = scene.size();
                let viewport = graphics::Viewport::with_physical_size(
                    Size::new(
                        (logical.width as f32 * scale) as u32,
                        (logical.height as f32 * scale) as u32,
                    ),
                    scale,
                );
                let (mut renderer, mut target) =
                    Target::new(gpu.as_ref(), viewport.physical_size());
                let theme = Theme::light();
                let origin = Instant::now();
                let mut cache = Cache::default();
                let ripple = Default::default();
                let mut recording = Vec::new();
                let mut painting = Vec::new();
                let mut cold = 0.;
                let staggered = matches!(
                    scene,
                    AnimatedScene::StaggeredLoaders | AnimatedScene::StaggeredRings
                );
                for frame in 0..if staggered { 50 } else { 36 } {
                    let start = Instant::now();
                    if matches!(
                        scene,
                        AnimatedScene::RippleExpand | AnimatedScene::RippleFade
                    ) {
                        renderer.reset(Rectangle::with_size(viewport.logical_size()));
                        let t = if paused { 0.5 } else { frame as f32 / 36. };
                        let (progress, alpha) = match scene {
                            AnimatedScene::RippleExpand => (t, 0.12),
                            _ => (1., 0.12 * (1. - t)),
                        };
                        crate::ripple::draw_bounded(
                            &mut renderer,
                            Rectangle::new(Point::new(32., 32.), Size::new(1330., 250.)),
                            Rectangle::with_size(viewport.logical_size()),
                            12.into(),
                            Point::new(0.2, 0.6),
                            progress,
                            iced::Color::BLACK.scale_alpha(alpha),
                            &ripple,
                        );
                    } else {
                        let mut ui = UserInterface::build(
                            scene.view(frame, paused),
                            viewport.logical_size(),
                            cache,
                            &mut renderer,
                        );
                        // Staggered populations also sample across the complete
                        // morph/loading cycle, not just one shared initial pose.
                        let now = origin
                            + Duration::from_millis(frame * if staggered { 173 } else { 16 });
                        crate::motion::with_time(now, || {
                            ui.update(
                                &[Event::Window(window::Event::RedrawRequested(now))],
                                mouse::Cursor::Unavailable,
                                &mut renderer,
                                &mut iced::advanced::clipboard::Null,
                                &mut Vec::new(),
                            );
                        });
                        ui.draw(
                            &mut renderer,
                            &theme,
                            &renderer::Style {
                                text_color: theme.colors.on_surface,
                            },
                            mouse::Cursor::Unavailable,
                        );
                        cache = ui.into_cache();
                    }
                    let record_ms = start.elapsed().as_secs_f64() * 1000.;
                    let start = Instant::now();
                    target.render(
                        &mut renderer,
                        &viewport,
                        theme.colors.surface,
                        Damage::Tracked,
                    );
                    let paint_ms = start.elapsed().as_secs_f64() * 1000.;
                    if frame == 0 {
                        cold = paint_ms;
                    }
                    if frame >= if staggered { 20 } else { 6 } {
                        recording.push(record_ms);
                        painting.push(paint_ms);
                    }
                }
                let median = |values: &mut Vec<f64>| {
                    values.sort_by(f64::total_cmp);
                    (values[(values.len() - 1) / 2] + values[values.len() / 2]) / 2.
                };
                println!(
                    "AUDIT {backend} {scene:?} scale={scale} paused={paused}: cold={cold:.3}ms record={:.3}ms repaint={:.3}ms",
                    median(&mut recording),
                    median(&mut painting)
                );
            }
        }
    }
}
