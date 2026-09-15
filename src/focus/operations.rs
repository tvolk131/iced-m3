use iced::advanced::widget::{
    Operation,
    operation::{Focusable, Scrollable, TextInput, scrollable::AbsoluteOffset},
};
use iced::{Rectangle, Vector, widget::Id};
use std::any::Any;

#[derive(Default)]
pub(super) struct Entries {
    pub roving: bool,
    pub items: Vec<(Option<Id>, Rectangle, bool)>,
}
pub(super) struct Traversal(pub bool);
impl Operation for Entries {
    fn custom(&mut self, _: Option<&Id>, _: Rectangle, state: &mut dyn Any) {
        if let Some(mode) = state.downcast_mut::<Traversal>() {
            mode.0 = self.roving;
        }
    }
    fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation)) {
        visit(self);
    }
    fn focusable(&mut self, id: Option<&Id>, bounds: Rectangle, state: &mut dyn Focusable) {
        self.items.push((id.cloned(), bounds, state.is_focused()));
    }
}
pub(super) struct Select {
    pub index: usize,
    pub current: usize,
    pub roving: bool,
}
impl Operation for Select {
    fn custom(&mut self, _: Option<&Id>, _: Rectangle, state: &mut dyn Any) {
        if let Some(mode) = state.downcast_mut::<Traversal>() {
            mode.0 = self.roving;
        }
    }
    fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation)) {
        visit(self);
    }
    fn focusable(&mut self, _: Option<&Id>, _: Rectangle, state: &mut dyn Focusable) {
        if self.current == self.index {
            state.focus();
        } else {
            state.unfocus();
        }
        self.current += 1;
    }
}
/// Only the active descendant participates in outer focus traversal. Other
/// operations (text lookup, scrolling, native editing) retain their full scope.
pub(super) struct Roving<'a> {
    pub inner: &'a mut dyn Operation,
    pub index: &'a mut usize,
    pub active: usize,
}
impl Operation for Roving<'_> {
    fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation)) {
        let active = self.active;
        let index = &mut *self.index;
        self.inner.traverse(&mut |inner| {
            visit(&mut Roving {
                inner,
                index,
                active,
            })
        });
    }
    fn focusable(&mut self, id: Option<&Id>, b: Rectangle, s: &mut dyn Focusable) {
        if *self.index == self.active {
            self.inner.focusable(id, b, s);
        }
        *self.index += 1;
    }
    fn container(&mut self, id: Option<&Id>, b: Rectangle) {
        self.inner.container(id, b);
    }
    fn scrollable(
        &mut self,
        id: Option<&Id>,
        b: Rectangle,
        c: Rectangle,
        t: Vector,
        s: &mut dyn Scrollable,
    ) {
        self.inner.scrollable(id, b, c, t, s);
    }
    fn text_input(&mut self, id: Option<&Id>, b: Rectangle, s: &mut dyn TextInput) {
        self.inner.text_input(id, b, s);
    }
    fn text(&mut self, id: Option<&Id>, b: Rectangle, text: &str) {
        self.inner.text(id, b, text);
    }
    fn custom(&mut self, id: Option<&Id>, b: Rectangle, s: &mut dyn Any) {
        self.inner.custom(id, b, s);
    }
}
#[derive(Clone, Copy)]
struct Scroll {
    index: usize,
    bounds: Rectangle,
    content: Rectangle,
    translation: Vector,
}
#[derive(Default)]
pub(super) struct Reveal {
    pub target: Option<usize>,
    focus_index: usize,
    pending: Option<Scroll>,
    path: Vec<Scroll>,
    count: usize,
    pub changes: Vec<(usize, AbsoluteOffset)>,
}
impl Reveal {
    pub(super) fn at(target: Option<usize>) -> Self {
        Self {
            target,
            ..Default::default()
        }
    }
}
impl Operation for Reveal {
    fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation)) {
        let pending = self.pending.take();
        if let Some(scroll) = pending {
            self.path.push(scroll);
        }
        visit(self);
        if pending.is_some() {
            self.path.pop();
        }
    }
    fn scrollable(
        &mut self,
        _: Option<&Id>,
        bounds: Rectangle,
        content: Rectangle,
        translation: Vector,
        _: &mut dyn Scrollable,
    ) {
        self.pending = Some(Scroll {
            index: self.count,
            bounds,
            content,
            translation,
        });
        self.count += 1;
    }
    fn focusable(&mut self, _: Option<&Id>, mut target: Rectangle, state: &mut dyn Focusable) {
        let selected = self
            .target
            .map_or(state.is_focused(), |index| index == self.focus_index);
        self.focus_index += 1;
        if !selected {
            return;
        }
        for scroll in self.path.iter().rev() {
            target = target - scroll.translation;
            let delta =
                |start: f32, size: f32, edge: f32, extent: f32, content: f32, offset: f32| {
                    let raw = if start < edge {
                        start - edge
                    } else if start + size > edge + extent {
                        (start + size - edge - extent).min(start - edge)
                    } else {
                        0.0
                    };
                    (offset + raw).clamp(0.0, (content - extent).max(0.0)) - offset
                };
            let x = delta(
                target.x,
                target.width,
                scroll.bounds.x,
                scroll.bounds.width,
                scroll.content.width,
                scroll.translation.x,
            );
            let y = delta(
                target.y,
                target.height,
                scroll.bounds.y,
                scroll.bounds.height,
                scroll.content.height,
                scroll.translation.y,
            );
            if x != 0.0 || y != 0.0 {
                self.changes.push((scroll.index, AbsoluteOffset { x, y }));
            }
            target = (target - Vector::new(x, y))
                .intersection(&scroll.bounds)
                .unwrap_or(scroll.bounds);
        }
    }
}
pub(super) struct ScrollChanges {
    pub changes: Vec<(usize, AbsoluteOffset)>,
    pub index: usize,
}
impl Operation for ScrollChanges {
    fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation)) {
        visit(self);
    }
    fn scrollable(
        &mut self,
        _: Option<&Id>,
        b: Rectangle,
        c: Rectangle,
        _: Vector,
        s: &mut dyn Scrollable,
    ) {
        if let Some((_, delta)) = self.changes.iter().find(|(i, _)| *i == self.index) {
            s.scroll_by(*delta, b, c);
        }
        self.index += 1;
    }
}
