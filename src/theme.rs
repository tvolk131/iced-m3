//! Semantic colors, shared motion, and iced widget catalogs.
use crate::tokens::Motion;
use iced::{Color, Shadow, Vector, theme, widget};

/// Semantic roles. Custom palettes can be supplied by editing `Theme::colors`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorScheme {
    pub primary: Color,
    pub on_primary: Color,
    pub primary_container: Color,
    pub on_primary_container: Color,
    pub secondary_container: Color,
    pub on_secondary_container: Color,
    pub tertiary_container: Color,
    pub on_tertiary_container: Color,
    pub surface: Color,
    pub surface_variant: Color,
    pub surface_container: Color,
    pub surface_container_low: Color,
    pub surface_container_high: Color,
    pub surface_container_highest: Color,
    pub on_surface: Color,
    pub on_surface_variant: Color,
    pub outline: Color,
    pub outline_variant: Color,
    pub error: Color,
    pub on_error: Color,
    pub error_container: Color,
    pub on_error_container: Color,
    pub inverse_surface: Color,
    pub inverse_on_surface: Color,
    pub inverse_primary: Color,
    pub secondary: Color,
    pub on_secondary: Color,
    pub tertiary: Color,
    pub on_tertiary: Color,
    pub surface_dim: Color,
    pub surface_bright: Color,
    pub surface_container_lowest: Color,
    pub background: Color,
    pub on_background: Color,
    pub surface_tint: Color,
    pub shadow: Color,
    pub primary_fixed: Color,
    pub primary_fixed_dim: Color,
    pub secondary_fixed: Color,
    pub secondary_fixed_dim: Color,
    pub tertiary_fixed: Color,
    pub tertiary_fixed_dim: Color,
    pub on_primary_fixed: Color,
    pub on_primary_fixed_variant: Color,
    pub on_secondary_fixed: Color,
    pub on_secondary_fixed_variant: Color,
    pub on_tertiary_fixed: Color,
    pub on_tertiary_fixed_variant: Color,
    /// Opaque semantic scrim color; modal hosts apply their 32% state opacity.
    pub scrim: Color,
}

/// A light or dark Material theme with configurable accent and semantic roles.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    pub colors: ColorScheme,
    pub motion: Motion,
    dark: bool,
    pub(crate) shadows: bool,
}
impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}
impl Theme {
    pub fn light() -> Self {
        Self::from_accent(Color::from_rgb8(103, 80, 164), false)
    }
    pub fn dark() -> Self {
        Self::from_accent(Color::from_rgb8(103, 80, 164), true)
    }
    /// Generate Material HCT tonal-spot colors using the pinned Rust port of
    /// material-color-utilities. Schemes are cached so view rebuilding is cheap.
    pub fn from_accent(accent: Color, dark: bool) -> Self {
        Self::from_accent_with_contrast(accent, dark, 0.0)
    }
    /// Contrast ranges from -1 (reduced) through 0 (standard) to 1 (maximum).
    pub fn from_accent_with_contrast(accent: Color, dark: bool, contrast: f64) -> Self {
        use material_colors::{color::Argb, scheme::variant::SchemeTonalSpot};
        type Key = (u32, bool, u64);
        static CACHE: std::sync::OnceLock<
            std::sync::Mutex<std::collections::VecDeque<(Key, ColorScheme)>>,
        > = std::sync::OnceLock::new();
        let channel = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        let argb = Argb {
            alpha: 255,
            red: channel(accent.r),
            green: channel(accent.g),
            blue: channel(accent.b),
        };
        let contrast = if contrast.is_finite() {
            contrast.clamp(-1.0, 1.0)
        } else {
            0.0
        };
        let key = (
            (argb.red as u32) << 16 | (argb.green as u32) << 8 | argb.blue as u32,
            dark,
            contrast.to_bits(),
        );
        let cache = CACHE.get_or_init(Default::default);
        if let Some((_, colors)) = cache
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .iter()
            .find(|(k, _)| *k == key)
        {
            return Self {
                colors: *colors,
                dark,
                motion: Motion::default(),
                shadows: true,
            };
        }
        let scheme = SchemeTonalSpot::new(argb.into(), dark, Some(contrast)).scheme;
        let rgb = |color: Argb| Color::from_rgb8(color.red, color.green, color.blue);
        let colors = ColorScheme {
            primary: rgb(scheme.primary()),
            on_primary: rgb(scheme.on_primary()),
            primary_container: rgb(scheme.primary_container()),
            on_primary_container: rgb(scheme.on_primary_container()),
            secondary_container: rgb(scheme.secondary_container()),
            on_secondary_container: rgb(scheme.on_secondary_container()),
            tertiary_container: rgb(scheme.tertiary_container()),
            on_tertiary_container: rgb(scheme.on_tertiary_container()),
            surface: rgb(scheme.surface()),
            surface_variant: rgb(scheme.surface_variant()),
            surface_container: rgb(scheme.surface_container()),
            surface_container_low: rgb(scheme.surface_container_low()),
            surface_container_high: rgb(scheme.surface_container_high()),
            surface_container_highest: rgb(scheme.surface_container_highest()),
            on_surface: rgb(scheme.on_surface()),
            on_surface_variant: rgb(scheme.on_surface_variant()),
            outline: rgb(scheme.outline()),
            outline_variant: rgb(scheme.outline_variant()),
            error: rgb(scheme.error()),
            on_error: rgb(scheme.on_error()),
            error_container: rgb(scheme.error_container()),
            on_error_container: rgb(scheme.on_error_container()),
            inverse_surface: rgb(scheme.inverse_surface()),
            inverse_on_surface: rgb(scheme.inverse_on_surface()),
            inverse_primary: rgb(scheme.inverse_primary()),
            secondary: rgb(scheme.secondary()),
            on_secondary: rgb(scheme.on_secondary()),
            tertiary: rgb(scheme.tertiary()),
            on_tertiary: rgb(scheme.on_tertiary()),
            surface_dim: rgb(scheme.surface_dim()),
            surface_bright: rgb(scheme.surface_bright()),
            surface_container_lowest: rgb(scheme.surface_container_lowest()),
            background: rgb(scheme.background()),
            on_background: rgb(scheme.on_background()),
            surface_tint: rgb(scheme.surface_tint()),
            shadow: rgb(scheme.shadow()),
            primary_fixed: rgb(scheme.primary_fixed()),
            primary_fixed_dim: rgb(scheme.primary_fixed_dim()),
            secondary_fixed: rgb(scheme.secondary_fixed()),
            secondary_fixed_dim: rgb(scheme.secondary_fixed_dim()),
            tertiary_fixed: rgb(scheme.tertiary_fixed()),
            tertiary_fixed_dim: rgb(scheme.tertiary_fixed_dim()),
            on_primary_fixed: rgb(scheme.on_primary_fixed()),
            on_primary_fixed_variant: rgb(scheme.on_primary_fixed_variant()),
            on_secondary_fixed: rgb(scheme.on_secondary_fixed()),
            on_secondary_fixed_variant: rgb(scheme.on_secondary_fixed_variant()),
            on_tertiary_fixed: rgb(scheme.on_tertiary_fixed()),
            on_tertiary_fixed_variant: rgb(scheme.on_tertiary_fixed_variant()),
            scrim: rgb(scheme.scrim()),
        };
        let mut cache = cache.lock().unwrap_or_else(|p| p.into_inner());
        cache.push_front((key, colors));
        cache.truncate(32);
        Self {
            colors,
            dark,
            motion: Motion::default(),
            shadows: true,
        }
    }
    /// Apply the application's reduced-motion preference to all shared animations.
    pub fn reduced_motion(mut self, reduced: bool) -> Self {
        self.motion = if reduced {
            Motion::reduced()
        } else {
            Motion::default()
        };
        self
    }
    pub fn is_dark(&self) -> bool {
        self.dark
    }
    /// An iced theme for widgets outside this library's catalogs.
    pub fn iced(&self) -> iced::Theme {
        iced::Theme::custom("Material", self.palette_value())
    }
    fn palette_value(&self) -> theme::Palette {
        theme::Palette {
            background: self.colors.surface,
            text: self.colors.on_surface,
            primary: self.colors.primary,
            success: self.colors.primary,
            warning: self.colors.error,
            danger: self.colors.error,
        }
    }
    /// Key-shadow fallback for native iced containers with a single shadow slot.
    /// Use `elevated` for both Material layers, including ambient spread.
    pub fn elevation(&self, level: u8) -> Shadow {
        let layer = crate::elevation::layers(f32::from(level))[0];
        Shadow {
            color: alpha(
                self.colors.shadow,
                if self.shadows { layer.opacity } else { 0. },
            ),
            offset: Vector::new(0., layer.y),
            blur_radius: layer.blur,
        }
    }
}
impl theme::Base for Theme {
    fn default(preference: theme::Mode) -> Self {
        if preference == theme::Mode::Dark {
            Self::dark()
        } else {
            Self::light()
        }
    }
    fn mode(&self) -> theme::Mode {
        if self.dark {
            theme::Mode::Dark
        } else {
            theme::Mode::Light
        }
    }
    fn base(&self) -> theme::Style {
        theme::Style {
            background_color: self.colors.surface,
            text_color: self.colors.on_surface,
        }
    }
    fn palette(&self) -> Option<theme::Palette> {
        Some(self.palette_value())
    }
    fn name(&self) -> &str {
        if self.dark {
            "Material Dark"
        } else {
            "Material Light"
        }
    }
}

pub(crate) fn alpha(mut color: Color, opacity: f32) -> Color {
    color.a = opacity;
    color
}
pub(crate) fn mix(a: Color, b: Color, t: f32) -> Color {
    Color::from_rgba(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}
impl widget::text::Catalog for Theme {
    type Class<'a> = widget::text::StyleFn<'a, Self>;
    fn default<'a>() -> Self::Class<'a> {
        Box::new(|_| widget::text::Style::default())
    }
    fn style(&self, class: &Self::Class<'_>) -> widget::text::Style {
        class(self)
    }
}
impl widget::container::Catalog for Theme {
    type Class<'a> = widget::container::StyleFn<'a, Self>;
    fn default<'a>() -> Self::Class<'a> {
        Box::new(|_| widget::container::Style::default())
    }
    fn style(&self, class: &Self::Class<'_>) -> widget::container::Style {
        class(self)
    }
}
impl widget::text_input::Catalog for Theme {
    type Class<'a> = widget::text_input::StyleFn<'a, Self>;
    fn default<'a>() -> Self::Class<'a> {
        Box::new(|t, status| widget::text_input::default(&t.iced(), status))
    }
    fn style(
        &self,
        class: &Self::Class<'_>,
        status: widget::text_input::Status,
    ) -> widget::text_input::Style {
        class(self, status)
    }
}
impl widget::scrollable::Catalog for Theme {
    type Class<'a> = widget::scrollable::StyleFn<'a, Self>;
    fn default<'a>() -> Self::Class<'a> {
        Box::new(|t, status| widget::scrollable::default(&t.iced(), status))
    }
    fn style(
        &self,
        class: &Self::Class<'_>,
        status: widget::scrollable::Status,
    ) -> widget::scrollable::Style {
        class(self, status)
    }
}

impl widget::svg::Catalog for Theme {
    type Class<'a> = widget::svg::StyleFn<'a, Self>;
    fn default<'a>() -> Self::Class<'a> {
        Box::new(|_, _| widget::svg::Style::default())
    }
    fn style(&self, class: &Self::Class<'_>, status: widget::svg::Status) -> widget::svg::Style {
        class(self, status)
    }
}

#[cfg(test)]
fn luminance(c: Color) -> f32 {
    let linear = |v: f32| {
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(c.r) + 0.7152 * linear(c.g) + 0.0722 * linear(c.b)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_roles_retain_their_tones_across_modes_and_surfaces_follow_elevation() {
        let light = Theme::light().colors;
        let dark = Theme::dark().colors;
        for (a, b) in [
            (light.primary_fixed, dark.primary_fixed),
            (light.primary_fixed_dim, dark.primary_fixed_dim),
            (light.on_primary_fixed, dark.on_primary_fixed),
            (
                light.on_primary_fixed_variant,
                dark.on_primary_fixed_variant,
            ),
            (light.secondary_fixed, dark.secondary_fixed),
            (light.secondary_fixed_dim, dark.secondary_fixed_dim),
            (light.on_secondary_fixed, dark.on_secondary_fixed),
            (
                light.on_secondary_fixed_variant,
                dark.on_secondary_fixed_variant,
            ),
            (light.tertiary_fixed, dark.tertiary_fixed),
            (light.tertiary_fixed_dim, dark.tertiary_fixed_dim),
            (light.on_tertiary_fixed, dark.on_tertiary_fixed),
            (
                light.on_tertiary_fixed_variant,
                dark.on_tertiary_fixed_variant,
            ),
        ] {
            assert_eq!(a, b);
        }
        for c in [light, dark] {
            assert_eq!(c.background, c.surface);
            assert_eq!(c.on_background, c.on_surface);
            assert_eq!(c.surface_tint, c.primary);
            assert_eq!(c.shadow, Color::BLACK);
            assert_eq!(c.scrim, Color::BLACK);
            assert!(luminance(c.surface_dim) < luminance(c.surface_bright));
            for pair in [
                (c.primary_fixed, c.primary_fixed_dim),
                (c.secondary_fixed, c.secondary_fixed_dim),
                (c.tertiary_fixed, c.tertiary_fixed_dim),
            ] {
                assert!(luminance(pair.0) > luminance(pair.1));
            }
        }
        for (c, is_dark) in [(light, false), (dark, true)] {
            let surfaces = [
                c.surface_container_lowest,
                c.surface_container_low,
                c.surface_container,
                c.surface_container_high,
                c.surface_container_highest,
            ];
            for pair in surfaces.windows(2) {
                assert_eq!(luminance(pair[0]) < luminance(pair[1]), is_dark);
            }
        }
    }
    #[test]
    fn maximum_contrast_increases_primary_legibility() {
        let ratio = |c: ColorScheme| {
            let (a, b) = (luminance(c.primary), luminance(c.on_primary));
            (a.max(b) + 0.05) / (a.min(b) + 0.05)
        };
        for dark in [false, true] {
            let accent = Color::from_rgb8(103, 80, 164);
            assert!(
                ratio(Theme::from_accent_with_contrast(accent, dark, 1.0).colors)
                    > ratio(Theme::from_accent(accent, dark).colors)
            );
        }
    }
    #[test]
    fn arbitrary_accents_keep_readable_role_pairs() {
        for accent in [
            Color::BLACK,
            Color::WHITE,
            Color::from_rgb(1.0, 0.0, 0.0),
            Color::from_rgb(0.0, 1.0, 0.0),
            Color::from_rgb(0.0, 0.0, 1.0),
        ] {
            for dark in [false, true] {
                let c = Theme::from_accent(accent, dark).colors;
                for (a, b) in [
                    (c.primary, c.on_primary),
                    (c.secondary, c.on_secondary),
                    (c.tertiary, c.on_tertiary),
                    (c.primary_fixed, c.on_primary_fixed_variant),
                    (c.secondary_fixed, c.on_secondary_fixed_variant),
                    (c.tertiary_fixed, c.on_tertiary_fixed_variant),
                    (c.inverse_surface, c.inverse_on_surface),
                    (c.inverse_surface, c.inverse_primary),
                    (c.surface, c.on_surface),
                    (c.primary_container, c.on_primary_container),
                    (c.secondary_container, c.on_secondary_container),
                    (c.tertiary_container, c.on_tertiary_container),
                    (c.surface_container_low, c.on_surface),
                    (c.surface_container, c.on_surface_variant),
                ] {
                    let (a, b) = (luminance(a), luminance(b));
                    assert!((a.max(b) + 0.05) / (a.min(b) + 0.05) >= 4.5);
                }
            }
        }
    }
}
