//! Controlled search bar with an expanding, scrollable search view.
use crate::{
    Button, ButtonVariant, Element, Theme, TypeScale, glyph::Glyph, motion::Transition, tokens,
    typography,
};
use iced::advanced::{
    Clipboard, Layout, Overlay, Shell, Widget, layout, overlay, renderer,
    widget::{Operation, Tree, tree},
};
use iced::{
    Alignment, Border, Color, Event, Length, Point, Rectangle, Renderer, Size, Vector, keyboard,
    mouse, widget, window,
};
use std::{cell::Cell, rc::Rc};

/// Keep the bar mounted when closed to preserve editing/scroll state and exit motion.
/// Results, query, and visibility belong to the application. The view captures
/// outside input; Escape, the back button, and outside clicks request dismissal.
pub struct Search<'a, Message> {
    placeholder: String,
    value: String,
    results: Element<'a, Message>,
    open: bool,
    on_open: Option<Message>,
    on_close: Option<Message>,
    on_input: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_submit: Option<Message>,
    width: Length,
    height: f32,
    full_screen: Option<bool>,
}
pub fn search_bar<'a, Message: 'a>(
    placeholder: impl Into<String>,
    value: &str,
) -> Search<'a, Message> {
    Search {
        placeholder: placeholder.into(),
        value: value.into(),
        results: widget::space().into(),
        open: false,
        on_open: None,
        on_close: None,
        on_input: None,
        on_submit: None,
        width: Length::Fill,
        height: 400.0,
        full_screen: None,
    }
}
impl<'a, Message: 'a> Search<'a, Message> {
    pub fn results(mut self, results: impl Into<Element<'a, Message>>) -> Self {
        self.results = results.into();
        self
    }
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
    pub fn on_open(mut self, message: Message) -> Self {
        self.on_open = Some(message);
        self
    }
    pub fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
        self
    }
    pub fn on_input(mut self, callback: impl Fn(String) -> Message + 'a) -> Self {
        self.on_input = Some(Box::new(callback));
        self
    }
    pub fn on_submit(mut self, message: Message) -> Self {
        self.on_submit = Some(message);
        self
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
    /// Desired docked height, clamped to the window. The default is 400px.
    pub fn height(mut self, height: f32) -> Self {
        if height.is_finite() && height >= 56.0 {
            self.height = height;
        }
        self
    }
    /// Override automatic full-window presentation below a 600px window width.
    pub fn full_screen(mut self, full_screen: bool) -> Self {
        self.full_screen = Some(full_screen);
        self
    }
}
impl<'a, Message: Clone + 'a> From<Search<'a, Message>> for Element<'a, Message> {
    fn from(search: Search<'a, Message>) -> Self {
        let input_id = widget::Id::unique();
        let empty = search.value.is_empty();
        let label = if empty {
            search.placeholder.clone()
        } else {
            search.value.clone()
        };
        let bar = Button::new(
            widget::row![
                Element::new(Glyph::Search),
                typography(label, TypeScale::BodyLarge)
                    .width(Length::Fill)
                    .wrapping(widget::text::Wrapping::None)
            ]
            .spacing(16)
            .align_y(Alignment::Center),
        )
        .height(56)
        .padding([0, 16])
        .width(Length::Fill)
        .radius(28.0)
        .palette(move |t| {
            (
                t.colors.surface_container_high,
                if empty {
                    t.colors.on_surface_variant
                } else {
                    t.colors.on_surface
                },
            )
        })
        .on_press_maybe(search.on_open);
        let action = |glyph, message| {
            Button::new(Element::new(glyph))
                .variant(ButtonVariant::Text)
                .width(40)
                .height(40)
                .padding(8)
                .palette(|t| (Color::TRANSPARENT, t.colors.on_surface))
                .on_press_maybe(message)
        };
        let clear = search
            .on_input
            .as_ref()
            .map(|callback| callback(String::new()));
        let mut input = widget::text_input(&search.placeholder, &search.value)
            .id(input_id.clone())
            .font(crate::fonts::REGULAR)
            .size(16)
            .line_height(widget::text::LineHeight::Absolute(24.0.into()))
            .padding([8, 0])
            .width(Length::Fill)
            .on_submit_maybe(search.on_submit)
            .style(|t: &Theme, _| widget::text_input::Style {
                background: Color::TRANSPARENT.into(),
                border: Border::default(),
                icon: t.colors.on_surface_variant,
                placeholder: t.colors.on_surface_variant,
                value: t.colors.on_surface,
                selection: t.colors.secondary_container,
            });
        if let Some(callback) = search.on_input {
            input = input.on_input(callback);
        }
        let mut header = widget::row![action(Glyph::Back, search.on_close.clone()), input]
            .spacing(8)
            .align_y(Alignment::Center)
            .height(56)
            .padding([0, 8]);
        // A stable trailing slot preserves the input's tree and width while clearing.
        header = header.push(if empty {
            widget::space().width(40).height(40).into()
        } else {
            crate::staged::part(
                action(Glyph::Close, clear),
                crate::staged::Part::SearchClear,
            )
        });
        let full_header = Rc::new(Cell::new(search.full_screen.unwrap_or(false)));
        let panel = widget::column![
            crate::staged::header(header, full_header.clone()),
            crate::staged::part(crate::divider(), crate::staged::Part::SearchResults),
            widget::container(
                widget::scrollable(
                    widget::container(crate::staged::part(
                        search.results,
                        crate::staged::Part::SearchResults
                    ))
                    .width(Length::Fill)
                    .padding([0, 8])
                )
                .direction(widget::scrollable::Direction::Vertical(
                    widget::scrollable::Scrollbar::new()
                        .width(4)
                        .scroller_width(4)
                        .spacing(4)
                ))
                .width(Length::Fill)
                .height(Length::Fill)
            )
            .padding([16, 8])
            .width(Length::Fill)
            .height(Length::Fill)
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into();
        Element::new(SearchWidget {
            bar: bar.into(),
            panel,
            input_id,
            open: search.open,
            close: search.on_close,
            width: search.width,
            height: search.height,
            full_screen: search.full_screen,
            full_header,
        })
    }
}
struct SearchWidget<'a, Message> {
    full_header: Rc<Cell<bool>>,
    bar: Element<'a, Message>,
    panel: Element<'a, Message>,
    input_id: widget::Id,
    open: bool,
    close: Option<Message>,
    width: Length,
    height: f32,
    full_screen: Option<bool>,
}
struct State {
    inactive: bool,
    bounds: Rectangle,
    shadow: crate::elevation::Cache,
    progress: Transition,
    was_open: bool,
    focus_pending: bool,
    unfocus_pending: bool,
    outside_pressed: bool,
    motion: Cell<tokens::Motion>,
}
impl State {
    fn present(&self, open: bool) -> bool {
        open || self.progress.value > 0.0
    }
}
impl<Message: Clone> Widget<Message, Theme, Renderer> for SearchWidget<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State {
            inactive: false,
            bounds: Rectangle::default(),
            shadow: Default::default(),
            progress: Transition::standard(f32::from(self.open)),
            was_open: self.open,
            focus_pending: self.open,
            unfocus_pending: false,
            outside_pressed: false,
            motion: Cell::new(tokens::Motion::default()),
        })
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.bar), Tree::new(&self.panel)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.children[0].diff(&self.bar);
        tree.children[1].diff(&self.panel);
        let state = tree.state.downcast_mut::<State>();
        if state.was_open != self.open {
            state.was_open = self.open;
            state.focus_pending = self.open;
            state.unfocus_pending = !self.open;
            state.outside_pressed = false;
        }
    }
    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Fixed(56.0))
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits
            .max_width(720.0)
            .resolve(self.width, 56.0, Size::new(360.0, 56.0));
        self.bar.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &layout::Limits::new(size, size),
        )
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        if !tree.state.downcast_ref::<State>().present(self.open) {
            self.bar
                .as_widget_mut()
                .operate(&mut tree.children[0], layout, renderer, operation);
        }
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
        let state = tree.state.downcast_mut::<State>();
        if let Some(focused) = crate::activity::window_focus(event) {
            state.inactive = !focused;
        }
        let now = match event {
            Event::Window(window::Event::RedrawRequested(now)) => *now,
            _ => crate::motion::now(),
        };
        let before = state.progress.value;
        let motion = state.motion.get();
        let changed = state.progress.set(
            f32::from(self.open),
            now,
            if self.open {
                motion.search_enter
            } else {
                motion.search_exit
            },
        );
        if state.progress.tick(now) || changed {
            shell.request_redraw();
        }
        if before != state.progress.value {
            shell.invalidate_layout();
        }
        if !state.present(self.open) {
            self.bar.as_widget_mut().update(
                &mut tree.children[0],
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
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
        let state = tree.state.downcast_ref::<State>();
        state.motion.set(theme.motion);
        self.bar.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            if state.present(self.open) {
                mouse::Cursor::Unavailable
            } else {
                cursor
            },
            viewport,
        );
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if tree.state.downcast_ref::<State>().present(self.open) {
            mouse::Interaction::None
        } else {
            self.bar.as_widget().mouse_interaction(
                &tree.children[0],
                layout,
                cursor,
                viewport,
                renderer,
            )
        }
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        _: &Renderer,
        _: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_mut::<State>();
        if !state.present(self.open) {
            return None;
        }
        Some(overlay::Element::new(Box::new(SearchOverlay {
            panel: &mut self.panel,
            tree: &mut tree.children[1],
            state,
            input_id: &self.input_id,
            anchor: layout.bounds() + translation,
            open: self.open,
            close: &self.close,
            height: self.height,
            full_screen: self.full_screen,
            full_header: &self.full_header,
        })))
    }
}
struct SearchOverlay<'a, 'b, Message> {
    full_header: &'a Cell<bool>,
    panel: &'a mut Element<'b, Message>,
    tree: &'a mut Tree,
    state: &'a mut State,
    input_id: &'a widget::Id,
    anchor: Rectangle,
    open: bool,
    close: &'a Option<Message>,
    height: f32,
    full_screen: Option<bool>,
}
impl<Message: Clone> Overlay<Message, Theme, Renderer> for SearchOverlay<'_, '_, Message> {
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let target = view_bounds(self.anchor, bounds, self.height, self.full_screen);
        self.full_header
            .set(self.full_screen.unwrap_or(bounds.width < 600.));
        let p = self.state.progress.value;
        let area = Rectangle {
            x: self.anchor.x + (target.x - self.anchor.x) * p,
            y: self.anchor.y + (target.y - self.anchor.y) * p,
            width: self.anchor.width + (target.width - self.anchor.width) * p,
            height: self.anchor.height + (target.height - self.anchor.height) * p,
        };
        self.state.bounds = area;
        let node = self
            .panel
            .as_widget_mut()
            .layout(
                self.tree,
                renderer,
                &layout::Limits::new(target.size(), target.size()),
            )
            .move_to(area.position());
        if self.state.unfocus_pending {
            self.state.unfocus_pending = false;
            self.panel.as_widget_mut().operate(
                self.tree,
                Layout::new(&node),
                renderer,
                &mut iced::advanced::widget::operation::focusable::unfocus::<()>(),
            );
        }
        if self.state.focus_pending {
            self.state.focus_pending = false;
            self.panel.as_widget_mut().operate(
                self.tree,
                Layout::new(&node),
                renderer,
                &mut iced::advanced::widget::operation::focusable::focus::<()>(
                    self.input_id.clone(),
                ),
            );
        }
        layout::Node::with_children(bounds, vec![node])
    }
    fn operate(&mut self, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        if self.open {
            self.panel
                .as_widget_mut()
                .operate(self.tree, layout.child(0), renderer, operation);
        }
    }
    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let motion = self.state.motion.get();
        let _phase = crate::staged::enter(crate::staged::Phase {
            kind: crate::staged::Kind::Search,
            open: self.open,
            initial: self.state.progress.value,
            base: 1.,
            enter: motion.search_enter,
            exit: motion.search_exit,
            above: false,
        });
        if !self.open {
            crate::activity::update_covered(
                self.panel,
                self.tree,
                event,
                layout.child(0),
                renderer,
                clipboard,
                shell,
                &layout.bounds(),
                !self.state.inactive,
            );
            if !matches!(event, Event::Window(_)) {
                shell.capture_event();
            }
            return;
        }
        if crate::focus::tab(
            self.panel,
            self.tree,
            layout.child(0),
            renderer,
            event,
            shell,
        ) {
            return;
        }
        if matches!(event, Event::Mouse(mouse::Event::ButtonPressed(_))) {
            crate::focus::clear(self.panel, self.tree, layout.child(0), renderer);
        }
        let outside = cursor.position().is_some() && !cursor.is_over(self.state.bounds);
        let mut dismiss = matches!(
            event,
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            })
        );
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                self.state.outside_pressed = outside
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                dismiss |= self.state.outside_pressed && outside;
                self.state.outside_pressed = false;
            }
            Event::Window(window::Event::Unfocused) | Event::Mouse(mouse::Event::CursorLeft) => {
                self.state.outside_pressed = false
            }
            _ => {}
        }
        if dismiss {
            if self.open
                && let Some(message) = self.close
            {
                shell.publish(message.clone());
            }
        } else if self.open {
            self.panel.as_widget_mut().update(
                self.tree,
                event,
                layout.child(0),
                cursor,
                renderer,
                clipboard,
                shell,
                &self.state.bounds,
            );
        }
        if !matches!(event, Event::Window(_)) {
            shell.capture_event();
        }
    }
    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        _: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        use iced::advanced::Renderer as _;
        let panel = layout.child(0);
        let bounds = self.state.bounds;
        let full = self.full_screen.unwrap_or(layout.bounds().width < 600.0);
        let radius = if full {
            28.0 * (1.0 - self.state.progress.value)
        } else {
            28.0
        };
        crate::elevation::draw_resizing(
            renderer,
            theme,
            bounds,
            panel.bounds().height,
            radius,
            if full { 0.0 } else { 3.0 },
            layout.bounds(),
            &self.state.shadow,
        );
        // A separate surface layer covers background text and SVG primitives on both renderers.
        renderer.with_layer(layout.bounds(), |renderer| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        radius: radius.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                theme.colors.surface_container_high,
            )
        });
        let Some(clip) = bounds.intersection(&layout.bounds()) else {
            return;
        };
        renderer.with_layer(clip, |renderer| {
            self.panel.as_widget().draw(
                self.tree,
                renderer,
                theme,
                &renderer::Style {
                    text_color: theme.colors.on_surface,
                },
                panel,
                if self.open {
                    cursor
                } else {
                    mouse::Cursor::Unavailable
                },
                &clip,
            )
        });
    }
    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.open {
            self.panel.as_widget().mouse_interaction(
                self.tree,
                layout.child(0),
                cursor,
                &self.state.bounds,
                renderer,
            )
        } else {
            mouse::Interaction::None
        }
    }
    fn overlay<'a>(
        &'a mut self,
        layout: Layout<'a>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        if self.open {
            self.panel.as_widget_mut().overlay(
                self.tree,
                layout.child(0),
                renderer,
                &self.state.bounds,
                Vector::ZERO,
            )
        } else {
            None
        }
    }
}
fn view_bounds(anchor: Rectangle, window: Size, height: f32, full: Option<bool>) -> Rectangle {
    if full.unwrap_or(window.width < 600.0) {
        return Rectangle::new(Point::ORIGIN, window);
    }
    let margin = 8.0_f32.min(window.width / 4.0).min(window.height / 4.0);
    let width = anchor
        .width
        .max(360.0)
        .min((window.width - 2.0 * margin).max(0.0));
    let height = height.min((window.height - 2.0 * margin).max(0.0));
    Rectangle {
        x: anchor
            .x
            .clamp(margin, (window.width - margin - width).max(margin)),
        y: anchor
            .y
            .clamp(margin, (window.height - margin - height).max(margin)),
        width,
        height,
    }
}
