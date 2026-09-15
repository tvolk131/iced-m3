//! Anchored action menus, context menus, and controlled dropdown selection.
pub use crate::anchored::Placement;
use crate::{
    Button, ButtonVariant, Element, Theme, TypeScale,
    anchored::{Anchored, PopupState, forward_shell},
    tokens, typography,
};
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, overlay, renderer,
    widget::{Operation, Tree, tree},
};
use iced::{
    Color, Event, Length, Rectangle, Renderer, Size, Vector, keyboard, mouse, widget, window,
};

use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

pub struct MenuItem<'a, Message> {
    label: String,
    action: Option<Message>,
    shortcut: Option<String>,
    supporting: Option<String>,
    leading: Option<Element<'a, Message>>,
    selected: bool,
    destructive: bool,
    separator: bool,
    children: Option<Vec<MenuItem<'a, Message>>>,
    disabled: bool,
}
impl<'a, Message> MenuItem<'a, Message> {
    pub fn new(label: impl Into<String>, action: Message) -> Self {
        Self {
            label: label.into(),
            action: Some(action),
            shortcut: None,
            supporting: None,
            leading: None,
            selected: false,
            destructive: false,
            separator: false,
            children: None,
            disabled: false,
        }
    }
    /// A cascading submenu. Right/Enter/Space open it; Left/Escape return.
    pub fn submenu(label: impl Into<String>, items: impl IntoIterator<Item = Self>) -> Self {
        let mut item = Self::separator();
        item.label = label.into();
        item.separator = false;
        item.children = Some(items.into_iter().collect());
        item
    }
    pub fn separator() -> Self {
        Self {
            label: String::new(),
            action: None,
            shortcut: None,
            supporting: None,
            leading: None,
            selected: false,
            destructive: false,
            separator: true,
            children: None,
            disabled: false,
        }
    }
    pub fn supporting_text(mut self, text: impl Into<String>) -> Self {
        self.supporting = Some(text.into());
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        if disabled {
            self.action = None;
        }
        self
    }
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    pub fn destructive(mut self, destructive: bool) -> Self {
        self.destructive = destructive;
        self
    }
    /// A visual hint only; the application owns shortcut bindings.
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }
    /// Passive leading content such as a square icon.
    pub fn leading(mut self, content: impl Into<Element<'a, Message>>) -> Self {
        self.leading = Some(content.into());
        self
    }
}
#[derive(Clone)]
enum Trigger<Message> {
    Toggle,
    Forward(Message),
}

pub struct Menu<'a, Message> {
    trigger: Element<'a, Trigger<Message>>,
    content: Element<'a, Message>,
    width: f32,
    max_height: f32,
    placement: Placement,
    disabled: bool,
    context: bool,
    match_width: bool,
    field_trigger: bool,
    item: bool,
    submenu: bool,
    active: usize,
    labels: Vec<String>,
    surface: (f32, u8),
    plain_surface: bool,
    surface_high: bool,
    close_when: Option<Box<dyn Fn(&Message) -> bool + 'a>>,
}
struct State {
    popup: PopupState,
    context_anchor: Option<Rectangle>,
    right_pressed: bool,
    parent: Option<Arc<AtomicU64>>,
    id: u64,
    hover_since: Option<Instant>,
}

impl Default for State {
    fn default() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self {
            popup: PopupState::default(),
            context_anchor: None,
            right_pressed: false,
            parent: None,
            id: NEXT.fetch_add(1, Ordering::Relaxed),
            hover_since: None,
        }
    }
}
/// Share the root dismissal signal and sibling selection through the widget tree.
pub(crate) fn link<Message>(
    content: &mut Element<'_, Message>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
    parent: &PopupState,
) {
    struct Link {
        signal: Arc<AtomicU64>,
        child: Arc<AtomicU64>,
        ancestors: Vec<Rectangle>,
        pointer: Arc<std::sync::Mutex<crate::anchored::PointerIntent>>,
    }
    impl Operation for Link {
        fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation)) {
            visit(self);
        }
        fn custom(&mut self, _: Option<&widget::Id>, _: Rectangle, state: &mut dyn std::any::Any) {
            if let Some(state) = state.downcast_mut::<State>() {
                if !Arc::ptr_eq(&state.popup.signal, &self.signal) {
                    state.popup.signal = self.signal.clone();
                    state.popup.generation = self.signal.load(Ordering::Relaxed);
                }
                state.popup.ancestors = self.ancestors.clone();
                state.parent = Some(self.child.clone());
                state.popup.parent_pointer = Some(self.pointer.clone());
            }
        }
    }
    let mut ancestors = parent.ancestors.clone();
    ancestors.push(layout.bounds());
    content.as_widget_mut().operate(
        tree,
        layout,
        renderer,
        &mut Link {
            signal: parent.signal.clone(),
            child: parent.child.clone(),
            ancestors,
            pointer: parent.pointer.clone(),
        },
    );
}

pub fn menu<'a, Message: Clone + 'a>(
    label: impl widget::text::IntoFragment<'a>,
    items: impl IntoIterator<Item = MenuItem<'a, Message>>,
) -> Menu<'a, Message> {
    menu_button(typography(label, TypeScale::LabelLarge), items)
}
/// An action menu with passive custom trigger content.
pub fn menu_button<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    items: impl IntoIterator<Item = MenuItem<'a, Message>>,
) -> Menu<'a, Message> {
    let content = widget::row![content.into(), Element::new(DropArrow)]
        .spacing(12)
        .align_y(iced::Alignment::Center);
    let content: Element<'a, Message> = content.into();
    let trigger = Button::new(content.map(Trigger::Forward))
        .variant(ButtonVariant::Outlined)
        .on_press(Trigger::Toggle);
    // The content cannot publish input messages inside Button; map its type to the trigger enum.
    build(trigger.into(), items, false)
}
/// Right-click a normal interactive region to open its contextual actions.
pub fn context_menu<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    items: impl IntoIterator<Item = MenuItem<'a, Message>>,
) -> Menu<'a, Message> {
    build(content.into().map(Trigger::Forward), items, true)
}
fn build<'a, Message: Clone + 'a>(
    trigger: Element<'a, Trigger<Message>>,
    items: impl IntoIterator<Item = MenuItem<'a, Message>>,
    context: bool,
) -> Menu<'a, Message> {
    let items: Vec<_> = items.into_iter().collect();
    let labels = items
        .iter()
        .filter(|i| !i.separator && !i.disabled && (i.action.is_some() || i.children.is_some()))
        .map(|i| i.label.trim().to_lowercase())
        .collect();
    let active = items
        .iter()
        .filter(|i| !i.separator && !i.disabled && (i.action.is_some() || i.children.is_some()))
        .position(|i| i.selected)
        .unwrap_or(0);
    let count = items.iter().filter(|item| !item.separator).count();
    let mut index = 0;
    let rows = widget::Column::with_children(items.into_iter().map(|item| {
        if item.separator {
            return widget::container(crate::divider()).padding([8, 0]).into();
        }
        let mut row = widget::Row::new()
            .spacing(12)
            .align_y(iced::Alignment::Center);
        if let Some(leading) = item.leading {
            row = row.push(widget::container(leading).center_x(24).center_y(24));
        }
        let mut label = widget::column![typography(item.label, TypeScale::BodyLarge)]
            .spacing(2)
            .width(Length::Fill);
        if let Some(supporting) = item.supporting {
            label = label.push(typography(supporting, TypeScale::BodySmall));
        }
        row = row.push(label);
        if let Some(shortcut) = item.shortcut {
            row = row.push(typography(shortcut, TypeScale::BodySmall));
        }
        if item.children.is_some() {
            row = row.push(Element::new(crate::glyph::Glyph::Next));
        }
        let selected = item.selected;
        let destructive = item.destructive;
        let row: Element<'a, Message> = row.into();
        let trigger = Button::new(row.map(Trigger::Forward))
            .variant(ButtonVariant::Text)
            .radius(0.0)
            .padding([12, 12])
            .width(Length::Fill)
            .palette(move |theme| {
                let c = theme.colors;
                if selected {
                    (c.secondary_container, c.on_secondary_container)
                } else {
                    (
                        Color::TRANSPARENT,
                        if destructive { c.error } else { c.on_surface },
                    )
                }
            })
            .on_press_maybe(if item.disabled {
                None
            } else if item.children.is_some() {
                Some(Trigger::Toggle)
            } else {
                item.action.map(Trigger::Forward)
            });
        let submenu = item.children.is_some();
        let mut menu = build(trigger.into(), item.children.unwrap_or_default(), false);
        menu.item = true;
        menu.submenu = submenu;
        menu.disabled = item.disabled;
        menu.placement = Placement::RightStart;
        let row = crate::staged::part(
            Element::from(menu),
            crate::staged::Part::MenuRow(index, count),
        );
        index += 1;
        row
    }));
    let content = widget::container(
        widget::scrollable(rows).width(Length::Fill).direction(
            widget::scrollable::Direction::Vertical(
                widget::scrollable::Scrollbar::new()
                    .width(3)
                    .scroller_width(3)
                    .spacing(3),
            ),
        ),
    )
    .padding([8, 0])
    .width(Length::Fill)
    .style(|theme: &Theme| widget::container::Style {
        text_color: Some(theme.colors.on_surface),
        ..Default::default()
    })
    .into();
    Menu {
        trigger,
        content,
        width: tokens::size::MENU_WIDTH,
        max_height: 360.0,
        placement: Placement::BottomStart,
        disabled: false,
        context,
        match_width: false,
        field_trigger: false,
        item: false,
        submenu: false,
        active,
        labels,
        surface: (4.0, 2),
        plain_surface: true,
        surface_high: false,
        close_when: None,
    }
}
pub(crate) fn split_menu<'a, Message: Clone + 'a>(
    items: impl IntoIterator<Item = MenuItem<'a, Message>>,
) -> Menu<'a, Message> {
    let trigger = Button::new(Element::new(crate::glyph::Glyph::Down))
        .padding([8, 12])
        .corner_radius(iced::border::Radius {
            top_left: 4.0,
            bottom_left: 4.0,
            top_right: 20.0,
            bottom_right: 20.0,
        })
        .expressive(true)
        .on_press(Trigger::Toggle)
        .into();
    build(trigger, items, false)
}
/// An explicitly opened rich hint. Its actions dismiss it; Escape and outside
/// clicks dismiss it without a message. Passive trigger content is wrapped in a button.
pub(crate) fn rich_popup<'a, Message: Clone + 'a>(
    trigger: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
) -> Menu<'a, Message> {
    let trigger: Element<'a, Message> = trigger.into();
    Menu {
        trigger: Button::new(trigger.map(Trigger::Forward))
            .variant(ButtonVariant::Text)
            .on_press(Trigger::Toggle)
            .into(),
        content: content.into(),
        width: 312.0,
        max_height: 480.0,
        placement: Placement::BottomStart,
        disabled: false,
        context: false,
        match_width: false,
        field_trigger: false,
        item: false,
        submenu: false,
        active: 0,
        labels: Vec::new(),
        surface: (12.0, 2),
        plain_surface: false,
        surface_high: false,
        close_when: None,
    }
}
impl<'a, Message> Menu<'a, Message> {
    pub(crate) fn surface(mut self, radius: f32, elevation: u8, high: bool) -> Self {
        self.surface = (radius, elevation);
        self.surface_high = high;
        self
    }
    pub(crate) fn close_when(mut self, predicate: impl Fn(&Message) -> bool + 'a) -> Self {
        self.close_when = Some(Box::new(predicate));
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(112.0);
        self
    }
    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = height.max(64.0);
        self
    }
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}
impl<Message: Clone> Widget<Message, Theme, Renderer> for Menu<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.trigger), Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.children[0].diff(&self.trigger);
        tree.children[1].diff(&self.content);
        if self.disabled {
            *tree.state.downcast_mut::<State>() = State::default();
            if !self.context {
                tree.children[0] = Tree::new(&self.trigger);
            }
        }
    }
    fn size(&self) -> Size<Length> {
        self.trigger.as_widget().size()
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.trigger
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
        operation.custom(None, layout.bounds(), tree.state.downcast_mut::<State>());
        if self.disabled || tree.state.downcast_ref::<State>().popup.open {
            crate::focus::without_focus(
                &mut self.trigger,
                &mut tree.children[0],
                layout,
                renderer,
                operation,
            );
        } else {
            self.trigger.as_widget_mut().operate(
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
        state.popup.synchronize();
        let motion = state.popup.presence.motion.get();
        state.popup.presence.update_surface(
            state.popup.open,
            event,
            motion.menu_enter,
            motion.menu_exit,
            false,
            shell,
        );
        if self.item {
            let over = cursor.is_over(layout.bounds()) && cursor.is_over(*viewport);
            let now = match event {
                Event::Window(window::Event::RedrawRequested(now)) => *now,
                _ => crate::motion::now(),
            };
            if over
                && let Some(deadline) = state
                    .popup
                    .parent_pointer
                    .as_ref()
                    .and_then(|pointer| pointer.lock().unwrap().deadline(now))
                && !matches!(
                    event,
                    Event::Mouse(mouse::Event::ButtonPressed(_) | mouse::Event::ButtonReleased(_))
                        | Event::Keyboard(_)
                )
            {
                state.hover_since = None;
                shell.request_redraw_at(deadline);
                return;
            }
            if matches!(
                event,
                Event::Mouse(mouse::Event::CursorMoved { .. })
                    | Event::Window(window::Event::RedrawRequested(_))
            ) {
                if over && !self.disabled {
                    if self.submenu && !state.popup.open {
                        state.hover_since.get_or_insert(now);
                    } else if !self.submenu
                        && let Some(parent) = &state.parent
                        && parent.swap(0, Ordering::Relaxed) != 0
                    {
                        shell.invalidate_layout();
                        shell.request_redraw();
                    }
                } else {
                    state.hover_since = None;
                }
            }
            if self.submenu
                && over
                && !self.disabled
                && let Some(since) = state.hover_since
            {
                let now = match event {
                    Event::Window(window::Event::RedrawRequested(now)) => *now,
                    _ => crate::motion::now(),
                };
                if now >= since + Duration::from_millis(200) {
                    if !state.popup.open {
                        state.popup.show(match event {
                            Event::Window(window::Event::RedrawRequested(now)) => *now,
                            _ => crate::motion::now(),
                        });
                        tree.children[1] = Tree::new(&self.content);
                        shell.invalidate_layout();
                        shell.request_redraw();
                    }
                    if let Some(parent) = &state.parent {
                        parent.store(state.id, Ordering::Relaxed);
                    }
                    state.hover_since = None;
                } else {
                    shell.request_redraw_at(window::RedrawRequest::At(
                        since + Duration::from_millis(200),
                    ));
                }
            }
            if state
                .parent
                .as_ref()
                .is_some_and(|p| p.load(Ordering::Relaxed) != state.id)
            {
                state.popup.open = false;
            }
        }
        if matches!(event, Event::Window(window::Event::Unfocused)) {
            state.popup.close(shell);
            state.right_pressed = false;
        }
        if state.popup.open || (self.disabled && !self.context) {
            // Input belongs to the popup, but the invoker must still finish its
            // press/release animation while that popup remains open.
            if state.popup.open && matches!(event, Event::Window(_)) {
                let mut ignored = Vec::new();
                let mut local = Shell::new(&mut ignored);
                self.trigger.as_widget_mut().update(
                    &mut tree.children[0],
                    event,
                    layout,
                    mouse::Cursor::Unavailable,
                    renderer,
                    clipboard,
                    &mut local,
                    viewport,
                );
                forward_shell(shell, &local);
            }
            return;
        }
        if self.context && !self.disabled {
            let over = cursor.is_over(layout.bounds()) && cursor.is_over(*viewport);
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) if over => {
                    state.right_pressed = true;
                    shell.capture_event();
                    return;
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right))
                    if state.right_pressed =>
                {
                    state.right_pressed = false;
                    if over {
                        state.popup.show(match event {
                            Event::Window(window::Event::RedrawRequested(now)) => *now,
                            _ => crate::motion::now(),
                        });
                        state.context_anchor = cursor
                            .position()
                            .map(|point| Rectangle::new(point, Size::ZERO));
                        tree.children[1] = Tree::new(&self.content);
                        shell.invalidate_layout();
                        shell.request_redraw();
                    }
                    shell.capture_event();
                    return;
                }
                Event::Mouse(mouse::Event::CursorLeft) => state.right_pressed = false,
                _ => {}
            }
        }
        let keyboard_open = if let Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(key),
            modifiers,
            ..
        }) = event
        {
            let desired = if self.submenu {
                *key == keyboard::key::Named::ArrowRight
            } else if self.context {
                *key == keyboard::key::Named::ContextMenu
                    || (*key == keyboard::key::Named::F10 && modifiers.shift())
            } else if self.item {
                false
            } else {
                matches!(
                    key,
                    keyboard::key::Named::ArrowDown | keyboard::key::Named::ArrowUp
                )
            };
            desired
                && !self.disabled
                && crate::focus::has_focus(
                    &mut self.trigger,
                    &mut tree.children[0],
                    layout,
                    renderer,
                )
        } else {
            false
        };
        if keyboard_open {
            state.popup.show(match event {
                Event::Window(window::Event::RedrawRequested(now)) => *now,
                _ => crate::motion::now(),
            });
            state.popup.initial_focus = Some(
                if matches!(
                    event,
                    Event::Keyboard(keyboard::Event::KeyPressed {
                        key: keyboard::Key::Named(keyboard::key::Named::ArrowUp),
                        ..
                    })
                ) {
                    usize::MAX
                } else {
                    self.active
                },
            );
            tree.children[1] = Tree::new(&self.content);
            if let Some(parent) = &state.parent {
                parent.store(state.id, Ordering::Relaxed);
            }
            shell.capture_event();
            shell.invalidate_layout();
            shell.request_redraw();
            return;
        }
        let mut messages = Vec::new();
        let mut local = Shell::new(&mut messages);
        self.trigger.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            &mut local,
            viewport,
        );
        forward_shell(shell, &local);
        for message in messages {
            match message {
                Trigger::Forward(message) => shell.publish(message),
                Trigger::Toggle if !self.disabled => {
                    state.popup.show(match event {
                        Event::Window(window::Event::RedrawRequested(now)) => *now,
                        _ => crate::motion::now(),
                    });
                    state.popup.initial_focus =
                        matches!(event, Event::Keyboard(_)).then_some(self.active);
                    if let Some(parent) = &state.parent {
                        parent.store(state.id, Ordering::Relaxed);
                    }
                    state.context_anchor = None;
                    tree.children[1] = Tree::new(&self.content);
                    shell.invalidate_layout();
                    shell.request_redraw();
                }
                Trigger::Toggle => {}
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
            .popup
            .presence
            .motion
            .set(theme.motion);
        if self.item && tree.state.downcast_ref::<State>().popup.open {
            use iced::advanced::Renderer as _;
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    ..Default::default()
                },
                crate::theme::alpha(theme.colors.on_surface, 0.08),
            );
        }
        let mut disabled_theme = theme.clone();
        if self.disabled && !self.context && !self.item && !self.field_trigger {
            disabled_theme.colors.primary = crate::theme::alpha(theme.colors.on_surface, 0.38);
            disabled_theme.colors.on_surface = disabled_theme.colors.primary;
            disabled_theme.colors.outline = crate::theme::alpha(theme.colors.on_surface, 0.12);
        }
        self.trigger.as_widget().draw(
            &tree.children[0],
            renderer,
            &disabled_theme,
            style,
            layout,
            if (self.disabled && !self.context) || tree.state.downcast_ref::<State>().popup.open {
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
        if self.disabled && !self.context {
            mouse::Interaction::None
        } else {
            self.trigger.as_widget().mouse_interaction(
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
        let state = tree.state.downcast_mut::<State>();
        state.popup.synchronize();
        if self.item
            && state
                .parent
                .as_ref()
                .is_some_and(|p| p.load(Ordering::Relaxed) != state.id)
        {
            state.popup.open = false;
        }
        crate::focus::suppress_ring(
            &mut self.trigger,
            &mut tree.children[0],
            layout,
            renderer,
            state.popup.open,
        );
        if !state.popup.presence.visible(state.popup.open) {
            return self
                .trigger
                .as_widget_mut()
                .overlay(
                    &mut tree.children[0],
                    layout,
                    renderer,
                    viewport,
                    translation,
                )
                .map(|overlay| {
                    overlay.map(&|message| match message {
                        Trigger::Forward(message) => message,
                        Trigger::Toggle => unreachable!("passive button has no overlay"),
                    })
                });
        }
        if (layout.bounds() + translation)
            .intersection(viewport)
            .is_none()
        {
            state.popup.open = false;
            return None;
        }
        let trigger_bounds = if self.field_trigger {
            Rectangle {
                y: layout.bounds().y + 8.0,
                height: tokens::size::FIELD,
                ..layout.bounds()
            }
        } else {
            layout.bounds()
        };
        let anchor = state.context_anchor.unwrap_or(trigger_bounds) + translation;
        Some(overlay::Element::new(Box::new(Anchored {
            content: &mut self.content,
            tree: &mut tree.children[1],
            anchor,
            placement: self.placement,
            width: if self.match_width {
                self.width.max(anchor.width)
            } else {
                self.width
            },
            max_height: self.max_height,
            gap: if self.context || self.item { 0.0 } else { 4.0 },
            menu: Some(&mut state.popup),
            close_when: self.close_when.as_deref(),
            labels: &self.labels,
            reveal: None,
            surface: Some(self.surface),
            plain_surface: self.plain_surface,
            surface_high: self.surface_high,
        })))
    }
}
impl<'a, Message: Clone + 'a> From<Menu<'a, Message>> for Element<'a, Message> {
    fn from(menu: Menu<'a, Message>) -> Self {
        Self::new(menu)
    }
}

#[derive(Clone, Debug)]
pub struct SelectOption<Value> {
    pub value: Value,
    pub label: String,
    pub disabled: bool,
}
impl<Value> SelectOption<Value> {
    pub fn new(value: Value, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
            disabled: false,
        }
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}
pub struct Select<'a, Value, Message> {
    label: String,
    options: Vec<SelectOption<Value>>,
    selected: Option<Value>,
    handler: Option<Box<dyn Fn(Value) -> Message + 'a>>,
    width: f32,
    variant: crate::TextFieldVariant,
    supporting: Option<String>,
    error: bool,
    leading: Option<Element<'a, Message>>,
    background: Option<Box<dyn Fn(&Theme) -> iced::Color + 'a>>,
}
pub fn select<'a, Value, Message>(
    label: impl Into<String>,
    options: impl IntoIterator<Item = SelectOption<Value>>,
    selected: Option<Value>,
) -> Select<'a, Value, Message> {
    Select {
        label: label.into(),
        options: options.into_iter().collect(),
        selected,
        handler: None,
        width: 240.0,
        variant: crate::TextFieldVariant::Outlined,
        supporting: None,
        error: false,
        leading: None,
        background: None,
    }
}
impl<'a, Value, Message> Select<'a, Value, Message> {
    /// Set an explicit trigger fill. Outlined selects are transparent by default,
    /// with a real gap in the outline around the floating label.
    pub fn background(mut self, color: iced::Color) -> Self {
        self.background = Some(Box::new(move |_| color));
        self
    }
    /// Resolve the trigger surface from the active theme, as for a text field.
    /// This does not change the popup menu's elevated surface.
    pub fn background_with(mut self, color: impl Fn(&Theme) -> iced::Color + 'a) -> Self {
        self.background = Some(Box::new(color));
        self
    }
    pub fn variant(mut self, variant: crate::TextFieldVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn supporting_text(mut self, text: impl Into<String>) -> Self {
        self.supporting = Some(text.into());
        self
    }
    pub fn error(mut self, text: impl Into<String>) -> Self {
        self.supporting = Some(text.into());
        self.error = true;
        self
    }
    pub fn leading(mut self, icon: impl Into<Element<'a, Message>>) -> Self {
        self.leading = Some(icon.into());
        self
    }
    pub fn on_select(mut self, handler: impl Fn(Value) -> Message + 'a) -> Self {
        self.handler = Some(Box::new(handler));
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        if disabled {
            self.handler = None;
        }
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(112.0);
        self
    }
}
impl<'a, Value: Clone + PartialEq + 'a, Message: Clone + 'a> From<Select<'a, Value, Message>>
    for Element<'a, Message>
{
    fn from(select: Select<'a, Value, Message>) -> Self {
        let current = select
            .options
            .iter()
            .find(|option| select.selected.as_ref() == Some(&option.value))
            .map(|option| option.label.clone())
            .unwrap_or_default();
        let disabled =
            select.handler.is_none() || select.options.iter().all(|option| option.disabled);
        let items = select.options.into_iter().map(|option| MenuItem {
            children: None,
            disabled: option.disabled,
            selected: select.selected.as_ref() == Some(&option.value),
            action: if option.disabled {
                None
            } else {
                select.handler.as_ref().map(|handler| handler(option.value))
            },
            label: option.label,
            shortcut: None,
            supporting: None,
            leading: None,
            destructive: false,
            separator: false,
        });
        let mut trigger = crate::text_field(&select.label, &current)
            .variant(select.variant)
            .width(select.width)
            .trailing(Element::new(DropArrow))
            .on_activate((!disabled).then_some(Trigger::Toggle));
        if let Some(background) = select.background {
            trigger = trigger.background_with(background);
        }
        if let Some(leading) = select.leading {
            trigger = trigger.leading(leading.map(Trigger::Forward));
        }
        if let Some(text) = select.supporting {
            trigger = if select.error {
                trigger.error(text)
            } else {
                trigger.supporting_text(text)
            };
        }
        let mut menu = build(trigger.into(), items, false)
            .width(select.width)
            .disabled(disabled);
        menu.match_width = true;
        menu.field_trigger = true;
        menu.into()
    }
}

// A vector arrow shares the trigger foreground and avoids font-baseline offsets.
struct DropArrow;
impl<Message> Widget<Message, Theme, Renderer> for DropArrow {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(24.0), Length::Fixed(24.0))
    }
    fn layout(&mut self, _: &mut Tree, _: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::atomic(limits, 24.0, 24.0)
    }
    fn draw(
        &self,
        _: &Tree,
        renderer: &mut Renderer,
        _: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        _: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        use iced::advanced::{
            Renderer as _,
            svg::{Handle, Renderer as _},
        };
        static HANDLE: std::sync::OnceLock<Handle> = std::sync::OnceLock::new();
        let handle = HANDLE.get_or_init(|| Handle::from_memory(br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path d="M7 9.5 L12 14.5 L17 9.5 Z"/></svg>"#.as_slice()));
        let bounds = layout.bounds();
        if let Some(clip) = bounds.intersection(viewport) {
            renderer.with_layer(clip, |renderer| {
                renderer.draw_svg(
                    crate::icon::tinted(handle.clone(), style.text_color),
                    bounds,
                    clip,
                )
            });
        }
    }
}
