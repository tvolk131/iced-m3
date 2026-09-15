//! Small components composed from shared tokens, iced widgets, and Button.
use crate::{
    Button, ButtonVariant, Element, Theme,
    theme::alpha,
    tokens::{self, TypeScale},
};
use iced::{Border, Color, Length, widget};

/// Material text styles. Text remains an ordinary iced text widget.
pub fn typography<'a>(
    content: impl widget::text::IntoFragment<'a>,
    scale: TypeScale,
) -> widget::Text<'a, Theme> {
    crate::fonts::ensure_loaded();
    let (size, line) = scale.metrics();
    widget::text(content)
        .size(size)
        .line_height(iced::widget::text::LineHeight::Absolute(line.into()))
        .font(scale.font())
        .shaping(iced::widget::text::Shaping::Advanced)
        .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SurfaceVariant {
    #[default]
    Filled,
    Outlined,
    Elevated,
}
/// A card/surface accepting arbitrary iced content.
pub struct Surface<'a, Message> {
    content: Element<'a, Message>,
    variant: SurfaceVariant,
    padding: iced::Padding,
    width: Length,
}
pub fn surface<'a, Message: 'a>(content: impl Into<Element<'a, Message>>) -> Surface<'a, Message> {
    Surface {
        content: content.into(),
        variant: SurfaceVariant::Filled,
        padding: tokens::spacing::LG.into(),
        width: Length::Fill,
    }
}
impl<Message> Surface<'_, Message> {
    pub fn variant(mut self, variant: SurfaceVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn padding(mut self, padding: impl Into<iced::Padding>) -> Self {
        self.padding = padding.into();
        self
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
}
impl<'a, Message: 'a> From<Surface<'a, Message>> for Element<'a, Message> {
    fn from(value: Surface<'a, Message>) -> Self {
        crate::elevation::elevated(
            widget::container(value.content)
                .padding(value.padding)
                .width(value.width)
                .style(move |t: &Theme| {
                    let c = t.colors;
                    widget::container::Style {
                        background: Some(
                            match value.variant {
                                SurfaceVariant::Outlined => c.surface,
                                SurfaceVariant::Filled => c.surface_container_highest,
                                SurfaceVariant::Elevated => c.surface_container_low,
                            }
                            .into(),
                        ),
                        text_color: Some(c.on_surface),
                        border: Border {
                            radius: tokens::shape::MEDIUM.into(),
                            width: if value.variant == SurfaceVariant::Outlined {
                                1.0
                            } else {
                                0.0
                            },
                            color: c.outline_variant,
                        },

                        ..Default::default()
                    }
                }),
            if value.variant == SurfaceVariant::Elevated {
                1.0
            } else {
                0.0
            },
            tokens::shape::MEDIUM,
        )
    }
}

/// A horizontal semantic divider.
pub fn divider<'a>() -> widget::Rule<'a, Theme> {
    widget::rule::horizontal(tokens::size::OUTLINE)
}

/// A circular 40px action using the same finite press effect as Button.
/// Content is centered in a 24×24px area. Prefer a vector icon or an icon font
/// with square metrics; ordinary text glyphs include baseline/line-height space.
pub fn icon_button<'a, Message: 'a>(icon: impl Into<Element<'a, Message>>) -> Button<'a, Message> {
    Button::new(
        widget::container(icon)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .icon_style()
    .variant(ButtonVariant::Text)
    .padding(8.0)
    .width(tokens::size::BUTTON)
    .height(tokens::size::BUTTON)
}

/// A selectable filter/assist chip. App state owns the selection.
pub fn chip<'a, Message: 'a>(
    label: impl widget::text::IntoFragment<'a>,
    selected: bool,
) -> Button<'a, Message> {
    Button::new(typography(label, TypeScale::Label))
        .variant(ButtonVariant::Outlined)
        .chip_style(selected, false)
        .padding([6.0, 16.0])
        .height(tokens::size::CHIP)
        .radius(tokens::shape::SMALL * 2.0)
}

/// A compact standalone count/status badge. Use in a row or an iced stack.
/// Counts above 99 are displayed as `99+`; zero is displayed deliberately.
pub fn badge<'a, Message: 'a>(count: u32) -> widget::Container<'a, Message, Theme> {
    let label = if count > 99 {
        "99+".into()
    } else {
        count.to_string()
    };
    widget::container(typography(label, TypeScale::LabelSmall))
        .padding([0.0, 5.0])
        .center_x(Length::Shrink)
        .style(|theme: &Theme| widget::container::Style {
            background: Some(theme.colors.error.into()),
            text_color: Some(theme.colors.on_error),
            border: Border {
                radius: tokens::shape::FULL.into(),
                ..Default::default()
            },
            ..Default::default()
        })
}

/// Compatibility re-export; the animated checkbox now has its own builder.
pub use crate::checkbox::checkbox;

/// A Material switch: 52×32 track, changing thumb size, and internal animation.
pub struct Switch<'a, Message> {
    icons: bool,
    checked: bool,
    label: Option<String>,
    on_toggle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
    width: Length,
}
pub fn switch<'a, Message>(checked: bool) -> Switch<'a, Message> {
    Switch {
        icons: false,
        checked,
        label: None,
        on_toggle: None,
        width: Length::Shrink,
    }
}
impl<'a, Message> Switch<'a, Message> {
    pub fn icons(mut self, enabled: bool) -> Self {
        self.icons = enabled;
        self
    }
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
    pub fn on_toggle(mut self, callback: impl Fn(bool) -> Message + 'a) -> Self {
        self.on_toggle = Some(Box::new(callback));
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        if disabled {
            self.on_toggle = None;
        }
        self
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
}
impl<'a, Message: 'a> From<Switch<'a, Message>> for Element<'a, Message> {
    fn from(value: Switch<'a, Message>) -> Self {
        let mut control = crate::checkbox(value.checked)
            .into_switch()
            .switch_icons(value.icons)
            .width(value.width);
        if let Some(label) = value.label {
            control = control.label(label);
        }
        if let Some(handler) = value.on_toggle {
            control = control.on_toggle(handler);
        }
        control.into()
    }
}

impl widget::checkbox::Catalog for Theme {
    type Class<'a> = widget::checkbox::StyleFn<'a, Self>;
    fn default<'a>() -> Self::Class<'a> {
        Box::new(|theme, status| {
            use widget::checkbox::Status;
            let (checked, hovered, enabled) = match status {
                Status::Active { is_checked } => (is_checked, false, true),
                Status::Hovered { is_checked } => (is_checked, true, true),
                Status::Disabled { is_checked } => (is_checked, false, false),
            };
            let c = theme.colors;
            let outline = if enabled {
                if checked {
                    c.primary
                } else {
                    c.on_surface_variant
                }
            } else {
                alpha(c.on_surface, 0.38)
            };
            widget::checkbox::Style {
                background: if checked {
                    outline
                } else if hovered {
                    alpha(c.primary, 0.08)
                } else {
                    Color::TRANSPARENT
                }
                .into(),
                icon_color: if enabled { c.on_primary } else { c.surface },
                border: Border {
                    radius: 2.0.into(),
                    width: 2.0,
                    color: outline,
                },
                text_color: Some(if enabled {
                    c.on_surface
                } else {
                    alpha(c.on_surface, 0.38)
                }),
            }
        })
    }
    fn style(
        &self,
        class: &Self::Class<'_>,
        status: widget::checkbox::Status,
    ) -> widget::checkbox::Style {
        class(self, status)
    }
}
impl widget::rule::Catalog for Theme {
    type Class<'a> = widget::rule::StyleFn<'a, Self>;
    fn default<'a>() -> Self::Class<'a> {
        Box::new(|theme| widget::rule::Style {
            color: theme.colors.outline_variant,
            radius: 0.0.into(),
            fill_mode: widget::rule::FillMode::Full,
            snap: true,
        })
    }
    fn style(&self, class: &Self::Class<'_>) -> widget::rule::Style {
        class(self)
    }
}
