use iced::{
    Event, Length, Point, Size, keyboard, mouse,
    widget::{column, container},
};
use iced_material::{
    Element, Theme, button,
    dialog::{dialog, modal},
    text_field,
};
use iced_test::{Simulator, simulator};

#[derive(Clone, Debug, PartialEq)]
enum Message {
    Action,
    Dismiss,
    Input(String),
    Submit,
    Toggle(bool),
}

#[test]
fn selection_controls_emit_new_application_values_and_respect_disabled() {
    use iced_material::{TypeScale, checkbox, chip, icon_button, switch, typography};
    let mut ui = Simulator::new(
        column![
            checkbox(false).label("Check me").on_toggle(Message::Toggle),
            checkbox(true).label("Disabled checkbox"),
            switch(true).label("Switch me").on_toggle(Message::Toggle),
            switch(false).label("Disabled switch"),
            chip("Filter", false).on_press(Message::Action),
            icon_button(typography("+", TypeScale::Title)).on_press(Message::Submit),
        ]
        .spacing(16),
    );
    for label in [
        "Check me",
        "Disabled checkbox",
        "Switch me",
        "Disabled switch",
        "Filter",
        "+",
    ] {
        ui.click(label).unwrap();
    }
    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        [
            Message::Toggle(true),
            Message::Toggle(false),
            Message::Action,
            Message::Submit
        ]
    );
}

#[test]
fn native_focus_and_selection_survive_field_rebuilds() {
    use iced::advanced::{Layout, Shell, layout::Limits, widget::Tree};
    type Paragraph = <iced::Renderer as iced::advanced::text::Renderer>::Paragraph;
    type InputState = iced::widget::text_input::State<Paragraph>;
    let mut field: Element<'_, Message> = text_field("Name", "Ada").on_input(Message::Input).into();
    let mut tree = Tree::new(&field);
    let renderer = renderer();
    let limits = Limits::new(Size::ZERO, Size::new(320.0, 100.0));
    let node = field.as_widget_mut().layout(&mut tree, &renderer, &limits);
    let layout = Layout::new(&node);
    let mut messages = Vec::new();
    for event in simulator::click() {
        field.as_widget_mut().update(
            &mut tree,
            &event,
            layout,
            mouse::Cursor::Available(Point::new(40.0, 36.0)),
            &renderer,
            &mut iced::advanced::clipboard::Null,
            &mut Shell::new(&mut messages),
            &layout.bounds(),
        );
    }
    let native = tree.children[0].state.downcast_mut::<InputState>();
    assert!(native.is_focused());
    native.select_all();
    let selection = native.cursor();
    let rebuilt: Element<'_, Message> = text_field("Name", "Ada")
        .on_input(Message::Input)
        .error("Demo validation")
        .into();
    tree.diff(&rebuilt);
    let native = tree.children[0].state.downcast_ref::<InputState>();
    assert!(native.is_focused());
    assert_eq!(native.cursor(), selection);
    let disabled: Element<'_, Message> = text_field("Name", "Ada").into();
    tree.diff(&disabled);
    assert!(
        !tree.children[0]
            .state
            .downcast_ref::<InputState>()
            .is_focused()
    );
}

#[test]
fn empty_field_floating_label_settles_after_focus_leaves() {
    use iced::advanced::{Layout, Shell, layout::Limits, widget::Tree};
    let mut field: Element<'_, Message> = text_field("Name", "").on_input(Message::Input).into();
    let mut tree = Tree::new(&field);
    let renderer = renderer();
    let node = field.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &Limits::new(Size::ZERO, Size::new(320.0, 100.0)),
    );
    let layout = Layout::new(&node);
    let now = iced::time::Instant::now();
    let mut send = |event: Event, cursor: mouse::Cursor| {
        let mut messages = Vec::new();
        let mut shell = Shell::new(&mut messages);
        field.as_widget_mut().update(
            &mut tree,
            &event,
            layout,
            cursor,
            &renderer,
            &mut iced::advanced::clipboard::Null,
            &mut shell,
            &layout.bounds(),
        );
        shell.redraw_request()
    };
    send(
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        mouse::Cursor::Available(Point::new(40.0, 36.0)),
    );
    send(
        Event::Window(iced::window::Event::RedrawRequested(
            now + std::time::Duration::from_secs(1),
        )),
        mouse::Cursor::Unavailable,
    );
    send(
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        mouse::Cursor::Available(Point::new(500.0, 500.0)),
    );
    assert_eq!(
        send(
            Event::Window(iced::window::Event::RedrawRequested(
                now + std::time::Duration::from_secs(2)
            )),
            mouse::Cursor::Unavailable
        ),
        iced::window::RedrawRequest::Wait
    );
}

#[test]
fn pressed_button_rendering_is_clipped_to_viewport_and_rounded_shape() {
    use iced::advanced::{
        Layout, Renderer as _, Shell, layout::Limits, renderer::Headless, widget::Tree,
    };
    let mut element: Element<'_, Message> =
        container(button("Save").width(120).on_press(Message::Action))
            .padding(20)
            .into();
    let mut tree = Tree::new(&element);
    let mut renderer = renderer();
    let node = element.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &Limits::new(Size::ZERO, Size::new(200.0, 100.0)),
    );
    let layout = Layout::new(&node);
    let viewport = iced::Rectangle::new(Point::ORIGIN, Size::new(80.0, 100.0));
    let mut messages = Vec::new();
    element.as_widget_mut().update(
        &mut tree,
        &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        layout,
        mouse::Cursor::Available(Point::new(45.0, 40.0)),
        &renderer,
        &mut iced::advanced::clipboard::Null,
        &mut Shell::new(&mut messages),
        &viewport,
    );
    renderer.reset(iced::Rectangle::new(Point::ORIGIN, Size::new(200.0, 100.0)));
    element.as_widget().draw(
        &tree,
        &mut renderer,
        &Theme::light(),
        &Default::default(),
        layout,
        mouse::Cursor::Unavailable,
        &viewport,
    );
    let rgba = renderer.screenshot(Size::new(200, 100), 1.0, Theme::light().colors.surface);
    let pixel = |x: usize, y: usize| &rgba[(y * 200 + x) * 4..(y * 200 + x) * 4 + 4];
    assert_eq!(
        pixel(20, 20),
        pixel(0, 0),
        "rounded corner remains background"
    );
    assert_eq!(
        pixel(90, 40),
        pixel(0, 0),
        "press effect cannot escape the viewport"
    );
    assert_ne!(pixel(40, 40), pixel(0, 0), "button actually rendered");
}

#[test]
fn oversized_dialog_keeps_actions_reachable_by_scrolling() {
    use iced::widget::text;
    let dialog = dialog(
        column![
            text("Long dialog"),
            container(text("Long body")).height(1000),
            button("Bottom action").on_press(Message::Submit)
        ]
        .spacing(24),
    )
    .on_dismiss(Message::Dismiss);
    let mut ui = Simulator::with_size(
        Default::default(),
        Size::new(320.0, 480.0),
        modal(
            container(text("Background"))
                .width(Length::Fill)
                .height(Length::Fill),
            Some(dialog),
        ),
    );
    ui.point_at((150.0, 240.0));
    ui.simulate([Event::Mouse(mouse::Event::WheelScrolled {
        delta: mouse::ScrollDelta::Lines { x: 0.0, y: -100.0 },
    })]);
    ui.click("Bottom action").unwrap();
    assert_eq!(ui.into_messages().collect::<Vec<_>>(), [Message::Submit]);
}

#[test]
fn button_emits_once_on_release_and_disabled_does_nothing() {
    let mut ui = Simulator::new(column![
        button("Enabled").on_press(Message::Action),
        button("Disabled")
    ]);
    ui.click("Enabled").unwrap();
    ui.click("Disabled").unwrap();
    assert_eq!(ui.into_messages().collect::<Vec<_>>(), [Message::Action]);
}

#[test]
fn release_outside_cancels_and_release_without_press_is_ignored() {
    let mut ui = Simulator::new(button("Save").on_press(Message::Action));
    ui.point_at((30.0, 20.0));
    ui.simulate([Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    ))]);
    ui.point_at((900.0, 700.0));
    ui.simulate([Event::Mouse(mouse::Event::ButtonReleased(
        mouse::Button::Left,
    ))]);
    ui.point_at((30.0, 20.0));
    ui.simulate([Event::Mouse(mouse::Event::ButtonReleased(
        mouse::Button::Left,
    ))]);
    assert_eq!(ui.into_messages().count(), 0);
}

fn modal_view(outside: bool, escape: bool) -> Element<'static, Message> {
    modal(
        container(button("Background").on_press(Message::Action))
            .width(Length::Fill)
            .height(Length::Fill),
        Some(
            dialog(column![
                button("Dialog action").on_press(Message::Submit),
                text_field("Input", "").on_input(Message::Input)
            ])
            .on_dismiss(Message::Dismiss)
            .dismiss_on_outside(outside)
            .dismiss_on_escape(escape),
        ),
    )
}

#[test]
fn dialog_blocks_background_mouse_wheel_and_keyboard() {
    let mut ui = Simulator::new(modal_view(false, false));
    ui.point_at((30.0, 20.0));
    let statuses = ui.simulate(simulator::click().chain([Event::Mouse(
        mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: -2.0 },
        },
    )]));
    assert!(statuses.iter().all(|s| *s == iced::event::Status::Captured));
    assert_eq!(
        ui.tap_key(keyboard::key::Named::Escape),
        iced::event::Status::Captured
    );
    assert_eq!(ui.typewrite("blocked"), iced::event::Status::Captured);
    assert_eq!(ui.into_messages().count(), 0);
}

#[test]
fn dialog_actions_escape_and_configured_scrim_dismissal() {
    let mut ui = Simulator::new(modal_view(true, true));
    ui.click("Dialog action").unwrap();
    ui.tap_key(keyboard::key::Named::Escape);
    ui.point_at((10.0, 10.0));
    ui.simulate(simulator::click());
    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        [Message::Submit, Message::Dismiss, Message::Dismiss]
    );
}

#[test]
fn a_gesture_that_starts_inside_does_not_dismiss_on_outside_release() {
    let mut ui = Simulator::new(modal_view(true, true));
    ui.point_at((512.0, 384.0));
    ui.simulate([Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    ))]);
    ui.point_at((10.0, 10.0));
    ui.simulate([Event::Mouse(mouse::Event::ButtonReleased(
        mouse::Button::Left,
    ))]);
    assert!(
        !ui.into_messages()
            .any(|message| message == Message::Dismiss)
    );
}

#[test]
fn native_text_editing_unicode_backspace_submit_and_disabled() {
    let mut ui = Simulator::new(
        text_field("Name", "")
            .on_input(Message::Input)
            .on_submit(Message::Submit),
    );
    ui.point_at((40.0, 36.0));
    ui.simulate(simulator::click());
    ui.typewrite("café");
    ui.tap_key(keyboard::key::Named::Backspace);
    ui.tap_key(keyboard::key::Named::Enter);
    let messages: Vec<_> = ui.into_messages().collect();
    assert!(messages.contains(&Message::Input("café".into())));
    assert!(messages.contains(&Message::Input("caf".into())));
    assert_eq!(messages.last(), Some(&Message::Submit));
    let mut disabled = Simulator::new(text_field::<Message>("Disabled", "Read only"));
    disabled.point_at((40.0, 36.0));
    disabled.simulate(simulator::click());
    disabled.typewrite("ignored");
    assert_eq!(disabled.into_messages().count(), 0);
}

#[test]
fn widget_state_survives_view_rebuilds_and_disabling_cancels_press() {
    use iced::advanced::widget::Tree;
    let mut element: Element<'_, Message> = button("Save").on_press(Message::Action).into();
    let mut tree = Tree::new(&element);
    let mut renderer = renderer();
    let node = element.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &iced::advanced::layout::Limits::new(Size::ZERO, Size::new(200.0, 100.0)),
    );
    let layout = iced::advanced::Layout::new(&node);
    let mut messages = Vec::new();
    let mut send = |element: &mut Element<'_, Message>, tree: &mut Tree, event: Event| {
        element.as_widget_mut().update(
            tree,
            &event,
            layout,
            mouse::Cursor::Available(Point::new(30.0, 20.0)),
            &renderer,
            &mut iced::advanced::clipboard::Null,
            &mut iced::advanced::Shell::new(&mut messages),
            &layout.bounds(),
        );
    };
    send(
        &mut element,
        &mut tree,
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
    );
    let mut rebuilt: Element<'_, Message> = button("Save").on_press(Message::Submit).into();
    tree.diff(&rebuilt);
    send(
        &mut rebuilt,
        &mut tree,
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
    );
    send(
        &mut rebuilt,
        &mut tree,
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
    );
    let mut disabled: Element<'_, Message> = button("Save").into();
    tree.diff(&disabled);
    send(
        &mut disabled,
        &mut tree,
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
    );
    assert_eq!(messages, [Message::Submit]);
    // Drawing uses the same renderer as iced_test's real software snapshots.
    disabled.as_widget().draw(
        &tree,
        &mut renderer,
        &Theme::light(),
        &Default::default(),
        layout,
        mouse::Cursor::Unavailable,
        &layout.bounds(),
    );
}

fn renderer() -> iced::Renderer {
    use iced::advanced::renderer::Headless;
    iced::futures::executor::block_on(<iced::Renderer as Headless>::new(
        iced::Font::DEFAULT,
        16.0.into(),
        Some("tiny-skia"),
    ))
    .unwrap()
}

#[test]
fn button_animation_stops_requesting_redraw_when_settled() {
    use iced::advanced::{Layout, Shell, layout::Limits, widget::Tree};
    let mut element: Element<'_, Message> = button("Save").on_press(Message::Action).into();
    let mut tree = Tree::new(&element);
    let renderer = renderer();
    let node = element.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &Limits::new(Size::ZERO, Size::new(200.0, 100.0)),
    );
    let layout = Layout::new(&node);
    let now = iced::time::Instant::now();
    for (event, cursor, expected_redraw) in [
        (
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            mouse::Cursor::Available(Point::new(30.0, 20.0)),
            true,
        ),
        (
            Event::Window(iced::window::Event::RedrawRequested(
                now + std::time::Duration::from_secs(1),
            )),
            mouse::Cursor::Available(Point::new(30.0, 20.0)),
            false,
        ),
        (
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            mouse::Cursor::Unavailable,
            true,
        ),
        (
            Event::Window(iced::window::Event::RedrawRequested(
                now + std::time::Duration::from_secs(2),
            )),
            mouse::Cursor::Unavailable,
            false,
        ),
    ] {
        let mut messages = Vec::new();
        let mut shell = Shell::new(&mut messages);
        element.as_widget_mut().update(
            &mut tree,
            &event,
            layout,
            cursor,
            &renderer,
            &mut iced::advanced::clipboard::Null,
            &mut shell,
            &layout.bounds(),
        );
        assert_eq!(
            shell.redraw_request() != iced::window::RedrawRequest::Wait,
            expected_redraw
        );
    }
}

#[test]
#[ignore = "writes visual artifacts for manual inspection"]
fn milestone_one_snapshots() {
    for (name, theme, width) in [
        ("light", Theme::light(), 800.0),
        ("dark", Theme::dark(), 800.0),
        ("narrow", Theme::light(), 360.0),
    ] {
        let content: Element<'_, Message> = container(
            column![
                button("Save changes").on_press(Message::Action),
                text_field(
                    "A long label that should stay safely inside the field",
                    "Ada Lovelace"
                )
                .on_input(Message::Input)
                .supporting_text("Supporting text can wrap onto multiple lines at narrow widths."),
                text_field("Email address", "not an email")
                    .on_input(Message::Input)
                    .error("Enter a valid email address."),
            ]
            .spacing(24),
        )
        .padding(32)
        .width(Length::Fill)
        .height(Length::Fill)
        .into();
        let mut ui = Simulator::with_size(Default::default(), Size::new(width, 440.0), content);
        ui.snapshot(&theme)
            .unwrap()
            .matches_image(format!("target/visuals/m1-{name}.png"))
            .unwrap();
    }
    let mut ui = Simulator::new(modal_view(true, true));
    ui.snapshot(&Theme::light())
        .unwrap()
        .matches_image("target/visuals/m1-dialog.png")
        .unwrap();
}

#[test]
fn text_entry_inside_dialog_relayouts_and_renders_after_focus() {
    let content = text_field("Dialog input", "").on_input(Message::Input);
    let mut ui = Simulator::new(modal(
        container("Background")
            .width(Length::Fill)
            .height(Length::Fill),
        Some(dialog(content).on_dismiss(Message::Dismiss)),
    ));
    ui.point_at((512.0, 388.0));
    ui.simulate(simulator::click());
    ui.typewrite("dialog note");
    ui.snapshot(&Theme::light()).unwrap();
    assert!(
        ui.into_messages()
            .any(|message| message == Message::Input("dialog note".into()))
    );
}

#[test]
fn focused_modal_input_only_schedules_future_caret_redraws_after_animation() {
    use iced_test::runtime::{UserInterface, user_interface};
    let content = text_field("Dialog input", "").on_input(Message::Input);
    let mut renderer = renderer();
    let mut ui = UserInterface::build(
        modal(
            container("Background")
                .width(Length::Fill)
                .height(Length::Fill),
            Some(dialog(content)),
        ),
        Size::new(1024.0, 768.0),
        user_interface::Cache::default(),
        &mut renderer,
    );
    let mut messages = Vec::new();
    let cursor = mouse::Cursor::Available(Point::new(512.0, 388.0));
    ui.update(
        &simulator::click().collect::<Vec<_>>(),
        cursor,
        &mut renderer,
        &mut iced::advanced::clipboard::Null,
        &mut messages,
    );
    let now = iced::time::Instant::now() + std::time::Duration::from_secs(1);
    for step in 0..10 {
        let now = now + std::time::Duration::from_millis(step * 100);
        let (state, _) = ui.update(
            &[Event::Window(iced::window::Event::RedrawRequested(now))],
            cursor,
            &mut renderer,
            &mut iced::advanced::clipboard::Null,
            &mut messages,
        );
        match state {
            user_interface::State::Updated {
                redraw_request: iced::window::RedrawRequest::At(next),
                ..
            } => assert!(next > now),
            _ => panic!("expected only a future caret blink after settling"),
        }
        ui.draw(&mut renderer, &Theme::light(), &Default::default(), cursor);
    }
}
