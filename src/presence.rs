//! Shared lifetime for retained opening/closing surfaces.
use crate::{motion::Transition, tokens::Motion};
use iced::{Event, advanced::Shell, time::Duration, window};
use std::cell::Cell;
pub(crate) struct Presence {
    pub progress: Transition,
    pub motion: Cell<Motion>,
    pub height: Transition,
    pub slide: Transition,
    pub content: Transition,
    pub paper: Transition,
    surface_open: bool,
}
impl Default for Presence {
    fn default() -> Self {
        Self::new(false)
    }
}
impl Presence {
    pub fn new(open: bool) -> Self {
        Self {
            progress: Transition::standard(f32::from(open)),
            motion: Cell::new(Motion::default()),
            height: Transition::standard(f32::from(open)),
            slide: Transition::standard(f32::from(open)),
            content: Transition::linear(f32::from(open)),
            paper: Transition::linear(f32::from(open)),
            surface_open: open,
        }
    }
    pub fn begin_surface(
        &mut self,
        open: bool,
        now: iced::time::Instant,
        enter: Duration,
        exit: Duration,
        dialog: bool,
    ) -> bool {
        if !open && !self.surface_open && self.progress.value == 0.0 {
            return false;
        }
        if open && !self.surface_open && self.progress.value == 0.0 {
            self.height = Transition::standard(if dialog { 0.35 } else { 0.0 });
        }
        self.surface_open = open;
        let duration = if open { enter } else { exit };
        if let Some(scheme) = self.motion.get().scheme {
            let spatial = Some(scheme.default_spatial);
            let effects = Some(scheme.default_effects);
            return self
                .progress
                .set_motion(f32::from(open), now, duration, effects)
                | self
                    .height
                    .set_motion(if open { 1.0 } else { 0.35 }, now, duration, spatial)
                | self
                    .slide
                    .set_motion(f32::from(open), now, duration, spatial)
                | if dialog {
                    // Dialog content fades by covering child ink with its
                    // surface color. It must finish before that surface fades;
                    // otherwise the cover itself becomes an opaque rectangle.
                    // Resolve both phases from the same reversible spring below.
                    false
                } else {
                    self.content
                        .set_motion(f32::from(open), now, duration, effects)
                        | self
                            .paper
                            .set_motion(f32::from(open), now, duration, effects)
                };
        }
        let curve = if open {
            [0.2, 0.0, 0.0, 1.0]
        } else {
            [0.3, 0.0, 0.8, 0.15]
        };
        self.progress
            .set_eased(f32::from(open), now, duration, [0.0, 0.0, 1.0, 1.0])
            | self
                .height
                .set_eased(if open { 1.0 } else { 0.35 }, now, duration, curve)
            | self.slide.set_eased(f32::from(open), now, duration, curve)
            | self.content.set_delayed(
                f32::from(open),
                now,
                if open { enter / 10 } else { Duration::ZERO },
                if open {
                    if dialog { enter * 2 / 5 } else { enter / 2 }
                } else {
                    exit * 2 / 3
                },
            )
            | self.paper.set_delayed(
                f32::from(open),
                now,
                if open { Duration::ZERO } else { exit * 2 / 3 },
                if open { enter / 10 } else { exit / 3 },
            )
    }
    /// Baseline menu/basic-dialog surface motion. Opacity is a linear lifetime;
    /// the independently retargeted size/translation never jump on reversal.
    pub fn update_surface<Message>(
        &mut self,
        open: bool,
        event: &Event,
        enter: Duration,
        exit: Duration,
        dialog: bool,
        shell: &mut Shell<'_, Message>,
    ) {
        let now = match event {
            Event::Window(window::Event::RedrawRequested(now)) => *now,
            _ => crate::motion::now(),
        };
        let before = (
            self.progress.value,
            self.height.value,
            self.slide.value,
            self.content.value,
            self.paper.value,
        );
        let changed = self.begin_surface(open, now, enter, exit, dialog);
        let active = self.progress.tick(now)
            | self.height.tick(now)
            | self.slide.tick(now)
            | self.content.tick(now)
            | self.paper.tick(now);
        self.progress.value = self.progress.value.clamp(0.0, 1.0);
        if dialog && self.motion.get().scheme.is_some() {
            // The final third of the fade range belongs to the rounded paper.
            // Content is absent whenever paper is translucent, on entrance,
            // exit and reversal. Neither phase adds another settling delay.
            let paper_end = 1.0 / 3.0;
            let progress = self.progress.value;
            self.content.set(
                ((progress - paper_end) / (1.0 - paper_end)).clamp(0.0, 1.0),
                now,
                Duration::ZERO,
            );
            self.paper
                .set((progress / paper_end).clamp(0.0, 1.0), now, Duration::ZERO);
        }
        self.height.value = self.height.value.max(0.0);
        self.content.value = self.content.value.clamp(0.0, 1.0);
        self.paper.value = self.paper.value.clamp(0.0, 1.0);
        let after = (
            self.progress.value,
            self.height.value,
            self.slide.value,
            self.content.value,
            self.paper.value,
        );
        if changed || active || before != after {
            shell.request_redraw();
        }
        if before != after {
            shell.invalidate_layout();
        }
    }
    pub fn visible(&self, open: bool) -> bool {
        open || self.progress.value > 0.0
    }
    pub fn update<Message>(
        &mut self,
        open: bool,
        event: &Event,
        enter: Duration,
        exit: Duration,
        shell: &mut Shell<'_, Message>,
    ) {
        let now = match event {
            Event::Window(window::Event::RedrawRequested(now)) => *now,
            _ => crate::motion::now(),
        };
        let before = self.progress.value;
        let changed = self.progress.set_motion(
            f32::from(open),
            now,
            if open { enter } else { exit },
            self.motion.get().effects(crate::MotionSpeed::Default),
        );
        let animating = self.progress.tick(now);
        // A preference change while animating must settle the current target too.
        if (if open { enter } else { exit }).is_zero() {
            self.progress = Transition::standard(f32::from(open));
        }
        if changed || animating || before != self.progress.value {
            shell.request_redraw();
        }
        if before != self.progress.value {
            shell.invalidate_layout();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_scrim_and_content_follow_independent_linear_timelines() {
        let origin = iced::time::Instant::now();
        let mut p = Presence::new(false);
        let mut messages = Vec::<()>::new();
        let mut shell = Shell::new(&mut messages);
        let mut at = |p: &mut Presence, open, ms, enter, exit| {
            p.update_surface(
                open,
                &Event::Window(window::Event::RedrawRequested(
                    origin + Duration::from_millis(ms),
                )),
                enter,
                exit,
                true,
                &mut shell,
            );
        };
        let enter = Duration::from_millis(500);
        let exit = Duration::from_millis(150);
        at(&mut p, true, 0, enter, exit);
        assert_eq!(p.height.value, 0.35);
        at(&mut p, true, 50, enter, exit);
        assert_eq!(p.content.value, 0.0);
        at(&mut p, true, 120, enter, exit);
        assert!((p.progress.value - 0.24).abs() < 0.00001);
        assert!((p.content.value - 0.35).abs() < 0.00001);
        let before = (p.height.value, p.slide.value, p.content.value);
        at(&mut p, false, 120, enter, exit);
        assert_eq!((p.height.value, p.slide.value, p.content.value), before);
        at(&mut p, false, 270, enter, exit);
        assert!(!p.visible(false));
        assert_eq!(p.height.value, 0.35);
        assert_eq!(p.content.value, 0.0);
        at(&mut p, true, 270, Duration::ZERO, Duration::ZERO);
        assert_eq!(
            (
                p.progress.value,
                p.height.value,
                p.slide.value,
                p.content.value
            ),
            (1.0, 1.0, 1.0, 1.0)
        );
        assert!(!p.progress.tick(origin + Duration::from_millis(271)));
    }
    #[test]
    fn spring_dialog_keeps_paper_opaque_until_content_is_gone_even_on_reversal() {
        for scheme in [
            crate::MotionScheme::standard(),
            crate::MotionScheme::expressive(),
        ] {
            let now = iced::time::Instant::now();
            let mut p = Presence::new(false);
            let motion = Motion {
                scheme: Some(scheme),
                ..Motion::default()
            };
            p.motion.set(motion);
            let mut messages = Vec::<()>::new();
            let mut shell = Shell::new(&mut messages);
            let mut update = |p: &mut Presence, open, ms| {
                p.update_surface(
                    open,
                    &Event::Window(window::Event::RedrawRequested(
                        now + Duration::from_millis(ms),
                    )),
                    motion.dialog_enter,
                    motion.dialog_exit,
                    true,
                    &mut shell,
                );
                assert!(
                    p.content.value == 0. || p.paper.value == 1.,
                    "content cover must never outlive opaque paper"
                );
            };
            for (open, ms) in [
                (true, 0),
                (true, 40),
                (false, 40),
                (false, 65),
                (true, 65),
                (true, 400),
                (false, 400),
                (false, 440),
                (false, 480),
                (false, 520),
                (false, 600),
                (false, 800),
            ] {
                let before = (p.progress.value, p.content.value, p.paper.value);
                update(&mut p, open, ms);
                if ms == 40 && !open || ms == 65 && open || ms == 400 && !open {
                    assert_eq!(
                        (p.progress.value, p.content.value, p.paper.value),
                        before,
                        "retargeting at the same time must not jump"
                    );
                }
            }
            assert!(!p.visible(false));
            assert_eq!((p.content.value, p.paper.value), (0., 0.));
            p.motion.set(Motion::reduced());
            p.update_surface(
                true,
                &Event::Window(window::Event::RedrawRequested(
                    now + Duration::from_millis(801),
                )),
                Duration::ZERO,
                Duration::ZERO,
                true,
                &mut shell,
            );
            assert_eq!(
                (p.progress.value, p.content.value, p.paper.value),
                (1., 1., 1.)
            );
        }
    }
}
