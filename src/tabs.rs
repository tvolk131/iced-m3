//! Controlled primary and secondary tabs with a finite animated indicator.
use crate::{
    Button, ButtonVariant, Element, Theme, TypeScale, badge, motion::Transition, tokens, typography,
};
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, overlay, renderer,
    widget::{Operation, Tree, tree},
};
use iced::{
    Background, Border, Color, Event, Length, Point, Rectangle, Renderer, Size, Vector, mouse,
    widget,
};
use std::cell::Cell;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabVariant {
    #[default]
    Primary,
    Secondary,
}
pub struct Tab<'a, Value, Message> {
    value: Value,
    label: String,
    icon: Option<Element<'a, Message>>,
    badge: Option<u32>,
    disabled: bool,
}
impl<'a, Value, Message> Tab<'a, Value, Message> {
    pub fn new(value: Value, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
            icon: None,
            badge: None,
            disabled: false,
        }
    }
    pub fn icon(mut self, icon: impl Into<Element<'a, Message>>) -> Self {
        self.icon = Some(icon.into());
        self
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
pub struct Tabs<'a, Value, Message> {
    items: Vec<Tab<'a, Value, Message>>,
    selected: Option<Value>,
    on_select: Option<Box<dyn Fn(Value) -> Message + 'a>>,
    variant: TabVariant,
    scrollable: bool,
    disabled: bool,
    width: Length,
    background: Option<Box<dyn Fn(&Theme) -> Background + 'a>>,
}
pub fn tabs<'a, Value, Message>(
    items: impl IntoIterator<Item = Tab<'a, Value, Message>>,
    selected: Option<Value>,
) -> Tabs<'a, Value, Message> {
    Tabs {
        items: items.into_iter().collect(),
        selected,
        on_select: None,
        variant: TabVariant::Primary,
        scrollable: false,
        disabled: false,
        width: Length::Fill,
        background: None,
    }
}
impl<'a, Value, Message> Tabs<'a, Value, Message> {
    pub fn on_select(mut self, handler: impl Fn(Value) -> Message + 'a) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }
    pub fn variant(mut self, variant: TabVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Scroll horizontally instead of distributing equal widths. Labels remain on one line.
    pub fn scrollable(mut self, scrollable: bool) -> Self {
        self.scrollable = scrollable;
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
    /// Set an explicit container fill. Tabs are transparent by default so they
    /// compose on the enclosing surface. The fill includes the scrollbar region.
    pub fn background(mut self, background: impl Into<Background>) -> Self {
        let background = background.into();
        self.background = Some(Box::new(move |_| background));
        self
    }
    /// Resolve a container color or gradient from the active theme.
    pub fn background_with<B: Into<Background>>(
        mut self,
        background: impl Fn(&Theme) -> B + 'a,
    ) -> Self {
        self.background = Some(Box::new(move |theme| background(theme).into()));
        self
    }
}
impl<'a, Value: PartialEq + 'a, Message: Clone + 'a> From<Tabs<'a, Value, Message>>
    for Element<'a, Message>
{
    fn from(tabs: Tabs<'a, Value, Message>) -> Self {
        let selected = tabs
            .items
            .iter()
            .position(|tab| Some(&tab.value) == tabs.selected.as_ref());
        let height =
            if tabs.variant == TabVariant::Primary && tabs.items.iter().any(|t| t.icon.is_some()) {
                tokens::size::TAB_WITH_ICON
            } else {
                tokens::size::TAB
            };
        let active = tabs
            .items
            .iter()
            .filter(|i| !i.disabled)
            .position(|i| Some(&i.value) == tabs.selected.as_ref())
            .unwrap_or(0);
        let enabled = !tabs.disabled && tabs.on_select.is_some();
        let variant = tabs.variant;
        let children = tabs
            .items
            .into_iter()
            .enumerate()
            .map(|(index, item)| {
                let active = selected == Some(index);
                let mut label = widget::row![
                    typography(item.label, TypeScale::TitleSmall)
                        .wrapping(widget::text::Wrapping::None)
                ]
                .spacing(6)
                .align_y(iced::Alignment::Center);
                if let Some(count) = item.badge
                    && (item.icon.is_none() || variant == TabVariant::Secondary)
                {
                    label = label.push(badge(count));
                }
                let content: Element<'a, Message> = if let Some(icon) = item.icon {
                    let icon = widget::container(icon).center_x(24).center_y(24);
                    if variant == TabVariant::Primary {
                        let icon: Element<'a, Message> = if let Some(count) = item.badge {
                            widget::stack![
                                widget::container(icon).center_x(56).center_y(24),
                                widget::container(badge(count))
                                    .align_right(Length::Fill)
                                    .align_top(Length::Fill)
                            ]
                            .into()
                        } else {
                            icon.into()
                        };
                        widget::column![icon, label]
                            .spacing(2)
                            .align_x(iced::Alignment::Center)
                            .into()
                    } else {
                        widget::row![icon, label]
                            .spacing(8)
                            .align_y(iced::Alignment::Center)
                            .into()
                    }
                } else {
                    label.into()
                };
                let message = if enabled && !item.disabled {
                    tabs.on_select.as_ref().map(|handler| handler(item.value))
                } else {
                    None
                };
                // Clicking the current tab is harmless and reports the same controlled value.
                Button::new(content)
                    .variant(ButtonVariant::Text)
                    .radius(0.0)
                    .height(height)
                    .padding([8, 16])
                    .width(if tabs.scrollable {
                        Length::Shrink
                    } else {
                        Length::Fill
                    })
                    .palette(move |theme| {
                        (
                            Color::TRANSPARENT,
                            if active && variant == TabVariant::Primary {
                                theme.colors.primary
                            } else if active {
                                theme.colors.on_surface
                            } else {
                                theme.colors.on_surface_variant
                            },
                        )
                    })
                    .on_press_maybe(message)
                    .into()
            })
            .collect::<Vec<_>>();
        let strip: Element<'a, Message> = Element::new(Strip {
            row: widget::Row::with_children(children)
                .width(if tabs.scrollable {
                    Length::Shrink
                } else {
                    tabs.width
                })
                .into(),
            selected,
            variant,
        });
        let group = crate::focus::group_with_active(
            if tabs.scrollable {
                widget::scrollable(strip)
                    .width(tabs.width)
                    .direction(widget::scrollable::Direction::Horizontal(
                        widget::scrollable::Scrollbar::new()
                            .width(3)
                            .scroller_width(3)
                            .spacing(3),
                    ))
                    .into()
            } else {
                strip
            },
            active,
        );
        widget::container(group)
            .width(tabs.width)
            .style(move |theme: &Theme| widget::container::Style {
                background: tabs.background.as_ref().map(|background| background(theme)),
                ..Default::default()
            })
            .into()
    }
}
struct Strip<'a, Message> {
    row: Element<'a, Message>,
    selected: Option<usize>,
    variant: TabVariant,
}
struct State {
    x: Transition,
    width: Transition,
    initialized: bool,
    motion: Cell<tokens::Motion>,
}
impl<Message> Strip<'_, Message> {
    fn target(&self, layout: Layout<'_>) -> Option<(f32, f32)> {
        let tab = layout.children().nth(self.selected?)?;
        let bounds = tab.bounds();
        let width = if self.variant == TabVariant::Primary {
            tab.child(0)
                .bounds()
                .width
                .min(bounds.width)
                .max(24.0)
                .min(bounds.width)
        } else {
            bounds.width
        };
        Some((bounds.center_x() - width / 2.0 - layout.bounds().x, width))
    }
}
impl<Message: Clone> Widget<Message, Theme, Renderer> for Strip<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State {
            x: Transition::new(0.0),
            width: Transition::new(0.0),
            initialized: false,
            motion: Cell::new(tokens::Motion::default()),
        })
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.row)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.children[0].diff(&self.row);
        if self.selected.is_none() {
            tree.state.downcast_mut::<State>().initialized = false;
        }
    }
    fn size(&self) -> Size<Length> {
        self.row.as_widget().size()
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.row
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        op: &mut dyn Operation,
    ) {
        self.row
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, op);
    }
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.row.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        if let Some((x, width)) = self.target(layout) {
            let state = tree.state.downcast_mut::<State>();
            if !state.initialized {
                state.x = Transition::new(x);
                state.width = Transition::new(width);
                state.initialized = true;
            } else {
                let now = match event {
                    Event::Window(iced::window::Event::RedrawRequested(now)) => *now,
                    _ => crate::motion::now(),
                };
                let duration = state.motion.get().medium;
                let changed = state.x.set(x, now, duration) | state.width.set(width, now, duration);
                let active = state.x.tick(now) | state.width.tick(now);
                if changed || active {
                    shell.request_redraw();
                }
            }
        }
    }
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;
        let bounds = layout.bounds();
        let Some(clip) = bounds.intersection(viewport) else {
            return;
        };
        let state = tree.state.downcast_ref::<State>();
        state.motion.set(theme.motion);
        renderer.with_layer(clip, |renderer| {
            self.row.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor,
                viewport,
            );
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle::new(
                        Point::new(bounds.x, bounds.y + bounds.height - 1.0),
                        Size::new(bounds.width, 1.0),
                    ),
                    ..Default::default()
                },
                theme.colors.outline_variant,
            );
            if let Some(target) = self.target(layout) {
                let (x, width) = if state.initialized {
                    (state.x.value, state.width.value)
                } else {
                    target
                };
                let height = if self.variant == TabVariant::Primary {
                    3.0
                } else {
                    2.0
                };
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle::new(
                            Point::new(bounds.x + x, bounds.y + bounds.height - height),
                            Size::new(width, height),
                        ),
                        border: Border {
                            radius: if self.variant == TabVariant::Primary {
                                iced::border::Radius {
                                    top_left: 3.0,
                                    top_right: 3.0,
                                    bottom_left: 0.0,
                                    bottom_right: 0.0,
                                }
                            } else {
                                0.0.into()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    theme.colors.primary,
                );
            }
        });
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.row.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.row.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}
