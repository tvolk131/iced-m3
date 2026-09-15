//! Material app bars with application-owned scroll and collapse state. The application composes its actions and scroll state.
use crate::{Element, Theme, tokens};
mod title;
use iced::{Length, widget};
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AppBarVariant {
    #[default]
    Small,
    CenterAligned,
    Medium,
    Large,
}
pub struct AppBar<'a, Message> {
    title: String,
    leading: Option<Element<'a, Message>>,
    actions: Vec<Element<'a, Message>>,
    scrolled: bool,
    variant: AppBarVariant,
    collapse: f32,
}
pub fn app_bar<'a, Message>(title: impl Into<String>) -> AppBar<'a, Message> {
    AppBar {
        title: title.into(),
        leading: None,
        actions: Vec::new(),
        scrolled: false,
        variant: AppBarVariant::Small,
        collapse: 0.0,
    }
}
impl<'a, Message> AppBar<'a, Message> {
    pub fn variant(mut self, variant: AppBarVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Collapse progress: zero is expanded, one is the 64px small bar.
    /// Set this from the content's scroll offset; the app retains scroll ownership.
    pub fn collapse(mut self, progress: f32) -> Self {
        self.collapse = if progress.is_finite() {
            progress.clamp(0.0, 1.0)
        } else {
            0.0
        };
        self
    }
    pub fn leading(mut self, content: impl Into<Element<'a, Message>>) -> Self {
        self.leading = Some(content.into());
        self
    }
    pub fn action(mut self, content: impl Into<Element<'a, Message>>) -> Self {
        self.actions.push(content.into());
        self
    }
    /// Set from your content's scroll offset to use the elevated surface treatment.
    pub fn scrolled(mut self, scrolled: bool) -> Self {
        self.scrolled = scrolled;
        self
    }
}
impl<'a, Message: 'a> From<AppBar<'a, Message>> for Element<'a, Message> {
    fn from(bar: AppBar<'a, Message>) -> Self {
        let expanded = matches!(bar.variant, AppBarVariant::Medium | AppBarVariant::Large);
        let progress = if expanded { bar.collapse } else { 1.0 };
        let extra = match bar.variant {
            AppBarVariant::Medium => 48.0,
            AppBarVariant::Large => 88.0,
            _ => 0.0,
        } * (1.0 - progress);
        let height = tokens::size::APP_BAR + extra;
        let font_size = 22.0
            + match bar.variant {
                AppBarVariant::Medium => 6.0,
                AppBarVariant::Large => 10.0,
                _ => 0.0,
            } * (1.0 - progress);
        let line_height = 28.0
            + match bar.variant {
                AppBarVariant::Medium => 8.0,
                AppBarVariant::Large => 12.0,
                _ => 0.0,
            } * (1.0 - progress);
        let title = widget::container(Element::new(title::Title::new(
            bar.title,
            font_size,
            line_height,
            bar.variant == AppBarVariant::CenterAligned,
        )))
        .width(Length::Fill)
        .clip(true)
        .into();
        let mut row = widget::Row::new()
            .spacing(12)
            .width(Length::Fill)
            .align_y(iced::Alignment::Center);
        let title_index = usize::from(bar.leading.is_some());
        if let Some(leading) = bar.leading {
            row = row.push(leading);
        }
        row = row.push(widget::space().width(Length::Fill));
        if !bar.actions.is_empty() {
            row = row.push(
                widget::Row::with_children(bar.actions)
                    .spacing(4)
                    .align_y(iced::Alignment::Center),
            );
        }
        let content: Element<'a, Message> = Element::new(BarContent {
            row: row.into(),
            title,
            title_index,
            centered: bar.variant == AppBarVariant::CenterAligned,
            progress,
            height,
            line_height,
        });
        crate::elevation::elevated(
            widget::container(content)
                .height(height)
                .width(Length::Fill)
                .clip(true)
                .style(move |theme: &Theme| widget::container::Style {
                    background: Some(
                        if bar.scrolled {
                            theme.colors.surface_container
                        } else {
                            theme.colors.surface
                        }
                        .into(),
                    ),
                    text_color: Some(theme.colors.on_surface),

                    ..Default::default()
                }),
            if bar.scrolled { 2.0 } else { 0.0 },
            0.0,
        )
    }
}

use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, overlay, renderer,
    widget::{Operation, Tree},
};
use iced::{Event, Point, Rectangle, Renderer, Size, Vector, mouse};
struct BarContent<'a, Message> {
    row: Element<'a, Message>,
    title: Element<'a, Message>,
    title_index: usize,
    centered: bool,
    progress: f32,
    height: f32,
    line_height: f32,
}
impl<Message> Widget<Message, Theme, Renderer> for BarContent<'_, Message> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.row), Tree::new(&self.title)]
    }
    fn diff(&self, t: &mut Tree) {
        t.diff_children(&[&self.row, &self.title]);
    }
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fixed(self.height))
    }
    fn layout(&mut self, t: &mut Tree, r: &Renderer, limits: &layout::Limits) -> layout::Node {
        let size = limits.resolve(
            Length::Fill,
            Length::Fixed(self.height),
            Size::new(320.0, self.height),
        );
        let row = self
            .row
            .as_widget_mut()
            .layout(
                &mut t.children[0],
                r,
                &layout::Limits::new(Size::ZERO, Size::new((size.width - 32.0).max(0.0), 40.0)),
            )
            .move_to(Point::new(16.0, 12.0));
        let space = row.children()[self.title_index].bounds();
        let (x, width) = if self.centered {
            let inset = (space.x + 16.0).max(size.width - (space.x + space.width + 16.0));
            (inset, (size.width - 2.0 * inset).max(0.0))
        } else {
            (
                16.0 + space.x * (self.progress * 3.0).min(1.0),
                ((size.width - 32.0) * (1.0 - (self.progress * 3.0).min(1.0))
                    + space.width * (self.progress * 3.0).min(1.0))
                .max(0.0),
            )
        };
        let title = self
            .title
            .as_widget_mut()
            .layout(
                &mut t.children[1],
                r,
                &layout::Limits::new(Size::ZERO, Size::new(width, self.line_height)),
            )
            .move_to(Point::new(
                x,
                18.0 + self.height - 64.0 - (self.line_height - 28.0),
            ));
        layout::Node::with_children(size, vec![row, title])
    }
    fn operate(&mut self, t: &mut Tree, l: Layout<'_>, r: &Renderer, o: &mut dyn Operation) {
        self.row
            .as_widget_mut()
            .operate(&mut t.children[0], l.child(0), r, o);
        self.title
            .as_widget_mut()
            .operate(&mut t.children[1], l.child(1), r, o);
    }
    fn update(
        &mut self,
        t: &mut Tree,
        e: &Event,
        l: Layout<'_>,
        c: mouse::Cursor,
        r: &Renderer,
        cb: &mut dyn Clipboard,
        s: &mut Shell<'_, Message>,
        v: &Rectangle,
    ) {
        self.row
            .as_widget_mut()
            .update(&mut t.children[0], e, l.child(0), c, r, cb, s, v);
    }
    fn draw(
        &self,
        t: &Tree,
        r: &mut Renderer,
        th: &Theme,
        s: &renderer::Style,
        l: Layout<'_>,
        c: mouse::Cursor,
        v: &Rectangle,
    ) {
        self.title
            .as_widget()
            .draw(&t.children[1], r, th, s, l.child(1), c, v);
        self.row
            .as_widget()
            .draw(&t.children[0], r, th, s, l.child(0), c, v);
    }
    fn mouse_interaction(
        &self,
        t: &Tree,
        l: Layout<'_>,
        c: mouse::Cursor,
        v: &Rectangle,
        r: &Renderer,
    ) -> mouse::Interaction {
        self.row
            .as_widget()
            .mouse_interaction(&t.children[0], l.child(0), c, v, r)
    }
    fn overlay<'b>(
        &'b mut self,
        t: &'b mut Tree,
        l: Layout<'b>,
        r: &Renderer,
        v: &Rectangle,
        tr: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.row
            .as_widget_mut()
            .overlay(&mut t.children[0], l.child(0), r, v, tr)
    }
}
