//! Controlled, mutually exclusive choices with Material circular state layers.
use crate::{Element, checkbox::Checkbox};
use iced::Length;
use std::rc::Rc;

pub struct Radio<'a, Value, Message> {
    control: Checkbox<'a, Message>,
    value: Value,
}

/// Select one value. Selecting the current value produces no message.
pub fn radio<'a, Value: PartialEq, Message>(
    label: impl iced::widget::text::IntoFragment<'a>,
    value: Value,
    selected: Option<Value>,
) -> Radio<'a, Value, Message> {
    Radio {
        control: crate::checkbox(selected.as_ref() == Some(&value))
            .into_radio()
            .label(label),
        value,
    }
}
impl<'a, Value: Clone + 'a, Message: 'a> Radio<'a, Value, Message> {
    pub fn on_select(mut self, handler: impl Fn(Value) -> Message + 'a) -> Self {
        let value = self.value.clone();
        self.control = self.control.on_toggle(move |_| handler(value.clone()));
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.control = self.control.disabled(disabled);
        self
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.control = self.control.width(width);
        self
    }
}
impl<'a, Value: 'a, Message: 'a> From<Radio<'a, Value, Message>> for Element<'a, Message> {
    fn from(radio: Radio<'a, Value, Message>) -> Self {
        radio.control.into()
    }
}

#[derive(Clone, Debug)]
pub struct RadioOption<Value> {
    pub value: Value,
    pub label: String,
    pub disabled: bool,
}
impl<Value> RadioOption<Value> {
    pub fn new(value: Value, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
            disabled: false,
        }
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}
pub struct RadioGroup<'a, Value, Message> {
    options: Vec<RadioOption<Value>>,
    selected: Option<Value>,
    on_select: Option<Rc<dyn Fn(Value) -> Message + 'a>>,
    disabled: bool,
    width: Length,
    spacing: f32,
}
pub fn radio_group<'a, Value, Message>(
    options: impl IntoIterator<Item = RadioOption<Value>>,
    selected: Option<Value>,
) -> RadioGroup<'a, Value, Message> {
    RadioGroup {
        options: options.into_iter().collect(),
        selected,
        on_select: None,
        disabled: false,
        width: Length::Shrink,
        spacing: 0.0,
    }
}
impl<'a, Value, Message> RadioGroup<'a, Value, Message> {
    pub fn on_select(mut self, handler: impl Fn(Value) -> Message + 'a) -> Self {
        self.on_select = Some(Rc::new(handler));
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
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing.max(0.0);
        self
    }
}
impl<'a, Value: Clone + PartialEq + 'a, Message: 'a> From<RadioGroup<'a, Value, Message>>
    for Element<'a, Message>
{
    fn from(group: RadioGroup<'a, Value, Message>) -> Self {
        let active = group
            .options
            .iter()
            .filter(|i| !i.disabled)
            .position(|i| Some(&i.value) == group.selected.as_ref())
            .unwrap_or(0);
        crate::focus::group_with_active(
            iced::widget::Column::with_children(group.options.into_iter().map(|option| {
                let mut choice = radio(option.label, option.value, group.selected.clone());
                if let Some(handler) = &group.on_select {
                    let handler = Rc::clone(handler);
                    choice = choice.on_select(move |value| handler(value));
                }
                choice
                    .disabled(group.disabled || option.disabled)
                    .width(group.width)
                    .into()
            }))
            .spacing(group.spacing)
            .width(group.width),
            active,
        )
    }
}
