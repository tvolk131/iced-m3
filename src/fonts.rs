//! Bundled Roboto typography. Material components register this font once using
//! iced's shared font system; applications need no font-loading messages.
use iced::{Font, font::Weight};
use std::{borrow::Cow, sync::Once};

pub const REGULAR: Font = Font::with_name("Roboto");
pub const MEDIUM: Font = Font {
    weight: Weight::Medium,
    ..REGULAR
};
/// Unmodified Google Fonts Roboto, including the variable weight axis.
pub const ROBOTO: &[u8] = include_bytes!("../assets/fonts/Roboto-Variable.ttf");
/// Redistribution license for the embedded font.
pub const LICENSE: &str = include_str!("../assets/fonts/OFL.txt");

pub(crate) fn ensure_loaded() {
    static LOAD: Once = Once::new();
    LOAD.call_once(|| {
        iced::advanced::graphics::text::font_system()
            .write()
            .expect("font system lock")
            .load_font(Cow::Borrowed(ROBOTO));
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn roboto_is_registered_once_in_the_renderer_font_database() {
        super::ensure_loaded();
        let version = iced::advanced::graphics::text::font_system()
            .read()
            .unwrap()
            .version();
        super::ensure_loaded();
        let mut system = iced::advanced::graphics::text::font_system()
            .write()
            .unwrap();
        assert_eq!(version, system.version());
        assert!(
            system
                .raw()
                .db()
                .faces()
                .any(|face| face.families.iter().any(|(name, _)| name == "Roboto"))
        );
    }
}
