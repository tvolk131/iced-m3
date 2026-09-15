//! Controlled desktop navigation rail. Page content and routing belong to the app.
use crate::{Button, ButtonVariant, Element, Theme, TypeScale, badge, tokens, typography};
use iced::{Border, Color, Length, widget};
pub struct NavigationItem<'a, Value, Message> {
    value: Value,
    label: String,
    icon: Element<'a, Message>,
    badge: Option<u32>,
    disabled: bool,
}
impl<'a, Value, Message> NavigationItem<'a, Value, Message> {
    pub fn new(
        value: Value,
        label: impl Into<String>,
        icon: impl Into<Element<'a, Message>>,
    ) -> Self {
        Self {
            value,
            label: label.into(),
            icon: icon.into(),
            badge: None,
            disabled: false,
        }
    }
    pub fn badge(mut self, count: u32) -> Self {
        self.badge = Some(count);
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}
pub struct NavigationRail<'a, Value, Message> {
    items: Vec<NavigationItem<'a, Value, Message>>,
    selected: Option<Value>,
    handler: Option<Box<dyn Fn(Value) -> Message + 'a>>,
    header: Option<Element<'a, Message>>,
    footer: Option<Element<'a, Message>>,
    disabled: bool,
    expanded: bool,
    modal_surface: bool,
}
pub fn navigation_rail<'a, Value, Message>(
    items: impl IntoIterator<Item = NavigationItem<'a, Value, Message>>,
    selected: Option<Value>,
) -> NavigationRail<'a, Value, Message> {
    NavigationRail {
        items: items.into_iter().collect(),
        selected,
        handler: None,
        header: None,
        footer: None,
        disabled: false,
        expanded: false,
        modal_surface: false,
    }
}
impl<'a, Value, Message> NavigationRail<'a, Value, Message> {
    /// Expanded 280px rail with horizontal destinations.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }
    pub fn on_select(mut self, handler: impl Fn(Value) -> Message + 'a) -> Self {
        self.handler = Some(Box::new(handler));
        self
    }
    pub fn header(mut self, content: impl Into<Element<'a, Message>>) -> Self {
        self.header = Some(content.into());
        self
    }
    pub fn footer(mut self, content: impl Into<Element<'a, Message>>) -> Self {
        self.footer = Some(content.into());
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}
/// Navigation treatment for the available logical width.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationLayout {
    Bar,
    Rail,
    ExpandedRail,
}
impl NavigationLayout {
    pub fn for_width(width: f32) -> Self {
        if width < 600.0 {
            Self::Bar
        } else if width < 1200.0 {
            Self::Rail
        } else {
            Self::ExpandedRail
        }
    }
    pub fn rail_width(self) -> f32 {
        match self {
            Self::Bar => 0.0,
            Self::Rail => tokens::size::NAV_RAIL,
            Self::ExpandedRail => 280.0,
        }
    }
}
/// Rebuild a navigation composition when its available size changes.
pub fn adaptive_navigation<'a, Message: 'a>(
    view: impl Fn(NavigationLayout) -> Element<'a, Message> + 'a,
) -> Element<'a, Message> {
    widget::responsive(move |size| view(NavigationLayout::for_width(size.width))).into()
}
/// Bottom navigation for three to five top-level destinations.
pub struct NavigationBar<'a, Value, Message> {
    items: Vec<NavigationItem<'a, Value, Message>>,
    selected: Option<Value>,
    handler: Option<Box<dyn Fn(Value) -> Message + 'a>>,
    disabled: bool,
}
pub fn navigation_bar<'a, Value, Message>(
    items: impl IntoIterator<Item = NavigationItem<'a, Value, Message>>,
    selected: Option<Value>,
) -> NavigationBar<'a, Value, Message> {
    NavigationBar {
        items: items.into_iter().collect(),
        selected,
        handler: None,
        disabled: false,
    }
}
impl<'a, Value, Message> NavigationBar<'a, Value, Message> {
    pub fn on_select(mut self, handler: impl Fn(Value) -> Message + 'a) -> Self {
        self.handler = Some(Box::new(handler));
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}
fn destination<'a, Message: Clone + 'a>(
    item: NavigationItem<'a, (), Message>,
    selected: bool,
    message: Option<Message>,
    layout: NavigationLayout,
) -> Element<'a, Message> {
    let expanded = layout == NavigationLayout::ExpandedRail;
    let enabled = message.is_some();
    let icon = widget::container(widget::container(item.icon).center_x(24).center_y(24))
        .center_x(if expanded {
            24.0
        } else if layout == NavigationLayout::Bar {
            64.0
        } else {
            tokens::size::NAV_INDICATOR_WIDTH
        })
        .center_y(if expanded {
            24.0
        } else {
            tokens::size::NAV_INDICATOR_HEIGHT
        })
        .style(move |theme: &Theme| widget::container::Style {
            background: Some(
                if selected && !expanded {
                    theme.colors.secondary_container
                } else {
                    Color::TRANSPARENT
                }
                .into(),
            ),
            text_color: Some(if !enabled {
                crate::theme::alpha(theme.colors.on_surface, 0.38)
            } else if selected {
                theme.colors.on_secondary_container
            } else {
                theme.colors.on_surface_variant
            }),
            border: Border {
                radius: tokens::shape::FULL.into(),
                ..Default::default()
            },
            ..Default::default()
        });
    let icon: Element<'a, Message> = if let Some(count) = item.badge {
        widget::stack![
            icon,
            widget::container(badge(count))
                .align_right(Length::Fill)
                .align_top(Length::Fill)
        ]
        .into()
    } else {
        icon.into()
    };
    let body: Element<'a, Message> = if expanded {
        widget::row![
            icon,
            typography(item.label, TypeScale::LabelLarge).width(Length::Fill)
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center)
        .into()
    } else {
        widget::column![
            icon,
            typography(item.label, TypeScale::LabelMedium)
                .align_x(iced::alignment::Horizontal::Center)
                .width(Length::Fill)
        ]
        .spacing(4)
        .align_x(iced::Alignment::Center)
        .into()
    };
    let button = Button::new(body)
        .width(Length::Fill)
        .variant(ButtonVariant::Text)
        .padding(if expanded {
            iced::Padding::from([16, 16])
        } else {
            iced::Padding::from([8, 4])
        })
        .radius(tokens::shape::FULL)
        .palette(move |theme| {
            (
                Color::TRANSPARENT,
                if selected {
                    theme.colors.on_surface
                } else {
                    theme.colors.on_surface_variant
                },
            )
        })
        .on_press_maybe(message);
    if expanded {
        button.segment_style(selected, 28.0.into()).into()
    } else {
        button
            .feedback_bounds(|l| l.child(0).child(0).bounds())
            .into()
    }
}
impl<'a, Value: PartialEq + 'a, Message: Clone + 'a> From<NavigationBar<'a, Value, Message>>
    for Element<'a, Message>
{
    fn from(bar: NavigationBar<'a, Value, Message>) -> Self {
        let active = bar
            .items
            .iter()
            .filter(|i| !i.disabled)
            .position(|i| Some(&i.value) == bar.selected.as_ref())
            .unwrap_or(0);
        let mut row = widget::Row::new()
            .width(Length::Fill)
            .align_y(iced::Alignment::Center);
        for item in bar.items {
            let selected = Some(&item.value) == bar.selected.as_ref();
            let message = if !bar.disabled && !item.disabled {
                bar.handler.as_ref().map(|f| f(item.value))
            } else {
                None
            };
            row = row.push(destination(
                NavigationItem {
                    value: (),
                    label: item.label,
                    icon: item.icon,
                    badge: item.badge,
                    disabled: item.disabled,
                },
                selected,
                message,
                NavigationLayout::Bar,
            ));
        }
        crate::focus::group_with_active(
            widget::container(row)
                .padding([4, 8])
                .center_y(80)
                .width(Length::Fill)
                .style(|theme: &Theme| widget::container::Style {
                    background: Some(theme.colors.surface_container.into()),
                    ..Default::default()
                }),
            active,
        )
    }
}
impl<'a, Value: PartialEq + 'a, Message: Clone + 'a> From<NavigationRail<'a, Value, Message>>
    for Element<'a, Message>
{
    fn from(rail: NavigationRail<'a, Value, Message>) -> Self {
        let layout = if rail.expanded {
            NavigationLayout::ExpandedRail
        } else {
            NavigationLayout::Rail
        };
        let width = layout.rail_width();
        let active = rail
            .items
            .iter()
            .filter(|i| !i.disabled)
            .position(|i| Some(&i.value) == rail.selected.as_ref())
            .unwrap_or(0);
        let mut column = widget::Column::new()
            .spacing(if rail.expanded { 4 } else { 8 })
            .align_x(iced::Alignment::Center)
            .width(Length::Fill);
        for item in rail.items {
            let selected = Some(&item.value) == rail.selected.as_ref();
            let message = if !rail.disabled && !item.disabled {
                rail.handler.as_ref().map(|f| f(item.value))
            } else {
                None
            };
            column = column.push(destination(
                NavigationItem {
                    value: (),
                    label: item.label,
                    icon: item.icon,
                    badge: item.badge,
                    disabled: item.disabled,
                },
                selected,
                message,
                layout,
            ));
        }
        let destinations = crate::focus::group_with_active(
            widget::scrollable(column)
                .width(Length::Fill)
                .height(Length::Fill),
            active,
        );
        let mut body = widget::Column::new();
        if let Some(header) = rail.header {
            body = body
                .push(widget::container(header).center_x(Length::Fill))
                .push(widget::space().height(16));
        }
        let mut body = body.push(destinations).width(Length::Fill).spacing(8);
        if let Some(footer) = rail.footer {
            body = body.push(widget::container(footer).center_x(Length::Fill));
        }
        crate::rail_motion::animated_width(
            widget::container(body)
                .padding(if rail.expanded {
                    iced::Padding::from([12, 20])
                } else {
                    iced::Padding::from([12, 0])
                })
                .width(width)
                .height(Length::Fill)
                .style(move |theme: &Theme| widget::container::Style {
                    background: (!rail.modal_surface).then_some(
                        if rail.expanded {
                            theme.colors.surface_container
                        } else {
                            theme.colors.surface
                        }
                        .into(),
                    ),
                    ..Default::default()
                }),
            width,
        )
    }
}

/// An expanded rail presented over the leading edge with a scrim. Visibility and
/// selection remain app-owned; keep this host mounted through its exit animation.
/// A destination action does not implicitly close it: update `open` in your app.
pub fn modal_navigation_rail<'a, Value: PartialEq + 'a, Message: Clone + 'a>(
    background: impl Into<Element<'a, Message>>,
    mut rail: NavigationRail<'a, Value, Message>,
    open: bool,
    on_dismiss: Message,
) -> Element<'a, Message> {
    rail.expanded = true;
    rail.modal_surface = true;
    crate::sheet::host(
        background,
        crate::sheet::navigation_panel(rail)
            .open(open)
            .on_dismiss(on_dismiss),
    )
}
