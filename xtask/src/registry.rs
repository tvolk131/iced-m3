//! Interpret the renderer's catalog as metadata without depending on iced.
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Example {
    pub id: String,
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub scale: u32,
    pub budget: usize,
    pub compare: bool,
    pub animated: bool,
}
impl Example {
    pub fn parts(&self) -> Vec<&'static str> {
        let mut parts = vec!["primary", "variants"];
        if self.compare {
            parts.push("comparison");
        }
        parts
    }
}
macro_rules! doc_examples {
    ($($id:ident {
        title: $title:literal, size: ($width:literal, $height:literal),
        scale: $scale:literal, budget: $budget:literal,
        compare: $compare:literal, animated: $animated:literal,
        variants_size: ($vw:literal, $vh:literal), caption: $caption:literal,
        view: $view:item, make: $make:expr, variants: $variants:expr,
    })*) => {
        pub fn catalog() -> Vec<Example> {
            vec![$(Example { id: stringify!($id).into(), title: $title.into(),
                width: $width as u32, height: $height as u32, scale: $scale,
                budget: $budget, compare: $compare, animated: $animated }),*]
        }
    };
}
include!("../../tests/visual/doc_media/catalog.rs");

pub fn validate(cases: &[Example]) -> Result<()> {
    ensure!(!cases.is_empty(), "Empty documentation registry");
    let mut ids = BTreeSet::new();
    for case in cases {
        ensure!(
            ids.insert(&case.id),
            "Duplicate documentation example: {}",
            case.id
        );
        ensure!(
            !case.compare || case.animated,
            "Static examples cannot have motion comparisons"
        );
        ensure!(
            case.width > 0 && case.height > 0 && case.scale > 0,
            "Invalid preview dimensions"
        );
    }
    Ok(())
}
