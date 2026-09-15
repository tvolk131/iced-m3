//! Finite size transitions used by extended actions and expandable action menus.
use crate::{Element, Theme, motion::Transition, tokens};
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, overlay, renderer,
    widget::{Operation, Tree, tree},
};
use iced::{Event, Length, Rectangle, Renderer, Size, Vector, mouse, window};
use std::cell::Cell;
pub(crate) fn reveal<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    open: bool,
    horizontal: bool,
) -> Element<'a, Message> {
    Element::new(Reveal {
        content: content.into(),
        open,
        horizontal,
    })
}
struct Reveal<'a, Message> {
    content: Element<'a, Message>,
    open: bool,
    horizontal: bool,
}
struct State {
    progress: Transition,
    motion: Cell<tokens::Motion>,
    was_open: bool,
    cancel: bool,
}
impl<Message> Widget<Message, Theme, Renderer> for Reveal<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State {
            progress: Transition::standard(f32::from(self.open)),
            motion: Cell::new(tokens::Motion::default()),
            was_open: self.open,
            cancel: false,
        })
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, t: &mut Tree) {
        t.diff_children(std::slice::from_ref(&self.content));
        let state = t.state.downcast_mut::<State>();
        if state.was_open && !self.open {
            state.cancel = true;
        }
        state.was_open = self.open;
    }
    fn size(&self) -> Size<Length> {
        let mut size = self.content.as_widget().size();
        if self.horizontal {
            size.width = Length::Shrink;
        } else {
            size.height = Length::Shrink;
        }
        size
    }
    fn layout(&mut self, t: &mut Tree, r: &Renderer, l: &layout::Limits) -> layout::Node {
        let child = self
            .content
            .as_widget_mut()
            .layout(&mut t.children[0], r, &l.loose());
        let mut size = child.size();
        let progress = t.state.downcast_ref::<State>().progress.value;
        if self.horizontal {
            size.width *= progress;
        } else {
            size.height *= progress;
        }
        layout::Node::with_children(size, vec![child])
    }
    fn operate(&mut self, t: &mut Tree, l: Layout<'_>, r: &Renderer, o: &mut dyn Operation) {
        if self.open {
            self.content
                .as_widget_mut()
                .operate(&mut t.children[0], l.child(0), r, o);
        }
    }
    fn update(
        &mut self,
        t: &mut Tree,
        event: &Event,
        l: Layout<'_>,
        c: mouse::Cursor,
        r: &Renderer,
        cb: &mut dyn Clipboard,
        s: &mut Shell<'_, Message>,
        v: &Rectangle,
    ) {
        let state = t.state.downcast_mut::<State>();
        let now = match event {
            Event::Window(window::Event::RedrawRequested(now)) => *now,
            _ => crate::motion::now(),
        };
        let before = state.progress.value;
        let changed = state
            .progress
            .set(f32::from(self.open), now, state.motion.get().medium);
        if changed || state.progress.tick(now) {
            s.request_redraw();
        }
        if before != state.progress.value {
            s.invalidate_layout();
        }
        if state.cancel {
            state.cancel = false;
            let mut ignored = Vec::new();
            self.content.as_widget_mut().update(
                &mut t.children[0],
                &Event::Window(window::Event::Unfocused),
                l.child(0),
                mouse::Cursor::Unavailable,
                r,
                cb,
                &mut Shell::new(&mut ignored),
                v,
            );
        }
        if self.open {
            self.content.as_widget_mut().update(
                &mut t.children[0],
                event,
                l.child(0),
                c,
                r,
                cb,
                s,
                &l.bounds().intersection(v).unwrap_or_default(),
            );
        }
    }
    fn draw(
        &self,
        t: &Tree,
        r: &mut Renderer,
        th: &Theme,
        s: &renderer::Style,
        l: Layout<'_>,
        c: mouse::Cursor,
        v: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;
        t.state.downcast_ref::<State>().motion.set(th.motion);
        if let Some(clip) = l.bounds().intersection(v) {
            r.with_layer(clip, |r| {
                self.content
                    .as_widget()
                    .draw(&t.children[0], r, th, s, l.child(0), c, &clip)
            });
        }
    }
    fn mouse_interaction(
        &self,
        t: &Tree,
        l: Layout<'_>,
        c: mouse::Cursor,
        v: &Rectangle,
        r: &Renderer,
    ) -> mouse::Interaction {
        if self.open {
            self.content.as_widget().mouse_interaction(
                &t.children[0],
                l.child(0),
                c,
                &l.bounds().intersection(v).unwrap_or_default(),
                r,
            )
        } else {
            mouse::Interaction::None
        }
    }
    fn overlay<'b>(
        &'b mut self,
        t: &'b mut Tree,
        l: Layout<'b>,
        r: &Renderer,
        v: &Rectangle,
        tr: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        if self.open {
            self.content
                .as_widget_mut()
                .overlay(&mut t.children[0], l.child(0), r, v, tr)
        } else {
            None
        }
    }
}
