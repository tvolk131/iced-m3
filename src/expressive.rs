//! Expressive action compositions, sharing the existing Material interaction layer.
use crate::{Button, Element, MenuItem, Theme, TypeScale, typography};
use iced::{Border, Length, widget};
pub struct ButtonGroup<'a, Message> {
    buttons: Vec<Button<'a, Message>>,
    connected: bool,
}
pub fn button_group<'a, Message>(
    buttons: impl IntoIterator<Item = Button<'a, Message>>,
) -> ButtonGroup<'a, Message> {
    ButtonGroup {
        buttons: buttons.into_iter().collect(),
        connected: false,
    }
}
impl<Message> ButtonGroup<'_, Message> {
    pub fn connected(mut self, connected: bool) -> Self {
        self.connected = connected;
        self
    }
}
impl<'a, Message: Clone + 'a> From<ButtonGroup<'a, Message>> for Element<'a, Message> {
    fn from(group: ButtonGroup<'a, Message>) -> Self {
        let last = group.buttons.len().saturating_sub(1);
        let row =
            widget::Row::with_children(group.buttons.into_iter().enumerate().map(|(i, button)| {
                let button = button.expressive(true).width(Length::Fill);
                if group.connected {
                    button
                        .corner_radius(
                            Border::default()
                                .rounded(iced::border::Radius {
                                    top_left: if i == 0 { 20.0 } else { 4.0 },
                                    bottom_left: if i == 0 { 20.0 } else { 4.0 },
                                    top_right: if i == last { 20.0 } else { 4.0 },
                                    bottom_right: if i == last { 20.0 } else { 4.0 },
                                })
                                .radius,
                        )
                        .into()
                } else {
                    button.into()
                }
            }))
            .spacing(if group.connected { 2 } else { 8 });
        crate::focus::group(row)
    }
}
pub fn split_button<'a, Message: Clone + 'a>(
    label: impl widget::text::IntoFragment<'a>,
    action: Message,
    items: impl IntoIterator<Item = MenuItem<'a, Message>>,
) -> Element<'a, Message> {
    widget::row![
        crate::button(label)
            .on_press(action)
            .expressive(true)
            .corner_radius(iced::border::Radius {
                top_left: 20.0,
                bottom_left: 20.0,
                top_right: 4.0,
                bottom_right: 4.0
            }),
        crate::menu::split_menu(items)
    ]
    .spacing(2)
    .into()
}
pub struct Toolbar<'a, Message> {
    items: Vec<Element<'a, Message>>,
    floating: bool,
    vertical: bool,
}
pub fn toolbar<'a, Message>(
    items: impl IntoIterator<Item = Element<'a, Message>>,
) -> Toolbar<'a, Message> {
    Toolbar {
        items: items.into_iter().collect(),
        floating: false,
        vertical: false,
    }
}
impl<Message> Toolbar<'_, Message> {
    pub fn floating(mut self, floating: bool) -> Self {
        self.floating = floating;
        self
    }
    pub fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }
}
impl<'a, Message: 'a> From<Toolbar<'a, Message>> for Element<'a, Message> {
    fn from(toolbar: Toolbar<'a, Message>) -> Self {
        let items: Element<'a, Message> = if toolbar.vertical {
            widget::Column::with_children(toolbar.items)
                .spacing(8)
                .into()
        } else {
            widget::Row::with_children(toolbar.items)
                .spacing(8)
                .align_y(iced::Alignment::Center)
                .into()
        };
        crate::focus::group(crate::elevation::elevated(
            widget::container(items)
                .padding(12)
                .width(if toolbar.floating || toolbar.vertical {
                    Length::Shrink
                } else {
                    Length::Fill
                })
                .style(move |theme: &Theme| widget::container::Style {
                    background: Some(theme.colors.surface_container.into()),
                    border: Border {
                        radius: if toolbar.floating { 32.0 } else { 0.0 }.into(),
                        ..Default::default()
                    },

                    ..Default::default()
                }),
            if toolbar.floating { 3.0 } else { 0.0 },
            if toolbar.floating { 32.0 } else { 0.0 },
        ))
    }
}
pub struct FabMenuItem<'a, Message> {
    label: String,
    icon: Element<'a, Message>,
    action: Message,
}
impl<'a, Message> FabMenuItem<'a, Message> {
    pub fn new(
        label: impl Into<String>,
        icon: impl Into<Element<'a, Message>>,
        action: Message,
    ) -> Self {
        Self {
            label: label.into(),
            icon: icon.into(),
            action,
        }
    }
}
/// Keep the menu in the tree while closed for exit animation. App messages close
/// it after choosing an action. Position this composition at the bottom end.
pub fn fab_menu<'a, Message: Clone + 'a>(
    icon: impl Into<Element<'a, Message>>,
    open: bool,
    toggle: Message,
    items: impl IntoIterator<Item = FabMenuItem<'a, Message>>,
) -> Element<'a, Message> {
    let items = widget::Column::with_children(items.into_iter().map(|item| {
        Button::new(
            widget::row![
                typography(item.label, TypeScale::LabelLarge),
                widget::container(item.icon).center_x(24).center_y(24)
            ]
            .spacing(16)
            .align_y(iced::Alignment::Center),
        )
        .variant(crate::ButtonVariant::Tonal)
        .height(56)
        .on_press(item.action)
        .expressive(true)
        .into()
    }))
    .spacing(8)
    .align_x(iced::Alignment::End);
    crate::focus::group(
        widget::column![
            crate::reveal::reveal(
                widget::column![items, widget::space().height(8)],
                open,
                false
            ),
            crate::fab(if open {
                Element::new(crate::glyph::Glyph::Close)
            } else {
                icon.into()
            })
            .on_press(toggle)
        ]
        .align_x(iced::Alignment::End),
    )
}
