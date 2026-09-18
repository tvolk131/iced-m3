//! Release-only repaint measurements. Timings are diagnostic, never CI gates.
//! Adapted from the Comet consumer reproduction with matched viewport geometry.
use crate::{Element, Theme};
use iced::advanced::{graphics, renderer};
use iced::{Event, Length, Point, Rectangle, Renderer, Size, mouse, widget, window};
#[cfg(feature = "wgpu")]
use iced_test::renderer::wgpu::{self as iced_wgpu, wgpu};
use iced_test::runtime::{UserInterface, user_interface::Cache};
use std::time::{Duration, Instant};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 800;
const SCALE: f32 = 2.;
const WARMUP: usize = 8;
const SAMPLES: usize = 30;

#[derive(Clone, Copy, Debug)]
enum Damage {
    Tracked,
    Union,
    Full,
}
impl Damage {
    fn from_env() -> Self {
        match std::env::var("ICED_PERF_DAMAGE")
            .as_deref()
            .unwrap_or("tracked")
        {
            "tracked" => Self::Tracked,
            "union" => Self::Union,
            "full" => Self::Full,
            value => panic!("Unknown ICED_PERF_DAMAGE={value}; use tracked, union or full"),
        }
    }
}
#[derive(Clone, Copy, Debug)]
struct Stats {
    layers: usize,
    regions: usize,
    area: f32,
}

#[cfg(not(feature = "wgpu"))]
struct Gpu;
fn cpu_renderer() -> Renderer {
    let renderer = iced_tiny_skia::Renderer::new(crate::fonts::REGULAR, 16.into());
    #[cfg(feature = "wgpu")]
    {
        Renderer::Secondary(renderer)
    }
    #[cfg(not(feature = "wgpu"))]
    {
        renderer
    }
}
fn software(renderer: &mut Renderer) -> &mut iced_tiny_skia::Renderer {
    #[cfg(feature = "wgpu")]
    {
        let Renderer::Secondary(renderer) = renderer else {
            panic!("Expected software renderer");
        };
        renderer
    }
    #[cfg(not(feature = "wgpu"))]
    {
        renderer
    }
}
#[cfg(feature = "wgpu")]
struct Gpu {
    device: wgpu::Device,
    engine: iced_wgpu::Engine,
    format: wgpu::TextureFormat,
}
#[cfg(feature = "wgpu")]
impl Gpu {
    async fn new() -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: wgpu::InstanceFlags::empty(),
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .expect("A hardware GPU is required for this benchmark");
        let info = adapter.get_info();
        assert_ne!(info.device_type, wgpu::DeviceType::Cpu);
        println!(
            "GPU: {} ({:?}, {:?})",
            info.name, info.backend, info.device_type
        );
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("iced-m3 rendering benchmark"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits {
                    max_bind_groups: 2,
                    max_non_sampler_bindings: 2048,
                    ..Default::default()
                },
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                ..Default::default()
            })
            .await
            .expect("Create GPU device");
        let format = if graphics::color::GAMMA_CORRECTION {
            wgpu::TextureFormat::Rgba8UnormSrgb
        } else {
            wgpu::TextureFormat::Rgba8Unorm
        };
        let engine = iced_wgpu::Engine::new(
            &adapter,
            device.clone(),
            queue,
            format,
            None, // Match the desktop app's default antialiasing setting.
            graphics::Shell::headless(),
        );
        Self {
            device,
            engine,
            format,
        }
    }
}

enum Target {
    Cpu {
        pixmap: tiny_skia::Pixmap,
        mask: tiny_skia::Mask,
        previous: Option<Vec<iced_tiny_skia::Layer>>,
    },
    #[cfg(feature = "wgpu")]
    Gpu {
        device: wgpu::Device,
        texture: wgpu::Texture,
        format: wgpu::TextureFormat,
    },
}
impl Target {
    fn new(gpu: Option<&Gpu>, size: Size<u32>) -> (Renderer, Self) {
        match gpu {
            #[cfg(feature = "wgpu")]
            Some(gpu) => {
                let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("iced-m3 benchmark target"),
                    size: wgpu::Extent3d {
                        width: size.width,
                        height: size.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: gpu.format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                });
                (
                    Renderer::Primary(iced_wgpu::Renderer::new(
                        gpu.engine.clone(),
                        crate::fonts::REGULAR,
                        16.into(),
                    )),
                    Self::Gpu {
                        device: gpu.device.clone(),
                        texture,
                        format: gpu.format,
                    },
                )
            }
            _ => (
                cpu_renderer(),
                Self::Cpu {
                    pixmap: tiny_skia::Pixmap::new(size.width, size.height).unwrap(),
                    mask: tiny_skia::Mask::new(size.width, size.height).unwrap(),
                    previous: None,
                },
            ),
        }
    }

    fn render(
        &mut self,
        renderer: &mut Renderer,
        viewport: &graphics::Viewport,
        background: iced::Color,
        mode: Damage,
    ) -> Option<Stats> {
        match (self, renderer) {
            (
                Self::Cpu {
                    pixmap,
                    mask,
                    previous,
                },
                renderer,
            ) => {
                let renderer = software(renderer);
                let bounds = Rectangle::with_size(viewport.logical_size());
                let damage = previous.as_ref().map_or_else(
                    || vec![bounds],
                    |previous| {
                        graphics::damage::diff(
                            previous,
                            renderer.layers(),
                            |layer| vec![layer.bounds],
                            iced_tiny_skia::Layer::damage,
                        )
                    },
                );
                *previous = Some(renderer.layers().to_vec());
                let mut damage = graphics::damage::group(damage, bounds);
                match mode {
                    Damage::Full if !damage.is_empty() => damage = vec![bounds],
                    Damage::Union if !damage.is_empty() => {
                        damage = vec![damage.iter().copied().reduce(|a, b| a.union(&b)).unwrap()]
                    }
                    _ => {}
                }
                let stats = Stats {
                    layers: renderer.layers().len(),
                    regions: damage.len(),
                    area: damage.iter().map(Rectangle::area).sum(),
                };
                if !damage.is_empty() {
                    renderer.draw(&mut pixmap.as_mut(), mask, viewport, &damage, background);
                }
                Some(stats)
            }
            #[cfg(feature = "wgpu")]
            (
                Self::Gpu {
                    device,
                    texture,
                    format,
                },
                Renderer::Primary(renderer),
            ) => {
                let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                let submission = renderer.present(Some(background), *format, &view, viewport);
                device
                    .poll(wgpu::PollType::Wait {
                        submission_index: Some(submission),
                        timeout: Some(Duration::from_secs(30)),
                    })
                    .expect("GPU render must complete");
                None
            }
            #[cfg(feature = "wgpu")]
            _ => unreachable!("The renderer and target must use the same backend"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Scene {
    Form,
    Panel,
    Scrim,
    DialogNoShadow,
    Dialog,
    NativeForm,
    NativeScrim,
}
#[derive(Clone, Debug)]
enum Message {
    Input,
}
fn view(scene: Scene) -> Element<'static, Message> {
    let fields: Element<'static, Message> =
        if matches!(scene, Scene::NativeForm | Scene::NativeScrim) {
            // This control uses only native iced widgets and iced's own Theme.
            // It isolates renderer cost from the Material text-field implementation.
            let fields: iced::Element<'static, Message> =
                widget::Column::with_children((0..12).map(|_| {
                    widget::text_input("Setting", "sample value")
                        .on_input(|_| Message::Input)
                        .size(16)
                        .line_height(iced::advanced::text::LineHeight::Absolute(24.into()))
                        .padding([20, 16])
                        .into()
                }))
                .spacing(16)
                .into();
            widget::themer(Some(iced::Theme::Light), fields).into()
        } else {
            widget::Column::with_children((0..12).map(|_| {
                crate::text_field("Setting", "sample value")
                    .on_input(|_| Message::Input)
                    .into()
            }))
            .spacing(16)
            .into()
        };
    if matches!(scene, Scene::Dialog | Scene::DialogNoShadow) {
        crate::dialog::modal(
            widget::space().width(Length::Fill).height(Length::Fill),
            crate::dialog(fields).width(640.).max_height(648),
            true,
        )
    } else {
        let body = widget::scrollable(fields)
            .width(Length::Fill)
            .height(600)
            .direction(widget::scrollable::Direction::Vertical(
                widget::scrollable::Scrollbar::new()
                    .width(4)
                    .scroller_width(4)
                    .spacing(8),
            ));
        let panel = widget::container(body)
            .width(640)
            .padding(24)
            .style(move |theme: &Theme| widget::container::Style {
                background: (!matches!(scene, Scene::Form | Scene::NativeForm))
                    .then_some(theme.colors.surface_container_high.into()),
                border: iced::Border {
                    radius: 28.into(),
                    ..Default::default()
                },
                ..Default::default()
            });
        widget::container(panel)
            .center(Length::Fill)
            .style(move |theme: &Theme| widget::container::Style {
                background: matches!(scene, Scene::Scrim | Scene::NativeScrim)
                    .then_some(crate::theme::alpha(theme.colors.scrim, 0.32).into()),
                ..Default::default()
            })
            .into()
    }
}
fn run(scene: Scene, gpu: Option<&Gpu>, mode: Damage) {
    let mut theme = Theme::light();
    theme.shadows = !matches!(scene, Scene::DialogNoShadow);
    let viewport = graphics::Viewport::with_physical_size(
        Size::new(
            (WIDTH as f32 * SCALE) as u32,
            (HEIGHT as f32 * SCALE) as u32,
        ),
        SCALE,
    );
    let (mut renderer, mut target) = Target::new(gpu, viewport.physical_size());
    let mut ui = UserInterface::build(
        view(scene),
        viewport.logical_size(),
        Cache::default(),
        &mut renderer,
    );
    let mut samples = Vec::with_capacity(SAMPLES);
    let mut work = Vec::with_capacity(SAMPLES);
    let cursor = mouse::Cursor::Available(Point::new(640., 300.));
    let mut cold = 0.;
    let mut stats = None;
    for frame in 0..WARMUP + SAMPLES {
        let start = Instant::now();
        let mut messages = Vec::new();
        ui.update(
            &[
                Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Lines {
                        x: 0.,
                        y: if frame % 6 < 3 { -1. } else { 1. },
                    },
                }),
                Event::Window(window::Event::RedrawRequested(
                    Instant::now() + Duration::from_secs(5),
                )),
            ],
            cursor,
            &mut renderer,
            &mut iced::advanced::clipboard::Null,
            &mut messages,
        );
        assert!(messages.is_empty());
        ui.draw(
            &mut renderer,
            &theme,
            &renderer::Style {
                text_color: theme.colors.on_surface,
            },
            cursor,
        );
        let ui_ms = start.elapsed().as_secs_f64() * 1000.;
        let start = Instant::now();
        stats = target.render(&mut renderer, &viewport, theme.colors.surface, mode);
        let render_ms = start.elapsed().as_secs_f64() * 1000.;
        if frame == 0 {
            cold = render_ms;
        }
        if frame >= WARMUP {
            samples.push(render_ms);
            work.push(ui_ms);
        }
    }
    if let Some(stats) = stats {
        eprintln!(
            "last frame: {} layers, {} damage regions, {:.0} logical pixels damaged (sum, may overlap)",
            stats.layers, stats.regions, stats.area
        );
    }
    samples.sort_by(f64::total_cmp);
    work.sort_by(f64::total_cmp);
    eprintln!(
        "{:9} {scene:14?} repaint median={:.3}ms p95={:.3}ms; UI median={:.3}ms cold={cold:.3}ms",
        if gpu.is_some() { "wgpu" } else { "tiny-skia" },
        samples[SAMPLES / 2],
        samples[SAMPLES * 95 / 100],
        work[SAMPLES / 2]
    );
}
#[test]
#[ignore = "release rendering benchmark; no wall-clock pass/fail thresholds"]
// Debug test builds must compile, but invoking this benchmark needs release.
#[allow(clippy::assertions_on_constants)]
fn profile_dialog_rendering() {
    assert!(!cfg!(debug_assertions), "Run this benchmark with --release");
    crate::fonts::ensure_loaded();
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    assert!(["tiny-skia", "wgpu"].contains(&backend.as_str()));
    #[cfg(feature = "wgpu")]
    let gpu = (backend == "wgpu").then(|| iced::futures::executor::block_on(Gpu::new()));
    #[cfg(not(feature = "wgpu"))]
    let gpu: Option<Gpu> = {
        assert_eq!(backend, "tiny-skia");
        None
    };
    let mode = Damage::from_env();
    let filter = std::env::var("ICED_PERF_SCENE").ok();
    eprintln!(
        "{WIDTH}x{HEIGHT} logical pixels, {SCALE}x scale; {WARMUP} warmup + {SAMPLES} measured scrolling frames; {mode:?} damage"
    );
    let mut matched = false;
    for scene in [
        Scene::Form,
        Scene::Panel,
        Scene::Scrim,
        Scene::DialogNoShadow,
        Scene::Dialog,
        Scene::NativeForm,
        Scene::NativeScrim,
    ] {
        if filter
            .as_ref()
            .is_none_or(|filter| format!("{scene:?}").eq_ignore_ascii_case(filter))
        {
            matched = true;
            run(scene, gpu.as_ref(), mode);
        }
    }
    assert!(matched, "No matching ICED_PERF_SCENE");
}

mod checks;
