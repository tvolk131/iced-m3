//! Root modal host with a full-window overlay and explicit event isolation.
mod content;
mod stack;
use crate::{Element, Theme, activity::update_covered, tokens};
use iced::advanced::{
    Clipboard, Layout, Overlay, Shell, Widget, layout, overlay, renderer,
    widget::{Operation, Tree, tree},
};
use iced::{
    Border, Event, Length, Point, Rectangle, Renderer, Size, Vector, keyboard, mouse, widget,
};
pub use stack::stack;

/// Dialog content with configurable dismissal. Place it over your application
/// with [`modal`], keeping that host in the tree even while the dialog is closed.
pub struct Dialog<'a, Message> {
    content: Element<'a, Message>,
    actions: Option<Element<'a, Message>>,
    dismiss: Option<Message>,
    outside: bool,
    escape: bool,
    width: f32,
    max_height: f32,
    full_screen: bool,
    initial_focus: Option<usize>,
}
pub fn dialog<'a, Message: 'a>(content: impl Into<Element<'a, Message>>) -> Dialog<'a, Message> {
    Dialog::new(content)
}
/// Mark a basic dialog's action row for its separate Material entrance timeline.
/// Keep the row in the dialog's normal content layout; no extra spacing is added.
/// This marker does not fix the row in place. Prefer [`Dialog::actions`] for a
/// fixed footer; it applies this animation automatically.
pub fn actions<'a, Message: 'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    crate::staged::part(content, crate::staged::Part::DialogActions)
}
impl<'a, Message: 'a> Dialog<'a, Message> {
    /// Content is arbitrary iced content, commonly a column with title and body.
    /// Oversized content scrolls automatically. Use [`Self::actions`] to keep
    /// action buttons outside that scrolling region; no inner scrollable is needed.
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            actions: None,
            dismiss: None,
            outside: true,
            escape: true,
            width: tokens::size::DIALOG_MAX,
            max_height: f32::INFINITY,
            full_screen: false,
            initial_focus: None,
        }
    }
    /// Keep action content in a fixed, trailing-aligned footer below the scrolling
    /// body. A 24px gap separates the regions; the footer gets the Material action
    /// entrance animation automatically. Calling this again replaces the footer.
    ///
    /// Supply a compact row with `.wrap()` so actions can fit narrow windows.
    /// The dialog measures this footer first and gives the body the remaining
    /// height. Do not wrap the body in another scrollable just to keep actions visible.
    /// After converting a [`FullScreenDialog`] into a `Dialog`, this adds a bottom
    /// footer with 24px padding while preserving its fixed header and body scroller.
    ///
    /// ```rust
    /// use iced::widget::{column, row};
    /// use iced_m3::{Dialog, button, dialog, text_field};
    ///
    /// #[derive(Clone)]
    /// enum Message { Name(String), Cancel, Save }
    ///
    /// fn settings(name: &str) -> Dialog<'_, Message> {
    ///     dialog(column![text_field("Workspace name", name).on_input(Message::Name)])
    ///         .actions(row![
    ///             button("Cancel").on_press(Message::Cancel),
    ///             button("Save").on_press(Message::Save),
    ///         ].spacing(8).wrap())
    ///         .max_height(600)
    /// }
    /// ```
    pub fn actions(mut self, actions: impl Into<Element<'a, Message>>) -> Self {
        self.actions = Some(actions.into());
        self
    }
    /// Limit a basic dialog's total panel height, including padding and actions,
    /// in logical pixels. It also fits the available window height; short content
    /// stays compact instead of expanding to this limit. Defaults to no extra cap.
    /// Negative values clamp to zero, NaN is ignored, and infinity removes the cap.
    /// Full-screen dialogs always fill the window.
    pub fn max_height(mut self, height: impl Into<iced::Pixels>) -> Self {
        let height = height.into().0;
        if !height.is_nan() {
            self.max_height = height.max(0.0);
        }
        self
    }
    fn prepare(mut self) -> Self {
        if self.full_screen {
            if let Some(footer) = self.actions.take() {
                self.content = content::with_actions(
                    self.content,
                    actions(
                        widget::container(footer)
                            .padding(24)
                            .align_right(Length::Fill),
                    ),
                    0.0,
                );
            }
            return self;
        }
        let body: Element<'a, Message> = widget::scrollable(self.content)
            .width(Length::Fill)
            .direction(widget::scrollable::Direction::Vertical(
                widget::scrollable::Scrollbar::new()
                    .width(tokens::spacing::XS)
                    .scroller_width(tokens::spacing::XS)
                    .spacing(tokens::spacing::SM),
            ))
            .into();
        let body = if let Some(footer) = self.actions.take() {
            content::with_actions(
                body,
                actions(widget::container(footer).align_right(Length::Fill)),
                tokens::spacing::LG,
            )
        } else {
            body
        };
        // The host paints the rounded panel independently during its reveal.
        self.content = widget::container(body)
            .padding(tokens::spacing::LG)
            .width(Length::Fill)
            .style(|theme: &Theme| widget::container::Style {
                text_color: Some(theme.colors.on_surface),
                ..Default::default()
            })
            .into();
        self
    }
    /// Focus an enabled control when this dialog opens (zero-based traversal index).
    pub fn initial_focus(mut self, index: usize) -> Self {
        self.initial_focus = Some(index);
        self
    }
    /// Shared message for enabled outside-click and Escape dismissal.
    pub fn on_dismiss(mut self, message: Message) -> Self {
        self.dismiss = Some(message);
        self
    }
    pub fn dismiss_on_outside(mut self, enabled: bool) -> Self {
        self.outside = enabled;
        self
    }
    pub fn dismiss_on_escape(mut self, enabled: bool) -> Self {
        self.escape = enabled;
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(0.0);
        self
    }
}

/// Show a dialog over the application with animated opening and closing.
///
/// Keep this root host and its dialog content in the view even while closed;
/// change `open` to animate visibility and preserve the child widget state.
/// Background input stays blocked through the exit animation, and closing
/// content cannot publish actions. A host first mounted open starts fully visible;
/// subsequent changes to `open` animate. The theme's reduced-motion setting is
/// respected. Use [`stack`] for multiple dialogs.
///
/// ```rust
/// use iced_m3::{Element, button, dialog};
///
/// #[derive(Clone)]
/// enum Message { Open, Close }
///
/// fn view(open: bool) -> Element<'static, Message> {
///     dialog::modal(
///         button("Open dialog").on_press(Message::Open),
///         dialog(button("Close").on_press(Message::Close))
///             .on_dismiss(Message::Close),
///         open,
///     )
/// }
/// ```
pub fn modal<'a, Message: Clone + 'a>(
    background: impl Into<Element<'a, Message>>,
    dialog: Dialog<'a, Message>,
    open: bool,
) -> Element<'a, Message> {
    Element::new(Modal {
        background: background.into(),
        dialog: dialog.prepare(),
        open,
    })
}
struct Modal<'a, Message> {
    background: Element<'a, Message>,
    dialog: Dialog<'a, Message>,
    open: bool,
}
#[derive(Default)]
struct State {
    outside_pressed: bool,
    was_open: bool,
    cancel_background: bool,
    resume_background: bool,
    inactive: bool,
    focus_pending: bool,
    presence: crate::presence::Presence,
    shadow: crate::elevation::Cache,
    cancel_dialog: bool,
}

impl<Message: Clone> Widget<Message, Theme, Renderer> for Modal<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State {
            was_open: self.open,
            cancel_background: self.open,
            focus_pending: self.open,
            presence: crate::presence::Presence::new(self.open),
            ..Default::default()
        })
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.background), Tree::new(&self.dialog.content)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.children[0].diff(&self.background);
        tree.children[1].diff(&self.dialog.content);
        let state = tree.state.downcast_mut::<State>();
        let open = self.open;
        if open != state.was_open {
            state.outside_pressed = false;
            state.cancel_background = open;
            state.focus_pending = open;
            state.resume_background = !open;
            state.cancel_dialog = !open;
            state.was_open = open;
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
        self.background
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
        if !tree
            .state
            .downcast_ref::<State>()
            .presence
            .visible(self.open)
        {
            self.background.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                operation,
            );
        }
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
        let motion = state.presence.motion.get();
        state.presence.update_surface(
            self.open,
            event,
            motion.dialog_enter,
            motion.dialog_exit,
            true,
            shell,
        );
        if let Some(focused) = crate::activity::window_focus(event) {
            state.inactive = !focused;
        }
        if !state.presence.visible(self.open) {
            if state.resume_background && !crate::activity::is_covered() {
                state.resume_background = false;
                if !state.inactive {
                    // Resume timers suspended by our synthetic Unfocused event.
                    // This does not restore text-input focus or replay gestures.
                    crate::activity::interaction(|| {
                        self.background.as_widget_mut().update(
                            &mut tree.children[0],
                            &Event::Window(iced::window::Event::Focused),
                            layout,
                            cursor,
                            renderer,
                            clipboard,
                            shell,
                            viewport,
                        );
                    });
                }
            }
            self.background.as_widget_mut().update(
                &mut tree.children[0],
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        } else {
            let state = tree.state.downcast_mut::<State>();
            if state.cancel_background {
                state.cancel_background = false;
                // Cancel an in-flight pointer gesture when a dialog opens programmatically.
                // Discard actions from cancellation; subsequent redraws finish
                // visual feedback while input and interaction timers stay suspended.
                let mut messages = Vec::new();
                crate::activity::interaction(|| {
                    self.background.as_widget_mut().update(
                        &mut tree.children[0],
                        &Event::Window(iced::window::Event::Unfocused),
                        layout,
                        mouse::Cursor::Unavailable,
                        renderer,
                        clipboard,
                        &mut Shell::new(&mut messages),
                        viewport,
                    );
                });
            }
            update_covered(
                &mut self.background,
                &mut tree.children[0],
                event,
                layout,
                renderer,
                clipboard,
                shell,
                viewport,
                !state.inactive,
            );
            if is_input(event) {
                shell.capture_event();
            }
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
        tree.state
            .downcast_ref::<State>()
            .presence
            .motion
            .set(theme.motion);
        self.background.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            if tree
                .state
                .downcast_ref::<State>()
                .presence
                .visible(self.open)
            {
                mouse::Cursor::Unavailable
            } else {
                cursor
            },
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
        if tree
            .state
            .downcast_ref::<State>()
            .presence
            .visible(self.open)
        {
            mouse::Interaction::None
        } else {
            self.background.as_widget().mouse_interaction(
                &tree.children[0],
                layout,
                cursor,
                viewport,
                renderer,
            )
        }
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        if tree
            .state
            .downcast_ref::<State>()
            .presence
            .visible(self.open)
        {
            Some(overlay::Element::new(Box::new(DialogOverlay {
                dialog: &mut self.dialog,
                tree: &mut tree.children[1],
                state: tree.state.downcast_mut::<State>(),
                open: self.open,
            })))
        } else {
            self.background.as_widget_mut().overlay(
                &mut tree.children[0],
                layout,
                renderer,
                viewport,
                translation,
            )
        }
    }
}
struct DialogOverlay<'a, 'b, Message> {
    dialog: &'a mut Dialog<'b, Message>,
    tree: &'a mut Tree,
    state: &'a mut State,
    open: bool,
}
impl<Message: Clone> Overlay<Message, Theme, Renderer> for DialogOverlay<'_, '_, Message> {
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        if self.dialog.full_screen {
            let content = self.dialog.content.as_widget_mut().layout(
                self.tree,
                renderer,
                &layout::Limits::new(bounds, bounds),
            );
            return layout::Node::with_children(
                bounds,
                vec![content.move_to(Point::new(
                    0.0,
                    bounds.height * (1.0 - self.state.presence.slide.value),
                ))],
            );
        }
        let margin = tokens::spacing::LG
            .min(bounds.width / 8.0)
            .min(bounds.height / 8.0);
        let max = Size::new(
            self.dialog
                .width
                .min((bounds.width - margin * 2.0).max(0.0)),
            self.dialog
                .max_height
                .min((bounds.height - margin * 2.0).max(0.0)),
        );
        let content = self.dialog.content.as_widget_mut().layout(
            self.tree,
            renderer,
            &layout::Limits::new(Size::ZERO, max),
        );
        let size = content.size();
        layout::Node::with_children(
            bounds,
            vec![content.move_to(Point::new(
                (bounds.width - size.width) / 2.0,
                (bounds.height - size.height) / 2.0
                    - 50.0 * (1.0 - self.state.presence.slide.value),
            ))],
        )
    }
    fn operate(&mut self, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        if !self.open {
            return;
        }
        self.dialog.content.as_widget_mut().operate(
            self.tree,
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
    }
    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        if let Some(focused) = crate::activity::window_focus(event) {
            self.state.inactive = !focused;
        }
        let content = layout.children().next().unwrap();
        let motion = self.state.presence.motion.get();
        self.state.presence.update_surface(
            self.open,
            event,
            motion.dialog_enter,
            motion.dialog_exit,
            true,
            shell,
        );
        let _phase = crate::staged::enter(crate::staged::Phase {
            kind: crate::staged::Kind::Dialog,
            open: self.open,
            initial: self.state.presence.progress.value,
            base: self.state.presence.content.value,
            enter: motion.dialog_enter,
            exit: motion.dialog_exit,
            above: false,
        });
        if !self.open {
            if self.state.cancel_dialog {
                self.state.cancel_dialog = false;
                let mut ignored = Vec::new();
                crate::activity::interaction(|| {
                    self.dialog.content.as_widget_mut().update(
                        self.tree,
                        &Event::Window(iced::window::Event::Unfocused),
                        content,
                        mouse::Cursor::Unavailable,
                        renderer,
                        clipboard,
                        &mut Shell::new(&mut ignored),
                        &layout.bounds(),
                    );
                });
            }
            update_covered(
                &mut self.dialog.content,
                self.tree,
                event,
                content,
                renderer,
                clipboard,
                shell,
                &layout.bounds(),
                !self.state.inactive,
            );
            if is_input(event) {
                shell.capture_event();
            }
            return;
        }
        if self.state.focus_pending {
            self.state.focus_pending = false;
            if let Some(index) = self.dialog.initial_focus {
                crate::focus::select(
                    &mut self.dialog.content,
                    self.tree,
                    content,
                    renderer,
                    index,
                );
                shell.request_redraw();
            }
        }
        if crate::focus::tab(
            &mut self.dialog.content,
            self.tree,
            content,
            renderer,
            event,
            shell,
        ) {
            return;
        }
        if matches!(event, Event::Mouse(mouse::Event::ButtonPressed(_))) {
            crate::focus::clear(&mut self.dialog.content, self.tree, content, renderer);
        }

        let escape = matches!(
            event,
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            })
        );
        if escape {
            if self.dialog.escape
                && let Some(message) = &self.dialog.dismiss
            {
                shell.publish(message.clone());
            }
            shell.capture_event();
            return;
        }
        let outside = cursor.position().is_some() && !cursor.is_over(content.bounds());
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                self.state.outside_pressed = outside;
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if self.state.outside_pressed
                    && outside
                    && self.dialog.outside
                    && let Some(message) = &self.dialog.dismiss
                {
                    shell.publish(message.clone());
                }
                self.state.outside_pressed = false;
            }
            Event::Window(iced::window::Event::Unfocused) => {
                self.state.outside_pressed = false;
            }
            _ => {}
        }
        self.dialog.content.as_widget_mut().update(
            self.tree,
            event,
            content,
            cursor,
            renderer,
            clipboard,
            shell,
            &layout.bounds(),
        );
        if matches!(event, Event::Keyboard(_)) {
            crate::focus::reveal(&mut self.dialog.content, self.tree, content, renderer);
        }
        // Capture even unhandled presses, releases, wheel, keyboard and IME events.
        if is_input(event) {
            shell.capture_event();
        }
    }
    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        use iced::advanced::Renderer as _;
        self.state.presence.motion.set(theme.motion);
        let mut scrim = crate::theme::alpha(theme.colors.scrim, theme.colors.scrim.a * 0.32);
        scrim.a *= self.state.presence.progress.value;
        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                ..Default::default()
            },
            scrim,
        );
        draw_dialog(
            self.dialog,
            self.state,
            self.tree,
            renderer,
            theme,
            style,
            layout.child(0),
            if self.open {
                cursor
            } else {
                mouse::Cursor::Unavailable
            },
            &layout.bounds(),
        );
    }
    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if !self.open {
            return mouse::Interaction::Idle;
        }
        self.dialog.content.as_widget().mouse_interaction(
            self.tree,
            layout.children().next().unwrap(),
            cursor,
            &layout.bounds(),
            renderer,
        )
    }
    fn overlay<'a>(
        &'a mut self,
        layout: Layout<'a>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        if !self.open {
            return None;
        }
        self.dialog.content.as_widget_mut().overlay(
            self.tree,
            layout.children().next().unwrap(),
            renderer,
            &layout.bounds(),
            Vector::ZERO,
        )
    }
}

fn is_input(event: &Event) -> bool {
    !matches!(event, Event::Window(_))
}

/// A full-window task dialog with a fixed title/action bar and a scrolling body.
/// Convert to [`Dialog`] with `.into()` when passing it to [`modal`].
pub struct FullScreenDialog<'a, Message> {
    title: String,
    body: Element<'a, Message>,
    actions: Vec<Element<'a, Message>>,
    dismiss: Option<Message>,
    escape: bool,
    initial_focus: Option<usize>,
}
pub fn full_screen_dialog<'a, Message>(
    title: impl Into<String>,
    body: impl Into<Element<'a, Message>>,
) -> FullScreenDialog<'a, Message> {
    FullScreenDialog {
        title: title.into(),
        body: body.into(),
        actions: Vec::new(),
        dismiss: None,
        escape: true,
        initial_focus: Some(0),
    }
}
impl<'a, Message> FullScreenDialog<'a, Message> {
    pub fn on_dismiss(mut self, message: Message) -> Self {
        self.dismiss = Some(message);
        self
    }
    pub fn action(mut self, action: impl Into<Element<'a, Message>>) -> Self {
        self.actions.push(action.into());
        self
    }
    pub fn dismiss_on_escape(mut self, enabled: bool) -> Self {
        self.escape = enabled;
        self
    }
    pub fn initial_focus(mut self, index: usize) -> Self {
        self.initial_focus = Some(index);
        self
    }
}
impl<'a, Message: Clone + 'a> From<FullScreenDialog<'a, Message>> for Dialog<'a, Message> {
    fn from(dialog: FullScreenDialog<'a, Message>) -> Self {
        let mut header = crate::app_bar(dialog.title).leading(
            crate::icon_button(Element::new(crate::glyph::Glyph::Close))
                .on_press_maybe(dialog.dismiss.clone()),
        );
        for action in dialog.actions {
            header = header.action(action);
        }
        let body = widget::scrollable(
            widget::container(dialog.body)
                .padding(24)
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .direction(widget::scrollable::Direction::Vertical(
            widget::scrollable::Scrollbar::new()
                .width(4)
                .scroller_width(4)
                .spacing(8),
        ));
        let content =
            widget::container(widget::column![header, crate::divider(), body].height(Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|theme: &Theme| widget::container::Style {
                    background: Some(theme.colors.surface.into()),
                    text_color: Some(theme.colors.on_surface),
                    ..Default::default()
                });
        Dialog {
            content: content.into(),
            actions: None,
            dismiss: dialog.dismiss,
            outside: false,
            escape: dialog.escape,
            width: f32::INFINITY,
            max_height: f32::INFINITY,
            full_screen: true,
            initial_focus: dialog.initial_focus,
        }
    }
}

/// Both modal hosts share the exact same clipping and content fade.
#[allow(clippy::too_many_arguments)]
fn draw_dialog<Message>(
    dialog: &Dialog<'_, Message>,
    state: &State,
    tree: &Tree,
    renderer: &mut Renderer,
    theme: &Theme,
    style: &renderer::Style,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    viewport: &Rectangle,
) {
    use iced::advanced::Renderer as _;
    if dialog.full_screen {
        dialog
            .content
            .as_widget()
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
        return;
    }
    let bounds = layout.bounds();
    let height = state.presence.height.value;
    let reveal = Rectangle {
        height: bounds.height * height,
        ..bounds
    };
    let Some(surface_clip) = reveal.intersection(viewport) else {
        return;
    };
    let mut surface_theme = theme.clone();
    surface_theme.colors.shadow.a *= state.presence.paper.value;
    crate::elevation::draw_resizing(
        renderer,
        &surface_theme,
        reveal,
        bounds.height,
        28.0,
        3.0,
        *viewport,
        &state.shadow,
    );
    renderer.fill_quad(
        renderer::Quad {
            bounds: reveal,
            border: Border {
                radius: 28.0.into(),
                ..Default::default()
            },
            ..Default::default()
        },
        crate::theme::alpha(
            theme.colors.surface_container_high,
            state.presence.paper.value,
        ),
    );
    if state.presence.content.value <= 0.0 {
        return;
    }
    // Retain the full layout for wrapping, caret position and scrolling. Only
    // the available content area changes during the surface's size transition.
    let content_bounds = if height < 1.0 {
        reveal.shrink(24.0)
    } else {
        bounds
    };
    let Some(clip) = content_bounds
        .intersection(&surface_clip)
        .filter(|r| r.width > 0.0 && r.height > 0.0)
    else {
        return;
    };
    renderer.with_layer(clip, |renderer| {
        dialog
            .content
            .as_widget()
            .draw(tree, renderer, theme, style, layout, cursor, &clip);
        if state.presence.content.value < 1.0 {
            renderer.with_layer(clip, |renderer| {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: clip,
                        ..Default::default()
                    },
                    crate::theme::alpha(
                        theme.colors.surface_container_high,
                        1.0 - state.presence.content.value,
                    ),
                )
            });
        }
    });
}
