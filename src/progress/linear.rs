//! Baseline Material Web linear indeterminate motion.
//!
//! Keyframe measurements and easing control points:
//! <https://github.com/material-components/material-web/blob/main/progress/internal/_linear-progress.scss>
//! The two bars have independent position AND width curves over 2000ms.
//! This evaluates those measurements in Rust; it does not run or copy the CSS.
use std::time::Duration;

type Curve = [f32; 4];
type Keyframe = (f32, f32, Curve);
const LINEAR: Curve = [0.0, 0.0, 1.0, 1.0];

const FIRST_POSITION: &[Keyframe] = &[
    (0.0, 0.0, LINEAR),
    (0.2, 0.0, [0.5, 0.0, 0.701732, 0.495819]),
    (0.5915, 0.836714, [0.302435, 0.381352, 0.55, 0.956352]),
    (1.0, 2.00611, LINEAR),
];
const FIRST_WIDTH: &[Keyframe] = &[
    (0.0, 0.08, LINEAR),
    (0.3665, 0.08, [0.334731, 0.12482, 0.785844, 1.0]),
    (0.6915, 0.661479, [0.06, 0.11, 0.6, 1.0]),
    (1.0, 0.08, LINEAR),
];
const SECOND_POSITION: &[Keyframe] = &[
    (0.0, 0.0, [0.15, 0.0, 0.515058, 0.409685]),
    (0.25, 0.376519, [0.31033, 0.284058, 0.8, 0.733712]),
    (0.4835, 0.843862, [0.4, 0.627035, 0.6, 0.902026]),
    (1.0, 1.60278, LINEAR),
];
const SECOND_WIDTH: &[Keyframe] = &[
    (0.0, 0.08, [0.205028, 0.057051, 0.57661, 0.453971]),
    (0.1915, 0.457104, [0.152313, 0.196432, 0.648374, 1.00432]),
    (0.4415, 0.72796, [0.257759, -0.003163, 0.211762, 1.38179]),
    (1.0, 0.08, LINEAR),
];

/// Unclipped track fractions; the renderer clips the portions entering/leaving.
pub(super) fn segments(elapsed: Duration) -> [(f32, f32); 2] {
    let phase = (elapsed.as_secs_f64() % 2.0) as f32 / 2.0;
    [
        (-1.45167, FIRST_POSITION, FIRST_WIDTH),
        (-0.548889, SECOND_POSITION, SECOND_WIDTH),
    ]
    .map(|(offset, position, width)| {
        let width = sample(phase, width);
        // CSS scales the inner bar about its center, then translates its parent.
        let start = offset + sample(phase, position) + (1.0 - width) / 2.0;
        (start, start + width)
    })
}

fn sample(time: f32, frames: &[Keyframe]) -> f32 {
    for pair in frames.windows(2) {
        let (start_time, start_value, curve) = pair[0];
        let (end_time, end_value, _) = pair[1];
        if time < end_time {
            let t = ((time - start_time) / (end_time - start_time)).clamp(0.0, 1.0);
            return start_value + (end_value - start_value) * bezier(t, curve);
        }
    }
    frames.last().unwrap().1
}

pub(super) fn bezier(time: f32, [x1, y1, x2, y2]: Curve) -> f32 {
    if time == 0.0 || time == 1.0 {
        return time;
    }
    let coordinate = |u: f32, p1: f32, p2: f32| {
        3.0 * (1.0 - u).powi(2) * u * p1 + 3.0 * (1.0 - u) * u * u * p2 + u.powi(3)
    };
    let (mut low, mut high) = (0.0, 1.0);
    // Invert x before evaluating y. Fixed iterations keep reference frames stable.
    for _ in 0..20 {
        let u = (low + high) / 2.0;
        if coordinate(u, x1, x2) < time {
            low = u;
        } else {
            high = u;
        }
    }
    coordinate((low + high) / 2.0, y1, y2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn independent_widths_follow_material_growth_contraction_and_period() {
        let at = |ms| segments(Duration::from_millis(ms));
        let width = |bar: (f32, f32)| bar.1 - bar.0;
        // Reference keyframe measurements: first grows 8% -> 66%; second 8% -> 73%.
        assert!((width(at(733)[0]) - 0.08).abs() < 0.00001);
        assert!((width(at(1383)[0]) - 0.661479).abs() < 0.00001);
        assert!((width(at(883)[1]) - 0.72796).abs() < 0.00001);
        assert!(width(at(1950)[0]) < 0.1 && width(at(1950)[1]) < 0.1);
        assert_eq!(at(500), at(2500));
        assert_ne!(
            at(500),
            at(1500),
            "the two-second cycle must not collapse to identical half cycles"
        );
        for ms in 0..2000 {
            for (start, end) in at(ms) {
                assert!(start.is_finite() && end.is_finite() && end >= start);
            }
        }
    }
}
