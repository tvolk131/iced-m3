//! Determinate and indeterminate progress, driven by widget redraw timestamps.
use crate::{Element, Theme, motion::Transition, tokens};
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, renderer,
    svg::{Handle, Renderer as _, Svg},
    widget::{Tree, tree},
};
use iced::{Color, Event, Length, Rectangle, Renderer, Size, mouse, window};
use std::{
    cell::{Cell, RefCell},
    time::{Duration, Instant},
};

mod circular;
mod linear;
mod spring;
mod wave;
use spring::Spring;
type ArcCache = RefCell<Option<((u32, u32), Handle)>>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Linear,
    Circular,
}
/// `value` is a fraction in 0..=1. Values outside it are clamped; NaN means zero.
pub struct Progress {
    kind: Kind,
    value: f32,
    buffer: Option<f32>,
    colors: Vec<Color>,
    track_color: Option<Color>,
    indeterminate: bool,
    paused: bool,
    width: Length,
    size: f32,
    wavy: bool,
    wave_amplitude: Option<f32>,
    wavelength: Option<f32>,
    wave_speed: f32,
}
pub fn linear_progress(value: f32) -> Progress {
    Progress {
        kind: Kind::Linear,
        value,
        buffer: None,
        colors: Vec::new(),
        track_color: None,
        indeterminate: false,
        paused: false,
        width: Length::Fill,
        size: tokens::size::PROGRESS_LINEAR,
        wavy: false,
        wave_amplitude: None,
        wavelength: None,
        wave_speed: 0.0,
    }
}
pub fn circular_progress(value: f32) -> Progress {
    Progress {
        kind: Kind::Circular,
        value,
        buffer: None,
        colors: Vec::new(),
        track_color: None,
        indeterminate: false,
        paused: false,
        width: Length::Shrink,
        size: tokens::size::PROGRESS_CIRCULAR,
        wavy: false,
        wave_amplitude: None,
        wavelength: None,
        wave_speed: 0.0,
    }
}
impl Progress {
    /// Opt into Material Expressive waves. Linear layout reserves the thickness
    /// plus twice the amplitude; circular uses its existing diameter. Circular
    /// loading uses one grow/retreat section over a visible inactive track.
    pub fn wavy(mut self, wavy: bool) -> Self {
        self.wavy = wavy;
        self
    }
    /// Wave amplitude in logical pixels (default: linear 3, circular 1.6).
    /// Zero draws a flat indicator while retaining the wavy motion profile.
    pub fn wave_amplitude(mut self, amplitude: f32) -> Self {
        if amplitude.is_finite() {
            self.wave_amplitude = Some(amplitude.clamp(0.0, 128.0));
        }
        self
    }
    /// Wavelength in logical pixels for both modes. Defaults: linear 40 when
    /// measured / 20 while loading, circular 15. Fitted to a whole wave count.
    pub fn wavelength(mut self, wavelength: f32) -> Self {
        if wavelength.is_finite() {
            self.wavelength = Some(wavelength.max(1.0));
        }
        self
    }
    /// Wave travel in logical pixels per second. Zero (Material's default) keeps
    /// the wave pattern stationary. Negative values reverse its direction.
    /// Travel pauses with `.paused(true)`, window inactivity and reduced motion.
    pub fn wave_speed(mut self, speed: f32) -> Self {
        if speed.is_finite() {
            self.wave_speed = speed.clamp(-10000.0, 10000.0);
        }
        self
    }
    fn amplitude(&self) -> f32 {
        if !self.wavy {
            0.0
        } else {
            self.wave_amplitude
                .unwrap_or(if self.kind == Kind::Linear { 3.0 } else { 1.6 })
        }
    }
    fn wavelength_for(&self, indeterminate: bool) -> f32 {
        self.wavelength.unwrap_or(if self.kind == Kind::Circular {
            15.0
        } else if indeterminate {
            20.0
        } else {
            40.0
        })
    }
    fn height(&self) -> f32 {
        self.size
            + if self.kind == Kind::Linear {
                self.amplitude() * 2.0
            } else {
                0.0
            }
    }
    /// Override the inactive track color. Baseline circular loading hides the
    /// track; the wavy circular profile keeps it visible.
    pub fn track_color(mut self, color: Color) -> Self {
        self.track_color = Some(color);
        self
    }

    /// Buffered fraction for determinate linear progress. Clamped to at least the
    /// completed fraction; ignored by circular and indeterminate indicators.
    pub fn buffer(mut self, value: f32) -> Self {
        self.buffer = Some(fraction(value));
        self
    }
    /// Indicator colors. Loading blends colors at cycle handoff, except wavy
    /// linear progress which changes color at its empty cycle boundary.
    /// Measured indicators use the first. Empty uses theme primary.
    pub fn colors(mut self, colors: impl IntoIterator<Item = Color>) -> Self {
        self.colors = colors.into_iter().collect();
        self
    }
    fn buffer_value(&self) -> f32 {
        if self.kind == Kind::Linear && !self.indeterminate {
            self.buffer.unwrap_or(self.value()).max(self.value())
        } else {
            self.value()
        }
    }
    /// On exit, finish the current loading sequence before animating the measured
    /// value: 333ms for baseline circular / 500ms for wavy circular, the rest of
    /// the linear cycle (2s baseline / 1.8s wavy).
    /// Reduced motion, paused, hidden and unfocused indicators skip this handoff.
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }
    /// Freeze loading and wave travel without removing their current frame.
    /// Measured value changes still update.
    pub fn paused(mut self, paused: bool) -> Self {
        self.paused = paused;
        self
    }
    /// Linear width. Circular indicators use `size` for both dimensions.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
    /// Circular diameter, or linear thickness, in logical pixels.
    pub fn size(mut self, size: f32) -> Self {
        if size.is_finite() {
            self.size = size.max(1.0);
        }
        self
    }
    fn value(&self) -> f32 {
        fraction(self.value)
    }
}
enum Handoff {
    Circular {
        progress: Transition,
        deadline: Duration,
    },
    Linear(Duration),
}
struct State {
    progress: Spring,
    handoff: Option<Handoff>,
    buffer: Transition,
    indeterminate: bool,
    elapsed: Duration,
    animation_origin: Instant,
    last: Option<Instant>,
    focused: bool,
    motion: Cell<tokens::Motion>,
    arc: ArcCache,
    track_arc: ArcCache,
    wave_cache: wave::Cache,
    amplitude: Transition,
    wave_elapsed: Duration,
    wave_last: Option<Instant>,
}
impl Widget<(), Theme, Renderer> for Progress {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State {
            progress: Spring::new(self.value()),
            handoff: None,
            buffer: Transition::standard(self.buffer_value()),
            indeterminate: self.indeterminate,
            elapsed: Duration::ZERO,
            animation_origin: crate::motion::now(),
            last: None,
            focused: true,
            motion: Cell::new(tokens::Motion::default()),
            arc: RefCell::new(None),
            track_arc: RefCell::new(None),
            wave_cache: RefCell::new(None),
            amplitude: Transition::standard(f32::from(
                self.indeterminate || (0.1..=0.9).contains(&self.value()),
            )),
            wave_elapsed: Duration::ZERO,
            wave_last: None,
        })
    }
    fn size(&self) -> Size<Length> {
        Size::new(
            if self.kind == Kind::Circular {
                Length::Fixed(self.size)
            } else {
                self.width
            },
            Length::Fixed(self.height()),
        )
    }
    fn layout(&mut self, _: &mut Tree, _: &Renderer, limits: &layout::Limits) -> layout::Node {
        let size = <Self as Widget<(), Theme, Renderer>>::size(self);
        layout::Node::new(limits.resolve(size.width, size.height, Size::new(160.0, self.height())))
    }
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        _: mouse::Cursor,
        _: &Renderer,
        _: &mut dyn Clipboard,
        shell: &mut Shell<'_, ()>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        if let Some(focused) = crate::activity::window_focus(event) {
            if state.focused != focused {
                state.last = None;
                state.wave_last = None;
                if focused {
                    shell.request_redraw();
                }
            }
            state.focused = focused;
        }
        let now = match event {
            Event::Window(window::Event::RedrawRequested(now)) => *now,
            _ => crate::motion::now(),
        };
        let visible = layout
            .bounds()
            .intersection(viewport)
            .is_some_and(|r| r.width > 0.0 && r.height > 0.0);
        let motion = state.motion.get();
        let track_length = if self.kind == Kind::Circular {
            layout.bounds().width.min(layout.bounds().height) * std::f32::consts::PI
        } else {
            layout.bounds().width
        };
        let animate = visible && state.focused && !self.paused && !motion.medium.is_zero();
        let circular_finish = motion.medium.mul_f32(if self.wavy {
            500.0 / 200.0
        } else {
            333.0 / 200.0
        });
        if self.indeterminate && !state.indeterminate {
            state.indeterminate = true;
            state.elapsed = Duration::ZERO;
            state.last = None;
            state.handoff = None;
            shell.request_redraw();
        }
        if state.indeterminate {
            if animate {
                if let Some(last) = state.last {
                    state.elapsed += now.saturating_duration_since(last);
                }
                state.last = Some(now);
                let animation_time = state.animation_origin + state.elapsed;
                if !self.indeterminate && state.handoff.is_none() {
                    state.handoff = Some(if self.kind == Kind::Circular {
                        let mut closing = Transition::standard(0.0);
                        closing.set_eased(
                            1.0,
                            animation_time,
                            circular_finish,
                            [0.4, 0.0, 0.2, 1.0],
                        );
                        Handoff::Circular {
                            progress: closing,
                            deadline: state.elapsed + circular_finish,
                        }
                    } else {
                        let cycle = if self.wavy { 1800 } else { 2000 };
                        let ms = (state.elapsed.as_millis() / cycle + 1) * cycle;
                        Handoff::Linear(Duration::from_millis(ms.min(u64::MAX as u128) as u64))
                    });
                }
                let mut completed_at = None;
                let mut cancelled = false;
                match &mut state.handoff {
                    Some(Handoff::Circular {
                        progress: closing,
                        deadline,
                    }) => {
                        let duration = circular_finish;
                        if closing.set(f32::from(!self.indeterminate), animation_time, duration) {
                            *deadline = state.elapsed + duration;
                        }
                        closing.tick(animation_time);
                        if !self.indeterminate && closing.value >= 1.0 {
                            completed_at = Some(*deadline);
                        }
                        cancelled = self.indeterminate && closing.value <= 0.0;
                    }
                    Some(Handoff::Linear(end)) => {
                        if state.elapsed >= *end {
                            completed_at = Some(*end);
                        }
                        cancelled = self.indeterminate;
                    }
                    None => {}
                }
                if cancelled {
                    state.handoff = None;
                } else if let Some(completed_at) = completed_at {
                    state.indeterminate = false;
                    state.handoff = None;
                    state.progress = Spring::new(0.0);
                    state.buffer = Transition::standard(0.0);
                    // Integrate the part of this frame after the handoff, too.
                    // Sparse redraws and dense redraws must reach the same value.
                    let finished = now - state.elapsed.saturating_sub(completed_at);
                    state
                        .progress
                        .set(self.value(), finished, motion.medium, track_length);
                    state
                        .buffer
                        .set(self.buffer_value(), finished, motion.medium);
                    state.last = None;
                }
                shell.request_redraw();
            } else {
                state.last = None;
                if !self.indeterminate {
                    state.indeterminate = false;
                    state.handoff = None;
                    state.progress = Spring::new(self.value());
                    state.buffer = Transition::standard(self.buffer_value());
                    shell.request_redraw();
                }
            }
        }
        if !state.indeterminate {
            if visible && state.focused {
                let changed = state
                    .progress
                    .set(self.value(), now, motion.medium, track_length);
                let buffer_changed = state.buffer.set(self.buffer_value(), now, motion.medium);
                let progress_running = state.progress.tick(now);
                let buffer_running = state.buffer.tick(now);
                if changed || buffer_changed || progress_running || buffer_running {
                    shell.request_redraw();
                }
            } else {
                state.progress = Spring::new(self.value());
                state.buffer = Transition::standard(self.buffer_value());
                state.last = None;
            }
        }
        if self.wavy {
            let target =
                f32::from(state.indeterminate || (0.1..=0.9).contains(&state.progress.value));
            let duration = if animate {
                motion.medium.mul_f32(2.5)
            } else {
                Duration::ZERO
            };
            let changed = state.amplitude.set_eased(
                target,
                now,
                duration,
                if target > state.amplitude.value {
                    [0.2, 0.0, 0.0, 1.0]
                } else {
                    [0.3, 0.0, 0.8, 0.15]
                },
            );
            let running = state.amplitude.tick(now);
            if changed || running {
                shell.request_redraw();
            }
            if animate
                && self.wave_speed != 0.0
                && self.amplitude() > 0.0
                && state.amplitude.value > 0.0
                && (state.indeterminate || state.progress.value > 0.0)
            {
                if let Some(last) = state.wave_last {
                    state.wave_elapsed += now.saturating_duration_since(last);
                }
                state.wave_last = Some(now);
                shell.request_redraw();
            } else {
                state.wave_last = None;
            }
        } else {
            state.wave_last = None;
        }
    }
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _: &renderer::Style,
        layout: Layout<'_>,
        _: mouse::Cursor,
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
        let elapsed = if state.indeterminate && !theme.motion.medium.is_zero() {
            state.elapsed
        } else {
            Duration::ZERO
        };
        let indicator = if self.wavy
            && self.kind == Kind::Linear
            && state.indeterminate
            && !self.colors.is_empty()
        {
            self.colors[(elapsed.as_secs_f64() / 1.8).floor() as usize % self.colors.len()]
        } else if self.wavy && self.kind == Kind::Circular && state.indeterminate {
            wave::color(&self.colors, c.primary, elapsed)
        } else if self.kind == Kind::Circular && state.indeterminate {
            circular::color(&self.colors, c.primary, elapsed)
        } else {
            cycle_color(&self.colors, c.primary, elapsed.as_secs_f64() / 2.0)
        };
        let track_color = self.track_color.unwrap_or(if self.wavy {
            c.secondary_container
        } else {
            c.primary_container
        });
        renderer.with_layer(clip, |renderer| {
            if self.wavy {
                let elapsed = if state.indeterminate && theme.motion.medium.is_zero() {
                    Duration::from_millis(1000)
                } else {
                    elapsed
                };
                let mut segments = if state.indeterminate {
                    wave::linear_segments(elapsed).to_vec()
                } else {
                    vec![(0.0, state.progress.value)]
                };
                segments = segments
                    .into_iter()
                    .map(|(a, b)| (a.clamp(0., 1.), b.clamp(0., 1.)))
                    .filter(|(a, b)| b > a)
                    .collect();
                segments.sort_by(|a, b| a.0.total_cmp(&b.0));
                let (start, sweep) = if state.indeterminate {
                    let (start, sweep) = wave::retreat(elapsed);
                    let close = match &state.handoff {
                        Some(Handoff::Circular { progress, .. }) => progress.value,
                        _ => 0.0,
                    };
                    (start, sweep * (1. - close))
                } else {
                    (-90., state.progress.value * 360.)
                };
                let wavelength = self.wavelength_for(state.indeterminate);
                let phase = if theme.motion.medium.is_zero() {
                    0.
                } else {
                    (state.wave_elapsed.as_secs_f64() * self.wave_speed as f64 / wavelength as f64)
                        .rem_euclid(1.) as f32
                };
                wave::draw(
                    renderer,
                    &state.wave_cache,
                    bounds,
                    clip,
                    self.kind,
                    &segments,
                    start,
                    sweep,
                    if self.kind == Kind::Linear {
                        self.size
                    } else {
                        4.
                    },
                    self.amplitude() * state.amplitude.value,
                    wavelength,
                    phase,
                    (self.buffer.is_some() && !state.indeterminate && self.kind == Kind::Linear)
                        .then_some(state.buffer.value.max(state.progress.value)),
                    state.indeterminate,
                    indicator,
                    track_color,
                );
                return;
            }
            if self.kind == Kind::Linear {
                let elapsed = if state.indeterminate && theme.motion.medium.is_zero() {
                    Duration::from_millis(1000)
                } else {
                    elapsed
                };
                let mut segments = if state.indeterminate {
                    linear::segments(elapsed).to_vec()
                } else {
                    vec![(0.0, state.progress.value)]
                };
                segments = segments
                    .into_iter()
                    .map(|(a, b)| (a.clamp(0.0, 1.0), b.clamp(0.0, 1.0)))
                    .filter(|(a, b)| b > a)
                    .collect();
                segments.sort_by(|a, b| a.0.total_cmp(&b.0));
                let mut bar = |start: f32, end: f32, color| {
                    if end > start {
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: Rectangle {
                                    x: bounds.x + start,
                                    width: end - start,
                                    ..bounds
                                },
                                border: iced::Border {
                                    radius: (bounds.height / 2.0).min((end - start) / 2.0).into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            color,
                        );
                    }
                };
                let gap = 4.0;
                let mut position = 0.0;
                let mut tracks = Vec::new();
                for &(start, end) in &segments {
                    let a = start * bounds.width - gap * ((1.0 - start) / 0.01).min(1.0);
                    if a > position {
                        tracks.push((position, a));
                    }
                    position = position.max(end * bounds.width + gap * (end / 0.01).min(1.0));
                }
                if position < bounds.width {
                    tracks.push((position, bounds.width));
                }
                for (a, b) in tracks {
                    bar(a, b, track_color);
                    if self.buffer.is_some() && !state.indeterminate {
                        bar(
                            a,
                            b.min(bounds.width * state.buffer.value.max(state.progress.value)),
                            crate::theme::alpha(indicator, 0.24),
                        );
                    }
                }
                for (a, b) in segments {
                    bar(a * bounds.width, b * bounds.width, indicator);
                }
                if !state.indeterminate {
                    let size = 4.0_f32
                        .min(bounds.height)
                        .min(bounds.width * (1.0 - state.progress.value));
                    if size > 0.0 {
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: Rectangle {
                                    x: bounds.x + bounds.width - size,
                                    y: bounds.center_y() - size / 2.0,
                                    width: size,
                                    height: size,
                                },
                                border: iced::Border {
                                    radius: (size / 2.0).into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            indicator,
                        );
                    }
                }
            } else {
                let (start, sweep) = if state.indeterminate {
                    let (start, sweep) = circular::arc(elapsed);
                    let close = match &state.handoff {
                        Some(Handoff::Circular {
                            progress: closing, ..
                        }) => closing.value,
                        _ => 0.0,
                    };
                    (start + sweep * close, sweep * (1.0 - close))
                } else {
                    (-90.0, state.progress.value * 360.0)
                };
                if !state.indeterminate && sweep < 360.0 {
                    let gap =
                        (8.0_f32 / 18.0).to_degrees() * (state.progress.value / 0.01).min(1.0);
                    draw_arc(
                        renderer,
                        &state.track_arc,
                        bounds,
                        clip,
                        start + sweep + gap,
                        (360.0 - sweep - 2.0 * gap).max(0.0),
                        track_color,
                    );
                }
                draw_arc(renderer, &state.arc, bounds, clip, start, sweep, indicator);
            }
        });
    }
}
fn draw_arc(
    renderer: &mut Renderer,
    cache: &ArcCache,
    bounds: Rectangle,
    clip: Rectangle,
    start: f32,
    sweep: f32,
    color: Color,
) {
    if sweep <= 0.0 || color.a <= 0.0 {
        return;
    }
    let key = (start.to_bits(), sweep.to_bits());
    let mut arc = cache.borrow_mut();
    if arc.as_ref().is_none_or(|(old, _)| *old != key) {
        let shape = if sweep >= 359.99 {
            "<circle cx='24' cy='24' r='18'/>".to_owned()
        } else {
            let a = start.to_radians();
            let b = (start + sweep).to_radians();
            format!(
                "<path d='M {} {} A 18 18 0 {} 1 {} {}'/>",
                24.0 + 18.0 * a.cos(),
                24.0 + 18.0 * a.sin(),
                u8::from(sweep > 180.0),
                24.0 + 18.0 * b.cos(),
                24.0 + 18.0 * b.sin()
            )
        };
        let svg = format!(
            "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 48 48'><g fill='none' stroke='black' stroke-width='4' stroke-linecap='round'>{shape}</g></svg>"
        );
        *arc = Some((key, Handle::from_memory(svg.into_bytes())));
    }
    let side = bounds.width.min(bounds.height);
    renderer.draw_svg(
        Svg {
            handle: arc.as_ref().unwrap().1.clone(),
            color: Some(color),
            rotation: iced::Radians(0.0),
            opacity: 1.0,
        },
        Rectangle {
            x: bounds.center_x() - side / 2.0,
            y: bounds.center_y() - side / 2.0,
            width: side,
            height: side,
        },
        clip,
    );
}
fn fraction(value: f32) -> f32 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(0.0, 1.0)
    }
}
fn cycle_color(colors: &[Color], fallback: Color, cycles: f64) -> Color {
    if colors.is_empty() {
        return fallback;
    }
    let index = (cycles.floor() as usize) % colors.len();
    let blend = ((cycles.fract() as f32 - 0.6) / 0.4).clamp(0.0, 1.0);
    crate::theme::mix(
        colors[index],
        colors[(index + 1) % colors.len()],
        linear::bezier(blend, [0.4, 0.0, 0.2, 1.0]),
    )
}
impl<'a, Message: 'a> From<Progress> for Element<'a, Message> {
    fn from(progress: Progress) -> Self {
        Element::<()>::new(progress).map(|()| unreachable!("progress has no messages"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn buffers_clamp_invalid_values_and_never_trail_completed_work() {
        for (value, buffered, expected) in [
            (0.6, 0.2, 0.6),
            (0.2, f32::NAN, 0.2),
            (0.2, f32::INFINITY, 1.0),
            (f32::NAN, 0.4, 0.4),
            (2.0, -1.0, 1.0),
        ] {
            assert_eq!(
                linear_progress(value).buffer(buffered).buffer_value(),
                expected
            );
        }
    }
    #[test]
    fn color_cycles_are_continuous_and_wrap_without_a_flash() {
        let colors = [
            Color::BLACK,
            Color::WHITE,
            Color::from_rgb(1.0, 0.0, 0.0),
            Color::from_rgb(0.0, 1.0, 0.0),
        ];
        assert_eq!(cycle_color(&[], Color::WHITE, 9.0), Color::WHITE);
        assert_eq!(cycle_color(&colors[..1], Color::WHITE, 9.9), Color::BLACK);
        for i in 1..=8 {
            let before = cycle_color(&colors, Color::WHITE, i as f64 - 0.0001);
            let after = cycle_color(&colors, Color::WHITE, i as f64);
            assert!(
                (before.r - after.r).abs()
                    + (before.g - after.g).abs()
                    + (before.b - after.b).abs()
                    < 0.001
            );
        }
        assert_eq!(cycle_color(&colors, Color::WHITE, 0.5), Color::BLACK);
        assert_eq!(cycle_color(&colors, Color::WHITE, 4.5), Color::BLACK);
    }
}
