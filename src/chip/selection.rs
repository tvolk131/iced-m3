//! Retained, passive icon slot: selection never rebuilds the caller's icon tree.
use crate::{Element, Theme, motion::Transition, tokens};
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, renderer,
    widget::{Tree, tree},
};
use iced::{Event, Length, Rectangle, Renderer, Size, Transformation, mouse, window};
use std::cell::Cell;

pub(super) fn icon<'a, Message: 'a>(
    content: Element<'a, Message>,
    selected: bool,
    enabled: bool,
) -> Element<'a, Message> {
    Element::new(Selection {
        content: [content, Element::new(crate::glyph::Glyph::Check)],
        selected,
        enabled,
    })
}
struct Selection<'a, Message> {
    content: [Element<'a, Message>; 2],
    selected: bool,
    enabled: bool,
}
struct State {
    progress: Transition,
    motion: Cell<tokens::Motion>,
}
impl<Message> Widget<Message, Theme, Renderer> for Selection<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State {
            progress: Transition::standard(f32::from(self.selected)),
            motion: Cell::new(tokens::Motion::default()),
        })
    }
    fn children(&self) -> Vec<Tree> {
        self.content.iter().map(Tree::new).collect()
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.content);
    }
    fn size(&self) -> Size<Length> {
        Size::new(18.into(), 18.into())
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits.resolve(18, 18, Size::new(18.0, 18.0));
        let limits = layout::Limits::new(Size::ZERO, size);
        let children = self
            .content
            .iter_mut()
            .zip(&mut tree.children)
            .map(|(child, tree)| {
                let node = child.as_widget_mut().layout(tree, renderer, &limits);
                let point = iced::Point::new(
                    (size.width - node.size().width) / 2.0,
                    (size.height - node.size().height) / 2.0,
                );
                node.move_to(point)
            })
            .collect();
        layout::Node::with_children(size, children)
    }
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        _: Layout<'_>,
        _: mouse::Cursor,
        _: &Renderer,
        _: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let now = match event {
            Event::Window(window::Event::RedrawRequested(now)) => *now,
            _ => crate::motion::now(),
        };
        let duration = if self.enabled {
            state.motion.get().short
        } else {
            std::time::Duration::ZERO
        };
        let before = state.progress.value;
        let changed = state.progress.set(f32::from(self.selected), now, duration);
        let active = state.progress.tick(now);
        if changed || active || state.progress.value != before {
            shell.request_redraw();
        }
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
        use iced::advanced::Renderer as _;
        let state = tree.state.downcast_ref::<State>();
        state.motion.set(theme.motion);
        let Some(clip) = layout.bounds().intersection(viewport) else {
            return;
        };
        let center = layout.bounds().center();
        renderer.with_layer(clip, |renderer| {
            for (index, scale) in [1.0 - state.progress.value, state.progress.value]
                .into_iter()
                .enumerate()
            {
                if scale <= 0.0 {
                    continue;
                }
                let transform = Transformation::translate(center.x, center.y)
                    * Transformation::scale(scale)
                    * Transformation::translate(-center.x, -center.y);
                renderer.with_transformation(transform, |renderer| {
                    self.content[index].as_widget().draw(
                        &tree.children[index],
                        renderer,
                        theme,
                        style,
                        layout.child(index),
                        cursor,
                        viewport,
                    );
                });
            }
        });
    }
}
