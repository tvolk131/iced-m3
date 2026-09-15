//! Timings and endpoint motion from Android's baseline advance delegate.
//! See docs/DESKTOP_FIDELITY.md for the pinned reference and profile choices.
use iced::Color;
use std::time::Duration;

fn ease(value: f32) -> f32 {
    super::linear::bezier(value.clamp(0.0, 1.0), [0.4, 0.0, 0.2, 1.0])
}
pub(super) fn arc(elapsed: Duration) -> (f32, f32) {
    let ms = (elapsed.as_secs_f64() * 1000.0 % 5400.0) as f32;
    let rotation = 1520.0 * ms / 5400.0;
    let mut tail = rotation - 20.0;
    let mut head = rotation;
    for delay in [0.0, 1350.0, 2700.0, 4050.0] {
        head += 250.0 * ease((ms - delay) / 667.0);
        tail += 250.0 * ease((ms - delay - 667.0) / 667.0);
    }
    ((tail - 90.0).rem_euclid(360.0), head - tail)
}
pub(super) fn color(colors: &[Color], fallback: Color, elapsed: Duration) -> Color {
    if colors.is_empty() {
        return fallback;
    }
    let cycles = elapsed.as_secs_f64() * 1000.0 / 1350.0;
    let index = cycles.floor() as usize % colors.len();
    let blend = ease((cycles.fract() as f32 * 1350.0 - 1000.0) / 333.0);
    let a = colors[index];
    let b = colors[(index + 1) % colors.len()];
    // ArgbEvaluatorCompat interpolates RGB in linear light using gamma 2.2.
    let channel =
        |a: f32, b: f32| (a.powf(2.2) + (b.powf(2.2) - a.powf(2.2)) * blend).powf(1.0 / 2.2);
    Color::from_rgba(
        channel(a.r, b.r),
        channel(a.g, b.g),
        channel(a.b, b.b),
        a.a + (b.a - a.a) * blend,
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reference_endpoints_grow_contract_and_join_after_four_cycles() {
        let at = |ms| arc(Duration::from_millis(ms));
        assert_eq!(at(0), (250.0, 20.0));
        for start in [0, 1350, 2700, 4050] {
            assert!((at(start).1 - 20.0).abs() < 0.001);
            assert!((at(start + 667).1 - 270.0).abs() < 0.001);
            assert!((at(start + 1334).1 - 20.0).abs() < 0.001);
        }
        assert_eq!(at(400), at(5800));
        let distance = (at(5399).0 - at(5400).0 + 180.0).rem_euclid(360.0) - 180.0;
        assert!(distance.abs() < 0.3);
        for ms in 0..5400 {
            assert!((19.99..=270.01).contains(&at(ms).1));
        }
    }
    #[test]
    fn colors_hold_then_fade_without_resetting_at_the_cycle_boundary() {
        let palette = [Color::BLACK, Color::WHITE];
        let at = |ms| color(&palette, Color::TRANSPARENT, Duration::from_millis(ms));
        assert_eq!(at(1000), Color::BLACK);
        assert_eq!(at(1333), Color::WHITE);
        assert_eq!(at(1350), Color::WHITE);
        assert!(at(1166).r > 0.7);
        assert_eq!(at(2700), Color::BLACK);
    }
}
