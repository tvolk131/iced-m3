use super::{Theme, reference::Image, with_time};
use iced::advanced::{renderer::Headless, widget::Operation};
use iced::{Event, Point, Renderer, Size, mouse, window};
use iced_test::{
    Selector,
    runtime::{UserInterface, user_interface},
};
use std::time::{Duration, Instant};

/// Uses iced's actual UI runtime, including nested overlays and layout invalidation.
/// Capturing pixels never sends an event or reads the wall clock.
pub struct Harness<'a, Message> {
    ui: Option<UserInterface<'a, Message, Theme, Renderer>>,
    renderer: Renderer,
    size: Size,
    theme: Theme,
    origin: Instant,
    elapsed: u64,
    cursor: mouse::Cursor,
    pub messages: Vec<Message>,
}
impl<'a, Message> Harness<'a, Message> {
    pub fn new(
        element: impl Into<iced::Element<'a, Message, Theme>>,
        size: Size,
        theme: Theme,
    ) -> Self {
        Self::with_backend(element, size, theme, "tiny-skia")
    }
    pub fn with_backend(
        element: impl Into<iced::Element<'a, Message, Theme>>,
        size: Size,
        theme: Theme,
        backend: &str,
    ) -> Self {
        let mut renderer = iced::futures::executor::block_on(<Renderer as Headless>::new(
            iced::Font::with_name("Roboto"),
            16.0.into(),
            Some(backend),
        ))
        .expect("headless renderer");
        let ui = UserInterface::build(element, size, Default::default(), &mut renderer);
        Self {
            ui: Some(ui),
            renderer,
            size,
            theme,
            origin: Instant::now(),
            elapsed: 0,
            cursor: mouse::Cursor::Unavailable,
            messages: Vec::new(),
        }
    }
    pub fn rebuild(&mut self, element: impl Into<iced::Element<'a, Message, Theme>>) {
        let cache = self.ui.take().unwrap().into_cache();
        self.ui = Some(UserInterface::build(
            element,
            self.size,
            cache,
            &mut self.renderer,
        ));
    }
    pub fn resize(&mut self, size: Size) {
        self.size = size;
        self.ui = Some(self.ui.take().unwrap().relayout(size, &mut self.renderer));
    }
    pub fn event(&mut self, event: Event) -> window::RedrawRequest {
        let now = self.origin + Duration::from_millis(self.elapsed);
        let (state, _) = with_time(now, || {
            self.ui.as_mut().unwrap().update(
                &[event],
                self.cursor,
                &mut self.renderer,
                &mut iced::advanced::clipboard::Null,
                &mut self.messages,
            )
        });
        match state {
            user_interface::State::Updated { redraw_request, .. } => redraw_request,
            user_interface::State::Outdated => window::RedrawRequest::NextFrame,
        }
    }
    pub fn at(&mut self, milliseconds: u64) -> window::RedrawRequest {
        assert!(
            milliseconds >= self.elapsed,
            "test time must never go backwards"
        );
        self.elapsed = milliseconds;
        self.event(Event::Window(window::Event::RedrawRequested(
            self.origin + Duration::from_millis(milliseconds),
        )))
    }
    pub fn move_to(&mut self, point: impl Into<Point>) {
        let position = point.into();
        self.cursor = mouse::Cursor::Available(position);
        self.event(Event::Mouse(mouse::Event::CursorMoved { position }));
    }
    pub fn leave(&mut self) {
        self.cursor = mouse::Cursor::Unavailable;
        self.event(Event::Mouse(mouse::Event::CursorLeft));
    }
    pub fn down(&mut self) {
        self.event(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )));
    }
    pub fn up(&mut self) {
        self.event(Event::Mouse(mouse::Event::ButtonReleased(
            mouse::Button::Left,
        )));
    }
    pub fn find(&mut self, label: &str) -> iced::Rectangle {
        let mut op = Selector::find(label);
        self.ui.as_mut().unwrap().operate(
            &self.renderer,
            &mut iced::advanced::widget::operation::black_box(&mut op),
        );
        match op.finish() {
            iced::advanced::widget::operation::Outcome::Some(Some(found)) => {
                found.visible_bounds().expect("visible target")
            }
            _ => panic!("Missing target: {label}"),
        }
    }
    pub fn operate(&mut self, operation: &mut dyn Operation) {
        self.ui.as_mut().unwrap().operate(&self.renderer, operation);
    }
    pub fn click(&mut self, label: &str) {
        let point = self.find(label).center();
        self.move_to(point);
        self.down();
        self.up();
    }
    pub fn frame(&mut self) -> Image {
        self.ui.as_mut().unwrap().draw(
            &mut self.renderer,
            &self.theme,
            &iced::advanced::renderer::Style {
                text_color: self.theme.colors.on_surface,
            },
            self.cursor,
        );
        let size = Size::new(
            (self.size.width * 2.0) as u32,
            (self.size.height * 2.0) as u32,
        );
        Image {
            width: size.width,
            height: size.height,
            pixels: self
                .renderer
                .screenshot(size, 2.0, self.theme.colors.surface),
        }
    }
}
