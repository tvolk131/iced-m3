use iced::advanced::{Layout, Renderer as _, Shell, layout, renderer::Headless, widget::Tree};
use iced::{Event, Point, Rectangle, Size, mouse, window};
use iced_material::{Element, Theme, checkbox};
use std::time::Duration;

struct Harness {
    element: Element<'static, bool>,
    tree: Tree,
    node: layout::Node,
    renderer: iced::Renderer,
}
impl Harness {
    fn new(checked: bool, mixed: bool, enabled: bool) -> Self {
        let renderer = iced::futures::executor::block_on(<iced::Renderer as Headless>::new(
            iced::Font::DEFAULT,
            16.0.into(),
            Some("tiny-skia"),
        ))
        .unwrap();
        let mut element = Self::element(checked, mixed, enabled);
        let mut tree = Tree::new(&element);
        let node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(200.0, 80.0)),
        );
        Self {
            element,
            tree,
            node,
            renderer,
        }
    }
    fn element(checked: bool, mixed: bool, enabled: bool) -> Element<'static, bool> {
        checkbox(checked)
            .indeterminate(mixed)
            .label("Workspace updates")
            .on_toggle_maybe(enabled.then_some(std::convert::identity))
            .into()
    }
    fn rebuild(&mut self, checked: bool, mixed: bool, enabled: bool) {
        self.element = Self::element(checked, mixed, enabled);
        self.tree.diff(&self.element);
        self.node = self.element.as_widget_mut().layout(
            &mut self.tree,
            &self.renderer,
            &layout::Limits::new(Size::ZERO, Size::new(200.0, 80.0)),
        );
    }
    fn event(&mut self, event: Event, point: Option<Point>) -> (Vec<bool>, window::RedrawRequest) {
        let mut messages = Vec::new();
        let mut shell = Shell::new(&mut messages);
        self.element.as_widget_mut().update(
            &mut self.tree,
            &event,
            Layout::new(&self.node),
            point.map_or(mouse::Cursor::Unavailable, mouse::Cursor::Available),
            &self.renderer,
            &mut iced::advanced::clipboard::Null,
            &mut shell,
            &Rectangle::with_size(Size::new(200.0, 80.0)),
        );
        let request = shell.redraw_request();
        (messages, request)
    }
    fn frame(&mut self) -> Vec<u8> {
        self.scrolled_frame(0.0)
    }
    fn scrolled_frame(&mut self, offset: f32) -> Vec<u8> {
        let viewport = Rectangle::with_size(Size::new(200.0, 80.0));
        self.renderer.reset(viewport);
        self.renderer
            .with_translation(iced::Vector::new(0.0, -offset), |renderer| {
                self.element.as_widget().draw(
                    &self.tree,
                    renderer,
                    &Theme::light(),
                    &Default::default(),
                    Layout::with_offset(iced::Vector::new(0.0, offset), &self.node),
                    mouse::Cursor::Unavailable,
                    &(viewport + iced::Vector::new(0.0, offset)),
                );
            });
        self.renderer
            .screenshot(Size::new(400, 160), 2.0, Theme::light().colors.surface)
    }
}

#[test]
fn checkbox_marks_follow_scroll_translation_at_retina_scale() {
    for (checked, mixed, enabled) in [
        (true, false, true),
        (false, true, true),
        (true, false, false),
    ] {
        let mut ui = Harness::new(checked, mixed, enabled);
        let at_origin = ui.frame();
        assert!(
            at_origin == ui.scrolled_frame(500.0),
            "scrolling must keep the mark inside its box"
        );
    }
}

fn press() -> Event {
    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
}
fn release() -> Event {
    Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
}
fn redraw(now: iced::time::Instant) -> Event {
    Event::Window(window::Event::RedrawRequested(now))
}

#[test]
fn checkbox_has_a_real_hover_halo_in_both_selection_states_but_not_when_disabled() {
    for checked in [false, true] {
        let mut ui = Harness::new(checked, false, true);
        let before = ui.frame();
        let point = Point::new(110.0, 24.0); // hover on the label, outside the box
        ui.event(
            Event::Mouse(mouse::Event::CursorMoved { position: point }),
            Some(point),
        );
        let (_, request) = ui.event(
            redraw(iced::time::Instant::now() + Duration::from_secs(1)),
            Some(point),
        );
        assert_eq!(
            request,
            window::RedrawRequest::Wait,
            "hover animation must settle"
        );
        let after = ui.frame();
        let pixel = |image: &[u8], x: usize, y: usize| {
            image[(y * 400 + x) * 4..(y * 400 + x) * 4 + 4].to_vec()
        };
        assert_ne!(
            pixel(&before, 48, 14),
            pixel(&after, 48, 14),
            "halo extends beyond 18px box"
        );
        assert_eq!(
            pixel(&before, 2, 2),
            pixel(&after, 2, 2),
            "halo stays circular and within target"
        );

        ui.rebuild(checked, false, false);
        let disabled = ui.frame();
        ui.event(
            Event::Mouse(mouse::Event::CursorMoved { position: point }),
            Some(point),
        );
        assert_eq!(
            disabled,
            ui.frame(),
            "disabled controls have no hover effect"
        );
    }
}

#[test]
fn checkbox_selection_and_mixed_mark_animate_across_rebuilds_and_then_stop() {
    let mut ui = Harness::new(false, false, true);
    let unchecked = ui.frame();
    let now = iced::time::Instant::now();
    ui.rebuild(true, false, true);
    assert_ne!(ui.event(redraw(now), None).1, window::RedrawRequest::Wait);
    assert_ne!(
        ui.event(redraw(now + Duration::from_millis(80)), None).1,
        window::RedrawRequest::Wait
    );
    let during = ui.frame();
    assert_eq!(
        ui.event(redraw(now + Duration::from_millis(400)), None).1,
        window::RedrawRequest::Wait
    );
    let checked = ui.frame();
    assert_ne!(unchecked, during);
    assert_ne!(
        during, checked,
        "selection must have an intermediate painted state"
    );
    ui.rebuild(false, true, true);
    ui.event(redraw(now + Duration::from_millis(500)), None);
    assert_eq!(
        ui.event(redraw(now + Duration::from_secs(1)), None).1,
        window::RedrawRequest::Wait
    );
    assert_ne!(checked, ui.frame(), "mixed state uses a different mark");
    ui.rebuild(false, false, true);
    ui.event(redraw(now + Duration::from_millis(1100)), None);
    assert_eq!(
        ui.event(redraw(now + Duration::from_secs(2)), None).1,
        window::RedrawRequest::Wait
    );
    assert_eq!(unchecked, ui.frame());
}

#[test]
fn checkbox_padded_target_and_label_activate_once_and_cancel_interrupted_gestures() {
    let mut ui = Harness::new(false, false, true);
    for point in [Point::new(3.0, 24.0), Point::new(110.0, 24.0)] {
        assert!(ui.event(press(), Some(point)).0.is_empty());
        assert_eq!(ui.event(release(), Some(point)).0, [true]);
        assert!(ui.event(release(), Some(point)).0.is_empty());
    }
    let point = Some(Point::new(24.0, 24.0));
    ui.event(press(), point);
    assert!(ui.event(release(), None).0.is_empty());
    ui.event(press(), point);
    ui.event(Event::Window(window::Event::Unfocused), point);
    assert!(ui.event(release(), point).0.is_empty());
    ui.event(press(), point); // works again after a modal's synthetic Unfocused
    assert_eq!(ui.event(release(), point).0, [true]);
    ui.event(press(), point);
    ui.rebuild(false, false, false);
    assert!(ui.event(release(), point).0.is_empty());
    assert!(ui.event(press(), point).0.is_empty());
}
