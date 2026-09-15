//! A controlled, single-thumb baseline Material slider.
use crate::{Element, Theme, TypeScale, motion::Transition, theme::alpha, tokens, typography};
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, renderer,
    widget::{Tree, tree},
};
use iced::{Border, Event, Length, Point, Rectangle, Renderer, Size, mouse, window};
use std::{cell::Cell, ops::RangeInclusive};

pub struct Slider<'a, Message> {
    range: RangeInclusive<f32>,
    value: f32,
    step: Option<f32>,
    ticks: bool,
    labeled: bool,
    label: Option<String>,
    on_change: Option<Box<dyn Fn(f32) -> Message + 'a>>,
    on_release: Option<Message>,
    width: Length,
}
/// Values are clamped to the range. Invalid or empty ranges are inert.
pub fn slider<'a, Message>(range: RangeInclusive<f32>, value: f32) -> Slider<'a, Message> {
    Slider {
        range,
        value,
        step: None,
        ticks: false,
        labeled: false,
        label: None,
        on_change: None,
        on_release: None,
        width: Length::Fill,
    }
}
impl<'a, Message> Slider<'a, Message> {
    pub fn on_change(mut self, handler: impl Fn(f32) -> Message + 'a) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }
    /// Sent once after a completed drag, after its last value message.
    pub fn on_release(mut self, message: Message) -> Self {
        self.on_release = Some(message);
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        if disabled {
            self.on_change = None;
        }
        self
    }
    /// Snap to steps measured from the minimum. Both endpoints remain reachable.
    pub fn step(mut self, step: f32) -> Self {
        self.step = (step.is_finite() && step > 0.0).then_some(step);
        self
    }
    pub fn ticks(mut self, ticks: bool) -> Self {
        self.ticks = ticks;
        self
    }
    /// Reserve space for a value bubble, shown on hover, focus or dragging (no layout jump).
    pub fn labeled(mut self, labeled: bool) -> Self {
        self.labeled = labeled;
        self
    }
    /// Override the bubble text, for example `"80%"`.
    pub fn value_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self.labeled = true;
        self
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
    pub(crate) fn valid(&self) -> bool {
        self.range.start().is_finite()
            && self.range.end().is_finite()
            && self.range.end() > self.range.start()
    }
    pub(crate) fn enabled(&self) -> bool {
        self.valid() && self.on_change.is_some()
    }
    pub(crate) fn keyboard_value(
        &self,
        value: f32,
        key: iced::keyboard::key::Named,
    ) -> Option<f32> {
        use iced::keyboard::key::Named;
        let span = *self.range.end() as f64 - *self.range.start() as f64;
        let step = self.step.map(f64::from).unwrap_or(span / 100.0);
        let value = self.value_at(self.fraction(value)) as f64;
        let next = match key {
            Named::ArrowLeft | Named::ArrowDown => value - step,
            Named::ArrowRight | Named::ArrowUp => value + step,
            Named::PageDown => value - 10.0 * step,
            Named::PageUp => value + 10.0 * step,
            Named::Home => *self.range.start() as f64,
            Named::End => *self.range.end() as f64,
            _ => return None,
        };
        Some(self.value_at(((next - *self.range.start() as f64) / span) as f32))
    }
    pub(crate) fn fraction(&self, value: f32) -> f32 {
        if !self.valid() || value.is_nan() {
            return 0.0;
        }
        (((value as f64 - *self.range.start() as f64)
            / (*self.range.end() as f64 - *self.range.start() as f64))
            .clamp(0.0, 1.0)) as f32
    }
    pub(crate) fn value_at(&self, fraction: f32) -> f32 {
        let min = *self.range.start() as f64;
        let max = *self.range.end() as f64;
        let fraction = fraction.clamp(0.0, 1.0) as f64;
        if fraction == 1.0 {
            return max as f32;
        }
        let mut value = min + (max - min) * fraction;
        if let Some(step) = self.step {
            value = min + ((value - min) / step as f64).round() * step as f64;
        }
        value.clamp(min, max) as f32
    }
    pub(crate) fn track(&self, bounds: Rectangle) -> Rectangle {
        let inset = (tokens::size::SLIDER_TARGET / 2.0).min(bounds.width / 2.0);
        Rectangle {
            x: bounds.x + inset,
            y: bounds.y + bounds.height - tokens::size::SLIDER_TARGET / 2.0,
            width: (bounds.width - 2.0 * inset).max(0.0),
            height: 0.0,
        }
    }
    pub(crate) fn hit_bounds(&self, bounds: Rectangle) -> Rectangle {
        Rectangle {
            y: bounds.y + (bounds.height - tokens::size::SLIDER_TARGET).max(0.0),
            height: bounds.height.min(tokens::size::SLIDER_TARGET),
            ..bounds
        }
    }
    fn bubble(&self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        let value = if self.valid() {
            self.value_at(self.fraction(self.value))
        } else {
            0.0
        };
        let label = self.label.clone().unwrap_or_else(|| {
            format!("{value:.2}")
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_owned()
        });
        typography(label, TypeScale::LabelMedium)
            .wrapping(iced::widget::text::Wrapping::None)
            .into()
    }
}
struct State {
    focus: crate::focus::Focus,
    dragging: bool,
    last_value: Option<f32>,
    hover: Transition,
    press: Transition,
    label: Transition,
    shadow: crate::elevation::Cache,
    motion: Cell<tokens::Motion>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            focus: crate::focus::Focus::default(),
            dragging: false,
            last_value: None,
            hover: Transition::new(0.0),
            press: Transition::new(0.0),
            label: Transition::standard(0.0),
            shadow: Default::default(),
            motion: Cell::new(tokens::Motion::default()),
        }
    }
}
// Shared label geometry keeps single and range sliders in sync. The pointed
// outline is the union of the reference capsule and its rotated 14px square.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_label<Message>(
    renderer: &mut Renderer,
    theme: &Theme,
    content: Element<'_, Message>,
    tree: &Tree,
    text: Size,
    bounds: Rectangle,
    x: f32,
    track_y: f32,
    scale: f32,
    cursor: mouse::Cursor,
    clip: Rectangle,
) {
    if scale <= 0.0 {
        return;
    }
    let frame = label_bounds(text, bounds, x, track_y);
    draw_value_label(
        renderer, theme, content, tree, text, frame, x, scale, cursor, clip,
    );
}
pub(crate) fn label_bounds(text: Size, bounds: Rectangle, x: f32, track_y: f32) -> Rectangle {
    let height = tokens::size::SLIDER_LABEL;
    let width = (text.width + 8.0).max(height).min(bounds.width);
    Rectangle {
        x: (x - width / 2.0).clamp(bounds.x, (bounds.x + bounds.width - width).max(bounds.x)),
        y: track_y - tokens::size::SLIDER_STATE_LAYER / 2.0 - height,
        width,
        height,
    }
}
pub(crate) fn draw_value_label<Message>(
    renderer: &mut Renderer,
    theme: &Theme,
    content: Element<'_, Message>,
    tree: &Tree,
    text: Size,
    frame: Rectangle,
    pointer: f32,
    scale: f32,
    cursor: mouse::Cursor,
    clip: Rectangle,
) {
    use iced::Transformation;
    use iced::advanced::{
        Renderer as _,
        svg::{Handle, Renderer as _, Svg},
    };
    if scale <= 0.0 {
        return;
    }
    let Rectangle {
        x: left,
        y: top,
        width,
        height,
    } = frame;
    let origin = Point::new(left + width / 2.0, top + height);
    let transform = Transformation::translate(origin.x, origin.y)
        * Transformation::scale(scale)
        * Transformation::translate(-origin.x, -origin.y);
    // The renderer caches handles by content; quantization is unnecessary as
    // neither text nor geometry changes during the scale transition.
    let r = height / 2.0;
    let center = (pointer - left).clamp(width.min(20.0) / 2.0, width - width.min(20.0) / 2.0);
    let svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' width='{width}' height='34'><path d='M {center} 13.9005 L {} 23.8 L {center} 33.6995 L {} 23.8 Z'/><rect width='{width}' height='{height}' rx='{r}'/></svg>",
        center + 9.8995,
        center - 9.8995
    );
    renderer.with_transformation(transform, |renderer| {
        renderer.draw_svg(
            Svg {
                handle: Handle::from_memory(svg.into_bytes()),
                color: Some(theme.colors.primary),
                rotation: iced::Radians(0.0),
                opacity: 1.0,
            },
            Rectangle::new(Point::new(left, top), Size::new(width, 34.0)),
            clip,
        );
        let text_clip = Rectangle::new(
            Point::new(left + 4.0, top),
            Size::new((width - 8.0).max(0.0), height),
        );
        if let Some(text_clip) = text_clip.intersection(&clip) {
            content.as_widget().draw(
                tree,
                renderer,
                theme,
                &renderer::Style {
                    text_color: theme.colors.on_primary,
                },
                Layout::with_offset(
                    iced::Vector::new(
                        left + (width - text.width) / 2.0,
                        top + (height - text.height) / 2.0,
                    ),
                    &layout::Node::new(text),
                ),
                cursor,
                &text_clip,
            );
        }
    });
}
impl<'a, Message: Clone + 'a> Widget<Message, Theme, Renderer> for Slider<'a, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(self.bubble())]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.bubble()]);
        if !self.enabled() {
            *tree.state.downcast_mut::<State>() = State::default();
        }
    }
    fn size(&self) -> Size<Length> {
        Size::new(
            self.width,
            Length::Fixed(
                tokens::size::SLIDER_TARGET
                    + if self.labeled {
                        tokens::size::SLIDER_LABEL_SPACE
                    } else {
                        0.0
                    },
            ),
        )
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits.resolve(
            self.width,
            self.size().height,
            Size::new(160.0, tokens::size::SLIDER_TARGET),
        );
        let bubble = self.bubble().as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &layout::Limits::new(
                Size::ZERO,
                Size::new((size.width - 16.0).max(0.0), tokens::size::SLIDER_LABEL),
            ),
        );
        layout::Node::with_children(size, vec![bubble])
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _: &Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        if self.enabled() {
            let state = tree.state.downcast_mut::<State>();
            let id = state.focus.id.clone();
            operation.focusable(Some(&id), layout.bounds(), &mut state.focus);
        }
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
        if !self.enabled() {
            return;
        }
        let state = tree.state.downcast_mut::<State>();
        if state.focus.focused
            && let Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key: iced::keyboard::Key::Named(key),
                ..
            }) = event
            && let Some(next) = self.keyboard_value(self.value, *key)
        {
            if next != self.value_at(self.fraction(self.value)) {
                shell.publish(self.on_change.as_ref().unwrap()(next));
            }
            state.focus.key = Some(*key);
            shell.capture_event();
            shell.request_redraw();
            return;
        }
        if let Event::Keyboard(iced::keyboard::Event::KeyReleased {
            key: iced::keyboard::Key::Named(key),
            ..
        }) = event
            && state.focus.key == Some(*key)
        {
            state.focus.key = None;
            if state.focus.focused
                && let Some(message) = &self.on_release
            {
                shell.publish(message.clone());
            }
            shell.capture_event();
            return;
        }
        let over = cursor.is_over(self.hit_bounds(layout.bounds())) && cursor.is_over(*viewport);
        let now = match event {
            Event::Window(window::Event::RedrawRequested(now)) => *now,
            _ => crate::motion::now(),
        };
        let mut change = false;
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if over => {
                state.focus.focused = true;
                state.focus.visible = false;
                state.dragging = true;
                state.last_value = Some(self.value_at(self.fraction(self.value)));
                change = true;
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) if state.dragging => {
                change = true;
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if state.dragging => {
                change = true;
                state.dragging = false;
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Window(window::Event::Unfocused) | Event::Mouse(mouse::Event::CursorLeft) => {
                state.dragging = false;
                state.last_value = None;
                shell.request_redraw();
            }
            _ => {}
        }
        if change && let Some(point) = cursor.position() {
            let track = self.track(layout.bounds());
            if track.width > 0.0 {
                let value = self.value_at((point.x - track.x) / track.width);
                if state.last_value != Some(value) {
                    shell.publish(self.on_change.as_ref().unwrap()(value));
                    state.last_value = Some(value);
                }
            }
        }
        if change
            && !state.dragging
            && let Some(message) = &self.on_release
        {
            shell.publish(message.clone());
        }
        let hovered = over
            && !matches!(
                event,
                Event::Window(window::Event::Unfocused) | Event::Mouse(mouse::Event::CursorLeft)
            );
        let changed = state.hover.set(
            if hovered { 0.08 } else { 0.0 },
            now,
            state.motion.get().short,
        ) | state.press.set(
            if state.dragging { 1.0 } else { 0.0 },
            now,
            state.motion.get().short,
        );
        let label_changed = state.label.set(
            f32::from(self.labeled && (hovered || state.dragging || state.focus.focused)),
            now,
            state.motion.get().slider_label,
        );
        let active = state.hover.tick(now) | state.press.tick(now) | state.label.tick(now);
        let changed = changed | label_changed;
        if changed || active {
            shell.request_redraw();
        }
    }
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _: &renderer::Style,
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
        let enabled = self.enabled();
        let track = self.track(bounds);
        let fraction = self.fraction(if self.valid() {
            self.value_at(self.fraction(self.value))
        } else {
            0.0
        });
        let x = track.x + track.width * fraction;
        let thumb = tokens::size::SLIDER_THUMB / 2.0;
        let rail = tokens::size::SLIDER_TRACK / 2.0;
        let halo_radius = tokens::size::SLIDER_STATE_LAYER / 2.0;
        let active = if enabled {
            c.primary
        } else {
            alpha(c.on_surface, 0.38)
        };
        let inactive = if enabled {
            c.surface_container_highest
        } else {
            alpha(c.on_surface, 0.12)
        };
        renderer.with_layer(clip, |renderer| {
            if enabled && state.focus.focused && state.focus.visible {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle::new(
                            Point::new(x - 22.0, track.y - 22.0),
                            Size::new(44.0, 44.0),
                        ),
                        border: Border {
                            color: c.primary,
                            width: 2.0,
                            radius: 22.0.into(),
                        },
                        ..Default::default()
                    },
                    iced::Color::TRANSPARENT,
                );
            }
            if enabled {
                crate::elevation::draw(
                    renderer,
                    theme,
                    Rectangle::new(
                        Point::new(x - thumb, track.y - thumb),
                        Size::new(thumb * 2.0, thumb * 2.0),
                    ),
                    thumb.into(),
                    1.0,
                    clip,
                    &state.shadow,
                );
            }
            let mut quad = |area: Rectangle, radius: f32, color| {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: area,
                        border: Border {
                            radius: radius.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    color,
                )
            };
            // Split the tracks around the thumb so translucent disabled colors do not stack.
            quad(
                Rectangle {
                    x: track.x,
                    y: track.y - rail,
                    width: (x - track.x - thumb).max(0.0),
                    height: tokens::size::SLIDER_TRACK,
                },
                rail,
                active,
            );
            quad(
                Rectangle {
                    x: x + thumb,
                    y: track.y - rail,
                    width: (track.x + track.width - x - thumb).max(0.0),
                    height: tokens::size::SLIDER_TRACK,
                },
                rail,
                inactive,
            );
            if self.ticks
                && self.valid()
                && let Some(step) = self.step
            {
                let count =
                    ((*self.range.end() as f64 - *self.range.start() as f64) / step as f64).floor();
                // Dense steps stay interactive; omit ticks that cannot be visually separated.
                if count <= (track.width / 4.0) as f64 {
                    for index in 0..=count as u32 {
                        let t = self.fraction(
                            (*self.range.start() as f64 + index as f64 * step as f64) as f32,
                        );
                        let tx = track.x + t * track.width;
                        if (tx - x).abs() > thumb + 1.0 {
                            quad(
                                Rectangle::new(
                                    Point::new(tx - 1.0, track.y - 1.0),
                                    Size::new(2.0, 2.0),
                                ),
                                1.0,
                                if !enabled {
                                    alpha(c.on_surface, 0.38)
                                } else if t < fraction {
                                    c.on_primary
                                } else {
                                    c.on_surface_variant
                                },
                            );
                        }
                    }
                }
            }
            let halo = Rectangle::new(
                Point::new(x - halo_radius, track.y - halo_radius),
                Size::new(
                    tokens::size::SLIDER_STATE_LAYER,
                    tokens::size::SLIDER_STATE_LAYER,
                ),
            );
            if enabled {
                quad(
                    halo,
                    halo_radius,
                    alpha(
                        c.primary,
                        if state.focus.focused && state.focus.visible {
                            0.12
                        } else {
                            state.hover.value
                        },
                    ),
                );
                quad(
                    halo,
                    halo_radius,
                    alpha(c.primary, state.press.value * 0.12),
                );
            }
            quad(
                Rectangle::new(
                    Point::new(x - thumb, track.y - thumb),
                    Size::new(tokens::size::SLIDER_THUMB, tokens::size::SLIDER_THUMB),
                ),
                thumb,
                active,
            );
            if self.labeled && enabled {
                draw_label(
                    renderer,
                    theme,
                    self.bubble(),
                    &tree.children[0],
                    layout.child(0).bounds().size(),
                    bounds,
                    x,
                    track.y,
                    state.label.value,
                    cursor,
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
        _: &Renderer,
    ) -> mouse::Interaction {
        if self.enabled()
            && (tree.state.downcast_ref::<State>().dragging
                || (cursor.is_over(self.hit_bounds(layout.bounds())) && cursor.is_over(*viewport)))
        {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
        }
    }
}
impl<'a, Message: Clone + 'a> From<Slider<'a, Message>> for Element<'a, Message> {
    fn from(slider: Slider<'a, Message>) -> Self {
        Self::new(slider)
    }
}
