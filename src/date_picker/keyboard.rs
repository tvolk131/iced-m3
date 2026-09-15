//! Calendar navigation keeps focus separate from the application-owned selection.
use super::Date;
use crate::{Element, Theme, focus};
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, overlay, renderer,
    widget::{Operation, Tree, tree},
};
use iced::{Event, Length, Rectangle, Renderer, Size, Vector, keyboard, mouse};

pub(super) struct Calendar<'a, Message> {
    pub content: Element<'a, Message>,
    pub dates: Vec<Date>,
    pub month: Date,
    pub min: Date,
    pub max: Date,
    pub enabled: Option<Box<dyn Fn(Date) -> bool + 'a>>,
    pub on_month: Option<Box<dyn Fn(Date) -> Message + 'a>>,
}
#[derive(Default)]
struct State {
    pending: Option<Date>,
}
impl<Message> Widget<Message, Theme, Renderer> for Calendar<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, t: &mut Tree) {
        t.diff_children(std::slice::from_ref(&self.content));
    }
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }
    fn layout(&mut self, t: &mut Tree, r: &Renderer, limits: &layout::Limits) -> layout::Node {
        let node = self
            .content
            .as_widget_mut()
            .layout(&mut t.children[0], r, limits);
        let state = t.state.downcast_mut::<State>();
        if let Some(date) = state.pending
            && date.first_of_month() == self.month
        {
            state.pending = None;
            if let Some(index) = self.dates.iter().position(|d| *d == date) {
                focus::select_all(
                    &mut self.content,
                    &mut t.children[0],
                    Layout::new(&node),
                    r,
                    index,
                );
            }
        }
        node
    }
    fn operate(&mut self, t: &mut Tree, l: Layout<'_>, r: &Renderer, o: &mut dyn Operation) {
        self.content
            .as_widget_mut()
            .operate(&mut t.children[0], l, r, o);
    }
    fn update(
        &mut self,
        t: &mut Tree,
        e: &Event,
        l: Layout<'_>,
        c: mouse::Cursor,
        r: &Renderer,
        cb: &mut dyn Clipboard,
        s: &mut Shell<'_, Message>,
        v: &Rectangle,
    ) {
        if let Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(key),
            modifiers,
            ..
        }) = e
            && !modifiers.control()
            && !modifiers.alt()
            && !modifiers.logo()
            && let Some(index) = focus::focused_index(&mut self.content, &mut t.children[0], l, r)
            && let Some(&date) = self.dates.get(index)
        {
            use keyboard::key::Named;
            let target = match key {
                Named::ArrowLeft if !modifiers.shift() => date.add_days(-1),
                Named::ArrowRight if !modifiers.shift() => date.add_days(1),
                Named::ArrowUp if !modifiers.shift() => date.add_days(-7),
                Named::ArrowDown if !modifiers.shift() => date.add_days(7),
                Named::PageUp => date.add_months(if modifiers.shift() { -12 } else { -1 }),
                Named::PageDown => date.add_months(if modifiers.shift() { 12 } else { 1 }),
                _ => {
                    self.content
                        .as_widget_mut()
                        .update(&mut t.children[0], e, l, c, r, cb, s, v);
                    return;
                }
            };
            if self.min <= self.max
                && let Some(target) = target
            {
                let target = target.clamp(self.min, self.max);
                // Search only the destination month. An entirely disabled month
                // does not initiate a potentially unbounded scan through years.
                let candidate = (1..=target.days_in_month())
                    .filter_map(|day| Date::new(target.year(), target.month(), day).ok())
                    .filter(|d| {
                        *d >= self.min
                            && *d <= self.max
                            && if target > date {
                                *d > date
                            } else if target < date {
                                *d < date
                            } else {
                                true
                            }
                            && self.enabled.as_ref().is_none_or(|f| f(*d))
                    })
                    .min_by_key(|d| {
                        (
                            d.day().abs_diff(target.day()),
                            if target >= date {
                                32 - d.day()
                            } else {
                                d.day()
                            },
                        )
                    });
                if let Some(target) = candidate {
                    if target.first_of_month() == self.month {
                        if let Some(index) = self.dates.iter().position(|d| *d == target) {
                            focus::select_all(&mut self.content, &mut t.children[0], l, r, index);
                        }
                    } else if let Some(on_month) = &self.on_month {
                        t.state.downcast_mut::<State>().pending = Some(target);
                        s.publish(on_month(target.first_of_month()));
                    }
                    s.request_redraw();
                }
            }
            s.capture_event();
            return;
        }
        self.content
            .as_widget_mut()
            .update(&mut t.children[0], e, l, c, r, cb, s, v);
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
        self.content
            .as_widget()
            .draw(&t.children[0], r, theme, style, l, c, v);
    }
    fn mouse_interaction(
        &self,
        t: &Tree,
        l: Layout<'_>,
        c: mouse::Cursor,
        v: &Rectangle,
        r: &Renderer,
    ) -> mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(&t.children[0], l, c, v, r)
    }
    fn overlay<'b>(
        &'b mut self,
        t: &'b mut Tree,
        l: Layout<'b>,
        r: &Renderer,
        v: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(&mut t.children[0], l, r, v, translation)
    }
}
