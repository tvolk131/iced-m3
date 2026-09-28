//! Material spring schemes shared by built-in and application-defined motion.
//! Values follow AndroidX Material 3 motion tokens v0_14_0. See the Expressive
//! guide for the pinned source revision and the components using these tokens.

/// A unit-mass spring. Stiffness controls speed; damping controls oscillation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spring {
    stiffness: f32,
    damping: f32,
}
impl Spring {
    /// Construct a spring with finite positive stiffness and damping ratio.
    ///
    /// # Panics
    /// Panics for non-finite or non-positive parameters.
    pub fn new(stiffness: f32, damping: f32) -> Self {
        assert!(stiffness.is_finite() && stiffness > 0.0);
        assert!(damping.is_finite() && damping > 0.0);
        Self { stiffness, damping }
    }
    /// Spring stiffness, in the unit-mass model.
    pub fn stiffness(self) -> f32 {
        self.stiffness
    }
    /// Damping ratio: below one oscillates; one is critically damped.
    pub fn damping(self) -> f32 {
        self.damping
    }
    /// Evaluate position and velocity after `seconds`, relative to a stationary
    /// target. This analytic solution is independent of rendering frame rate.
    pub fn sample(self, displacement: f64, velocity: f64, seconds: f64) -> (f64, f64) {
        let t = seconds.max(0.0);
        let w = f64::from(self.stiffness).sqrt();
        let d = f64::from(self.damping);
        if (d - 1.0).abs() < 1e-6 {
            let b = velocity + w * displacement;
            let decay = (-w * t).exp();
            (
                (displacement + b * t) * decay,
                (velocity - w * b * t) * decay,
            )
        } else if d < 1.0 {
            let wd = w * (1.0 - d * d).sqrt();
            let a = d * w;
            let b = (velocity + a * displacement) / wd;
            let (sin, cos) = (wd * t).sin_cos();
            let x = displacement * cos + b * sin;
            let decay = (-a * t).exp();
            (
                x * decay,
                (-a * x - displacement * wd * sin + b * wd * cos) * decay,
            )
        } else {
            // Stable slow root avoids cancellation for very strong damping.
            let sum = d + (d * d - 1.0).sqrt();
            let r1 = -w / sum;
            let r2 = -w * sum;
            let a = (velocity - r2 * displacement) / (r1 - r2);
            let b = displacement - a;
            let e1 = (r1 * t).exp();
            let e2 = (r2 * t).exp();
            (a * e1 + b * e2, r1 * a * e1 + r2 * b * e2)
        }
    }
}

/// Motion roles correspond to component extent, rather than literal durations.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MotionSpeed {
    /// Small controls, such as switches and button shape changes.
    Fast,
    /// Partial-screen surfaces and ordinary movement.
    #[default]
    Default,
    /// Large or full-screen movement.
    Slow,
}

/// Shared spatial (position/size/shape) and effects (color/opacity) springs.
/// Fields are public so applications can customize individual motion roles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionScheme {
    /// Small-component movement.
    pub fast_spatial: Spring,
    /// Ordinary movement.
    pub default_spatial: Spring,
    /// Large-component movement.
    pub slow_spatial: Spring,
    /// Small-component color and opacity changes.
    pub fast_effects: Spring,
    /// Ordinary color and opacity changes.
    pub default_effects: Spring,
    /// Large-component color and opacity changes.
    pub slow_effects: Spring,
}
impl MotionScheme {
    /// Restrained Material motion. Both presets use springs.
    pub const fn standard() -> Self {
        Self {
            fast_spatial: Spring {
                stiffness: 1400.0,
                damping: 0.9,
            },
            default_spatial: Spring {
                stiffness: 700.0,
                damping: 0.9,
            },
            slow_spatial: Spring {
                stiffness: 300.0,
                damping: 0.9,
            },
            fast_effects: Spring {
                stiffness: 3800.0,
                damping: 1.0,
            },
            default_effects: Spring {
                stiffness: 1600.0,
                damping: 1.0,
            },
            slow_effects: Spring {
                stiffness: 800.0,
                damping: 1.0,
            },
        }
    }
    /// Material Expressive motion, with more spatial overshoot.
    pub const fn expressive() -> Self {
        Self {
            fast_spatial: Spring {
                stiffness: 800.0,
                damping: 0.6,
            },
            default_spatial: Spring {
                stiffness: 380.0,
                damping: 0.8,
            },
            slow_spatial: Spring {
                stiffness: 200.0,
                damping: 0.8,
            },
            ..Self::standard()
        }
    }
    /// Resolve a movement/shape role.
    pub const fn spatial(self, speed: MotionSpeed) -> Spring {
        match speed {
            MotionSpeed::Fast => self.fast_spatial,
            MotionSpeed::Default => self.default_spatial,
            MotionSpeed::Slow => self.slow_spatial,
        }
    }
    /// Resolve a color/opacity role. Preset effects never overshoot.
    pub const fn effects(self, speed: MotionSpeed) -> Spring {
        match speed {
            MotionSpeed::Fast => self.fast_effects,
            MotionSpeed::Default => self.default_effects,
            MotionSpeed::Slow => self.slow_effects,
        }
    }
}
impl Default for MotionScheme {
    fn default() -> Self {
        Self::standard()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analytic_springs_do_not_depend_on_frame_rate() {
        for damping in [0.6, 1.0, 1.8] {
            let spring = Spring::new(380., damping);
            let direct = spring.sample(-1., 0.7, 0.24);
            for frames in [3, 15, 60, 240] {
                let mut stepped = (-1., 0.7);
                for _ in 0..frames {
                    stepped = spring.sample(stepped.0, stepped.1, 0.24 / frames as f64);
                }
                assert!((direct.0 - stepped.0).abs() < 1e-10);
                assert!((direct.1 - stepped.1).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn expressive_spatial_overshoots_while_preset_effects_stay_bounded() {
        let spatial = MotionScheme::expressive().fast_spatial;
        assert!((1..500).any(|ms| spatial.sample(-1., 0., ms as f64 / 1000.).0 > 0.));
        for scheme in [MotionScheme::standard(), MotionScheme::expressive()] {
            for speed in [MotionSpeed::Fast, MotionSpeed::Default, MotionSpeed::Slow] {
                let mut last = -1.;
                for ms in 0..1000 {
                    let (offset, _) = scheme.effects(speed).sample(-1., 0., ms as f64 / 1000.);
                    assert!((-1. ..=0.).contains(&offset));
                    assert!(offset >= last);
                    last = offset;
                }
            }
        }
    }
}
