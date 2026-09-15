//! Baseline Material chips. Selection and removal are application-owned.
mod selection;
use crate::{Button, ButtonVariant, Element, TypeScale, glyph::Glyph, typography};
use iced::{Alignment, Length, widget};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipVariant {
    Assist,
    Suggestion,
    Filter,
    Input,
}

/// A compact action with an optional independent remove target.
/// Omit callbacks to disable actions. Icons should be passive, 18px content.
pub struct Chip<'a, Message> {
    label: String,
    variant: ChipVariant,
    selected: bool,
    leading: Option<Element<'a, Message>>,
    on_press: Option<Message>,
    on_remove: Option<Message>,
    disabled: bool,
    width: Length,
    elevated: bool,
    avatar: bool,
    dropdown: bool,
}
pub fn assist_chip<'a, Message>(label: impl Into<String>) -> Chip<'a, Message> {
    Chip::new(label, ChipVariant::Assist, false)
}
pub fn suggestion_chip<'a, Message>(label: impl Into<String>) -> Chip<'a, Message> {
    Chip::new(label, ChipVariant::Suggestion, false)
}
pub fn filter_chip<'a, Message>(label: impl Into<String>, selected: bool) -> Chip<'a, Message> {
    Chip::new(label, ChipVariant::Filter, selected)
}
pub fn input_chip<'a, Message>(label: impl Into<String>) -> Chip<'a, Message> {
    Chip::new(label, ChipVariant::Input, false)
}
impl<'a, Message> Chip<'a, Message> {
    fn new(label: impl Into<String>, variant: ChipVariant, selected: bool) -> Self {
        Self {
            label: label.into(),
            variant,
            selected,
            leading: None,
            on_press: None,
            on_remove: None,
            disabled: false,
            width: Length::Shrink,
            elevated: false,
            avatar: false,
            dropdown: false,
        }
    }
    pub fn elevated(mut self, elevated: bool) -> Self {
        self.elevated = elevated;
        self
    }
    pub fn avatar(mut self, avatar: impl Into<Element<'a, Message>>) -> Self {
        self.leading = Some(avatar.into());
        self.avatar = true;
        self
    }
    pub fn dropdown(mut self, dropdown: bool) -> Self {
        self.dropdown = dropdown;
        self
    }
    pub fn leading(mut self, icon: impl Into<Element<'a, Message>>) -> Self {
        self.leading = Some(icon.into());
        self
    }
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }
    /// Input chips expose removal separately from selection/activation.
    pub fn on_remove(mut self, message: Message) -> Self {
        self.on_remove = Some(message);
        self
    }
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
}
impl<'a, Message: Clone + 'a> From<Chip<'a, Message>> for Element<'a, Message> {
    fn from(chip: Chip<'a, Message>) -> Self {
        let selectable = matches!(chip.variant, ChipVariant::Filter | ChipVariant::Input);
        let selected = selectable && chip.selected;
        let enabled = !chip.disabled
            && (chip.on_press.is_some()
                || (chip.variant == ChipVariant::Input && chip.on_remove.is_some()));
        let variant = chip.variant;
        // Filter chips reserve their check slot so selection does not move adjacent chips.
        let has_leading = chip.leading.is_some() || chip.variant == ChipVariant::Filter;
        let mut content = widget::Row::new().spacing(8).align_y(Alignment::Center);
        if has_leading {
            let leading = chip
                .leading
                .unwrap_or_else(|| widget::space().width(18).height(18).into());
            // Input chips retain their avatar/icon when selected. Filter chips
            // exchange their leading icon and check within a stable 18px slot.
            let mark = if variant == ChipVariant::Filter {
                selection::icon(leading, selected, enabled)
            } else {
                leading
            };
            content = content.push(
                widget::container(mark)
                    .center_x(if chip.avatar { 24 } else { 18 })
                    .center_y(if chip.avatar { 24 } else { 18 })
                    .style(move |theme: &crate::Theme| widget::container::Style {
                        text_color: Some(if !enabled {
                            crate::theme::alpha(theme.colors.on_surface, 0.38)
                        } else if selected && variant == ChipVariant::Filter {
                            theme.colors.on_secondary_container
                        } else if !selected && variant == ChipVariant::Input {
                            theme.colors.on_surface_variant
                        } else {
                            theme.colors.primary
                        }),
                        ..Default::default()
                    }),
            );
        }
        content = content.push(
            typography(chip.label, TypeScale::LabelLarge).wrapping(widget::text::Wrapping::None),
        );
        if chip.dropdown {
            content = content.push(Element::new(Glyph::Down));
        }
        let remove = if chip.variant == ChipVariant::Input {
            chip.on_remove
        } else {
            None
        };
        let removable = remove.is_some();
        if removable {
            content = content.push(
                Button::new(Element::new(Glyph::Close))
                    .variant(ButtonVariant::Text)
                    .palette(move |theme| {
                        (
                            iced::Color::TRANSPARENT,
                            if selected {
                                theme.colors.on_secondary_container
                            } else {
                                theme.colors.on_surface_variant
                            },
                        )
                    })
                    .height(32)
                    .width(32)
                    .padding(7)
                    .radius(8.0)
                    .on_press_maybe(remove)
                    .disabled(chip.disabled),
            );
        }
        Button::new(content)
            .variant(if chip.elevated {
                ButtonVariant::Elevated
            } else {
                ButtonVariant::Outlined
            })
            .chip_style(selected, chip.variant == ChipVariant::Assist)
            .height(32)
            .radius(if chip.avatar && variant == ChipVariant::Input {
                16.0
            } else {
                8.0
            })
            .width(chip.width)
            .padding(iced::Padding {
                top: 0.0,
                bottom: 0.0,
                left: if chip.avatar && variant == ChipVariant::Input {
                    4.0
                } else if has_leading {
                    8.0
                } else {
                    16.0
                },
                right: if removable {
                    0.0
                } else if chip.dropdown {
                    8.0
                } else {
                    16.0
                },
            })
            .on_press_maybe(chip.on_press)
            .disabled(chip.disabled)
            .interactive_children(chip.disabled || !removable)
            .into()
    }
}
