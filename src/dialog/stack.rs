use super::*;

/// Retained dialogs in bottom-to-top order. Keep entries mounted in stable order
/// and change their open flags, including during exit animations. Only the top
/// visible entry receives input. Closing it restores focus in the dialog beneath
/// it (or in the background), without replaying a held pointer/key gesture.
///
/// Each entry defaults to focusing its first enabled control when first opened.
/// Nested menus and other popups remain available within the active dialog.
pub fn stack<'a, Message: Clone + 'a>(
    background: impl Into<Element<'a, Message>>,
    dialogs: impl IntoIterator<Item = (Dialog<'a, Message>, bool)>,
) -> Element<'a, Message> {
    Element::new(Stack {
        background: background.into(),
        dialogs: dialogs
            .into_iter()
            .map(|(mut dialog, open)| {
                dialog.initial_focus.get_or_insert(0);
                (dialog, open)
            })
            .collect(),
    })
}
struct Stack<'a, Message> {
    background: Element<'a, Message>,
    dialogs: Vec<(Dialog<'a, Message>, bool)>,
}
#[derive(Default)]
struct StackState {
    layers: Vec<State>,
    bookmarks: Vec<Option<crate::focus::Bookmark>>,
    background: Option<crate::focus::Bookmark>,
    covered: bool,
    active: Option<usize>,
    inactive: bool,
    resume_background: bool,
    resume_layer: Option<usize>,
}
impl<Message> Stack<'_, Message> {
    fn top(&self, state: &StackState) -> Option<usize> {
        self.dialogs
            .iter()
            .enumerate()
            .rev()
            .find(|(i, (_, open))| state.layers[*i].presence.visible(*open))
            .map(|(i, _)| i)
    }
}
impl<Message: Clone> Widget<Message, Theme, Renderer> for Stack<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<StackState>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(StackState {
            layers: self
                .dialogs
                .iter()
                .map(|(_, open)| State {
                    was_open: *open,
                    focus_pending: *open,
                    presence: crate::presence::Presence::new(*open),
                    ..Default::default()
                })
                .collect(),
            bookmarks: vec![None; self.dialogs.len()],
            ..Default::default()
        })
    }
    fn children(&self) -> Vec<Tree> {
        std::iter::once(Tree::new(&self.background))
            .chain(self.dialogs.iter().map(|(d, _)| Tree::new(&d.content)))
            .collect()
    }
    fn diff(&self, tree: &mut Tree) {
        let children: Vec<_> = std::iter::once(&self.background)
            .chain(self.dialogs.iter().map(|(d, _)| &d.content))
            .collect();
        tree.diff_children(&children);
        let state = tree.state.downcast_mut::<StackState>();
        state.layers.resize_with(self.dialogs.len(), State::default);
        state.bookmarks.resize(self.dialogs.len(), None);
        for (i, (_, open)) in self.dialogs.iter().enumerate() {
            let layer = &mut state.layers[i];
            if layer.was_open != *open {
                layer.was_open = *open;
                layer.focus_pending = *open;
                layer.cancel_dialog = !open;
                layer.outside_pressed = false;
                if *open {
                    state.bookmarks[i] = None;
                }
            }
        }
    }
    fn size(&self) -> Size<Length> {
        self.background.as_widget().size()
    }
    fn layout(&mut self, tree: &mut Tree, r: &Renderer, limits: &layout::Limits) -> layout::Node {
        self.background
            .as_widget_mut()
            .layout(&mut tree.children[0], r, limits)
    }
    fn operate(&mut self, tree: &mut Tree, l: Layout<'_>, r: &Renderer, op: &mut dyn Operation) {
        if self.top(tree.state.downcast_ref()).is_none() {
            self.background
                .as_widget_mut()
                .operate(&mut tree.children[0], l, r, op);
        }
    }
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        l: Layout<'_>,
        cursor: mouse::Cursor,
        r: &Renderer,
        cb: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<StackState>();
        if let Some(focused) = crate::activity::window_focus(event) {
            state.inactive = !focused;
        }
        for (i, (_, open)) in self.dialogs.iter().enumerate() {
            let p = &mut state.layers[i].presence;
            let m = p.motion.get();
            p.update_surface(*open, event, m.dialog_enter, m.dialog_exit, true, shell);
        }
        let covered = self.top(state).is_some();
        if covered && !state.covered {
            state.background =
                crate::focus::remember(&mut self.background, &mut tree.children[0], l, r);
            suspend(
                &mut self.background,
                &mut tree.children[0],
                l,
                r,
                cb,
                viewport,
            );
        } else if !covered && state.covered {
            state.resume_background = true;
            state.active = None;
            state.resume_layer = None;
        }
        if !covered && state.resume_background && !state.inactive && !crate::activity::is_covered()
        {
            state.resume_background = false;
            crate::activity::interaction(|| {
                self.background.as_widget_mut().update(
                    &mut tree.children[0],
                    &Event::Window(iced::window::Event::Focused),
                    l,
                    cursor,
                    r,
                    cb,
                    shell,
                    viewport,
                );
            });
            if let Some(bookmark) = state.background.take() {
                crate::focus::restore(&bookmark, &mut self.background, &mut tree.children[0], l, r);
            }
        }
        state.covered = covered;
        if covered {
            update_covered(
                &mut self.background,
                &mut tree.children[0],
                event,
                l,
                r,
                cb,
                shell,
                viewport,
                !state.inactive,
            );
            if is_input(event) {
                shell.capture_event();
            }
        } else {
            self.background.as_widget_mut().update(
                &mut tree.children[0],
                event,
                l,
                cursor,
                r,
                cb,
                shell,
                viewport,
            );
        }
    }
    fn draw(
        &self,
        tree: &Tree,
        r: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        l: Layout<'_>,
        cursor: mouse::Cursor,
        v: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<StackState>();
        for layer in &state.layers {
            layer.presence.motion.set(theme.motion);
        }
        self.background.as_widget().draw(
            &tree.children[0],
            r,
            theme,
            style,
            l,
            if self.top(state).is_some() {
                mouse::Cursor::Unavailable
            } else {
                cursor
            },
            v,
        );
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        l: Layout<'_>,
        cursor: mouse::Cursor,
        v: &Rectangle,
        r: &Renderer,
    ) -> mouse::Interaction {
        if self.top(tree.state.downcast_ref()).is_some() {
            mouse::Interaction::None
        } else {
            self.background
                .as_widget()
                .mouse_interaction(&tree.children[0], l, cursor, v, r)
        }
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        l: Layout<'b>,
        r: &Renderer,
        v: &Rectangle,
        tr: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        if let Some(top) = self.top(tree.state.downcast_ref()) {
            Some(overlay::Element::new(Box::new(StackOverlay {
                host: self,
                tree,
                top,
            })))
        } else {
            self.background
                .as_widget_mut()
                .overlay(&mut tree.children[0], l, r, v, tr)
        }
    }
}
struct StackOverlay<'a, 'b, Message> {
    host: &'a mut Stack<'b, Message>,
    tree: &'a mut Tree,
    top: usize,
}
impl<'a, 'b, Message: Clone> StackOverlay<'a, 'b, Message> {
    fn layer(&mut self, i: usize) -> DialogOverlay<'_, 'b, Message> {
        let (dialog, open) = &mut self.host.dialogs[i];
        DialogOverlay {
            dialog,
            open: *open,
            tree: &mut self.tree.children[i + 1],
            state: &mut self.tree.state.downcast_mut::<StackState>().layers[i],
        }
    }
}
impl<Message: Clone> Overlay<Message, Theme, Renderer> for StackOverlay<'_, '_, Message> {
    fn layout(&mut self, r: &Renderer, bounds: Size) -> layout::Node {
        layout::Node::with_children(
            bounds,
            (0..self.host.dialogs.len())
                .map(|i| self.layer(i).layout(r, bounds))
                .collect(),
        )
    }
    fn operate(&mut self, l: Layout<'_>, r: &Renderer, op: &mut dyn Operation) {
        let top = self.top;
        self.layer(top).operate(l.child(top), r, op);
    }
    fn update(
        &mut self,
        event: &Event,
        l: Layout<'_>,
        cursor: mouse::Cursor,
        r: &Renderer,
        cb: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let state = self.tree.state.downcast_mut::<StackState>();
        if let Some(focused) = crate::activity::window_focus(event) {
            state.inactive = !focused;
        }
        if state.active != Some(self.top) {
            if let Some(old) = state.active.filter(|i| *i < self.host.dialogs.len()) {
                let content = &mut self.host.dialogs[old].0.content;
                let tree = &mut self.tree.children[old + 1];
                // Save the traversal target before cancelling its key/pointer state.
                if let Some(bookmark) =
                    crate::focus::remember(content, tree, l.child(old).child(0), r)
                {
                    state.bookmarks[old] = Some(bookmark);
                }
                suspend(content, tree, l.child(old).child(0), r, cb, &l.bounds());
            }
            // Layers can be mounted already covered, without ever being active.
            // Suspend them too before allowing redraw-only feedback updates.
            for i in 0..self.top {
                if state.active == Some(i)
                    || !state.layers[i].presence.visible(self.host.dialogs[i].1)
                {
                    continue;
                }
                suspend(
                    &mut self.host.dialogs[i].0.content,
                    &mut self.tree.children[i + 1],
                    l.child(i).child(0),
                    r,
                    cb,
                    &l.bounds(),
                );
            }
            state.resume_layer = Some(self.top);
            state.active = Some(self.top);
            shell.request_redraw();
        }
        if state.resume_layer == Some(self.top) && !state.inactive && !crate::activity::is_covered()
        {
            state.resume_layer = None;
            let content = &mut self.host.dialogs[self.top].0.content;
            let tree = &mut self.tree.children[self.top + 1];
            let mut ignored = Vec::new();
            crate::activity::interaction(|| {
                content.as_widget_mut().update(
                    tree,
                    &Event::Window(iced::window::Event::Focused),
                    l.child(self.top).child(0),
                    cursor,
                    r,
                    cb,
                    &mut Shell::new(&mut ignored),
                    &l.bounds(),
                );
            });
            if let Some(bookmark) = state.bookmarks[self.top].take() {
                crate::focus::restore(&bookmark, content, tree, l.child(self.top).child(0), r);
                state.layers[self.top].focus_pending = false;
            }
            shell.request_redraw();
        }
        let top = self.top;
        if !self.host.dialogs[top].1 && state.layers[top].cancel_dialog {
            state.layers[top].cancel_dialog = false;
            suspend(
                &mut self.host.dialogs[top].0.content,
                &mut self.tree.children[top + 1],
                l.child(top).child(0),
                r,
                cb,
                &l.bounds(),
            );
        }
        for i in 0..top {
            if state.layers[i].presence.visible(self.host.dialogs[i].1) {
                let p = &state.layers[i].presence;
                let m = p.motion.get();
                let _phase = crate::staged::enter(crate::staged::Phase {
                    kind: crate::staged::Kind::Dialog,
                    open: self.host.dialogs[i].1,
                    initial: p.progress.value,
                    base: p.content.value,
                    enter: m.dialog_enter,
                    exit: m.dialog_exit,
                    above: false,
                });
                update_covered(
                    &mut self.host.dialogs[i].0.content,
                    &mut self.tree.children[i + 1],
                    event,
                    l.child(i).child(0),
                    r,
                    cb,
                    shell,
                    &l.bounds(),
                    !state.inactive,
                );
            }
        }
        // Closing top layers use the same real window activity as their host.
        state.layers[top].inactive = state.inactive;
        self.layer(top)
            .update(event, l.child(top), cursor, r, cb, shell);
    }
    fn draw(
        &self,
        r: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        l: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        use iced::advanced::Renderer as _;
        let state = self.tree.state.downcast_ref::<StackState>();
        for (i, (dialog, open)) in self.host.dialogs.iter().enumerate().take(self.top + 1) {
            let layer = &state.layers[i];
            if !layer.presence.visible(*open) {
                continue;
            }
            layer.presence.motion.set(theme.motion);
            // Separate renderer layers preserve opaque surface/text ordering on GPU.
            r.with_layer(l.bounds(), |r| {
                let mut scrim =
                    crate::theme::alpha(theme.colors.scrim, theme.colors.scrim.a * 0.32);
                scrim.a *= layer.presence.progress.value;
                r.fill_quad(
                    renderer::Quad {
                        bounds: l.bounds(),
                        ..Default::default()
                    },
                    scrim,
                );
                draw_dialog(
                    dialog,
                    layer,
                    &self.tree.children[i + 1],
                    r,
                    theme,
                    style,
                    l.child(i).child(0),
                    if i == self.top && *open {
                        cursor
                    } else {
                        mouse::Cursor::Unavailable
                    },
                    &l.bounds(),
                );
            });
        }
    }
    fn mouse_interaction(
        &self,
        l: Layout<'_>,
        c: mouse::Cursor,
        r: &Renderer,
    ) -> mouse::Interaction {
        let (dialog, open) = &self.host.dialogs[self.top];
        if !open {
            return mouse::Interaction::Idle;
        }
        dialog.content.as_widget().mouse_interaction(
            &self.tree.children[self.top + 1],
            l.child(self.top).child(0),
            c,
            &l.bounds(),
            r,
        )
    }
    fn overlay<'c>(
        &'c mut self,
        l: Layout<'c>,
        r: &Renderer,
    ) -> Option<overlay::Element<'c, Message, Theme, Renderer>> {
        let (dialog, open) = &mut self.host.dialogs[self.top];
        if !*open {
            return None;
        }
        dialog.content.as_widget_mut().overlay(
            &mut self.tree.children[self.top + 1],
            l.child(self.top).child(0),
            r,
            &l.bounds(),
            Vector::ZERO,
        )
    }
}

fn suspend<Message>(
    content: &mut Element<'_, Message>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
    clipboard: &mut dyn Clipboard,
    viewport: &Rectangle,
) {
    // Native input clears its drag only on release, not on window deactivation.
    // Cancel first, then release with an unavailable pointer; discard all messages.
    crate::activity::interaction(|| {
        for event in [
            Event::Window(iced::window::Event::Unfocused),
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
        ] {
            let mut ignored = Vec::new();
            content.as_widget_mut().update(
                tree,
                &event,
                layout,
                mouse::Cursor::Unavailable,
                renderer,
                clipboard,
                &mut Shell::new(&mut ignored),
                viewport,
            );
        }
    });
    crate::focus::clear(content, tree, layout, renderer);
}
