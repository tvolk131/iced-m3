//! Component previews rendered from executable Rust examples.
//!
//! Start with the constructor for a primary example; these pages add appearance
//! variants and motion comparisons. Motion respects the reader's reduced-motion
//! preference. Standard and Expressive are both spring schemes; the comparison
//! uses identical input timing.
//!
//! In a source checkout, run `cargo xtask doc-media build` to generate the
//! illustrated reference. Plain `cargo doc` provides the text/API reference.
//! Published docs.rs builds automatically include the packaged previews.

macro_rules! page {
    ($id:ident, $title:literal, motion $(,)?) => {
        page!(@layout $id, $title, cfg_attr(iced_m3_doc_media, doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/generated/", stringify!($id), "-comparison.md"))));
    };
    ($id:ident, $title:literal $(,)?) => {
        page!(@layout $id, $title, doc = "");
    };
    // Shared layout: primary animation/code, appearance variants, optional motion.
    // Keep section ordering here so every showcase changes together.
    (@layout $id:ident, $title:literal, $comparison:meta) => {
        #[doc = $title]
        #[doc = ""]
        #[cfg_attr(iced_m3_doc_media, doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/generated/", stringify!($id), "-primary.md")))]
        #[doc = ""]
        #[cfg_attr(iced_m3_doc_media, doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/generated/", stringify!($id), "-variants.md")))]
        #[doc = ""]
        #[$comparison]
        pub mod $id {}
    };
}
page!(switch, "Switch states, icons, and motion schemes.", motion);
page!(button_group, "Spaced and connected action groups.", motion);
page!(
    text_field,
    "Floating labels, validation, and disabled fields."
);
page!(
    dialog,
    "Retained modal lifecycle with a fixed action footer."
);
page!(
    loading_indicator,
    "Canonical Expressive shape morphs and container variants."
);

page!(button, "Common button treatments and press feedback.");
page!(icon_button, "SVG icon actions and toggles.");
page!(fab, "Floating action button sizes and extended labels.");
page!(
    checkbox,
    "Checked, indeterminate, error, and disabled states."
);
page!(radio, "Exclusive selection and disabled options.");
page!(slider, "Dragging, stepped values, and value labels.");
page!(typography, "Typography examples and appearance variants.");
page!(surface, "Surface examples and appearance variants.");
page!(divider, "Divider examples and appearance variants.");
page!(badge, "Badges examples and appearance variants.");
page!(card, "Card examples and appearance variants.");
page!(
    rich_tooltip,
    "Rich tooltip examples and appearance variants."
);
page!(filter_chip, "Chips examples and appearance variants.");
page!(tabs, "Tabs examples and appearance variants.");
page!(list_item, "Lists examples and appearance variants.");
page!(menu, "Menu examples and appearance variants.");
page!(select, "Select examples and appearance variants.");
page!(tooltip, "Tooltip examples and appearance variants.");
page!(snackbar, "Snackbar examples and appearance variants.");
page!(app_bar, "App bar examples and appearance variants.");
page!(
    navigation_bar,
    "Navigation bar examples and appearance variants."
);
page!(
    navigation_rail,
    "Navigation rail examples and appearance variants."
);
page!(
    segmented_buttons,
    "Segmented buttons examples and appearance variants."
);
page!(
    range_slider,
    "Range slider examples and appearance variants."
);
page!(
    linear_progress,
    "Linear progress examples and appearance variants."
);
page!(
    circular_progress,
    "Circular progress examples and appearance variants."
);
page!(date_picker, "Date picker examples and appearance variants.");
page!(time_picker, "Time picker examples and appearance variants.");
page!(carousel, "Carousel examples and appearance variants.");
page!(side_sheet, "Sheets examples and appearance variants.");
page!(search_bar, "Search examples and appearance variants.");
page!(
    split_button,
    "Split button examples and appearance variants."
);
page!(toolbar, "Toolbar examples and appearance variants.");
page!(fab_menu, "FAB menu examples and appearance variants.");
page!(full_screen_dialog, "Full-screen dialog composition.");
