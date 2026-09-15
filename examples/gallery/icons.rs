//! Example passive vector content. A 24px drawing has no font baseline and
//! inherits the enclosing button's selected/disabled foreground color.
use iced::advanced::{
    Layout, Widget, layout, renderer,
    svg::Handle,
    widget::{Tree, tree},
};
use iced::{Length, Rectangle, Renderer, Size, mouse};
use iced_material::{Element, Theme};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Icon {
    Add,
    Star,
    Close,
    Workspace,
    Activity,
    Settings,
    Visibility,
}

impl Icon {
    pub fn sized(self, size: f32) -> SizedIcon {
        SizedIcon { icon: self, size }
    }
    fn handle(self) -> &'static Handle {
        static ADD: OnceLock<Handle> = OnceLock::new();
        static STAR: OnceLock<Handle> = OnceLock::new();
        static CLOSE: OnceLock<Handle> = OnceLock::new();
        static WORKSPACE: OnceLock<Handle> = OnceLock::new();
        static ACTIVITY: OnceLock<Handle> = OnceLock::new();
        static VISIBILITY: OnceLock<Handle> = OnceLock::new();
        static SETTINGS: OnceLock<Handle> = OnceLock::new();
        let (cache, path) = match self {
            Self::Visibility => (
                &VISIBILITY,
                "M2 12 Q12 -2 22 12 Q12 26 2 12 Z M15 12 A3 3 0 1 1 9 12 A3 3 0 1 1 15 12",
            ),
            Self::Workspace => (&WORKSPACE, "M3 6 H10 L12 8 H21 V19 H3 Z"),
            Self::Activity => (&ACTIVITY, "M3 12 H7 L10 4 L14 20 L17 12 H21"),
            Self::Settings => (
                &SETTINGS,
                "M4 6 H20 M4 12 H20 M4 18 H20 M8 3 V9 M16 9 V15 M10 15 V21",
            ),
            Self::Add => (&ADD, "M 5 12 H 19 M 12 5 V 19"),
            Self::Star => (
                &STAR,
                "M 12 3 L 14.8 8.9 L 21 9.8 L 16.5 14.4 L 17.6 21 L 12 17.9 L 6.4 21 L 7.5 14.4 L 3 9.8 L 9.2 8.9 Z",
            ),
            Self::Close => (&CLOSE, "M 6 6 L 18 18 M 18 6 L 6 18"),
        };
        cache.get_or_init(|| Handle::from_memory(format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path d="{path}" fill="none" stroke="black" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg>"#
        ).into_bytes()))
    }
}

pub struct SizedIcon {
    icon: Icon,
    size: f32,
}
impl<Message> Widget<Message, Theme, Renderer> for SizedIcon {
    fn size(&self) -> Size<Length> {
        Size::new(self.size.into(), self.size.into())
    }
    fn layout(&mut self, _: &mut Tree, _: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::atomic(limits, self.size, self.size)
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
            &self.icon, tree, renderer, theme, style, layout, cursor, viewport,
        );
    }
}
impl<'a, Message: 'a> From<SizedIcon> for Element<'a, Message> {
    fn from(icon: SizedIcon) -> Self {
        Self::new(icon)
    }
}

impl<Message> Widget<Message, Theme, Renderer> for Icon {
    fn tag(&self) -> tree::Tag {
        tree::Tag::stateless()
    }

    fn state(&self) -> tree::State {
        tree::State::None
    }

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
        <iced_material::Icon as Widget<Message, Theme, Renderer>>::draw(
            &iced_material::icon(self.handle().clone()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use iced::advanced::{Renderer as _, renderer::Headless};
    use iced_material::{ButtonVariant, icon_button};

    #[test]
    fn icon_ink_is_centered_inside_the_button_outline() {
        let mut renderer = iced::futures::executor::block_on(<Renderer as Headless>::new(
            iced::Font::DEFAULT,
            16.0.into(),
            Some("tiny-skia"),
        ))
        .unwrap();
        let bounds = Rectangle::with_size(Size::new(40.0, 40.0));
        let theme = Theme::light();
        let mut render = |mut element: Element<'_, ()>, offset: f32| {
            let mut tree = Tree::new(&element);
            let node = element.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, bounds.size()),
            );
            renderer.reset(bounds);
            renderer.with_translation(iced::Vector::new(0.0, -offset), |renderer| {
                element.as_widget().draw(
                    &tree,
                    renderer,
                    &theme,
                    &Default::default(),
                    Layout::with_offset(iced::Vector::new(0.0, offset), &node),
                    mouse::Cursor::Unavailable,
                    &(bounds + iced::Vector::new(0.0, offset)),
                );
            });
            renderer.screenshot(Size::new(80, 80), 2.0, theme.colors.surface)
        };
        // Subtract an identical empty outlined button to measure the actual
        // painted icon, including antialiasing, independently of its layout box.
        let empty = render(
            icon_button(iced::widget::space().width(24).height(24))
                .variant(ButtonVariant::Outlined)
                .on_press(())
                .into(),
            0.0,
        );
        for icon in [Icon::Add, Icon::Star, Icon::Close] {
            let pixels = render(
                icon_button(icon)
                    .variant(ButtonVariant::Outlined)
                    .on_press(())
                    .into(),
                0.0,
            );
            let scrolled = render(
                icon_button(icon)
                    .variant(ButtonVariant::Outlined)
                    .on_press(())
                    .into(),
                500.0,
            );
            assert!(
                pixels == scrolled,
                "icon ink must remain centered after scrolling"
            );
            let mut extent = (80, 80, 0, 0);
            for (i, (a, b)) in pixels
                .chunks_exact(4)
                .zip(empty.chunks_exact(4))
                .enumerate()
            {
                if a != b {
                    let (x, y) = (i % 80, i / 80);
                    extent = (
                        extent.0.min(x),
                        extent.1.min(y),
                        extent.2.max(x),
                        extent.3.max(y),
                    );
                }
            }
            assert!(
                extent.0 < extent.2 && extent.1 < extent.3,
                "icon must paint pixels"
            );
            let center_x = (extent.0 + extent.2 + 1) as f32 / 2.0;
            let center_y = (extent.1 + extent.3 + 1) as f32 / 2.0;
            assert!(
                (center_x - 40.0).abs() <= 1.0,
                "horizontal ink center: {center_x}"
            );
            assert!(
                (center_y - 40.0).abs() <= 1.0,
                "vertical ink center: {center_y}"
            );
        }
    }
}
