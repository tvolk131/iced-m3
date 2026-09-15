//! Analytic critically damped progress spring (Android stiffness 50, damping 1).
use iced::time::Instant;
use std::time::Duration;

pub(super) struct Spring {
    pub value: f32,
    position: f64,
    velocity: f64,
    target: f64,
    last: Option<Instant>,
    scale: f64,
    threshold: f64,
}
impl Spring {
    pub fn new(value: f32) -> Self {
        Self {
            value,
            position: value as f64,
            velocity: 0.0,
            target: value as f64,
            last: None,
            scale: 1.0,
            threshold: 0.000075,
        }
    }
    pub fn set(
        &mut self,
        target: f32,
        now: Instant,
        duration: Duration,
        track_length: f32,
    ) -> bool {
        self.threshold = 0.75 / f64::from(track_length.max(1.0));
        self.tick(now);
        let changed = self.target != target as f64;
        if duration.is_zero() {
            let moved = self.value != target;
            *self = Self::new(target);
            return changed || moved;
        }
        self.scale = duration.as_secs_f64() / 0.2;
        if changed {
            self.target = target as f64;
            self.last = Some(now);
        }
        changed
    }
    pub fn tick(&mut self, now: Instant) -> bool {
        let Some(last) = self.last else {
            return false;
        };
        let dt = now.saturating_duration_since(last).as_secs_f64();
        let omega = (50.0 / self.scale).sqrt();
        let offset = self.position - self.target;
        let b = self.velocity + omega * offset;
        let decay = (-omega * dt).exp();
        self.position = self.target + (offset + b * dt) * decay;
        self.velocity = (self.velocity - omega * b * dt) * decay;
        if !(0.0..=1.0).contains(&self.position) {
            self.position = self.position.clamp(0.0, 1.0);
            self.velocity = 0.0;
        }
        // Android's one-pixel minimum change, with its 0.75 position and
        // 62.5 velocity multipliers. Extent is in this renderer's logical pixels.
        if (self.position - self.target).abs() < self.threshold
            && self.velocity.abs() < self.threshold * 62.5
        {
            self.position = self.target;
            self.velocity = 0.0;
            self.last = None;
        } else {
            self.last = Some(now);
        }
        self.value = self.position as f32;
        self.last.is_some()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn analytic_spring_preserves_velocity_and_is_frame_rate_independent() {
        let now = Instant::now();
        let at = |ms| now + Duration::from_millis(ms);
        let mut direct = Spring::new(0.0);
        let mut stepped = Spring::new(0.0);
        for s in [&mut direct, &mut stepped] {
            s.set(1.0, now, Duration::from_millis(200), 10000.0);
        }
        direct.tick(at(500));
        for ms in (10..=500).step_by(10) {
            stepped.tick(at(ms));
        }
        assert!((direct.value - stepped.value).abs() < 0.000001);
        assert!((direct.value - 0.8678208).abs() < 0.0001);
        let (position, velocity) = (direct.position, direct.velocity);
        direct.set(0.4, at(500), Duration::from_millis(200), 10000.0);
        assert_eq!(direct.position, position);
        assert_eq!(direct.velocity, velocity);
        assert!(!direct.tick(at(3500)));
        assert_eq!(direct.value, 0.4);
        direct.set(0.8, at(3500), Duration::ZERO, 10000.0);
        assert_eq!(direct.value, 0.8);
    }
}
