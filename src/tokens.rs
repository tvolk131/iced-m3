//! Shared Material 3 measurements, in logical pixels.
use std::time::Duration;

/// Spacing on a four-pixel grid.
pub mod spacing {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 16.0;
    pub const LG: f32 = 24.0;
    pub const XL: f32 = 32.0;
    pub const XXL: f32 = 48.0;
}
/// Corner radii.
pub mod shape {
    pub const SMALL: f32 = 4.0;
    pub const MEDIUM: f32 = 12.0;
    pub const LARGE: f32 = 16.0;
    pub const EXTRA_LARGE: f32 = 28.0;
    pub const FULL: f32 = 999.0;
}
/// Component dimensions.
pub mod size {
    pub const BUTTON: f32 = 40.0;
    pub const FIELD: f32 = 56.0;
    pub const CHIP: f32 = 32.0;
    pub const CHECKBOX: f32 = 18.0;
    pub const CHECKBOX_TARGET: f32 = 48.0;
    pub const CHECKBOX_STATE_LAYER: f32 = 40.0;
    pub const SWITCH: f32 = 32.0;
    pub const SWITCH_WIDTH: f32 = 52.0;
    pub const TAB: f32 = 48.0;
    pub const TAB_WITH_ICON: f32 = 64.0;
    pub const APP_BAR: f32 = 64.0;
    pub const NAV_RAIL: f32 = 80.0;
    pub const NAV_INDICATOR_WIDTH: f32 = 56.0;
    pub const NAV_INDICATOR_HEIGHT: f32 = 32.0;
    pub const LIST_ONE_LINE: f32 = 56.0;
    pub const LIST_TWO_LINE: f32 = 72.0;
    pub const LIST_THREE_LINE: f32 = 88.0;
    pub const RADIO: f32 = 20.0;
    pub const MENU_ITEM: f32 = 48.0;
    pub const MENU_WIDTH: f32 = 240.0;
    pub const POPUP_MARGIN: f32 = 8.0;
    pub const SNACKBAR_MAX: f32 = 568.0;
    pub const DIALOG_MAX: f32 = 560.0;
    pub const OUTLINE: f32 = 1.0;
    pub const FOCUS_OUTLINE: f32 = 2.0;
    pub const SLIDER_TARGET: f32 = 48.0;
    pub const SLIDER_THUMB: f32 = 20.0;
    pub const SLIDER_TRACK: f32 = 4.0;
    pub const SLIDER_STATE_LAYER: f32 = 40.0;
    pub const SLIDER_LABEL: f32 = 28.0;
    pub const SLIDER_LABEL_SPACE: f32 = 32.0;
    pub const SEGMENT: f32 = 40.0;
    pub const SEGMENT_ICON: f32 = 18.0;
    pub const PROGRESS_LINEAR: f32 = 4.0;
    pub const PROGRESS_CIRCULAR: f32 = 48.0;
    pub const FAB_SMALL: f32 = 40.0;
    pub const FAB: f32 = 56.0;
    pub const FAB_LARGE: f32 = 96.0;
    pub const SIDE_SHEET: f32 = 360.0;
    pub const BOTTOM_SHEET_MAX: f32 = 640.0;
    pub const BOTTOM_SHEET_HEIGHT: f32 = 480.0;
}
/// Material 3 baseline type roles. The original short names remain aliases.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TypeScale {
    DisplayLarge,
    DisplayMedium,
    DisplaySmall,
    HeadlineLarge,
    HeadlineMedium,
    HeadlineSmall,
    TitleLarge,
    TitleMedium,
    TitleSmall,
    #[default]
    BodyLarge,
    BodyMedium,
    BodySmall,
    LabelLarge,
    LabelMedium,
    LabelSmall,
}
#[allow(non_upper_case_globals)]
impl TypeScale {
    pub const Display: Self = Self::DisplayMedium;
    pub const Headline: Self = Self::HeadlineMedium;
    pub const Title: Self = Self::TitleLarge;
    pub const Body: Self = Self::BodyLarge;
    pub const Label: Self = Self::LabelLarge;
    pub const Supporting: Self = Self::BodySmall;

    pub const fn metrics(self) -> (f32, f32) {
        match self {
            Self::DisplayLarge => (57.0, 64.0),
            Self::DisplayMedium => (45.0, 52.0),
            Self::DisplaySmall => (36.0, 44.0),
            Self::HeadlineLarge => (32.0, 40.0),
            Self::HeadlineMedium => (28.0, 36.0),
            Self::HeadlineSmall => (24.0, 32.0),
            Self::TitleLarge => (22.0, 28.0),
            Self::TitleMedium | Self::BodyLarge => (16.0, 24.0),
            Self::TitleSmall | Self::BodyMedium | Self::LabelLarge => (14.0, 20.0),
            Self::BodySmall | Self::LabelMedium => (12.0, 16.0),
            Self::LabelSmall => (11.0, 16.0),
        }
    }

    pub const fn font(self) -> iced::Font {
        match self {
            Self::TitleMedium
            | Self::TitleSmall
            | Self::LabelLarge
            | Self::LabelMedium
            | Self::LabelSmall => crate::fonts::MEDIUM,
            _ => crate::fonts::REGULAR,
        }
    }

    /// M3 tracking in logical pixels. Exposed for reference/custom renderers;
    /// iced 0.14's native Text/TextInput APIs cannot apply letter spacing.
    pub const fn tracking(self) -> f32 {
        match self {
            Self::DisplayLarge => -0.25,
            Self::TitleMedium => 0.15,
            Self::TitleSmall | Self::LabelLarge => 0.1,
            Self::BodyLarge | Self::LabelMedium | Self::LabelSmall => 0.5,
            Self::BodyMedium => 0.25,
            Self::BodySmall => 0.4,
            _ => 0.0,
        }
    }
}
/// Finite interaction timings. A zero duration is also supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Motion {
    /// Search view expansion and collapse, using standard easing.
    pub search_enter: Duration,
    pub search_exit: Duration,
    /// Finite sheet entrance and exit durations.
    pub sheet_enter: Duration,
    pub sheet_exit: Duration,
    /// Anchored menu reveal and dismissal. Surface geometry remains unscaled.
    pub menu_enter: Duration,
    pub menu_exit: Duration,
    pub dialog_enter: Duration,
    pub dialog_exit: Duration,
    pub tooltip_enter: Duration,
    pub tooltip_exit: Duration,
    pub snackbar_enter: Duration,
    pub snackbar_exit: Duration,
    /// Slider value-label scale transition (Material short2).
    pub slider_label: Duration,
    pub short: Duration,
    pub medium: Duration,
    pub checkbox_select: Duration,
    /// Checkbox/radio press expansion. Quick presses remain visible for at
    /// least half this duration, then fade using `short`.
    pub ripple_expand: Duration,
}
impl Default for Motion {
    fn default() -> Self {
        Self {
            search_enter: Duration::from_millis(300),
            search_exit: Duration::from_millis(250),
            sheet_enter: Duration::from_millis(300),
            sheet_exit: Duration::from_millis(200),
            menu_enter: Duration::from_millis(500),
            menu_exit: Duration::from_millis(150),
            dialog_enter: Duration::from_millis(500),
            dialog_exit: Duration::from_millis(150),
            tooltip_enter: Duration::from_millis(150),
            tooltip_exit: Duration::from_millis(100),
            snackbar_enter: Duration::from_millis(250),
            snackbar_exit: Duration::from_millis(200),
            slider_label: Duration::from_millis(100),
            short: Duration::from_millis(150),
            medium: Duration::from_millis(200),
            checkbox_select: Duration::from_millis(350),
            ripple_expand: Duration::from_millis(450),
        }
    }
}

impl Motion {
    pub const fn reduced() -> Self {
        Self {
            search_enter: Duration::ZERO,
            search_exit: Duration::ZERO,
            sheet_enter: Duration::ZERO,
            sheet_exit: Duration::ZERO,
            menu_enter: Duration::ZERO,
            menu_exit: Duration::ZERO,
            dialog_enter: Duration::ZERO,
            dialog_exit: Duration::ZERO,
            tooltip_enter: Duration::ZERO,
            tooltip_exit: Duration::ZERO,
            snackbar_enter: Duration::ZERO,
            snackbar_exit: Duration::ZERO,
            slider_label: Duration::ZERO,
            short: Duration::ZERO,
            medium: Duration::ZERO,
            checkbox_select: Duration::ZERO,
            ripple_expand: Duration::ZERO,
        }
    }
}
