//! Official, offline Material Symbols for iced-m3.
//!
//! All 4,157 pinned names, Outlined/Rounded, both fills, optical sizes 20/24/40/48,
//! fixed weight 400 and grade 0. The original SVG bytes are losslessly compressed
//! and decoded in memory on demand. A local build script generates typed names
//! from the bundled JSON inventory; it performs no downloads or SVG extraction.
//! Enable iced-m3's `material-symbols` feature for the theme-aware Icon renderer.

use iced_core::svg::Handle;
mod storage;

/// Packed SVG bytes. The format is documented in `assets/README.md`.
#[doc(hidden)]
pub const BUNDLE: &[u8] = include_bytes!("../assets/symbols.bin");
/// Apache 2.0 license for the official Google assets.
pub const LICENSE: &str = include_str!("../assets/LICENSE");
/// Google attribution, bundled coordinates and pinned upstream revision.
pub const NOTICE: &str = include_str!("../assets/NOTICE");

/// Official Material Symbols design family. The default is Outlined.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Style {
    #[default]
    Outlined,
    Rounded,
}

/// Official `FILL` coordinate. Filled variants are useful for selected items.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Fill {
    /// `FILL=0` (the default).
    #[default]
    Unfilled,
    /// `FILL=1`.
    Filled,
}

/// Optical size selects official artwork designed for a particular logical size.
/// It is independent of display density and the renderer's logical size. No interpolation is
/// performed; weight remains 400 and grade remains 0.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpticalSize {
    Size20,
    #[default]
    Size24,
    Size40,
    Size48,
}

impl OpticalSize {
    /// Intended logical size of the artwork, in pixels.
    pub const fn pixels(self) -> u16 {
        match self {
            Self::Size20 => 20,
            Self::Size24 => 24,
            Self::Size40 => 40,
            Self::Size48 => 48,
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::Size20 => 0,
            Self::Size24 => 1,
            Self::Size40 => 2,
            Self::Size48 => 3,
        }
    }
}

macro_rules! catalog {
    ($($variant:ident => $name:literal),+ $(,)?) => {
        /// All names in the pinned official catalog. See [`Self::ALL`] for membership.
        /// Snake-case names become PascalCase; names starting with a digit gain
        /// an `Icon` prefix (for example, `10k` becomes `Icon10k`).
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[non_exhaustive]
        #[repr(u16)]
        pub enum IconName {
            $(#[doc = $name] $variant),+
        }

        impl IconName {
            /// Every bundled name, in alphabetical order by its upstream name.
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];
            const NAMES: &'static [&'static str] = &[$($name),+];

            /// Google's original snake_case name.
            pub const fn name(self) -> &'static str {
                Self::NAMES[self as usize]
            }

            /// Look up an exact upstream snake_case name.
            pub fn from_name(name: &str) -> Option<Self> {
                Self::NAMES.binary_search(&name).ok().map(|i| Self::ALL[i])
            }
        }
    };
}

include!(concat!(env!("OUT_DIR"), "/catalog.rs"));

impl IconName {
    /// A cached iced SVG handle for these official coordinates.
    /// Handles own their bytes and survive eviction from the 1 MiB chunk cache.
    pub fn handle(self, style: Style, fill: Fill, optical_size: OpticalSize) -> Handle {
        let style = match style {
            Style::Outlined => 0,
            Style::Rounded => 1,
        };
        let fill = match fill {
            Fill::Unfilled => 0,
            Fill::Filled => 1,
        };
        storage::handle(self as usize * 16 + style * 8 + fill * 4 + optical_size.index())
    }
}

impl From<IconName> for Handle {
    fn from(name: IconName) -> Self {
        name.handle(Style::default(), Fill::default(), OpticalSize::default())
    }
}
