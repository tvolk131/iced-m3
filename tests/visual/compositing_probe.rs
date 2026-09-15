//! Renderer experiments only: none of these wrappers are shipped by the library.
//! Compare rectangular-band replay with a CPU offscreen group uploaded as an SVG
//! containing a PNG. The latter isolates pixels correctly but crosses a readback,
//! encoding, upload and second-rasterization boundary whenever content changes.
use super::{harness::Harness, reference::Image, theme};
use crate::{Element, Theme, TypeScale, text_field, typography};
use base64::Engine as _;
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, overlay, renderer,
    widget::{Operation, Tree, tree},
};
use iced::{Color, Event, Length, Rectangle, Renderer, Size, Vector, mouse, widget};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Mode {
    Direct,
    Bands,
    Offscreen,
}
#[derive(Clone, Debug, PartialEq)]
enum Message {
    Input(String),
    Action,
}
#[derive(Default)]
struct Counts {
    draws: Cell<usize>,
    captures: Cell<usize>,
}
struct Probe {
    content: Element<'static, Message>,
    mode: Mode,
    opacity: f32,
    radius: f32,
    counts: Rc<Counts>,
}
struct State {
    renderer: RefCell<Option<Renderer>>,
    image: RefCell<Option<iced::advanced::svg::Handle>>,
    key: RefCell<Option<(Size, Theme, Color)>>,
    dirty: Cell<bool>,
}
fn probe(
    content: impl Into<Element<'static, Message>>,
    mode: Mode,
    opacity: f32,
    counts: Rc<Counts>,
) -> Element<'static, Message> {
    Element::new(Probe {
        content: content.into(),
        mode,
        opacity,
        radius: 28.,
        counts,
    })
}
fn png(pixels: &[u8], size: Size<u32>) -> Vec<u8> {
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, size.width, size.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(png::Compression::Fast);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(pixels)
            .unwrap();
    }
    bytes
}
fn data_uri(pixels: &[u8], size: Size<u32>) -> String {
    format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(png(pixels, size))
    )
}
impl Widget<Message, Theme, Renderer> for Probe {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State {
            renderer: RefCell::new(None),
            image: RefCell::new(None),
            key: RefCell::new(None),
            dirty: Cell::new(true),
        })
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
        tree.state.downcast_ref::<State>().dirty.set(true);
    }
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        tree.state.downcast_ref::<State>().dirty.set(true);
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let mut messages = Vec::new();
        let mut local = Shell::new(&mut messages);
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            &mut local,
            viewport,
        );
        // Conservative invalidation also captures a transition's final redraw,
        // when the child has changed but no longer requests another frame.
        tree.state.downcast_ref::<State>().dirty.set(true);
        shell.merge(local, |message| message);
    }
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        use iced::advanced::{
            Renderer as _, renderer::Headless, svg::Renderer as _, text::Renderer as _,
        };
        let bounds = layout.bounds();
        let Some(clip) = bounds.intersection(viewport) else {
            return;
        };
        let radius = self.radius.min(bounds.width / 2.).min(bounds.height / 2.);
        let draw = |renderer: &mut Renderer, layer_clip: Rectangle| {
            self.counts.draws.set(self.counts.draws.get() + 1);
            renderer.with_layer(layer_clip, |renderer| {
                self.content.as_widget().draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    style,
                    layout,
                    cursor,
                    &bounds,
                )
            });
        };
        match self.mode {
            Mode::Direct => draw(renderer, clip),
            Mode::Bands => {
                // One logical-pixel row per band. This is an inscribed stepped
                // approximation, not an antialiased rounded-clip primitive.
                let rows = radius.ceil() as u32;
                for i in 0..rows {
                    let y = i as f32;
                    let height = (radius - y).min(1.);
                    let dy = radius - y - height / 2.;
                    let inset = radius - (radius * radius - dy * dy).max(0.).sqrt();
                    for y in [bounds.y + y, bounds.y + bounds.height - y - height] {
                        let band = Rectangle {
                            x: bounds.x + inset,
                            y,
                            width: (bounds.width - 2. * inset).max(0.),
                            height,
                        };
                        if let Some(band) = band.intersection(&clip) {
                            draw(renderer, band);
                        }
                    }
                }
                let center = Rectangle {
                    y: bounds.y + radius,
                    height: (bounds.height - 2. * radius).max(0.),
                    ..bounds
                };
                if let Some(center) = center.intersection(&clip) {
                    draw(renderer, center);
                }
            }
            Mode::Offscreen => {
                let state = tree.state.downcast_ref::<State>();
                let key = (bounds.size(), theme.clone(), style.text_color);
                if state.dirty.replace(false) || state.key.borrow().as_ref() != Some(&key) {
                    // Fixed scale is deliberate in this fixture: Widget::draw does
                    // not receive the window's display scale. A production layer
                    // needs the compositor's scale and resource lifecycle.
                    let scale = 2.;
                    let size = Size::new(
                        (bounds.width * scale).ceil() as u32,
                        (bounds.height * scale).ceil() as u32,
                    );
                    assert!(
                        size.width as u64 * size.height as u64 <= 8_000_000,
                        "probe allocation bound"
                    );
                    let mut offscreen = state.renderer.borrow_mut();
                    let offscreen = offscreen.get_or_insert_with(|| {
                        iced::futures::executor::block_on(<Renderer as Headless>::new(
                            renderer.default_font(),
                            renderer.default_size(),
                            Some("tiny-skia"),
                        ))
                        .unwrap()
                    });
                    offscreen.reset(Rectangle::with_size(bounds.size()));
                    offscreen
                        .with_translation(Vector::new(-bounds.x, -bounds.y), |r| draw(r, bounds));
                    let mut rgba = offscreen.screenshot(size, scale, Color::TRANSPARENT);
                    // Tiny Skia's headless bytes are premultiplied. PNG expects
                    // straight alpha; preserve partially transparent descendants.
                    for pixel in rgba.chunks_exact_mut(4) {
                        let alpha = pixel[3] as u32;
                        for channel in &mut pixel[..3] {
                            *channel = if alpha == 0 {
                                0
                            } else {
                                ((*channel as u32 * 255 + alpha / 2) / alpha).min(255) as u8
                            };
                        }
                    }
                    let uri = data_uri(&rgba, size);
                    let svg = format!(
                        "<svg xmlns='http://www.w3.org/2000/svg' xmlns:xlink='http://www.w3.org/1999/xlink' viewBox='0 0 {} {}'><defs><clipPath id='round'><rect width='{}' height='{}' rx='{radius}'/></clipPath></defs><image width='{}' height='{}' xlink:href='{uri}' clip-path='url(#round)'/></svg>",
                        bounds.width,
                        bounds.height,
                        bounds.width,
                        bounds.height,
                        bounds.width,
                        bounds.height
                    );
                    *state.image.borrow_mut() =
                        Some(iced::advanced::svg::Handle::from_memory(svg.into_bytes()));
                    *state.key.borrow_mut() = Some(key);
                    self.counts.captures.set(self.counts.captures.get() + 1);
                }
                renderer.with_layer(clip, |renderer| {
                    renderer.draw_svg(
                        iced::advanced::svg::Svg::from(state.image.borrow().as_ref().unwrap())
                            .opacity(self.opacity),
                        bounds,
                        clip,
                    )
                });
            }
        }
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }
    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut Tree,
        layout: Layout<'a>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}
fn host(content: impl Into<Element<'static, Message>>) -> Element<'static, Message> {
    widget::stack![
        widget::row((0..20).map(|i| {
            widget::container(widget::space())
                .width(40)
                .height(Length::Fill)
                .style(move |theme: &Theme| widget::container::Style {
                    background: Some(
                        if i % 2 == 0 {
                            theme.colors.secondary_container
                        } else {
                            theme.colors.surface_container_highest
                        }
                        .into(),
                    ),
                    ..Default::default()
                })
                .into()
        })),
        widget::container(content).padding(24),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
fn ui(
    content: impl Into<Element<'static, Message>>,
    size: Size,
    dark: bool,
) -> Harness<'static, Message> {
    Harness::with_backend(
        host(content),
        size,
        theme(dark),
        &std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into()),
    )
}
struct Overlap {
    solid: bool,
    translucent: bool,
}
impl Widget<Message, Theme, Renderer> for Overlap {
    fn size(&self) -> Size<Length> {
        Size::new(280.into(), 180.into())
    }
    fn layout(&mut self, _: &mut Tree, _: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::atomic(limits, 280, 180)
    }
    fn draw(
        &self,
        _: &Tree,
        r: &mut Renderer,
        _: &Theme,
        _: &renderer::Style,
        l: Layout<'_>,
        _: mouse::Cursor,
        v: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;
        r.with_layer(*v, |r| {
            if self.solid {
                r.fill_quad(
                    renderer::Quad {
                        bounds: l.bounds(),
                        ..Default::default()
                    },
                    Color::from_rgb8(220, 80, 60),
                );
            }
            for (x, y, color) in [
                (32., 32., Color::from_rgb8(220, 80, 60)),
                (
                    92.,
                    52.,
                    Color {
                        a: if self.translucent { 0.5 } else { 1.0 },
                        ..Color::from_rgb8(40, 100, 220)
                    },
                ),
            ] {
                r.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle::new(
                            l.position() + Vector::new(x, y),
                            Size::new(140., 100.),
                        ),
                        ..Default::default()
                    },
                    color,
                );
            }
        });
    }
}
fn art() -> Element<'static, Message> {
    let pixels = [
        230, 130, 50, 255, 60, 130, 220, 255, 80, 160, 80, 255, 230, 200, 80, 255,
    ];
    let uri = data_uri(&pixels, Size::new(2, 2));
    widget::svg(iced::advanced::svg::Handle::from_memory(format!("<svg xmlns='http://www.w3.org/2000/svg' xmlns:xlink='http://www.w3.org/1999/xlink' viewBox='0 0 96 64'><image width='96' height='64' xlink:href='{uri}'/></svg>").into_bytes())).width(96).height(64).into()
}
fn mixed(value: &str, width: f32, height: f32) -> Element<'static, Message> {
    widget::container(
        widget::column![
            typography("Workspace preview", TypeScale::TitleLarge),
            widget::row![
                art(),
                crate::Button::new(
                    widget::row![
                        super::icon::Icon,
                        typography("Apply", TypeScale::LabelLarge)
                    ]
                    .spacing(8)
                )
                .on_press(Message::Action)
            ]
            .spacing(16),
            widget::container(typography("Opaque child surface", TypeScale::BodyLarge))
                .padding(12)
                .width(Length::Fill)
                .style(|theme: &Theme| widget::container::Style {
                    background: Some(theme.colors.tertiary_container.into()),
                    ..Default::default()
                }),
            text_field("Name", value)
                .id("probe-name")
                .on_input(Message::Input),
            crate::linear_progress(0.)
                .indeterminate(true)
                .width(Length::Fill),
        ]
        .spacing(12),
    )
    .padding(20)
    .width(width)
    .height(height)
    .style(|theme: &Theme| widget::container::Style {
        background: Some(theme.colors.surface_container_high.into()),
        ..Default::default()
    })
    .into()
}
fn write(name: &str, image: &Image) {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    image.write(&std::path::PathBuf::from(format!(
        "target/compositing-probe/{backend}-{name}.png"
    )));
}
#[test]
fn offscreen_probe_clips_corners_and_composites_overlaps_as_one_group() {
    let size = Size::new(328., 228.);
    for dark in [false, true] {
        let empty = ui(widget::space().width(280).height(180), size, dark).frame();
        for solid in [false, true] {
            let counts = Rc::new(Counts::default());
            let mut view = ui(
                probe(
                    Element::new(Overlap {
                        solid,
                        translucent: false,
                    }),
                    Mode::Offscreen,
                    0.5,
                    counts.clone(),
                ),
                size,
                dark,
            );
            let frame = view.frame();
            write(&format!("overlap-{dark}-{solid}"), &frame);
            assert!(
                frame.crop(48, 48, 8, 8) == empty.crop(48, 48, 8, 8),
                "rounded corner leaves parent intact"
            );
            let overlap = frame.crop(290, 178, 8, 8);
            let blue_only = frame.crop(450, 178, 8, 8);
            assert!(
                overlap == blue_only,
                "foreground overlap must not show the red layer beneath the opaque blue layer"
            );
            if !solid {
                assert!(
                    frame.crop(550, 100, 8, 8) == empty.crop(550, 100, 8, 8),
                    "transparent group interior preserves parent"
                );
            }
            assert!(frame == view.frame());
            assert_eq!(
                counts.captures.get(),
                1,
                "unchanged content can reuse the raster"
            );
            view.rebuild(host(probe(
                Element::new(Overlap {
                    solid,
                    translucent: false,
                }),
                Mode::Offscreen,
                0.25,
                counts.clone(),
            )));
            assert!(frame != view.frame());
            assert_eq!(counts.captures.get(), 2);
        }
    }
}
#[test]
fn mixed_content_probe_keeps_native_editing_and_updates_live_children() {
    for dark in [false, true] {
        let counts = Rc::new(Counts::default());
        let mut view = ui(
            probe(
                mixed("Studio", 360., 320.),
                Mode::Offscreen,
                0.65,
                counts.clone(),
            ),
            Size::new(408., 368.),
            dark,
        );
        view.at(0);
        let first = view.frame();
        write(&format!("mixed-{dark}-start"), &first);
        view.at(300);
        let animated = view.frame();
        assert!(first != animated, "indeterminate child continues updating");
        view.click("Apply");
        assert_eq!(view.messages, [Message::Action]);
        view.messages.clear();
        view.operate(
            &mut iced::advanced::widget::operation::focusable::focus::<()>(widget::Id::new(
                "probe-name",
            )),
        );
        view.event(Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Character("x".into()),
            modified_key: iced::keyboard::Key::Character("x".into()),
            physical_key: iced::keyboard::key::Physical::Unidentified(
                iced::keyboard::key::NativeCode::Unidentified,
            ),
            location: iced::keyboard::Location::Standard,
            modifiers: iced::keyboard::Modifiers::empty(),
            text: Some("x".into()),
            repeat: false,
        }));
        assert!(
            view.messages
                .iter()
                .any(|message| matches!(message,Message::Input(value) if value.contains('x'))),
            "native editor must emit its input"
        );
        write(&format!("mixed-{dark}-active"), &view.frame());
        assert!(counts.captures.get() >= 3);
    }
}
#[test]
fn offscreen_probe_respects_ancestor_scrolling_and_clipping() {
    for dark in [false, true] {
        let content = || {
            probe(
                Element::new(Overlap {
                    solid: true,
                    translucent: false,
                }),
                Mode::Offscreen,
                0.65,
                Rc::new(Counts::default()),
            )
        };
        let size = Size::new(328., 228.);
        let expected = ui(content(), size, dark).frame();
        let scroll = widget::scrollable(widget::column![
            widget::space().height(160),
            content(),
            widget::space().height(160)
        ])
        .width(280)
        .height(180);
        let mut view = ui(scroll, size, dark);
        view.at(0);
        view.frame();
        view.move_to((100., 60.));
        view.event(Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Pixels { x: 0., y: -160. },
        }));
        let actual = view.frame();
        write(&format!("scrolled-{dark}"), &actual);
        assert!(
            actual.crop(48, 48, 500, 360) == expected.crop(48, 48, 500, 360),
            "local raster coordinates survive ancestor scrolling"
        );
    }
}

#[test]
#[ignore = "profiles experimental clipping/opacity approaches; not a production renderer"]
fn profile_compositing_approaches() {
    let render_overlap = |mode| {
        ui(
            probe(
                Element::new(Overlap {
                    solid: false,
                    translucent: true,
                }),
                mode,
                1.,
                Rc::new(Counts::default()),
            ),
            Size::new(328., 228.),
            true,
        )
        .frame()
    };
    let direct = render_overlap(Mode::Direct);
    let isolated = render_overlap(Mode::Offscreen);
    let a = direct.crop(290, 178, 1, 1);
    let b = isolated.crop(290, 178, 1, 1);
    eprintln!(
        "Translucent child RGB: native={:?}, CPU-isolated={:?}",
        &a.pixels[..3],
        &b.pixels[..3]
    );
    write("translucent-native", &direct);
    write("translucent-offscreen", &isolated);
    for (width, height) in [(280., 320.), (560., 520.)] {
        for mode in [Mode::Direct, Mode::Bands, Mode::Offscreen] {
            let counts = Rc::new(Counts::default());
            let size = Size::new(width + 48., height + 48.);
            let mut view = ui(
                probe(mixed("Studio", width, height), mode, 0.65, counts.clone()),
                size,
                true,
            );
            view.at(0);
            view.frame();
            let mut times = Vec::new();
            counts.draws.set(0);
            counts.captures.set(0);
            for ms in (16..=480).step_by(16) {
                let start = std::time::Instant::now();
                view.at(ms);
                view.frame();
                times.push(start.elapsed().as_secs_f64() * 1000.);
            }
            let mean = times.iter().sum::<f64>() / times.len() as f64;
            times.sort_by(f64::total_cmp);
            eprintln!(
                "{mode:?} {width}x{height}: mean={mean:.2}ms p95={:.2}ms child-draws={} captures={} (30 dynamic frames, 2x, includes final readback)",
                times[times.len() * 95 / 100],
                counts.draws.get(),
                counts.captures.get()
            );
            write(&format!("profile-{mode:?}-{width}"), &view.frame());
            let mut cached = Vec::new();
            for _ in 0..20 {
                let start = std::time::Instant::now();
                view.frame();
                cached.push(start.elapsed().as_secs_f64() * 1000.);
            }
            eprintln!(
                "{mode:?} {width}x{height} unchanged-frame mean={:.2}ms",
                cached.iter().sum::<f64>() / cached.len() as f64
            );
        }
    }
}
