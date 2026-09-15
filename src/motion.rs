use iced::time::Instant;
use std::time::Duration;

/// Event time defaults to the system clock. Unit tests can scope an override to
/// their own thread; no test clock or global mutable state ships in the library.
pub(crate) fn now() -> Instant {
    #[cfg(test)]
    if let Some(now) = TEST_TIME.with(std::cell::Cell::get) {
        return now;
    }
    Instant::now()
}

#[cfg(test)]
thread_local! {
    static TEST_TIME: std::cell::Cell<Option<Instant>> = const { std::cell::Cell::new(None) };
}

#[cfg(test)]
pub(crate) fn with_time<T>(now: Instant, f: impl FnOnce() -> T) -> T {
    struct Restore(Option<Instant>);
    impl Drop for Restore {
        fn drop(&mut self) {
            TEST_TIME.with(|clock| clock.set(self.0));
        }
    }
    let _restore = Restore(TEST_TIME.with(|clock| clock.replace(Some(now))));
    f()
}

/// A retargetable finite transition. No subscriptions or application messages.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Transition {
    pub value: f32,
    target: f32,
    from: f32,
    start: Option<Instant>,
    duration: Duration,
    curve: Option<[f32; 4]>,
}
impl Transition {
    pub fn new(value: f32) -> Self {
        Self {
            value,
            target: value,
            from: value,
            start: None,
            duration: Duration::ZERO,
            curve: None,
        }
    }
    /// Baseline Material standard easing: cubic-bezier(0.2, 0, 0, 1).
    pub fn standard(value: f32) -> Self {
        Self {
            curve: Some([0.2, 0.0, 0.0, 1.0]),
            ..Self::new(value)
        }
    }
    pub fn linear(value: f32) -> Self {
        Self {
            curve: Some([0.0, 0.0, 1.0, 1.0]),
            ..Self::new(value)
        }
    }
    /// Retarget using a new curve after sampling the outgoing transition.
    pub fn set_eased(
        &mut self,
        target: f32,
        now: Instant,
        duration: Duration,
        curve: [f32; 4],
    ) -> bool {
        let changed = self.set(target, now, duration);
        if changed {
            self.curve = Some(curve);
        }
        changed
    }
    pub fn set(&mut self, target: f32, now: Instant, duration: Duration) -> bool {
        if self.target == target {
            if duration.is_zero() && self.start.is_some() {
                self.value = target;
                self.from = target;
                self.start = None;
                self.duration = duration;
                return true;
            }
            return false;
        }
        self.tick(now);
        self.from = self.value;
        self.target = target;
        self.start = Some(now);
        self.duration = duration;
        self.tick(now);
        true
    }
    pub fn set_delayed(
        &mut self,
        target: f32,
        now: Instant,
        delay: Duration,
        duration: Duration,
    ) -> bool {
        let changed = self.set(target, now, duration);
        if changed && !duration.is_zero() {
            self.start = Some(now + delay);
            self.value = self.from;
        }
        changed
    }
    pub fn tick(&mut self, now: Instant) -> bool {
        let Some(start) = self.start else {
            return false;
        };
        let t = if self.duration.is_zero() {
            1.0
        } else {
            (now.saturating_duration_since(start).as_secs_f32() / self.duration.as_secs_f32())
                .min(1.0)
        };
        let eased = if t == 0.0 {
            0.0
        } else if let Some([x1, y1, x2, y2]) = self.curve {
            // Invert the curve's time coordinate before evaluating its value.
            // Fixed iterations also make reference captures reproducible.
            let (mut low, mut high) = (0.0, 1.0);
            for _ in 0..20 {
                let u = (low + high) * 0.5;
                let x = 3.0 * x1 * u * (1.0 - u).powi(2) + 3.0 * x2 * u * u * (1.0 - u) + u * u * u;
                if x < t {
                    low = u;
                } else {
                    high = u;
                }
            }
            let u = (low + high) * 0.5;
            3.0 * y1 * u * (1.0 - u).powi(2) + 3.0 * y2 * u * u * (1.0 - u) + u * u * u
        } else {
            1.0 - (1.0 - t).powi(3)
        };
        self.value = self.from + (self.target - self.from) * eased;
        if t >= 1.0 {
            self.value = self.target;
            self.start = None;
        }
        self.start.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn settles_and_retargets_without_jumping() {
        let now = Instant::now();
        let mut transition = Transition::new(0.0);
        assert!(!transition.tick(now));
        transition.set(1.0, now, Duration::from_millis(200));
        assert!(transition.tick(now + Duration::from_millis(70)));
        let before = transition.value;
        transition.set(
            0.0,
            now + Duration::from_millis(70),
            Duration::from_millis(100),
        );
        assert_eq!(transition.value, before);
        assert!(!transition.tick(now + Duration::from_millis(200)));
        assert_eq!(transition.value, 0.0);
        assert!(!transition.tick(now + Duration::from_secs(1)));
    }
    #[test]
    fn zero_duration_is_immediate() {
        let now = Instant::now();
        let mut transition = Transition::new(0.0);
        transition.set(1.0, now, Duration::ZERO);
        assert_eq!(transition.value, 1.0);
        assert!(!transition.tick(now));
    }
    #[test]
    fn reduced_motion_settles_an_already_active_target() {
        let now = Instant::now();
        let mut transition = Transition::standard(0.0);
        transition.set(1.0, now, Duration::from_millis(300));
        transition.tick(now + Duration::from_millis(50));
        assert!(transition.value < 1.0);
        assert!(transition.set(1.0, now + Duration::from_millis(50), Duration::ZERO));
        assert_eq!(transition.value, 1.0);
        assert!(!transition.tick(now + Duration::from_millis(51)));
    }
}
