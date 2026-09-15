use super::{harness::Harness, reference, theme, themed_name};
use crate::{
    CarouselVariant, Element, MenuItem, Theme, TypeScale, button, carousel, dialog, lazy_carousel,
    menu, search_bar, staged, typography,
};
use iced::{Event, Length, Size, keyboard, widget, window};
use std::{cell::RefCell, rc::Rc, time::Duration};

#[derive(Clone, Debug, PartialEq)]
enum Message {
    Action(usize),
    Index(usize),
    Input(usize, String),
    Close,
}
fn ui<'a>(
    content: impl Into<Element<'a, Message>>,
    size: Size,
    theme: Theme,
) -> Harness<'a, Message> {
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
    Harness::with_backend(content, size, theme, &backend)
}
fn key(key: keyboard::Key, text: Option<&str>) -> Event {
    Event::Keyboard(keyboard::Event::KeyPressed {
        modified_key: key.clone(),
        key,
        physical_key: keyboard::key::Physical::Unidentified(
            keyboard::key::NativeCode::Unidentified,
        ),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::empty(),
        text: text.map(Into::into),
        repeat: false,
    })
}
fn escape() -> Event {
    key(keyboard::Key::Named(keyboard::key::Named::Escape), None)
}
fn unfocus(ui: &mut Harness<'_, Message>) {
    ui.operate(&mut iced::advanced::widget::operation::focusable::unfocus::<()>());
}
fn editor_bounds(view: &mut Harness<'_, Message>) -> iced::Rectangle {
    use iced::advanced::widget::{Operation, operation::TextInput};
    struct Inputs(Vec<iced::Rectangle>);
    impl Operation for Inputs {
        fn traverse(&mut self, walk: &mut dyn FnMut(&mut dyn Operation)) {
            walk(self);
        }
        fn text_input(
            &mut self,
            _: Option<&widget::Id>,
            bounds: iced::Rectangle,
            _: &mut dyn TextInput,
        ) {
            self.0.push(bounds);
        }
    }
    let mut inputs = Inputs(Vec::new());
    view.operate(&mut inputs);
    assert_eq!(inputs.0.len(), 1);
    inputs.0[0]
}
fn painted_part(part: staged::Part) -> Element<'static, Message> {
    staged::part(
        widget::container(widget::space().width(80).height(40)).style(|_| {
            widget::container::Style {
                background: Some(iced::Color::BLACK.into()),
                ..Default::default()
            }
        }),
        part,
    )
}

#[test]
fn semantic_fades_stagger_reverse_without_jumps_and_finish_without_idle_frames() {
    // Exercise the actual retained widgets and their pixels, with the same phase
    // scope used by overlay hosts. Isolating surface growth makes row order measurable.
    for above in [false, true] {
        let mut phase = staged::Phase {
            kind: staged::Kind::Menu,
            open: true,
            initial: 0.,
            base: 1.,
            enter: Duration::from_millis(500),
            exit: Duration::from_millis(150),
            above,
        };
        let mut view = ui(
            widget::column![
                painted_part(staged::Part::MenuRow(0, 3)),
                painted_part(staged::Part::MenuRow(1, 3)),
                painted_part(staged::Part::MenuRow(2, 3))
            ],
            Size::new(80., 120.),
            Theme::light(),
        );
        {
            let _phase = staged::enter(phase);
            view.at(0);
            view.at(100);
        }
        let frame = view.frame();
        let sample = |y: usize| frame.pixels[(y * frame.width as usize + 80) * 4];
        assert!(
            if above {
                sample(40) > sample(200)
            } else {
                sample(40) < sample(200)
            },
            "rows nearest the opening edge fade first"
        );
        phase.open = false;
        {
            let _phase = staged::enter(phase);
            view.at(100);
        }
        assert!(
            view.frame() == frame,
            "closing retargets from the visible opacity"
        );
        {
            let _phase = staged::enter(phase);
            view.at(125);
        }
        let reversed = view.frame();
        phase.open = true;
        {
            let _phase = staged::enter(phase);
            view.at(125);
        }
        assert!(view.frame() == reversed, "reopening does not reset rows");
        {
            let _phase = staged::enter(phase);
            assert_eq!(view.at(1000), window::RedrawRequest::Wait);
        }
        let settled = view.frame();
        assert!(settled.pixels.chunks_exact(4).all(|p| p[0] == 0));
    }
}

#[test]
fn dialog_actions_wait_for_their_stage_and_reduced_motion_skips_the_delay() {
    let mut phase = staged::Phase {
        kind: staged::Kind::Dialog,
        open: true,
        initial: 0.,
        base: 1.,
        enter: Duration::from_millis(500),
        exit: Duration::from_millis(150),
        above: false,
    };
    let mut view = ui(
        painted_part(staged::Part::DialogActions),
        Size::new(80., 40.),
        Theme::light(),
    );
    let hidden;
    {
        let _phase = staged::enter(phase);
        view.at(0);
        hidden = view.frame();
        view.at(149);
    }
    assert!(view.frame() == hidden);
    {
        let _phase = staged::enter(phase);
        view.at(225);
    }
    assert!(view.frame() != hidden);
    phase.enter = Duration::ZERO;
    {
        let _phase = staged::enter(phase);
        view.at(226);
    }
    assert!(view.frame().pixels.chunks_exact(4).all(|p| p[0] == 0));
    let _phase = staged::enter(phase);
    assert_eq!(view.at(227), window::RedrawRequest::Wait);
}

fn search(open: bool, full: bool) -> Element<'static, Message> {
    widget::container(
        search_bar(
            "Search",
            "A long query keeps its editor width while opening",
        )
        .open(open)
        .full_screen(full)
        .width(420)
        .height(360.)
        .on_input(|s| Message::Input(0, s))
        .on_close(Message::Close)
        .results(
            widget::column![
                typography("Results", TypeScale::TitleMedium),
                button("Open result").on_press(Message::Action(1))
            ]
            .spacing(16),
        ),
    )
    .padding(24)
    .into()
}
#[test]
fn search_retains_content_width_and_full_header_height_and_blocks_closing_input() {
    for full in [false, true] {
        let mut view = ui(search(false, full), Size::new(680., 500.), Theme::light());
        view.at(0);
        view.frame();
        view.rebuild(search(true, full));
        view.at(0);
        unfocus(&mut view);
        view.at(100);
        let early = view.find("Results");
        let early_editor = editor_bounds(&mut view);
        view.at(300);
        let settled = view.find("Results");
        let settled_editor = editor_bounds(&mut view);
        assert!(
            (early_editor.width - settled_editor.width).abs() < 0.01
                && early_editor.height == settled_editor.height,
            "native editor gets final bounds throughout the surface morph"
        );
        assert!(
            (early.width - settled.width).abs() < 0.01 && early.height == settled.height,
            "text keeps its final layout during expansion"
        );
        assert_eq!(settled.y, if full { 89. } else { 97. });
        let target = view.find("Open result").center();
        view.rebuild(search(false, full));
        view.at(300);
        view.move_to(target);
        view.down();
        view.up();
        view.event(escape());
        view.event(key(keyboard::Key::Named(keyboard::key::Named::Tab), None));
        view.event(key(keyboard::Key::Named(keyboard::key::Named::Enter), None));
        assert!(
            view.messages.is_empty(),
            "closing content cannot focus, activate or dismiss twice"
        );
        view.at(550);
        assert_eq!(view.at(551), window::RedrawRequest::Wait);
    }
}

fn lazy(count: usize, index: usize, built: Rc<RefCell<Vec<usize>>>) -> Element<'static, Message> {
    lazy_carousel(count, index, move |i| {
        built.borrow_mut().push(i);
        widget::column![
            typography(format!("Item {i}"), TypeScale::TitleLarge),
            widget::text_input("Name", "abcd")
                .id(format!("item-{i}"))
                .on_input(move |s| Message::Input(i, s)),
            button(format!("Open {i}")).on_press(Message::Action(i)),
        ]
        .padding(32)
        .spacing(8)
        .into()
    })
    .variant(CarouselVariant::Uncontained)
    .on_select(Message::Index)
    .into()
}
#[test]
fn lazy_carousel_bounds_work_preserves_overlapping_editor_state_and_global_callbacks() {
    let built = Rc::new(RefCell::new(Vec::new()));
    let mut view = ui(
        lazy(10_000, 3, built.clone()),
        Size::new(600., 200.),
        Theme::light(),
    );
    view.at(0);
    view.frame();
    assert!(
        built.borrow().len() < 24,
        "initial construction is independent of total item count"
    );
    view.operate(
        &mut iced::advanced::widget::operation::focusable::focus::<()>(widget::Id::new("item-4")),
    );
    // Rebuild and shift the mounted range while item 4 remains visible in both layouts.
    built.borrow_mut().clear();
    view.rebuild(lazy(10_000, 4, built.clone()));
    view.at(0);
    view.at(200);
    assert!(
        built.borrow().len() < 48,
        "scrolling only rebuilds a bounded overscan region"
    );
    view.event(key(keyboard::Key::Character("z".into()), Some("z")));
    assert_eq!(
        view.messages,
        [Message::Input(4, "abcdz".into())],
        "the overlapping native editor keeps focus and cursor"
    );
    view.click("Open 4");
    assert_eq!(view.messages.last(), Some(&Message::Action(4)));
    built.borrow_mut().clear();
    view.rebuild(lazy(10_000, 9999, built.clone()));
    view.at(200);
    view.at(400);
    assert!(
        built.borrow().len() < 48,
        "a distant jump does not visit every intervening item"
    );
    view.click("Open 9999");
    assert_eq!(view.messages.last(), Some(&Message::Action(9999)));
    view.rebuild(lazy(2, 9999, built.clone()));
    view.at(400);
    view.at(600);
    view.click("Open 1");
    assert_eq!(view.messages.last(), Some(&Message::Action(1)));
    view.rebuild(lazy(0, 9999, built));
    view.at(800);
    assert_eq!(view.at(1000), window::RedrawRequest::Wait);
}

fn cards(index: usize, variant: CarouselVariant) -> Element<'static, Message> {
    carousel(
        (0..12).map(|i| {
            widget::container(
                widget::column![
                    typography(format!("Collection {}", i + 1), TypeScale::HeadlineSmall),
                    button("Open collection").on_press(Message::Action(i))
                ]
                .spacing(16),
            )
            .padding(32)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |t: &Theme| widget::container::Style {
                background: Some(
                    if i % 2 == 0 {
                        t.colors.primary_container
                    } else {
                        t.colors.tertiary_container
                    }
                    .into(),
                ),
                text_color: Some(t.colors.on_surface),
                ..Default::default()
            })
            .into()
        }),
        index,
    )
    .variant(variant)
    .on_select(Message::Index)
    .into()
}
#[test]
fn carousel_flick_uses_recent_velocity_and_a_held_drag_snaps_to_nearest_item() {
    let drag = |held: bool| {
        let mut view = ui(
            cards(0, CarouselVariant::Uncontained),
            Size::new(600., 200.),
            Theme::light(),
        );
        view.at(0);
        view.frame();
        view.move_to((400., 170.));
        view.down();
        view.at(40);
        view.move_to((220., 170.));
        if held {
            view.at(200);
        }
        view.up();
        match view.messages.as_slice() {
            [Message::Index(i)] => *i,
            other => panic!("unexpected actions: {other:?}"),
        }
    };
    assert_eq!(drag(true), 1);
    assert!(
        drag(false) > drag(true),
        "a recent flick advances farther than a stopped drag"
    );
    // Endpoint geometry can end at a fractional keyline offset. It must not snap
    // backward from the final item when the pointer is released there.
    let mut view = ui(
        cards(0, CarouselVariant::MultiBrowse),
        Size::new(680., 200.),
        Theme::light(),
    );
    view.at(0);
    view.frame();
    view.move_to((600., 170.));
    view.down();
    view.move_to((-10000., 170.));
    view.at(200);
    view.up();
    assert_eq!(view.messages, [Message::Index(11)]);
    view.rebuild(cards(11, CarouselVariant::MultiBrowse));
    view.at(400);
    let last = view.find("Collection 12");
    assert!(last.x >= 0. && last.x + last.width <= 680.);
}

fn modal(open: bool) -> Element<'static, Message> {
    dialog::stack(
        widget::space().width(Length::Fill).height(Length::Fill),
        [(
            dialog::dialog(
                widget::column![
                    typography("Review workspace", TypeScale::HeadlineSmall),
                    typography(
                        "These preferences apply to your workspace.",
                        TypeScale::BodyMedium
                    ),
                    dialog::actions(
                        widget::row![
                            button("Cancel").on_press(Message::Close),
                            button("Save").on_press(Message::Action(1))
                        ]
                        .spacing(8)
                    )
                ]
                .spacing(24),
            )
            .width(440.)
            .on_dismiss(Message::Close),
            open,
        )],
    )
}

fn disabled_buttons() -> Element<'static, Message> {
    widget::container(
        widget::column![
            typography("New workspace", TypeScale::HeadlineSmall),
            typography(
                "Enter a name to enable Create workspace.",
                TypeScale::BodyMedium
            ),
            button("Create workspace")
                .on_press(Message::Action(0))
                .disabled(true),
            button("Tonal disabled").variant(crate::ButtonVariant::Tonal),
            button("Elevated disabled").variant(crate::ButtonVariant::Elevated),
            button("Outlined disabled").variant(crate::ButtonVariant::Outlined),
            button("Text disabled").variant(crate::ButtonVariant::Text),
        ]
        .spacing(16),
    )
    .padding(24)
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|theme: &Theme| widget::container::Style {
        background: Some(theme.colors.surface_container_high.into()),
        ..Default::default()
    })
    .into()
}

fn popup(calendar: bool) -> Element<'static, Message> {
    let trigger = typography("Open panel", TypeScale::LabelLarge);
    let content: Element<'static, Message> = if calendar {
        let date = crate::Date::new(2026, 9, 14).unwrap();
        crate::docked_date_picker(
            trigger,
            crate::date_picker(date, crate::DateSelection::Single(Some(date)))
                .on_select(|_| Message::Action(1)),
        )
    } else {
        crate::rich_tooltip(
            trigger,
            "Workspace access",
            widget::column![
                typography(
                    "Invite people to collaborate on this workspace.",
                    TypeScale::BodyMedium
                ),
                button("Invite").on_press(Message::Action(1))
            ]
            .spacing(12),
        )
        .into()
    };
    widget::container(content).padding(16).into()
}

#[test]
fn rich_popup_and_calendar_complete_fades_and_ignore_input_during_exit() {
    for calendar in [false, true] {
        let mut view = ui(popup(calendar), Size::new(420., 550.), Theme::dark());
        view.at(0);
        view.frame();
        view.click("Open panel");
        view.at(0);
        let start = view.frame();
        view.at(200);
        let middle = view.frame();
        view.at(500);
        let end = view.frame();
        assert!(start != middle && middle != end);
        assert_eq!(view.at(501), window::RedrawRequest::Wait);
        let point = view.find(if calendar { "15" } else { "Invite" }).center();
        view.event(escape());
        view.move_to(point);
        view.down();
        view.up();
        assert!(
            view.messages.is_empty(),
            "closing popups cannot publish actions"
        );
        view.at(651);
        assert_eq!(view.at(652), window::RedrawRequest::Wait);
        view.click("Open panel");
        view.at(1152);
        view.click(if calendar { "15" } else { "Invite" });
        assert_eq!(view.messages, [Message::Action(1)]);
    }
}

#[test]
fn disabled_button_labels_remain_distinct_from_the_fill_on_raised_surfaces() {
    for dark in [false, true] {
        let mut view = ui(disabled_buttons(), Size::new(420., 440.), theme(dark));
        view.at(0);
        let image = view.frame();
        for label in [
            "Create workspace",
            "Tonal disabled",
            "Elevated disabled",
            "Outlined disabled",
            "Text disabled",
        ] {
            let bounds = view.find(label);
            let crop = image.crop(
                (bounds.x * 2.) as u32,
                (bounds.y * 2.) as u32,
                (bounds.width * 2.) as u32,
                (bounds.height * 2.) as u32,
            );
            let min = crop.pixels.chunks_exact(4).map(|p| p[0]).min().unwrap();
            let max = crop.pixels.chunks_exact(4).map(|p| p[0]).max().unwrap();
            assert!(
                max - min > 25,
                "{label}: disabled foreground must not collapse onto its container (dark={dark}, range={min}..{max})"
            );
            view.click(label);
        }
        assert!(
            view.messages.is_empty(),
            "visually legible disabled buttons still reject activation"
        );
        if let Ok(folder) = std::env::var("DESKTOP_FINISH_ARTIFACTS") {
            image.write(&std::path::Path::new(&folder).join(format!(
                "disabled-{}.png",
                if dark { "dark" } else { "light" }
            )));
        }
    }
}
#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_desktop_finish() {
    for dark in [false, true] {
        let capture = |view: &mut Harness<'_, Message>, case: &str, step: &str| {
            reference::check(
                &format!(
                    "{}/{step}",
                    themed_name(&format!("desktop-finish-{case}"), dark)
                ),
                &view.frame(),
            );
        };
        let mut disabled = Harness::new(disabled_buttons(), Size::new(420., 440.), theme(dark));
        disabled.at(0);
        capture(&mut disabled, "disabled-buttons", "raised-surface");
        for calendar in [false, true] {
            let case = if calendar { "calendar" } else { "rich-tooltip" };
            let mut view = Harness::new(popup(calendar), Size::new(420., 550.), theme(dark));
            view.at(0);
            view.frame();
            view.click("Open panel");
            view.leave();
            for time in [0, 100, 200, 350, 500] {
                view.at(time);
                capture(&mut view, case, &format!("open-{time:03}"));
            }
            view.event(escape());
            for time in [0, 50, 100, 150] {
                view.at(500 + time);
                capture(&mut view, case, &format!("close-{time:03}"));
            }
        }
        let mut view = Harness::new(modal(false), Size::new(600., 380.), theme(dark));
        view.at(0);
        view.frame();
        view.rebuild(modal(true));
        for time in [0, 75, 150, 225, 300, 500] {
            view.at(time);
            capture(&mut view, "dialog", &format!("open-{time:03}"));
        }
        view.rebuild(modal(false));
        for time in [0, 50, 100, 150] {
            view.at(500 + time);
            capture(&mut view, "dialog", &format!("close-{time:03}"));
        }
        for above in [false, true] {
            let content = widget::container(menu(
                "Actions",
                (0..4).map(|i| MenuItem::new(format!("Menu item {}", i + 1), Message::Action(i))),
            ))
            .padding(16)
            .align_y(if above {
                iced::alignment::Vertical::Bottom
            } else {
                iced::alignment::Vertical::Top
            })
            .height(Length::Fill);
            let mut view = Harness::new(content, Size::new(360., 360.), theme(dark));
            view.at(0);
            view.frame();
            view.click("Actions");
            view.leave();
            let case = if above { "menu-above" } else { "menu-below" };
            for time in [0, 100, 200, 300, 400, 500] {
                view.at(time);
                capture(&mut view, case, &format!("open-{time:03}"));
            }
            view.event(escape());
            for time in [0, 50, 100, 150] {
                view.at(500 + time);
                capture(&mut view, case, &format!("close-{time:03}"));
            }
        }
        for full in [false, true] {
            let case = if full { "search-full" } else { "search-docked" };
            let mut view = Harness::new(search(false, full), Size::new(680., 500.), theme(dark));
            view.at(0);
            view.frame();
            view.rebuild(search(true, full));
            view.at(0);
            unfocus(&mut view);
            for time in [0, 75, 150, 225, 275, 300] {
                view.at(time);
                capture(&mut view, case, &format!("open-{time:03}"));
            }
            view.rebuild(search(false, full));
            for time in [0, 42, 83, 150, 250] {
                view.at(300 + time);
                capture(&mut view, case, &format!("close-{time:03}"));
            }
        }
        for variant in [
            CarouselVariant::MultiBrowse,
            CarouselVariant::Hero,
            CarouselVariant::HeroCenter,
            CarouselVariant::Uncontained,
            CarouselVariant::FullScreen,
        ] {
            let case = format!("carousel-{variant:?}");
            let mut view = Harness::new(cards(0, variant), Size::new(680., 200.), theme(dark));
            view.at(0);
            capture(&mut view, &case, "start");
            view.rebuild(cards(4, variant));
            for time in [0, 50, 100, 200] {
                view.at(time);
                capture(&mut view, &case, &format!("advance-{time:03}"));
            }
            view.rebuild(cards(11, variant));
            view.at(200);
            view.at(400);
            capture(&mut view, &case, "end");
        }
    }
}
