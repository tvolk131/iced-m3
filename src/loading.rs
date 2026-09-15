//! Material Expressive loading indicator with canonical rounded shape morphs.
//! Uses generated AndroidX cubic pairs and the Android 650ms spring cadence.
use crate::{Element, Theme, tokens};
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, renderer,
    svg::{Handle, Renderer as _, Svg},
    widget::{Tree, tree},
};
use iced::{Event, Length, Rectangle, Renderer, Size, mouse, window};
use std::{
    cell::{Cell, RefCell},
    time::{Duration, Instant},
};
mod shapes;

/// License for the bundled canonical Material shape and morph data.
pub const LICENSE: &str = include_str!("../assets/loading/LICENSE-APACHE-2.0.txt");
/// Attribution to include in application acknowledgments when redistributing.
pub const NOTICE: &str = include_str!("../assets/loading/NOTICE");

#[cfg(test)]
pub(crate) fn reference_shape(shape: usize, amount: f32) -> Handle {
    Handle::from_memory(format!("<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 48 48'><path transform='translate(24 24) rotate(-90) scale(17)' d='{}'/></svg>", shapes::path(shape, amount)).into_bytes())
}
pub struct LoadingIndicator {
    size: f32,
    contained: bool,
    paused: bool,
}
pub fn loading_indicator() -> LoadingIndicator {
    LoadingIndicator {
        size: 48.0,
        contained: false,
        paused: false,
    }
}
impl LoadingIndicator {
    pub fn size(mut self, size: f32) -> Self {
        if size.is_finite() {
            self.size = size.max(8.0);
        }
        self
    }
    pub fn contained(mut self, contained: bool) -> Self {
        self.contained = contained;
        self
    }
    pub fn paused(mut self, paused: bool) -> Self {
        self.paused = paused;
        self
    }
}
struct State {
    elapsed: Duration,
    last: Option<Instant>,
    focused: bool,
    motion: Cell<tokens::Motion>,
    path: RefCell<Option<(u64, Handle)>>,
}
impl Widget<(), Theme, Renderer> for LoadingIndicator {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State {
            elapsed: Duration::ZERO,
            last: None,
            focused: true,
            motion: Cell::new(tokens::Motion::default()),
            path: RefCell::new(None),
        })
    }
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(self.size), Length::Fixed(self.size))
    }
    fn layout(&mut self, _: &mut Tree, _: &Renderer, l: &layout::Limits) -> layout::Node {
        layout::Node::new(l.resolve(self.size, self.size, Size::ZERO))
    }
    fn update(
        &mut self,
        t: &mut Tree,
        e: &Event,
        l: Layout<'_>,
        _: mouse::Cursor,
        _: &Renderer,
        _: &mut dyn Clipboard,
        s: &mut Shell<'_, ()>,
        v: &Rectangle,
    ) {
        let state = t.state.downcast_mut::<State>();
        if let Some(focused) = crate::activity::window_focus(e) {
            if state.focused != focused {
                state.last = None;
                if focused {
                    s.request_redraw();
                }
            }
            state.focused = focused;
        }
        if let Event::Window(window::Event::RedrawRequested(now)) = e {
            if state.focused
                && !self.paused
                && !state.motion.get().medium.is_zero()
                && l.bounds()
                    .intersection(v)
                    .is_some_and(|bounds| bounds.width > 0.0 && bounds.height > 0.0)
            {
                if let Some(last) = state.last {
                    state.elapsed += now.saturating_duration_since(last);
                }
                state.last = Some(*now);
                s.request_redraw();
            } else {
                state.last = None;
            }
        }
    }
    fn draw(
        &self,
        t: &Tree,
        r: &mut Renderer,
        theme: &Theme,
        _: &renderer::Style,
        l: Layout<'_>,
        _: mouse::Cursor,
        v: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;
        let Some(clip) = l.bounds().intersection(v) else {
            return;
        };
        let state = t.state.downcast_ref::<State>();
        state.motion.set(theme.motion);
        if self.contained {
            r.fill_quad(
                renderer::Quad {
                    bounds: l.bounds(),
                    border: iced::Border {
                        radius: (self.size / 2.0).into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                theme.colors.primary_container,
            );
        }
        let elapsed = if theme.motion.medium.is_zero() {
            Duration::ZERO
        } else {
            state.elapsed
        };
        let ms = elapsed.as_millis() as u64;
        let mut cached = state.path.borrow_mut();
        if cached.as_ref().is_none_or(|(old, _)| *old != ms) {
            let (shape, amount, rotation) = shapes::frame(elapsed.as_secs_f64());
            *cached=Some((ms,Handle::from_memory(format!("<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 48 48'><path transform='translate(24 24) rotate({}) scale(17)' d='{}'/></svg>",rotation-90.,shapes::path(shape,amount)).into_bytes())));
        }
        let color = if self.contained {
            theme.colors.on_primary_container
        } else {
            theme.colors.primary
        };
        r.draw_svg(
            Svg::from(&cached.as_ref().unwrap().1)
                .color(iced::Color { a: 1., ..color })
                .opacity(color.a),
            l.bounds(),
            clip,
        );
    }
}
impl<'a, Message: 'a> From<LoadingIndicator> for Element<'a, Message> {
    fn from(value: LoadingIndicator) -> Self {
        Element::<()>::new(value).map(|_| unreachable!("Loading indicators publish no messages"))
    }
}
