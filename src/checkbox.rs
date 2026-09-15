//! Material checkbox with a circular state layer and finite selection motion.
use crate::{Element, Theme, fonts, motion::Transition, theme::alpha, tokens};
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, renderer,
    svg::{Handle, Renderer as _, Svg},
    widget::{Operation, Tree, tree},
};
use iced::widget::{self, text};
use iced::{
    Border, Color, Event, Font, Length, Pixels, Point, Rectangle, Renderer, Size, mouse, window,
};
use std::cell::{Cell, RefCell};

use crate::ripple::PressRipple;

/// A controlled checkbox. Omitting `on_toggle` makes it disabled.
/// The label and 48px target both activate on a matching pointer press/release.
pub struct Checkbox<'a, Message> {
    checked: bool,
    radio: bool,
    switch: bool,
    switch_icons: bool,
    indeterminate: bool,
    error: bool,
    label: Option<text::Fragment<'a>>,
    on_toggle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
    width: Length,
    size: f32,
    spacing: f32,
    font: Font,
    text_size: Pixels,
    line_height: text::LineHeight,
    shaping: text::Shaping,
    wrapping: text::Wrapping,
}

pub fn checkbox<'a, Message>(checked: bool) -> Checkbox<'a, Message> {
    fonts::ensure_loaded();
    Checkbox {
        checked,
        radio: false,
        switch: false,
        switch_icons: false,
        indeterminate: false,
        error: false,
        label: None,
        on_toggle: None,
        width: Length::Shrink,
        size: tokens::size::CHECKBOX,
        spacing: tokens::spacing::XS,
        font: fonts::REGULAR,
        text_size: 16.0.into(),
        line_height: text::LineHeight::Absolute(24.0.into()),
        shaping: text::Shaping::Advanced,
        wrapping: text::Wrapping::WordOrGlyph,
    }
}

impl<'a, Message> Checkbox<'a, Message> {
    pub(crate) fn switch_icons(mut self, icons: bool) -> Self {
        self.switch_icons = icons;
        self
    }
    pub(crate) fn into_switch(mut self) -> Self {
        self.switch = true;
        self.size = tokens::size::SWITCH_WIDTH;
        self.spacing = 8.0;
        self
    }
    pub(crate) fn into_radio(mut self) -> Self {
        self.radio = true;
        self.size = tokens::size::RADIO;
        self
    }
    pub fn label(mut self, label: impl text::IntoFragment<'a>) -> Self {
        self.label = Some(label.into_fragment());
        self
    }
    pub fn on_toggle(mut self, handler: impl Fn(bool) -> Message + 'a) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }
    pub fn on_toggle_maybe(mut self, handler: Option<impl Fn(bool) -> Message + 'a>) -> Self {
        self.on_toggle = handler.map(|f| Box::new(f) as _);
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        if disabled {
            self.on_toggle = None;
        }
        self
    }
    /// Display a mixed selection. The application still owns the Boolean value.
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }
    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
    /// Change the visible box size, keeping at least the standard target size.
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = size.into().0.max(1.0);
        self
    }
    /// Gap between the padded target and its label.
    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.spacing = spacing.into().0.max(0.0);
        self
    }
    pub fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }
    pub fn text_size(mut self, size: impl Into<Pixels>) -> Self {
        self.text_size = size.into();
        self
    }
    pub fn text_line_height(mut self, height: impl Into<text::LineHeight>) -> Self {
        self.line_height = height.into();
        self
    }
    pub fn text_shaping(mut self, shaping: text::Shaping) -> Self {
        self.shaping = shaping;
        self
    }
    pub fn text_wrapping(mut self, wrapping: text::Wrapping) -> Self {
        self.wrapping = wrapping;
        self
    }
}

impl<'a, Message: 'a> From<Checkbox<'a, Message>> for Element<'a, Message> {
    fn from(checkbox: Checkbox<'a, Message>) -> Self {
        let enabled = checkbox.on_toggle.is_some();
        let has_label = checkbox.label.is_some();
        let label = widget::text(checkbox.label.unwrap_or_else(|| "".into()))
            .font(checkbox.font)
            .size(checkbox.text_size)
            .line_height(checkbox.line_height)
            .shaping(checkbox.shaping)
            .wrapping(checkbox.wrapping)
            .style(move |theme: &Theme| text::Style {
                color: Some(if enabled {
                    theme.colors.on_surface
                } else {
                    alpha(theme.colors.on_surface, 0.38)
                }),
            });
        Element::new(Control {
            checked: checkbox.checked,
            radio: checkbox.radio,
            switch: checkbox.switch,
            switch_icons: checkbox.switch_icons,
            indeterminate: checkbox.indeterminate,
            error: checkbox.error,
            label: label.into(),
            has_label,
            on_toggle: checkbox.on_toggle,
            width: checkbox.width,
            size: checkbox.size,
            spacing: checkbox.spacing,
        })
    }
}

struct Control<'a, Message> {
    checked: bool,
    radio: bool,
    switch: bool,
    switch_icons: bool,
    indeterminate: bool,
    error: bool,
    label: Element<'a, Message>,
    has_label: bool,
    on_toggle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
    width: Length,
    size: f32,
    spacing: f32,
}

struct State {
    focus: crate::focus::Focus,
    keyboard_pressed: bool,
    selection: Transition,
    mixed: Transition,
    opacity: Transition,
    mark_growth: Transition,
    selected: bool,
    layer: Transition,
    ripple: PressRipple,
    pressure: Transition,
    pressed: bool,
    hovered: bool,
    enabled: bool,
    window_active: bool,
    motion: Cell<tokens::Motion>,
    mark: RefCell<Option<((f32, f32), Handle)>>,
}

impl<Message> Widget<Message, Theme, Renderer> for Control<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State {
            focus: crate::focus::Focus::default(),
            keyboard_pressed: false,
            selection: Transition::new(f32::from(self.checked || self.indeterminate)),
            mixed: Transition::new(f32::from(self.indeterminate)),
            opacity: Transition::linear(f32::from(self.checked || self.indeterminate)),
            mark_growth: Transition::new(1.0),
            selected: self.checked || self.indeterminate,
            layer: Transition::new(0.0),
            ripple: PressRipple::new(),
            pressure: Transition::new(0.0),
            pressed: false,
            hovered: false,
            enabled: self.on_toggle.is_some(),
            window_active: true,
            motion: Cell::new(tokens::Motion::default()),
            mark: RefCell::new(None),
        })
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.label)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.children[0].diff(&self.label);
        let state = tree.state.downcast_mut::<State>();
        let enabled = self.on_toggle.is_some();
        if !enabled || enabled != state.enabled {
            state.pressed = false;
            state.focus = crate::focus::Focus::default();
            state.hovered = false;
            state.layer = Transition::new(0.0);
            state.ripple = PressRipple::new();
            state.pressure = Transition::new(0.0);
            state.selection = Transition::new(f32::from(self.checked || self.indeterminate));
            state.mixed = Transition::new(f32::from(self.indeterminate));
            state.opacity = Transition::linear(f32::from(self.checked || self.indeterminate));
            state.mark_growth = Transition::new(1.0);
            state.selected = self.checked || self.indeterminate;
        }
        state.enabled = enabled;
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
        let target = (if self.switch {
            68.0
        } else {
            self.size + if self.radio { 28.0 } else { 30.0 }
        })
        .max(tokens::size::CHECKBOX_TARGET)
        .min(limits.max().width);
        let target_height = if self.switch { 48.0 } else { target };
        let gap = if self.has_label { self.spacing } else { 0.0 };
        let label = if self.has_label {
            self.label.as_widget_mut().layout(
                &mut tree.children[0],
                renderer,
                &limits.shrink(Size::new(target + gap, 0.0)).loose(),
            )
        } else {
            layout::Node::new(Size::ZERO)
        };
        let content = Size::new(
            target + gap + label.size().width,
            target_height.max(label.size().height),
        );
        let size = limits.resolve(self.width, Length::Shrink, content);
        let label_height = label.size().height;
        layout::Node::with_children(
            size,
            vec![
                layout::Node::new(Size::new(target, target_height))
                    .move_to(Point::new(0.0, (size.height - target_height) / 2.0)),
                label.move_to(Point::new(target + gap, (size.height - label_height) / 2.0)),
            ],
        )
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        if self.on_toggle.is_some() {
            let state = tree.state.downcast_mut::<State>();
            let id = state.focus.id.clone();
            operation.focusable(Some(&id), layout.bounds(), &mut state.focus);
        }
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.label.as_widget_mut().operate(
                &mut tree.children[0],
                layout.child(1),
                renderer,
                operation,
            )
        });
    }
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _: &Renderer,
        _: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let Some(on_toggle) = &self.on_toggle else {
            return;
        };
        let state = tree.state.downcast_mut::<State>();
        if state.keyboard_pressed && state.focus.key.is_none() {
            state.pressed = false;
            state.keyboard_pressed = false;
        }
        let now = match event {
            Event::Window(window::Event::RedrawRequested(now)) => *now,
            _ => crate::motion::now(),
        };
        let selected = self.checked || self.indeterminate;
        let duration = if self.radio || self.switch {
            state.motion.get().medium
        } else if selected {
            state.motion.get().checkbox_select
        } else {
            state.motion.get().short
        };
        let curve = if selected {
            [0.05, 0.7, 0.1, 1.0]
        } else {
            [0.3, 0.0, 0.8, 0.15]
        };
        let mut changed = if self.radio || self.switch {
            state.selection.set(f32::from(selected), now, duration)
        } else {
            state
                .selection
                .set_eased(f32::from(selected), now, duration, curve)
        };
        if !self.radio && !self.switch {
            changed |= state
                .opacity
                .set(f32::from(selected), now, state.motion.get().short / 3);
            if selected && !state.selected && !self.indeterminate {
                if state.opacity.value == 0.0 {
                    state.mark_growth = Transition::new(0.0);
                }
                changed |= state.mark_growth.set_eased(1.0, now, duration, curve);
            }
            // Changing the preference also settles an existing check stroke.
            if duration.is_zero() {
                changed |= state.mark_growth.set(1.0, now, duration);
            }
        }
        state.selected = selected;
        // Keep the outgoing mark while fading to unchecked.
        if self.checked || self.indeterminate {
            changed |= state.mixed.set_eased(
                f32::from(self.indeterminate),
                now,
                state.motion.get().checkbox_select,
                [0.05, 0.7, 0.1, 1.0],
            );
        }
        match event {
            Event::Window(window::Event::Unfocused) => {
                state.focus.key = None;
                state.window_active = false;
                state.pressed = false;
            }
            Event::Window(window::Event::Focused) => state.window_active = true,
            Event::Mouse(mouse::Event::CursorLeft) => state.pressed = false,
            // A modal host may send a synthetic Unfocused event to cancel a
            // gesture. New pointer input must reactivate this control afterward.
            Event::Mouse(_) => state.window_active = true,
            _ => {}
        }
        let over = state.window_active
            && cursor.is_over(layout.bounds())
            && cursor.is_over(*viewport)
            && !matches!(event, Event::Mouse(mouse::Event::CursorLeft));
        changed |= state.hovered != over;
        state.hovered = over;
        use iced::keyboard::{self, key::Named};
        match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key),
                repeat,
                ..
            }) if state.focus.focused && matches!(key, Named::Enter | Named::Space) => {
                if !repeat {
                    state.keyboard_pressed = true;
                    state.focus.key = Some(*key);
                    state.pressed = true;
                    state.ripple.start(now, state.motion.get());
                }
                shell.capture_event();
                changed = true;
            }
            Event::Keyboard(keyboard::Event::KeyReleased {
                key: keyboard::Key::Named(key),
                ..
            }) if state.focus.key == Some(*key) => {
                state.focus.key = None;
                state.keyboard_pressed = false;
                state.pressed = false;
                if state.focus.focused && !(self.radio && self.checked) {
                    shell.publish(on_toggle(!self.checked));
                }
                shell.capture_event();
                changed = true;
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if over => {
                state.focus.focused = true;
                state.focus.visible = false;
                state.pressed = true;
                if self.switch {
                    state.layer = Transition::new(0.12);
                } else {
                    state.ripple.start(now, state.motion.get());
                }
                changed = true;
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.pressed && !state.keyboard_pressed =>
            {
                state.pressed = false;
                if over && !(self.radio && self.checked) {
                    shell.publish(on_toggle(!self.checked));
                }
                changed = true;
                shell.capture_event();
            }
            _ => {}
        }
        let pressing = state.pressed && (over || state.focus.key.is_some());
        changed |= state.pressure.set(
            f32::from(self.switch && pressing),
            now,
            state.motion.get().short,
        );
        if !pressing {
            state.ripple.release(now, state.motion.get(), !over);
        }
        changed |= state.layer.set(
            if self.switch && pressing {
                0.12
            } else if over {
                0.08
            } else {
                0.0
            },
            now,
            state.motion.get().short,
        );
        if let Event::Window(window::Event::RedrawRequested(_)) = event {
            // Evaluate every transition; short-circuiting would strand one.
            let active = state.selection.tick(now)
                | state.mixed.tick(now)
                | state.opacity.tick(now)
                | state.mark_growth.tick(now)
                | state.layer.tick(now)
                | state.ripple.tick(now, state.motion.get())
                | state.pressure.tick(now);
            if active {
                shell.request_redraw();
            }
        }
        if changed {
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
        let Some(clip) = layout.bounds().intersection(viewport) else {
            return;
        };
        let state = tree.state.downcast_ref::<State>();
        state.motion.set(theme.motion);
        let enabled = self.on_toggle.is_some();
        let center = layout.child(0).bounds().center();
        let centered = |diameter| {
            Rectangle::new(
                Point::new(center.x - diameter / 2.0, center.y - diameter / 2.0),
                Size::new(diameter, diameter),
            )
        };
        let c = theme.colors;
        let selected = if self.error { c.error } else { c.primary };
        let outline = if !enabled {
            alpha(c.on_surface, 0.38)
        } else if self.error {
            c.error
        } else if state.hovered || state.pressed || (state.focus.focused && state.focus.visible) {
            c.on_surface
        } else {
            c.on_surface_variant
        };
        let layer_color = if self.error {
            c.error
        } else if self.checked || self.indeterminate {
            c.primary
        } else {
            c.on_surface
        };
        let layer_opacity = if state.focus.focused && state.focus.visible {
            state.layer.value.max(0.12)
        } else {
            state.layer.value
        };
        let press_color = if self.error {
            c.error
        } else if self.checked || self.indeterminate {
            c.on_surface
        } else {
            c.primary
        };
        let amount = state.selection.value;
        renderer.with_layer(clip, |renderer| {
            self.label.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout.child(1),
                cursor,
                &clip,
            );
            if enabled && state.focus.focused && state.focus.visible {
                renderer.fill_quad(renderer::Quad {bounds:layout.child(0).bounds().shrink(2.0),border:Border {color:c.primary,width:2.0,radius:tokens::shape::FULL.into()},..Default::default()},iced::Color::TRANSPARENT);
            }
            if self.switch {
                draw_switch(renderer, theme, state, center, enabled, self.checked,self.switch_icons);
                return;
            }
            if enabled {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: centered(tokens::size::CHECKBOX_STATE_LAYER),
                        border: Border {
                            radius: tokens::shape::FULL.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    alpha(layer_color, layer_opacity),
                );
                if state.ripple.opacity.value > 0.0 {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: centered(
                                tokens::size::CHECKBOX_STATE_LAYER
                                    * (0.35 + 0.65 * state.ripple.expansion.value),
                            ),
                            border: Border {
                                radius: tokens::shape::FULL.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        // Press feedback overlays the persistent hover circle,
                        // matching Material Web's separate hover/ripple layers.
                        alpha(press_color, 0.12 * state.ripple.opacity.value),
                    );
                }
            }
            if self.radio {
                let ink = if !enabled { alpha(c.on_surface, 0.38) } else {
                    crate::theme::mix(outline, selected, amount)
                };
                renderer.fill_quad(renderer::Quad {
                    bounds: centered(self.size),
                    border: Border { radius: tokens::shape::FULL.into(), width: 2.0, color: ink },
                    ..Default::default()
                }, Color::TRANSPARENT);
                if amount > 0.0 {
                    renderer.fill_quad(renderer::Quad {
                        bounds: centered(self.size * 0.5 * amount),
                        border: Border { radius: tokens::shape::FULL.into(), ..Default::default() },
                        ..Default::default()
                    }, ink);
                }
            } else {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: centered(self.size),
                    border: Border {
                        radius: 2.0.into(),
                        width: 2.0,
                        color: alpha(outline, outline.a * (1.0 - state.opacity.value)),
                    },
                    ..Default::default()
                },
                Color::TRANSPARENT,
            );
            if amount > 0.0 {
                let box_bounds = centered(self.size * (0.6 + 0.4 * amount));
                let opacity = state.opacity.value;
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: box_bounds,
                        border: Border {
                            radius: 2.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    if enabled {
                        alpha(selected, opacity)
                    } else {
                        alpha(c.on_surface, 0.38)
                    },
                );
                let mark_color = if !enabled {
                    c.surface
                } else if self.error {
                    c.on_error
                } else {
                    c.on_primary
                };
                let key = (state.mark_growth.value, state.mixed.value);
                let mut mark = state.mark.borrow_mut();
                if mark.as_ref().is_none_or(|(previous, _)| *previous != key) {
                    let m = state.mixed.value;
                    // Material's two perpendicular bars share their bottom-left
                    // vertex. Only the long bar grows on entry; mixed changes
                    // rotate and shorten both bars without an intermediate swap.
                    let angle = std::f32::consts::FRAC_PI_4 * (1.0-m);
                    let (sin, cos) = angle.sin_cos();
                    let x = 7.0 - 3.0*m;
                    let y = 14.0 - 4.0*m;
                    let long = (128.0_f32.sqrt()*(1.0-m)+10.0*m) * (state.mark_growth.value*(1.0-m)+m);
                    let short = 32.0_f32.sqrt()*(1.0-m)+2.0*m;
                    let svg = format!(
                        r#"<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 18 18"><g transform="matrix({cos} {} {} {} {x} {y})" fill="black"><rect width="{long}" height="2"/><rect width="2" height="{short}"/></g></svg>"#,
                        -sin, -sin, -cos
                    );
                    *mark = Some((key, Handle::from_memory(svg.into_bytes())));
                }
                // iced's SVG path handles ancestor scroll transforms correctly
                // on both backends. Tiny Skia 0.14 canvas geometry applies its
                // group clipping transform twice at nonzero scroll offsets.
                renderer.draw_svg(
                    Svg::from(&mark.as_ref().unwrap().1).color(mark_color).opacity(opacity),
                    box_bounds,
                    clip,
                );
            }
            }
        });
    }
    fn mouse_interaction(
        &self,
        _: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        _: &Renderer,
    ) -> mouse::Interaction {
        if self.on_toggle.is_some() && cursor.is_over(layout.bounds()) && cursor.is_over(*viewport)
        {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
        }
    }
}

fn draw_switch(
    renderer: &mut Renderer,
    theme: &Theme,
    state: &State,
    center: Point,
    enabled: bool,
    checked: bool,
    icons: bool,
) {
    use iced::advanced::Renderer as _;
    let c = theme.colors;
    let amount = state.selection.value;
    let track = Rectangle::new(
        Point::new(center.x - 26.0, center.y - 16.0),
        Size::new(52.0, 32.0),
    );
    let thumb_center = Point::new(center.x - 10.0 + 20.0 * amount, center.y);
    let thumb_bounds = |size| {
        Rectangle::new(
            Point::new(thumb_center.x - size / 2.0, thumb_center.y - size / 2.0),
            Size::new(size, size),
        )
    };
    let track_color = if enabled {
        crate::theme::mix(c.surface_container_highest, c.primary, amount)
    } else {
        alpha(
            crate::theme::mix(c.surface_container_highest, c.on_surface, amount),
            0.12,
        )
    };
    renderer.fill_quad(
        renderer::Quad {
            bounds: track,
            border: Border {
                radius: tokens::shape::FULL.into(),
                width: 2.0 * (1.0 - amount),
                color: if enabled {
                    c.outline
                } else {
                    alpha(c.on_surface, 0.12)
                },
            },
            ..Default::default()
        },
        track_color,
    );
    if enabled {
        renderer.fill_quad(
            renderer::Quad {
                bounds: thumb_bounds(40.0),
                border: Border {
                    radius: tokens::shape::FULL.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            alpha(
                crate::theme::mix(c.on_surface, c.primary, amount),
                if state.focus.focused && state.focus.visible {
                    state.layer.value.max(0.12)
                } else {
                    state.layer.value
                },
            ),
        );
    }
    let idle_size = if icons { 24.0 } else { 16.0 + 8.0 * amount };
    let diameter = idle_size + (28.0 - idle_size) * state.pressure.value;
    let thumb_color = if enabled {
        let interaction = if state.focus.focused && state.focus.visible {
            1.0
        } else {
            (state.layer.value / 0.08).clamp(0.0, 1.0)
        };
        crate::theme::mix(
            crate::theme::mix(c.outline, c.on_surface_variant, interaction),
            crate::theme::mix(c.on_primary, c.primary_container, interaction),
            amount,
        )
    } else if checked {
        c.surface
    } else {
        alpha(c.on_surface, 0.38)
    };
    renderer.fill_quad(
        renderer::Quad {
            bounds: thumb_bounds(diameter),
            border: Border {
                radius: tokens::shape::FULL.into(),
                ..Default::default()
            },
            ..Default::default()
        },
        thumb_color,
    );
    if icons {
        use iced::advanced::svg::{Handle, Renderer as _, Svg};
        static MARKS: std::sync::OnceLock<[Handle; 2]> = std::sync::OnceLock::new();
        let handles=MARKS.get_or_init(||["M5 5 L19 19 M19 5 L5 19","M4 12 L9 17 L20 6"].map(|path|Handle::from_memory(format!("<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'><path d='{path}' fill='none' stroke='black' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'/></svg>").into_bytes())));
        for (index, opacity) in [(0, 1.0 - amount), (1, amount)] {
            renderer.draw_svg(
                Svg::from(&handles[index])
                    .color(if enabled {
                        crate::theme::mix(
                            c.surface_container_highest,
                            c.on_primary_container,
                            amount,
                        )
                    } else {
                        crate::theme::mix(c.surface_container_highest, c.on_surface, amount)
                    })
                    .opacity(opacity * if enabled { 1.0 } else { 0.38 }),
                thumb_bounds(16.0),
                track,
            );
        }
    }
}
