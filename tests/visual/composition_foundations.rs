use super::{harness::Harness, reference, theme};
use crate::{Element, Tab, TabVariant, Theme, list, list_item, tabs};
use iced::{Background, Color, Event, Length, Size, mouse, widget};

#[derive(Clone, Debug, PartialEq)]
enum Message {
    Select(u8),
    Action,
}
fn ui(content: impl Into<Element<'static, Message>>, dark: bool) -> Harness<'static, Message> {
    Harness::with_backend(
        content,
        Size::new(440., 280.),
        theme(dark),
        &std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into()),
    )
}
fn host(content: impl Into<Element<'static, Message>>) -> Element<'static, Message> {
    widget::stack![
        widget::row((0..11).map(|i| {
            widget::container(widget::space())
                .width(40)
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
    .width(440)
    .height(280)
    .into()
}
fn tab_strip(selected: u8) -> crate::Tabs<'static, u8, Message> {
    tabs(
        [
            Tab::new(0, "Overview"),
            Tab::new(1, "Files"),
            Tab::new(2, "Archived").disabled(true),
        ],
        Some(selected),
    )
    .on_select(Message::Select)
}
fn rows() -> widget::Container<'static, Message, Theme> {
    list([
        list_item("Design notes")
            .supporting_text("Shared with your team")
            .on_press(Message::Action)
            .into(),
        list_item("Archived workspace")
            .supporting_text("Unavailable")
            .disabled(true)
            .into(),
    ])
}
fn gradient(theme: &Theme) -> Background {
    iced::gradient::Linear::new(iced::Radians(std::f32::consts::FRAC_PI_2))
        .add_stop(0., theme.colors.primary_container)
        .add_stop(1., theme.colors.tertiary_container)
        .into()
}
#[test]
fn tabs_and_lists_preserve_the_parent_and_keep_controlled_actions() {
    for dark in [false, true] {
        let empty = ui(host(widget::space()), dark).frame();
        let view = || host(widget::column![tab_strip(0), rows()].spacing(16));
        let mut content = ui(view(), dark);
        content.at(0);
        let frame = content.frame();
        for (x, y, w, h) in [(50, 50, 200, 4), (50, 184, 16, 280)] {
            assert!(
                frame.crop(x, y, w, h) == empty.crop(x, y, w, h),
                "parent pixels {dark}"
            );
        }
        content.click("Files");
        assert_eq!(content.messages, [Message::Select(1)]);
        content.messages.clear();
        content.rebuild(host(widget::column![tab_strip(1), rows()].spacing(16)));
        content.at(400);
        content.click("Design notes");
        assert_eq!(content.messages, [Message::Action]);
        content.messages.clear();
        content.click("Archived");
        content.click("Archived workspace");
        assert!(content.messages.is_empty());
    }
}
#[test]
fn explicit_tab_fill_covers_the_viewport_and_stays_fixed_while_scrolling() {
    for dark in [false, true] {
        let strip = tabs(
            (0..12).map(|i| Tab::new(i, format!("Destination {i}"))),
            Some(0),
        )
        .scrollable(true)
        .disabled(true)
        .background_with(gradient)
        .width(392);
        let mut view = ui(host(strip), dark);
        view.at(0);
        let before = view.frame();
        view.move_to((180., 45.));
        view.event(Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Pixels { x: -200., y: 0. },
        }));
        view.at(200);
        let after = view.frame();
        assert!(before != after, "the strip actually scrolls");
        assert!(
            before.crop(50, 50, 780, 4) == after.crop(50, 50, 780, 4),
            "gradient belongs to the viewport"
        );
        let expected = ui(
            host(
                widget::container(widget::space())
                    .width(392)
                    .height(60)
                    .style(|theme: &Theme| widget::container::Style {
                        background: Some(gradient(theme)),
                        ..Default::default()
                    }),
            ),
            dark,
        )
        .frame();
        assert!(after.crop(50, 50, 780, 4) == expected.crop(50, 50, 780, 4));
        assert!(view.messages.is_empty());
    }
}
#[test]
fn tab_and_native_list_background_overrides_accept_transparency_and_colors() {
    for dark in [false, true] {
        let plain = ui(
            host(widget::column![tab_strip(0), rows()].spacing(16)),
            dark,
        )
        .frame();
        let transparent = ui(
            host(
                widget::column![
                    tab_strip(0).background(Color::TRANSPARENT),
                    rows().style(|theme: &Theme| widget::container::Style {
                        background: Some(Color::TRANSPARENT.into()),
                        text_color: Some(theme.colors.on_surface),
                        ..Default::default()
                    })
                ]
                .spacing(16),
            ),
            dark,
        )
        .frame();
        assert!(plain == transparent);
        let filled = ui(
            host(
                widget::column![
                    tab_strip(0).background_with(|theme| theme.colors.surface),
                    rows().style(|theme: &Theme| widget::container::Style {
                        background: Some(theme.colors.surface.into()),
                        text_color: Some(theme.colors.on_surface),
                        ..Default::default()
                    })
                ]
                .spacing(16),
            ),
            dark,
        )
        .frame();
        assert!(filled.crop(50, 50, 200, 4) != plain.crop(50, 50, 200, 4));
        assert!(filled.crop(50, 184, 16, 280) != plain.crop(50, 184, 16, 280));
    }
}
#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_composition_foundations() {
    for dark in [false, true] {
        let mode = if dark { "dark" } else { "light" };
        for secondary in [false, true] {
            let view = |selected| {
                host(
                    widget::column![
                        tab_strip(selected).variant(if secondary {
                            TabVariant::Secondary
                        } else {
                            TabVariant::Primary
                        }),
                        rows()
                    ]
                    .spacing(16),
                )
            };
            let mut ui = Harness::new(view(0), Size::new(440., 280.), theme(dark));
            ui.at(0);
            reference::check(
                &format!("composition-foundations/{mode}-secondary-{secondary}-rest"),
                &ui.frame(),
            );
            let target = ui.find("Files");
            ui.move_to(target.center());
            ui.at(150);
            ui.down();
            for time in [200, 375, 600] {
                ui.at(time);
                reference::check(
                    &format!("composition-foundations/{mode}-secondary-{secondary}-held-{time}"),
                    &ui.frame(),
                );
            }
            ui.up();
            ui.rebuild(view(1));
            ui.at(700);
            reference::check(
                &format!("composition-foundations/{mode}-secondary-{secondary}-selection-moving"),
                &ui.frame(),
            );
            ui.at(1100);
            ui.leave();
            ui.at(1300);
            reference::check(
                &format!("composition-foundations/{mode}-secondary-{secondary}-selected"),
                &ui.frame(),
            );
        }
        for fill in [false, true] {
            let strip = if fill {
                tab_strip(0).background_with(gradient)
            } else {
                tab_strip(0).background_with(|theme| theme.colors.surface)
            };
            let content = host(
                widget::column![
                    strip,
                    rows().style(move |theme: &Theme| widget::container::Style {
                        background: Some(if fill {
                            gradient(theme)
                        } else {
                            theme.colors.surface.into()
                        }),
                        text_color: Some(theme.colors.on_surface),
                        ..Default::default()
                    })
                ]
                .spacing(16),
            );
            let mut ui = Harness::new(content, Size::new(440., 280.), theme(dark));
            ui.at(0);
            reference::check(
                &format!("composition-foundations/{mode}-gradient-{fill}"),
                &ui.frame(),
            );
        }
    }
}
