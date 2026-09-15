//! A two-thumb slider using the baseline slider's geometry and value scale.
use crate::{
    Element, Slider, Theme, TypeScale, motion::Transition, slider, theme::alpha, tokens, typography,
};
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, renderer,
    widget::{Tree, tree},
};
use iced::{Border, Event, Length, Point, Rectangle, Renderer, Size, mouse, window};
use std::{cell::Cell, ops::RangeInclusive};

pub struct RangeSlider<'a, Message> {
    scale: Slider<'a, Message>,
    range: RangeInclusive<f32>,
    values: (f32, f32),
    step: Option<f32>,
    ticks: bool,
    labeled: bool,
    labels: Option<(String, String)>,
    on_change: Option<Box<dyn Fn((f32, f32)) -> Message + 'a>>,
    on_release: Option<Message>,
    width: Length,
}
/// Endpoints are ordered, clamped and snapped. Invalid domains are inert;
/// NaN values fall back to the minimum. Handles meet but never cross.
pub fn range_slider<'a, Message>(
    range: RangeInclusive<f32>,
    values: (f32, f32),
) -> RangeSlider<'a, Message> {
    RangeSlider {
        scale: slider(range.clone(), values.0),
        range,
        values,
        step: None,
        ticks: false,
        labeled: false,
        labels: None,
        on_change: None,
        on_release: None,
        width: Length::Fill,
    }
}
impl<'a, Message> RangeSlider<'a, Message> {
    pub fn on_change(mut self, callback: impl Fn((f32, f32)) -> Message + 'a) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }
    /// Emitted once on a completed drag, after the final value change.
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
    pub fn step(mut self, step: f32) -> Self {
        self.step = (step.is_finite() && step > 0.0).then_some(step);
        self.scale = self.scale.step(step);
        self
    }
    pub fn ticks(mut self, ticks: bool) -> Self {
        self.ticks = ticks;
        self
    }
    /// Reserve label space. Hover shows the nearest value; focus/drag shows both.
    pub fn labeled(mut self, labeled: bool) -> Self {
        self.labeled = labeled;
        self
    }
    pub fn value_labels(mut self, lower: impl Into<String>, upper: impl Into<String>) -> Self {
        self.labels = Some((lower.into(), upper.into()));
        self.labeled = true;
        self
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
    fn enabled(&self) -> bool {
        self.scale.valid() && self.on_change.is_some()
    }
    fn values(&self) -> (f32, f32) {
        if !self.scale.valid() {
            return (0.0, 0.0);
        }
        let a = self.scale.value_at(self.scale.fraction(self.values.0));
        let b = self.scale.value_at(self.scale.fraction(self.values.1));
        (a.min(b), a.max(b))
    }
    fn bubble(&self, index: usize) -> Element<'a, Message>
    where
        Message: 'a,
    {
        let pair = self.values();
        let label = if let Some(labels) = &self.labels {
            if index == 0 {
                labels.0.clone()
            } else {
                labels.1.clone()
            }
        } else {
            format!("{:.2}", if index == 0 { pair.0 } else { pair.1 })
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_owned()
        };
        typography(label, TypeScale::LabelMedium)
            .wrapping(iced::widget::text::Wrapping::None)
            .into()
    }
}
struct State {
    focus: [crate::focus::Focus; 2],
    dragging: bool,
    active: Option<usize>,
    last_active: usize,
    last_value: Option<(f32, f32)>,
    hover: [Transition; 2],
    press: [Transition; 2],
    label: [Transition; 2],
    shadow: [crate::elevation::Cache; 2],
    motion: Cell<tokens::Motion>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            focus: std::array::from_fn(|_| crate::focus::Focus::default()),
            dragging: false,
            active: None,
            last_active: 1,
            last_value: None,
            hover: [Transition::new(0.0); 2],
            press: [Transition::new(0.0); 2],
            label: [Transition::standard(0.0); 2],
            shadow: Default::default(),
            motion: Cell::new(tokens::Motion::default()),
        }
    }
}
impl<'a, Message: Clone + 'a> Widget<Message, Theme, Renderer> for RangeSlider<'a, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(self.bubble(0)), Tree::new(self.bubble(1))]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.bubble(0), self.bubble(1)]);
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
        let children = (0..2)
            .map(|i| {
                self.bubble(i).as_widget_mut().layout(
                    &mut tree.children[i],
                    renderer,
                    &layout::Limits::new(
                        Size::ZERO,
                        Size::new((size.width - 16.0).max(0.0), tokens::size::SLIDER_LABEL),
                    ),
                )
            })
            .collect();
        layout::Node::with_children(size, children)
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
            for focus in &mut state.focus {
                let id = focus.id.clone();
                operation.focusable(Some(&id), layout.bounds(), focus);
            }
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
        let track = self.scale.track(layout.bounds());
        let over =
            cursor.is_over(self.scale.hit_bounds(layout.bounds())) && cursor.is_over(*viewport);
        let pair = self.values();
        if let Some(index) = state.focus.iter().position(|f| f.focused)
            && let Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key: iced::keyboard::Key::Named(key),
                ..
            }) = event
            && let Some(next) = self
                .scale
                .keyboard_value(if index == 0 { pair.0 } else { pair.1 }, *key)
        {
            let values = if index == 0 {
                (next.min(pair.1), pair.1)
            } else {
                (pair.0, next.max(pair.0))
            };
            if values != pair {
                shell.publish(self.on_change.as_ref().unwrap()(values));
            }
            state.focus[index].key = Some(*key);
            state.last_active = index;
            shell.capture_event();
            shell.request_redraw();
            return;
        }
        if let Event::Keyboard(iced::keyboard::Event::KeyReleased {
            key: iced::keyboard::Key::Named(key),
            ..
        }) = event
        {
            for focus in &mut state.focus {
                if focus.key == Some(*key) {
                    focus.key = None;
                    if focus.focused
                        && let Some(message) = &self.on_release
                    {
                        shell.publish(message.clone());
                    }
                    shell.capture_event();
                    return;
                }
            }
        }
        let xs = [
            track.x + track.width * self.scale.fraction(pair.0),
            track.x + track.width * self.scale.fraction(pair.1),
        ];
        let nearest = cursor.position().map(|p| {
            if (p.x - xs[0]).abs() < (p.x - xs[1]).abs() {
                0
            } else {
                1
            }
        });
        let mut change = false;
        let mut release = false;
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if over && track.width > 0.0 =>
            {
                state.dragging = true;
                state.last_value = Some(pair);
                state.active = if xs[0] == xs[1]
                    && cursor.position().is_some_and(|p| (p.x - xs[0]).abs() < 1.0)
                {
                    None
                } else {
                    nearest
                };
                for (i, focus) in state.focus.iter_mut().enumerate() {
                    focus.focused = state.active == Some(i);
                    focus.visible = false;
                }
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
                release = true;
                state.dragging = false;
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Window(window::Event::Unfocused) | Event::Mouse(mouse::Event::CursorLeft) => {
                state.dragging = false;
                state.active = None;
                state.last_value = None;
                shell.request_redraw();
            }
            _ => {}
        }
        if change && let Some(point) = cursor.position() {
            let value = self.scale.value_at((point.x - track.x) / track.width);
            if state.active.is_none() && value != pair.0 {
                state.active = Some(if value < pair.0 { 0 } else { 1 });
            }
            if let Some(index) = state.active {
                state.last_active = index;
                let values = if index == 0 {
                    (value.min(pair.1), pair.1)
                } else {
                    (pair.0, value.max(pair.0))
                };
                if state.last_value != Some(values) {
                    shell.publish(self.on_change.as_ref().unwrap()(values));
                    state.last_value = Some(values);
                }
            }
        }
        if release && let Some(message) = &self.on_release {
            shell.publish(message.clone());
        }
        let cancelled = matches!(
            event,
            Event::Window(window::Event::Unfocused) | Event::Mouse(mouse::Event::CursorLeft)
        );
        let now = match event {
            Event::Window(window::Event::RedrawRequested(now)) => *now,
            _ => crate::motion::now(),
        };
        let show_both = state.dragging || state.focus.iter().any(|focus| focus.focused);
        for i in 0..2 {
            let hovered = !cancelled && over && nearest == Some(i);
            let pressed = state.dragging && state.active.unwrap_or(state.last_active) == i;
            let changed = state.hover[i].set(
                if hovered { 0.08 } else { 0.0 },
                now,
                state.motion.get().short,
            ) | state.press[i].set(
                if pressed { 1.0 } else { 0.0 },
                now,
                state.motion.get().short,
            );
            let label_changed = state.label[i].set(
                f32::from(self.labeled && (hovered || show_both)),
                now,
                state.motion.get().slider_label,
            );
            if state.hover[i].tick(now)
                | state.press[i].tick(now)
                | state.label[i].tick(now)
                | changed
                | label_changed
            {
                shell.request_redraw();
            }
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
        let enabled = self.enabled();
        let c = theme.colors;
        let track = self.scale.track(bounds);
        let pair = self.values();
        let fractions = [self.scale.fraction(pair.0), self.scale.fraction(pair.1)];
        let xs = fractions.map(|t| track.x + track.width * t);
        let thumb = tokens::size::SLIDER_THUMB / 2.0;
        let rail = tokens::size::SLIDER_TRACK / 2.0;
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
            for (i, focus) in state.focus.iter().enumerate() {
                if enabled && focus.focused && focus.visible {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: Rectangle::new(
                                Point::new(xs[i] - 22.0, track.y - 22.0),
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
            }
            if enabled {
                for i in [1 - state.last_active, state.last_active] {
                    crate::elevation::draw(
                        renderer,
                        theme,
                        Rectangle::new(
                            Point::new(xs[i] - thumb, track.y - thumb),
                            Size::new(thumb * 2.0, thumb * 2.0),
                        ),
                        thumb.into(),
                        1.0,
                        clip,
                        &state.shadow[i],
                    );
                }
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
            for (left, right, color) in [
                (track.x, xs[0] - thumb, inactive),
                (xs[0] + thumb, xs[1] - thumb, active),
                (xs[1] + thumb, track.x + track.width, inactive),
            ] {
                quad(
                    Rectangle {
                        x: left,
                        y: track.y - rail,
                        width: (right - left).max(0.0),
                        height: tokens::size::SLIDER_TRACK,
                    },
                    rail,
                    color,
                );
            }
            if self.ticks
                && self.scale.valid()
                && let Some(step) = self.step
            {
                let count =
                    ((*self.range.end() as f64 - *self.range.start() as f64) / step as f64).floor();
                if count <= (track.width / 4.0) as f64 {
                    for i in 0..=count as u32 {
                        let fraction = self
                            .scale
                            .fraction((*self.range.start() as f64 + i as f64 * step as f64) as f32);
                        let x = track.x + track.width * fraction;
                        if xs.iter().all(|thumb_x| (x - thumb_x).abs() > thumb + 1.0) {
                            quad(
                                Rectangle::new(
                                    Point::new(x - 1.0, track.y - 1.0),
                                    Size::new(2.0, 2.0),
                                ),
                                1.0,
                                if !enabled {
                                    alpha(c.on_surface, 0.38)
                                } else if fraction > fractions[0] && fraction < fractions[1] {
                                    c.on_primary
                                } else {
                                    c.on_surface_variant
                                },
                            );
                        }
                    }
                }
            }
            for i in [1 - state.last_active, state.last_active] {
                let x = xs[i];
                let halo = tokens::size::SLIDER_STATE_LAYER / 2.0;
                if enabled {
                    let area = Rectangle::new(
                        Point::new(x - halo, track.y - halo),
                        Size::new(halo * 2.0, halo * 2.0),
                    );
                    quad(
                        area,
                        halo,
                        alpha(
                            c.primary,
                            if state.focus[i].focused && state.focus[i].visible {
                                0.12
                            } else {
                                state.hover[i].value
                            },
                        ),
                    );
                    quad(area, halo, alpha(c.primary, state.press[i].value * 0.12));
                }
                // Coincident disabled handles must not compound their translucent fills.
                if i == state.last_active || xs[0] != xs[1] {
                    quad(
                        Rectangle::new(
                            Point::new(x - thumb, track.y - thumb),
                            Size::new(thumb * 2.0, thumb * 2.0),
                        ),
                        thumb,
                        active,
                    );
                }
            }
            if enabled && (xs[1] - xs[0]).abs() < thumb * 2.0 {
                let x = xs[state.last_active];
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle::new(
                            Point::new(x - thumb, track.y - thumb),
                            Size::new(thumb * 2.0, thumb * 2.0),
                        ),
                        border: Border {
                            color: c.on_primary,
                            width: 1.0,
                            radius: thumb.into(),
                        },
                        ..Default::default()
                    },
                    iced::Color::TRANSPARENT,
                );
            }
            if self.labeled && enabled {
                let mut frames = std::array::from_fn(|i| {
                    crate::slider::label_bounds(
                        layout.child(i).bounds().size(),
                        bounds,
                        xs[i],
                        track.y,
                    )
                });
                if state.label.iter().all(|label| label.value > 0.0) {
                    separate_labels(&mut frames, bounds);
                }
                for i in [1 - state.last_active, state.last_active] {
                    crate::slider::draw_value_label(
                        renderer,
                        theme,
                        self.bubble(i),
                        &tree.children[i],
                        layout.child(i).bounds().size(),
                        frames[i],
                        xs[i],
                        state.label[i].value,
                        cursor,
                        clip,
                    );
                }
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
                || cursor.is_over(self.scale.hit_bounds(layout.bounds()))
                    && cursor.is_over(*viewport))
        {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
        }
    }
}
// Keep both values readable. Move the capsules outward while their pointers
// continue to identify the original handles; clip long labels in tiny hosts.
fn separate_labels(labels: &mut [Rectangle; 2], bounds: Rectangle) {
    let gap = 4.0_f32.min(bounds.width);
    if labels[0].width + labels[1].width + gap > bounds.width {
        let available = (bounds.width - gap).max(0.0);
        let first = labels[0].width.min(available / 2.0);
        labels[0].width = first;
        labels[1].width = available - first;
    }
    let overlap = (labels[0].x + labels[0].width + gap - labels[1].x).max(0.0);
    labels[0].x = (labels[0].x - overlap / 2.0).max(bounds.x);
    labels[1].x = labels[1].x.max(labels[0].x + labels[0].width + gap);
    let overflow = (labels[1].x + labels[1].width - bounds.x - bounds.width).max(0.0);
    labels[1].x -= overflow;
    labels[0].x = (labels[0].x - overflow).max(bounds.x);
}
impl<'a, Message: Clone + 'a> From<RangeSlider<'a, Message>> for Element<'a, Message> {
    fn from(slider: RangeSlider<'a, Message>) -> Self {
        Self::new(slider)
    }
}

#[cfg(test)]
mod label_tests {
    use super::*;
    #[test]
    fn overlapping_capsules_remain_inside_narrow_and_edge_aligned_hosts() {
        for width in [0.0, 20.0, 60.0, 300.0] {
            for x in [0.0, 0.5, 1.0] {
                let bounds = Rectangle {
                    x: 30.0,
                    y: 0.0,
                    width,
                    height: 100.0,
                };
                let frame = crate::slider::label_bounds(
                    Size::new(80.0, 16.0),
                    bounds,
                    30.0 + width * x,
                    80.0,
                );
                let mut frames = [frame; 2];
                separate_labels(&mut frames, bounds);
                assert!(
                    frames[0].x >= bounds.x
                        && frames[1].x + frames[1].width <= bounds.x + bounds.width + 0.001
                );
                assert!(frames[0].x + frames[0].width <= frames[1].x);
            }
        }
    }
}
