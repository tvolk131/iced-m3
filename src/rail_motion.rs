//! Reversible navigation-rail width without reflowing labels at every frame.
use crate::{Element, Theme, motion::Transition, tokens};
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, overlay, renderer,
    widget::{Operation, Tree, tree},
};
use iced::{Event, Length, Rectangle, Renderer, Size, Vector, mouse, window};
use std::cell::Cell;
pub(crate) fn animated_width<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    width: f32,
) -> Element<'a, Message> {
    Element::new(Rail {
        content: content.into(),
        width,
    })
}
struct Rail<'a, Message> {
    content: Element<'a, Message>,
    width: f32,
}
struct State {
    width: Transition,
    motion: Cell<tokens::Motion>,
    target: f32,
    cancel: bool,
}
impl<Message> Widget<Message, Theme, Renderer> for Rail<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State {
            width: Transition::standard(self.width),
            motion: Cell::new(tokens::Motion::default()),
            target: self.width,
            cancel: false,
        })
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, t: &mut Tree) {
        t.diff_children(std::slice::from_ref(&self.content));
        let s = t.state.downcast_mut::<State>();
        if s.target != self.width {
            s.target = self.width;
            s.cancel = true;
        }
    }
    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Fill)
    }
    fn layout(&mut self, t: &mut Tree, r: &Renderer, l: &layout::Limits) -> layout::Node {
        let current = t
            .state
            .downcast_ref::<State>()
            .width
            .value
            .min(l.max().width);
        let child = self.content.as_widget_mut().layout(
            &mut t.children[0],
            r,
            &layout::Limits::new(
                Size::ZERO,
                Size::new(self.width.min(l.max().width), l.max().height),
            ),
        );
        layout::Node::with_children(Size::new(current, child.size().height), vec![child])
    }
    fn operate(&mut self, t: &mut Tree, l: Layout<'_>, r: &Renderer, o: &mut dyn Operation) {
        self.content
            .as_widget_mut()
            .operate(&mut t.children[0], l.child(0), r, o);
    }
    fn update(
        &mut self,
        t: &mut Tree,
        e: &Event,
        l: Layout<'_>,
        c: mouse::Cursor,
        r: &Renderer,
        cb: &mut dyn Clipboard,
        s: &mut Shell<'_, Message>,
        v: &Rectangle,
    ) {
        let state = t.state.downcast_mut::<State>();
        let now = match e {
            Event::Window(window::Event::RedrawRequested(now)) => *now,
            _ => crate::motion::now(),
        };
        let before = state.width.value;
        let changed = state.width.set(self.width, now, state.motion.get().medium);
        if changed || state.width.tick(now) {
            s.request_redraw();
        }
        if before != state.width.value {
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
        let viewport = l.bounds().intersection(v).unwrap_or_default();
        self.content.as_widget_mut().update(
            &mut t.children[0],
            e,
            l.child(0),
            c,
            r,
            cb,
            s,
            &viewport,
        );
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
        if t.state.downcast_ref::<State>().width.value == self.width {
            self.content
                .as_widget()
                .draw(&t.children[0], r, th, s, l.child(0), c, v);
        } else if let Some(clip) = l.bounds().intersection(v) {
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
        self.content.as_widget().mouse_interaction(
            &t.children[0],
            l.child(0),
            c,
            &l.bounds().intersection(v).unwrap_or_default(),
            r,
        )
    }
    fn overlay<'b>(
        &'b mut self,
        t: &'b mut Tree,
        l: Layout<'b>,
        r: &Renderer,
        v: &Rectangle,
        tr: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut t.children[0],
            l.child(0),
            r,
            &l.bounds().intersection(v).unwrap_or_default(),
            tr,
        )
    }
}
