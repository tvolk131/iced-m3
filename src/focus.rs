//! Keyboard traversal for Material and native iced focusable widgets.
use crate::{Element, Theme};
mod operations;
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, overlay, renderer,
    widget::{
        Operation, Tree,
        operation::focusable::{self, Focusable},
        tree,
    },
};
use iced::{Event, Length, Rectangle, Renderer, Size, Vector, keyboard, mouse, widget};
use operations::{Entries, Reveal, Roving, ScrollChanges, Select, Traversal};

#[derive(Debug)]
pub(crate) struct Focus {
    pub focused: bool,
    pub visible: bool,
    pub suppressed: bool,
    pub key: Option<keyboard::key::Named>,
    pub id: widget::Id,
}
impl Default for Focus {
    fn default() -> Self {
        Self {
            focused: false,
            visible: false,
            suppressed: false,
            key: None,
            id: widget::Id::unique(),
        }
    }
}
impl Focusable for Focus {
    fn is_focused(&self) -> bool {
        self.focused
    }
    fn focus(&mut self) {
        self.focused = true;
        self.visible = true;
    }
    fn unfocus(&mut self) {
        self.focused = false;
        self.visible = false;
        self.key = None;
    }
}
fn apply<Message>(
    content: &mut Element<'_, Message>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
    mut op: Box<dyn Operation>,
) {
    loop {
        content
            .as_widget_mut()
            .operate(tree, layout, renderer, op.as_mut());
        match op.finish() {
            iced::advanced::widget::operation::Outcome::Chain(next) => op = next,
            _ => break,
        }
    }
}
/// Handle Tab inside an overlay or a focus scope. Traversal wraps within it.
pub(crate) fn tab<Message>(
    content: &mut Element<'_, Message>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
    event: &Event,
    shell: &mut Shell<'_, Message>,
) -> bool {
    if let Event::Keyboard(keyboard::Event::KeyPressed {
        key: keyboard::Key::Named(keyboard::key::Named::Tab),
        modifiers,
        ..
    }) = event
    {
        cycle(content, tree, layout, renderer, modifiers.shift());
        shell.capture_event();
        shell.request_redraw();
        true
    } else {
        false
    }
}
pub(crate) fn cycle<Message>(
    content: &mut Element<'_, Message>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
    reverse: bool,
) {
    let mut query = Entries {
        roving: true,
        ..Default::default()
    };
    content
        .as_widget_mut()
        .operate(tree, layout, renderer, &mut query);
    let total = query.items.len();
    if total > 0 {
        let focused = query.items.iter().position(|(_, _, focused)| *focused);
        let next = match (focused, reverse) {
            (Some(i), false) => (i + 1) % total,
            (Some(i), true) => (i + total - 1) % total,
            (None, false) => 0,
            (None, true) => total - 1,
        };
        select(content, tree, layout, renderer, next);
    }
}

pub(crate) fn clear<Message>(
    content: &mut Element<'_, Message>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
) {
    apply(
        content,
        tree,
        layout,
        renderer,
        Box::new(focusable::unfocus::<()>()),
    );
}
/// Reveal the focused descendant in every enclosing scrollable, including
/// nested horizontal/vertical scrolling. Only scroll as far as needed.
pub(crate) fn reveal<Message>(
    content: &mut Element<'_, Message>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
) {
    reveal_at(content, tree, layout, renderer, None);
}
fn reveal_at<Message>(
    content: &mut Element<'_, Message>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
    index: Option<usize>,
) {
    let mut query = Reveal::at(index);
    content
        .as_widget_mut()
        .operate(tree, layout, renderer, &mut query);
    if !query.changes.is_empty() {
        content.as_widget_mut().operate(
            tree,
            layout,
            renderer,
            &mut ScrollChanges {
                changes: query.changes,
                index: 0,
            },
        );
    }
}
pub(crate) fn has_focus<Message>(
    content: &mut Element<'_, Message>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
) -> bool {
    let mut query = Entries::default();
    content
        .as_widget_mut()
        .operate(tree, layout, renderer, &mut query);
    query.items.iter().any(|(_, _, focused)| *focused)
}
pub(crate) fn focused_index<Message>(
    content: &mut Element<'_, Message>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
) -> Option<usize> {
    let mut query = Entries::default();
    content
        .as_widget_mut()
        .operate(tree, layout, renderer, &mut query);
    query.items.iter().position(|(_, _, focused)| *focused)
}

#[derive(Clone)]
pub(crate) struct Bookmark {
    id: Option<widget::Id>,
    index: usize,
    cursor: Option<widget::text_input::cursor::State>,
}
type NativeInput =
    widget::text_input::State<<Renderer as iced::advanced::text::Renderer>::Paragraph>;
fn native_cursor(
    tree: &mut Tree,
    restore: Option<widget::text_input::cursor::State>,
) -> Option<widget::text_input::cursor::State> {
    if tree.tag == tree::Tag::of::<NativeInput>() {
        let state = tree.state.downcast_mut::<NativeInput>();
        if state.is_focused() {
            let cursor = state.cursor().state(&widget::text_input::Value::new(
                iced::advanced::widget::operation::TextInput::text(state),
            ));
            if let Some(restore) = restore {
                match restore {
                    widget::text_input::cursor::State::Index(index) => state.move_cursor_to(index),
                    widget::text_input::cursor::State::Selection { start, end } => {
                        state.select_range(start, end)
                    }
                }
            }
            return Some(cursor);
        }
    }
    tree.children
        .iter_mut()
        .find_map(|child| native_cursor(child, restore))
}
pub(crate) fn remember<Message>(
    content: &mut Element<'_, Message>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
) -> Option<Bookmark> {
    let mut query = Entries {
        roving: true,
        ..Default::default()
    };
    content
        .as_widget_mut()
        .operate(tree, layout, renderer, &mut query);
    query
        .items
        .iter()
        .enumerate()
        .find(|(_, (_, _, focused))| *focused)
        .map(|(index, (id, _, _))| Bookmark {
            id: id.clone(),
            index,
            cursor: native_cursor(tree, None),
        })
}
pub(crate) fn restore<Message>(
    bookmark: &Bookmark,
    content: &mut Element<'_, Message>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
) {
    let mut query = Entries {
        roving: true,
        ..Default::default()
    };
    content
        .as_widget_mut()
        .operate(tree, layout, renderer, &mut query);
    let index = bookmark
        .id
        .as_ref()
        .and_then(|id| {
            query
                .items
                .iter()
                .position(|(other, _, _)| other.as_ref() == Some(id))
        })
        .unwrap_or(bookmark.index.min(query.items.len().saturating_sub(1)));
    select(content, tree, layout, renderer, index);
    if let Some(cursor) = bookmark.cursor {
        // Native iced focus() moves the caret to the end. Restore its public
        // cursor/selection state after focus, without replacing the native editor.
        native_cursor(tree, Some(cursor));
    }
}
pub(crate) fn select<Message>(
    content: &mut Element<'_, Message>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
    index: usize,
) {
    content.as_widget_mut().operate(
        tree,
        layout,
        renderer,
        &mut Select {
            index,
            current: 0,
            roving: true,
        },
    );
    reveal(content, tree, layout, renderer);
}
pub(crate) fn select_all<Message>(
    content: &mut Element<'_, Message>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
    index: usize,
) {
    content.as_widget_mut().operate(
        tree,
        layout,
        renderer,
        &mut Select {
            index,
            current: 0,
            roving: false,
        },
    );
    reveal(content, tree, layout, renderer);
}
/// Wrap the app root to enable Tab/Shift+Tab traversal. Modal components provide
/// their own scope, keeping focus out of covered content.
pub fn scope<'a, Message: 'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    Element::new(Scope {
        content: content.into(),
        arrows: false,
        active: 0,
        grid: false,
    })
}
/// A single Tab stop whose enabled controls can be traversed with arrow keys.
/// Enter/Space activate. Focus returns to the most recently focused item.
pub fn group<'a, Message: 'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    group_with_active(content, 0)
}
/// Like [`group`], with an initial active index among the enabled controls.
/// Changing this index changes the group's next entry point without stealing focus.
pub fn group_with_active<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    active: usize,
) -> Element<'a, Message> {
    Element::new(Scope {
        content: content.into(),
        arrows: true,
        active,
        grid: false,
    })
}
pub(crate) fn grid<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    active: usize,
) -> Element<'a, Message> {
    Element::new(Scope {
        content: content.into(),
        arrows: true,
        active,
        grid: true,
    })
}
struct Scope<'a, Message> {
    content: Element<'a, Message>,
    arrows: bool,
    active: usize,
    grid: bool,
}
#[derive(Default)]
struct State {
    remembered: usize,
    preferred: usize,
    reveal_pending: bool,
}
impl<Message> Widget<Message, Theme, Renderer> for Scope<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State {
            remembered: self.active,
            preferred: self.active,
            reveal_pending: true,
        })
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, t: &mut Tree) {
        t.diff_children(std::slice::from_ref(&self.content));
        let state = t.state.downcast_mut::<State>();
        if state.preferred != self.active {
            state.preferred = self.active;
            state.remembered = self.active;
            state.reveal_pending = true;
        }
    }
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }
    fn layout(&mut self, t: &mut Tree, r: &Renderer, l: &layout::Limits) -> layout::Node {
        let node = self
            .content
            .as_widget_mut()
            .layout(&mut t.children[0], r, l);
        let state = t.state.downcast_mut::<State>();
        if self.arrows && state.reveal_pending {
            state.reveal_pending = false;
            let layout = Layout::new(&node);
            if !has_focus(&mut self.content, &mut t.children[0], layout, r) {
                reveal_at(
                    &mut self.content,
                    &mut t.children[0],
                    layout,
                    r,
                    Some(self.active),
                );
            }
        }
        node
    }
    fn operate(&mut self, t: &mut Tree, l: Layout<'_>, r: &Renderer, o: &mut dyn Operation) {
        if self.arrows {
            let mut query = Entries::default();
            self.content
                .as_widget_mut()
                .operate(&mut t.children[0], l, r, &mut query);
            let state = t.state.downcast_mut::<State>();
            if let Some(index) = query.items.iter().position(|(_, _, focused)| *focused) {
                state.remembered = index;
            }
            let active = state.remembered.min(query.items.len().saturating_sub(1));
            let mut mode = Traversal(false);
            o.custom(None, l.bounds(), &mut mode);
            if mode.0 {
                self.content.as_widget_mut().operate(
                    &mut t.children[0],
                    l,
                    r,
                    &mut Roving {
                        inner: o,
                        index: &mut 0,
                        active,
                    },
                );
                return;
            }
        }
        self.content
            .as_widget_mut()
            .operate(&mut t.children[0], l, r, o);
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
        if !self.arrows && tab(&mut self.content, &mut t.children[0], l, r, e, s) {
            return;
        }
        if self.arrows
            && let Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key),
                modifiers,
                ..
            }) = e
            && modifiers.is_empty()
            && matches!(
                key,
                keyboard::key::Named::ArrowLeft
                    | keyboard::key::Named::ArrowRight
                    | keyboard::key::Named::ArrowUp
                    | keyboard::key::Named::ArrowDown
                    | keyboard::key::Named::Home
                    | keyboard::key::Named::End
            )
        {
            use keyboard::key::Named;
            let mut query = Entries::default();
            self.content
                .as_widget_mut()
                .operate(&mut t.children[0], l, r, &mut query);
            if let Some(current) = query.items.iter().position(|(_, _, focused)| *focused) {
                let n = query.items.len();
                let next = match key {
                    Named::Home | Named::End if self.grid => {
                        let y = query.items[current].1.center_y();
                        let mut row = query
                            .items
                            .iter()
                            .enumerate()
                            .filter(|(_, (_, b, _))| (b.center_y() - y).abs() < 1.0);
                        if *key == Named::Home {
                            row.next().map_or(current, |(i, _)| i)
                        } else {
                            row.next_back().map_or(current, |(i, _)| i)
                        }
                    }
                    Named::Home => 0,
                    Named::End => n - 1,
                    Named::ArrowUp | Named::ArrowDown if self.grid => {
                        let bounds = query.items[current].1;
                        let direction = if *key == Named::ArrowUp { -1.0 } else { 1.0 };
                        query
                            .items
                            .iter()
                            .enumerate()
                            .filter(|(_, (_, b, _))| {
                                (b.center_y() - bounds.center_y()) * direction > 1.0
                            })
                            .min_by(|(_, (_, a, _)), (_, (_, b, _))| {
                                let score = |b: &Rectangle| {
                                    (b.center_x() - bounds.center_x()).abs() * 1000.0
                                        + (b.center_y() - bounds.center_y()).abs()
                                };
                                score(a).total_cmp(&score(b))
                            })
                            .map_or(current, |(i, _)| i)
                    }
                    Named::ArrowLeft | Named::ArrowUp => (current + n - 1) % n,
                    _ => (current + 1) % n,
                };
                select(&mut self.content, &mut t.children[0], l, r, next);
                t.state.downcast_mut::<State>().remembered = next;
                s.capture_event();
                s.request_redraw();
                return;
            }
        }
        if !self.arrows && matches!(e, Event::Mouse(mouse::Event::ButtonPressed(_))) {
            clear(&mut self.content, &mut t.children[0], l, r);
        }
        self.content
            .as_widget_mut()
            .update(&mut t.children[0], e, l, c, r, cb, s, v);
        if !self.arrows && matches!(e, Event::Keyboard(keyboard::Event::KeyPressed { .. })) {
            reveal(&mut self.content, &mut t.children[0], l, r);
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
        self.content
            .as_widget()
            .draw(&t.children[0], r, th, s, l, c, v);
    }
    fn mouse_interaction(
        &self,
        t: &Tree,
        l: Layout<'_>,
        c: mouse::Cursor,
        v: &Rectangle,
        r: &Renderer,
    ) -> mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(&t.children[0], l, c, v, r)
    }
    fn overlay<'b>(
        &'b mut self,
        t: &'b mut Tree,
        l: Layout<'b>,
        r: &Renderer,
        v: &Rectangle,
        tr: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(&mut t.children[0], l, r, v, tr)
    }
}

pub(crate) fn without_focus<Message>(
    content: &mut Element<'_, Message>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
    operation: &mut dyn Operation,
) {
    content.as_widget_mut().operate(
        tree,
        layout,
        renderer,
        &mut Roving {
            inner: operation,
            index: &mut 0,
            active: usize::MAX,
        },
    );
}

/// Keep an invoker's focus for restoration while drawing focus only in its popup.
pub(crate) fn suppress_ring<Message>(
    content: &mut Element<'_, Message>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
    suppressed: bool,
) {
    struct Ring(bool);
    impl Operation for Ring {
        fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation)) {
            visit(self);
        }
        fn custom(&mut self, _: Option<&widget::Id>, _: Rectangle, state: &mut dyn std::any::Any) {
            if let Some(focus) = state.downcast_mut::<Focus>() {
                focus.suppressed = self.0;
            }
        }
    }
    content
        .as_widget_mut()
        .operate(tree, layout, renderer, &mut Ring(suppressed));
}
