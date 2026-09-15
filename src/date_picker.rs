//! Controlled Gregorian calendar and ISO date input. No timezone conversion occurs.
use crate::{
    ButtonVariant, Element, TextField, Theme, TypeScale, button, focus, text_field, typography,
};
use iced::{Border, Length, widget};
use std::{fmt, str::FromStr};
mod keyboard;
/// A valid proleptic Gregorian date in years 1–9999.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    year: u16,
    month: u8,
    day: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidDate;
impl fmt::Display for InvalidDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Enter a valid date as YYYY-MM-DD")
    }
}
impl std::error::Error for InvalidDate {}
impl Date {
    pub const MIN: Self = Self {
        year: 1,
        month: 1,
        day: 1,
    };
    pub const MAX: Self = Self {
        year: 9999,
        month: 12,
        day: 31,
    };
    pub fn new(year: u16, month: u8, day: u8) -> Result<Self, InvalidDate> {
        if (1..=9999).contains(&year)
            && (1..=12).contains(&month)
            && day > 0
            && day <= days_in_month(year, month)
        {
            Ok(Self { year, month, day })
        } else {
            Err(InvalidDate)
        }
    }
    pub fn year(self) -> u16 {
        self.year
    }
    pub fn month(self) -> u8 {
        self.month
    }
    pub fn day(self) -> u8 {
        self.day
    }
    pub fn first_of_month(self) -> Self {
        Self { day: 1, ..self }
    }
    pub fn days_in_month(self) -> u8 {
        days_in_month(self.year, self.month)
    }
    /// Sunday = 0, Saturday = 6.
    pub fn weekday(self) -> u8 {
        let offsets = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        let y = self.year as i32 - i32::from(self.month < 3);
        ((y + y / 4 - y / 100 + y / 400 + offsets[self.month as usize - 1] + self.day as i32) % 7)
            as u8
    }
    /// Move by calendar months, clamping the day at the destination month end.
    pub fn add_months(self, months: i32) -> Option<Self> {
        let n = (self.year as i64 - 1) * 12 + self.month as i64 - 1 + months as i64;
        if !(0..9999 * 12).contains(&n) {
            return None;
        }
        let (year, month) = ((n / 12 + 1) as u16, (n % 12 + 1) as u8);
        Some(Self {
            year,
            month,
            day: self.day.min(days_in_month(year, month)),
        })
    }
    pub fn add_days(self, days: i32) -> Option<Self> {
        let mut day = self.day as i64 + days as i64;
        let mut month = self.first_of_month();
        // Bounded by 120,000 months, even for hostile offsets.
        if !(-3_652_059..=3_652_059).contains(&days) {
            return None;
        }
        while day < 1 {
            month = month.add_months(-1)?;
            day += month.days_in_month() as i64;
        }
        while day > month.days_in_month() as i64 {
            day -= month.days_in_month() as i64;
            month = month.add_months(1)?;
        }
        Some(Self {
            day: day as u8,
            ..month
        })
    }
}
fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        4 | 6 | 9 | 11 => 30,
        2 => {
            if year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400)) {
                29
            } else {
                28
            }
        }
        _ => 31,
    }
}
impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}
impl FromStr for Date {
    type Err = InvalidDate;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let b = value.as_bytes();
        if b.len() != 10
            || b[4] != b'-'
            || b[7] != b'-'
            || b.iter()
                .enumerate()
                .any(|(i, b)| i != 4 && i != 7 && !b.is_ascii_digit())
        {
            return Err(InvalidDate);
        }
        Self::new(
            value[..4].parse().map_err(|_| InvalidDate)?,
            value[5..7].parse().map_err(|_| InvalidDate)?,
            value[8..].parse().map_err(|_| InvalidDate)?,
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateSelection {
    Single(Option<Date>),
    Range {
        start: Option<Date>,
        end: Option<Date>,
    },
}
impl Default for DateSelection {
    fn default() -> Self {
        Self::Single(None)
    }
}
impl DateSelection {
    pub fn select(self, date: Date) -> Self {
        match self {
            Self::Single(_) => Self::Single(Some(date)),
            Self::Range {
                start: Some(start),
                end: None,
            } => Self::Range {
                start: Some(start.min(date)),
                end: Some(start.max(date)),
            },
            Self::Range { .. } => Self::Range {
                start: Some(date),
                end: None,
            },
        }
    }
    pub fn complete(self) -> bool {
        matches!(
            self,
            Self::Single(Some(_))
                | Self::Range {
                    start: Some(_),
                    end: Some(_)
                }
        )
    }
    fn endpoints(self) -> (Option<Date>, Option<Date>) {
        match self {
            Self::Single(date) => (date, None),
            Self::Range { start, end } => (start, end),
        }
    }
}
pub struct DatePicker<'a, Message> {
    month: Date,
    selection: DateSelection,
    today: Option<Date>,
    min: Date,
    max: Date,
    enabled: Option<Box<dyn Fn(Date) -> bool + 'a>>,
    on_select: Option<Box<dyn Fn(DateSelection) -> Message + 'a>>,
    on_confirm: Option<Box<dyn Fn(DateSelection) -> Message + 'a>>,
    on_cancel: Option<Message>,
    on_month: Option<Box<dyn Fn(Date) -> Message + 'a>>,
    first_weekday: u8,
    title: String,
    years: bool,
    on_toggle_years: Option<Message>,
    compact: bool,
    input_mode: bool,
    on_toggle_input: Option<Message>,
    input: String,
    end_input: String,
    on_input: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_end_input: Option<Box<dyn Fn(String) -> Message + 'a>>,
}
pub fn date_picker<'a, Message>(month: Date, selection: DateSelection) -> DatePicker<'a, Message> {
    DatePicker {
        month: month.first_of_month(),
        selection,
        today: None,
        min: Date::MIN,
        max: Date::MAX,
        enabled: None,
        on_select: None,
        on_confirm: None,
        on_cancel: None,
        on_month: None,
        first_weekday: 0,
        title: "Select date".into(),
        years: false,
        on_toggle_years: None,
        compact: false,
        input_mode: false,
        on_toggle_input: None,
        input: String::new(),
        end_input: String::new(),
        on_input: None,
        on_end_input: None,
    }
}
impl<'a, Message> DatePicker<'a, Message> {
    /// Add an OK action for a complete, valid selection. Calendar edits still
    /// use `on_select`; keep a separate application draft for transactional use.
    /// Manual OK/Enter sends this callback instead of `on_select`.
    pub fn on_confirm(mut self, f: impl Fn(DateSelection) -> Message + 'a) -> Self {
        self.on_confirm = Some(Box::new(f));
        self
    }
    /// Add a Cancel action. The application owns discarding its draft.
    pub fn on_cancel(mut self, message: Message) -> Self {
        self.on_cancel = Some(message);
        self
    }

    pub fn on_select(mut self, f: impl Fn(DateSelection) -> Message + 'a) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }
    pub fn on_month(mut self, f: impl Fn(Date) -> Message + 'a) -> Self {
        self.on_month = Some(Box::new(f));
        self
    }
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
    pub fn today(mut self, today: Date) -> Self {
        self.today = Some(today);
        self
    }
    pub fn bounds(mut self, min: Date, max: Date) -> Self {
        self.min = min;
        self.max = max;
        self
    }
    /// Disable individual endpoints. The app can additionally validate dates inside a range.
    pub fn date_enabled(mut self, f: impl Fn(Date) -> bool + 'a) -> Self {
        self.enabled = Some(Box::new(f));
        self
    }
    pub fn first_weekday(mut self, weekday: u8) -> Self {
        self.first_weekday = weekday % 7;
        self
    }
    /// The year grid follows the displayed month, with previous/next twelve-year pages.
    pub fn years(mut self, shown: bool) -> Self {
        self.years = shown;
        self
    }
    pub fn on_toggle_years(mut self, message: Message) -> Self {
        self.on_toggle_years = Some(message);
        self
    }
    /// A compact desktop calendar without the modal title and selected-date headline.
    /// This profile is selected automatically by [`docked_date_picker`].
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }
    /// Switch between calendar and ISO text entry. Keep raw text in application
    /// state so incomplete edits survive switching modes. Selection is committed
    /// by the app; typing never silently replaces it.
    pub fn input_mode(mut self, input_mode: bool) -> Self {
        self.input_mode = input_mode;
        self
    }
    pub fn on_toggle_input(mut self, message: Message) -> Self {
        self.on_toggle_input = Some(message);
        self
    }
    pub fn input(mut self, value: &str, on_input: impl Fn(String) -> Message + 'a) -> Self {
        self.input = value.into();
        self.on_input = Some(Box::new(on_input));
        self
    }
    pub fn end_input(mut self, value: &str, on_input: impl Fn(String) -> Message + 'a) -> Self {
        self.end_input = value.into();
        self.on_end_input = Some(Box::new(on_input));
        self
    }
    fn parse_input(&self, value: &str) -> Result<Date, String> {
        let date = value.parse::<Date>().map_err(|e| e.to_string())?;
        if date < self.min || date > self.max {
            return Err(format!("Choose a date from {} to {}", self.min, self.max));
        }
        if self.enabled.as_ref().is_some_and(|enabled| !enabled(date)) {
            return Err("This date is unavailable".into());
        }
        Ok(date)
    }
    /// Validate complete typed input with the same endpoint restrictions as the
    /// calendar. Use this when enabling a Save/OK action. Range interiors remain
    /// application-owned, as with calendar selection.
    pub fn input_selection(&self) -> Result<DateSelection, String> {
        let start = self.parse_input(&self.input)?;
        if matches!(self.selection, DateSelection::Range { .. }) {
            let end = self.parse_input(&self.end_input)?;
            if end < start {
                return Err("End date must be on or after start date".into());
            }
            Ok(DateSelection::Range {
                start: Some(start),
                end: Some(end),
            })
        } else {
            Ok(DateSelection::Single(Some(start)))
        }
    }
}
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
impl<'a, Message: Clone + 'a> From<DatePicker<'a, Message>> for Element<'a, Message> {
    fn from(mut picker: DatePicker<'a, Message>) -> Self {
        let confirmed = if picker.input_mode {
            picker.input_selection().ok()
        } else {
            let (start, end) = picker.selection.endpoints();
            (picker.selection.complete()
                && start
                    .into_iter()
                    .chain(end)
                    .all(|date| picker.parse_input(&date.to_string()).is_ok())
                && start.zip(end).is_none_or(|(a, b)| a <= b))
            .then_some(picker.selection)
        };
        let has_confirm = picker.on_confirm.is_some();
        let submit = confirmed.and_then(|value| {
            picker
                .on_confirm
                .as_ref()
                .or(if picker.input_mode {
                    picker.on_select.as_ref()
                } else {
                    None
                })
                .map(|f| f(value))
        });
        let input_content: Option<Element<'a, Message>> = if picker.input_mode {
            let range = matches!(picker.selection, DateSelection::Range { .. });
            let start_error = (!picker.input.is_empty())
                .then(|| picker.parse_input(&picker.input).err())
                .flatten();
            let end_error = if picker.end_input.is_empty() {
                None
            } else {
                picker.parse_input(&picker.end_input).err().or_else(|| {
                    picker
                        .parse_input(&picker.input)
                        .ok()
                        .zip(picker.parse_input(&picker.end_input).ok())
                        .filter(|(start, end)| end < start)
                        .map(|_| "End date must be on or after start date".to_owned())
                })
            };
            let mut start = date_input(if range { "Start date" } else { "Date" }, &picker.input);
            if let Some(error) = start_error {
                start = start.error(error);
            }
            if let Some(on_input) = picker.on_input.take() {
                start = start.on_input(on_input);
            }
            if let Some(message) = &submit {
                start = start.on_submit(message.clone());
            }
            let mut fields = widget::column![start].spacing(16);
            if range {
                let mut end = date_input("End date", &picker.end_input);
                if let Some(error) = end_error {
                    end = end.error(error);
                }
                if let Some(on_input) = picker.on_end_input.take() {
                    end = end.on_input(on_input);
                }
                if let Some(message) = &submit {
                    end = end.on_submit(message.clone());
                }
                fields = fields.push(end);
            }
            Some(fields.into())
        } else {
            None
        };
        let row_height = if picker.compact { 40 } else { 44 };
        let (start, end) = picker.selection.endpoints();
        let headline = if picker.input_mode {
            if matches!(picker.selection, DateSelection::Range { .. }) {
                "Enter dates".into()
            } else {
                "Enter date".into()
            }
        } else {
            match picker.selection {
                DateSelection::Single(Some(date)) => format!(
                    "{} {}, {}",
                    if picker.on_toggle_input.is_some() {
                        &MONTHS[date.month as usize - 1][..3]
                    } else {
                        MONTHS[date.month as usize - 1]
                    },
                    date.day,
                    date.year
                ),
                DateSelection::Single(None) => "Select date".into(),
                DateSelection::Range {
                    start: Some(start),
                    end: Some(end),
                } => format!("{} – {}", start, end),
                DateSelection::Range {
                    start: Some(start),
                    end: None,
                } => format!("{} – End date", start),
                _ => "Start date – End date".into(),
            }
        };
        let step = if picker.years { 144 } else { 1 };
        let year_base = ((picker.month.year - 1) / 12) * 12 + 1;
        let month_message = |months| {
            if picker.min > picker.max {
                return None;
            }
            let date = if picker.years {
                let page = year_base as i32 + if months < 0 { -12 } else { 12 };
                if page > picker.max.year as i32
                    || page + 11 < picker.min.year as i32
                    || !(1..=9999).contains(&page)
                {
                    None
                } else {
                    Date::new((page as u16).max(picker.min.year), picker.month.month, 1)
                        .ok()
                        .map(|d| d.clamp(picker.min.first_of_month(), picker.max.first_of_month()))
                }
            } else {
                picker.month.add_months(months).filter(|d| {
                    d.first_of_month() <= picker.max.first_of_month()
                        && d.first_of_month() >= picker.min.first_of_month()
                })
            };
            date.and_then(|d| picker.on_month.as_ref().map(|f| f(d)))
        };
        let label = if picker.years {
            format!("{}–{}", year_base, (year_base + 11).min(9999))
        } else {
            format!(
                "{} {}",
                MONTHS[picker.month.month as usize - 1],
                picker.month.year
            )
        };
        let month_label: Element<'a, Message> = if let Some(message) = picker.on_toggle_years {
            button(label)
                .variant(ButtonVariant::Text)
                .padding([8, 0])
                .on_press(message)
                .into()
        } else {
            widget::container(typography(label, TypeScale::TitleSmall))
                .center_y(40)
                .into()
        };
        let heading = widget::row![
            month_label,
            widget::space().width(Length::Fill),
            crate::Button::new(Element::new(crate::glyph::Glyph::Previous))
                .variant(ButtonVariant::Text)
                .width(40)
                .padding(0)
                .on_press_maybe(month_message(-step)),
            crate::Button::new(Element::new(crate::glyph::Glyph::Next))
                .variant(ButtonVariant::Text)
                .width(40)
                .padding(0)
                .on_press_maybe(month_message(step))
        ]
        .align_y(iced::Alignment::Center);
        let mut grid = widget::Column::new();
        let mut active = 0;
        let mut enabled_count = 0;
        let mut dates = Vec::new();
        if picker.input_mode {
            // Native fields replace the calendar grid in this mode.
        } else if picker.years {
            for row in 0..4 {
                let mut line = widget::Row::new().spacing(4);
                for column in 0..3 {
                    let year = year_base + row * 3 + column;
                    let date = Date::new(year, picker.month.month, 1).ok();
                    let message = date
                        .filter(|d| {
                            picker.min <= picker.max
                                && d.year >= picker.min.year
                                && d.year <= picker.max.year
                        })
                        .map(|d| d.clamp(picker.min.first_of_month(), picker.max.first_of_month()))
                        .and_then(|d| picker.on_month.as_ref().map(|f| f(d)));
                    if message.is_some() {
                        if year == picker.month.year {
                            active = enabled_count;
                        }
                        enabled_count += 1;
                    }
                    line = line.push(
                        widget::container(
                            crate::Button::new(typography(
                                if year <= 9999 {
                                    year.to_string()
                                } else {
                                    String::new()
                                },
                                TypeScale::BodyLarge,
                            ))
                            .width(72)
                            .height(36)
                            .padding(0)
                            .variant(if year == picker.month.year {
                                ButtonVariant::Filled
                            } else {
                                ButtonVariant::Text
                            })
                            .palette(move |theme| {
                                if year == picker.month.year {
                                    (theme.colors.primary, theme.colors.on_primary)
                                } else {
                                    (iced::Color::TRANSPARENT, theme.colors.on_surface_variant)
                                }
                            })
                            .on_press_maybe(message),
                        )
                        .center_x(Length::Fill)
                        .center_y(48),
                    );
                }
                grid = grid.push(line);
            }
        } else {
            let mut weekdays = widget::Row::new();
            for i in 0..7 {
                weekdays = weekdays.push(
                    widget::container(typography(
                        ["S", "M", "T", "W", "T", "F", "S"]
                            [(i + picker.first_weekday as usize) % 7],
                        TypeScale::BodyLarge,
                    ))
                    .center_x(Length::Fill)
                    .center_y(row_height),
                );
            }
            grid = grid.push(weekdays);
            let offset = (picker.month.weekday() + 7 - picker.first_weekday) % 7;
            for row in 0..6 {
                let mut line = widget::Row::new();
                for col in 0..7 {
                    let n = row * 7 + col - offset as i32 + 1;
                    if n < 1 || n > picker.month.days_in_month() as i32 {
                        line = line.push(widget::space().width(Length::Fill).height(row_height));
                        continue;
                    }
                    let date = Date {
                        day: n as u8,
                        ..picker.month
                    };
                    let endpoint = start == Some(date) || end == Some(date);
                    let inside = start.zip(end).is_some_and(|(a, b)| date >= a && date <= b);
                    let enabled = date >= picker.min
                        && date <= picker.max
                        && picker.enabled.as_ref().is_none_or(|f| f(date));
                    let message = if enabled {
                        picker
                            .on_select
                            .as_ref()
                            .map(|f| f(picker.selection.select(date)))
                    } else {
                        None
                    };
                    if message.is_some() {
                        if start.or(picker.today) == Some(date) {
                            active = enabled_count;
                        }
                        enabled_count += 1;
                    }
                    let day =
                        crate::Button::new(typography(date.day.to_string(), TypeScale::BodyLarge))
                            .width(40)
                            .height(40)
                            .minimum_height(40.0)
                            .padding(0)
                            .variant(if endpoint {
                                ButtonVariant::Filled
                            } else if picker.today == Some(date) {
                                ButtonVariant::Outlined
                            } else {
                                ButtonVariant::Text
                            })
                            .outline_color(|theme, _| theme.colors.primary)
                            .palette(move |theme| {
                                (
                                    if endpoint {
                                        theme.colors.primary
                                    } else {
                                        iced::Color::TRANSPARENT
                                    },
                                    if endpoint {
                                        theme.colors.on_primary
                                    } else if inside {
                                        theme.colors.on_secondary_container
                                    } else if picker.today == Some(date) {
                                        theme.colors.primary
                                    } else {
                                        theme.colors.on_surface
                                    },
                                )
                            })
                            .on_press_maybe(message);
                    if enabled && picker.on_select.is_some() {
                        dates.push(date);
                    }
                    line = line.push(
                        widget::container(
                            widget::container(day)
                                .center_x(Length::Fill)
                                .center_y(40)
                                .style(move |theme: &Theme| widget::container::Style {
                                    background: inside
                                        .then_some(theme.colors.secondary_container.into()),
                                    border: Border {
                                        radius: iced::border::Radius {
                                            top_left: if start == Some(date) { 20.0 } else { 0.0 },
                                            bottom_left: if start == Some(date) {
                                                20.0
                                            } else {
                                                0.0
                                            },
                                            top_right: if end == Some(date) { 20.0 } else { 0.0 },
                                            bottom_right: if end == Some(date) {
                                                20.0
                                            } else {
                                                0.0
                                            },
                                        },
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                        )
                        .center_y(row_height),
                    );
                }
                grid = grid.push(line);
            }
        }
        let grid = focus::grid(grid, active);
        let grid = if picker.years {
            grid
        } else {
            Element::new(keyboard::Calendar {
                content: grid,
                dates,
                month: picker.month,
                min: picker.min,
                max: picker.max,
                enabled: picker.enabled,
                on_month: picker.on_month,
            })
        };
        let toggle: Option<Element<'a, Message>> = picker.on_toggle_input.map(|message| {
            crate::tooltip(
                crate::icon_button(Element::new(if picker.input_mode {
                    crate::glyph::Glyph::Calendar
                } else {
                    crate::glyph::Glyph::Edit
                }))
                .on_press(message),
                if picker.input_mode {
                    "Choose from calendar"
                } else {
                    "Type a date"
                },
            )
            .into()
        });
        let mut content = widget::Column::new().spacing(12);
        let header = if !picker.compact {
            let single = matches!(picker.selection, DateSelection::Single(_));
            let title = typography(picker.title, TypeScale::LabelLarge).style(|theme: &Theme| {
                widget::text::Style {
                    color: Some(theme.colors.on_surface_variant),
                }
            });
            let headline = typography(
                headline,
                if single {
                    TypeScale::HeadlineLarge
                } else {
                    TypeScale::TitleLarge
                },
            )
            .width(Length::Fill)
            .style(|theme: &Theme| widget::text::Style {
                color: Some(theme.colors.on_surface_variant),
            });
            let row = widget::row![headline].align_y(iced::Alignment::Center);
            let row = if let Some(toggle) = toggle {
                row.push(toggle)
            } else {
                row
            };
            Some(
                widget::container(widget::column![
                    title,
                    widget::space().height(if single { 20 } else { 28 }),
                    widget::container(row.push(widget::space().width(1).height(48)))
                        .align_y(iced::Alignment::Center)
                ])
                .padding([16, 24]),
            )
        } else {
            if let Some(toggle) = toggle {
                content = content.push(
                    widget::row![
                        typography(
                            if picker.input_mode {
                                "Enter date"
                            } else {
                                "Choose date"
                            },
                            TypeScale::TitleSmall
                        )
                        .width(Length::Fill),
                        toggle
                    ]
                    .align_y(iced::Alignment::Center),
                );
            }
            None
        };
        if let Some(input) = input_content {
            content = content.push(input);
        } else {
            content = content.push(heading).push(grid);
        }
        if has_confirm || picker.on_cancel.is_some() || picker.input_mode {
            let mut actions = widget::row![widget::space().width(Length::Fill)].spacing(8);
            if let Some(message) = picker.on_cancel {
                actions = actions.push(
                    button("Cancel")
                        .variant(ButtonVariant::Text)
                        .on_press(message),
                );
            }
            if has_confirm || picker.input_mode {
                actions = actions.push(
                    button(if has_confirm { "OK" } else { "Apply" })
                        .variant(ButtonVariant::Text)
                        .on_press_maybe(submit),
                );
            }
            content = content.push(actions);
        }
        let body = widget::container(content).padding(if picker.compact { 16 } else { 24 });
        let content: Element<'a, Message> = if let Some(header) = header {
            widget::column![header, crate::divider(), body].into()
        } else {
            body.into()
        };
        Element::from(
            widget::container(content)
                .width(if picker.compact { 328 } else { 360 })
                .style(move |theme: &Theme| widget::container::Style {
                    background: Some(theme.colors.surface_container_high.into()),
                    text_color: Some(theme.colors.on_surface),
                    border: Border {
                        radius: if picker.compact { 16.0 } else { 28.0 }.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        )
    }
}
/// ISO input retains incomplete text in application state. Parse with `Date::from_str`
/// and apply the same bounds/validator before accepting the dialog.
pub fn date_input<'a, Message: Clone + 'a>(label: &str, value: &str) -> TextField<'a, Message> {
    let field = text_field(label, value).supporting_text("YYYY-MM-DD");
    if !value.is_empty() && value.parse::<Date>().is_err() {
        field.error(InvalidDate.to_string())
    } else {
        field
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn leap_year_centuries_and_month_clamping() {
        assert!(Date::new(1900, 2, 29).is_err());
        assert!(Date::new(2000, 2, 29).is_ok());
        assert_eq!(
            Date::new(2024, 1, 31).unwrap().add_months(1),
            Some(Date::new(2024, 2, 29).unwrap())
        );
        assert!(Date::MIN.add_days(-1).is_none());
        assert!(Date::MAX.add_months(1).is_none());
    }
    #[test]
    fn strict_input_and_known_weekdays() {
        assert_eq!("2026-09-13".parse::<Date>().unwrap().weekday(), 0);
        for invalid in ["2026-9-13", "2026-02-30", "0000-01-01", "2026-0é-1", ""] {
            assert!(invalid.parse::<Date>().is_err());
        }
    }
    #[test]
    fn range_selection_orders_and_restarts() {
        let a = Date::new(2026, 9, 20).unwrap();
        let b = Date::new(2026, 9, 10).unwrap();
        let range = DateSelection::Range {
            start: None,
            end: None,
        }
        .select(a)
        .select(b);
        assert_eq!(
            range,
            DateSelection::Range {
                start: Some(b),
                end: Some(a)
            }
        );
        assert_eq!(
            range.select(a),
            DateSelection::Range {
                start: Some(a),
                end: None
            }
        );
    }
}

#[derive(Clone)]
enum PopupEvent<Message> {
    Keep(Message),
    Select(Message, bool),
}
/// An anchored calendar that stays open during month/year navigation and closes
/// after a complete selection. Keep it mounted to preserve focus across updates.
/// A date range closes after its second endpoint. Escape/outside cancel the popup;
/// selected values and validation remain owned by the application.
pub fn docked_date_picker<'a, Message: Clone + 'a>(
    trigger: impl Into<Element<'a, Message>>,
    picker: DatePicker<'a, Message>,
) -> Element<'a, Message> {
    let confirm_required = picker.on_confirm.is_some();
    let picker = DatePicker {
        month: picker.month,
        selection: picker.selection,
        today: picker.today,
        min: picker.min,
        max: picker.max,
        enabled: picker.enabled,
        on_select: picker.on_select.map(|f| {
            Box::new(move |s: DateSelection| {
                PopupEvent::Select(f(s), !confirm_required && s.complete())
            }) as Box<dyn Fn(DateSelection) -> PopupEvent<Message>>
        }),
        on_confirm: picker.on_confirm.map(|f| {
            Box::new(move |s| PopupEvent::Select(f(s), true))
                as Box<dyn Fn(DateSelection) -> PopupEvent<Message>>
        }),
        on_cancel: picker
            .on_cancel
            .map(|message| PopupEvent::Select(message, true)),
        on_month: picker.on_month.map(|f| {
            Box::new(move |d| PopupEvent::Keep(f(d))) as Box<dyn Fn(Date) -> PopupEvent<Message>>
        }),
        first_weekday: picker.first_weekday,
        title: picker.title,
        years: picker.years,
        on_toggle_years: picker.on_toggle_years.map(PopupEvent::Keep),
        compact: true,
        input_mode: picker.input_mode,
        on_toggle_input: picker.on_toggle_input.map(PopupEvent::Keep),
        input: picker.input,
        end_input: picker.end_input,
        on_input: picker.on_input.map(|f| {
            Box::new(move |text| PopupEvent::Keep(f(text)))
                as Box<dyn Fn(String) -> PopupEvent<Message>>
        }),
        on_end_input: picker.on_end_input.map(|f| {
            Box::new(move |text| PopupEvent::Keep(f(text)))
                as Box<dyn Fn(String) -> PopupEvent<Message>>
        }),
    };
    let trigger = trigger.into().map(PopupEvent::Keep);
    let body: Element<'a, PopupEvent<Message>> = picker.into();
    let content = crate::elevation::elevated(
        widget::container(crate::staged::part(
            widget::scrollable(body).width(Length::Fill),
            crate::staged::Part::Popup(true),
        ))
        .style(|theme: &Theme| widget::container::Style {
            background: Some(theme.colors.surface_container_high.into()),
            border: Border {
                radius: 16.0.into(),
                ..Default::default()
            },

            ..Default::default()
        }),
        3.0,
        16.0,
    );
    let popup: Element<'a, PopupEvent<Message>> = crate::menu::rich_popup(trigger, content)
        .surface(16.0, 3, true)
        .width(328.0)
        .max_height(600.0)
        .close_when(|event| matches!(event, PopupEvent::Select(_, true)))
        .into();
    popup.map(|event| match event {
        PopupEvent::Keep(message) | PopupEvent::Select(message, _) => message,
    })
}
