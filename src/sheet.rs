//! Persistent sheet host with finite, reversible motion and modal input isolation.
use crate::{Element, Theme, motion::Transition, tokens};
use iced::advanced::{
    Clipboard, Layout, Overlay, Shell, Widget, layout, overlay, renderer,
    widget::{Operation, Tree, tree},
};
use iced::{
    Border, Event, Length, Rectangle, Renderer, Size, Vector, keyboard, mouse, widget, window,
};
use std::cell::Cell;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Placement {
    Side,
    Start,
    Bottom,
}

/// Application-owned visibility; keep this value and its content in [`host`]
/// while closed so exit motion, scroll position and child state are preserved.
pub struct Sheet<'a, Message> {
    content: Element<'a, Message>,
    placement: Placement,
    modal: bool,
    open: bool,
    width: f32,
    height: f32,
    dismiss: Option<Message>,
    outside: bool,
    escape: bool,
    on_height: Option<Box<dyn Fn(f32) -> Message + 'a>>,
    snaps: Vec<f32>,
}
/// A standard right-side sheet that reserves space beside the main content.
/// Use `.modal()` to show it over a scrim instead.
pub fn side_sheet<'a, Message: 'a>(content: impl Into<Element<'a, Message>>) -> Sheet<'a, Message> {
    Sheet::new(content, Placement::Side)
}
/// A modal bottom sheet, centered and capped at 640px wide by default.
pub fn bottom_sheet<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> Sheet<'a, Message> {
    Sheet::new(content, Placement::Bottom)
}
/// Internal navigation host: the rail supplies its own padding and scrolling.
pub(crate) fn navigation_panel<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> Sheet<'a, Message> {
    Sheet {
        content: content.into(),
        placement: Placement::Start,
        modal: true,
        open: false,
        width: 280.0,
        height: 0.0,
        dismiss: None,
        outside: true,
        escape: true,
        on_height: None,
        snaps: Vec::new(),
    }
}
impl<'a, Message: 'a> Sheet<'a, Message> {
    fn new(content: impl Into<Element<'a, Message>>, placement: Placement) -> Self {
        let content = widget::container(
            widget::scrollable(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .direction(widget::scrollable::Direction::Vertical(
                    widget::scrollable::Scrollbar::new()
                        .width(4)
                        .scroller_width(4)
                        .spacing(8),
                )),
        )
        .padding(tokens::spacing::LG)
        .width(Length::Fill)
        .height(Length::Fill);
        Self {
            content: content.into(),
            placement,
            modal: placement == Placement::Bottom,
            open: false,
            width: if placement == Placement::Side {
                tokens::size::SIDE_SHEET
            } else {
                tokens::size::BOTTOM_SHEET_MAX
            },
            height: tokens::size::BOTTOM_SHEET_HEIGHT,
            dismiss: None,
            outside: true,
            escape: true,
            on_height: None,
            snaps: Vec::new(),
        }
    }
    /// Enables dragging the top handle of a bottom sheet. Height remains app-owned.
    pub fn on_height(mut self, handler: impl Fn(f32) -> Message + 'a) -> Self {
        if self.on_height.is_none() {
            self.content = widget::container(self.content)
                .padding(iced::Padding {
                    top: 24.0,
                    ..iced::Padding::ZERO
                })
                .width(Length::Fill)
                .height(Length::Fill)
                .into();
        }
        self.on_height = Some(Box::new(handler));
        self
    }
    pub fn snap_points(mut self, heights: impl IntoIterator<Item = f32>) -> Self {
        self.snaps = heights
            .into_iter()
            .filter(|h| h.is_finite() && *h >= 56.0)
            .collect();
        self
    }
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
    pub fn standard(mut self) -> Self {
        self.modal = false;
        self
    }
    pub fn modal(mut self) -> Self {
        self.modal = true;
        self
    }
    pub fn on_dismiss(mut self, message: Message) -> Self {
        self.dismiss = Some(message);
        self
    }
    /// Applies to modal sheets. Standard sheets keep outside content interactive.
    pub fn dismiss_on_outside(mut self, enabled: bool) -> Self {
        self.outside = enabled;
        self
    }
    pub fn dismiss_on_escape(mut self, enabled: bool) -> Self {
        self.escape = enabled;
        self
    }
    /// Desired side width or maximum bottom width, clamped to the host.
    pub fn width(mut self, width: f32) -> Self {
        if width.is_finite() && width > 0.0 {
            self.width = width;
        }
        self
    }
    /// Desired bottom-sheet height, clamped to leave room above the scrim.
    /// Side sheets always fill the host height.
    pub fn height(mut self, height: f32) -> Self {
        if height.is_finite() && height > 0.0 {
            self.height = height;
        }
        self
    }
    fn drag(
        &self,
        state: &mut State,
        event: &Event,
        bounds: Rectangle,
        host_height: f32,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) -> bool
    where
        Message: Clone,
    {
        let Some(handler) = &self.on_height else {
            return false;
        };
        if !self.open || self.placement != Placement::Bottom {
            return false;
        }
        let handle = Rectangle {
            height: 32.0,
            ..bounds
        };
        let mut consumed = false;
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if cursor.is_over(handle) && state.progress.value >= 0.999 =>
            {
                state.drag = cursor.position().map(|p| (p.y, bounds.height));
                consumed = true;
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) if state.drag.is_some() => {
                let (start, height) = state.drag.unwrap();
                shell.publish(handler((height + start - position.y).clamp(
                    56.0,
                    (host_height - 24.0_f32.min(host_height / 4.0)).max(56.0),
                )));
                consumed = true;
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.drag.is_some() =>
            {
                let (start, height) = state.drag.take().unwrap();
                let height = cursor
                    .position()
                    .map_or(height, |p| height + start - p.y)
                    .clamp(56.0, host_height.max(56.0));
                if height < 80.0
                    && let Some(dismiss) = &self.dismiss
                {
                    shell.publish(dismiss.clone());
                } else {
                    let snapped = self
                        .snaps
                        .iter()
                        .filter(|h| **h < host_height)
                        .min_by(|a, b| (**a - height).abs().total_cmp(&(**b - height).abs()))
                        .copied()
                        .unwrap_or(height);
                    shell.publish(handler(snapped));
                }
                consumed = true;
            }
            Event::Window(window::Event::Unfocused) | Event::Mouse(mouse::Event::CursorLeft) => {
                if let Some((_, height)) = state.drag.take() {
                    shell.publish(handler(height));
                    consumed = true;
                }
            }
            _ => {}
        }
        if consumed {
            shell.capture_event();
            shell.request_redraw();
        }
        consumed
    }
    fn bounds(&self, size: Size, progress: f32) -> Rectangle {
        match self.placement {
            Placement::Side | Placement::Start => {
                let width = self
                    .width
                    .min((size.width - 56.0_f32.min(size.width / 4.0)).max(0.0));
                Rectangle {
                    x: if self.placement == Placement::Start {
                        -width * (1.0 - progress)
                    } else {
                        size.width - width * progress
                    },
                    y: 0.0,
                    width,
                    height: size.height,
                }
            }
            Placement::Bottom => {
                let width = self.width.min(size.width);
                let height = self
                    .height
                    .min((size.height - 56.0_f32.min(size.height / 4.0)).max(0.0));
                Rectangle {
                    x: (size.width - width) / 2.0,
                    y: size.height - height * progress,
                    width,
                    height,
                }
            }
        }
    }
    fn layout_content(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        size: Size,
        progress: f32,
    ) -> layout::Node {
        let bounds = self.bounds(size, progress);
        self.content
            .as_widget_mut()
            .layout(
                tree,
                renderer,
                &layout::Limits::new(bounds.size(), bounds.size()),
            )
            .move_to(bounds.position())
    }
    fn draw(
        &self,
        tree: &Tree,
        shadow: &crate::elevation::Cache,
        renderer: &mut Renderer,
        theme: &Theme,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;
        let bounds = layout.bounds();
        let Some(clip) = bounds.intersection(viewport) else {
            return;
        };
        let radius = match (self.placement, self.modal) {
            (Placement::Side, true) => iced::border::Radius {
                top_left: 16.0,
                bottom_left: 16.0,
                ..Default::default()
            },
            (Placement::Start, _) => iced::border::Radius {
                top_right: 16.0,
                bottom_right: 16.0,
                ..Default::default()
            },
            (Placement::Bottom, _) => iced::border::Radius {
                top_left: 28.0,
                top_right: 28.0,
                ..Default::default()
            },
            _ => 0.0.into(),
        };
        crate::elevation::draw(
            renderer,
            theme,
            bounds,
            radius,
            if self.placement == Placement::Bottom || self.modal {
                1.0
            } else {
                0.0
            },
            *viewport,
            shadow,
        );
        // Separate renderer layers ensure the opaque surface covers background
        // text/SVG primitives on both Tiny Skia and wgpu.
        renderer.with_layer(*viewport, |renderer| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        radius,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                if self.modal || self.placement == Placement::Bottom {
                    theme.colors.surface_container_low
                } else {
                    theme.colors.surface
                },
            );
            if self.on_height.is_some() && self.placement == Placement::Bottom {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.center_x() - 16.0,
                            y: bounds.y + 12.0,
                            width: 32.0,
                            height: 4.0,
                        },
                        border: Border {
                            radius: 2.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    theme.colors.on_surface_variant,
                );
            }
            if !self.modal {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            width: if self.placement == Placement::Bottom {
                                bounds.width
                            } else {
                                tokens::size::OUTLINE
                            },
                            height: if self.placement == Placement::Bottom {
                                tokens::size::OUTLINE
                            } else {
                                bounds.height
                            },
                            ..bounds
                        },
                        ..Default::default()
                    },
                    theme.colors.outline_variant,
                );
            }
        });
        renderer.with_layer(clip, |renderer| {
            self.content.as_widget().draw(
                tree,
                renderer,
                theme,
                &renderer::Style {
                    text_color: theme.colors.on_surface,
                },
                layout,
                cursor,
                &clip,
            );
        });
    }
}

/// Place at the root, inside any dialog host. The host fills available space.
/// Always retain it, using `sheet.open(false)` to close; no animation messages or
/// subscriptions are required. Modal sheets block input until their exit ends.
pub fn host<'a, Message: Clone + 'a>(
    background: impl Into<Element<'a, Message>>,
    sheet: Sheet<'a, Message>,
) -> Element<'a, Message> {
    Element::new(Host {
        background: background.into(),
        sheet,
    })
}
struct Host<'a, Message> {
    background: Element<'a, Message>,
    sheet: Sheet<'a, Message>,
}
struct State {
    shadow: crate::elevation::Cache,
    progress: Transition,
    motion: Cell<tokens::Motion>,
    was_open: bool,
    was_modal: bool,
    cancel_content: bool,
    content_suspended: bool,
    cancel_background: bool,
    background_suspended: bool,
    outside_pressed: bool,
    inactive: bool,
    drag: Option<(f32, f32)>,
    focus_pending: bool,
}
impl State {
    fn present(&self, open: bool) -> bool {
        open || self.progress.value > 0.0
    }
}
impl<'s, Message: Clone + 's> Widget<Message, Theme, Renderer> for Host<'s, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State {
            shadow: Default::default(),
            progress: Transition::standard(0.0),
            motion: Cell::new(tokens::Motion::default()),
            was_open: self.sheet.open,
            was_modal: self.sheet.modal,
            cancel_content: false,
            content_suspended: false,
            cancel_background: self.sheet.open && self.sheet.modal,
            background_suspended: false,
            outside_pressed: false,
            inactive: false,
            drag: None,
            focus_pending: self.sheet.open && self.sheet.placement == Placement::Start,
        })
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.background), Tree::new(&self.sheet.content)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.children[0].diff(&self.background);
        tree.children[1].diff(&self.sheet.content);
        let state = tree.state.downcast_mut::<State>();
        if state.was_open != self.sheet.open || state.was_modal != self.sheet.modal {
            state.outside_pressed = false;
            state.focus_pending = self.sheet.open && self.sheet.placement == Placement::Start;
            state.drag = None;
            state.cancel_content |= !self.sheet.open;
            state.cancel_background |= self.sheet.open && self.sheet.modal;
            state.was_open = self.sheet.open;
            state.was_modal = self.sheet.modal;
        }
    }
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits.resolve(Length::Fill, Length::Fill, Size::ZERO);
        let progress = tree.state.downcast_ref::<State>().progress.value;
        let panel = self
            .sheet
            .layout_content(&mut tree.children[1], renderer, size, progress);
        let background_size = Size::new(
            if !self.sheet.modal && self.sheet.placement == Placement::Side {
                self.sheet.bounds(size, progress).x
            } else {
                size.width
            },
            if !self.sheet.modal && self.sheet.placement == Placement::Bottom {
                self.sheet.bounds(size, progress).y
            } else {
                size.height
            },
        );
        let background = self.background.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &layout::Limits::new(background_size, background_size),
        );
        layout::Node::with_children(size, vec![background, panel])
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let present = tree.state.downcast_ref::<State>().present(self.sheet.open);
        if !(self.sheet.modal && present) {
            operation.container(None, layout.bounds());
            operation.traverse(&mut |op| {
                self.background.as_widget_mut().operate(
                    &mut tree.children[0],
                    layout.child(0),
                    renderer,
                    op,
                );
                if self.sheet.open {
                    self.sheet.content.as_widget_mut().operate(
                        &mut tree.children[1],
                        layout.child(1),
                        renderer,
                        op,
                    );
                }
            });
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
        if let Some(focused) = crate::activity::window_focus(event) {
            state.inactive = !focused;
        }
        if matches!(event, Event::Window(window::Event::Unfocused)) {
            state.outside_pressed = false;
        }
        let now = match event {
            Event::Window(window::Event::RedrawRequested(now)) => *now,
            _ => crate::motion::now(),
        };
        let before = state.progress.value;
        let motion = state.motion.get();
        let changed = state.progress.set(
            f32::from(self.sheet.open),
            now,
            if self.sheet.open {
                motion.sheet_enter
            } else {
                motion.sheet_exit
            },
        );
        if !state.inactive {
            if state.progress.tick(now) || changed {
                shell.request_redraw();
            }
            if state.progress.value != before {
                shell.invalidate_layout();
            }
        }
        // Cancellation flags survive even an entire open/close between redraws.
        // This prevents a covered background press from activating on release.
        if state.cancel_content {
            state.cancel_content = false;
            state.content_suspended = true;
            cancel(
                &mut self.sheet.content,
                &mut tree.children[1],
                layout.child(1),
                renderer,
                clipboard,
                viewport,
            );
        }
        if state.cancel_background {
            state.cancel_background = false;
            state.background_suspended = true;
            cancel(
                &mut self.background,
                &mut tree.children[0],
                layout.child(0),
                renderer,
                clipboard,
                viewport,
            );
        }
        let present = state.present(self.sheet.open);
        let blocked = self.sheet.modal && present;
        if self.sheet.open
            && !self.sheet.modal
            && state.content_suspended
            && !state.inactive
            && !crate::activity::is_covered()
        {
            state.content_suspended = false;
            crate::activity::interaction(|| {
                self.sheet.content.as_widget_mut().update(
                    &mut tree.children[1],
                    &Event::Window(window::Event::Focused),
                    layout.child(1),
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                );
            });
        }
        if !blocked && state.background_suspended && !crate::activity::is_covered() {
            state.background_suspended = false;
            if !state.inactive {
                crate::activity::interaction(|| {
                    self.background.as_widget_mut().update(
                        &mut tree.children[0],
                        &Event::Window(window::Event::Focused),
                        layout.child(0),
                        cursor,
                        renderer,
                        clipboard,
                        shell,
                        viewport,
                    );
                });
            }
        }
        if !self.sheet.modal
            && self.sheet.drag(
                state,
                event,
                layout.child(1).bounds(),
                layout.bounds().height,
                cursor,
                shell,
            )
        {
            return;
        }
        if blocked {
            crate::activity::update_covered(
                &mut self.background,
                &mut tree.children[0],
                event,
                layout.child(0),
                renderer,
                clipboard,
                shell,
                viewport,
                !state.inactive,
            );
            // An enclosing root dialog sends focus events directly to this
            // widget while hiding our overlay. Forward those to its content too.
            if matches!(
                event,
                Event::Window(window::Event::Focused | window::Event::Unfocused)
            ) {
                self.sheet.content.as_widget_mut().update(
                    &mut tree.children[1],
                    event,
                    layout.child(1),
                    mouse::Cursor::Unavailable,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                );
            }
            if is_input(event) {
                shell.capture_event();
            }
            return;
        }
        if self.sheet.open {
            self.sheet.content.as_widget_mut().update(
                &mut tree.children[1],
                event,
                layout.child(1),
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
            if !shell.is_event_captured() && is_escape(event) && self.sheet.escape {
                if let Some(message) = &self.sheet.dismiss {
                    shell.publish(message.clone());
                }
                shell.capture_event();
            }
        }
        if shell.is_event_captured() {
            return;
        }
        self.background.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.child(0),
            cursor,
            renderer,
            clipboard,
            shell,
            &layout.child(0).bounds(),
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
        use iced::advanced::Renderer as _;
        let state = tree.state.downcast_ref::<State>();
        state.motion.set(theme.motion);
        let present = state.present(self.sheet.open);
        renderer.with_layer(*viewport, |renderer| {
            self.background.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout.child(0),
                if self.sheet.modal && present {
                    mouse::Cursor::Unavailable
                } else {
                    cursor
                },
                viewport,
            );
        });
        if present && !self.sheet.modal {
            self.sheet.draw(
                &tree.children[1],
                &state.shadow,
                renderer,
                theme,
                layout.child(1),
                if self.sheet.open {
                    cursor
                } else {
                    mouse::Cursor::Unavailable
                },
                viewport,
            );
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
        if self.sheet.modal && tree.state.downcast_ref::<State>().present(self.sheet.open) {
            return mouse::Interaction::None;
        }
        if self.sheet.open && cursor.is_over(layout.child(1).bounds()) {
            return self.sheet.content.as_widget().mouse_interaction(
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
        let state = tree.state.downcast_mut::<State>();
        if self.sheet.modal && state.present(self.sheet.open) {
            return Some(overlay::Element::new(Box::new(SheetOverlay {
                sheet: &mut self.sheet,
                tree: &mut tree.children[1],
                state,
            })));
        }
        let (background, content) = tree.children.split_at_mut(1);
        let mut overlays = Vec::new();
        if let Some(overlay) = self.background.as_widget_mut().overlay(
            &mut background[0],
            layout.child(0),
            renderer,
            viewport,
            translation,
        ) {
            overlays.push(overlay);
        }
        if self.sheet.open
            && let Some(overlay) = self.sheet.content.as_widget_mut().overlay(
                &mut content[0],
                layout.child(1),
                renderer,
                viewport,
                translation,
            )
        {
            overlays.push(overlay);
        }
        if overlays.is_empty() {
            None
        } else {
            Some(overlay::Group::with_children(overlays).overlay())
        }
    }
}
fn cancel<Message>(
    content: &mut Element<'_, Message>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
    clipboard: &mut dyn Clipboard,
    viewport: &Rectangle,
) {
    crate::activity::interaction(|| {
        content.as_widget_mut().update(
            tree,
            &Event::Window(window::Event::Unfocused),
            layout,
            mouse::Cursor::Unavailable,
            renderer,
            clipboard,
            &mut Shell::new(&mut Vec::new()),
            viewport,
        );
    });
}
struct SheetOverlay<'a, 'b, Message> {
    sheet: &'a mut Sheet<'b, Message>,
    tree: &'a mut Tree,
    state: &'a mut State,
}
impl<'s, Message: Clone + 's> Overlay<Message, Theme, Renderer> for SheetOverlay<'_, 's, Message> {
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        layout::Node::with_children(
            bounds,
            vec![
                self.sheet
                    .layout_content(self.tree, renderer, bounds, self.state.progress.value),
            ],
        )
    }
    fn operate(&mut self, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        if self.sheet.open {
            self.sheet.content.as_widget_mut().operate(
                self.tree,
                layout.child(0),
                renderer,
                operation,
            );
        }
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
        if self.state.cancel_content {
            self.state.cancel_content = false;
            self.state.content_suspended = true;
            cancel(
                &mut self.sheet.content,
                self.tree,
                layout.child(0),
                renderer,
                clipboard,
                &layout.bounds(),
            );
        }
        if self.sheet.open
            && self.state.content_suspended
            && !self.state.inactive
            && !crate::activity::is_covered()
        {
            self.state.content_suspended = false;
            crate::activity::interaction(|| {
                self.sheet.content.as_widget_mut().update(
                    self.tree,
                    &Event::Window(window::Event::Focused),
                    layout.child(0),
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    &layout.bounds(),
                );
            });
        }

        if self.state.focus_pending && self.sheet.open {
            self.state.focus_pending = false;
            crate::focus::select(
                &mut self.sheet.content,
                self.tree,
                layout.child(0),
                renderer,
                0,
            );
            shell.request_redraw();
        }
        if crate::focus::tab(
            &mut self.sheet.content,
            self.tree,
            layout.child(0),
            renderer,
            event,
            shell,
        ) {
            return;
        }
        if matches!(event, Event::Mouse(mouse::Event::ButtonPressed(_))) {
            crate::focus::clear(
                &mut self.sheet.content,
                self.tree,
                layout.child(0),
                renderer,
            );
        }
        if self.sheet.drag(
            self.state,
            event,
            layout.child(0).bounds(),
            layout.bounds().height,
            cursor,
            shell,
        ) {
            return;
        }
        if is_escape(event) {
            if self.sheet.open
                && self.sheet.escape
                && let Some(message) = &self.sheet.dismiss
            {
                shell.publish(message.clone());
            }
            shell.capture_event();
            return;
        }
        let outside = cursor.position().is_some() && !cursor.is_over(layout.child(0).bounds());
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                self.state.outside_pressed = outside && self.sheet.open
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if self.state.outside_pressed
                    && outside
                    && self.sheet.open
                    && self.sheet.outside
                    && let Some(message) = &self.sheet.dismiss
                {
                    shell.publish(message.clone());
                }
                self.state.outside_pressed = false;
            }
            Event::Window(window::Event::Unfocused) => self.state.outside_pressed = false,
            _ => {}
        }
        if self.sheet.open {
            self.sheet.content.as_widget_mut().update(
                self.tree,
                event,
                layout.child(0),
                cursor,
                renderer,
                clipboard,
                shell,
                &layout.bounds(),
            );
        }
        if is_input(event) {
            shell.capture_event();
        }
    }
    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        _: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        use iced::advanced::Renderer as _;
        renderer.with_layer(layout.bounds(), |renderer| {
            let mut scrim = crate::theme::alpha(theme.colors.scrim, theme.colors.scrim.a * 0.32);
            scrim.a *= self.state.progress.value;
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    ..Default::default()
                },
                scrim,
            );
        });
        if self.state.progress.value > 0.0 {
            self.sheet.draw(
                self.tree,
                &self.state.shadow,
                renderer,
                theme,
                layout.child(0),
                if self.sheet.open {
                    cursor
                } else {
                    mouse::Cursor::Unavailable
                },
                &layout.bounds(),
            );
        }
    }
    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.sheet.open {
            self.sheet.content.as_widget().mouse_interaction(
                self.tree,
                layout.child(0),
                cursor,
                &layout.bounds(),
                renderer,
            )
        } else {
            mouse::Interaction::None
        }
    }
    fn overlay<'a>(
        &'a mut self,
        layout: Layout<'a>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        if self.sheet.open {
            self.sheet.content.as_widget_mut().overlay(
                self.tree,
                layout.child(0),
                renderer,
                &layout.bounds(),
                Vector::ZERO,
            )
        } else {
            None
        }
    }
}
fn is_input(event: &Event) -> bool {
    !matches!(event, Event::Window(_))
}
fn is_escape(event: &Event) -> bool {
    matches!(
        event,
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(keyboard::key::Named::Escape),
            ..
        })
    )
}
