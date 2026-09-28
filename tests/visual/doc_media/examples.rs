//! Single registry: each view is both compiled Rust and the displayed rustdoc snippet.
use super::*;
use crate::*;

macro_rules! doc_examples {
    ($($id:ident {
        title: $title:literal, size: ($width:literal, $height:literal),
        scale: $scale:literal, budget: $budget:literal,
        compare: $compare:literal, animated: $animated:literal, variants_size: ($vw:literal, $vh:literal), caption: $caption:literal,
        view: $view:item,
        make: $make:expr,
        variants: $variants:expr,
    })*) => {
        pub(super) fn registry() -> Vec<Example> {
            vec![$({
                $view
                Example {
                    id: stringify!($id), title: $title,
                    size: Size::new($width, $height), scale: $scale,
                    budget: $budget, compare: $compare, animated: $animated, variants_size: Size::new($vw, $vh), caption: $caption,
                    source: stringify!($view), variant_source: stringify!($variants), make: $make, variants: $variants,
                }
            }),*]
        }
    };
}

include!("catalog.rs");
