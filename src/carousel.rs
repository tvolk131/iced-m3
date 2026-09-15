//! Controlled horizontal carousels with finite snapping and bounded drag gestures.
use crate::{Element, Theme, motion::Transition, tokens};
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, overlay, renderer,
    widget::{Operation, Tree, tree},
};
use iced::{Event, Length, Point, Rectangle, Renderer, Size, Vector, keyboard, mouse, window};
use std::{
    cell::{Cell, RefCell},
    time::{Duration, Instant},
};
mod geometry;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CarouselVariant {
    #[default]
    MultiBrowse,
    Hero,
    /// One or more large focal items between small previews on both sides.
    HeroCenter,
    Uncontained,
    FullScreen,
}
pub struct Carousel<'a, Message> {
    items: RefCell<Vec<Element<'a, Message>>>,
    count: usize,
    factory: Option<Box<dyn Fn(usize) -> Element<'a, Message> + 'a>>,
    selected: usize,
    on_select: Option<Box<dyn Fn(usize) -> Message + 'a>>,
    variant: CarouselVariant,
    item_width: f32,
    height: f32,
    spacing: f32,
    background: Option<iced::Color>,
}
pub fn carousel<'a, Message>(
    items: impl IntoIterator<Item = Element<'a, Message>>,
    selected: usize,
) -> Carousel<'a, Message> {
    let items: Vec<_> = items.into_iter().collect();
    Carousel {
        count: items.len(),
        items: RefCell::new(items),
        factory: None,
        selected,
        on_select: None,
        variant: CarouselVariant::MultiBrowse,
        item_width: 280.0,
        height: 200.0,
        spacing: 8.0,
        background: None,
    }
}
/// Build only the visible items and a small overscan region. Keep durable item
/// values in application state; widget-local state is released when an item leaves
/// that region. Indices must retain their meaning across view rebuilds.
pub fn lazy_carousel<'a, Message>(
    count: usize,
    selected: usize,
    build: impl Fn(usize) -> Element<'a, Message> + 'a,
) -> Carousel<'a, Message> {
    let mut carousel = carousel(std::iter::empty(), selected);
    carousel.count = count;
    carousel.factory = Some(Box::new(build));
    carousel
}
impl<'a, Message> Carousel<'a, Message> {
    /// Match the surrounding surface used outside rounded image masks.
    pub fn background(mut self, color: iced::Color) -> Self {
        self.background = Some(color);
        self
    }
    pub fn on_select(mut self, f: impl Fn(usize) -> Message + 'a) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }
    pub fn variant(mut self, variant: CarouselVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn item_width(mut self, width: f32) -> Self {
        if width.is_finite() {
            self.item_width = width.max(40.0);
        }
        self
    }
    pub fn height(mut self, height: f32) -> Self {
        if height.is_finite() {
            self.height = height.max(40.0);
        }
        self
    }
    fn last(&self) -> usize {
        self.count.saturating_sub(1)
    }
    fn plan(&self, width: f32) -> geometry::Plan {
        geometry::Plan::new(
            width,
            self.height,
            self.item_width,
            self.spacing,
            self.count,
            self.variant,
        )
    }
    fn mount(&self, tree: &mut Tree, range: std::ops::Range<usize>) {
        let Some(build) = &self.factory else {
            return;
        };
        let first = tree.state.downcast_ref::<State>().first;
        let mut previous: std::collections::HashMap<_, _> = std::mem::take(&mut tree.children)
            .into_iter()
            .enumerate()
            .map(|(i, t)| (first + i, t))
            .collect();
        let mut items = Vec::with_capacity(range.len());
        for i in range.clone() {
            let item = build(i);
            let mut child = previous.remove(&i).unwrap_or_else(|| Tree::new(&item));
            child.diff(&item);
            tree.children.push(child);
            items.push(item);
        }
        tree.state.downcast_mut::<State>().first = range.start;
        self.items.replace(items);
    }
}
struct Gesture {
    start: Point,
    index: f32,
    last: Point,
    sampled: Instant,
    velocity: f32,
}

struct State {
    position: Transition,
    focus: crate::focus::Focus,
    gesture: Option<Gesture>,
    first: usize,
    dragging: bool,
    wheel: f32,
    motion: Cell<tokens::Motion>,
}
impl<Message> Widget<Message, Theme, Renderer> for Carousel<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State {
            position: Transition::standard(self.selected.min(self.last()) as f32),
            focus: Default::default(),
            gesture: None,
            first: 0,
            dragging: false,
            wheel: 0.0,
            motion: Cell::new(tokens::Motion::default()),
        })
    }
    fn children(&self) -> Vec<Tree> {
        self.items.borrow().iter().map(Tree::new).collect()
    }
    fn diff(&self, t: &mut Tree) {
        if self.factory.is_some() {
            let first = t.state.downcast_ref::<State>().first.min(self.count);
            self.mount(t, first..(first + t.children.len()).min(self.count));
        } else {
            t.state.downcast_mut::<State>().first = 0;
            t.diff_children(&self.items.borrow());
        }
        if self.on_select.is_none() {
            let state = t.state.downcast_mut::<State>();
            state.gesture = None;
            state.dragging = false;
            state.focus = Default::default();
        }
    }
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fixed(self.height))
    }
    fn layout(&mut self, t: &mut Tree, r: &Renderer, l: &layout::Limits) -> layout::Node {
        let size = l.resolve(
            Length::Fill,
            Length::Fixed(self.height),
            Size::new(320.0, self.height),
        );
        let plan = self.plan(size.width);
        let position = t.state.downcast_ref::<State>().position.value;
        if self.factory.is_some() {
            let range = plan.visible_range(position, self.count);
            if t.state.downcast_ref::<State>().first != range.start
                || self.items.borrow().len() != range.len()
            {
                self.mount(t, range);
            }
        }
        let first = t.state.downcast_ref::<State>().first;
        let geometry = plan.geometry(position, first..first + self.items.borrow().len());
        let full_width = plan.pitch - self.spacing;
        let children = self
            .items
            .get_mut()
            .iter_mut()
            .enumerate()
            .map(|(i, item)| {
                let (x, width) = geometry[i];
                let bounds = Size::new(full_width, size.height);
                let content = item
                    .as_widget_mut()
                    .layout(&mut t.children[i], r, &layout::Limits::new(bounds, bounds))
                    .move_to(Point::new((width - full_width) / 2.0, 0.0));
                layout::Node::with_children(Size::new(width, size.height), vec![content])
                    .move_to(Point::new(x, 0.0))
            })
            .collect();
        layout::Node::with_children(size, children)
    }
    fn operate(&mut self, t: &mut Tree, l: Layout<'_>, r: &Renderer, o: &mut dyn Operation) {
        if self.on_select.is_some() && self.count > 0 {
            let state = t.state.downcast_mut::<State>();
            let id = state.focus.id.clone();
            o.focusable(Some(&id), l.bounds(), &mut state.focus);
        }
        o.container(None, l.bounds());
        o.traverse(&mut |o| {
            for (i, item) in self.items.get_mut().iter_mut().enumerate() {
                if l.child(i).bounds().intersection(&l.bounds()).is_some() {
                    item.as_widget_mut()
                        .operate(&mut t.children[i], l.child(i).child(0), r, o);
                }
            }
        });
    }
    fn update(
        &mut self,
        t: &mut Tree,
        event: &Event,
        l: Layout<'_>,
        cursor: mouse::Cursor,
        r: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let plan = self.plan(l.bounds().width);
        let state = t.state.downcast_mut::<State>();
        let now = match event {
            Event::Window(window::Event::RedrawRequested(now)) => *now,
            _ => crate::motion::now(),
        };
        let mut consume = false;
        let mut cancel = false;
        let mut proposal = None;
        let over = cursor.is_over(l.bounds()) && cursor.is_over(*viewport);
        if self.on_select.is_some() && self.count > 0 {
            match event {
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(key),
                    ..
                }) if state.focus.focused => {
                    use keyboard::key::Named;
                    proposal = match key {
                        Named::ArrowLeft => Some(self.selected.saturating_sub(1)),
                        Named::ArrowRight => Some(self.selected.saturating_add(1).min(self.last())),
                        Named::Home => Some(0),
                        Named::End => Some(self.last()),
                        _ => None,
                    };
                    consume = proposal.is_some();
                }
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if over => {
                    state.gesture = cursor.position().map(|p| Gesture {
                        start: p,
                        index: state.position.value.min(plan.max),
                        last: p,
                        sampled: now,
                        velocity: 0.,
                    });
                    state.focus.focused = false;
                    state.focus.visible = false;
                }
                Event::Mouse(mouse::Event::CursorMoved { position }) if state.gesture.is_some() => {
                    let gesture = state.gesture.as_mut().unwrap();
                    let dx = position.x - gesture.start.x;
                    let dt = now.saturating_duration_since(gesture.sampled);
                    if !dt.is_zero() {
                        gesture.velocity = if dt <= Duration::from_millis(100) {
                            -(position.x - gesture.last.x) / plan.pitch / dt.as_secs_f32()
                        } else {
                            0.
                        };
                    }
                    gesture.last = *position;
                    gesture.sampled = now;
                    if dx.abs() > 8.0 || state.dragging {
                        cancel = !state.dragging;
                        state.dragging = true;
                        let step = plan.pitch;
                        state.position = Transition::standard(
                            (gesture.index - dx / step.max(1.0)).clamp(0.0, plan.max),
                        );
                        shell.invalidate_layout();
                        consume = true;
                    }
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                    if state.gesture.is_some() =>
                {
                    if state.dragging {
                        let gesture = state.gesture.as_ref().unwrap();
                        let velocity = if now.saturating_duration_since(gesture.sampled)
                            <= Duration::from_millis(100)
                        {
                            gesture.velocity
                        } else {
                            0.
                        };
                        let coast = if velocity.abs() >= 0.5 {
                            (velocity * 0.18).clamp(-3., 3.)
                        } else {
                            0.
                        };
                        let target = (state.position.value + coast).clamp(0., plan.max);
                        proposal = Some(
                            if plan.max > 0. && (target >= plan.max || target.round() >= plan.max) {
                                self.last()
                            } else {
                                target.round() as usize
                            },
                        );
                        consume = true;
                    }
                    state.gesture = None;
                    state.dragging = false;
                }
                Event::Mouse(mouse::Event::WheelScrolled { delta }) if over => {
                    let x = match delta {
                        mouse::ScrollDelta::Lines { x, .. } => x * 40.0,
                        mouse::ScrollDelta::Pixels { x, .. } => *x,
                    };
                    if x != 0.0 {
                        state.wheel += x;
                        consume = true;
                        if state.wheel.abs() >= 40.0 {
                            proposal = Some(if state.wheel > 0.0 {
                                self.selected.saturating_sub(1)
                            } else {
                                self.selected.saturating_add(1).min(self.last())
                            });
                            state.wheel = 0.0;
                        }
                    }
                }
                Event::Window(window::Event::Unfocused)
                | Event::Mouse(mouse::Event::CursorLeft) => {
                    cancel = state.gesture.is_some();
                    state.gesture = None;
                    state.dragging = false;
                }
                _ => {}
            }
        }
        if let Some(index) = proposal {
            shell.publish(self.on_select.as_ref().unwrap()(index.min(self.last())));
        }
        if !state.dragging {
            let before = state.position.value;
            let changed = state.position.set(
                (proposal.unwrap_or(self.selected).min(self.last()) as f32).min(plan.max),
                now,
                state.motion.get().medium,
            );
            if changed || state.position.tick(now) {
                shell.request_redraw();
            }
            if state.position.value != before {
                shell.invalidate_layout();
            }
        }
        if cancel {
            let mut dropped = Vec::new();
            let mut local = Shell::new(&mut dropped);
            crate::activity::interaction(|| {
                for (i, item) in self.items.get_mut().iter_mut().enumerate() {
                    item.as_widget_mut().update(
                        &mut t.children[i],
                        &Event::Window(window::Event::Unfocused),
                        l.child(i).child(0),
                        mouse::Cursor::Unavailable,
                        r,
                        clipboard,
                        &mut local,
                        viewport,
                    );
                }
            });
        }
        if consume {
            shell.capture_event();
            shell.request_redraw();
            return;
        }
        let clip = l
            .bounds()
            .intersection(viewport)
            .unwrap_or(Rectangle::new(Point::ORIGIN, Size::ZERO));
        for (i, item) in self.items.get_mut().iter_mut().enumerate() {
            item.as_widget_mut().update(
                &mut t.children[i],
                event,
                l.child(i).child(0),
                cursor,
                r,
                clipboard,
                shell,
                &l.child(i).bounds().intersection(&clip).unwrap_or_default(),
            );
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
        let Some(clip) = l.bounds().intersection(v) else {
            return;
        };
        let state = t.state.downcast_ref::<State>();
        state.motion.set(theme.motion);
        r.with_layer(clip, |r| {
            for (i, item) in self.items.borrow().iter().enumerate() {
                if let Some(item_clip) = l.child(i).bounds().intersection(&clip) {
                    r.with_layer(item_clip, |r| {
                        item.as_widget().draw(
                            &t.children[i],
                            r,
                            theme,
                            style,
                            l.child(i).child(0),
                            c,
                            &item_clip,
                        )
                    });
                    rounded_mask(
                        r,
                        l.child(i).bounds(),
                        item_clip,
                        self.background.unwrap_or(theme.colors.surface),
                    );
                }
            }

            if state.focus.focused && state.focus.visible {
                r.fill_quad(
                    renderer::Quad {
                        bounds: l.bounds().shrink(1.0),
                        border: iced::Border {
                            color: theme.colors.primary,
                            width: 2.0,
                            radius: 12.0.into(),
                        },
                        ..Default::default()
                    },
                    iced::Color::TRANSPARENT,
                );
            }
        });
    }
    fn mouse_interaction(
        &self,
        t: &Tree,
        l: Layout<'_>,
        c: mouse::Cursor,
        v: &Rectangle,
        r: &Renderer,
    ) -> mouse::Interaction {
        if t.state.downcast_ref::<State>().dragging {
            return mouse::Interaction::Grabbing;
        }
        self.items
            .borrow()
            .iter()
            .enumerate()
            .map(|(i, item)| {
                item.as_widget().mouse_interaction(
                    &t.children[i],
                    l.child(i).child(0),
                    c,
                    &l.child(i)
                        .bounds()
                        .intersection(&l.bounds())
                        .and_then(|bounds| bounds.intersection(v))
                        .unwrap_or_default(),
                    r,
                )
            })
            .max()
            .unwrap_or_default()
    }
    fn overlay<'b>(
        &'b mut self,
        t: &'b mut Tree,
        l: Layout<'b>,
        r: &Renderer,
        v: &Rectangle,
        tr: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let overlays = self
            .items
            .get_mut()
            .iter_mut()
            .zip(t.children.iter_mut())
            .enumerate()
            .filter_map(|(i, (item, tree))| {
                let clip = l
                    .child(i)
                    .bounds()
                    .intersection(&l.bounds())?
                    .intersection(v)?;
                item.as_widget_mut()
                    .overlay(tree, l.child(i).child(0), r, &clip, tr)
            })
            .collect::<Vec<_>>();
        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
    }
}
impl<'a, Message: 'a> From<Carousel<'a, Message>> for Element<'a, Message> {
    fn from(value: Carousel<'a, Message>) -> Self {
        Element::new(value)
    }
}

fn rounded_mask(renderer: &mut Renderer, bounds: Rectangle, clip: Rectangle, color: iced::Color) {
    use iced::advanced::Renderer as _;
    use iced::advanced::svg::{Handle, Renderer as _, Svg};
    static CORNERS: std::sync::OnceLock<[Handle; 4]> = std::sync::OnceLock::new();
    let handles=CORNERS.get_or_init(||["M0 0H1A1 1 0 0 0 0 1Z","M0 0H1V1A1 1 0 0 0 0 0Z","M1 0V1H0A1 1 0 0 0 1 0Z","M1 1H0V0A1 1 0 0 0 1 1Z"].map(|p|Handle::from_memory(format!("<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 1 1'><path d='{p}'/></svg>").into_bytes())));
    let radius = 28.0_f32.min(bounds.width / 2.0).min(bounds.height / 2.0);
    renderer.with_layer(clip, |r| {
        for (i, (x, y)) in [
            (bounds.x, bounds.y),
            (bounds.x + bounds.width - radius, bounds.y),
            (
                bounds.x + bounds.width - radius,
                bounds.y + bounds.height - radius,
            ),
            (bounds.x, bounds.y + bounds.height - radius),
        ]
        .into_iter()
        .enumerate()
        {
            r.draw_svg(
                Svg::from(&handles[i]).color(color),
                Rectangle::new(Point::new(x, y), Size::new(radius, radius)),
                clip,
            );
        }
    });
}
