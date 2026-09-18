use crate::{Element, Theme};
use iced::advanced::{Layout, Widget, layout, renderer, svg::Handle, widget::Tree};
use iced::{Length, Rectangle, Renderer, Size, mouse};
use std::sync::OnceLock;

/// Centered artwork shared by the FAB fixtures, using the public icon helper.
pub fn add() -> crate::Icon {
    static ADD: OnceLock<Handle> = OnceLock::new();
    crate::icon(ADD.get_or_init(|| Handle::from_memory(br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M11 4h2v7h7v2h-7v7h-2v-7H4v-2h7z"/></svg>"#)).clone())
}

pub struct Icon;
pub struct LargeIcon;
impl<Message> Widget<Message, Theme, Renderer> for LargeIcon {
    fn size(&self) -> Size<Length> {
        Size::new(36.0.into(), 36.0.into())
    }
    fn layout(&mut self, _: &mut Tree, _: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::atomic(limits, 36.0, 36.0)
    }
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        <Icon as Widget<Message, Theme, Renderer>>::draw(
            &Icon, tree, renderer, theme, style, layout, cursor, viewport,
        );
    }
}
impl<'a, Message: 'a> From<LargeIcon> for Element<'a, Message> {
    fn from(icon: LargeIcon) -> Self {
        Self::new(icon)
    }
}
impl<Message> Widget<Message, Theme, Renderer> for Icon {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(24.0), Length::Fixed(24.0))
    }
    fn layout(&mut self, _: &mut Tree, _: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::atomic(limits, 24.0, 24.0)
    }
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        _: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        static ICON: OnceLock<Handle> = OnceLock::new();
        let handle = ICON.get_or_init(|| Handle::from_memory(br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path d="M12 3 15 9 22 10 17 15 18 22 12 18 6 22 7 15 2 10 9 9Z" fill="none" stroke="black" stroke-width="1.8" stroke-linejoin="round"/></svg>"#));
        <crate::Icon as Widget<Message, Theme, Renderer>>::draw(
            &crate::icon(handle.clone()),
            tree,
            renderer,
            theme,
            style,
            layout,
            mouse::Cursor::Unavailable,
            viewport,
        );
    }
}
impl<'a, Message: 'a> From<Icon> for Element<'a, Message> {
    fn from(icon: Icon) -> Self {
        Self::new(icon)
    }
}
