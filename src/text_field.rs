//! Filled and outlined floating-label fields backed by iced's unmodified text input.
use crate::{
    Element, Theme,
    motion::Transition,
    theme::alpha,
    tokens::{self, Motion},
};
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, overlay, renderer,
    text::{self, Renderer as _},
    widget::{Operation, Tree, tree},
};
use iced::{
    Border, Color, Event, Length, Point, Rectangle, Renderer, Size, Vector, keyboard, mouse,
    widget::{self, text_input},
    window,
};
use std::cell::Cell;

type Paragraph = <Renderer as text::Renderer>::Paragraph;
type InputState = text_input::State<Paragraph>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextFieldVariant {
    #[default]
    Outlined,
    Filled,
}
/// A Material text field. No callback means disabled, matching iced.
pub struct TextField<'a, Message> {
    input: text_input::TextInput<'a, Message, Theme>,
    label: String,
    populated: bool,
    supporting: Option<String>,
    error: bool,
    enabled: bool,
    width: Length,
    background: Option<Box<dyn Fn(&Theme) -> Color + 'a>>,
    font: iced::Font,
    variant: TextFieldVariant,
    leading: Element<'a, Message>,
    has_leading: bool,
    trailing: Trailing<'a, Message>,
    prefix: String,
    suffix: String,
    activate: Option<Message>,
    selection_only: bool,
    value: String,
}
enum Trailing<'a, Message> {
    Passive(Element<'a, Message>, bool),
    Action(crate::Button<'a, Message>, Message),
}
impl<'a, Message: Clone + 'a> Trailing<'a, Message> {
    fn widget(&self) -> &dyn Widget<Message, Theme, Renderer> {
        match self {
            Self::Passive(e, _) => e.as_widget(),
            Self::Action(b, _) => b,
        }
    }
    fn widget_mut(&mut self) -> &mut dyn Widget<Message, Theme, Renderer> {
        match self {
            Self::Passive(e, _) => e.as_widget_mut(),
            Self::Action(b, _) => b,
        }
    }
    fn present(&self) -> bool {
        !matches!(self, Self::Passive(_, false))
    }
    fn sync(self, enabled: bool) -> Self {
        match self {
            Self::Action(b, message) => Self::Action(
                b.on_press_maybe(enabled.then_some(message.clone())),
                message,
            ),
            passive => passive,
        }
    }
}
pub fn text_field<'a, Message: Clone + 'a>(label: &str, value: &str) -> TextField<'a, Message> {
    TextField::new(label, value)
}
impl<'a, Message: Clone + 'a> TextField<'a, Message> {
    pub fn new(label: &str, value: &str) -> Self {
        crate::fonts::ensure_loaded();
        Self {
            input: text_input::TextInput::new("", value)
                .font(crate::fonts::REGULAR)
                .size(tokens::TypeScale::Body.metrics().0)
                .line_height(text::LineHeight::Absolute(24.0.into()))
                .padding([16.0, 16.0])
                .style(|theme: &Theme, status| {
                    let c = theme.colors;
                    text_input::Style {
                        background: Color::TRANSPARENT.into(),
                        border: Border::default(),
                        icon: c.on_surface_variant,
                        placeholder: c.on_surface_variant,
                        value: if status == text_input::Status::Disabled {
                            alpha(c.on_surface, 0.38)
                        } else {
                            c.on_surface
                        },
                        selection: c.primary_container,
                    }
                }),
            label: label.into(),
            populated: !value.is_empty(),
            supporting: None,
            error: false,
            enabled: false,
            width: Length::Fill,
            background: None,
            font: crate::fonts::REGULAR,
            variant: TextFieldVariant::Outlined,
            leading: widget::space().into(),
            has_leading: false,
            trailing: Trailing::Passive(widget::space().into(), false),
            prefix: String::new(),
            suffix: String::new(),
            activate: None,
            selection_only: false,
            value: value.into(),
        }
    }
    /// Passive 24px icon slot, 12px from the leading edge.
    pub fn leading(mut self, icon: impl Into<Element<'a, Message>>) -> Self {
        self.leading = widget::container(icon).center_x(24).center_y(24).into();
        self.has_leading = true;
        self
    }
    /// Passive trailing icon. Use `trailing_action` for a keyboard-reachable action.
    pub fn trailing(mut self, icon: impl Into<Element<'a, Message>>) -> Self {
        self.trailing = Trailing::Passive(
            widget::container(icon).center_x(24).center_y(24).into(),
            true,
        );
        self
    }
    /// A 40px trailing icon button, disabled together with its field.
    pub fn trailing_action(
        mut self,
        icon: impl Into<Element<'a, Message>>,
        message: Message,
    ) -> Self {
        self.trailing = Trailing::Action(crate::icon_button(icon), message).sync(self.enabled);
        self
    }
    /// Display an affix without adding it to the editable value or clipboard.
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }
    /// Display a unit or other suffix outside the editable value.
    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = suffix.into();
        self
    }
    // A selection-only field shares the exact field drawing recipe, but owns a
    // focus target instead of focusing the native text editor or starting an IME.
    pub(crate) fn on_activate(mut self, message: Option<Message>) -> Self {
        self.selection_only = true;
        self.enabled = message.is_some();
        let enabled = self.enabled;
        // A selection-only field never updates/focuses the native editor, so its
        // cached native status stays Disabled. Its visual status belongs to us.
        self.input = self.input.style(move |theme: &Theme, _| text_input::Style {
            background: Color::TRANSPARENT.into(),
            border: Border::default(),
            icon: theme.colors.on_surface_variant,
            placeholder: theme.colors.on_surface_variant,
            value: if enabled {
                theme.colors.on_surface
            } else {
                alpha(theme.colors.on_surface, 0.38)
            },
            selection: theme.colors.primary_container,
        });
        self.activate = message;
        self.trailing = self.trailing.sync(self.enabled);
        self
    }
    pub fn variant(mut self, variant: TextFieldVariant) -> Self {
        self.variant = variant;
        self.input = self.input.padding(if variant == TextFieldVariant::Filled {
            iced::Padding {
                top: 24.0,
                bottom: 8.0,
                left: 16.0,
                right: 16.0,
            }
        } else {
            iced::Padding::from([16, 16])
        });
        self
    }
    pub fn on_input(mut self, handler: impl Fn(String) -> Message + 'a) -> Self {
        self.input = self.input.on_input(handler);
        self.enabled = true;
        self.trailing = self.trailing.sync(true);
        self
    }
    pub fn on_submit(mut self, message: Message) -> Self {
        self.input = self.input.on_submit(message);
        self
    }
    pub fn on_paste(mut self, handler: impl Fn(String) -> Message + 'a) -> Self {
        self.input = self.input.on_paste(handler);
        self
    }
    pub fn id(mut self, id: impl Into<iced::widget::Id>) -> Self {
        self.input = self.input.id(id);
        self
    }
    pub fn secure(mut self, secure: bool) -> Self {
        self.input = self.input.secure(secure);
        self
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
    /// Set the same typeface for the value, floating label, and supporting text.
    pub fn font(mut self, font: iced::Font) -> Self {
        self.input = self.input.font(font);
        self.font = font;
        self
    }
    pub fn supporting_text(mut self, text: impl Into<String>) -> Self {
        self.supporting = Some(text.into());
        self
    }
    /// Error text replaces supporting text and changes outline and label roles.
    pub fn error(mut self, text: impl Into<String>) -> Self {
        self.supporting = Some(text.into());
        self.error = true;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        if disabled {
            self.input = self.input.on_input_maybe(None::<fn(String) -> Message>);
            self.enabled = false;
            self.activate = None;
            self.trailing = self.trailing.sync(false);
        }
        self
    }
    /// Set an explicit field fill. Outlined fields are transparent by default;
    /// their floating label uses a real gap in the border, without a painted patch.
    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(Box::new(move |_| color));
        self
    }
    /// Resolve an explicit field fill from the theme. This does not paint outside
    /// the field behind its floating label.
    pub fn background_with(mut self, color: impl Fn(&Theme) -> Color + 'a) -> Self {
        self.background = Some(Box::new(color));
        self
    }
}

struct State {
    floating: Transition,
    hover: Transition,
    motion: Cell<Motion>,
    label: text::paragraph::Plain<Paragraph>,
    supporting: text::paragraph::Plain<Paragraph>,
    prefix: text::paragraph::Plain<Paragraph>,
    suffix: text::paragraph::Plain<Paragraph>,
    prefix_width: f32,
    suffix_width: f32,
    focus: crate::focus::Focus,
    pressed: bool,
}
impl<'a, Message: Clone + 'a> Widget<Message, Theme, Renderer> for TextField<'a, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State {
            floating: Transition::new(if self.populated { 1.0 } else { 0.0 }),
            hover: Transition::new(0.0),
            motion: Cell::new(Motion::default()),
            label: Default::default(),
            supporting: Default::default(),
            prefix: Default::default(),
            suffix: Default::default(),
            prefix_width: 0.0,
            suffix_width: 0.0,
            focus: Default::default(),
            pressed: false,
        })
    }
    fn children(&self) -> Vec<Tree> {
        vec![
            Tree::new(&self.input as &dyn Widget<Message, Theme, Renderer>),
            Tree::new(self.leading.as_widget()),
            Tree::new(self.trailing.widget()),
        ]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.children[0].diff(&self.input as &dyn Widget<Message, Theme, Renderer>);
        tree.children[1].diff(self.leading.as_widget());
        tree.children[2].diff(self.trailing.widget());
        if !self.enabled {
            let state = tree.state.downcast_mut::<State>();
            state.focus.focused = false;
            state.focus.key = None;
            state.pressed = false;
            tree.children[0]
                .state
                .downcast_mut::<InputState>()
                .unfocus();
        }
    }
    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Shrink)
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width);
        let width = limits
            .resolve(self.width, Length::Shrink, Size::new(280.0, 0.0))
            .width;
        let state = tree.state.downcast_mut::<State>();
        let leading = if self.has_leading { 36.0 } else { 0.0 };
        let trailing = if self.trailing.present() { 36.0 } else { 0.0 };
        let available = (width - 32.0 - leading - trailing).max(0.0);
        for (text, paragraph) in [
            (&self.prefix, &mut state.prefix),
            (&self.suffix, &mut state.suffix),
        ] {
            paragraph.update(super::text_field::paragraph(
                text,
                available / 3.0,
                16.0,
                24.0,
                text::Wrapping::None,
                self.font,
            ));
        }
        state.prefix_width = if self.prefix.is_empty() {
            0.0
        } else {
            state.prefix.min_width().min(available / 3.0) + 4.0
        };
        state.suffix_width = if self.suffix.is_empty() {
            0.0
        } else {
            state.suffix.min_width().min(available / 3.0) + 4.0
        };
        // TextInput consumes padding in layout. Moving it out and back preserves
        // its callbacks and its separate, stable widget tree (caret/selection/IME).
        let input = std::mem::replace(&mut self.input, text_input::TextInput::new("", ""));
        self.input = input.padding(iced::Padding {
            left: 16.0 + leading + state.prefix_width,
            right: 16.0 + trailing + state.suffix_width,
            top: if self.variant == TextFieldVariant::Filled {
                24.0
            } else {
                16.0
            },
            bottom: if self.variant == TextFieldVariant::Filled {
                8.0
            } else {
                16.0
            },
        });
        let label_size = 16.0 - 4.0 * state.floating.value;
        state.label.update(paragraph(
            &self.label,
            available,
            label_size,
            24.0 - 8.0 * state.floating.value,
            text::Wrapping::None,
            self.font,
        ));
        state.supporting.update(paragraph(
            self.supporting.as_deref().unwrap_or(""),
            width - 32.0,
            12.0,
            16.0,
            text::Wrapping::WordOrGlyph,
            self.font,
        ));
        let input = Widget::layout(
            &mut self.input,
            &mut tree.children[0],
            renderer,
            &layout::Limits::new(Size::ZERO, Size::new(width, tokens::size::FIELD)),
        )
        .move_to(Point::new(0.0, 8.0));
        let support = if self.supporting.is_some() {
            state.supporting.min_height() + 8.0
        } else {
            0.0
        };
        let leading_node = self
            .leading
            .as_widget_mut()
            .layout(
                &mut tree.children[1],
                renderer,
                &layout::Limits::new(Size::ZERO, Size::new(24.0, 24.0)),
            )
            .move_to(Point::new(12.0, 24.0));
        let side = if matches!(self.trailing, Trailing::Action(..)) {
            40.0
        } else {
            24.0
        };
        let trailing_node = self
            .trailing
            .widget_mut()
            .layout(
                &mut tree.children[2],
                renderer,
                &layout::Limits::new(Size::ZERO, Size::new(side, side)),
            )
            .move_to(Point::new(
                (width - 28.0 - side / 2.0).max(0.0),
                36.0 - side / 2.0,
            ));
        layout::Node::with_children(
            Size::new(width, 8.0 + tokens::size::FIELD + support),
            vec![input, leading_node, trailing_node],
        )
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        if self.selection_only {
            let state = tree.state.downcast_mut::<State>();
            let id = state.focus.id.clone();
            if self.enabled {
                operation.focusable(Some(&id), layout.child(0).bounds(), &mut state.focus);
                operation.custom(Some(&id), layout.child(0).bounds(), &mut state.focus);
            }
            operation.text(None, layout.child(0).bounds(), &self.label);
            operation.text(None, layout.child(0).bounds(), &self.value);
        } else {
            self.input
                .operate(&mut tree.children[0], layout.child(0), renderer, operation);
            if self.enabled && matches!(self.trailing, Trailing::Action(..)) {
                self.trailing.widget_mut().operate(
                    &mut tree.children[2],
                    layout.child(2),
                    renderer,
                    operation,
                );
            }
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
        let over_slot = (self.has_leading && cursor.is_over(layout.child(1).bounds()))
            || (self.trailing.present() && cursor.is_over(layout.child(2).bounds()));
        if let Some(message) = &self.activate {
            let state = tree.state.downcast_mut::<State>();
            let over = cursor.is_over(layout.child(0).bounds()) && cursor.is_over(*viewport);
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    state.pressed = over;
                    state.focus.focused = over;
                    state.focus.visible = false;
                    if over {
                        shell.capture_event();
                        shell.request_redraw();
                    }
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                    if state.pressed =>
                {
                    state.pressed = false;
                    if over {
                        shell.publish(message.clone());
                    }
                    shell.capture_event();
                    shell.request_redraw();
                }
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(key),
                    repeat,
                    ..
                }) if state.focus.focused
                    && matches!(
                        key,
                        keyboard::key::Named::Enter | keyboard::key::Named::Space
                    ) =>
                {
                    if !repeat {
                        state.focus.key = Some(*key);
                    }
                    shell.capture_event();
                }
                Event::Keyboard(keyboard::Event::KeyReleased {
                    key: keyboard::Key::Named(key),
                    ..
                }) if state.focus.key == Some(*key) => {
                    state.focus.key = None;
                    if state.focus.focused {
                        shell.publish(message.clone());
                    }
                    shell.capture_event();
                    shell.request_redraw();
                }
                Event::Window(window::Event::Unfocused)
                | Event::Mouse(mouse::Event::CursorLeft) => {
                    state.pressed = false;
                    state.focus.key = None;
                }
                _ => {}
            }
        } else {
            if matches!(self.trailing, Trailing::Action(..)) {
                self.trailing.widget_mut().update(
                    &mut tree.children[2],
                    event,
                    layout.child(2),
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                );
            }
            if !(shell.is_event_captured()
                || over_slot && matches!(event, Event::Mouse(mouse::Event::ButtonPressed(_))))
            {
                self.input.update(
                    &mut tree.children[0],
                    event,
                    layout.child(0),
                    if over_slot {
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
        }
        let focused = if self.activate.is_some() {
            tree.state.downcast_ref::<State>().focus.focused
        } else {
            tree.children[0]
                .state
                .downcast_ref::<InputState>()
                .is_focused()
        };
        let state = tree.state.downcast_mut::<State>();
        let now = match event {
            Event::Window(window::Event::RedrawRequested(now)) => *now,
            _ => crate::motion::now(),
        };
        let field = layout.children().next().unwrap().bounds();
        let hovered = self.enabled && cursor.is_over(field) && cursor.is_over(*viewport);
        if state
            .hover
            .set(f32::from(hovered), now, state.motion.get().short)
        {
            shell.request_redraw();
        }
        if state.floating.set(
            if focused || self.populated { 1.0 } else { 0.0 },
            now,
            state.motion.get().medium,
        ) {
            shell.request_redraw();
            shell.invalidate_layout();
        }
        if let Event::Window(window::Event::RedrawRequested(_)) = event {
            if state.hover.tick(now) {
                shell.request_redraw();
            }
            let before = state.floating.value;
            if state.floating.tick(now) {
                shell.request_redraw();
            }
            if before != state.floating.value {
                shell.invalidate_layout();
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
        use iced::advanced::Renderer as _;
        let bounds = layout.bounds();
        let Some(clip) = bounds.intersection(viewport) else {
            return;
        };
        let field = Rectangle {
            y: bounds.y + 8.0,
            height: tokens::size::FIELD,
            ..bounds
        };
        let state = tree.state.downcast_ref::<State>();
        state.motion.set(theme.motion);
        let focused = self.enabled
            && if self.activate.is_some() {
                state.focus.focused
            } else {
                tree.children[0]
                    .state
                    .downcast_ref::<InputState>()
                    .is_focused()
            };
        let c = theme.colors;
        let filled = self.variant == TextFieldVariant::Filled;
        let hovered = cursor.is_over(field) && cursor.is_over(*viewport);
        let outline = if !self.enabled {
            alpha(c.on_surface, if filled { 0.38 } else { 0.12 })
        } else if self.error && hovered && !focused {
            c.on_error_container
        } else if self.error {
            c.error
        } else if focused {
            c.primary
        } else if hovered {
            c.on_surface
        } else if filled {
            c.on_surface_variant
        } else {
            c.outline
        };
        let label_color = if !self.enabled {
            alpha(c.on_surface, 0.38)
        } else if focused || self.error || (hovered && !filled) {
            outline
        } else {
            c.on_surface_variant
        };
        let background = if filled && !self.enabled {
            alpha(c.on_surface, 0.04)
        } else {
            self.background
                .as_ref()
                .map(|color| color(theme))
                .unwrap_or(if filled {
                    c.surface_container_highest
                } else {
                    Color::TRANSPARENT
                })
        };
        let progress = state.floating.value;
        let leading = if self.has_leading { 36.0 } else { 0.0 };
        let label_x = field.x + 16.0 + leading * if filled { 1.0 } else { 1.0 - progress };
        let label_height = 24.0 - 8.0 * progress;
        let label_y = field.y + 16.0 - if filled { 8.0 } else { 24.0 } * progress;
        let label_width = state.label.min_width().min((field.width - 32.0).max(0.0));
        renderer.with_layer(clip, |renderer| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: field,
                    border: Border {
                        color: outline,
                        width: 0.0,
                        radius: if filled {
                            iced::border::Radius {
                                top_left: 4.0,
                                top_right: 4.0,
                                bottom_left: 0.0,
                                bottom_right: 0.0,
                            }
                        } else {
                            tokens::shape::SMALL.into()
                        },
                    },
                    ..Default::default()
                },
                background,
            );
            if !filled {
                let border = renderer::Quad {
                    bounds: field,
                    border: Border {
                        color: outline,
                        width: if focused {
                            tokens::size::FOCUS_OUTLINE
                        } else {
                            tokens::size::OUTLINE
                        },
                        radius: tokens::shape::SMALL.into(),
                    },
                    ..Default::default()
                };
                // Clip the outline around the moving label instead of erasing
                // it with an assumed parent color. The fill remains independent.
                let notch_left = (label_x - 4.0).max(field.x + tokens::shape::SMALL);
                let notch_right =
                    (label_x + label_width + 4.0).min(field.x + field.width - tokens::shape::SMALL);
                let notch = Rectangle {
                    x: notch_left,
                    y: label_y,
                    width: (notch_right - notch_left).max(0.0),
                    height: label_height,
                };
                if progress > 0.0
                    && notch.y < field.y + border.border.width
                    && let Some(gap) = notch.intersection(&clip)
                {
                    for piece in [
                        Rectangle {
                            height: gap.y - clip.y,
                            ..clip
                        },
                        Rectangle {
                            y: gap.y + gap.height,
                            height: clip.y + clip.height - gap.y - gap.height,
                            ..clip
                        },
                        Rectangle {
                            x: clip.x,
                            width: gap.x - clip.x,
                            ..gap
                        },
                        Rectangle {
                            x: gap.x + gap.width,
                            width: clip.x + clip.width - gap.x - gap.width,
                            ..gap
                        },
                    ] {
                        if piece.width > 0.0 && piece.height > 0.0 {
                            renderer.with_layer(piece, |renderer| {
                                renderer.fill_quad(border, Color::TRANSPARENT);
                            });
                        }
                    }
                } else {
                    renderer.fill_quad(border, Color::TRANSPARENT);
                }
            }
            if filled && self.enabled && state.hover.value > 0.0 {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: field,
                        border: Border {
                            radius: iced::border::Radius {
                                top_left: 4.0,
                                top_right: 4.0,
                                bottom_left: 0.0,
                                bottom_right: 0.0,
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    alpha(c.on_surface, 0.08 * state.hover.value),
                );
            }
            if filled {
                let height = if focused { 2.0 } else { 1.0 };
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: field.x,
                            y: field.y + field.height - height,
                            width: field.width,
                            height,
                        },
                        ..Default::default()
                    },
                    outline,
                );
            }
            Widget::draw(
                &self.input,
                &tree.children[0],
                renderer,
                theme,
                style,
                layout.children().next().unwrap(),
                cursor,
                &clip,
            );
            let adornment_style = renderer::Style {
                text_color: if self.enabled {
                    c.on_surface_variant
                } else {
                    alpha(c.on_surface, 0.38)
                },
            };
            self.leading.as_widget().draw(
                &tree.children[1],
                renderer,
                theme,
                &adornment_style,
                layout.child(1),
                cursor,
                &clip,
            );
            self.trailing.widget().draw(
                &tree.children[2],
                renderer,
                theme,
                &adornment_style,
                layout.child(2),
                cursor,
                &clip,
            );
            let trailing = if self.trailing.present() { 36.0 } else { 0.0 };
            let text_y = field.y + if filled { 24.0 } else { 16.0 };
            let affix_color = alpha(
                adornment_style.text_color,
                progress * adornment_style.text_color.a,
            );
            for (paragraph, x, width) in [
                (
                    &state.prefix,
                    field.x + 16.0 + leading,
                    state.prefix_width - 4.0,
                ),
                (
                    &state.suffix,
                    field.x + field.width - 16.0 - trailing - state.suffix_width + 4.0,
                    state.suffix_width - 4.0,
                ),
            ] {
                if width > 0.0
                    && let Some(affix_clip) = (Rectangle {
                        x,
                        y: text_y,
                        width,
                        height: 24.0,
                    })
                    .intersection(&clip)
                {
                    renderer.fill_paragraph(
                        paragraph.raw(),
                        Point::new(x, text_y),
                        affix_color,
                        affix_clip,
                    );
                }
            }
            let label_clip = Rectangle {
                x: label_x,
                y: bounds.y,
                width: (field.x + field.width - 16.0 - label_x).max(0.0),
                height: field.height + 8.0,
            };
            if let Some(label_clip) = label_clip.intersection(&clip) {
                renderer.fill_paragraph(
                    state.label.raw(),
                    Point::new(label_x, label_y),
                    label_color,
                    label_clip,
                );
            }
            if self.supporting.is_some() {
                renderer.fill_paragraph(
                    state.supporting.raw(),
                    Point::new(field.x + 16.0, field.y + field.height + 4.0),
                    if !self.enabled {
                        alpha(c.on_surface, 0.38)
                    } else if self.error {
                        c.error
                    } else {
                        c.on_surface_variant
                    },
                    clip,
                );
            }
        });
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.activate.is_some()
            && cursor.is_over(layout.child(0).bounds())
            && cursor.is_over(*viewport)
        {
            return mouse::Interaction::Pointer;
        }
        if self.trailing.present() && cursor.is_over(layout.child(2).bounds()) {
            return self.trailing.widget().mouse_interaction(
                &tree.children[2],
                layout.child(2),
                cursor,
                viewport,
                renderer,
            );
        }
        self.input.mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
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
        self.input.overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}
fn paragraph(
    content: &str,
    width: f32,
    size: f32,
    height: f32,
    wrapping: text::Wrapping,
    font: iced::Font,
) -> text::Text<&str> {
    text::Text {
        content,
        bounds: Size::new(width.max(0.0), f32::INFINITY),
        size: size.into(),
        line_height: text::LineHeight::Absolute(height.into()),
        font,
        align_x: text::Alignment::Left,
        align_y: iced::alignment::Vertical::Top,
        shaping: text::Shaping::Advanced,
        wrapping,
    }
}
impl<'a, Message: Clone + 'a> From<TextField<'a, Message>> for Element<'a, Message> {
    fn from(value: TextField<'a, Message>) -> Self {
        Self::new(value)
    }
}
