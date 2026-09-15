//! Passive, monochrome SVG icons that inherit their parent's foreground.
use crate::{Element, Theme};
use iced::advanced::{Layout, Widget, layout, renderer, svg, widget::Tree};
use iced::{Color, Length, Rectangle, Renderer, Size, mouse};

/// A square SVG icon, 24 logical pixels by default. Use a square SVG view box.
/// Its color (including disabled-state alpha) comes from the enclosing component.
pub struct Icon {
    handle: svg::Handle,
    size: f32,
}

pub fn icon(handle: impl Into<svg::Handle>) -> Icon {
    Icon {
        handle: handle.into(),
        size: 24.0,
    }
}

impl Icon {
    pub fn size(mut self, size: impl Into<iced::Pixels>) -> Self {
        let size = size.into().0;
        if size.is_finite() {
            self.size = size.max(0.0);
        }
        self
    }
}

// iced's SVG tint replaces RGB only. Pass alpha as image opacity so the
// renderer composites against the pixels underneath, including custom surfaces.
pub(crate) fn tinted(handle: impl Into<svg::Handle>, color: Color) -> svg::Svg {
    svg::Svg {
        handle: handle.into(),
        color: Some(Color { a: 1.0, ..color }),
        rotation: iced::Radians(0.0),
        opacity: color.a,
    }
}

impl<Message> Widget<Message, Theme, Renderer> for Icon {
    fn size(&self) -> Size<Length> {
        Size::new(self.size.into(), self.size.into())
    }
    fn layout(&mut self, _: &mut Tree, _: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::atomic(limits, self.size, self.size)
    }
    fn draw(
        &self,
        _: &Tree,
        renderer: &mut Renderer,
        _: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        _: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        use iced::advanced::{Renderer as _, svg::Renderer as _};
        let bounds = layout.bounds();
        if let Some(clip) = bounds.intersection(viewport) {
            renderer.with_layer(clip, |renderer| {
                renderer.draw_svg(tinted(self.handle.clone(), style.text_color), bounds, clip);
            });
        }
    }
}
impl<'a, Message: 'a> From<Icon> for Element<'a, Message> {
    fn from(icon: Icon) -> Self {
        Self::new(icon)
    }
}
