//! An application-owned component with no dependency on the Material theme.
use iced::{Element, Length, widget};

pub fn summary<'a, Message: 'a, Theme>(refreshes: u32) -> Element<'a, Message, Theme>
where
    Theme: widget::text::Catalog + widget::container::Catalog + 'a,
{
    widget::row![
        metric("6", "Active projects"),
        metric("12", "Team members"),
        metric(&refreshes.to_string(), "Activity refreshes"),
    ]
    .spacing(24)
    .into()
}

fn metric<'a, Message: 'a, Theme>(value: &str, label: &str) -> Element<'a, Message, Theme>
where
    Theme: widget::text::Catalog + widget::container::Catalog + 'a,
{
    widget::container(
        widget::column![
            widget::text(value.to_owned()).size(28),
            widget::text(label.to_owned()).size(14),
        ]
        .spacing(4),
    )
    .width(Length::Fill)
    .into()
}
