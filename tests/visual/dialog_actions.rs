use super::{harness::Harness, reference, theme};
use crate::{Element, TypeScale, button, dialog, focus, typography};
use iced::advanced::widget::{Operation, operation};
use iced::{Event, Length, Rectangle, Size, Vector, keyboard, mouse, widget};

#[derive(Clone, Debug, PartialEq)]
enum Message {
    Input(String),
    Select(u8),
    Cancel,
    Save,
}

fn scene(open: bool, stacked: bool, fields: usize, max_height: f32) -> Element<'static, Message> {
    let mut body = widget::column![
        typography("Workspace settings", TypeScale::HeadlineSmall),
        typography("Changes apply to this workspace.", TypeScale::BodyMedium),
    ]
    .spacing(16);
    for i in 0..fields {
        body = body.push(
            crate::text_field(&format!("Setting {}", i + 1), "Workspace value")
                .id(widget::Id::from(format!("setting-{i}")))
                .on_input(Message::Input)
                .supporting_text("A full-width field with room for its scrollbar."),
        );
    }
    body = body.push(
        crate::select(
            "Workspace access",
            [
                crate::SelectOption::new(0, "Viewer"),
                crate::SelectOption::new(1, "Editor"),
            ],
            Some(1),
        )
        .on_select(Message::Select),
    );
    let panel = dialog(body)
        .actions(
            widget::container(
                widget::row![
                    button("Cancel")
                        .variant(crate::ButtonVariant::Text)
                        .on_press(Message::Cancel),
                    button("Save changes").on_press(Message::Save),
                ]
                .spacing(8)
                .wrap(),
            )
            .id("settings-footer"),
        )
        .max_height(max_height)
        .initial_focus(0)
        .on_dismiss(Message::Cancel);
    let background = widget::space().width(Length::Fill).height(Length::Fill);
    focus::scope(if stacked {
        dialog::stack(background, [(panel, open)])
    } else {
        dialog::modal(background, panel, open)
    })
}
fn ui(
    open: bool,
    stacked: bool,
    fields: usize,
    max_height: f32,
    size: Size,
    dark: bool,
) -> Harness<'static, Message> {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    let mut ui = Harness::with_backend(
        scene(open, stacked, fields, max_height),
        size,
        theme(dark),
        &backend,
    );
    ui.at(0);
    ui.frame();
    ui
}
fn bounds(ui: &mut Harness<'_, Message>, id: &str) -> Rectangle {
    let mut query = iced_test::Selector::find(widget::Id::from(id.to_owned()));
    ui.operate(&mut operation::black_box(&mut query));
    match query.finish() {
        operation::Outcome::Some(Some(found)) => found.visible_bounds().expect("visible widget"),
        _ => panic!("Missing {id}"),
    }
}
#[derive(Default)]
struct Scrolls(Vec<(Rectangle, Rectangle, Vector)>);
impl Operation for Scrolls {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }
    fn scrollable(
        &mut self,
        _: Option<&widget::Id>,
        bounds: Rectangle,
        content: Rectangle,
        translation: Vector,
        _: &mut dyn operation::scrollable::Scrollable,
    ) {
        self.0.push((bounds, content, translation));
    }
}
fn scroll(ui: &mut Harness<'_, Message>) -> (Rectangle, Rectangle, Vector) {
    let mut query = Scrolls::default();
    ui.operate(&mut query);
    assert_eq!(
        query.0.len(),
        1,
        "One body scroller; actions need no nested scroller"
    );
    query.0[0]
}
fn to_bottom(ui: &mut Harness<'_, Message>) {
    let (body, _, _) = scroll(ui);
    ui.move_to(body.center());
    ui.event(Event::Mouse(mouse::Event::WheelScrolled {
        delta: mouse::ScrollDelta::Pixels { x: 0., y: -4000. },
    }));
    ui.frame();
}
fn key(ui: &mut Harness<'_, Message>, name: keyboard::key::Named) {
    let key = keyboard::Key::Named(name);
    let physical_key =
        keyboard::key::Physical::Unidentified(keyboard::key::NativeCode::Unidentified);
    ui.event(Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key.clone(),
        physical_key,
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::empty(),
        text: None,
        repeat: false,
    }));
    ui.event(Event::Keyboard(keyboard::Event::KeyReleased {
        key: key.clone(),
        modified_key: key,
        physical_key,
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::empty(),
    }));
}

#[test]
fn dialog_footer_stays_fixed_while_body_scrolls_in_both_hosts() {
    for stacked in [false, true] {
        let mut ui = ui(true, stacked, 8, 400., Size::new(720., 700.), false);
        let footer = bounds(&mut ui, "settings-footer");
        let (body, content, _) = scroll(&mut ui);
        assert!(content.height > body.height);
        assert_eq!(body.height + 24. + footer.height + 48., 400.);
        assert_eq!(body.y + body.height + 24., footer.y);
        let field = bounds(&mut ui, "setting-0");
        assert!(
            field.x + field.width <= body.x + body.width - 12.,
            "Scrollbar has its own gutter"
        );
        to_bottom(&mut ui);
        assert!(scroll(&mut ui).2.y > 0.);
        assert_eq!(bounds(&mut ui, "settings-footer"), footer);
        ui.click("Save changes");
        assert_eq!(ui.messages, [Message::Save]);
    }
}

#[test]
fn dialog_height_cap_shrinks_short_content_and_fits_resized_windows() {
    let mut short = ui(true, false, 0, 600., Size::new(720., 800.), false);
    let (body, _, _) = scroll(&mut short);
    let footer = bounds(&mut short, "settings-footer");
    assert!(footer.y + footer.height - body.y + 48. < 600.);
    for stacked in [false, true] {
        let mut ui = ui(true, stacked, 8, 600., Size::new(720., 800.), false);
        to_bottom(&mut ui);
        for size in [
            Size::new(320., 280.),
            Size::new(260., 280.),
            Size::new(720., 460.),
            Size::new(720., 800.),
        ] {
            ui.resize(size);
            ui.frame();
            let (body, content, translation) = scroll(&mut ui);
            let footer = bounds(&mut ui, "settings-footer");
            assert!(body.y >= 24.);
            assert!(footer.y + footer.height <= size.height - 24.);
            assert!(footer.y + footer.height - body.y + 48. <= 600.);
            assert!(translation.y > 0. && translation.y <= content.height - body.height);
            assert!(ui.find("Cancel").width > 0. && ui.find("Save changes").width > 0.);
        }
        ui.click("Cancel");
        assert_eq!(ui.messages, [Message::Cancel]);
    }
}

#[test]
fn dialog_footer_preserves_body_first_tab_order_and_focus_reveal() {
    for stacked in [false, true] {
        let mut ui = ui(true, stacked, 8, 400., Size::new(720., 550.), false);
        let footer = bounds(&mut ui, "settings-footer");
        for i in 1..8 {
            key(&mut ui, keyboard::key::Named::Tab);
            let field = bounds(&mut ui, &format!("setting-{i}"));
            let body = scroll(&mut ui).0;
            assert!(field.y >= body.y && field.y + field.height <= body.y + body.height);
            assert_eq!(bounds(&mut ui, "settings-footer"), footer);
        }
        key(&mut ui, keyboard::key::Named::Tab); // Select
        key(&mut ui, keyboard::key::Named::Tab); // Cancel
        key(&mut ui, keyboard::key::Named::Enter);
        key(&mut ui, keyboard::key::Named::Tab); // Save
        key(&mut ui, keyboard::key::Named::Enter);
        assert_eq!(ui.messages, [Message::Cancel, Message::Save]);
    }
}

#[test]
fn dialog_footer_preserves_nested_select_and_escape_routing() {
    for stacked in [false, true] {
        let mut ui = ui(true, stacked, 8, 400., Size::new(720., 550.), false);
        to_bottom(&mut ui);
        ui.click("Editor");
        ui.at(300);
        ui.click("Viewer");
        assert_eq!(ui.messages, [Message::Select(0)]);
        ui.messages.clear();
        ui.click("Editor");
        key(&mut ui, keyboard::key::Named::Escape);
        assert!(
            ui.messages.is_empty(),
            "First Escape belongs to the dropdown"
        );
        key(&mut ui, keyboard::key::Named::Escape);
        assert_eq!(ui.messages, [Message::Cancel]);
    }
}

#[test]
fn dialog_footer_is_inert_during_retained_exit() {
    for stacked in [false, true] {
        let mut ui = ui(true, stacked, 8, 400., Size::new(720., 550.), false);
        let save = ui.find("Save changes").center();
        ui.rebuild(scene(false, stacked, 8, 400.));
        ui.at(1);
        ui.move_to(save);
        ui.down();
        ui.up();
        key(&mut ui, keyboard::key::Named::Enter);
        assert!(ui.messages.is_empty());
        ui.at(500);
        ui.frame();
    }
}

#[test]
fn full_screen_dialog_can_add_a_footer_without_moving_its_header_into_the_scroller() {
    let full: crate::Dialog<'_, Message> = crate::full_screen_dialog(
        "Edit workspace",
        widget::column![
            widget::space().height(800),
            button("Last field").on_press(Message::Save)
        ],
    )
    .on_dismiss(Message::Cancel)
    .into();
    let mut ui = Harness::new(
        dialog::modal(
            widget::space().width(Length::Fill).height(Length::Fill),
            full.actions(button("Save changes").on_press(Message::Save))
                .max_height(200),
            true,
        ),
        Size::new(390., 500.),
        theme(false),
    );
    ui.at(0);
    ui.frame();
    let header = ui.find("Edit workspace");
    let footer = ui.find("Save changes");
    assert!(
        footer.y > 400.,
        "Full-screen dialogs ignore the basic panel height cap"
    );
    to_bottom(&mut ui);
    assert_eq!(ui.find("Edit workspace"), header);
    assert_eq!(ui.find("Save changes"), footer);
    ui.click("Last field");
    ui.click("Save changes");
    assert_eq!(ui.messages, [Message::Save, Message::Save]);
}

#[test]
#[ignore = "canonical font/raster reference environment required"]
fn visual_references_dialog_fixed_actions() {
    for dark in [false, true] {
        let mode = if dark { "dark" } else { "light" };
        let prefix = format!("dialog-fixed-actions-{mode}");
        let mut ui = ui(false, false, 8, 400., Size::new(640., 540.), dark);
        ui.rebuild(scene(true, false, 8, 400.));
        ui.at(0);
        for (time, name) in [
            (149, "01-before-actions"),
            (225, "02-entering"),
            (275, "02b-actions-fade"),
            (600, "03-open"),
        ] {
            ui.at(time);
            reference::check(&format!("{prefix}/{name}"), &ui.frame());
        }
        to_bottom(&mut ui);
        ui.leave();
        ui.at(900);
        reference::check(&format!("{prefix}/04-scrolled"), &ui.frame());
        ui.resize(Size::new(320., 280.));
        ui.at(1000);
        reference::check(&format!("{prefix}/05-narrow-short"), &ui.frame());
        ui.resize(Size::new(260., 280.));
        ui.at(1100);
        let cancel = ui.find("Cancel");
        let save = ui.find("Save changes");
        assert!(
            save.y >= cancel.y + cancel.height,
            "Narrow actions wrap onto separate lines"
        );
        reference::check(&format!("{prefix}/06-wrapped-actions"), &ui.frame());
    }
}
