mod adaptive;
mod baseline;
mod completion;
mod compositing_probe;
mod composition_foundations;
mod desktop;
mod desktop_completion;
mod desktop_fidelity;
mod desktop_finish;
mod dialog_actions;
mod dialog_regressions;
mod fidelity;
mod loading_progress;
mod modal_activity;
mod pickers;
mod polish;
mod surface_composition;
use crate::{Element, Theme, TypeScale, snackbar, typography};
use iced::{Length, Size, widget};
use std::time::Instant;
mod harness;
mod reference;
mod search;
mod sheets;
mod values;
mod variants;
use crate::{
    ButtonVariant, MenuItem, NavigationItem, SelectOption, SurfaceVariant, Tab, TabVariant,
    app_bar, badge, button, checkbox, chip, divider, icon_button, list, list_item, menu,
    navigation_rail, radio, select, surface, switch, tabs, text_field, tooltip,
};
use harness::Harness;
use iced::{Event, mouse, window};

fn with_time<T>(now: Instant, f: impl FnOnce() -> T) -> T {
    crate::motion::with_time(now, f)
}

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Action,
    Toggle(bool),
    Select(u8),
    Input(String),
    Dismiss,
    Value(f32),
    Segments(crate::SegmentSelection<u8>),
}

fn notice(with_text: bool) -> Element<'static, Message> {
    snackbar::host(
        widget::container(typography(
            if with_text {
                "Underlying footer text crosses behind Workspace unpinned"
            } else {
                ""
            },
            TypeScale::BodyMedium,
        ))
        .align_bottom(Length::Fill)
        .width(Length::Fill)
        .padding([40, 16]),
        Some(snackbar("Workspace unpinned").persistent()),
    )
}
#[test]
fn snackbar_surface_occludes_background_text() {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    for dark in [false, true] {
        let theme = if dark { Theme::dark() } else { Theme::light() };
        let mut covered = Harness::with_backend(
            notice(true),
            Size::new(600.0, 400.0),
            theme.clone(),
            &backend,
        );
        let mut empty =
            Harness::with_backend(notice(false), Size::new(600.0, 400.0), theme, &backend);
        let covered = covered.frame().crop(64, 680, 1072, 72);
        let empty = empty.frame().crop(64, 680, 1072, 72);
        covered.write(&std::path::PathBuf::from(format!(
            "target/visual-report/snackbar-occlusion-{backend}-{dark}.png"
        )));
        assert!(
            covered == empty,
            "An opaque snackbar must hide all background text ({backend}, dark={dark})"
        );
    }
}

fn theme(dark: bool) -> Theme {
    if dark { Theme::dark() } else { Theme::light() }
}
fn themed_name(name: &str, dark: bool) -> String {
    format!("{name}/{}", if dark { "dark" } else { "light" })
}
fn padded(content: impl Into<Element<'static, Message>>) -> Element<'static, Message> {
    widget::container(content).padding(16).into()
}
fn capture(ui: &mut Harness<'_, Message>, case: &str, label: &str) {
    reference::check(&format!("{case}/{label}"), &ui.frame());
}
mod icon;
fn icon() -> Element<'static, Message> {
    icon::Icon.into()
}

fn control(kind: &str, selected: bool, disabled: bool) -> Element<'static, Message> {
    let control: Element<'static, Message> = match kind {
        "switch" => switch(selected)
            .label("Workspace updates")
            .on_toggle(Message::Toggle)
            .disabled(disabled)
            .into(),
        "checkbox" => checkbox(selected)
            .label("Workspace updates")
            .on_toggle(Message::Toggle)
            .disabled(disabled)
            .into(),
        "radio" => radio("Workspace updates", 1, selected.then_some(1))
            .on_select(Message::Select)
            .disabled(disabled)
            .into(),
        _ => unreachable!(),
    };
    padded(control)
}

/// Every time here is relative to one virtual origin; PNG I/O cannot advance it.
#[test]
#[ignore = "reference suite; run with --ignored (CI has a dedicated job)"]
fn visual_references_selection_animations() {
    for kind in ["switch", "checkbox", "radio"] {
        for dark in [false, true] {
            for initial in [false, true] {
                let case = themed_name(
                    &format!("{kind}-{}", if initial { "on" } else { "off" }),
                    dark,
                );
                let mut ui = Harness::new(
                    control(kind, initial, false),
                    Size::new(280.0, 80.0),
                    theme(dark),
                );
                ui.at(0);
                capture(&mut ui, &case, "00-idle");
                ui.move_to((40.0, 40.0));
                ui.at(75);
                capture(&mut ui, &case, "01-hover-75ms");
                ui.at(150);
                capture(&mut ui, &case, "02-hover-settled");
                ui.down();
                let press_times: &[u64] = if kind == "switch" {
                    &[0, 50, 100, 150]
                } else {
                    &[0, 50, 100, 150, 250, 350, 450]
                };
                for &elapsed in press_times {
                    ui.at(150 + elapsed);
                    capture(&mut ui, &case, &format!("03-press-{elapsed:03}ms"));
                }
                let held = ui.frame();
                assert_eq!(ui.at(650), window::RedrawRequest::Wait);
                assert!(held == ui.frame(), "held state stays visually stable");
                assert!(ui.messages.is_empty(), "holding must not toggle");
                capture(&mut ui, &case, "04-held-500ms");
                ui.up();
                let next = if kind == "radio" { true } else { !initial };
                assert_eq!(
                    ui.messages.len(),
                    usize::from(!(kind == "radio" && initial))
                );
                ui.rebuild(control(kind, next, false));
                for elapsed in [0, 50, 100, 200, 350] {
                    ui.at(650 + elapsed);
                    capture(&mut ui, &case, &format!("05-release-{elapsed:03}ms"));
                }
                ui.leave();
                ui.at(1200);
                capture(&mut ui, &case, "06-settled");
                let settled = ui.frame();
                assert_eq!(ui.at(3000), window::RedrawRequest::Wait);
                assert!(settled == ui.frame());
                // Rebuilding disabled during a press must remove feedback and cancel release.
                ui.move_to((40.0, 40.0));
                ui.down();
                ui.rebuild(control(kind, next, true));
                ui.at(3200);
                capture(&mut ui, &case, "07-disabled-during-press");
                ui.messages.clear();
                ui.up();
                assert!(ui.messages.is_empty());

                if kind != "switch" {
                    let mut quick = Harness::new(
                        control(kind, initial, false),
                        Size::new(280.0, 80.0),
                        theme(dark),
                    );
                    quick.at(0);
                    quick.frame();
                    quick.move_to((40.0, 40.0));
                    quick.at(150);
                    quick.down();
                    quick.up();
                    quick.rebuild(control(kind, next, false));
                    for elapsed in [0, 100, 225, 300, 375] {
                        quick.at(150 + elapsed);
                        capture(&mut quick, &case, &format!("08-quick-tap-{elapsed:03}ms"));
                    }
                }
            }
        }
    }
}

#[test]
fn selection_ripple_expands_holds_and_only_fades_after_release() {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    for kind in ["checkbox", "radio"] {
        for dark in [false, true] {
            let mut ui = Harness::with_backend(
                control(kind, false, false),
                Size::new(280.0, 80.0),
                theme(dark),
                &backend,
            );
            ui.at(0);
            ui.frame();
            ui.move_to((40.0, 40.0));
            ui.at(150);
            let hovered = ui.frame();
            ui.down();
            ui.at(250);
            let early = ui.frame();
            assert_eq!(ui.at(400), window::RedrawRequest::NextFrame);
            let middle = ui.frame();
            assert_eq!(ui.at(600), window::RedrawRequest::Wait);
            let held = ui.frame();
            assert!(
                early != middle && middle != held,
                "growth remains visible beyond 200 ms"
            );
            assert!(
                held != hovered,
                "a held press retains its stronger feedback"
            );
            assert_eq!(ui.at(2000), window::RedrawRequest::Wait);
            assert!(
                held == ui.frame(),
                "a long hold stays visible without scheduling frames"
            );
            assert!(ui.messages.is_empty());
            ui.up();
            assert_eq!(
                ui.messages,
                [if kind == "radio" {
                    Message::Select(1)
                } else {
                    Message::Toggle(true)
                }]
            );
            assert!(
                held == ui.frame(),
                "release begins fading without snapping the halo"
            );
            ui.at(2075);
            let fading = ui.frame();
            assert!(fading != held && fading != hovered);
            assert_eq!(ui.at(2150), window::RedrawRequest::Wait);
            assert!(hovered == ui.frame());
        }
    }
}

#[test]
fn selection_ripple_preserves_hover_circle() {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    for kind in ["checkbox", "radio"] {
        for selected in [false, true] {
            for dark in [false, true] {
                let mut ui = Harness::with_backend(
                    control(kind, selected, false),
                    Size::new(280.0, 80.0),
                    theme(dark),
                    &backend,
                );
                ui.at(0);
                let idle = ui.frame();
                ui.move_to((40.0, 40.0));
                ui.at(150);
                let hovered = ui.frame();
                // The 2x fixture's halo is centered at (80, 80). This patch
                // lies inside the hover circle, outside the early ripple.
                let outer_hover = hovered.crop(76, 42, 8, 2);
                assert!(outer_hover != idle.crop(76, 42, 8, 2));
                ui.down();
                for time in [250, 300] {
                    ui.at(time);
                    let growing = ui.frame();
                    assert!(
                        outer_hover == growing.crop(76, 42, 8, 2),
                        "pressing must preserve hover outside the growing ripple ({kind}, selected={selected}, dark={dark}, time={time})"
                    );
                    assert!(
                        hovered.crop(78, 54, 4, 4) != growing.crop(78, 54, 4, 4),
                        "the ripple adds feedback inside the existing hover circle"
                    );
                }
                assert!(ui.messages.is_empty());
                ui.at(650);
                assert!(outer_hover != ui.frame().crop(76, 42, 8, 2));
                ui.up();
                assert_eq!(ui.at(800), window::RedrawRequest::Wait);
                assert!(
                    hovered == ui.frame(),
                    "release returns to the same hover appearance"
                );
                ui.leave();
                assert_eq!(ui.at(950), window::RedrawRequest::Wait);
                assert!(idle == ui.frame(), "leaving still clears hover and ripple");
            }
        }
    }
}

#[test]
fn quick_checkbox_ripple_survives_release_and_cancels_stale_deadlines() {
    let mut ui = Harness::new(
        control("checkbox", false, false),
        Size::new(280.0, 80.0),
        Theme::light(),
    );
    ui.at(0);
    let idle = ui.frame();
    ui.move_to((40.0, 40.0));
    ui.at(150);
    let hovered = ui.frame();
    ui.down();
    ui.up();
    assert_eq!(
        ui.messages,
        [Message::Toggle(true)],
        "visual minimum never delays the action"
    );
    ui.at(375);
    assert!(
        hovered != ui.frame(),
        "a quick click remains visible through the minimum interval"
    );
    assert_eq!(ui.at(525), window::RedrawRequest::Wait);
    assert!(hovered == ui.frame());

    ui.down();
    ui.up();
    ui.event(Event::Window(window::Event::Unfocused));
    assert_eq!(ui.at(675), window::RedrawRequest::Wait);
    assert!(
        idle == ui.frame(),
        "window deactivation cancels the pending minimum interval"
    );

    ui.move_to((40.0, 40.0));
    ui.down();
    ui.up();
    ui.at(775);
    ui.down();
    assert_eq!(ui.at(1225), window::RedrawRequest::Wait);
    let held = ui.frame();
    assert!(
        held != hovered,
        "the preceding click's deadline cannot fade a new hold"
    );
    assert_eq!(ui.at(2000), window::RedrawRequest::Wait);
    assert!(held == ui.frame());
    ui.rebuild(control("checkbox", false, true));
    ui.messages.clear();
    ui.up();
    assert_eq!(ui.at(2200), window::RedrawRequest::Wait);
    assert!(ui.messages.is_empty());
    let mut disabled = Harness::new(
        control("checkbox", false, true),
        Size::new(280.0, 80.0),
        Theme::light(),
    );
    assert!(
        ui.frame() == disabled.frame(),
        "disabling immediately clears all ripple feedback"
    );
}

#[test]
fn checkbox_ripple_respects_zero_motion_durations() {
    let mut theme = Theme::light();
    theme.motion.short = std::time::Duration::ZERO;
    theme.motion.ripple_expand = std::time::Duration::ZERO;
    let mut ui = Harness::new(
        control("checkbox", false, false),
        Size::new(280.0, 80.0),
        theme,
    );
    ui.at(0);
    ui.frame(); // Applies the theme's motion settings to the widget.
    ui.move_to((40.0, 40.0));
    ui.at(0);
    let hovered = ui.frame();
    ui.down();
    assert_eq!(ui.at(0), window::RedrawRequest::Wait);
    assert!(hovered != ui.frame());
    ui.up();
    assert_eq!(ui.messages, [Message::Toggle(true)]);
    assert_eq!(ui.at(0), window::RedrawRequest::Wait);
    assert!(hovered == ui.frame());
}

#[test]
fn virtual_time_replays_identical_switch_frames() {
    let sequence = || {
        let mut ui = Harness::new(
            control("switch", false, false),
            Size::new(280.0, 80.0),
            Theme::light(),
        );
        ui.at(0);
        ui.frame();
        ui.move_to((40.0, 40.0));
        ui.down();
        [0, 50, 100, 150, 500].map(|time| {
            ui.at(time);
            ui.frame()
        })
    };
    let first = sequence();
    let second = sequence();
    assert!(
        first == second,
        "separate wall-clock origins must produce identical pixels"
    );
    assert!(first[0] != first[1] && first[1] != first[2] && first[2] != first[3]);
    assert!(first[3] == first[4]);
}

#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_actions() {
    for dark in [false, true] {
        for (name, variant) in [
            ("filled", ButtonVariant::Filled),
            ("outlined", ButtonVariant::Outlined),
            ("text", ButtonVariant::Text),
            ("tonal", ButtonVariant::Tonal),
        ] {
            let case = themed_name(&format!("button-{name}"), dark);
            let make = |disabled| {
                padded(
                    button("Save workspace")
                        .variant(variant)
                        .on_press(Message::Action)
                        .disabled(disabled),
                )
            };
            let mut ui = Harness::new(make(false), Size::new(240.0, 80.0), theme(dark));
            ui.at(0);
            capture(&mut ui, &case, "00-idle");
            ui.move_to((60.0, 36.0));
            ui.at(75);
            capture(&mut ui, &case, "01-hover-75ms");
            ui.at(150);
            capture(&mut ui, &case, "02-hover");
            ui.down();
            capture(&mut ui, &case, "03-held");
            ui.at(300);
            ui.up();
            for time in [0, 75, 150] {
                ui.at(300 + time);
                capture(&mut ui, &case, &format!("04-release-{time:03}ms"));
            }
            ui.rebuild(make(true));
            ui.at(600);
            capture(&mut ui, &case, "05-disabled");
        }
        let mut rows = widget::Column::new().spacing(12);
        for selected in [false, true] {
            let mut icons = widget::Row::new().spacing(12);
            for variant in [
                ButtonVariant::Text,
                ButtonVariant::Outlined,
                ButtonVariant::Filled,
                ButtonVariant::Tonal,
            ] {
                icons = icons.push(
                    icon_button(icon())
                        .variant(variant)
                        .selected(selected)
                        .on_press(Message::Action),
                );
                icons = icons.push(icon_button(icon()).variant(variant).selected(selected));
            }
            rows = rows.push(icons);
        }
        rows = rows.push(
            widget::row![
                chip("Filter", false).on_press(Message::Action),
                chip("Selected", true).on_press(Message::Action),
                chip("Disabled", false)
            ]
            .spacing(12),
        );
        let mut ui = Harness::new(padded(rows), Size::new(480.0, 220.0), theme(dark));
        ui.at(0);
        capture(
            &mut ui,
            &themed_name("icon-buttons-chips", dark),
            "00-variants",
        );
        ui.move_to((36.0, 36.0));
        ui.down();
        ui.at(75);
        capture(&mut ui, &themed_name("icon-buttons-chips", dark), "01-held");
    }
}

#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_tabs_and_fields() {
    for dark in [false, true] {
        for variant in [TabVariant::Primary, TabVariant::Secondary] {
            let case = themed_name(
                if variant == TabVariant::Primary {
                    "tabs-primary"
                } else {
                    "tabs-secondary"
                },
                dark,
            );
            let make = |selected| {
                padded(
                    tabs(
                        [
                            Tab::new(1, "Files").icon(icon()),
                            Tab::new(2, "Updates").icon(icon()).badge(3),
                            Tab::new(3, "Archived").icon(icon()).disabled(true),
                        ],
                        Some(selected),
                    )
                    .variant(variant)
                    .on_select(Message::Select),
                )
            };
            let mut ui = Harness::new(make(1), Size::new(360.0, 110.0), theme(dark));
            ui.at(0);
            capture(&mut ui, &case, "00-first");
            ui.click("Updates");
            ui.rebuild(make(2));
            ui.leave();
            for time in [0, 50, 100, 200] {
                ui.at(time);
                capture(&mut ui, &case, &format!("01-slide-{time:03}ms"));
            }
            assert_eq!(ui.at(1000), window::RedrawRequest::Wait);
        }
        let case = themed_name("floating-label", dark);
        let make = |value| {
            padded(
                text_field("Workspace name", value)
                    .on_input(Message::Input)
                    .supporting_text("Your team's shared space"),
            )
        };
        let mut ui = Harness::new(make(""), Size::new(320.0, 120.0), theme(dark));
        ui.at(0);
        capture(&mut ui, &case, "00-empty");
        // Programmatic value changes exercise label movement without relying on
        // the native text editor's independent wall-clock caret blink.
        ui.rebuild(make("Studio"));
        for time in [0, 50, 100, 200] {
            ui.at(time);
            capture(&mut ui, &case, &format!("01-float-{time:03}ms"));
        }
        ui.rebuild(make(""));
        for time in [200, 250, 300, 400] {
            ui.at(time);
            capture(&mut ui, &case, &format!("02-return-{:03}ms", time - 200));
        }
        let mut ui = Harness::new(
            padded(
                widget::column![
                    text_field("Workspace", "Studio").on_input(Message::Input),
                    text_field("Email", "invalid")
                        .on_input(Message::Input)
                        .error("Enter a valid email address. Long supporting text wraps."),
                    text_field("Managed", "Read only"),
                    text_field("Password", "secret")
                        .on_input(Message::Input)
                        .secure(true),
                ]
                .spacing(12),
            ),
            Size::new(320.0, 430.0),
            theme(dark),
        );
        ui.at(0);
        capture(&mut ui, &themed_name("text-fields", dark), "00-states");
    }
}

#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_foundations() {
    for dark in [false, true] {
        let mut types = widget::Column::new().spacing(8);
        for (name, role) in [
            ("Display Large", TypeScale::DisplayLarge),
            ("Display Medium", TypeScale::DisplayMedium),
            ("Display Small", TypeScale::DisplaySmall),
            ("Headline Large", TypeScale::HeadlineLarge),
            ("Headline Medium", TypeScale::HeadlineMedium),
            ("Headline Small", TypeScale::HeadlineSmall),
            ("Title Large", TypeScale::TitleLarge),
            ("Title Medium", TypeScale::TitleMedium),
            ("Title Small", TypeScale::TitleSmall),
            ("Body Large", TypeScale::BodyLarge),
            ("Body Medium", TypeScale::BodyMedium),
            ("Body Small", TypeScale::BodySmall),
            ("Label Large", TypeScale::LabelLarge),
            ("Label Medium", TypeScale::LabelMedium),
            ("Label Small", TypeScale::LabelSmall),
        ] {
            types = types.push(typography(name, role));
        }
        let mut ui = Harness::<Message>::new(padded(types), Size::new(500.0, 650.0), theme(dark));
        ui.at(0);
        capture(&mut ui, &themed_name("typography", dark), "00-scale");
        let mut cards = widget::Column::new().spacing(20);
        for (name, variant) in [
            ("Filled surface", SurfaceVariant::Filled),
            ("Outlined surface", SurfaceVariant::Outlined),
            ("Elevated surface", SurfaceVariant::Elevated),
        ] {
            cards = cards.push(
                surface(
                    widget::column![
                        typography(name, TypeScale::TitleMedium),
                        divider(),
                        typography("Supporting content", TypeScale::BodyMedium)
                    ]
                    .spacing(12),
                )
                .variant(variant),
            );
        }
        cards = cards.push(widget::row![badge(0), badge(1), badge(99), badge(100)].spacing(16));
        cards = cards.push(
            checkbox(false)
                .indeterminate(true)
                .label("Mixed selection")
                .on_toggle(Message::Toggle),
        );
        cards = cards.push(
            checkbox(true)
                .error(true)
                .label("Error selection")
                .on_toggle(Message::Toggle),
        );
        let mut ui = Harness::new(padded(cards), Size::new(420.0, 540.0), theme(dark));
        ui.at(0);
        capture(
            &mut ui,
            &themed_name("surfaces-badges-divider", dark),
            "00-variants",
        );
    }
}

#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_structured_content() {
    for dark in [false, true] {
        let make = |selected| {
            widget::row![
                navigation_rail(
                    [
                        NavigationItem::new(1, "Workspace", icon()),
                        NavigationItem::new(2, "Activity", icon()).badge(3),
                        NavigationItem::new(3, "Disabled", icon()).disabled(true),
                    ],
                    Some(selected)
                )
                .on_select(Message::Select)
                .footer(button("Help").on_press(Message::Action)),
                widget::column![
                    app_bar("An unusually long workspace title")
                        .action(menu("More", [MenuItem::new("Settings", Message::Action)]))
                        .scrolled(true),
                    list([
                        list_item("Design notes")
                            .leading(icon())
                            .supporting_text("A longer description wraps in narrow windows.")
                            .trailing(checkbox(false).on_toggle(Message::Toggle))
                            .selected(true)
                            .on_press(Message::Action)
                            .into(),
                        list_item("Shared file")
                            .overline("TEAM")
                            .supporting_text("Updated today")
                            .trailing(menu(
                                "Actions",
                                [MenuItem::new("Duplicate", Message::Action)]
                            ))
                            .into(),
                        list_item("Archived").disabled(true).into(),
                    ]),
                ]
                .width(Length::Fill)
            ]
            .height(Length::Fill)
        };
        let case = themed_name("rail-app-bar-lists", dark);
        let mut ui = Harness::new(make(1), Size::new(520.0, 410.0), theme(dark));
        ui.at(0);
        capture(&mut ui, &case, "00-default");
        ui.click("Activity");
        ui.rebuild(make(2));
        ui.at(150);
        capture(&mut ui, &case, "01-selected-hover");
        ui.click("Actions");
        ui.at(700);
        capture(&mut ui, &case, "02-trailing-menu");
    }
}

#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_overlays() {
    for dark in [false, true] {
        let choices = || {
            [
                MenuItem::new("Duplicate workspace", Message::Action).shortcut("Ctrl+D"),
                MenuItem::new("Save", Message::Action).selected(true),
                MenuItem::separator(),
                MenuItem::new("Managed", Message::Action).disabled(true),
                MenuItem::new("Delete", Message::Action).destructive(true),
            ]
        };
        let mut ui = Harness::new(
            widget::container(menu("Actions", choices()))
                .align_right(Length::Fill)
                .align_bottom(Length::Fill)
                .padding(16),
            Size::new(320.0, 280.0),
            theme(dark),
        );
        ui.click("Actions");
        ui.at(500);
        capture(&mut ui, &themed_name("menus", dark), "00-bottom-edge");
        let mut ui = Harness::new(
            padded(crate::context_menu(
                surface(typography("Right-click here", TypeScale::BodyLarge)),
                choices(),
            )),
            Size::new(320.0, 300.0),
            theme(dark),
        );
        ui.move_to((80.0, 50.0));
        ui.event(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Right,
        )));
        ui.event(Event::Mouse(mouse::Event::ButtonReleased(
            mouse::Button::Right,
        )));
        ui.at(500);
        capture(&mut ui, &themed_name("context-menu", dark), "00-open");
        let dialog = crate::dialog::dialog(
            widget::column![
                typography("Review preferences", TypeScale::HeadlineSmall),
                typography(
                    "Choose access before saving these changes.",
                    TypeScale::BodyMedium
                ),
                select(
                    "Access",
                    [
                        SelectOption::new(1, "Viewer"),
                        SelectOption::new(2, "Editor"),
                        SelectOption::new(3, "Owner").disabled(true)
                    ],
                    Some(2)
                )
                .on_select(Message::Select),
                button("Save").on_press(Message::Action),
            ]
            .spacing(16),
        )
        .on_dismiss(Message::Dismiss);
        let mut ui = Harness::new(
            crate::dialog::modal(notice(true), dialog, true),
            Size::new(390.0, 460.0),
            theme(dark),
        );
        ui.at(0);
        capture(&mut ui, &themed_name("dialog", dark), "00-over-snackbar");
        ui.click("Access");
        ui.at(500);
        capture(&mut ui, &themed_name("select", dark), "00-inside-dialog");
        let mut ui = Harness::new(
            widget::container(tooltip(
                button("Pin").on_press(Message::Action),
                "Keep this workspace near the top",
            ))
            .align_right(Length::Fill)
            .padding(16),
            Size::new(320.0, 140.0),
            theme(dark),
        );
        ui.at(0);
        let point = ui.find("Pin").center();
        ui.move_to(point);
        ui.at(499);
        capture(&mut ui, &themed_name("tooltip", dark), "00-before-delay");
        ui.at(500);
        capture(&mut ui, &themed_name("tooltip", dark), "01-visible-500ms");
        ui.at(650);
        capture(&mut ui, &themed_name("tooltip", dark), "01-visible-650ms");
        ui.down();
        ui.at(750);
        capture(
            &mut ui,
            &themed_name("tooltip", dark),
            "02-dismissed-on-press",
        );
        let mut ui = Harness::new(notice(true), Size::new(600.0, 400.0), theme(dark));
        ui.at(0);
        capture(&mut ui, &themed_name("snackbar", dark), "00-over-text");
        let mut ui = Harness::new(
            snackbar::host(
                widget::space().width(Length::Fill).height(Length::Fill),
                Some(
                    snackbar("Workspace duplicated. Changes are saved for this session.")
                        .action("Undo", Message::Action)
                        .on_dismiss(Message::Dismiss),
                ),
            ),
            Size::new(320.0, 160.0),
            theme(dark),
        );
        ui.at(0);
        capture(&mut ui, &themed_name("snackbar", dark), "01-narrow-action");
        ui.at(3999);
        capture(&mut ui, &themed_name("snackbar", dark), "02-before-timeout");
        ui.at(4000);
        capture(&mut ui, &themed_name("snackbar", dark), "03-expired");
        assert_eq!(ui.messages, [Message::Dismiss]);
    }
}
