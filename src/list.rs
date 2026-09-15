//! Material list rows with independently interactive leading and trailing content.
use crate::{Button, ButtonVariant, Element, Theme, TypeScale, tokens, typography};
use iced::{Color, Length, widget};

pub struct ListItem<'a, Message> {
    headline: String,
    supporting: Option<String>,
    overline: Option<String>,
    leading: Option<Element<'a, Message>>,
    trailing: Option<Element<'a, Message>>,
    action: Option<Message>,
    selected: bool,
    disabled: bool,
    width: Length,
}
pub fn list_item<'a, Message>(headline: impl Into<String>) -> ListItem<'a, Message> {
    ListItem {
        headline: headline.into(),
        supporting: None,
        overline: None,
        leading: None,
        trailing: None,
        action: None,
        selected: false,
        disabled: false,
        width: Length::Fill,
    }
}
impl<'a, Message> ListItem<'a, Message> {
    pub fn supporting_text(mut self, text: impl Into<String>) -> Self {
        self.supporting = Some(text.into());
        self
    }
    pub fn overline(mut self, text: impl Into<String>) -> Self {
        self.overline = Some(text.into());
        self
    }
    /// Icons, avatars, images or controls. Size custom content before passing it in.
    pub fn leading(mut self, content: impl Into<Element<'a, Message>>) -> Self {
        self.leading = Some(content.into());
        self
    }
    /// Interactive controls receive clicks before the row action.
    pub fn trailing(mut self, content: impl Into<Element<'a, Message>>) -> Self {
        self.trailing = Some(content.into());
        self
    }
    pub fn on_press(mut self, message: Message) -> Self {
        self.action = Some(message);
        self
    }
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    /// Disables the row and its child controls. A row without an action is simply static.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
}
impl<'a, Message: Clone + 'a> From<ListItem<'a, Message>> for Element<'a, Message> {
    fn from(item: ListItem<'a, Message>) -> Self {
        let disabled = item.disabled;
        let selected = item.selected;
        let height = if item.supporting.is_some() && item.overline.is_some() {
            tokens::size::LIST_THREE_LINE
        } else if item.supporting.is_some() || item.overline.is_some() {
            tokens::size::LIST_TWO_LINE
        } else {
            tokens::size::LIST_ONE_LINE
        };
        let color = move |theme: &Theme, supporting: bool| {
            if disabled {
                crate::theme::alpha(theme.colors.on_surface, 0.38)
            } else if selected {
                theme.colors.on_secondary_container
            } else if supporting {
                theme.colors.on_surface_variant
            } else {
                theme.colors.on_surface
            }
        };
        let mut text = widget::Column::new().width(Length::Fill);
        if let Some(overline) = item.overline {
            text = text.push(
                typography(overline, TypeScale::LabelSmall).style(move |theme| {
                    widget::text::Style {
                        color: Some(color(theme, true)),
                    }
                }),
            );
        }
        text = text.push(
            typography(item.headline, TypeScale::BodyLarge).style(move |theme| {
                widget::text::Style {
                    color: Some(color(theme, false)),
                }
            }),
        );
        if let Some(supporting) = item.supporting {
            text = text.push(
                typography(supporting, TypeScale::BodyMedium).style(move |theme| {
                    widget::text::Style {
                        color: Some(color(theme, true)),
                    }
                }),
            );
        }
        let mut row = widget::Row::new()
            .spacing(16)
            .align_y(iced::Alignment::Center);
        if let Some(leading) = item.leading {
            row = row.push(leading);
        }
        row = row.push(text);
        if let Some(trailing) = item.trailing {
            row = row.push(trailing);
        }
        let body = widget::container(row)
            .padding([8, 16])
            .center_y(Length::Shrink)
            .width(Length::Fill);
        Button::new(body)
            .minimum_height(height)
            .variant(ButtonVariant::Text)
            .radius(0.0)
            .padding(0)
            .width(item.width)
            .palette(move |theme| {
                (
                    if selected {
                        theme.colors.secondary_container
                    } else {
                        Color::TRANSPARENT
                    },
                    theme.colors.on_surface_variant,
                )
            })
            .on_press_maybe(if disabled { None } else { item.action })
            .interactive_children(disabled)
            .into()
    }
}
/// A transparent list provides grouping and vertical padding; compose dividers
/// between items. This returns a native container: use its `.style(...)` builder
/// for an explicit background color or gradient.
pub fn list<'a, Message: 'a>(
    items: impl IntoIterator<Item = Element<'a, Message>>,
) -> widget::Container<'a, Message, Theme> {
    widget::container(widget::Column::with_children(items).width(Length::Fill))
        .padding([8, 0])
        .width(Length::Fill)
        .style(|theme: &Theme| widget::container::Style {
            text_color: Some(theme.colors.on_surface),
            ..Default::default()
        })
}
