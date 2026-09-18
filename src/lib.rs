//! Material 3 components for desktop applications built with upstream iced 0.14.
//!
//! Compose these controls with ordinary iced layouts. Your application owns values
//! and messages; widgets retain editing, interaction and animation state in iced's
//! persistent widget tree. No application animation subscription is needed.
//!
//! # Start here
//!
//! - [Getting started](guide::getting_started): dependency setup and a complete app.
//! - [Cookbook](guide::cookbook): selection, navigation, dialogs, sheets and feedback.
//! - [Theming](guide::theming): colors, fonts and mixing native iced widgets.
//!
//! Return this crate's [Element] from your view, supply a [Theme] from the
//! application's theme callback, and set [fonts::REGULAR] as the default font.
//! Wrap the view in [focus::scope] for keyboard traversal and activation. Missing
//! callbacks disable controls; store callback values in your model and rebuild.
//! Keep overlay hosts mounted when you want retained focus and exit animations.
//!
//! # Find a component
//!
//! | Area | APIs |
//! | --- | --- |
//! | Actions | [button](mod@button), [icon_button], [fab](mod@fab), [chip](mod@chip), [segmented] |
//! | Input | [text_field](mod@text_field), [checkbox](mod@checkbox), [switch], [radio](mod@radio), [menu::select], [slider](mod@slider), [range_slider](mod@range_slider), [date_picker](mod@date_picker), [time_picker](mod@time_picker) |
//! | Navigation | [app_bar](mod@app_bar), [tabs](mod@tabs), [navigation], [search], [menu](mod@menu) |
//! | Content | [surface], [variants::card], [list](mod@list), [carousel](mod@carousel), [typography], [divider] |
//! | Feedback | [dialog](mod@dialog), [sheet], [snackbar](mod@snackbar), [tooltip](mod@tooltip), [badge], [progress], [loading] |
//! | Foundations | [theme], [tokens], [fonts], [focus], [elevation] |
//! | Expressive previews | [expressive] |
//!
//! # Compatibility and scope
//!
//! This is a desktop beta with an evolving API, requiring Rust 1.88+
//! and released iced 0.14.0. The default `wgpu` feature enables the GPU renderer
//! with software fallback; disable default features for a software-only build.
//! Cargo may still enable GPU support through another dependency's features.
//!
//! CI builds and tests on macOS, Windows and Linux. Native macOS behavior has
//! also been checked locally. Keyboard support and reduced motion are included;
//! native screen-reader integration, full localization/RTL, arbitrary-child rounded
//! clipping/group opacity, and some Expressive variants remain unfinished.
//! Full M3 conformance is not claimed.
//! Pin beta versions exactly and review the
//! [release notes and compatibility policy](https://github.com/tvolk131/iced-m3/blob/master/CHANGELOG.md)
//! when upgrading. Native Windows/Linux interaction remains unverified.
//!
//! Rust code is MIT licensed. Preserve the bundled font and loading-asset notices
//! when redistributing them: [fonts::LICENSE], [loading::LICENSE], [loading::NOTICE].

// Compile the consumer-facing README example even though the API homepage has
// its own introduction. This module adds no production API or runtime code.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme_examples {}

mod activity;
mod anchored;
pub mod app_bar;
pub mod button;
pub mod carousel;
pub mod checkbox;
pub mod chip;
pub mod components;
pub mod date_picker;
pub mod dialog;
pub mod elevation;
pub mod expressive;
pub use elevation::elevated;
pub mod fab;
pub mod focus;
pub mod fonts;
mod glyph;
pub mod guide;
pub mod icon;
pub use icon::{Icon, icon};
pub mod list;
pub mod loading;
pub mod menu;
mod motion;
pub mod navigation;
mod presence;
pub mod progress;
pub mod radio;
mod rail_motion;
pub mod range_slider;
mod reveal;
mod ripple;
pub mod search;
pub mod segmented;
pub mod sheet;
pub mod slider;
pub mod snackbar;
mod staged;
pub mod tabs;
pub mod text_field;
pub mod theme;
pub mod time_picker;
pub mod tokens;
pub mod tooltip;
pub mod variants;
#[cfg(test)]
#[path = "../tests/visual/mod.rs"]
mod visual_tests;

pub use app_bar::{AppBar, AppBarVariant, app_bar};
pub use fab::{Fab, FabColor, FabSize, extended_fab, fab};
pub use list::{ListItem, list, list_item};
pub use navigation::{
    NavigationBar, NavigationItem, NavigationLayout, NavigationRail, adaptive_navigation,
    modal_navigation_rail, navigation_bar, navigation_rail,
};
pub use progress::{Progress, circular_progress, linear_progress};
pub use range_slider::{RangeSlider, range_slider};
pub use search::{Search, search_bar};
pub use segmented::{Segment, SegmentSelection, SegmentedButtons, segmented_buttons};
pub use sheet::{Sheet, bottom_sheet, side_sheet};
pub use slider::{Slider, slider};
pub use tabs::{Tab, TabVariant, Tabs, tabs};

pub use menu::{
    Menu, MenuItem, Placement, Select, SelectOption, context_menu, menu, menu_button, select,
};
pub use radio::{Radio, RadioGroup, RadioOption, radio, radio_group};
pub use snackbar::{Snackbar, snackbar};
pub use tooltip::{Tooltip, tooltip};

pub use button::{Button, ButtonVariant, button};
pub use checkbox::{Checkbox, checkbox};
pub use chip::{Chip, ChipVariant, assist_chip, filter_chip, input_chip, suggestion_chip};
pub use components::{
    Surface, SurfaceVariant, Switch, badge, chip, divider, icon_button, surface, switch, typography,
};
pub use dialog::{Dialog, FullScreenDialog, dialog, full_screen_dialog};
pub use text_field::{TextField, TextFieldVariant, text_field};
pub use theme::{ColorScheme, Theme};
pub use tokens::TypeScale;

/// An ordinary iced element using the Material theme and iced's default renderer.
pub type Element<'a, Message> = iced::Element<'a, Message, Theme>;

pub use date_picker::{
    Date, DatePicker, DateSelection, InvalidDate, date_input, date_picker, docked_date_picker,
};

pub use time_picker::{InvalidTime, Time, TimePart, TimePicker, time_input, time_picker};

pub use variants::{Card, badge_dot, badged, card, rich_tooltip, vertical_divider};

pub use carousel::{Carousel, CarouselVariant, carousel, lazy_carousel};

pub use expressive::{
    ButtonGroup, FabMenuItem, Toolbar, button_group, fab_menu, split_button, toolbar,
};

pub use loading::{LoadingIndicator, loading_indicator};

#[cfg(test)]
#[path = "../tests/performance/mod.rs"]
mod performance;
