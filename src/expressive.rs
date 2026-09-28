//! Expressive action compositions, sharing the existing Material interaction layer.
use crate::{Button, Element, MenuItem, Theme, TypeScale, typography};
use iced::{Border, Length, widget};
mod group;
/// Equal-width actions with retained press expansion and neighbor compression.
/// Children keep their widget/focus state while their layout changes.
pub struct ButtonGroup<'a, Message> {
    buttons: Vec<Button<'a, Message>>,
    connected: bool,
    expanded_ratio: f32,
}
/// Group actions with a 15% requested press expansion. The resting target is
/// bounded by neighboring padding; springs retain overshoot and release rebound
/// while spare padding protects the original space available to child content.
pub fn button_group<'a, Message>(
    buttons: impl IntoIterator<Item = Button<'a, Message>>,
) -> ButtonGroup<'a, Message> {
    ButtonGroup {
        buttons: buttons.into_iter().collect(),
        connected: false,
        expanded_ratio: 0.15,
    }
}
impl<Message> ButtonGroup<'_, Message> {
    /// Use 2px gaps and connected corners instead of 12px gaps (default: false).
    pub fn connected(mut self, connected: bool) -> Self {
        self.connected = connected;
        self
    }
    /// Requested width growth as a fraction of the resting button width.
    /// Zero disables deformation. Values are clamped to 0..=1; non-finite values
    /// are ignored. The target is limited by immediate neighbors' padding.
    /// Spring overshoot may use their remaining padding, but never content space.
    pub fn expanded_ratio(mut self, ratio: f32) -> Self {
        if ratio.is_finite() {
            self.expanded_ratio = ratio.clamp(0.0, 1.0);
        }
        self
    }
}
impl<'a, Message: Clone + 'a> From<ButtonGroup<'a, Message>> for Element<'a, Message> {
    fn from(group: ButtonGroup<'a, Message>) -> Self {
        let last = group.buttons.len().saturating_sub(1);
        let buttons = group
            .buttons
            .into_iter()
            .enumerate()
            .map(|(i, button)| {
                let button = button.grouped().width(Length::Fill);
                if group.connected {
                    button
                        .corner_radius(
                            Border::default()
                                .rounded(iced::border::Radius {
                                    top_left: if i == 0 { 999.0 } else { 8.0 },
                                    bottom_left: if i == 0 { 999.0 } else { 8.0 },
                                    top_right: if i == last { 999.0 } else { 8.0 },
                                    bottom_right: if i == last { 999.0 } else { 8.0 },
                                })
                                .radius,
                        )
                        .pressed_corners(iced::border::Radius {
                            top_left: if i == 0 { 999.0 } else { 4.0 },
                            bottom_left: if i == 0 { 999.0 } else { 4.0 },
                            top_right: if i == last { 999.0 } else { 4.0 },
                            bottom_right: if i == last { 999.0 } else { 4.0 },
                        })
                } else {
                    button
                }
            })
            .collect();
        crate::focus::group(Element::new(group::Group {
            buttons,
            spacing: if group.connected { 2.0 } else { 12.0 },
            ratio: group.expanded_ratio,
        }))
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
/// A row or column of controls sharing one Tab stop. Unhandled arrows/Home/End
/// move focus; focused editors and sliders retain their editing keys. Tab exits
/// the group. Defaults to a horizontal, non-floating toolbar.
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
