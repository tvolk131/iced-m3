//! Transient feedback hosted inside the root modal host. No application timer.
use crate::{ButtonVariant, Element, Theme, TypeScale, button, tokens, typography};
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, overlay, renderer,
    widget::{Operation, Tree, tree},
};
use iced::time::{Duration, Instant};
use iced::{Color, Event, Length, Point, Rectangle, Renderer, Size, Vector, mouse, widget, window};

pub struct Snackbar<Message> {
    text: String,
    id: u64,
    action: Option<(String, Message)>,
    dismiss: Option<Message>,
    duration: Option<Duration>,
    visible: bool,
}
/// A concise message. Add `on_dismiss` for automatic timeout; otherwise it persists.
pub fn snackbar<Message>(text: impl Into<String>) -> Snackbar<Message> {
    Snackbar {
        text: text.into(),
        id: 0,
        action: None,
        dismiss: None,
        duration: Some(Duration::from_secs(4)),
        visible: true,
    }
}
impl<Message> Snackbar<Message> {
    /// Keep the current notice mounted with `visible(false)` for animated removal.
    /// Passing `None` to the host removes it immediately.
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
    /// Change this ID when showing the same text as a new notification.
    pub fn id(mut self, id: u64) -> Self {
        self.id = id;
        self
    }
    pub fn action(mut self, label: impl Into<String>, message: Message) -> Self {
        self.action = Some((label.into(), message));
        self
    }
    pub fn on_dismiss(mut self, message: Message) -> Self {
        self.dismiss = Some(message);
        self
    }
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }
    pub fn persistent(mut self) -> Self {
        self.duration = None;
        self
    }
}
struct Notice<'a, Message> {
    bar: Element<'a, Message>,
    text: String,
    id: u64,
    dismiss: Option<Message>,
    duration: Option<Duration>,
    visible: bool,
}
struct Host<'a, Message> {
    background: Element<'a, Message>,
    notice: Option<Notice<'a, Message>>,
}
/// Keep this host mounted. Place it *inside* `dialog::modal` so a modal covers it.
/// The application owns the current notice; timeout publishes its dismissal message once.
pub fn host<'a, Message: Clone + 'a>(
    background: impl Into<Element<'a, Message>>,
    notice: Option<Snackbar<Message>>,
) -> Element<'a, Message> {
    let notice = notice.map(|notice| {
        let mut row = widget::row![
            widget::container(typography(notice.text.clone(), TypeScale::BodyMedium))
                .padding([10, 0])
                .width(Length::Fill)
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);
        if let Some((label, message)) = notice.action {
            row = row.push(
                button(label)
                    .variant(ButtonVariant::Text)
                    .padding([10, 12])
                    .palette(|theme| (Color::TRANSPARENT, theme.colors.inverse_primary))
                    .on_press(message),
            );
        }
        let bar = crate::elevation::elevated(
            widget::container(row)
                .padding([4, 16])
                .width(Length::Fill)
                .style(|theme: &Theme| widget::container::Style {
                    background: Some(theme.colors.inverse_surface.into()),
                    text_color: Some(theme.colors.inverse_on_surface),
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },

                    ..Default::default()
                }),
            3.0,
            4.0,
        );
        Notice {
            bar,
            text: notice.text,
            id: notice.id,
            duration: notice.duration.filter(|_| notice.dismiss.is_some()),
            dismiss: notice.dismiss,
            visible: notice.visible,
        }
    });
    Element::new(Host {
        background: background.into(),
        notice,
    })
}
#[derive(Default)]
struct State {
    identity: Option<(u64, String, Option<Duration>)>,
    remaining: Option<Duration>,
    last: Option<Instant>,
    expired: bool,
    paused: bool,
    inactive: bool,
    presence: crate::presence::Presence,
}
impl State {
    fn for_notice<Message>(notice: Option<&Notice<'_, Message>>) -> Self {
        Self {
            identity: notice.map(|n| (n.id, n.text.clone(), n.duration)),
            remaining: notice.and_then(|n| n.duration),
            presence: crate::presence::Presence::new(notice.is_some_and(|n| n.visible)),
            ..Default::default()
        }
    }
}
impl<Message: Clone> Widget<Message, Theme, Renderer> for Host<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::for_notice(self.notice.as_ref()))
    }
    fn children(&self) -> Vec<Tree> {
        let mut children = vec![Tree::new(&self.background)];
        if let Some(notice) = &self.notice {
            children.push(Tree::new(&notice.bar));
        }
        children
    }
    fn diff(&self, tree: &mut Tree) {
        tree.children[0].diff(&self.background);
        let mut fresh = State::for_notice(self.notice.as_ref());
        let state = tree.state.downcast_mut::<State>();
        fresh.inactive = state.inactive;
        fresh.paused = state.inactive;
        let changed = state.identity != fresh.identity;
        if changed {
            fresh.presence.progress =
                crate::motion::Transition::standard(if self.notice.is_some() {
                    state.presence.progress.value
                } else {
                    0.0
                });
            fresh.presence.motion.set(state.presence.motion.get());
            *state = fresh;
        }
        if let Some(notice) = &self.notice {
            if tree.children.len() == 1 {
                tree.children.push(Tree::new(&notice.bar));
            } else if changed {
                tree.children[1] = Tree::new(&notice.bar);
            } else {
                tree.children[1].diff(&notice.bar);
            }
        } else {
            tree.children.truncate(1);
        }
    }
    fn size(&self) -> Size<Length> {
        self.background.as_widget().size()
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let background =
            self.background
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, limits);
        let size = background.size();
        let mut children = vec![background];
        if let Some(notice) = &mut self.notice {
            let margin = 16.0_f32.min(size.width / 8.0).min(size.height / 8.0);
            let bar = notice.bar.as_widget_mut().layout(
                &mut tree.children[1],
                renderer,
                &layout::Limits::new(
                    Size::ZERO,
                    Size::new(
                        tokens::size::SNACKBAR_MAX.min((size.width - 2.0 * margin).max(0.0)),
                        (size.height - 2.0 * margin).max(0.0),
                    ),
                ),
            );
            let bar_size = bar.size();
            children.push(bar.move_to(Point::new(
                (size.width - bar_size.width) / 2.0,
                size.height - margin - bar_size.height
                    + (bar_size.height + margin)
                        * (1.0 - tree.state.downcast_ref::<State>().presence.progress.value),
            )));
        }
        layout::Node::with_children(size, children)
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.background.as_widget_mut().operate(
                &mut tree.children[0],
                layout.child(0),
                renderer,
                operation,
            );
            if let Some(notice) = &mut self.notice
                && !tree.state.downcast_ref::<State>().expired
                && notice.visible
            {
                notice.bar.as_widget_mut().operate(
                    &mut tree.children[1],
                    layout.child(1),
                    renderer,
                    operation,
                );
            }
        });
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
        let active = self.notice.as_ref().is_some_and(|n| n.visible) && !state.expired;
        let motion = state.presence.motion.get();
        state.presence.update(
            active,
            event,
            motion.snackbar_enter,
            motion.snackbar_exit,
            shell,
        );
        match event {
            Event::Window(window::Event::Unfocused) => state.inactive = true,
            Event::Window(window::Event::Focused) => state.inactive = false,
            _ => {}
        }
        let mut over = false;
        if let Some(notice) = &mut self.notice
            && !state.expired
            && notice.visible
        {
            let bounds = layout.child(1).bounds();
            over = cursor.is_over(bounds) && cursor.is_over(*viewport);
            let now = match event {
                Event::Window(window::Event::RedrawRequested(now)) => *now,
                _ => crate::motion::now(),
            };
            if let Some(remaining) = &mut state.remaining
                && let Some(last) = state.last
                && !state.paused
            {
                *remaining = remaining.saturating_sub(now.saturating_duration_since(last));
            }
            state.last = Some(now);
            state.paused = state.inactive
                || (over && !matches!(event, Event::Mouse(mouse::Event::CursorLeft)));
            if !state.paused && state.remaining.is_some_and(|remaining| remaining.is_zero()) {
                state.expired = true;
                if let Some(message) = &notice.dismiss {
                    shell.publish(message.clone());
                }
                shell.request_redraw();
            } else {
                let mut messages = Vec::new();
                let mut local = Shell::new(&mut messages);
                notice.bar.as_widget_mut().update(
                    &mut tree.children[1],
                    event,
                    layout.child(1),
                    cursor,
                    renderer,
                    clipboard,
                    &mut local,
                    viewport,
                );
                if !local.is_empty() {
                    state.expired = true;
                    shell.request_redraw();
                }
                shell.merge(local, std::convert::identity);
                if !state.expired
                    && !state.paused
                    && let Some(remaining) = state.remaining
                {
                    shell.request_redraw_at(now + remaining);
                }
            }
        }
        let active = self.notice.as_ref().is_some_and(|n| n.visible) && !state.expired;
        state.presence.update(
            active,
            event,
            motion.snackbar_enter,
            motion.snackbar_exit,
            shell,
        );
        if self.notice.is_some() && state.presence.visible(active) && !active {
            over = cursor.is_over(layout.child(1).bounds()) && cursor.is_over(*viewport);
            let mut ignored = Vec::new();
            self.notice.as_mut().unwrap().bar.as_widget_mut().update(
                &mut tree.children[1],
                &Event::Window(window::Event::Unfocused),
                layout.child(1),
                mouse::Cursor::Unavailable,
                renderer,
                clipboard,
                &mut Shell::new(&mut ignored),
                viewport,
            );
        }
        if !active {
            state.paused = true;
            state.last = Some(match event {
                Event::Window(window::Event::RedrawRequested(now)) => *now,
                _ => crate::motion::now(),
            });
        }
        if over && matches!(event, Event::Mouse(_) | Event::Touch(_)) {
            // Deliver the end of a gesture to the covered content without a hit
            // target. This clears an earlier press without clicking through.
            let mut ignored = Vec::new();
            let mut local = Shell::new(&mut ignored);
            self.background.as_widget_mut().update(
                &mut tree.children[0],
                event,
                layout.child(0),
                mouse::Cursor::Unavailable,
                renderer,
                clipboard,
                &mut local,
                viewport,
            );
            crate::anchored::forward_shell(shell, &local);
            shell.capture_event();
            return;
        }
        self.background.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.child(0),
            if over {
                mouse::Cursor::Unavailable
            } else {
                cursor
            },
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
        let state = tree.state.downcast_ref::<State>();
        state.presence.motion.set(theme.motion);
        let active = self.notice.as_ref().is_some_and(|n| n.visible) && !state.expired;
        let visible = self.notice.is_some() && state.presence.visible(active);
        let over = visible && cursor.is_over(layout.child(1).bounds());
        self.background.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout.child(0),
            if over {
                mouse::Cursor::Unavailable
            } else {
                cursor
            },
            viewport,
        );
        if visible && let Some(notice) = &self.notice {
            use iced::advanced::Renderer as _;
            // Like iced's Stack, isolate the foreground in its own layer.
            // Renderers batch text and quads within a layer; draw-call order
            // alone would let the page's text paint over the bar's background.
            renderer.with_layer(*viewport, |renderer| {
                notice.bar.as_widget().draw(
                    &tree.children[1],
                    renderer,
                    theme,
                    style,
                    layout.child(1),
                    if active {
                        cursor
                    } else {
                        mouse::Cursor::Unavailable
                    },
                    viewport,
                );
            });
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
        if let Some(notice) = &self.notice
            && !tree.state.downcast_ref::<State>().expired
            && notice.visible
            && cursor.is_over(layout.child(1).bounds())
        {
            return notice.bar.as_widget().mouse_interaction(
                &tree.children[1],
                layout.child(1),
                cursor,
                viewport,
                renderer,
            );
        }
        self.background.as_widget().mouse_interaction(
            &tree.children[0],
            layout.child(0),
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
        self.background.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.child(0),
            renderer,
            viewport,
            translation,
        )
    }
}
