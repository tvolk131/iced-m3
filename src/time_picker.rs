//! Controlled time pickers. Values are local wall-clock times without a timezone.
use crate::{ButtonVariant, Element, TextField, Theme, TypeScale, focus, text_field, typography};
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, renderer,
    widget::{Operation, Tree, tree},
};
use iced::{
    Border, Event, Length, Point, Rectangle, Renderer, Size, keyboard, mouse, widget, window,
};
use std::{fmt, str::FromStr};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time {
    hour: u8,
    minute: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidTime;
impl fmt::Display for InvalidTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Enter a valid time as HH:MM (00:00–23:59)")
    }
}
impl std::error::Error for InvalidTime {}
impl Time {
    pub fn new(hour: u8, minute: u8) -> Result<Self, InvalidTime> {
        if hour < 24 && minute < 60 {
            Ok(Self { hour, minute })
        } else {
            Err(InvalidTime)
        }
    }
    pub fn hour(self) -> u8 {
        self.hour
    }
    pub fn minute(self) -> u8 {
        self.minute
    }
    pub fn is_pm(self) -> bool {
        self.hour >= 12
    }
    pub fn hour12(self) -> u8 {
        let hour = self.hour % 12;
        if hour == 0 { 12 } else { hour }
    }
    pub fn with_period(self, pm: bool) -> Self {
        Self {
            hour: self.hour % 12 + if pm { 12 } else { 0 },
            ..self
        }
    }
    fn adjust(self, part: TimePart, delta: i32) -> Self {
        match part {
            TimePart::Hour => Self {
                hour: (self.hour as i32 + delta).rem_euclid(24) as u8,
                ..self
            },
            TimePart::Minute => Self {
                minute: (self.minute as i32 + delta).rem_euclid(60) as u8,
                ..self
            },
        }
    }
}
impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.hour, self.minute)
    }
}
impl FromStr for Time {
    type Err = InvalidTime;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let b = value.as_bytes();
        if b.len() != 5
            || b[2] != b':'
            || b.iter()
                .enumerate()
                .any(|(i, b)| i != 2 && !b.is_ascii_digit())
        {
            return Err(InvalidTime);
        }
        Self::new(
            value[..2].parse().map_err(|_| InvalidTime)?,
            value[3..].parse().map_err(|_| InvalidTime)?,
        )
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimePart {
    #[default]
    Hour,
    Minute,
}
pub struct TimePicker<'a, Message> {
    value: Time,
    part: TimePart,
    format24: bool,
    title: String,
    on_change: Option<Box<dyn Fn(Time) -> Message + 'a>>,
    on_confirm: Option<Box<dyn Fn(Time) -> Message + 'a>>,
    on_cancel: Option<Message>,
    on_part: Option<Box<dyn Fn(TimePart) -> Message + 'a>>,
    input_mode: bool,
    on_toggle_input: Option<Message>,
    hour_input: Option<(String, Box<dyn Fn(String) -> Message + 'a>)>,
    minute_input: Option<(String, Box<dyn Fn(String) -> Message + 'a>)>,
    input_period: Option<(bool, Box<dyn Fn(bool) -> Message + 'a>)>,
}
pub fn time_picker<'a, Message>(value: Time, part: TimePart) -> TimePicker<'a, Message> {
    TimePicker {
        value,
        part,
        format24: false,
        title: "Select time".into(),
        on_change: None,
        on_confirm: None,
        on_cancel: None,
        on_part: None,
        input_mode: false,
        on_toggle_input: None,
        hour_input: None,
        minute_input: None,
        input_period: None,
    }
}
impl<'a, Message> TimePicker<'a, Message> {
    /// Add an OK action. Selection edits still use `on_change`; keep a separate
    /// draft in application state when confirmation must be transactional.
    /// Numeric OK/Enter validates the draft and sends this callback only.
    pub fn on_confirm(mut self, f: impl Fn(Time) -> Message + 'a) -> Self {
        self.on_confirm = Some(Box::new(f));
        self
    }
    /// Add a Cancel action. Discarding the controlled draft is application-owned.
    pub fn on_cancel(mut self, message: Message) -> Self {
        self.on_cancel = Some(message);
        self
    }

    /// Switch between the dial and numeric entry. Draft text remains caller-owned.
    pub fn input_mode(mut self, input: bool) -> Self {
        self.input_mode = input;
        self
    }
    pub fn on_toggle_input(mut self, message: Message) -> Self {
        self.on_toggle_input = Some(message);
        self
    }
    /// Hour draft: 0–23 in 24-hour mode, 1–12 in 12-hour mode.
    pub fn hour_input(mut self, text: &str, on_input: impl Fn(String) -> Message + 'a) -> Self {
        self.hour_input = Some((text.into(), Box::new(on_input)));
        self
    }
    pub fn minute_input(mut self, text: &str, on_input: impl Fn(String) -> Message + 'a) -> Self {
        self.minute_input = Some((text.into(), Box::new(on_input)));
        self
    }
    /// Separate AM/PM draft, so changing period does not commit the time early.
    pub fn input_period(mut self, pm: bool, on_input: impl Fn(bool) -> Message + 'a) -> Self {
        self.input_period = Some((pm, Box::new(on_input)));
        self
    }
    /// Validate the numeric draft without committing it. Apply/Enter uses this
    /// same result and emits `on_change` only when both fields are valid.
    pub fn input_time(&self) -> Result<Time, InvalidTime> {
        let hour = self
            .hour_input
            .as_ref()
            .map(|(s, _)| number(s))
            .unwrap_or(Ok(if self.format24 {
                self.value.hour
            } else {
                self.value.hour12()
            }))?;
        let minute = self
            .minute_input
            .as_ref()
            .map(|(s, _)| number(s))
            .unwrap_or(Ok(self.value.minute))?;
        let pm = self
            .input_period
            .as_ref()
            .map_or(self.value.is_pm(), |(pm, _)| *pm);
        let hour = if self.format24 {
            hour
        } else {
            if !(1..=12).contains(&hour) {
                return Err(InvalidTime);
            }
            hour % 12 + if pm { 12 } else { 0 }
        };
        Time::new(hour, minute)
    }
    pub fn on_change(mut self, f: impl Fn(Time) -> Message + 'a) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }
    pub fn on_part(mut self, f: impl Fn(TimePart) -> Message + 'a) -> Self {
        self.on_part = Some(Box::new(f));
        self
    }
    pub fn format24(mut self, format24: bool) -> Self {
        self.format24 = format24;
        self
    }
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
}
impl<'a, Message: Clone + 'a> From<TimePicker<'a, Message>> for Element<'a, Message> {
    fn from(picker: TimePicker<'a, Message>) -> Self {
        if picker.input_mode {
            return numeric_picker(picker);
        }
        let display = |value: u8, part| {
            crate::Button::new(typography(typography_time(value), TypeScale::DisplayLarge))
                .width(if picker.format24 { 114 } else { 96 })
                .height(80)
                .padding(0)
                .radius(8.0)
                .variant(if picker.part == part {
                    ButtonVariant::Tonal
                } else {
                    ButtonVariant::Text
                })
                .palette(move |theme| {
                    (
                        if picker.part == part {
                            theme.colors.primary_container
                        } else {
                            theme.colors.surface_container_highest
                        },
                        if picker.part == part {
                            theme.colors.on_primary_container
                        } else {
                            theme.colors.on_surface
                        },
                    )
                })
                .on_press_maybe(picker.on_part.as_ref().map(|f| f(part)))
        };
        let hour = if picker.format24 {
            picker.value.hour
        } else {
            picker.value.hour12()
        };
        let mut header = widget::row![
            display(hour, TimePart::Hour),
            typography(":", TypeScale::DisplayLarge),
            display(picker.value.minute, TimePart::Minute)
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);
        if !picker.format24 {
            header = header.push(period_selector(
                picker.value.is_pm(),
                picker
                    .on_change
                    .as_ref()
                    .map(|f| f(picker.value.with_period(false))),
                picker
                    .on_change
                    .as_ref()
                    .map(|f| f(picker.value.with_period(true))),
            ));
        }
        let labels = (0..if picker.format24 && picker.part == TimePart::Hour {
            24
        } else {
            12
        })
            .map(|i| {
                let value = if picker.part == TimePart::Minute {
                    i * 5
                } else if i == 0 {
                    12
                } else if i == 12 {
                    0
                } else {
                    i
                };
                let selected = if picker.part == TimePart::Minute {
                    picker.value.minute == value as u8
                } else if picker.format24 {
                    picker.value.hour == value as u8
                } else {
                    picker.value.hour12() == value as u8
                };
                typography(
                    if picker.part == TimePart::Minute {
                        format!("{value:02}")
                    } else {
                        value.to_string()
                    },
                    if i >= 12 {
                        TypeScale::BodyMedium
                    } else {
                        TypeScale::BodyLarge
                    },
                )
                .style(move |theme: &Theme| widget::text::Style {
                    color: Some(if selected {
                        theme.colors.on_primary
                    } else {
                        theme.colors.on_surface
                    }),
                })
                .into()
            })
            .collect();
        let clock: Element<'a, Message> = Element::new(Clock {
            value: picker.value,
            part: picker.part,
            format24: picker.format24,
            on_change: picker.on_change,
            on_part: picker.on_part,
            labels,
        });
        let mut content = widget::column![
            widget::column![
                typography(picker.title, TypeScale::LabelMedium).style(|theme: &Theme| {
                    widget::text::Style {
                        color: Some(theme.colors.on_surface_variant),
                    }
                }),
                header
            ]
            .spacing(12),
            widget::container(clock).center_x(Length::Fill)
        ]
        .spacing(24);
        if picker.on_toggle_input.is_some()
            || picker.on_cancel.is_some()
            || picker.on_confirm.is_some()
        {
            let mut actions = widget::row![].spacing(8).align_y(iced::Alignment::Center);
            if let Some(message) = picker.on_toggle_input {
                actions = actions.push(mode_toggle(false, message));
            }
            actions = actions.push(widget::space().width(Length::Fill));
            if let Some(message) = picker.on_cancel {
                actions = actions.push(
                    crate::button("Cancel")
                        .variant(ButtonVariant::Text)
                        .on_press(message),
                );
            }
            if let Some(confirm) = picker.on_confirm {
                actions = actions.push(
                    crate::button("OK")
                        .variant(ButtonVariant::Text)
                        .on_press(confirm(picker.value)),
                );
            }
            content = content.push(actions);
        }
        widget::container(content)
            .padding(iced::Padding {
                top: 16.0,
                right: 24.0,
                bottom: 24.0,
                left: 24.0,
            })
            .width(360)
            .style(|theme: &Theme| widget::container::Style {
                background: Some(theme.colors.surface_container_high.into()),
                text_color: Some(theme.colors.on_surface),
                border: Border {
                    radius: 28.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }
}
// One outer outline and one shared divider, including the selected half.
fn period_selector<'a, Message: Clone + 'a>(
    pm: bool,
    am: Option<Message>,
    pm_message: Option<Message>,
) -> Element<'a, Message> {
    let period = |is_pm, message| {
        crate::Button::new(typography(
            if is_pm { "PM" } else { "AM" },
            TypeScale::TitleMedium,
        ))
        .width(52)
        .height(40)
        .padding(0)
        .corner_radius(iced::border::Radius {
            top_left: if is_pm { 0.0 } else { 8.0 },
            top_right: if is_pm { 0.0 } else { 8.0 },
            bottom_left: if is_pm { 8.0 } else { 0.0 },
            bottom_right: if is_pm { 8.0 } else { 0.0 },
        })
        .variant(ButtonVariant::Text)
        .palette(move |theme| {
            if pm == is_pm {
                (
                    theme.colors.tertiary_container,
                    theme.colors.on_tertiary_container,
                )
            } else {
                (iced::Color::TRANSPARENT, theme.colors.on_surface_variant)
            }
        })
        .on_press_maybe(message)
    };
    let frame =
        widget::container(
            widget::rule::horizontal(1).style(|theme: &Theme| widget::rule::Style {
                color: theme.colors.outline,
                radius: 0.0.into(),
                fill_mode: widget::rule::FillMode::Full,
                snap: true,
            }),
        )
        .width(52)
        .center_y(80)
        .style(|theme: &Theme| widget::container::Style {
            border: Border {
                color: theme.colors.outline,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        });
    widget::stack![
        widget::column![period(false, am), period(true, pm_message)],
        frame
    ]
    .into()
}
fn number(value: &str) -> Result<u8, InvalidTime> {
    if value.is_empty() || value.len() > 2 || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err(InvalidTime);
    }
    value.parse().map_err(|_| InvalidTime)
}
fn mode_toggle<'a, Message: Clone + 'a>(input: bool, message: Message) -> Element<'a, Message> {
    crate::tooltip(
        crate::icon_button(Element::new(if input {
            crate::glyph::Glyph::Clock
        } else {
            crate::glyph::Glyph::Keyboard
        }))
        .on_press(message),
        if input {
            "Switch to clock"
        } else {
            "Type time"
        },
    )
    .into()
}
fn numeric_picker<'a, Message: Clone + 'a>(
    picker: TimePicker<'a, Message>,
) -> Element<'a, Message> {
    let valid = picker.input_time();
    let submit = valid.ok().and_then(|time| {
        picker
            .on_confirm
            .as_ref()
            .or(picker.on_change.as_ref())
            .map(|f| f(time))
    });
    let pm = picker
        .input_period
        .as_ref()
        .map_or(picker.value.is_pm(), |(pm, _)| *pm);
    let entry =
        |label, draft: Option<(String, Box<dyn Fn(String) -> Message + 'a>)>, fallback: u8| {
            let (value, handler) =
                draft.map_or_else(|| (format!("{fallback:02}"), None), |(s, f)| (s, Some(f)));
            let invalid = number(&value).is_err()
                || number(&value).is_ok_and(|n| {
                    if label == "Minute" {
                        n > 59
                    } else if picker.format24 {
                        n > 23
                    } else {
                        !(1..=12).contains(&n)
                    }
                });
            let input = widget::text_input("", &value)
                .font(crate::fonts::REGULAR)
                .size(57)
                .line_height(iced::advanced::text::LineHeight::Absolute(64.0.into()))
                .padding([8, 4])
                .align_x(iced::alignment::Horizontal::Center)
                .width(if picker.format24 { 114 } else { 96 })
                .on_input_maybe(handler)
                .on_submit_maybe(submit.clone())
                .style(move |theme: &Theme, status| {
                    let focused = matches!(status, widget::text_input::Status::Focused { .. });
                    let disabled = status == widget::text_input::Status::Disabled;
                    let c = theme.colors;
                    widget::text_input::Style {
                        background: (if focused {
                            c.primary_container
                        } else {
                            c.surface_container_highest
                        })
                        .into(),
                        border: Border {
                            radius: 8.0.into(),
                            width: if focused || invalid { 2.0 } else { 0.0 },
                            color: if invalid { c.error } else { c.primary },
                        },
                        value: if disabled {
                            crate::theme::alpha(c.on_surface, 0.38)
                        } else if focused {
                            c.on_primary_container
                        } else {
                            c.on_surface
                        },
                        placeholder: c.on_surface_variant,
                        icon: c.on_surface_variant,
                        selection: c.secondary_container,
                    }
                });
            widget::column![
                input,
                typography(label, TypeScale::BodySmall).style(move |theme: &Theme| {
                    widget::text::Style {
                        color: Some(if invalid {
                            theme.colors.error
                        } else {
                            theme.colors.on_surface_variant
                        }),
                    }
                })
            ]
            .spacing(4)
        };
    let mut header = widget::row![
        entry(
            "Hour",
            picker.hour_input,
            if picker.format24 {
                picker.value.hour
            } else {
                picker.value.hour12()
            }
        ),
        widget::container(typography(":", TypeScale::DisplayLarge)).center_y(80),
        entry("Minute", picker.minute_input, picker.value.minute),
    ]
    .spacing(8);
    if !picker.format24 {
        header = header.push(period_selector(
            pm,
            picker.input_period.as_ref().map(|(_, f)| f(false)),
            picker.input_period.as_ref().map(|(_, f)| f(true)),
        ));
    }
    let mut content = widget::column![
        typography(picker.title, TypeScale::LabelMedium).style(|theme: &Theme| {
            widget::text::Style {
                color: Some(theme.colors.on_surface_variant),
            }
        }),
        header
    ]
    .spacing(12);
    if valid.is_err() {
        content = content.push(
            typography(
                if picker.format24 {
                    "Enter hours 00–23 and minutes 00–59"
                } else {
                    "Enter hours 01–12 and minutes 00–59"
                },
                TypeScale::BodySmall,
            )
            .style(|theme: &Theme| widget::text::Style {
                color: Some(theme.colors.error),
            }),
        );
    }
    let mut actions = widget::row![];
    if let Some(message) = picker.on_toggle_input {
        actions = actions.push(mode_toggle(true, message));
    }
    actions = actions.push(widget::space().width(Length::Fill)).spacing(8);
    if let Some(message) = picker.on_cancel {
        actions = actions.push(
            crate::button("Cancel")
                .variant(ButtonVariant::Text)
                .on_press(message),
        );
    }
    content = content.push(
        actions
            .push(
                crate::button(if picker.on_confirm.is_some() {
                    "OK"
                } else {
                    "Apply"
                })
                .variant(ButtonVariant::Text)
                .on_press_maybe(submit),
            )
            .align_y(iced::Alignment::Center),
    );
    widget::container(content)
        .padding(iced::Padding {
            top: 16.0,
            right: 24.0,
            bottom: 24.0,
            left: 24.0,
        })
        .width(360)
        .style(|theme: &Theme| widget::container::Style {
            background: Some(theme.colors.surface_container_high.into()),
            text_color: Some(theme.colors.on_surface),
            border: Border {
                radius: 28.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}
fn typography_time(value: u8) -> String {
    format!("{value:02}")
}
/// Numeric 24-hour input. Incomplete text remains application-owned.
pub fn time_input<'a, Message: Clone + 'a>(label: &str, value: &str) -> TextField<'a, Message> {
    let field = text_field(label, value).supporting_text("HH:MM · 24-hour time");
    if !value.is_empty() && value.parse::<Time>().is_err() {
        field.error(InvalidTime.to_string())
    } else {
        field
    }
}
struct Clock<'a, Message> {
    value: Time,
    part: TimePart,
    format24: bool,
    on_change: Option<Box<dyn Fn(Time) -> Message + 'a>>,
    on_part: Option<Box<dyn Fn(TimePart) -> Message + 'a>>,
    labels: Vec<Element<'a, Message>>,
}
#[derive(Default)]
struct State {
    focus: focus::Focus,
    dragging: bool,
    last: Option<Time>,
    hand: std::cell::RefCell<Option<((Time, TimePart, bool), iced::advanced::svg::Handle)>>,
}
fn point(center: Point, radius: f32, fraction: f32) -> Point {
    let angle = fraction * std::f32::consts::TAU;
    Point::new(
        center.x + angle.sin() * radius,
        center.y - angle.cos() * radius,
    )
}
impl<Message> Clock<'_, Message> {
    fn picked(&self, p: Point, bounds: Rectangle) -> Time {
        let center = bounds.center();
        let dx = p.x - center.x;
        let dy = p.y - center.y;
        let fraction = dx.atan2(-dy).rem_euclid(std::f32::consts::TAU) / std::f32::consts::TAU;
        if self.part == TimePart::Minute {
            Time {
                minute: (fraction * 60.0).round() as u8 % 60,
                ..self.value
            }
        } else {
            let mut hour = (fraction * 12.0).round() as u8 % 12;
            if self.format24 {
                let inner = dx.hypot(dy) < bounds.width * 0.335;
                if inner {
                    if hour != 0 {
                        hour += 12;
                    }
                } else if hour == 0 {
                    hour = 12;
                }
            } else if self.value.is_pm() {
                hour += 12;
            }
            Time { hour, ..self.value }
        }
    }
}
impl<Message> Widget<Message, Theme, Renderer> for Clock<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }
    fn children(&self) -> Vec<Tree> {
        self.labels.iter().map(Tree::new).collect()
    }
    fn diff(&self, t: &mut Tree) {
        t.diff_children(&self.labels);
        if self.on_change.is_none() {
            *t.state.downcast_mut::<State>() = State::default();
        }
    }
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(256.0), Length::Fixed(256.0))
    }
    fn layout(&mut self, t: &mut Tree, r: &Renderer, l: &layout::Limits) -> layout::Node {
        let side = l.max().width.min(l.max().height).min(256.0);
        let center = Point::new(side / 2.0, side / 2.0);
        let children = self
            .labels
            .iter_mut()
            .enumerate()
            .map(|(i, label)| {
                let child = label.as_widget_mut().layout(
                    &mut t.children[i],
                    r,
                    &layout::Limits::new(Size::ZERO, Size::new(40.0, 28.0)),
                );
                let p = point(
                    center,
                    side * if i >= 12 { 0.265625 } else { 0.40625 },
                    (i % 12) as f32 / 12.0,
                );
                let size = child.size();
                child.move_to(Point::new(p.x - size.width / 2.0, p.y - size.height / 2.0))
            })
            .collect();
        layout::Node::with_children(Size::new(side, side), children)
    }
    fn operate(&mut self, t: &mut Tree, l: Layout<'_>, renderer: &Renderer, o: &mut dyn Operation) {
        if self.on_change.is_some() {
            let state = t.state.downcast_mut::<State>();
            let id = state.focus.id.clone();
            o.focusable(Some(&id), l.bounds(), &mut state.focus);
        }
        o.container(None, l.bounds());
        o.traverse(&mut |operation| {
            for (index, label) in self.labels.iter_mut().enumerate() {
                label.as_widget_mut().operate(
                    &mut t.children[index],
                    l.child(index),
                    renderer,
                    operation,
                );
            }
        });
    }
    fn update(
        &mut self,
        t: &mut Tree,
        event: &Event,
        l: Layout<'_>,
        cursor: mouse::Cursor,
        _: &Renderer,
        _: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let Some(change) = &self.on_change else {
            return;
        };
        let state = t.state.downcast_mut::<State>();
        let mut update = false;
        match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key),
                ..
            }) if state.focus.focused => {
                use keyboard::key::Named;
                let next = match key {
                    Named::ArrowLeft | Named::ArrowDown => Some(self.value.adjust(self.part, -1)),
                    Named::ArrowRight | Named::ArrowUp => Some(self.value.adjust(self.part, 1)),
                    Named::Home => Some(match self.part {
                        TimePart::Hour => Time {
                            hour: 0,
                            ..self.value
                        },
                        TimePart::Minute => Time {
                            minute: 0,
                            ..self.value
                        },
                    }),
                    Named::End => Some(match self.part {
                        TimePart::Hour => Time {
                            hour: 23,
                            ..self.value
                        },
                        TimePart::Minute => Time {
                            minute: 59,
                            ..self.value
                        },
                    }),
                    _ => None,
                };
                if let Some(next) = next {
                    shell.publish(change(next));
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if cursor.is_over(l.bounds()) && cursor.is_over(*viewport) =>
            {
                state.dragging = true;
                state.focus.focused = true;
                state.focus.visible = false;
                state.last = None;
                update = true;
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) if state.dragging => update = true,
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if state.dragging => {
                state.dragging = false;
                update = true;
            }
            Event::Window(window::Event::Unfocused) | Event::Mouse(mouse::Event::CursorLeft) => {
                state.dragging = false;
                state.last = None;
            }
            _ => {}
        }
        if update {
            if let Some(p) = cursor.position() {
                let value = self.picked(p, l.bounds());
                if state.last != Some(value) {
                    shell.publish(change(value));
                    state.last = Some(value);
                }
            }
            if !state.dragging
                && self.part == TimePart::Hour
                && let Some(part) = &self.on_part
            {
                shell.publish(part(TimePart::Minute));
            }
            shell.capture_event();
            shell.request_redraw();
        }
    }
    fn draw(
        &self,
        t: &Tree,
        r: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        l: Layout<'_>,
        c: mouse::Cursor,
        v: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;
        use iced::advanced::svg::{Handle, Renderer as _, Svg};
        let bounds = l.bounds();
        let Some(clip) = bounds.intersection(v) else {
            return;
        };
        let center = bounds.center();
        let inner = self.format24
            && self.part == TimePart::Hour
            && (self.value.hour == 0 || self.value.hour > 12);
        let radius = bounds.width * if inner { 0.265625 } else { 0.40625 };
        let fraction = if self.part == TimePart::Hour {
            (self.value.hour % 12) as f32 / 12.0
        } else {
            self.value.minute as f32 / 60.0
        };
        let selected = point(center, radius, fraction);
        let state = t.state.downcast_ref::<State>();
        r.with_layer(clip,|r|{
            r.fill_quad(renderer::Quad{bounds,border:Border{radius:(bounds.width/2.0).into(),color:theme.colors.primary,width:if state.focus.focused&&state.focus.visible{2.0}else{0.0}},..Default::default()},theme.colors.surface_container_highest);
            let key=(self.value,self.part,self.format24);let mut hand=state.hand.borrow_mut();
            if hand.as_ref().is_none_or(|(old,_)|*old!=key){let end=point(Point::new(128.0,128.0),if inner{68.0}else{104.0},fraction);*hand=Some((key,Handle::from_memory(format!("<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 256 256'><path d='M128 128 L{} {}' stroke='black' stroke-width='2'/></svg>",end.x,end.y).into_bytes())));}
            r.draw_svg(Svg::from(&hand.as_ref().unwrap().1).color(theme.colors.primary),bounds,clip);
            for (p,radius) in [(center,4.0),(selected,24.0)]{r.fill_quad(renderer::Quad{bounds:Rectangle::new(Point::new(p.x-radius,p.y-radius),Size::new(radius*2.0,radius*2.0)),border:Border{radius:radius.into(),..Default::default()},..Default::default()},theme.colors.primary);}
            for (i,label) in self.labels.iter().enumerate(){label.as_widget().draw(&t.children[i],r,theme,style,l.child(i),c,&clip);}
            if self.part==TimePart::Minute&&!self.value.minute.is_multiple_of(5){r.fill_quad(renderer::Quad{bounds:Rectangle::new(Point::new(selected.x-2.0,selected.y-2.0),Size::new(4.0,4.0)),border:Border{radius:2.0.into(),..Default::default()},..Default::default()},theme.colors.on_primary);}
        });
    }
    fn mouse_interaction(
        &self,
        _: &Tree,
        l: Layout<'_>,
        c: mouse::Cursor,
        v: &Rectangle,
        _: &Renderer,
    ) -> mouse::Interaction {
        if self.on_change.is_some() && c.is_over(l.bounds()) && c.is_over(*v) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strict_time_and_period_conversion() {
        assert_eq!("00:00".parse::<Time>().unwrap().hour12(), 12);
        assert_eq!(
            Time::new(12, 30).unwrap().with_period(false),
            Time::new(0, 30).unwrap()
        );
        for invalid in ["24:00", "12:60", "1:05", "01:0é", ""] {
            assert!(invalid.parse::<Time>().is_err());
        }
    }
}
