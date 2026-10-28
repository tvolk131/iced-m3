//! Optional, offline [Material Symbols](https://fonts.google.com/icons).
//!
//! Enable `material-symbols` for all **4,157** names in the pinned catalog:
//! Outlined and Rounded, both fills, and optical sizes 20, 24, 40 and 48.
//! Defaults: [`Style::Outlined`], weight **400**, [`OpticalSize::Size24`],
//! grade **0**, and [`Fill::Unfilled`]. Weight and grade are fixed.
//!
//! ```
//! use iced_m3::{icon, NavigationItem};
//! use iced_m3::material_symbols::{Fill, IconName, OpticalSize, Style, Symbol};
//!
//! let home = icon(IconName::Home);
//! let small = Symbol::new(IconName::Search)
//!     .style(Style::Rounded)
//!     .optical_size(OpticalSize::Size20)
//!     .icon().size(20);
//! let item: NavigationItem<'_, u8, ()> = NavigationItem::new(0, "Home", home)
//!     .selected_icon(Symbol::new(IconName::Home).fill(Fill::Filled));
//! ```
//!
//! [`Symbol`] uses the existing theme-aware [`Icon`] renderer, including disabled
//! alpha. [`Icon::size`] controls logical layout size independently of optical
//! size: choosing 20px artwork does not change the default 24px layout size.
//! Some symbols have identical filled and unfilled artwork.
//!
//! Original SVG bytes are embedded as independently compressed chunks of at most
//! 64 KiB. First use decodes one chunk in memory; handles retain their own bytes
//! and are cached for the process lifetime. A separate LRU cache retains at most
//! 16 decoded chunks (1 MiB). There are no downloads, filesystem extraction,
//! fonts, registration or ligatures. A local companion build script generates
//! typed names from bundled JSON without unpacking SVGs. Preserve Google's
//! [`LICENSE`] and [`NOTICE`] when redistributing these assets.

use crate::{Element, Icon};
use iced::advanced::svg::Handle;

#[doc(inline)]
pub use iced_m3_material_symbols::{Fill, IconName, LICENSE, NOTICE, OpticalSize, Style};

/// A typed symbol with configurable style, fill and optical size.
/// Converts into an [`Element`] or SVG handle; [`Self::icon`] exposes the shared
/// renderer's logical size. Weight is fixed at 400 and grade at 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Symbol {
    name: IconName,
    style: Style,
    fill: Fill,
    optical_size: OpticalSize,
}

impl Symbol {
    /// Outlined, weight 400, optical size 24, grade 0, unfilled.
    pub const fn new(name: IconName) -> Self {
        Self {
            name,
            style: Style::Outlined,
            fill: Fill::Unfilled,
            optical_size: OpticalSize::Size24,
        }
    }

    /// Choose an official design family, preserving the other coordinates.
    pub const fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Choose official unfilled or filled artwork, preserving the other coordinates.
    pub const fn fill(mut self, fill: Fill) -> Self {
        self.fill = fill;
        self
    }

    /// Choose official artwork at optical size 20, 24, 40 or 48.
    /// Set [`Icon::size`] separately to change the logical layout size.
    pub const fn optical_size(mut self, optical_size: OpticalSize) -> Self {
        self.optical_size = optical_size;
        self
    }

    /// Render at 24 logical pixels with the enclosing component's foreground.
    pub fn icon(self) -> Icon {
        crate::icon(self)
    }
}

impl From<Symbol> for Handle {
    fn from(symbol: Symbol) -> Self {
        symbol
            .name
            .handle(symbol.style, symbol.fill, symbol.optical_size)
    }
}

impl<'a, Message: 'a> From<Symbol> for Element<'a, Message> {
    fn from(symbol: Symbol) -> Self {
        symbol.icon().into()
    }
}
