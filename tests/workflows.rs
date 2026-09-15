use iced::advanced::{Layout, Renderer as _, Shell, layout, renderer::Headless, widget::Tree};
use iced::{Event, Length, Point, Rectangle, Size, keyboard, mouse, widget, window};
use iced_material::{
    Element, MenuItem, Placement, RadioOption, SelectOption, Theme, button, context_menu,
    dialog::{dialog, modal},
    menu, radio, radio_group, select, snackbar, text_field, tooltip,
};
use iced_test::{Simulator, simulator};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq)]
enum Message {
    Action,
    Background,
    Dismiss,
    Select(u8),
    Input(String),
}
fn press() -> Event {
    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
}
fn release() -> Event {
    Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
}
fn redraw(now: Instant) -> Event {
    Event::Window(window::Event::RedrawRequested(now))
}
fn moved(point: Point) -> Event {
    Event::Mouse(mouse::Event::CursorMoved { position: point })
}
fn actions() -> iced_material::Menu<'static, Message> {
    menu(
        "Actions",
        [
            MenuItem::new("Duplicate", Message::Action),
            MenuItem::separator(),
            MenuItem::new("Unavailable", Message::Background).disabled(true),
        ],
    )
}
fn full(content: impl Into<Element<'static, Message>>) -> Element<'static, Message> {
    widget::container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

#[test]
fn menus_select_once_ignore_disabled_items_and_capture_outside_clicks() {
    let mut ui = Simulator::new(
        widget::column![
            actions(),
            button("Background").on_press(Message::Background)
        ]
        .spacing(240),
    );
    ui.click("Actions").unwrap();
    ui.click("Unavailable").unwrap();
    assert!(ui.find("Duplicate").is_ok());
    ui.click("Duplicate").unwrap();
    assert!(ui.find("Duplicate").is_err());
    ui.click("Actions").unwrap();
    ui.click("Background").unwrap();
    assert!(ui.find("Duplicate").is_err());
    ui.click("Background").unwrap();
    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        [Message::Action, Message::Background]
    );
}
#[test]
fn menus_cancel_drag_escape_and_outside_wheel_without_click_through() {
    let mut ui = Simulator::new(actions());
    ui.click("Actions").unwrap();
    ui.point_at((40.0, 76.0));
    ui.simulate([press()]);
    ui.point_at((900.0, 700.0));
    ui.simulate([release()]);
    assert!(
        ui.find("Duplicate").is_ok(),
        "drag from inside does not dismiss"
    );
    ui.tap_key(keyboard::key::Named::Escape);
    assert!(ui.find("Duplicate").is_err());
    ui.click("Actions").unwrap();
    ui.point_at((900.0, 700.0));
    let statuses = ui.simulate([Event::Mouse(mouse::Event::WheelScrolled {
        delta: mouse::ScrollDelta::Lines { x: 0.0, y: -2.0 },
    })]);
    assert_eq!(statuses, [iced::event::Status::Captured]);
    assert!(ui.find("Duplicate").is_err());
    assert_eq!(ui.into_messages().count(), 0);
}
#[test]
fn long_menu_scrolls_and_flips_at_bottom_of_small_window() {
    let content = widget::container(menu(
        "Actions",
        (0..30).map(|i| MenuItem::new(format!("Choice {i}"), Message::Select(i))),
    ))
    .align_bottom(Length::Fill)
    .align_right(Length::Fill)
    .padding(16);
    let mut ui = Simulator::with_size(Default::default(), Size::new(320.0, 280.0), content);
    ui.click("Actions").unwrap();
    ui.click("Choice 0").unwrap();
    ui.click("Actions").unwrap();
    ui.point_at((220.0, 150.0));
    ui.simulate([Event::Mouse(mouse::Event::WheelScrolled {
        delta: mouse::ScrollDelta::Lines { x: 0.0, y: -100.0 },
    })]);
    ui.click("Choice 29").unwrap();
    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        [Message::Select(0), Message::Select(29)]
    );
}
#[test]
fn context_menu_preserves_left_click_and_uses_right_click_position() {
    let mut ui = Simulator::new(context_menu(
        button("Workspace").on_press(Message::Background),
        [MenuItem::new("Duplicate", Message::Action)],
    ));
    ui.click("Workspace").unwrap();
    ui.point_at((30.0, 20.0));
    ui.simulate([
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)),
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right)),
    ]);
    ui.click("Duplicate").unwrap();
    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        [Message::Background, Message::Action]
    );
}
#[test]
fn dropdown_inside_dialog_selects_and_escape_only_closes_the_menu() {
    let dropdown = select(
        "Access",
        [
            SelectOption::new(1, "Viewer"),
            SelectOption::new(2, "Editor"),
            SelectOption::new(3, "Owner").disabled(true),
        ],
        Some(1),
    )
    .on_select(Message::Select);
    let mut ui = Simulator::new(modal(
        full(button("Background").on_press(Message::Background)),
        Some(dialog(dropdown).on_dismiss(Message::Dismiss)),
    ));
    ui.click("Access").unwrap();
    ui.click("Owner").unwrap();
    ui.click("Editor").unwrap();
    ui.click("Access").unwrap();
    ui.tap_key(keyboard::key::Named::Escape);
    assert!(ui.find("Editor").is_err());
    ui.tap_key(keyboard::key::Named::Escape);
    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        [Message::Select(2), Message::Dismiss]
    );
}
#[test]
fn radio_group_reports_single_selection_and_ignores_current_and_disabled_options() {
    let mut ui = Simulator::new(
        radio_group(
            [
                RadioOption::new(1, "Daily"),
                RadioOption::new(2, "Weekly"),
                RadioOption::new(3, "Unavailable").disabled(true),
            ],
            Some(1),
        )
        .on_select(Message::Select),
    );
    ui.click("Daily").unwrap();
    ui.click("Unavailable").unwrap();
    ui.click("Weekly").unwrap();
    assert_eq!(ui.into_messages().collect::<Vec<_>>(), [Message::Select(2)]);
}

struct Harness {
    element: Element<'static, Message>,
    tree: Tree,
    node: layout::Node,
    renderer: iced::Renderer,
}
impl Harness {
    fn new(element: impl Into<Element<'static, Message>>) -> Self {
        let renderer = iced::futures::executor::block_on(<iced::Renderer as Headless>::new(
            iced::Font::DEFAULT,
            16.0.into(),
            Some("tiny-skia"),
        ))
        .unwrap();
        let mut element = element.into();
        let mut tree = Tree::new(&element);
        let node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(600.0, 400.0)),
        );
        Self {
            element,
            tree,
            node,
            renderer,
        }
    }
    fn rebuild(&mut self, element: impl Into<Element<'static, Message>>) {
        self.element = element.into();
        self.tree.diff(&self.element);
        self.node = self.element.as_widget_mut().layout(
            &mut self.tree,
            &self.renderer,
            &layout::Limits::new(Size::ZERO, Size::new(600.0, 400.0)),
        );
    }
    fn event(
        &mut self,
        event: Event,
        point: Option<Point>,
    ) -> (Vec<Message>, window::RedrawRequest) {
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
            &Rectangle::with_size(Size::new(600.0, 400.0)),
        );
        let redraw = shell.redraw_request();
        (messages, redraw)
    }
    fn popup_bounds(&mut self) -> Option<Rectangle> {
        let mut overlay = self.element.as_widget_mut().overlay(
            &mut self.tree,
            Layout::new(&self.node),
            &self.renderer,
            &Rectangle::with_size(Size::new(600.0, 400.0)),
            iced::Vector::ZERO,
        )?;
        let node = overlay
            .as_overlay_mut()
            .layout(&self.renderer, Size::new(600.0, 400.0));
        Some(node.bounds())
    }
    fn frame(&mut self) -> Vec<u8> {
        let viewport = Rectangle::with_size(Size::new(600.0, 400.0));
        self.renderer.reset(viewport);
        self.element.as_widget().draw(
            &self.tree,
            &mut self.renderer,
            &Theme::light(),
            &Default::default(),
            Layout::new(&self.node),
            mouse::Cursor::Unavailable,
            &viewport,
        );
        self.renderer
            .screenshot(Size::new(600, 400), 1.0, Theme::light().colors.surface)
    }
}
#[test]
fn open_menu_survives_rebuild_and_disabling_cancels_it() {
    let point = Some(Point::new(40.0, 20.0));
    let mut ui = Harness::new(actions());
    ui.event(press(), point);
    ui.event(release(), point);
    assert!(ui.popup_bounds().is_some());
    ui.rebuild(actions());
    assert!(ui.popup_bounds().is_some());
    ui.rebuild(actions().disabled(true));
    assert!(ui.popup_bounds().is_none());
    ui.event(press(), point);
    ui.event(release(), point);
    assert!(ui.popup_bounds().is_none());
    ui.rebuild(actions());
    ui.event(release(), point);
    assert!(ui.popup_bounds().is_none());
}
#[test]
fn tooltip_waits_then_stops_redrawing_and_never_intercepts_the_action() {
    let point = Some(Point::new(30.0, 20.0));
    let now = Instant::now();
    let mut ui = Harness::new(tooltip(
        button("Pin").on_press(Message::Action),
        "Pin this workspace",
    ));
    ui.event(redraw(now), point);
    assert!(ui.popup_bounds().is_none());
    ui.event(redraw(now + Duration::from_millis(499)), point);
    assert!(ui.popup_bounds().is_none());
    ui.event(redraw(now + Duration::from_millis(500)), point);
    assert!(ui.popup_bounds().is_some());
    ui.event(redraw(now + Duration::from_secs(1)), point);
    assert_eq!(
        ui.event(redraw(now + Duration::from_millis(1001)), point).1,
        window::RedrawRequest::Wait
    );
    ui.event(press(), point);
    assert!(ui.popup_bounds().is_none());
    assert_eq!(ui.event(release(), point).0, [Message::Action]);
    ui.event(redraw(now + Duration::from_secs(2)), point);
    assert!(ui.popup_bounds().is_none());
    ui.event(redraw(now + Duration::from_secs(3)), None);
    ui.event(redraw(now + Duration::from_secs(4)), point);
    ui.event(redraw(now + Duration::from_secs(5)), point);
    assert!(ui.popup_bounds().is_some());
    ui.event(Event::Window(window::Event::Unfocused), point);
    ui.event(redraw(now + Duration::from_secs(6)), point);
    assert!(ui.popup_bounds().is_none());
}
#[test]
fn open_tooltip_is_passive_in_the_real_overlay_runtime() {
    let mut ui = Simulator::new(
        tooltip(button("Pin").on_press(Message::Action), "Hint").delay(Duration::ZERO),
    );
    ui.point_at((30.0, 20.0));
    ui.simulate([redraw(Instant::now())]);
    ui.click("Pin").unwrap();
    assert_eq!(ui.into_messages().collect::<Vec<_>>(), [Message::Action]);
}
fn notice(id: u64) -> Element<'static, Message> {
    snackbar::host(
        full(button("Background").on_press(Message::Background)),
        Some(
            snackbar("Saved")
                .id(id)
                .on_dismiss(Message::Dismiss)
                .action("Undo", Message::Action),
        ),
    )
}
#[test]
fn snackbar_timeout_is_once_only_and_rebuild_preserves_or_restarts_by_id() {
    let now = Instant::now();
    let mut ui = Harness::new(notice(1));
    ui.event(redraw(now), None);
    ui.rebuild(notice(1));
    assert!(
        ui.event(redraw(now + Duration::from_secs(3)), None)
            .0
            .is_empty()
    );
    assert_eq!(
        ui.event(redraw(now + Duration::from_secs(4)), None).0,
        [Message::Dismiss]
    );
    ui.event(redraw(now + Duration::from_secs(5)), None);
    ui.event(redraw(now + Duration::from_millis(5200)), None);
    assert_eq!(
        ui.event(redraw(now + Duration::from_millis(5201)), None),
        (vec![], window::RedrawRequest::Wait)
    );
    ui.rebuild(notice(2));
    ui.event(redraw(now + Duration::from_secs(6)), None);
    assert!(
        ui.event(redraw(now + Duration::from_secs(9)), None)
            .0
            .is_empty()
    );
    assert_eq!(
        ui.event(redraw(now + Duration::from_secs(10)), None).0,
        [Message::Dismiss]
    );
}
#[test]
fn snackbar_pauses_while_hovered_or_window_is_inactive() {
    let mut ui = Harness::new(notice(1));
    let now = Instant::now();
    let point = Some(Point::new(300.0, 360.0));
    ui.event(moved(Point::new(300.0, 360.0)), point);
    assert!(
        ui.event(redraw(now + Duration::from_secs(10)), point)
            .0
            .is_empty()
    );
    ui.event(Event::Window(window::Event::Unfocused), None);
    assert!(
        ui.event(redraw(now + Duration::from_secs(20)), None)
            .0
            .is_empty()
    );
    ui.event(Event::Window(window::Event::Focused), None);
    assert_eq!(
        ui.event(redraw(now + Duration::from_secs(30)), None).0,
        [Message::Dismiss]
    );
}
#[test]
fn snackbar_action_dismisses_without_clicking_through_and_background_keeps_working() {
    let mut ui = Simulator::with_size(Default::default(), Size::new(600.0, 400.0), notice(1));
    ui.click("Background").unwrap();
    ui.click("Undo").unwrap();
    assert!(ui.find("Undo").is_err());
    ui.click("Background").unwrap();
    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        [Message::Background, Message::Action, Message::Background]
    );
}
#[test]
fn snackbar_cancels_a_drag_ending_over_it_and_preserves_typing_under_hover() {
    let background = button("Whole window")
        .width(Length::Fill)
        .height(Length::Fill)
        .on_press(Message::Background);
    let mut ui = Simulator::with_size(
        Default::default(),
        Size::new(600.0, 400.0),
        snackbar::host(background, Some(snackbar("Saved").persistent())),
    );
    ui.point_at((10.0, 10.0));
    ui.simulate([press()]);
    ui.point_at((300.0, 360.0));
    ui.simulate([release()]);
    ui.point_at((10.0, 10.0));
    ui.simulate([release()]);
    assert_eq!(ui.into_messages().count(), 0);
    let mut ui = Simulator::with_size(
        Default::default(),
        Size::new(600.0, 400.0),
        snackbar::host(
            full(text_field("Name", "").on_input(Message::Input)),
            Some(snackbar("Saved").persistent()),
        ),
    );
    ui.point_at((40.0, 36.0));
    ui.simulate(simulator::click());
    ui.point_at((300.0, 360.0));
    ui.simulate([moved(Point::new(300.0, 360.0))]);
    ui.typewrite("Ada");
    assert!(
        ui.into_messages()
            .any(|m| m == Message::Input("Ada".into()))
    );
}
#[test]
fn radio_selection_animates_and_disabled_controls_cancel_pending_presses() {
    let make = |selected, disabled| {
        radio("Daily", 1u8, selected)
            .on_select(Message::Select)
            .disabled(disabled)
    };
    let mut ui = Harness::new(make(None, false));
    let blank = ui.frame();
    let now = Instant::now();
    ui.rebuild(make(Some(1), false));
    ui.event(redraw(now), None);
    ui.event(redraw(now + Duration::from_millis(60)), None);
    let during = ui.frame();
    assert_eq!(
        ui.event(redraw(now + Duration::from_secs(1)), None).1,
        window::RedrawRequest::Wait
    );
    let selected = ui.frame();
    assert!(blank != during && during != selected && selected != blank);
    let point = Some(Point::new(24.0, 24.0));
    ui.rebuild(make(None, false));
    ui.event(press(), point);
    ui.rebuild(make(None, true));
    assert!(ui.event(release(), point).0.is_empty());
    ui.rebuild(make(None, false));
    assert!(ui.event(release(), point).0.is_empty());
}

#[test]
#[ignore = "writes workflow visual artifacts for manual inspection"]
fn workflow_snapshots() {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    for (name, width, height, dark) in [
        ("menu", 600.0, 400.0, false),
        ("menu-dark", 600.0, 400.0, true),
        ("edge", 320.0, 280.0, false),
        ("tooltip", 600.0, 400.0, false),
        ("tooltip-dark", 320.0, 280.0, true),
        ("snackbar", 600.0, 400.0, false),
        ("snackbar-dark", 320.0, 280.0, true),
        ("radio", 600.0, 320.0, false),
        ("radio-dark", 320.0, 320.0, true),
        ("dialog-menu", 390.0, 560.0, false),
    ] {
        let content: Element<'static, Message> = match name {
            "edge" => widget::container(actions())
                .padding(16)
                .align_bottom(Length::Fill)
                .align_right(Length::Fill)
                .into(),
            "tooltip" | "tooltip-dark" => widget::container(
                tooltip(
                    button("Pin").on_press(Message::Action),
                    "Keep this workspace at the top of your list",
                )
                .delay(Duration::ZERO)
                .placement(Placement::Top),
            )
            .padding(16)
            .align_top(Length::Fill)
            .align_right(Length::Fill)
            .into(),
            "snackbar" | "snackbar-dark" => snackbar::host(
                full("Your workspace"),
                Some(
                    snackbar("Workspace duplicated. Changes are saved for this session.")
                        .persistent()
                        .action("Undo", Message::Action),
                ),
            ),
            "radio" | "radio-dark" => widget::container(
                radio_group(
                    [
                        RadioOption::new(1, "Daily summary"),
                        RadioOption::new(2, "As things happen"),
                        RadioOption::new(3, "Weekly (managed)").disabled(true),
                    ],
                    Some(1),
                )
                .on_select(Message::Select),
            )
            .padding(24)
            .into(),
            "dialog-menu" => modal(
                full("Workspace"),
                Some(dialog(
                    widget::column![
                        widget::text("Choose workspace access"),
                        select(
                            "Access",
                            [
                                SelectOption::new(1, "Viewer"),
                                SelectOption::new(2, "Editor"),
                                SelectOption::new(3, "Owner (managed)").disabled(true)
                            ],
                            Some(1)
                        )
                        .on_select(Message::Select),
                        text_field("Note", "").on_input(Message::Input)
                    ]
                    .spacing(24),
                )),
            ),
            _ => widget::container(menu(
                "Workspace actions",
                [
                    MenuItem::new("Duplicate workspace", Message::Action).shortcut("⌘D"),
                    MenuItem::new("Save preferences", Message::Action).selected(true),
                    MenuItem::separator(),
                    MenuItem::new("Share (coming soon)", Message::Action).disabled(true),
                    MenuItem::new("Delete workspace", Message::Action).destructive(true),
                ],
            ))
            .padding(24)
            .into(),
        };
        let mut ui = Simulator::with_size(Default::default(), Size::new(width, height), content);
        match name {
            "menu" | "menu-dark" => {
                ui.click("Workspace actions").unwrap();
            }
            "edge" => {
                ui.click("Actions").unwrap();
            }
            "dialog-menu" => {
                ui.click("Access").unwrap();
            }
            "tooltip" | "tooltip-dark" => {
                let bounds = ui.find("Pin").unwrap().visible_bounds().unwrap();
                ui.point_at(bounds.center());
                ui.simulate([redraw(Instant::now())]);
            }
            "radio" | "radio-dark" => {
                ui.point_at((90.0, 96.0));
                ui.simulate([
                    redraw(Instant::now()),
                    redraw(Instant::now() + Duration::from_secs(1)),
                ]);
            }
            _ => {}
        }
        let destination = format!("target/visuals/workflow-{name}-{backend}.png");
        if std::path::Path::new(&destination).exists() {
            std::fs::remove_file(destination).unwrap();
        }
        ui.snapshot(&if dark { Theme::dark() } else { Theme::light() })
            .unwrap()
            .matches_image(format!("target/visuals/workflow-{name}.png"))
            .unwrap();
    }
}

#[test]
fn closing_a_modal_resumes_the_snackbar_timer_without_pointer_movement() {
    let make = |open: bool| modal(notice(1), open.then(|| dialog("Modal content")));
    let mut ui = Harness::new(make(true));
    ui.event(redraw(Instant::now()), None);
    assert!(
        ui.event(redraw(Instant::now() + Duration::from_secs(30)), None)
            .0
            .is_empty()
    );
    ui.rebuild(make(false));
    ui.event(redraw(Instant::now()), None);
    assert_eq!(
        ui.event(redraw(Instant::now() + Duration::from_secs(5)), None)
            .0,
        [Message::Dismiss]
    );
}

#[test]
fn a_menu_in_scrolled_content_anchors_to_its_visible_trigger() {
    let content = widget::scrollable(widget::column![
        widget::space().height(500),
        actions(),
        widget::space().height(500)
    ]);
    let mut ui = Simulator::with_size(Default::default(), Size::new(320.0, 300.0), content);
    ui.point_at((40.0, 150.0));
    ui.simulate([Event::Mouse(mouse::Event::WheelScrolled {
        delta: mouse::ScrollDelta::Pixels { x: 0.0, y: -450.0 },
    })]);
    ui.click("Actions").unwrap();
    ui.click("Duplicate").unwrap();
    assert_eq!(ui.into_messages().collect::<Vec<_>>(), [Message::Action]);
}

#[test]
fn a_tooltip_in_scrolled_content_stays_near_its_visible_trigger() {
    let content = widget::scrollable(widget::column![
        widget::space().height(500),
        tooltip(button("Pin").on_press(Message::Action), "Hint").delay(Duration::ZERO),
        widget::space().height(500)
    ])
    .width(Length::Fill);
    let mut ui = Simulator::with_size(Default::default(), Size::new(320.0, 300.0), content);
    ui.point_at((40.0, 150.0));
    ui.simulate([Event::Mouse(mouse::Event::WheelScrolled {
        delta: mouse::ScrollDelta::Pixels { x: 0.0, y: -450.0 },
    })]);
    let bounds = ui.find("Pin").unwrap().visible_bounds().unwrap();
    ui.point_at(bounds.center());
    ui.simulate([redraw(Instant::now())]);
    let hint = ui.find("Hint").unwrap().visible_bounds().unwrap();
    assert!(hint.y < bounds.y && bounds.y - hint.y < 80.0);
    ui.click("Pin").unwrap();
    assert_eq!(ui.into_messages().collect::<Vec<_>>(), [Message::Action]);
}

#[test]
fn a_new_snackbar_keeps_the_window_deactivation_pause() {
    let mut ui = Harness::new(notice(1));
    ui.event(Event::Window(window::Event::Unfocused), None);
    ui.rebuild(notice(2));
    let now = Instant::now();
    ui.event(redraw(now), None);
    assert_eq!(
        ui.event(redraw(now + Duration::from_secs(20)), None),
        (vec![], window::RedrawRequest::Wait)
    );
    ui.event(Event::Window(window::Event::Focused), None);
    assert_eq!(
        ui.event(redraw(Instant::now() + Duration::from_secs(5)), None)
            .0,
        [Message::Dismiss]
    );
}
