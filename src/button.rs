//! Buttons with separate hover feedback and bounded pointer-origin press ripples.
use crate::{
    Element, Theme,
    motion::Transition,
    theme::{alpha, mix},
    tokens::{self, Motion},
};
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, overlay, renderer,
    widget::{Operation, Tree, tree},
};
use iced::{Border, Color, Event, Length, Rectangle, Renderer, Size, Vector, mouse, window};
use std::cell::{Cell, RefCell};

/// Material button treatment.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    #[default]
    Filled,
    Outlined,
    Text,
    Tonal,
    Elevated,
}

/// A pointer and keyboard action. Omitting `on_press` makes it disabled.
/// Child content should be non-interactive; rows of text and icons are supported.
pub struct Button<'a, Message> {
    content: Element<'a, Message>,
    on_press: Option<Message>,
    variant: ButtonVariant,
    width: Length,
    height: Length,
    padding: iced::Padding,
    custom_padding: bool,
    radius: f32,
    selected: bool,
    palette: Option<Box<dyn Fn(&Theme) -> (Color, Color) + 'a>>,
    icon: bool,
    toggle: bool,
    interactive_content: bool,
    content_disabled: bool,
    feedback_bounds: Option<fn(Layout<'_>) -> Rectangle>,
    min_height: f32,
    segment: bool,
    corners: Option<iced::border::Radius>,
    elevated: bool,
    chip: bool,
    expressive: bool,
    chip_assist: bool,
    elevation: Option<(u8, u8)>,
    card: Option<(crate::SurfaceVariant, bool, bool)>,
    outline_color: Option<Box<dyn Fn(&Theme, bool) -> Color + 'a>>,
}
/// Construct a button with Material label typography.
/// Use `Button::new` for arbitrary passive iced content.
pub fn button<'a, Message: 'a>(
    label: impl iced::widget::text::IntoFragment<'a>,
) -> Button<'a, Message> {
    Button::new(crate::components::typography(
        label,
        tokens::TypeScale::Label,
    ))
}
impl<'a, Message: 'a> Button<'a, Message> {
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            on_press: None,
            variant: ButtonVariant::Filled,
            width: Length::Shrink,
            height: Length::Shrink,
            padding: [10.0, tokens::spacing::LG].into(),
            custom_padding: false,
            radius: tokens::shape::FULL,
            selected: false,
            palette: None,
            icon: false,
            toggle: false,
            interactive_content: false,
            content_disabled: false,
            feedback_bounds: None,
            min_height: tokens::size::BUTTON,
            segment: false,
            corners: None,
            elevated: false,
            chip: false,
            expressive: false,
            chip_assist: false,
            elevation: None,
            card: None,
            outline_color: None,
        }
    }
    /// Override enabled background and foreground while retaining state layers.
    pub fn palette(mut self, palette: impl Fn(&Theme) -> (Color, Color) + 'a) -> Self {
        self.palette = Some(Box::new(palette));
        self
    }
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }
    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        if disabled {
            self.on_press = None;
        }
        self
    }
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        if !self.custom_padding {
            self.padding = [
                10.0,
                if variant == ButtonVariant::Text {
                    12.0
                } else {
                    24.0
                },
            ]
            .into();
        }
        self
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
    pub fn padding(mut self, padding: impl Into<iced::Padding>) -> Self {
        self.padding = padding.into();
        self.custom_padding = true;
        self
    }
    pub fn expressive(mut self, expressive: bool) -> Self {
        self.expressive = expressive;
        self
    }
    pub fn corner_radius(mut self, radius: iced::border::Radius) -> Self {
        self.corners = Some(radius);
        self
    }
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius.max(0.0);
        self
    }
    /// Selected treatment for toggle chips and icon buttons. The value is application-owned.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self.toggle = true;
        self
    }
    pub(crate) fn icon_style(mut self) -> Self {
        self.icon = true;
        self
    }
    pub(crate) fn elevated_style(mut self) -> Self {
        self.elevated = true;
        self
    }
    pub(crate) fn chip_style(mut self, selected: bool, assist: bool) -> Self {
        self.chip = true;
        self.chip_assist = assist;
        self.selected = selected;
        self
    }
    pub(crate) fn card_state(
        mut self,
        variant: crate::SurfaceVariant,
        disabled: bool,
        dragged: bool,
    ) -> Self {
        self.card = Some((variant, disabled, dragged));
        self
    }
    pub(crate) fn elevation_levels(mut self, resting: u8, hovered: u8) -> Self {
        self.elevation = Some((resting, hovered));
        self
    }
    pub(crate) fn outline_color(mut self, color: impl Fn(&Theme, bool) -> Color + 'a) -> Self {
        self.outline_color = Some(Box::new(color));
        self
    }
    pub(crate) fn minimum_height(mut self, height: f32) -> Self {
        self.min_height = height;
        self
    }
    pub(crate) fn segment_style(mut self, selected: bool, corners: iced::border::Radius) -> Self {
        self.segment = true;
        self.selected = selected;
        self.corners = Some(corners);
        self
    }
    /// List rows give interactive children first refusal of input.
    pub(crate) fn interactive_children(mut self, disabled: bool) -> Self {
        self.interactive_content = true;
        self.content_disabled = disabled;
        self
    }
    pub(crate) fn feedback_bounds(mut self, bounds: fn(Layout<'_>) -> Rectangle) -> Self {
        self.feedback_bounds = Some(bounds);
        self
    }
}

struct State {
    focus: crate::focus::Focus,
    keyboard_pressed: bool,
    shape: Transition,
    pressed: bool,
    layer: Transition,
    ripple: crate::ripple::PressRipple,
    origin: iced::Point,
    ripple_image: RefCell<Option<([f32; 10], iced::advanced::svg::Handle)>>,
    shadow: crate::elevation::Cache,
    motion: Cell<Motion>,
    content_disabled: bool,
    selection: Transition,
}
impl Default for State {
    fn default() -> Self {
        Self {
            focus: crate::focus::Focus::default(),
            keyboard_pressed: false,
            shape: Transition::standard(0.0),
            pressed: false,
            layer: Transition::new(0.0),
            ripple: crate::ripple::PressRipple::new(),
            origin: iced::Point::new(0.5, 0.5),
            ripple_image: RefCell::new(None),
            shadow: Default::default(),
            motion: Cell::new(Motion::default()),
            content_disabled: false,
            selection: Transition::standard(0.0),
        }
    }
}

impl<Message: Clone> Widget<Message, Theme, Renderer> for Button<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State {
            selection: Transition::standard(f32::from(self.selected)),
            ..State::default()
        })
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
        if self.content_disabled && !tree.state.downcast_ref::<State>().content_disabled {
            tree.children[0] = Tree::new(&self.content);
        }
        if self.on_press.is_none()
            && !(self.chip && self.interactive_content && !self.content_disabled)
        {
            *tree.state.downcast_mut::<State>() = State::default();
        }
        tree.state.downcast_mut::<State>().content_disabled = self.content_disabled;
    }
    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::positioned(
            &limits.min_height(match self.height {
                Length::Fixed(h) => h,
                _ => self.min_height,
            }),
            self.width,
            self.height,
            self.padding,
            |limits| {
                self.content.as_widget_mut().layout(
                    &mut tree.children[0],
                    renderer,
                    &limits.loose(),
                )
            },
            |content, available| {
                let size = content.size();
                content.translate(Vector::new(
                    (available.width - size.width) / 2.0,
                    (available.height - size.height) / 2.0,
                ))
            },
        )
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        if self.on_press.is_some() {
            let state = tree.state.downcast_mut::<State>();
            let id = state.focus.id.clone();
            operation.focusable(Some(&id), layout.bounds(), &mut state.focus);
            operation.custom(Some(&id), layout.bounds(), &mut state.focus);
        }
        if self.content_disabled {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout.child(0),
                renderer,
                &mut Passive(operation),
            );
            return;
        }
        operation.container(None, layout.bounds());
        operation.traverse(&mut |op| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                op,
            )
        });
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
        let mut child_captured = false;
        if matches!(event, Event::Window(_)) || (self.interactive_content && !self.content_disabled)
        {
            let mut messages = Vec::new();
            let mut local = Shell::new(&mut messages);
            self.content.as_widget_mut().update(
                &mut tree.children[0],
                event,
                layout.children().next().unwrap(),
                cursor,
                renderer,
                clipboard,
                &mut local,
                viewport,
            );
            child_captured = local.is_event_captured() || !local.is_empty();
            shell.merge(local, std::convert::identity);
        }
        let state = tree.state.downcast_mut::<State>();
        let was_pressed = state.pressed;
        if child_captured {
            state.pressed = false;
        }
        if self.on_press.is_none()
            && !(self.chip && self.interactive_content && !self.content_disabled)
        {
            return;
        }
        let child_target = self.interactive_content
            && self.content.as_widget().mouse_interaction(
                &tree.children[0],
                layout.child(0),
                cursor,
                viewport,
                renderer,
            ) != mouse::Interaction::None;
        let over = !child_captured
            && !child_target
            && cursor.is_over(layout.bounds())
            && cursor.is_over(*viewport);
        if state.keyboard_pressed && state.focus.key.is_none() {
            state.pressed = false;
            state.keyboard_pressed = false;
        }
        let now = match event {
            Event::Window(window::Event::RedrawRequested(now)) => *now,
            _ => crate::motion::now(),
        };
        if self.segment || self.chip {
            let changed =
                state
                    .selection
                    .set(f32::from(self.selected), now, state.motion.get().medium);
            if state.selection.tick(now) || changed {
                shell.request_redraw();
            }
        }
        match event {
            Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key: iced::keyboard::Key::Named(key),
                repeat,
                ..
            }) if state.focus.focused
                && self.on_press.is_some()
                && matches!(
                    key,
                    iced::keyboard::key::Named::Enter | iced::keyboard::key::Named::Space
                ) =>
            {
                if !repeat {
                    state.keyboard_pressed = true;
                    state.focus.key = Some(*key);
                    state.pressed = true;
                    state.origin = iced::Point::new(0.5, 0.5);
                    state.ripple.start(now, state.motion.get());
                }
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Keyboard(iced::keyboard::Event::KeyReleased {
                key: iced::keyboard::Key::Named(key),
                ..
            }) if state.focus.key == Some(*key) => {
                state.focus.key = None;
                state.keyboard_pressed = false;
                state.pressed = false;
                if state.focus.focused
                    && let Some(message) = &self.on_press
                {
                    shell.publish(message.clone());
                }
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if over && self.on_press.is_some() =>
            {
                state.pressed = true;
                state.focus.focused = true;
                state.focus.visible = false;
                let bounds = layout.bounds();
                if let Some(point) = cursor.position() {
                    state.origin = iced::Point::new(
                        ((point.x - bounds.x) / bounds.width.max(1.0)).clamp(0.0, 1.0),
                        ((point.y - bounds.y) / bounds.height.max(1.0)).clamp(0.0, 1.0),
                    );
                }
                state.ripple.start(now, state.motion.get());
                shell.request_redraw();
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.pressed && !state.keyboard_pressed =>
            {
                state.pressed = false;
                if over {
                    shell.publish(self.on_press.as_ref().unwrap().clone());
                }
                shell.capture_event();
            }
            Event::Window(window::Event::Unfocused) | Event::Mouse(mouse::Event::CursorLeft) => {
                state.pressed = false;
                state.focus.key = None;
            }
            _ => {}
        }
        if self.expressive {
            let changed = state
                .shape
                .set(f32::from(state.pressed), now, state.motion.get().short);
            if changed || state.shape.tick(now) {
                shell.request_redraw();
            }
        }
        if (was_pressed && !state.pressed) || (state.pressed && !over && !state.keyboard_pressed) {
            state.ripple.release(
                now,
                state.motion.get(),
                !over
                    && !matches!(
                        event,
                        Event::Keyboard(iced::keyboard::Event::KeyReleased { .. })
                    ),
            );
        }
        if state.ripple.tick(now, state.motion.get()) {
            shell.request_redraw();
        }
        let target = if over { 0.08 } else { 0.0 };
        if state.layer.set(target, now, state.motion.get().short) {
            shell.request_redraw();
        }
        if let Event::Window(window::Event::RedrawRequested(_)) = event
            && state.layer.tick(now)
        {
            shell.request_redraw();
        }
    }
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;
        let bounds = layout.bounds();
        let Some(clip) = bounds.intersection(viewport) else {
            return;
        };
        let state = tree.state.downcast_ref::<State>();
        state.motion.set(theme.motion);
        let c = theme.colors;
        let enabled =
            self.on_press.is_some() || (self.interactive_content && !self.content_disabled);
        let focused = state.focus.focused && state.focus.visible && !state.focus.suppressed;
        let layer_opacity = if focused {
            if self.icon && self.variant == ButtonVariant::Outlined {
                0.08
            } else {
                0.12
            }
        } else {
            state.layer.value
        };
        let dragged = self
            .card
            .is_some_and(|(_, disabled, dragged)| dragged && !disabled);
        let enabled = enabled || dragged;
        let layer_opacity = if dragged { 0.16 } else { layer_opacity };
        let (background, foreground) = if let Some((variant, true, _)) = self.card {
            (
                match variant {
                    crate::SurfaceVariant::Filled => alpha(c.surface_variant, 0.38),
                    crate::SurfaceVariant::Elevated => alpha(c.surface, 0.38),
                    crate::SurfaceVariant::Outlined => c.surface,
                },
                alpha(c.on_surface, 0.38),
            )
        } else if self.segment || self.chip {
            if enabled {
                (
                    if self.chip && self.variant == ButtonVariant::Elevated {
                        mix(
                            c.surface_container_low,
                            c.secondary_container,
                            state.selection.value,
                        )
                    } else {
                        alpha(c.secondary_container, state.selection.value)
                    },
                    mix(
                        if self.chip && !self.chip_assist {
                            c.on_surface_variant
                        } else {
                            c.on_surface
                        },
                        c.on_secondary_container,
                        state.selection.value,
                    ),
                )
            } else {
                (
                    if self.selected || (self.chip && self.variant == ButtonVariant::Elevated) {
                        alpha(c.on_surface, 0.12)
                    } else {
                        Color::TRANSPARENT
                    },
                    alpha(c.on_surface, 0.38),
                )
            }
        } else if !enabled {
            (
                if matches!(
                    self.variant,
                    ButtonVariant::Filled | ButtonVariant::Tonal | ButtonVariant::Elevated
                ) || (self.icon && self.selected && self.variant == ButtonVariant::Outlined)
                {
                    alpha(c.on_surface, 0.12)
                } else {
                    Color::TRANSPARENT
                },
                alpha(c.on_surface, 0.38),
            )
        } else if let Some(palette) = &self.palette {
            palette(theme)
        } else if self.icon {
            match self.variant {
                ButtonVariant::Elevated => (c.surface_container_low, c.primary),
                ButtonVariant::Text => (
                    Color::TRANSPARENT,
                    if self.selected {
                        c.primary
                    } else {
                        c.on_surface_variant
                    },
                ),
                ButtonVariant::Outlined if self.selected => {
                    (c.inverse_surface, c.inverse_on_surface)
                }
                ButtonVariant::Outlined => (
                    Color::TRANSPARENT,
                    if state.pressed {
                        c.on_surface
                    } else {
                        c.on_surface_variant
                    },
                ),
                ButtonVariant::Filled if self.toggle && !self.selected => {
                    (c.surface_container_highest, c.primary)
                }
                ButtonVariant::Filled => (c.primary, c.on_primary),
                ButtonVariant::Tonal if self.toggle && !self.selected => {
                    (c.surface_container_highest, c.on_surface_variant)
                }
                ButtonVariant::Tonal => (c.secondary_container, c.on_secondary_container),
            }
        } else if self.selected {
            (c.secondary_container, c.on_secondary_container)
        } else {
            match self.variant {
                ButtonVariant::Elevated => (c.surface_container_low, c.primary),
                ButtonVariant::Filled => (c.primary, c.on_primary),
                ButtonVariant::Tonal => (c.secondary_container, c.on_secondary_container),
                ButtonVariant::Outlined | ButtonVariant::Text => (Color::TRANSPARENT, c.primary),
            }
        };
        let mut radius = self.corners.unwrap_or(self.radius.into());
        if self.expressive {
            let shrink = |value: f32| {
                let value = value.min(bounds.height / 2.0);
                value + (12.0_f32.min(bounds.height / 2.0) - value) * state.shape.value
            };
            radius = iced::border::Radius {
                top_left: shrink(radius.top_left),
                top_right: shrink(radius.top_right),
                bottom_left: shrink(radius.bottom_left),
                bottom_right: shrink(radius.bottom_right),
            };
        }
        let border = Border {
            radius,
            width: if self.chip && self.variant == ButtonVariant::Elevated {
                0.0
            } else if self.chip {
                if enabled {
                    1.0 - state.selection.value
                } else if self.selected {
                    0.0
                } else {
                    1.0
                }
            } else if self.variant == ButtonVariant::Outlined && !(self.icon && self.selected) {
                tokens::size::OUTLINE
            } else {
                0.0
            },
            color: if enabled {
                if let Some(color) = &self.outline_color {
                    color(theme, focused)
                } else if self.chip && focused {
                    if self.chip_assist {
                        c.on_surface
                    } else {
                        c.on_surface_variant
                    }
                } else if focused && !self.icon && !self.segment {
                    c.primary
                } else {
                    c.outline
                }
            } else {
                alpha(
                    if self.card.is_some() {
                        c.outline
                    } else {
                        c.on_surface
                    },
                    0.12,
                )
            },
        };
        let elevation = self.elevation.or_else(|| {
            if self.elevated {
                Some((3, 4))
            } else if self.variant == ButtonVariant::Elevated {
                Some((1, 2))
            } else if !self.icon
                && !self.chip
                && !self.segment
                && self.palette.is_none()
                && matches!(self.variant, ButtonVariant::Filled | ButtonVariant::Tonal)
            {
                Some((0, 1))
            } else {
                None
            }
        });
        let elevation = match self.card {
            Some((variant, false, true)) => {
                let level = if variant == crate::SurfaceVariant::Elevated {
                    4
                } else {
                    3
                };
                Some((level, level))
            }
            Some((crate::SurfaceVariant::Elevated, true, _)) => Some((1, 1)),
            _ => elevation.filter(|_| enabled),
        };
        if let Some((resting, hovered)) = elevation {
            let amount = if state.pressed {
                0.0
            } else {
                (state.layer.value / 0.08).clamp(0.0, 1.0)
            };
            crate::elevation::draw(
                renderer,
                theme,
                bounds,
                radius,
                resting as f32 + (hovered as f32 - resting as f32) * amount,
                *viewport,
                &state.shadow,
            );
        }

        renderer.with_layer(clip, |renderer| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border,
                    ..Default::default()
                },
                background,
            );
            if enabled && layer_opacity > 0.0 && self.feedback_bounds.is_none() {
                // A shape-matched state layer avoids rectangular ripple leakage at rounded corners.
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: self.feedback_bounds.map_or(bounds, |area| area(layout)),
                        border: Border {
                            radius,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    alpha(foreground, layer_opacity),
                );
            }
            if enabled && self.feedback_bounds.is_none() && state.ripple.opacity.value > 0.0 {
                crate::ripple::draw_bounded(
                    renderer,
                    bounds,
                    clip,
                    radius,
                    state.origin,
                    state.ripple.expansion.value,
                    alpha(foreground, 0.12 * state.ripple.opacity.value),
                    &state.ripple_image,
                );
            }
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                &renderer::Style {
                    // Keep the disabled foreground's alpha: the renderer must
                    // blend it over this button's actual parent/container. An
                    // opaque sRGB mix against the page surface nearly vanishes
                    // on raised dark surfaces with wgpu's linear compositing.
                    text_color: foreground,
                },
                layout.children().next().unwrap(),
                cursor,
                &clip,
            );
            if enabled && state.focus.focused && state.focus.visible && !state.focus.suppressed {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x + 2.0,
                            y: bounds.y + 2.0,
                            width: (bounds.width - 4.0).max(0.0),
                            height: (bounds.height - 4.0).max(0.0),
                        },
                        border: Border {
                            width: 2.0,
                            color: foreground,
                            radius,
                        },
                        ..Default::default()
                    },
                    Color::TRANSPARENT,
                );
            }
            if enabled
                && layer_opacity > 0.0
                && let Some(area) = self.feedback_bounds
            {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: area(layout),
                        border: Border {
                            radius,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    alpha(foreground, layer_opacity),
                );
            }
            if enabled
                && state.ripple.opacity.value > 0.0
                && let Some(area) = self.feedback_bounds
            {
                let feedback = area(layout);
                crate::ripple::draw_bounded(
                    renderer,
                    feedback,
                    clip,
                    radius,
                    iced::Point::new(0.5, 0.5),
                    state.ripple.expansion.value,
                    alpha(foreground, 0.12 * state.ripple.opacity.value),
                    &state.ripple_image,
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
        if self.interactive_content && !self.content_disabled {
            let interaction = self.content.as_widget().mouse_interaction(
                &tree.children[0],
                layout.child(0),
                cursor,
                viewport,
                renderer,
            );
            if interaction != mouse::Interaction::None {
                return interaction;
            }
        }
        if self.on_press.is_some() && cursor.is_over(layout.bounds()) && cursor.is_over(*viewport) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
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
        if self.interactive_content && !self.content_disabled {
            self.content.as_widget_mut().overlay(
                &mut tree.children[0],
                layout.child(0),
                renderer,
                viewport,
                translation,
            )
        } else {
            None
        }
    }
}
impl<'a, Message: Clone + 'a> From<Button<'a, Message>> for Element<'a, Message> {
    fn from(value: Button<'a, Message>) -> Self {
        Self::new(value)
    }
}

// Disabled compound controls remain discoverable by their text, but their
// descendants must not enter Tab traversal or receive native editing operations.
struct Passive<'a>(&'a mut dyn Operation);
impl Operation for Passive<'_> {
    fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation)) {
        self.0.traverse(&mut |op| visit(&mut Passive(op)));
    }
    fn container(&mut self, id: Option<&iced::widget::Id>, bounds: Rectangle) {
        self.0.container(id, bounds);
    }
    fn text(&mut self, id: Option<&iced::widget::Id>, bounds: Rectangle, text: &str) {
        self.0.text(id, bounds, text);
    }
}
