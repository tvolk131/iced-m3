//! Shared viewport-aware placement for menus and plain tooltips.
use crate::{Element, Theme, tokens};
use iced::advanced::{
    Clipboard, Layout, Overlay, Shell, layout, renderer,
    widget::{Operation, Tree},
};
use iced::{Event, Point, Rectangle, Renderer, Size, Vector, keyboard, mouse, window};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Placement {
    #[default]
    BottomStart,
    BottomEnd,
    Bottom,
    TopStart,
    TopEnd,
    Top,
    Left,
    Right,
    /// A cascading panel aligned with the top of its row; flips at the window edge.
    RightStart,
}
impl Placement {
    fn above(self) -> bool {
        matches!(self, Self::Top | Self::TopStart | Self::TopEnd)
    }
    fn horizontal(self) -> bool {
        matches!(self, Self::Left | Self::Right | Self::RightStart)
    }
}

pub(crate) fn place(
    anchor: Rectangle,
    size: Size,
    viewport: Size,
    placement: Placement,
    gap: f32,
) -> Point {
    let margin = tokens::size::POPUP_MARGIN
        .min(viewport.width / 4.0)
        .min(viewport.height / 4.0);
    let below = viewport.height - margin - anchor.y - anchor.height - gap;
    let above = anchor.y - margin - gap;
    let (mut x, mut y) = if placement.horizontal() {
        let left = anchor.x - margin - gap;
        let right = viewport.width - margin - anchor.x - anchor.width - gap;
        let use_left = if placement == Placement::Left {
            left >= size.width || left >= right
        } else {
            right < size.width && left > right
        };
        (
            if use_left {
                anchor.x - gap - size.width
            } else {
                anchor.x + anchor.width + gap
            },
            if placement == Placement::RightStart {
                anchor.y
            } else {
                anchor.center_y() - size.height / 2.0
            },
        )
    } else {
        let use_above = if placement.above() {
            above >= size.height || above >= below
        } else {
            below < size.height && above > below
        };
        let x = match placement {
            Placement::BottomEnd | Placement::TopEnd => anchor.x + anchor.width - size.width,
            Placement::Bottom | Placement::Top => anchor.center_x() - size.width / 2.0,
            _ => anchor.x,
        };
        (
            x,
            if use_above {
                anchor.y - gap - size.height
            } else {
                anchor.y + anchor.height + gap
            },
        )
    };
    x = x.clamp(margin, (viewport.width - margin - size.width).max(margin));
    y = y.clamp(margin, (viewport.height - margin - size.height).max(margin));
    Point::new(x, y)
}

#[derive(Default)]
pub(crate) struct PointerIntent {
    pub child: Option<Rectangle>,
    last: Option<Point>,
    origin: Option<Point>,
    until: Option<iced::time::Instant>,
}
impl PointerIntent {
    fn track(&mut self, point: Point, now: iced::time::Instant) {
        let previous = self.last.replace(point);
        let Some(child) = self.child else {
            self.until = None;
            self.origin = None;
            return;
        };
        let Some(previous) = previous else {
            return;
        };
        let origin = self.origin.unwrap_or(previous);
        let edge = if origin.x <= child.x {
            child.x
        } else {
            child.x + child.width
        };
        let toward = (point.x - edge).abs() < (previous.x - edge).abs();
        let t = (point.x - origin.x) / (edge - origin.x);
        let top = origin.y + (child.y - 8.0 - origin.y) * t;
        let bottom = origin.y + (child.y + child.height + 8.0 - origin.y) * t;
        if !child.contains(point)
            && toward
            && (0.0..=1.0).contains(&t)
            && point.y >= top
            && point.y <= bottom
        {
            self.origin = Some(origin);
            self.until
                .get_or_insert(now + std::time::Duration::from_millis(300));
        } else {
            self.origin = None;
            self.until = None;
        }
    }
    pub fn deadline(&self, now: iced::time::Instant) -> Option<iced::time::Instant> {
        self.until.filter(|until| now < *until)
    }
}

#[derive(Default)]
pub(crate) struct PopupState {
    inactive: bool,
    shadow: crate::elevation::Cache,
    pub open: bool,
    pub outside_pressed: bool,
    pub initial_focus: Option<usize>,
    pub signal: Arc<AtomicU64>,
    pub generation: u64,
    pub child: Arc<AtomicU64>,
    pub ancestors: Vec<Rectangle>,
    typeahead: String,
    typed_at: Option<iced::time::Instant>,
    pub pointer: Arc<Mutex<PointerIntent>>,
    pub parent_pointer: Option<Arc<Mutex<PointerIntent>>>,
    pub presence: crate::presence::Presence,
}
impl PopupState {
    pub fn show(&mut self, now: iced::time::Instant) {
        if !self.open {
            self.typeahead.clear();
            self.typed_at = None;
        }
        self.open = true;
        let motion = self.presence.motion.get();
        self.presence
            .begin_surface(true, now, motion.menu_enter, motion.menu_exit, false);
    }
    pub fn synchronize(&mut self) {
        let generation = self.signal.load(Ordering::Relaxed);
        if self.generation != generation {
            self.open = false;
            self.outside_pressed = false;
            self.generation = generation;
        }
    }
    pub fn dismiss_tree<Message>(&mut self, shell: &mut Shell<'_, Message>) {
        self.signal.fetch_add(1, Ordering::Relaxed);
        self.close(shell);
    }

    pub fn close<Message>(&mut self, shell: &mut Shell<'_, Message>) {
        self.open = false;
        self.child.store(0, Ordering::Relaxed);
        *self.pointer.lock().unwrap() = PointerIntent::default();
        self.outside_pressed = false;
        self.typeahead.clear();
        self.typed_at = None;
        shell.invalidate_layout();
        shell.request_redraw();
    }
}
pub(crate) struct Anchored<'a, 'b, Message> {
    pub content: &'a mut Element<'b, Message>,
    pub tree: &'a mut Tree,
    pub anchor: Rectangle,
    pub placement: Placement,
    pub width: f32,
    pub max_height: f32,
    pub gap: f32,
    pub menu: Option<&'a mut PopupState>,
    pub close_when: Option<&'a dyn Fn(&Message) -> bool>,
    pub labels: &'a [String],
    pub reveal: Option<f32>,
    pub surface: Option<(f32, u8)>,
    pub plain_surface: bool,
    pub surface_high: bool,
}
impl<Message: Clone> Overlay<Message, Theme, Renderer> for Anchored<'_, '_, Message> {
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let margin = tokens::size::POPUP_MARGIN
            .min(bounds.width / 4.0)
            .min(bounds.height / 4.0);
        let available = if self.menu.is_some() && !self.placement.horizontal() {
            (self.anchor.y - self.gap - margin)
                .max(bounds.height - self.anchor.y - self.anchor.height - self.gap - margin)
        } else {
            bounds.height - 2.0 * margin
        };
        let max = Size::new(
            self.width.min((bounds.width - 2.0 * margin).max(0.0)),
            self.max_height.min(available.max(0.0)),
        );
        let content = self.content.as_widget_mut().layout(
            self.tree,
            renderer,
            &layout::Limits::new(Size::ZERO, max),
        );
        let point = place(
            self.anchor,
            content.size(),
            bounds,
            self.placement,
            self.gap,
        );
        if let Some(state) = &self.menu
            && state.open
            && let Some(parent) = &state.parent_pointer
        {
            parent.lock().unwrap().child = Some(Rectangle::new(point, content.size()));
        }
        layout::Node::with_children(bounds, vec![content.move_to(point)])
    }
    fn operate(&mut self, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        if self.menu.as_ref().is_some_and(|state| !state.open) {
            return;
        }
        self.content
            .as_widget_mut()
            .operate(self.tree, layout.child(0), renderer, operation);
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
        let Some(state) = &mut self.menu else {
            return;
        };
        if let Some(focused) = crate::activity::window_focus(event) {
            state.inactive = !focused;
        }
        let motion = state.presence.motion.get();
        state.presence.update_surface(
            state.open,
            event,
            motion.menu_enter,
            motion.menu_exit,
            false,
            shell,
        );
        let _phase = crate::staged::enter(crate::staged::Phase {
            kind: if self.plain_surface {
                crate::staged::Kind::Menu
            } else {
                crate::staged::Kind::Popup
            },
            open: state.open,
            initial: state.presence.progress.value,
            base: 1.,
            enter: motion.menu_enter,
            exit: motion.menu_exit,
            above: layout.child(0).bounds().center_y() < self.anchor.center_y()
                && !self.placement.horizontal(),
        });
        if !state.open {
            crate::activity::update_covered(
                self.content,
                self.tree,
                event,
                layout.child(0),
                renderer,
                clipboard,
                shell,
                &layout.bounds(),
                !state.inactive,
            );
            if matches!(event, Event::Mouse(_) | Event::Touch(_))
                && cursor.is_over(layout.child(0).bounds())
            {
                shell.capture_event();
            }
            return;
        }
        let panel = layout.child(0);
        if let Some(index) = state.initial_focus.take() {
            if index == usize::MAX {
                crate::focus::cycle(self.content, self.tree, panel, renderer, true);
            } else {
                crate::focus::select(self.content, self.tree, panel, renderer, index);
            }
            shell.request_redraw();
        }
        // A cascade shares dismissal but allows interaction with ancestor panels.
        if matches!(event, Event::Mouse(_))
            && state.ancestors.iter().any(|b| cursor.is_over(*b))
            && !cursor.is_over(panel.bounds())
        {
            if matches!(
                event,
                Event::Mouse(mouse::Event::ButtonPressed(_) | mouse::Event::ButtonReleased(_))
            ) {
                state.outside_pressed = false;
            }
            return;
        }
        if matches!(
            event,
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::ArrowLeft),
                ..
            })
        ) && !state.ancestors.is_empty()
        {
            state.close(shell);
            shell.capture_event();
            return;
        }
        crate::menu::link(self.content, self.tree, panel, renderer, state);
        if let Event::Mouse(mouse::Event::CursorMoved { position }) = event {
            let mut pointer = state.pointer.lock().unwrap();
            if state.child.load(Ordering::Relaxed) == 0 {
                pointer.child = None;
            }
            pointer.track(*position, crate::motion::now());
        }

        if let Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Character(text),
            modifiers,
            ..
        }) = event
            && !self.labels.is_empty()
            && !modifiers.control()
            && !modifiers.alt()
            && !modifiers.logo()
            && !text.chars().any(char::is_control)
        {
            let now = crate::motion::now();
            if state.typed_at.is_none_or(|t| {
                now.saturating_duration_since(t) > std::time::Duration::from_millis(200)
            }) {
                state.typeahead.clear();
            }
            let text = text.to_lowercase();
            let repeated = state.typeahead == text;
            if !repeated {
                state.typeahead.push_str(&text);
            }
            state.typed_at = Some(now);
            let current = crate::focus::focused_index(self.content, self.tree, panel, renderer);
            let start = current.map_or(0, |i| {
                if repeated || state.typeahead == text {
                    i + 1
                } else {
                    i
                }
            });
            let find = |prefix: &str| {
                (0..self.labels.len())
                    .map(|n| (start + n) % self.labels.len())
                    .find(|i| self.labels[*i].starts_with(prefix))
            };
            let found = find(&state.typeahead).or_else(|| {
                state.typeahead.clone_from(&text);
                find(&text)
            });
            if let Some(index) = found {
                state.child.store(0, Ordering::Relaxed);
                crate::focus::select(self.content, self.tree, panel, renderer, index);
                shell.request_redraw();
            }
            shell.capture_event();
            return;
        }
        if let Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(key),
            ..
        }) = event
            && !self.labels.is_empty()
            && matches!(
                key,
                keyboard::key::Named::ArrowUp
                    | keyboard::key::Named::ArrowDown
                    | keyboard::key::Named::Home
                    | keyboard::key::Named::End
            )
        {
            state.typeahead.clear();
            state.typed_at = None;
            if *key == keyboard::key::Named::Home {
                crate::focus::select(self.content, self.tree, panel, renderer, 0);
            } else {
                if *key == keyboard::key::Named::End {
                    crate::focus::clear(self.content, self.tree, panel, renderer);
                }
                crate::focus::cycle(
                    self.content,
                    self.tree,
                    panel,
                    renderer,
                    matches!(
                        key,
                        keyboard::key::Named::ArrowUp | keyboard::key::Named::End
                    ),
                );
            }
            state.child.store(0, Ordering::Relaxed);
            shell.capture_event();
            shell.request_redraw();
            return;
        }
        let outside = cursor.position().is_some() && !cursor.is_over(panel.bounds());
        if crate::focus::tab(self.content, self.tree, panel, renderer, event, shell) {
            return;
        }
        if matches!(event, Event::Mouse(mouse::Event::ButtonPressed(_))) {
            crate::focus::clear(self.content, self.tree, panel, renderer);
        }

        let escape = matches!(
            event,
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            })
        );
        if escape
            || matches!(event, Event::Window(window::Event::Unfocused))
            || (outside && matches!(event, Event::Mouse(mouse::Event::WheelScrolled { .. })))
        {
            if escape {
                state.close(shell);
            } else {
                state.dismiss_tree(shell);
            }
            if !matches!(event, Event::Window(_)) {
                shell.capture_event();
            }
            return;
        }
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left | mouse::Button::Right,
            )) => state.outside_pressed = outside,
            Event::Mouse(mouse::Event::ButtonReleased(
                mouse::Button::Left | mouse::Button::Right,
            )) => {
                if state.outside_pressed && outside {
                    state.dismiss_tree(shell);
                    shell.capture_event();
                    return;
                }
                state.outside_pressed = false;
            }
            Event::Mouse(mouse::Event::CursorLeft) => state.outside_pressed = false,
            _ => {}
        }
        let mut messages = Vec::new();
        let mut local = Shell::new(&mut messages);
        self.content.as_widget_mut().update(
            self.tree,
            event,
            panel,
            cursor,
            renderer,
            clipboard,
            &mut local,
            &layout.bounds(),
        );
        forward_shell(shell, &local);
        if messages
            .iter()
            .any(|m| self.close_when.is_none_or(|f| f(m)))
        {
            state.dismiss_tree(shell);
        }
        for message in messages {
            shell.publish(message);
        }
        crate::focus::reveal(self.content, self.tree, panel, renderer);
        if !matches!(event, Event::Window(_)) {
            shell.capture_event();
        }
    }
    fn overlay<'c>(
        &'c mut self,
        layout: Layout<'c>,
        renderer: &Renderer,
    ) -> Option<iced::advanced::overlay::Element<'c, Message, Theme, Renderer>> {
        if self.menu.as_ref().is_some_and(|state| !state.open) {
            return None;
        }
        if let Some(state) = &mut self.menu {
            crate::menu::link(self.content, self.tree, layout.child(0), renderer, state);
        }
        self.content.as_widget_mut().overlay(
            self.tree,
            layout.child(0),
            renderer,
            &layout.bounds(),
            Vector::ZERO,
        )
    }
    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        use iced::advanced::Renderer as _;
        let progress = if let Some(state) = &self.menu {
            state.presence.motion.set(theme.motion);
            if !state.presence.visible(state.open) {
                return;
            }
            state.presence.height.value
        } else {
            self.reveal.unwrap_or(1.0)
        };
        if progress <= 0.0 {
            return;
        }
        let bounds = layout.child(0).bounds();
        let height = bounds.height * progress;
        let above = bounds.center_y() < self.anchor.center_y() && !self.placement.horizontal();
        let clip = if progress >= 1.0 {
            layout.bounds()
        } else {
            Rectangle {
                y: if above {
                    bounds.y + bounds.height - height
                } else {
                    bounds.y
                },
                height,
                ..bounds
            }
        };
        if self.menu.is_some() {
            let reveal = Rectangle {
                y: if above {
                    bounds.y + bounds.height - height
                } else {
                    bounds.y
                },
                height,
                ..bounds
            };
            let (radius, elevation) = self.surface.unwrap_or((4.0, 2));
            let fallback = crate::elevation::Cache::default();
            let cache = self.menu.as_ref().map_or(&fallback, |state| &state.shadow);
            let paper = self.menu.as_ref().map_or(1.0, |s| s.presence.paper.value);
            let mut surface_theme = theme.clone();
            surface_theme.colors.shadow.a *= paper;
            renderer.with_layer(layout.bounds(), |renderer| {
                crate::elevation::draw_resizing(
                    renderer,
                    &surface_theme,
                    reveal,
                    bounds.height,
                    radius,
                    elevation as f32,
                    layout.bounds(),
                    cache,
                );
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: reveal,
                        border: iced::Border {
                            radius: radius.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    crate::theme::alpha(
                        if self.surface_high {
                            theme.colors.surface_container_high
                        } else {
                            theme.colors.surface_container
                        },
                        paper,
                    ),
                );
                // Rich content has at least 16px horizontal padding; keep its
                // rectangular backgrounds/fades away from the rounded edge.
                let inset = if self.plain_surface { 0. } else { radius };
                let content_clip = Rectangle {
                    x: reveal.x + inset,
                    width: (reveal.width - 2. * inset).max(0.),
                    y: reveal.y + 8.0,
                    height: (reveal.height - 16.0).max(0.0),
                };
                if content_clip.height > 0.0
                    && (self.plain_surface
                        || self
                            .menu
                            .as_ref()
                            .is_some_and(|s| s.presence.content.value > 0.))
                {
                    let mut content_theme = theme.clone();
                    content_theme.shadows = theme.shadows && progress >= 1.;
                    renderer.with_layer(content_clip, |renderer| {
                        self.content.as_widget().draw(
                            self.tree,
                            renderer,
                            &content_theme,
                            style,
                            layout.child(0),
                            cursor,
                            &content_clip,
                        );
                    });
                }
            });
            return;
        }
        // Paint the outer shadow at the revealed bounds.
        // Paint a shadow with the revealed bounds, and suppress full-height
        // descendant shadows during the reveal. Settled rendering is unchanged.
        let mut clipped_theme = theme.clone();
        if progress < 1.0 {
            clipped_theme.shadows = false;
            if let Some((radius, elevation)) = self.surface {
                let fallback = crate::elevation::Cache::default();
                let cache = self.menu.as_ref().map_or(&fallback, |state| &state.shadow);
                crate::elevation::draw_resizing(
                    renderer,
                    theme,
                    clip,
                    bounds.height,
                    radius,
                    elevation as f32,
                    layout.bounds(),
                    cache,
                );
            }
        }
        let draw = |renderer: &mut Renderer| {
            self.content.as_widget().draw(
                self.tree,
                renderer,
                &clipped_theme,
                style,
                layout.child(0),
                cursor,
                &clip,
            );
        };
        if progress >= 1.0 {
            draw(renderer);
        } else {
            renderer.with_layer(clip, draw);
        }
    }
    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.menu.is_none()
            || (self.menu.as_ref().is_some_and(|s| !s.ancestors.is_empty())
                && !cursor.is_over(layout.child(0).bounds()))
        {
            return mouse::Interaction::None;
        }
        if self.menu.as_ref().is_some_and(|state| !state.open) {
            return if cursor.is_over(layout.child(0).bounds()) {
                mouse::Interaction::Idle
            } else {
                mouse::Interaction::None
            };
        }
        let interaction = self.content.as_widget().mouse_interaction(
            self.tree,
            layout.child(0),
            cursor,
            &layout.bounds(),
            renderer,
        );
        if interaction == mouse::Interaction::None {
            mouse::Interaction::Idle
        } else {
            interaction
        }
    }
}

pub(crate) fn forward_shell<A, B>(target: &mut Shell<'_, A>, source: &Shell<'_, B>) {
    target.request_redraw_at(source.redraw_request());
    if source.is_event_captured() {
        target.capture_event();
    }
    if source.is_layout_invalid() {
        target.invalidate_layout();
    }
    if source.are_widgets_invalid() {
        target.invalidate_widgets();
    }
    target.request_input_method(source.input_method());
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn placement_flips_and_stays_inside_all_window_edges() {
        let viewport = Size::new(320.0, 280.0);
        let size = Size::new(200.0, 120.0);
        for anchor in [
            Rectangle::new(Point::ORIGIN, Size::new(40.0, 40.0)),
            Rectangle::new(Point::new(280.0, 240.0), Size::new(40.0, 40.0)),
        ] {
            for placement in [
                Placement::BottomStart,
                Placement::BottomEnd,
                Placement::Bottom,
                Placement::TopStart,
                Placement::TopEnd,
                Placement::Top,
                Placement::Left,
                Placement::Right,
            ] {
                let p = place(anchor, size, viewport, placement, 4.0);
                assert!(p.x >= 8.0 && p.y >= 8.0);
                assert!(p.x + size.width <= viewport.width - 8.0);
                assert!(p.y + size.height <= viewport.height - 8.0);
                if anchor.y == 240.0 && !placement.horizontal() {
                    assert!(p.y + size.height <= anchor.y - 4.0);
                }
            }
        }
        let tiny = place(
            Rectangle::default(),
            Size::ZERO,
            Size::new(1.0, 1.0),
            Placement::Bottom,
            4.0,
        );
        assert!(tiny.x.is_finite() && tiny.y.is_finite());
    }
}
