//! Small internal vector marks that inherit their parent's foreground color.
use crate::Theme;
use iced::advanced::{Layout, Widget, layout, renderer, widget::Tree};
use iced::{Length, Rectangle, Renderer, Size, mouse};

#[derive(Clone, Copy)]
pub(crate) enum Glyph {
    Check,
    Close,
    Search,
    Back,
    Down,
    Previous,
    Next,
    Edit,
    Calendar,
    Clock,
    Keyboard,
}
impl<Message> Widget<Message, Theme, Renderer> for Glyph {
    fn size(&self) -> Size<Length> {
        let size = if matches!(self, Self::Check | Self::Close) {
            18.0
        } else {
            24.0
        };
        Size::new(Length::Fixed(size), Length::Fixed(size))
    }
    fn layout(&mut self, _: &mut Tree, _: &Renderer, limits: &layout::Limits) -> layout::Node {
        let size = if matches!(self, Self::Check | Self::Close) {
            18.0
        } else {
            24.0
        };
        layout::Node::new(limits.resolve(size, size, Size::ZERO))
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
        use iced::advanced::svg::{Handle, Renderer as _};
        let Some(clip) = layout.bounds().intersection(viewport) else {
            return;
        };
        static HANDLES: std::sync::OnceLock<[Handle; 11]> = std::sync::OnceLock::new();
        let handles = HANDLES.get_or_init(|| [
            "M4 12 L9 17 L20 6",
            "M6 6 L18 18 M18 6 L6 18",
            "M20 20 L16 16 M18 10 A8 8 0 1 1 2 10 A8 8 0 1 1 18 10",
            "M20 12 H4 M11 5 L4 12 L11 19",
            "M6 9 L12 15 L18 9",
            "M15 6 L9 12 L15 18",
            "M9 6 L15 12 L9 18",
            "M4 16 L4 20 L8 20 L20 8 L16 4 Z M13 7 L17 11",
            "M4 5 H20 V21 H4 Z M4 10 H20 M8 3 V7 M16 3 V7 M8 14 H10 M14 14 H16 M8 17 H10",
            "M22 12 A10 10 0 1 1 2 12 A10 10 0 1 1 22 12 M12 6 V12 L16 15",
            "M2 5 H22 V19 H2 Z M5 9 H7 M11 9 H13 M17 9 H19 M5 12 H7 M11 12 H13 M17 12 H19 M7 16 H17",
        ].map(|path| Handle::from_memory(format!("<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'><path d='{path}' fill='none' stroke='black' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'/></svg>").into_bytes())));
        let index = match self {
            Self::Check => 0,
            Self::Close => 1,
            Self::Search => 2,
            Self::Back => 3,
            Self::Down => 4,
            Self::Previous => 5,
            Self::Next => 6,
            Self::Edit => 7,
            Self::Calendar => 8,
            Self::Clock => 9,
            Self::Keyboard => 10,
        };
        renderer.draw_svg(
            crate::icon::tinted(handles[index].clone(), style.text_color),
            layout.bounds(),
            clip,
        );
    }
}
