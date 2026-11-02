//! Rounded Material shape roles, independent of component size and density.
use iced::border::Radius;

/// Semantic corner-size roles. Circular artwork does not use this scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeScale {
    None,
    ExtraSmall,
    Small,
    Medium,
    Large,
    LargeIncreased,
    ExtraLarge,
    ExtraLargeIncreased,
    ExtraExtraLarge,
    Full,
}

/// Theme-owned rounded corners in logical pixels. None and Full retain their
/// structural meanings; other roles may be customized independently.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapeScheme {
    pub extra_small: f32,
    pub small: f32,
    pub medium: f32,
    pub large: f32,
    pub large_increased: f32,
    pub extra_large: f32,
    pub extra_large_increased: f32,
    pub extra_extra_large: f32,
}

impl Default for ShapeScheme {
    fn default() -> Self {
        Self {
            extra_small: 4.0,
            small: 8.0,
            medium: 12.0,
            large: 16.0,
            large_increased: 20.0,
            extra_large: 28.0,
            extra_large_increased: 32.0,
            extra_extra_large: 48.0,
        }
    }
}

impl ShapeScheme {
    pub(crate) fn mixed(self, roles: [ShapeScale; 4]) -> Radius {
        Radius {
            top_left: self.radius(roles[0]),
            top_right: self.radius(roles[1]),
            bottom_right: self.radius(roles[2]),
            bottom_left: self.radius(roles[3]),
        }
    }
    pub fn radius(self, role: ShapeScale) -> f32 {
        let value = match role {
            ShapeScale::None => return 0.0,
            ShapeScale::ExtraSmall => self.extra_small,
            ShapeScale::Small => self.small,
            ShapeScale::Medium => self.medium,
            ShapeScale::Large => self.large,
            ShapeScale::LargeIncreased => self.large_increased,
            ShapeScale::ExtraLarge => self.extra_large,
            ShapeScale::ExtraLargeIncreased => self.extra_large_increased,
            ShapeScale::ExtraExtraLarge => self.extra_extra_large,
            ShapeScale::Full => return crate::tokens::shape::FULL,
        };
        if value.is_finite() && value >= 0.0 {
            value
        } else {
            Self::default().radius(role)
        }
    }

    pub fn corners(self, role: ShapeScale) -> Radius {
        self.radius(role).into()
    }

    pub fn top(self, role: ShapeScale) -> Radius {
        let radius = self.radius(role);
        Radius {
            top_left: radius,
            top_right: radius,
            bottom_left: 0.0,
            bottom_right: 0.0,
        }
    }

    /// Physical left corners; layout direction is not inferred.
    pub fn left(self, role: ShapeScale) -> Radius {
        let radius = self.radius(role);
        Radius {
            top_left: radius,
            bottom_left: radius,
            top_right: 0.0,
            bottom_right: 0.0,
        }
    }

    /// Physical right corners; layout direction is not inferred.
    pub fn right(self, role: ShapeScale) -> Radius {
        let radius = self.radius(role);
        Radius {
            top_right: radius,
            bottom_right: radius,
            top_left: 0.0,
            bottom_left: 0.0,
        }
    }
}
