use iced::advanced::{
    Layout, Renderer as _, Shell, Widget, layout,
    renderer::{self, Headless},
    widget::Tree,
};
use iced::{Event, Length, Point, Rectangle, Size, Vector, mouse, widget, window};
use iced_m3::{
    ButtonVariant, Element, MenuItem, NavigationItem, Tab, TabVariant, Theme, app_bar, button,
    checkbox, icon_button, list, list_item, menu, navigation_rail, switch, tabs,
};
use iced_test::{Simulator, simulator};
use std::time::{Duration, Instant};
#[derive(Clone, Debug, PartialEq)]
enum Message {
    Select(u8),
    Row,
    Child,
    Toggle(bool),
    Dismiss,
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
fn tab_view(selected: u8, disabled: bool, variant: TabVariant) -> Element<'static, Message> {
    tabs(
        [
            Tab::new(1, "Overview"),
            Tab::new(2, "Files"),
            Tab::new(3, "Unavailable").disabled(true),
        ],
        Some(selected),
    )
    .on_select(Message::Select)
    .disabled(disabled)
    .variant(variant)
    .width(360)
    .into()
}
#[test]
fn tabs_emit_controlled_values_and_ignore_disabled_options() {
    for variant in [TabVariant::Primary, TabVariant::Secondary] {
        let mut ui = Simulator::new(tab_view(1, false, variant));
        let label = ui.find("Files").unwrap().visible_bounds().unwrap();
        assert!(
            (label.center_x() - 180.0).abs() < 0.01,
            "tab content stays centered in its equal-width slot"
        );
        ui.click("Files").unwrap();
        ui.click("Unavailable").unwrap();
        assert_eq!(ui.into_messages().collect::<Vec<_>>(), [Message::Select(2)]);
        let mut ui = Simulator::new(tab_view(1, true, variant));
        ui.click("Files").unwrap();
        assert_eq!(ui.into_messages().count(), 0);
    }
}
#[test]
fn overflowing_tabs_scroll_to_a_reachable_action() {
    let mut ui = Simulator::with_size(
        Default::default(),
        Size::new(320.0, 200.0),
        tabs(
            (0..16).map(|i| Tab::new(i, format!("Destination {i}"))),
            Some(0),
        )
        .on_select(Message::Select)
        .scrollable(true),
    );
    ui.point_at((150.0, 24.0));
    ui.simulate([Event::Mouse(mouse::Event::WheelScrolled {
        delta: mouse::ScrollDelta::Pixels { x: -4000.0, y: 0.0 },
    })]);
    ui.click("Destination 15").unwrap();
    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        [Message::Select(15)]
    );
}
fn row(disabled: bool) -> Element<'static, Message> {
    list_item("Document")
        .supporting_text("Updated today")
        .trailing(button("Pin").on_press(Message::Child))
        .on_press(Message::Row)
        .disabled(disabled)
        .width(360)
        .into()
}
#[test]
fn list_row_and_trailing_action_each_emit_once_and_disabled_blocks_both() {
    let mut ui = Simulator::new(row(false));
    ui.click("Pin").unwrap();
    ui.click("Document").unwrap();
    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        [Message::Child, Message::Row]
    );
    let mut ui = Simulator::new(row(true));
    ui.click("Pin").unwrap();
    ui.click("Document").unwrap();
    assert_eq!(ui.into_messages().count(), 0);
}
#[test]
fn static_list_rows_keep_their_child_controls_interactive() {
    let mut ui = Simulator::new(
        list_item("Notifications")
            .trailing(checkbox(false).label("Enabled").on_toggle(Message::Toggle)),
    );
    ui.click("Notifications").unwrap();
    ui.click("Enabled").unwrap();
    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        [Message::Toggle(true)]
    );
}
#[test]
fn crossing_between_a_row_and_its_action_cancels_both_gestures() {
    for child_first in [false, true] {
        let mut ui = Simulator::new(row(false));
        let child = ui.find("Pin").unwrap().visible_bounds().unwrap().center();
        let label = ui
            .find("Document")
            .unwrap()
            .visible_bounds()
            .unwrap()
            .center();
        ui.point_at(if child_first { child } else { label });
        ui.simulate([press()]);
        ui.point_at(if child_first { label } else { child });
        ui.simulate([release()]);
        ui.point_at(child);
        ui.simulate([release()]);
        assert_eq!(ui.into_messages().count(), 0);
    }
}
#[test]
fn a_trailing_menu_forwards_its_overlay_without_activating_the_row() {
    let mut ui = Simulator::new(
        list_item("Document")
            .trailing(menu("More", [MenuItem::new("Duplicate", Message::Child)]))
            .on_press(Message::Row),
    );
    ui.click("More").unwrap();
    ui.click("Duplicate").unwrap();
    assert_eq!(ui.into_messages().collect::<Vec<_>>(), [Message::Child]);
}
#[test]
fn app_bar_actions_and_overflow_work_with_a_long_title_at_minimum_width() {
    let mut ui = Simulator::with_size(
        Default::default(),
        Size::new(320.0, 240.0),
        app_bar("An unusually long workspace title that cannot push actions away")
            .leading(button("Back").on_press(Message::Row))
            .action(menu("More", [MenuItem::new("Settings", Message::Child)])),
    );
    ui.click("Back").unwrap();
    ui.click("More").unwrap();
    ui.click("Settings").unwrap();
    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        [Message::Row, Message::Child]
    );
}
fn rail(selected: u8) -> Element<'static, Message> {
    navigation_rail(
        [
            NavigationItem::new(1, "Workspace", Ink),
            NavigationItem::new(2, "Activity", Ink).badge(3),
            NavigationItem::new(3, "Disabled", Ink).disabled(true),
        ],
        Some(selected),
    )
    .on_select(Message::Select)
    .into()
}
#[test]
fn rail_destinations_are_controlled_and_disabled_items_do_not_emit() {
    let mut ui = Simulator::new(rail(1));
    ui.click("Activity").unwrap();
    ui.click("Disabled").unwrap();
    assert_eq!(ui.into_messages().collect::<Vec<_>>(), [Message::Select(2)]);
}
#[test]
fn long_rails_scroll_while_footer_actions_stay_reachable() {
    let mut ui = Simulator::with_size(
        Default::default(),
        Size::new(320.0, 300.0),
        navigation_rail(
            (0..12).map(|i| NavigationItem::new(i, format!("Page {i}"), Ink)),
            Some(0),
        )
        .on_select(Message::Select)
        .footer(button("Help").on_press(Message::Child)),
    );
    ui.click("Help").unwrap();
    ui.point_at((40.0, 150.0));
    ui.simulate([Event::Mouse(mouse::Event::WheelScrolled {
        delta: mouse::ScrollDelta::Lines { x: 0.0, y: -100.0 },
    })]);
    ui.click("Page 11").unwrap();
    assert_eq!(
        ui.into_messages().collect::<Vec<_>>(),
        [Message::Child, Message::Select(11)]
    );
}
#[test]
fn root_dialog_blocks_navigation_and_list_actions() {
    let mut ui = Simulator::new(iced_m3::dialog::modal(
        widget::row![rail(1), row(false)],
        Some(
            iced_m3::dialog::dialog("Dialog")
                .on_dismiss(Message::Dismiss)
                .dismiss_on_outside(false),
        ),
    ));
    ui.point_at((40.0, 135.0));
    ui.simulate(simulator::click());
    ui.point_at((330.0, 30.0));
    ui.simulate(simulator::click());
    assert_eq!(ui.into_messages().count(), 0);
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
            &layout::Limits::new(Size::ZERO, Size::new(400.0, 260.0)),
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
            &layout::Limits::new(Size::ZERO, Size::new(400.0, 260.0)),
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
            &Rectangle::with_size(Size::new(400.0, 260.0)),
        );
        let request = shell.redraw_request();
        (messages, request)
    }
    fn frame(&mut self, offset: f32) -> Vec<u8> {
        let viewport = Rectangle::with_size(Size::new(400.0, 260.0));
        self.renderer.reset(viewport);
        self.renderer
            .with_translation(Vector::new(0.0, -offset), |renderer| {
                self.element.as_widget().draw(
                    &self.tree,
                    renderer,
                    &Theme::light(),
                    &Default::default(),
                    Layout::with_offset(Vector::new(0.0, offset), &self.node),
                    mouse::Cursor::Unavailable,
                    &(viewport + Vector::new(0.0, offset)),
                )
            });
        self.renderer
            .screenshot(Size::new(800, 520), 2.0, Theme::light().colors.surface)
    }
}
fn pixel(image: &[u8], x: usize, y: usize) -> &[u8] {
    &image[(y * 800 + x) * 4..(y * 800 + x + 1) * 4]
}
#[test]
fn tab_indicator_animates_across_rebuilds_settles_and_follows_scroll_translation() {
    for variant in [TabVariant::Primary, TabVariant::Secondary] {
        let mut ui = Harness::new(tab_view(1, false, variant));
        let now = Instant::now();
        ui.event(redraw(now), None);
        let before = ui.frame(0.0);
        ui.rebuild(tab_view(2, false, variant));
        ui.event(redraw(now), None);
        ui.event(redraw(now + Duration::from_millis(50)), None);
        let during = ui.frame(0.0);
        assert_eq!(
            ui.event(redraw(now + Duration::from_secs(1)), None).1,
            window::RedrawRequest::Wait
        );
        let after = ui.frame(0.0);
        assert!(before != during && during != after);
        assert!(
            after == ui.frame(450.0),
            "indicator must stay aligned when scrolled"
        );
    }
}
#[test]
fn disabling_tabs_during_a_press_cancels_it() {
    let mut ui = Harness::new(tab_view(1, false, TabVariant::Primary));
    let point = Some(Point::new(180.0, 24.0));
    ui.event(press(), point);
    ui.rebuild(tab_view(1, true, TabVariant::Primary));
    assert!(ui.event(release(), point).0.is_empty());
    ui.rebuild(tab_view(1, false, TabVariant::Primary));
    assert!(ui.event(release(), point).0.is_empty());
}
#[test]
fn long_app_bar_titles_cannot_paint_over_actions() {
    let make = |title| app_bar(title).action(button("More").on_press(Message::Child));
    let mut empty = Harness::new(make(""));
    let empty = empty.frame(0.0);
    let mut long = Harness::new(make(
        "A long workspace title that keeps going past its available space",
    ));
    let long = long.frame(0.0);
    assert!((620..800).all(|x| (0..128).all(|y| pixel(&empty, x, y) == pixel(&long, x, y))));
}
#[test]
fn primary_icon_tabs_reserve_room_for_badges_at_narrow_width() {
    let mut ui = Simulator::with_size(
        Default::default(),
        Size::new(288.0, 100.0),
        tabs(
            [
                Tab::new(1, "Files").icon(Ink),
                Tab::new(2, "Updates").icon(Ink).badge(3),
                Tab::new(3, "Archive").icon(Ink),
            ],
            Some(2),
        )
        .on_select(Message::Select),
    );
    let badge = ui.find("3").unwrap().visible_bounds().unwrap();
    let mut reference = Simulator::<Message, Theme>::new(iced_m3::badge(3));
    let natural = reference.find("3").unwrap().visible_bounds().unwrap();
    assert_eq!(badge.width, natural.width);
    assert!(badge.x + badge.width < 192.0);
    ui.click("Updates").unwrap();
    assert_eq!(ui.into_messages().collect::<Vec<_>>(), [Message::Select(2)]);
}
#[test]
fn list_rows_grow_for_wrapped_text_and_preserve_child_gestures_on_rebuild() {
    let mut ui = Harness::new(list_item("Headline").supporting_text("A long description that wraps over many lines at narrow widths and must remain fully inside the list row.").width(160));
    assert!(ui.node.size().height > 72.0);
    assert!(ui.frame(0.0) == ui.frame(450.0));
    let point = Some(Point::new(312.0, 36.0));
    ui.rebuild(row(false));
    ui.event(press(), point);
    ui.rebuild(row(false));
    assert_eq!(ui.event(release(), point).0, [Message::Child]);
    ui.event(press(), point);
    ui.rebuild(row(true));
    ui.event(release(), point);
    ui.rebuild(row(false));
    assert!(ui.event(release(), point).0.is_empty());
}
#[test]
fn switch_label_remains_visible_and_hover_only_surrounds_the_thumb() {
    for checked in [false, true] {
        let mut ui = Harness::new(
            switch(checked)
                .label("Switch label")
                .on_toggle(Message::Toggle),
        );
        let before = ui.frame(0.0);
        assert!(
            (152..320).any(|x| (24..72).any(|y| pixel(&before, x, y) != pixel(&before, 799, 519))),
            "label must actually render"
        );
        let position = Point::new(100.0, 24.0);
        let point = Some(position);
        ui.event(Event::Mouse(mouse::Event::CursorMoved { position }), point);
        assert_eq!(
            ui.event(redraw(Instant::now() + Duration::from_secs(1)), point)
                .1,
            window::RedrawRequest::Wait
        );
        let hovered = ui.frame(0.0);
        let thumb_x = if checked { 88 } else { 48 };
        assert_ne!(
            pixel(&before, thumb_x, 10),
            pixel(&hovered, thumb_x, 10),
            "halo extends outside the track"
        );
        assert!(
            (152..360).all(|x| (0..96).all(|y| pixel(&before, x, y) == pixel(&hovered, x, y))),
            "hover does not tint the label"
        );
        assert!(hovered == ui.frame(450.0));
        ui.event(press(), point);
        ui.event(redraw(Instant::now() + Duration::from_secs(2)), point);
        let pressed = ui.frame(0.0);
        assert!(pressed != hovered, "pressed thumb grows");
        assert_eq!(ui.event(release(), point).0, [Message::Toggle(!checked)]);
    }
}
#[test]
fn disabled_switch_cancels_an_active_gesture_and_has_no_hover_motion() {
    let mut ui = Harness::new(switch(false).label("Switch").on_toggle(Message::Toggle));
    let point = Some(Point::new(24.0, 24.0));
    ui.event(press(), point);
    ui.rebuild(switch(false).label("Switch"));
    let before = ui.frame(0.0);
    assert!(ui.event(release(), point).0.is_empty());
    assert_eq!(
        ui.event(redraw(Instant::now() + Duration::from_secs(1)), point)
            .1,
        window::RedrawRequest::Wait
    );
    assert!(before == ui.frame(0.0));
}
// A passive square inherits foreground just as a monochrome SVG icon does.
struct Ink;
impl Widget<Message, Theme, iced::Renderer> for Ink {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(24.0), Length::Fixed(24.0))
    }
    fn layout(
        &mut self,
        _: &mut Tree,
        _: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, 24.0, 24.0)
    }
    fn draw(
        &self,
        _: &Tree,
        renderer: &mut iced::Renderer,
        _: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        _: mouse::Cursor,
        _: &Rectangle,
    ) {
        let center = layout.bounds().center();
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle::new(
                    Point::new(center.x - 4.0, center.y - 4.0),
                    Size::new(8.0, 8.0),
                ),
                ..Default::default()
            },
            style.text_color,
        );
    }
}
impl From<Ink> for Element<'static, Message> {
    fn from(ink: Ink) -> Self {
        Self::new(ink)
    }
}
#[test]
fn icon_button_variants_use_distinct_material_foregrounds_and_selection_surfaces() {
    let mut standard = Harness::new(icon_button(Ink).on_press(Message::Child));
    let standard = standard.frame(0.0);
    let mut selected = Harness::new(icon_button(Ink).selected(true).on_press(Message::Child));
    let selected = selected.frame(0.0);
    assert_ne!(pixel(&standard, 40, 40), pixel(&selected, 40, 40));
    assert_eq!(
        pixel(&standard, 40, 16),
        pixel(&selected, 40, 16),
        "standard selection stays transparent"
    );
    for variant in [
        ButtonVariant::Outlined,
        ButtonVariant::Filled,
        ButtonVariant::Tonal,
    ] {
        let mut off = Harness::new(
            icon_button(Ink)
                .variant(variant)
                .selected(false)
                .on_press(Message::Child),
        );
        let off = off.frame(0.0);
        let mut on = Harness::new(
            icon_button(Ink)
                .variant(variant)
                .selected(true)
                .on_press(Message::Child),
        );
        let on = on.frame(0.0);
        assert_ne!(
            pixel(&off, 40, 16),
            pixel(&on, 40, 16),
            "selection changes the container"
        );
    }
}
#[test]
fn selected_rail_item_retains_visible_hover_feedback() {
    let mut ui = Harness::new(rail(1));
    let before = ui.frame(0.0);
    let point = Some(Point::new(40.0, 32.0));
    ui.event(redraw(Instant::now()), point);
    ui.event(redraw(Instant::now() + Duration::from_secs(1)), point);
    let after = ui.frame(0.0);
    assert!(
        before != after,
        "selected container must not cover its hover layer"
    );
    assert_eq!(
        ui.event(redraw(Instant::now() + Duration::from_secs(2)), point)
            .1,
        window::RedrawRequest::Wait
    );
}

#[test]
#[ignore = "writes navigation and selection-control renders for manual inspection"]
fn navigation_snapshots() {
    use iced_m3::{TypeScale, typography};
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    for (name, width, dark, pressed) in [
        ("light", 660.0, false, false),
        ("dark", 660.0, true, false),
        ("narrow", 320.0, false, false),
        ("pressed", 660.0, false, true),
    ] {
        let primary = tabs(
            [
                Tab::new(1, "Files").icon(Ink),
                Tab::new(2, "Updates").icon(Ink).badge(3),
                Tab::new(3, "Archive").icon(Ink).disabled(true),
            ],
            Some(2),
        )
        .on_select(Message::Select);
        let secondary = tabs(
            [
                Tab::new(1, "Overview"),
                Tab::new(2, "Recent"),
                Tab::new(3, "Shared"),
            ],
            Some(1),
        )
        .variant(TabVariant::Secondary)
        .on_select(Message::Select);
        let rows = list([
            list_item("One line")
                .leading(Ink)
                .on_press(Message::Row)
                .into(),
            list_item("Two lines")
                .supporting_text("Supporting text wraps when space is limited.")
                .trailing(checkbox(true).on_toggle(Message::Toggle))
                .selected(true)
                .on_press(Message::Row)
                .into(),
            list_item("Three lines")
                .overline("COLLECTION")
                .supporting_text("A longer description remains readable in a narrow window.")
                .trailing(menu("More", [MenuItem::new("Duplicate", Message::Child)]))
                .into(),
            list_item("Managed by your team")
                .leading(Ink)
                .disabled(true)
                .into(),
        ]);
        let mut icons = widget::Row::new().spacing(8);
        for selected in [false, true] {
            for variant in [
                ButtonVariant::Text,
                ButtonVariant::Outlined,
                ButtonVariant::Filled,
                ButtonVariant::Tonal,
            ] {
                icons = icons.push(
                    icon_button(Ink)
                        .variant(variant)
                        .selected(selected)
                        .on_press(Message::Child),
                );
            }
        }
        let content = widget::column![
            app_bar("A workspace title that can grow very long")
                .leading(icon_button(Ink).on_press(Message::Child))
                .action(menu("More", [MenuItem::new("Settings", Message::Child)]))
                .scrolled(true),
            widget::container(
                widget::column![
                    typography("Primary tabs", TypeScale::TitleMedium),
                    primary,
                    typography("Secondary tabs", TypeScale::TitleMedium),
                    secondary,
                    rows,
                    typography("Switch feedback and icon variants", TypeScale::TitleMedium),
                    switch(false).label("Off switch").on_toggle(Message::Toggle),
                    switch(true).label("On switch").on_toggle(Message::Toggle),
                    switch(true).label("Disabled switch"),
                    icons.wrap(),
                ]
                .spacing(12)
            )
            .padding(16),
        ];
        let mut ui = Simulator::with_size(Default::default(), Size::new(width, 1050.0), content);
        let point = ui
            .find("On switch")
            .unwrap()
            .visible_bounds()
            .unwrap()
            .center();
        ui.point_at(point);
        let now = Instant::now();
        ui.simulate([redraw(now), redraw(now + Duration::from_secs(1))]);
        if pressed {
            ui.simulate([press(), redraw(now + Duration::from_secs(2))]);
        }
        let destination = format!("target/visuals/navigation-{name}-{backend}.png");
        if std::path::Path::new(&destination).exists() {
            std::fs::remove_file(destination).unwrap();
        }
        ui.snapshot(&if dark { Theme::dark() } else { Theme::light() })
            .unwrap()
            .matches_image(format!("target/visuals/navigation-{name}.png"))
            .unwrap();
    }
}
