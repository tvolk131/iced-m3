//! Canonical Material shape/morph data generated with AndroidX graphics-shapes.
//! See assets/loading/README.md for the pinned inputs and Apache-2.0 notices.
use std::{fmt::Write, sync::LazyLock};

// Each record contains the start and end cubic (anchor, two controls, anchor).
static MORPHS: LazyLock<[Vec<[f32; 16]>; 7]> = LazyLock::new(|| {
    let mut bytes = include_bytes!("../../assets/loading/morphs.bin").as_slice();
    std::array::from_fn(|_| {
        let count = u32::from_le_bytes(bytes[..4].try_into().unwrap()) as usize;
        bytes = &bytes[4..];
        (0..count)
            .map(|_| {
                std::array::from_fn(|_| {
                    let value = f32::from_le_bytes(bytes[..4].try_into().unwrap());
                    bytes = &bytes[4..];
                    value
                })
            })
            .collect()
    })
});

pub(super) fn path(shape: usize, amount: f32) -> String {
    let mut path = String::with_capacity(6000);
    for (i, pair) in MORPHS[shape % 7].iter().enumerate() {
        let p: [f32; 8] = std::array::from_fn(|j| pair[j] + (pair[j + 8] - pair[j]) * amount);
        if i == 0 {
            write!(path, "M {} {} ", p[0], p[1]).unwrap();
        }
        write!(
            path,
            "C {} {} {} {} {} {} ",
            p[2], p[3], p[4], p[5], p[6], p[7]
        )
        .unwrap();
    }
    path.push('Z');
    path
}

/// Analytical response of a stiffness-200, damping-0.6 spring to 650ms steps.
/// Retaining the preceding residuals preserves velocity at each new target.
pub(super) fn frame(elapsed: f64) -> (usize, f32, f32) {
    let cycle = elapsed / 0.65;
    let n = cycle.floor();
    let local = (cycle - n) * 0.65;
    let residual = |time: f64| {
        (-8.48528137423857 * time).exp()
            * ((11.313708498984761 * time).cos() + 0.75 * (11.313708498984761 * time).sin())
    };
    let offset = 1.0
        - (0..=6)
            .filter(|i| *i as f64 <= n)
            .map(|i| residual(local + i as f64 * 0.65))
            .sum::<f64>();
    let step = offset.floor();
    let shape = (n.rem_euclid(7.0) + step).rem_euclid(7.0) as usize;
    let rotation =
        (140.0 * n.rem_euclid(18.0) + 50.0 * (cycle - n) + 90.0 * offset).rem_euclid(360.0) as f32;
    (shape, (offset - step) as f32, rotation)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spring_and_rotation_are_continuous_across_shape_targets() {
        assert_eq!(frame(0.0), (0, 0.0, 0.0));
        for n in 1..140 {
            let (a, t, r) = frame(n as f64 * 0.65 - 0.000001);
            let (b, u, s) = frame(n as f64 * 0.65 + 0.000001);
            let position = |i: usize, f: f32| i as f32 + f;
            let distance = (position(a, t) - position(b, u) + 3.5).rem_euclid(7.0) - 3.5;
            assert!(distance.abs() < 0.0001);
            assert!(((r - s + 180.0).rem_euclid(360.0) - 180.0).abs() < 0.01);
        }
        for elapsed in [0., 0.3, 0.649, 0.65, 4.55, 86400., 1e9] {
            let (shape, amount, rotation) = frame(elapsed);
            assert!(shape < 7 && (0.0..1.0).contains(&amount));
            assert!((0.0..360.0).contains(&rotation));
        }
    }
}
