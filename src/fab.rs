//! Baseline Material floating action buttons, composed from the shared button.
use crate::{Button, Element, Theme, TypeScale, tokens, typography};
use iced::{Alignment, Length, widget};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FabSize {
    Small,
    #[default]
    Regular,
    Large,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FabColor {
    #[default]
    Primary,
    Secondary,
    Tertiary,
    Surface,
}
/// A primary action with passive icon content and optional label. The app owns
/// placement; use ordinary iced layout/stack widgets to float it over content.
pub struct Fab<'a, Message> {
    icon: Element<'a, Message>,
    label: Option<Element<'a, Message>>,
    size: FabSize,
    color: FabColor,
    action: Option<Message>,
    extended: bool,
    lowered: bool,
}
pub fn fab<'a, Message: 'a>(icon: impl Into<Element<'a, Message>>) -> Fab<'a, Message> {
    Fab {
        icon: icon.into(),
        label: None,
        size: FabSize::Regular,
        color: FabColor::Primary,
        action: None,
        extended: true,
        lowered: false,
    }
}
/// Extended FABs have a 56px height, a 24px icon, and LabelLarge typography.
pub fn extended_fab<'a, Message: 'a>(
    icon: impl Into<Element<'a, Message>>,
    label: impl widget::text::IntoFragment<'a>,
) -> Fab<'a, Message> {
    Fab {
        label: Some(typography(label, TypeScale::LabelLarge).into()),
        ..fab(icon)
    }
}
impl<'a, Message: 'a> Fab<'a, Message> {
    /// Icon-only size. Supply a 24px icon, or 36px for Large; content is centered
    /// in that slot without distorting arbitrary child widgets. Extended FABs
    /// always use Regular dimensions.
    pub fn extended(mut self, extended: bool) -> Self {
        self.extended = extended;
        self
    }
    pub fn size(mut self, size: FabSize) -> Self {
        self.size = size;
        self
    }
    /// Use elevation 1 at rest and 2 on hover for a less prominent FAB.
    pub fn lowered(mut self, lowered: bool) -> Self {
        self.lowered = lowered;
        self
    }
    pub fn color(mut self, color: FabColor) -> Self {
        self.color = color;
        self
    }
    pub fn on_press(mut self, message: Message) -> Self {
        self.action = Some(message);
        self
    }
    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.action = message;
        self
    }
    /// Omitting the action also disables the button. Disabled FABs are a library
    /// convenience; apps should normally hide unavailable primary actions.
    pub fn disabled(mut self, disabled: bool) -> Self {
        if disabled {
            self.action = None;
        }
        self
    }
}
impl<'a, Message: Clone + 'a> From<Fab<'a, Message>> for Element<'a, Message> {
    fn from(value: Fab<'a, Message>) -> Self {
        let size = if value.label.is_some() {
            FabSize::Regular
        } else {
            value.size
        };
        let (dimension, radius, icon_size) = match size {
            FabSize::Small => (tokens::size::FAB_SMALL, tokens::shape::MEDIUM, 24.0),
            FabSize::Regular => (tokens::size::FAB, tokens::shape::LARGE, 24.0),
            FabSize::Large => (tokens::size::FAB_LARGE, tokens::shape::EXTRA_LARGE, 36.0),
        };
        let icon = widget::container(value.icon)
            .center_x(icon_size)
            .center_y(icon_size);
        let extended = value.label.is_some();
        let content: Element<'a, Message> = if let Some(label) = value.label {
            widget::row![
                icon,
                crate::reveal::reveal(
                    widget::row![widget::space().width(12), label, widget::space().width(4)],
                    value.extended,
                    true
                )
            ]
            .spacing(0)
            .align_y(Alignment::Center)
            .into()
        } else {
            icon.into()
        };
        Button::new(content)
            .width(if extended {
                Length::Shrink
            } else {
                Length::Fixed(dimension)
            })
            .height(dimension)
            .padding(if extended {
                iced::Padding {
                    top: 0.0,
                    bottom: 0.0,
                    left: 16.0,
                    right: 16.0,
                }
            } else {
                iced::Padding::ZERO
            })
            .radius(radius)
            .elevated_style()
            .elevation_levels(
                if value.lowered { 1 } else { 3 },
                if value.lowered { 2 } else { 4 },
            )
            .palette(move |theme: &Theme| {
                let c = theme.colors;
                match value.color {
                    FabColor::Primary => (c.primary_container, c.on_primary_container),
                    FabColor::Secondary => (c.secondary_container, c.on_secondary_container),
                    FabColor::Tertiary => (c.tertiary_container, c.on_tertiary_container),
                    FabColor::Surface => (
                        if value.lowered {
                            c.surface_container_low
                        } else {
                            c.surface_container_high
                        },
                        c.primary,
                    ),
                }
            })
            .on_press_maybe(value.action)
            .into()
    }
}
