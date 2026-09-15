use crate::{motion::Transition, tokens};
use iced::time::Instant;

/// Expansion and release opacity have separate lifetimes: a held ripple does
/// not disappear, and a quick tap remains visible without delaying its message.
pub(crate) struct PressRipple {
    pub(crate) expansion: Transition,
    pub(crate) opacity: Transition,
    started: Option<Instant>,
    release_at: Option<Instant>,
}
impl PressRipple {
    pub(crate) fn new() -> Self {
        Self {
            expansion: Transition::standard(1.0),
            opacity: Transition::new(0.0),
            started: None,
            release_at: None,
        }
    }
    pub(crate) fn start(&mut self, now: Instant, motion: tokens::Motion) {
        self.expansion = Transition::standard(0.0);
        self.expansion.set(1.0, now, motion.ripple_expand);
        self.opacity = Transition::new(1.0);
        self.started = Some(now);
        self.release_at = None;
    }
    pub(crate) fn release(&mut self, now: Instant, motion: tokens::Motion, cancelled: bool) {
        let Some(started) = self.started else { return };
        let deadline = if cancelled {
            now
        } else {
            now.max(started + motion.ripple_expand / 2)
        };
        if self
            .release_at
            .is_none_or(|previous| cancelled && deadline < previous)
        {
            self.release_at = Some(deadline);
        }
        self.tick(now, motion);
    }
    pub(crate) fn tick(&mut self, now: Instant, motion: tokens::Motion) -> bool {
        if self.started.is_none() {
            return false;
        }
        if motion.ripple_expand.is_zero() {
            self.expansion = Transition::standard(1.0);
            if self.release_at.is_some() {
                self.release_at = Some(now);
            }
        }
        let expanding = self.expansion.tick(now);
        if let Some(deadline) = self.release_at.filter(|deadline| now >= *deadline) {
            self.opacity.set(0.0, deadline, motion.short);
        }
        let fading = self.opacity.tick(now);
        if self.opacity.value == 0.0 {
            // Hidden geometry needs no further frames, even after a quick tap.
            *self = Self::new();
            return false;
        }
        expanding || fading || self.release_at.is_some_and(|deadline| now < deadline)
    }
}

/// Cache the bounded SVG so its clipping follows ancestor transforms on both
/// renderers. The hover layer is painted separately and remains visible.
pub(crate) fn draw_bounded(
    renderer: &mut iced::Renderer,
    bounds: iced::Rectangle,
    clip: iced::Rectangle,
    corners: iced::border::Radius,
    origin: iced::Point,
    progress: f32,
    color: iced::Color,
    cache: &std::cell::RefCell<Option<([f32; 10], iced::advanced::svg::Handle)>>,
) {
    use iced::advanced::svg::{Handle, Renderer as _, Svg};
    let key = [
        bounds.width,
        bounds.height,
        corners.top_left,
        corners.top_right,
        corners.bottom_right,
        corners.bottom_left,
        origin.x,
        origin.y,
        progress,
        color.a,
    ];
    let mut cache = cache.borrow_mut();
    if cache.as_ref().is_none_or(|(old, _)| *old != key) {
        let (w, h) = (bounds.width, bounds.height);
        let limit = w.min(h) / 2.0;
        let [tl, tr, br, bl] = [
            corners.top_left,
            corners.top_right,
            corners.bottom_right,
            corners.bottom_left,
        ]
        .map(|r| r.min(limit));
        let cx = w * (origin.x + (0.5 - origin.x) * progress);
        let cy = h * (origin.y + (0.5 - origin.y) * progress);
        let initial = w.max(h) * 0.1;
        let radius = initial + ((w * w + h * h).sqrt() / 2.0 + 10.0 - initial) * progress;
        let path = format!(
            "M{tl} 0H{}Q{w} 0 {w} {tr}V{}Q{w} {h} {} {h}H{bl}Q0 {h} 0 {}V{tl}Q0 0 {tl} 0Z",
            w - tr,
            h - br,
            w - br,
            h - bl
        );
        let svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}"><defs><clipPath id="clip"><path d="{path}"/></clipPath><radialGradient id="edge"><stop offset="80%" stop-opacity="1"/><stop offset="100%" stop-opacity="0"/></radialGradient></defs><g opacity="{}" clip-path="url(#clip)"><circle cx="{cx}" cy="{cy}" r="{radius}" fill="url(#edge)"/></g></svg>"#,
            color.a
        );
        *cache = Some((key, Handle::from_memory(svg.into_bytes())));
    }
    renderer.draw_svg(
        Svg::from(&cache.as_ref().unwrap().1).color(iced::Color { a: 1.0, ..color }),
        bounds,
        clip,
    );
}
