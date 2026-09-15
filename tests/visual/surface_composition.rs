use super::{harness::Harness, reference, theme};
use crate::{Element, MenuItem, NavigationItem, SelectOption, TextFieldVariant, Theme};
use iced::advanced::{Layout, Widget, layout, renderer, widget::Tree};
use iced::{Color, Length, Size, widget};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug, PartialEq)]
enum Message {
    Input(String),
    Action,
    Select(u8),
}
fn ui(
    content: impl Into<Element<'static, Message>>,
    size: Size,
    theme: Theme,
) -> Harness<'static, Message> {
    Harness::with_backend(
        content,
        size,
        theme,
        &std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into()),
    )
}
fn pattern(content: impl Into<Element<'static, Message>>) -> Element<'static, Message> {
    widget::stack![
        widget::row((0..12).map(|i| {
            widget::container(widget::space())
                .width(30)
                .height(Length::Fill)
                .style(move |theme: &Theme| widget::container::Style {
                    background: Some(
                        if i % 2 == 0 {
                            theme.colors.surface_container_highest
                        } else {
                            theme.colors.secondary_container
                        }
                        .into(),
                    ),
                    ..Default::default()
                })
                .into()
        })),
        widget::container(content).padding(24).width(Length::Fill),
    ]
    .width(360)
    .height(120)
    .into()
}
fn field(kind: &str, populated: bool, disabled: bool) -> Element<'static, Message> {
    if kind == "select" {
        return crate::select(
            "Name",
            [SelectOption::new(0, "Value")],
            populated.then_some(0),
        )
        .on_select(Message::Select)
        .disabled(disabled)
        .width(312.)
        .into();
    }
    let mut field = crate::text_field("Name", if populated { "Value" } else { "" })
        .on_input(Message::Input)
        .disabled(disabled)
        .width(312);
    if kind == "error" {
        field = field.error("Needs attention");
    }
    field.into()
}
#[test]
fn outlined_fields_and_selects_preserve_parent_pixels_in_body_and_label_gap() {
    for dark in [false, true] {
        let background = ui(pattern(widget::space()), Size::new(360., 120.), theme(dark)).frame();
        for kind in ["field", "error", "select"] {
            for disabled in [false, true] {
                let mut view = ui(
                    pattern(field(kind, true, disabled)),
                    Size::new(360., 120.),
                    theme(dark),
                );
                view.at(0);
                if !disabled {
                    view.frame();
                    view.move_to((150., 60.));
                    view.down();
                    view.up();
                    view.at(400);
                    // Dismiss the select popup, retaining a focused trigger.
                    if kind == "select" {
                        view.event(iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
                            modified_key: iced::keyboard::Key::Named(
                                iced::keyboard::key::Named::Escape,
                            ),
                            physical_key: iced::keyboard::key::Physical::Unidentified(
                                iced::keyboard::key::NativeCode::Unidentified,
                            ),
                            location: iced::keyboard::Location::Standard,
                            modifiers: iced::keyboard::Modifiers::empty(),
                            text: None,
                            repeat: false,
                        }));
                        view.at(800);
                    }
                }
                let frame = view.frame();
                assert!(
                    frame.crop(400, 100, 160, 40) == background.crop(400, 100, 160, 40),
                    "transparent interior: {kind}, {dark}, {disabled}"
                );
                assert!(
                    frame.crop(74, 62, 4, 8) == background.crop(74, 62, 4, 8),
                    "real outline gap: {kind}, {dark}, {disabled}"
                );
            }
        }
    }
}
#[test]
fn explicit_and_filled_backgrounds_remain_intentional() {
    for dark in [false, true] {
        let color = Color::from_rgb8(72, 110, 140);
        let background = ui(pattern(widget::space()), Size::new(360., 120.), theme(dark)).frame();
        for filled in [false, true] {
            let mut view = ui(
                pattern(
                    crate::text_field("Name", "Value")
                        .on_input(Message::Input)
                        .variant(if filled {
                            TextFieldVariant::Filled
                        } else {
                            TextFieldVariant::Outlined
                        })
                        .background(color),
                ),
                Size::new(360., 120.),
                theme(dark),
            );
            let frame = view.frame();
            assert!(frame.crop(400, 100, 80, 30) != background.crop(400, 100, 80, 30));
            // An explicit fill is bounded by the field; it cannot paint the label's surroundings.
            assert!(frame.crop(74, 50, 4, 10) == background.crop(74, 50, 4, 10));
        }
    }
}
fn solid_icon() -> crate::Icon {
    crate::icon(iced::advanced::svg::Handle::from_memory(b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'><path d='M0 0H24V24H0Z'/></svg>".as_slice()))
}
#[test]
fn svg_foreground_alpha_matches_renderer_compositing_and_rebuilds() {
    for dark in [false, true] {
        let render = |alpha, svg| {
            let color = crate::theme::alpha(theme(dark).colors.on_surface, alpha);
            let content: Element<'static, Message> = if svg {
                solid_icon().into()
            } else {
                widget::container(widget::space())
                    .width(24)
                    .height(24)
                    .style(move |_| widget::container::Style {
                        background: Some(color.into()),
                        ..Default::default()
                    })
                    .into()
            };
            pattern(
                widget::container(content).style(move |_| widget::container::Style {
                    text_color: Some(color),
                    ..Default::default()
                }),
            )
        };
        let mut view = ui(render(1., true), Size::new(360., 120.), theme(dark));
        for alpha in [1., 0.38, 0., 0.12, 0.38, 1.] {
            view.rebuild(render(alpha, true));
            let icon = view.frame();
            let expected = ui(render(alpha, false), Size::new(360., 120.), theme(dark)).frame();
            // Both stripes cross the icon. Skip antialiasing at the edges; image
            // opacity and quad alpha take separate 8-bit rounding paths in tiny-skia.
            for x in [52, 68] {
                let a = icon.crop(x, 52, 8, 32);
                let b = expected.crop(x, 52, 8, 32);
                assert!(
                    a.pixels
                        .iter()
                        .zip(&b.pixels)
                        .all(|(a, b)| a.abs_diff(*b) <= if alpha == 0. { 0 } else { 2 }),
                    "alpha={alpha}, dark={dark}, icon={:?}, quad={:?}",
                    &a.pixels[..4],
                    &b.pixels[..4]
                );
            }
        }
    }
}
struct Probe(Rc<RefCell<Vec<Color>>>);
impl Widget<Message, Theme, iced::Renderer> for Probe {
    fn size(&self) -> Size<Length> {
        Size::new(24.into(), 24.into())
    }
    fn layout(
        &mut self,
        _: &mut Tree,
        _: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, 24, 24)
    }
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        self.0.borrow_mut().push(style.text_color);
        <crate::Icon as Widget<Message, Theme, iced::Renderer>>::draw(
            &solid_icon(),
            tree,
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }
}
#[test]
fn disabled_component_foregrounds_keep_alpha_and_do_not_activate() {
    for dark in [false, true] {
        for kind in ["field", "chip", "list", "navigation", "menu"] {
            let colors = Rc::new(RefCell::new(Vec::new()));
            let probe = Element::new(Probe(colors.clone()));
            let content: Element<'static, Message> = match kind {
                "field" => crate::text_field("Name", "Value")
                    .leading(probe)
                    .on_input(Message::Input)
                    .disabled(true)
                    .into(),
                "chip" => crate::assist_chip("Action")
                    .leading(probe)
                    .on_press(Message::Action)
                    .disabled(true)
                    .into(),
                "list" => crate::list_item("Action")
                    .overline("Overline")
                    .supporting_text("Supporting")
                    .leading(probe)
                    .on_press(Message::Action)
                    .disabled(true)
                    .into(),
                "navigation" => crate::navigation_bar(
                    [NavigationItem::new(0, "Action", probe).disabled(true)],
                    None,
                )
                .on_select(Message::Select)
                .into(),
                _ => crate::menu_button(probe, [MenuItem::new("Action", Message::Action)])
                    .disabled(true)
                    .into(),
            };
            let mut view = ui(pattern(content), Size::new(360., 120.), theme(dark));
            view.at(0);
            view.frame();
            assert!(!colors.borrow().is_empty());
            assert!(
                colors
                    .borrow()
                    .iter()
                    .all(|c| *c == crate::theme::alpha(theme(dark).colors.on_surface, 0.38)),
                "{kind} must inherit actual alpha"
            );
            view.move_to((50., 50.));
            view.down();
            view.up();
            view.at(500);
            assert!(view.messages.is_empty(), "disabled {kind}");
        }
        let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
        let mut view = ui(disabled_components(), Size::new(360., 460.), theme(dark));
        view.at(0);
        view.frame().write(&std::path::PathBuf::from(format!(
            "target/surface-composition-renderer/{backend}-{}.png",
            if dark { "dark" } else { "light" }
        )));
    }
}
#[test]
fn narrow_floating_labels_remain_clipped_during_focus_and_reversal() {
    for dark in [false, true] {
        let background = ui(pattern(widget::space()), Size::new(360., 120.), theme(dark)).frame();
        let mut view = ui(
            pattern(
                crate::text_field("A very long field label", "")
                    .leading(super::icon::Icon)
                    .width(124)
                    .on_input(Message::Input),
            ),
            Size::new(360., 120.),
            theme(dark),
        );
        view.at(0);
        view.frame();
        view.move_to((100., 60.));
        view.down();
        view.up();
        for time in [0, 40, 80, 120, 160, 200, 400] {
            if time == 120 {
                view.operate(&mut iced::advanced::widget::operation::focusable::unfocus::<()>());
                view.leave();
            }
            view.at(time);
            let frame = view.frame();
            assert!(
                frame.crop(300, 48, 200, 144) == background.crop(300, 48, 200, 144),
                "label outside narrow field at {time}ms"
            );
        }
    }
}

#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_surface_composition() {
    for dark in [false, true] {
        let mode = if dark { "dark" } else { "light" };
        for kind in ["field", "error", "select"] {
            for disabled in [false, true] {
                let mut view = Harness::new(
                    pattern(field(kind, true, disabled)),
                    Size::new(360., 120.),
                    theme(dark),
                );
                view.at(0);
                reference::check(
                    &format!("surface-composition/{mode}-{kind}-disabled-{disabled}"),
                    &view.frame(),
                );
            }
        }
        let mut view = Harness::new(
            pattern(field("field", false, false)),
            Size::new(360., 120.),
            theme(dark),
        );
        view.at(0);
        view.frame();
        view.move_to((150., 60.));
        view.down();
        view.up();
        for time in [0, 40, 80, 120, 200] {
            view.at(time);
            reference::check(
                &format!("surface-composition/{mode}-floating-{time:03}ms"),
                &view.frame(),
            );
        }
        let content = disabled_components();
        let mut view = Harness::new(content, Size::new(360., 460.), theme(dark));
        view.at(0);
        reference::check(
            &format!("surface-composition/{mode}-disabled-components"),
            &view.frame(),
        );
    }
}

fn disabled_components() -> Element<'static, Message> {
    let content = widget::column![
        crate::text_field("Name", "Value")
            .leading(super::icon::Icon)
            .trailing(super::icon::Icon)
            .disabled(true),
        crate::assist_chip("Action")
            .leading(super::icon::Icon)
            .disabled(true),
        crate::list_item("Headline")
            .overline("Overline")
            .supporting_text("Supporting")
            .leading(super::icon::Icon)
            .disabled(true),
        crate::menu("Menu", [MenuItem::new("Action", Message::Action)]).disabled(true),
        crate::navigation_bar(
            [NavigationItem::new(0, "Workspace", super::icon::Icon).disabled(true)],
            Some(0)
        )
        .on_select(Message::Select),
    ]
    .spacing(16);
    let content = widget::container(content)
        .padding(24)
        .width(360)
        .height(460)
        .style(|theme: &Theme| widget::container::Style {
            background: Some(theme.colors.surface_container_highest.into()),
            ..Default::default()
        });
    content.into()
}
