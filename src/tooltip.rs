//! Noninteractive hints with delayed hover and shared popup placement.
pub use crate::anchored::Placement;
use crate::{Element, Theme, TypeScale, anchored::Anchored, typography};
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, overlay, renderer,
    widget::{Operation, Tree, tree},
};
use iced::time::{Duration, Instant};
use iced::{Border, Event, Rectangle, Renderer, Size, Vector, keyboard, mouse, widget, window};

pub struct Tooltip<'a, Message> {
    content: Element<'a, Message>,
    hint: Element<'a, Message>,
    delay: Duration,
    placement: Placement,
    disabled: bool,
}
pub fn tooltip<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    hint: impl widget::text::IntoFragment<'a>,
) -> Tooltip<'a, Message> {
    Tooltip {
        content: content.into(),
        hint: widget::container(typography(hint, TypeScale::BodySmall))
            .max_width(240)
            .padding([4, 8])
            .style(|theme: &Theme| widget::container::Style {
                background: Some(theme.colors.inverse_surface.into()),
                text_color: Some(theme.colors.inverse_on_surface),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into(),
        delay: Duration::from_millis(500),
        placement: Placement::Top,
        disabled: false,
    }
}
impl<Message> Tooltip<'_, Message> {
    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }
    /// Disable only the hint; the wrapped control remains functional.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}
#[derive(Default)]
struct State {
    since: Option<Instant>,
    open: bool,
    suppressed: bool,
    inactive: bool,
    presence: crate::presence::Presence,
}
impl<Message: Clone> Widget<Message, Theme, Renderer> for Tooltip<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content), Tree::new(&self.hint)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.children[0].diff(&self.content);
        tree.children[1].diff(&self.hint);
        if self.disabled {
            *tree.state.downcast_mut::<State>() = State::default();
        }
    }
    fn size(&self) -> Size<iced::Length> {
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
        let state = tree.state.downcast_mut::<State>();
        let now = match event {
            Event::Window(window::Event::RedrawRequested(now)) => *now,
            _ => crate::motion::now(),
        };
        let open = state.open;
        match event {
            Event::Window(window::Event::Unfocused) | Event::Mouse(mouse::Event::CursorLeft) => {
                state.inactive = true
            }
            Event::Window(window::Event::Focused) | Event::Mouse(_) => state.inactive = false,
            _ => {}
        }
        let over = !state.inactive
            && cursor.is_over(layout.bounds())
            && cursor.is_over(*viewport)
            && !matches!(
                event,
                Event::Mouse(mouse::Event::CursorLeft) | Event::Window(window::Event::Unfocused)
            );
        let dismiss = matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed(_)) | Event::Window(window::Event::Unfocused)
        ) || matches!(
            event,
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            })
        );
        if !over || self.disabled {
            state.since = None;
            state.open = false;
            state.suppressed = false;
        } else if dismiss {
            state.open = false;
            state.since = None;
            state.suppressed = true;
        } else if !state.suppressed {
            let since = *state.since.get_or_insert(now);
            if now.saturating_duration_since(since) >= self.delay {
                state.open = true;
            } else {
                shell.request_redraw_at(since + self.delay);
            }
        }
        if open != state.open {
            shell.invalidate_layout();
            shell.request_redraw();
        }
        let motion = state.presence.motion.get();
        state.presence.update(
            state.open,
            event,
            motion.tooltip_enter,
            motion.tooltip_exit,
            shell,
        );
        if dismiss || self.disabled {
            state.presence.progress = crate::motion::Transition::standard(0.0);
        }
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
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
        tree.state
            .downcast_ref::<State>()
            .presence
            .motion
            .set(theme.motion);
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
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
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let mut trees = tree.children.iter_mut();
        let content = self.content.as_widget_mut().overlay(
            trees.next().unwrap(),
            layout,
            renderer,
            viewport,
            translation,
        );
        let state = tree.state.downcast_ref::<State>();
        let hint = if state.presence.visible(state.open)
            && (layout.bounds() + translation)
                .intersection(viewport)
                .is_some()
        {
            Some(overlay::Element::new(Box::new(Anchored {
                content: &mut self.hint,
                tree: trees.next().unwrap(),
                anchor: layout.bounds() + translation,
                placement: self.placement,
                width: 240.0,
                max_height: f32::INFINITY,
                gap: 8.0,
                menu: None,
                close_when: None,
                labels: &[],
                reveal: Some(state.presence.progress.value),
                surface: None,
                plain_surface: false,
                surface_high: false,
            })))
        } else {
            None
        };
        if content.is_some() || hint.is_some() {
            Some(overlay::Group::with_children(content.into_iter().chain(hint).collect()).overlay())
        } else {
            None
        }
    }
}
impl<'a, Message: Clone + 'a> From<Tooltip<'a, Message>> for Element<'a, Message> {
    fn from(tooltip: Tooltip<'a, Message>) -> Self {
        Self::new(tooltip)
    }
}
