//! Semantic content timelines inside retained surfaces. Scopes are synchronous
//! widget updates, never application timers; each child retains its own reversal.
use crate::{Element, Theme, motion::Transition};
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, overlay, renderer,
    widget::{Operation, Tree, tree},
};
use iced::{Event, Length, Rectangle, Renderer, Size, Vector, mouse, window};
use std::{cell::Cell, rc::Rc, time::Duration};

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Kind {
    Dialog,
    Menu,
    Search,
    Popup,
}
#[derive(Clone, Copy)]
pub(crate) struct Phase {
    pub kind: Kind,
    pub open: bool,
    pub initial: f32,
    pub base: f32,
    pub enter: Duration,
    pub exit: Duration,
    pub above: bool,
}
thread_local! { static PHASE: Cell<Option<Phase>> = const { Cell::new(None) }; }
pub(crate) fn enter(phase: Phase) -> impl Drop {
    struct Restore(Option<Phase>);
    impl Drop for Restore {
        fn drop(&mut self) {
            PHASE.set(self.0);
        }
    }
    Restore(PHASE.replace(Some(phase)))
}
#[derive(Clone, Copy)]
pub(crate) enum Part {
    DialogActions,
    MenuRow(usize, usize),
    SearchHeader,
    SearchResults,
    SearchClear,
    Popup(bool),
}
impl Part {
    fn kind(self) -> Kind {
        match self {
            Self::DialogActions => Kind::Dialog,
            Self::MenuRow(..) => Kind::Menu,
            Self::SearchHeader | Self::SearchResults | Self::SearchClear => Kind::Search,
            Self::Popup(_) => Kind::Popup,
        }
    }
    fn timing(self, p: Phase) -> (Duration, Duration) {
        match self {
            Self::DialogActions => {
                if p.open {
                    (p.enter * 3 / 10, p.enter * 3 / 10)
                } else {
                    (Duration::ZERO, p.exit * 2 / 3)
                }
            }
            Self::MenuRow(index, count) => {
                let count = count.max(1);
                let reverse = if p.open { p.above } else { !p.above };
                let i = if reverse {
                    count - 1 - index.min(count - 1)
                } else {
                    index.min(count - 1)
                };
                if p.open {
                    ((p.enter / 2).mul_f32(i as f32 / count as f32), p.enter / 2)
                } else {
                    (
                        p.exit / 3 + (p.exit / 3).mul_f32(i as f32 / count as f32),
                        p.exit / 3,
                    )
                }
            }
            Self::SearchHeader => (Duration::ZERO, Duration::ZERO),
            Self::SearchResults => {
                if p.open {
                    (p.enter / 4, p.enter / 2)
                } else {
                    (Duration::ZERO, p.exit.mul_f32(83. / 250.))
                }
            }
            Self::SearchClear => {
                if p.open {
                    (p.enter * 5 / 6, p.enter / 6)
                } else {
                    (Duration::ZERO, p.exit.mul_f32(42. / 250.))
                }
            }
            Self::Popup(_) => {
                if p.open {
                    (p.enter / 10, p.enter / 2)
                } else {
                    (Duration::ZERO, p.exit * 2 / 3)
                }
            }
        }
    }
}
pub(crate) fn part<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    part: Part,
) -> Element<'a, Message> {
    Element::new(Staged {
        content: content.into(),
        part,
        full_header: None,
    })
}
pub(crate) fn header<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    full: Rc<Cell<bool>>,
) -> Element<'a, Message> {
    Element::new(Staged {
        content: content.into(),
        part: Part::SearchHeader,
        full_header: Some(full),
    })
}
struct Staged<'a, Message> {
    content: Element<'a, Message>,
    part: Part,
    full_header: Option<Rc<Cell<bool>>>,
}
struct State {
    opacity: Transition,
    seen: bool,
    base: f32,
}
impl<Message> Widget<Message, Theme, Renderer> for Staged<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State {
            opacity: Transition::linear(1.),
            seen: false,
            base: 1.,
        })
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
    fn layout(&mut self, t: &mut Tree, r: &Renderer, l: &layout::Limits) -> layout::Node {
        let extra = if self.full_header.as_ref().is_some_and(|full| full.get()) {
            16.
        } else {
            0.
        };
        let child = self.content.as_widget_mut().layout(
            &mut t.children[0],
            r,
            &layout::Limits::new(
                Size::new(l.min().width, (l.min().height - extra).max(0.)),
                Size::new(l.max().width, (l.max().height - extra).max(0.)),
            ),
        );
        let size = Size::new(child.size().width, child.size().height + extra);
        layout::Node::with_children(size, vec![child.move_to((0., extra / 2.))])
    }
    fn operate(&mut self, t: &mut Tree, l: Layout<'_>, r: &Renderer, o: &mut dyn Operation) {
        self.content
            .as_widget_mut()
            .operate(&mut t.children[0], l.child(0), r, o);
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
        let state = t.state.downcast_mut::<State>();
        // The search editor travels with the surface; only results and clear
        // action fade. Keeping the editor opaque also preserves query continuity.
        if let Some(phase) = PHASE
            .get()
            .filter(|p| p.kind == self.part.kind() && !matches!(self.part, Part::SearchHeader))
        {
            if !state.seen {
                state.opacity = Transition::linear(phase.initial);
                state.seen = true;
            }
            state.base = phase.base;
            let now = match e {
                Event::Window(window::Event::RedrawRequested(now)) => *now,
                _ => crate::motion::now(),
            };
            let (delay, duration) = self.part.timing(phase);
            let changed = state
                .opacity
                .set_delayed(f32::from(phase.open), now, delay, duration);
            if state.opacity.tick(now) || changed {
                s.request_redraw();
            }
        }
        self.content
            .as_widget_mut()
            .update(&mut t.children[0], e, l.child(0), c, r, cb, s, v);
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
        let state = t.state.downcast_ref::<State>();
        // Dialog actions are nested inside the host's existing body fade. Apply
        // only the additional opacity so the result is not multiplied twice.
        let alpha = if state.base > 0. {
            (state.opacity.value / state.base).clamp(0., 1.)
        } else {
            0.
        };
        if alpha <= 0. {
            return;
        }
        self.content
            .as_widget()
            .draw(&t.children[0], r, theme, style, l.child(0), c, v);
        if alpha < 1.
            && let Some(clip) = l.bounds().intersection(v)
        {
            let surface = match self.part.kind() {
                Kind::Dialog | Kind::Search => theme.colors.surface_container_high,
                _ if matches!(self.part, Part::Popup(true)) => theme.colors.surface_container_high,
                _ => theme.colors.surface_container,
            };
            r.with_layer(clip, |r| {
                r.fill_quad(
                    renderer::Quad {
                        bounds: clip,
                        ..Default::default()
                    },
                    crate::theme::alpha(surface, 1. - alpha),
                )
            });
        }
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
            .mouse_interaction(&t.children[0], l.child(0), c, v, r)
    }
    fn overlay<'b>(
        &'b mut self,
        t: &'b mut Tree,
        l: Layout<'b>,
        r: &Renderer,
        v: &Rectangle,
        tr: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(&mut t.children[0], l.child(0), r, v, tr)
    }
}
