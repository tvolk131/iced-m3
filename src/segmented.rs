//! Connected outlined buttons with application-owned single or multiple selection.
use crate::{Button, ButtonVariant, Element, Theme, TypeScale, theme::alpha, tokens, typography};
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, renderer,
    widget::{Operation, Tree},
};
use iced::{Border, Color, Event, Length, Rectangle, Renderer, Size, mouse, widget};

/// Single-selection clicks keep one value selected; multiple selection may be empty.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SegmentSelection<Value> {
    Single(Option<Value>),
    Multiple(Vec<Value>),
}
impl<Value: PartialEq> SegmentSelection<Value> {
    pub fn contains(&self, value: &Value) -> bool {
        match self {
            Self::Single(current) => current.as_ref() == Some(value),
            Self::Multiple(values) => values.contains(value),
        }
    }
}
pub struct Segment<'a, Value, Message> {
    value: Value,
    label: String,
    icon: Option<Element<'a, Message>>,
    disabled: bool,
}
impl<'a, Value, Message> Segment<'a, Value, Message> {
    pub fn new(value: Value, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
            icon: None,
            disabled: false,
        }
    }
    /// Passive content, displayed in an 18px slot. Selection replaces it with a check.
    pub fn icon(mut self, icon: impl Into<Element<'a, Message>>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}
pub struct SegmentedButtons<'a, Value, Message> {
    items: Vec<Segment<'a, Value, Message>>,
    selection: SegmentSelection<Value>,
    on_change: Option<Box<dyn Fn(SegmentSelection<Value>) -> Message + 'a>>,
    disabled: bool,
    width: Length,
}
pub fn segmented_buttons<'a, Value, Message>(
    items: impl IntoIterator<Item = Segment<'a, Value, Message>>,
    selection: SegmentSelection<Value>,
) -> SegmentedButtons<'a, Value, Message> {
    SegmentedButtons {
        items: items.into_iter().collect(),
        selection,
        on_change: None,
        disabled: false,
        width: Length::Shrink,
    }
}
impl<'a, Value, Message> SegmentedButtons<'a, Value, Message> {
    pub fn on_change(mut self, handler: impl Fn(SegmentSelection<Value>) -> Message + 'a) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// `Fill` distributes equal widths. Keep labels short or provide horizontal scrolling.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
}
impl<'a, Value: Clone + PartialEq + 'a, Message: Clone + 'a>
    From<SegmentedButtons<'a, Value, Message>> for Element<'a, Message>
{
    fn from(group: SegmentedButtons<'a, Value, Message>) -> Self {
        let active = group
            .items
            .iter()
            .filter(|i| !i.disabled)
            .position(|i| group.selection.contains(&i.value))
            .unwrap_or(0);
        let count = group.items.len();
        let enabled = !group.disabled && group.on_change.is_some();
        let disabled: Vec<_> = group
            .items
            .iter()
            .map(|item| !enabled || item.disabled)
            .collect();
        let children = group.items.into_iter().enumerate().map(|(index, item)| {
            let selected = group.selection.contains(&item.value);
            let next = match &group.selection {
                SegmentSelection::Single(_) => SegmentSelection::Single(Some(item.value.clone())),
                SegmentSelection::Multiple(values) => {
                    let mut values = values.clone();
                    if selected {
                        values.retain(|value| value != &item.value);
                    } else {
                        values.push(item.value.clone());
                    }
                    SegmentSelection::Multiple(values)
                }
            };
            let message = if !disabled[index] {
                group.on_change.as_ref().map(|handler| handler(next))
            } else {
                None
            };
            let mark: Element<'a, Message> = if selected {
                Element::new(Check)
            } else {
                item.icon.unwrap_or_else(|| {
                    widget::space()
                        .width(tokens::size::SEGMENT_ICON)
                        .height(tokens::size::SEGMENT_ICON)
                        .into()
                })
            };
            let content = widget::row![
                widget::container(mark)
                    .center_x(tokens::size::SEGMENT_ICON)
                    .center_y(tokens::size::SEGMENT_ICON),
                typography(item.label, TypeScale::LabelLarge)
                    .wrapping(widget::text::Wrapping::None)
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center);
            Button::new(content)
                .variant(ButtonVariant::Text)
                .height(tokens::size::SEGMENT)
                .padding([10, 12])
                .width(if group.width == Length::Shrink {
                    Length::Shrink
                } else {
                    Length::Fill
                })
                .segment_style(
                    selected,
                    iced::border::Radius {
                        top_left: if index == 0 { 20.0 } else { 0.0 },
                        bottom_left: if index == 0 { 20.0 } else { 0.0 },
                        top_right: if index + 1 == count { 20.0 } else { 0.0 },
                        bottom_right: if index + 1 == count { 20.0 } else { 0.0 },
                    },
                )
                .on_press_maybe(message)
                .into()
        });
        crate::focus::group_with_active(
            Element::new(Outline {
                content: widget::Row::with_children(children)
                    .width(group.width)
                    .into(),
                disabled,
            }),
            active,
        )
    }
}
struct Outline<'a, Message> {
    content: Element<'a, Message>,
    disabled: Vec<bool>,
}
impl<Message: Clone> Widget<Message, Theme, Renderer> for Outline<'_, Message> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
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
        use iced::advanced::Renderer as _;
        let bounds = layout.bounds();
        let Some(clip) = bounds.intersection(viewport) else {
            return;
        };
        renderer.with_layer(clip, |renderer| {
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor,
                &clip,
            );
        });
        // Child buttons have their own renderer layers. Put the shared outline
        // in a subsequent layer so selected backgrounds cannot cover it.
        renderer.with_layer(clip, |renderer| {
            if self.disabled.is_empty() {
                return;
            }
            let outline = if self.disabled.iter().all(|disabled| *disabled) {
                alpha(theme.colors.on_surface, 0.12)
            } else {
                theme.colors.outline
            };
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        radius: 20.0.into(),
                        width: 1.0,
                        color: outline,
                    },
                    ..Default::default()
                },
                Color::TRANSPARENT,
            );
            for (index, child) in layout.children().enumerate().skip(1) {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: child.bounds().x - 0.5,
                            y: bounds.y,
                            width: 1.0,
                            height: bounds.height,
                        },
                        ..Default::default()
                    },
                    if self.disabled[index - 1] && self.disabled[index] {
                        alpha(theme.colors.on_surface, 0.12)
                    } else {
                        theme.colors.outline
                    },
                );
            }
        });
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }
}
struct Check;
impl<Message> Widget<Message, Theme, Renderer> for Check {
    fn size(&self) -> Size<Length> {
        Size::new(
            Length::Fixed(tokens::size::SEGMENT_ICON),
            Length::Fixed(tokens::size::SEGMENT_ICON),
        )
    }
    fn layout(&mut self, _: &mut Tree, _: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::Node::new(limits.resolve(18.0, 18.0, Size::new(18.0, 18.0)))
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
        static HANDLE: std::sync::OnceLock<Handle> = std::sync::OnceLock::new();
        let handle = HANDLE.get_or_init(|| Handle::from_memory(b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 18 18'><path d='M3 9 L7 13 L15 5' fill='none' stroke='black' stroke-width='2'/></svg>".as_slice()));
        renderer.draw_svg(
            crate::icon::tinted(handle.clone(), style.text_color),
            layout.bounds(),
            clip,
        );
    }
}
