//! Direct widget-tree tests, separate from the visual suite's UserInterface runtime.
use iced::advanced::{Layout, Renderer as _, Shell, layout, renderer::Headless, widget::Tree};
use iced::{Event, Point, Rectangle, Size, Vector, mouse, window};
use iced_m3::{Element, Theme};

pub struct Harness<Message> {
    pub element: Element<'static, Message>,
    pub tree: Tree,
    pub node: layout::Node,
    pub renderer: iced::Renderer,
    pub size: Size,
    scale: f32,
}

impl<Message> Harness<Message> {
    pub fn new(element: impl Into<Element<'static, Message>>, size: Size, scale: f32) -> Self {
        let renderer = iced::futures::executor::block_on(<iced::Renderer as Headless>::new(
            iced::Font::DEFAULT,
            16.0.into(),
            Some("tiny-skia"),
        ))
        .unwrap();
        let mut element = element.into();
        let mut tree = Tree::new(&element);
        let node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, size),
        );
        Self {
            element,
            tree,
            node,
            renderer,
            size,
            scale,
        }
    }

    pub fn rebuild(&mut self, element: impl Into<Element<'static, Message>>) {
        self.element = element.into();
        self.tree.diff(&self.element);
        self.node = self.element.as_widget_mut().layout(
            &mut self.tree,
            &self.renderer,
            &layout::Limits::new(Size::ZERO, self.size),
        );
    }

    pub fn event(
        &mut self,
        event: Event,
        point: Option<Point>,
    ) -> (Vec<Message>, window::RedrawRequest) {
        let mut messages = Vec::new();
        let mut shell = Shell::new(&mut messages);
        self.element.as_widget_mut().update(
            &mut self.tree,
            &event,
            Layout::new(&self.node),
            point.map_or(mouse::Cursor::Unavailable, mouse::Cursor::Available),
            &self.renderer,
            &mut iced::advanced::clipboard::Null,
            &mut shell,
            &Rectangle::with_size(self.size),
        );
        let redraw = shell.redraw_request();
        (messages, redraw)
    }

    /// Translate both layout and viewport, then undo the translation in the renderer.
    /// This exercises drawing in a scrolled ancestor without moving the final pixels.
    pub fn frame(&mut self, offset: f32) -> Vec<u8> {
        let viewport = Rectangle::with_size(self.size);
        self.renderer.reset(viewport);
        self.renderer
            .with_translation(Vector::new(0.0, -offset), |renderer| {
                self.element.as_widget().draw(
                    &self.tree,
                    renderer,
                    &Theme::light(),
                    &Default::default(),
                    Layout::with_offset(Vector::new(0.0, offset), &self.node),
                    mouse::Cursor::Unavailable,
                    &(viewport + Vector::new(0.0, offset)),
                );
            });
        let scale = self.scale;
        self.renderer.screenshot(
            Size::new(
                (self.size.width * scale) as u32,
                (self.size.height * scale) as u32,
            ),
            scale,
            Theme::light().colors.surface,
        )
    }
}
