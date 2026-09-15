//! Composable variants built on the shared focus, state, and popup primitives.
use crate::{Button, ButtonVariant, Element, SurfaceVariant, Theme, TypeScale, typography};
use iced::{Border, Length, widget};
pub struct Card<'a, Message> {
    content: Element<'a, Message>,
    variant: SurfaceVariant,
    action: Option<Message>,
    width: Length,
    disabled: bool,
    dragged: bool,
}
pub fn card<'a, Message>(content: impl Into<Element<'a, Message>>) -> Card<'a, Message> {
    Card {
        content: content.into(),
        variant: SurfaceVariant::Filled,
        action: None,
        width: Length::Fill,
        disabled: false,
        dragged: false,
    }
}
impl<'a, Message> Card<'a, Message> {
    pub fn variant(mut self, variant: SurfaceVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn on_press(mut self, message: Message) -> Self {
        self.action = Some(message);
        self
    }
    /// Disable the card action and all interactive children.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Application-controlled drag appearance. Suppresses activation while dragging;
    /// the application owns drag recognition, positioning, and drop behavior.
    pub fn dragged(mut self, dragged: bool) -> Self {
        self.dragged = dragged;
        self
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
}
impl<'a, Message: Clone + 'a> From<Card<'a, Message>> for Element<'a, Message> {
    fn from(card: Card<'a, Message>) -> Self {
        Button::new(widget::container(card.content).width(Length::Fill))
            .variant(match card.variant {
                SurfaceVariant::Outlined => ButtonVariant::Outlined,
                SurfaceVariant::Elevated => ButtonVariant::Elevated,
                SurfaceVariant::Filled => ButtonVariant::Text,
            })
            .palette(move |theme| {
                (
                    match card.variant {
                        SurfaceVariant::Outlined => theme.colors.surface,
                        SurfaceVariant::Filled => theme.colors.surface_container_highest,
                        SurfaceVariant::Elevated => theme.colors.surface_container_low,
                    },
                    theme.colors.on_surface,
                )
            })
            .outline_color(|theme, focused| {
                if focused {
                    theme.colors.on_surface
                } else {
                    theme.colors.outline_variant
                }
            })
            .elevation_levels(
                u8::from(card.variant == SurfaceVariant::Elevated),
                if card.variant == SurfaceVariant::Elevated {
                    2
                } else {
                    1
                },
            )
            .radius(12.0)
            .padding(16)
            .width(card.width)
            .interactive_children(card.disabled || card.dragged)
            .on_press_maybe(if card.disabled || card.dragged {
                None
            } else {
                card.action
            })
            .card_state(card.variant, card.disabled, card.dragged)
            .into()
    }
}
/// A 6px status dot; an anchored badge preserves its target's size.
pub fn badge_dot<'a, Message: 'a>() -> widget::Container<'a, Message, Theme> {
    widget::container(widget::space())
        .width(6)
        .height(6)
        .style(|theme: &Theme| widget::container::Style {
            background: Some(theme.colors.error.into()),
            border: Border {
                radius: 3.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
}
pub fn badged<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    count: Option<u32>,
) -> Element<'a, Message> {
    widget::stack![
        content.into(),
        widget::container(match count {
            Some(count) => crate::badge(count).into(),
            None => Element::from(badge_dot()),
        })
        .align_right(Length::Fill)
        .align_top(Length::Fill)
    ]
    .into()
}
pub fn vertical_divider<'a>() -> widget::Rule<'a, Theme> {
    widget::rule::vertical(1)
}
/// Rich hints open on click or keyboard activation, so their actions stay reachable.
pub fn rich_tooltip<'a, Message: Clone + 'a>(
    trigger: impl Into<Element<'a, Message>>,
    title: impl Into<String>,
    body: impl Into<Element<'a, Message>>,
) -> crate::Menu<'a, Message> {
    crate::menu::rich_popup(
        trigger,
        crate::elevation::elevated(
            widget::container(crate::staged::part(
                widget::column![typography(title.into(), TypeScale::TitleSmall), body.into()]
                    .spacing(12),
                crate::staged::Part::Popup(false),
            ))
            .padding(16)
            .width(Length::Fill)
            .style(|theme: &Theme| widget::container::Style {
                background: Some(theme.colors.surface_container.into()),
                text_color: Some(theme.colors.on_surface_variant),
                border: Border {
                    radius: 12.0.into(),
                    ..Default::default()
                },

                ..Default::default()
            }),
            2.0,
            12.0,
        ),
    )
}
