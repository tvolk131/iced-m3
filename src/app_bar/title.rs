//! Single-line titles measured with the same shaper used to draw them.
use crate::{Theme, fonts};
use iced::advanced::{
    Layout, Widget, layout, renderer, text,
    widget::{Operation, Tree, tree},
};
use iced::{Length, Rectangle, Renderer, Size, mouse};
use text::Paragraph as _;
use unicode_segmentation::UnicodeSegmentation;

type Paragraph = <Renderer as text::Renderer>::Paragraph;

pub(super) struct Title {
    text: String,
    size: f32,
    line_height: f32,
    centered: bool,
}
#[derive(Default)]
struct State {
    key: Option<(String, f32, f32, f32, bool)>,
    paragraph: Paragraph,
}
impl Title {
    pub(super) fn new(text: String, size: f32, line_height: f32, centered: bool) -> Self {
        fonts::ensure_loaded();
        Self {
            text,
            size,
            line_height,
            centered,
        }
    }
    fn paragraph(&self, content: &str) -> Paragraph {
        Paragraph::with_text(text::Text {
            content,
            bounds: Size::new(f32::INFINITY, self.line_height),
            size: self.size.into(),
            line_height: text::LineHeight::Absolute(self.line_height.into()),
            font: fonts::REGULAR,
            align_x: if self.centered {
                text::Alignment::Center
            } else {
                text::Alignment::Left
            },
            align_y: iced::alignment::Vertical::Top,
            shaping: text::Shaping::Advanced,
            wrapping: text::Wrapping::None,
        })
    }
    fn fit(&self, width: f32) -> (String, Paragraph) {
        let text = self.text.replace(['\n', '\r'], " ");
        let full = self.paragraph(&text);
        if full.min_width() <= width {
            return (text, full);
        }
        let ellipsis = self.paragraph("…");
        if ellipsis.min_width() > width {
            return (String::new(), self.paragraph(""));
        }
        let ends: Vec<_> = text.grapheme_indices(true).map(|(i, _)| i).collect();
        let (mut low, mut high) = (0, ends.len());
        while low < high {
            let mid = (low + high).div_ceil(2);
            let candidate = format!("{}…", &text[..ends[mid - 1]]);
            if self.paragraph(&candidate).min_width() <= width {
                low = mid;
            } else {
                high = mid - 1;
            }
        }
        let fitted = if low == 0 {
            "…".to_owned()
        } else {
            format!("{}…", &text[..ends[low - 1]])
        };
        let paragraph = self.paragraph(&fitted);
        (fitted, paragraph)
    }
}
impl<Message> Widget<Message, Theme, Renderer> for Title {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fixed(self.line_height))
    }
    fn layout(&mut self, tree: &mut Tree, _: &Renderer, limits: &layout::Limits) -> layout::Node {
        let size = limits.resolve(Length::Fill, self.line_height, Size::ZERO);
        let state = tree.state.downcast_mut::<State>();
        let key = (
            self.text.clone(),
            self.size,
            self.line_height,
            size.width,
            self.centered,
        );
        if state.key.as_ref() != Some(&key) {
            state.paragraph = self.fit(size.width).1;
            state.key = Some(key);
        }
        layout::Node::new(size)
    }
    fn operate(&mut self, _: &mut Tree, l: Layout<'_>, _: &Renderer, op: &mut dyn Operation) {
        op.text(None, l.bounds(), &self.text);
    }
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _: &Theme,
        style: &renderer::Style,
        l: Layout<'_>,
        _: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        if let Some(clip) = l.bounds().intersection(viewport) {
            iced::widget::text::draw(
                renderer,
                style,
                l.bounds(),
                &tree.state.downcast_ref::<State>().paragraph,
                Default::default(),
                &clip,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn titles_fit_without_splitting_graphemes_and_expand_again() {
        for original in ["Workspace release planning", "Cafe\u{301} 👩🏽‍💻 family 👨‍👩‍👧‍👦"]
        {
            let title = Title::new(original.into(), 22.0, 28.0, false);
            for width in [0.0, 8.0, 25.0, 60.0, 120.0, 240.0] {
                let (fitted, paragraph) = title.fit(width);
                assert!(paragraph.min_width() <= width);
                if let Some(prefix) = fitted.strip_suffix('…') {
                    assert!(
                        original
                            .grapheme_indices(true)
                            .any(|(i, _)| i == prefix.len())
                    );
                }
            }
            assert_eq!(title.fit(2000.0).0, original);
        }
    }
}
