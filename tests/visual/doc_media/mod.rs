//! Development-only documentation renderer. No production build script or clock API.
pub(super) mod encoder;
mod examples;
use super::{harness::Harness, reference::Image, theme};
use crate::{Element, MotionScheme, Theme, TypeScale, typography};
use iced::{Length, Size, widget};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Action,
    Toggle(bool),
    Select(u8),
    Input(String),
    Dismiss,
    Value(f32),
    Segments(crate::SegmentSelection<u8>),
    Range((f32, f32)),
    Date(crate::DateSelection),
    Time(crate::Time),
    Part(crate::TimePart),
    Index(usize),
}

#[derive(Default)]
struct State {
    active: bool,
    value: f32,
    progress: f32,
}
struct Example {
    id: &'static str,
    title: &'static str,
    size: Size,
    scale: u32,
    budget: usize,
    compare: bool,
    animated: bool,
    variants_size: Size,
    caption: &'static str,
    source: &'static str,
    variant_source: &'static str,
    make: fn(&State) -> Element<'static, Message>,
    variants: fn() -> Element<'static, Message>,
}
fn host(example: &Example, state: &State) -> Element<'static, Message> {
    let content = (example.make)(state);
    if matches!(
        example.id,
        "dialog" | "full_screen_dialog" | "side_sheet" | "snackbar"
    ) {
        content
    } else {
        let content = widget::container(content)
            .padding(24)
            .width(Length::Fill)
            .height(Length::Fill);
        if matches!(example.id, "slider" | "range_slider" | "select") {
            crate::focus::scope(content)
        } else {
            content.into()
        }
    }
}
fn render(example: &Example, standard: bool) -> Vec<Image> {
    let mut state = State {
        active: example.id == "dialog",
        value: 50.0,
        progress: 0.0,
    };
    let mut ui = Harness::new(
        host(example, &state),
        example.size,
        Theme::light().expressive().motion_scheme(if standard {
            MotionScheme::standard()
        } else {
            MotionScheme::expressive()
        }),
    );
    ui.at(0);
    ui.frame();
    // Settle the initially open dialog; no independent caret clock is focused.
    ui.at(1000);
    ui.frame();
    if matches!(example.id, "date_picker" | "time_picker") {
        ui.find("OK");
        ui.find("Cancel");
    }
    if !example.animated {
        return vec![ui.frame_at_scale(example.scale as f32)];
    }
    let mut frames = Vec::new();
    let mut activations = 0;
    let clickable = matches!(
        example.id,
        "switch"
            | "button_group"
            | "button"
            | "icon_button"
            | "fab"
            | "checkbox"
            | "radio"
            | "card"
            | "filter_chip"
            | "tabs"
            | "list_item"
            | "navigation_bar"
            | "segmented_buttons"
            | "date_picker"
            | "time_picker"
            | "split_button"
            | "toolbar"
    );
    let repeat = !matches!(
        example.id,
        "button_group" | "button" | "fab" | "card" | "split_button" | "toolbar"
    );
    for tick in 0..180 {
        ui.at(1000 + tick * 1000 / 60);
        if clickable && (repeat || tick < 100) {
            match tick {
                18 | 102 => {
                    let target = match example.id {
                        "switch" => ui.find("Workspace notifications").center(),
                        "checkbox" => ui.find("Workspace updates").center(),
                        "radio" => ui
                            .find(if state.active { "Personal" } else { "Team" })
                            .center(),
                        "button_group" => ui.find("Review").center(),
                        "button" => ui.find("Save").center(),
                        "card" | "list_item" => ui.find("Design notes").center(),
                        "filter_chip" => ui.find("Unread").center(),
                        "tabs" => ui
                            .find(if state.active { "Overview" } else { "Activity" })
                            .center(),
                        "navigation_bar" => ui
                            .find(if state.active { "Home" } else { "Notes" })
                            .center(),
                        "segmented_buttons" => {
                            ui.find(if state.active { "Day" } else { "Week" }).center()
                        }
                        "date_picker" => ui.find(if state.active { "15" } else { "16" }).center(),
                        "time_picker" => ui.find(if state.active { "AM" } else { "PM" }).center(),
                        "split_button" => ui.find("Create").center(),
                        "toolbar" => ui.find("Bold").center(),
                        "icon_button" => (44.0, 44.0).into(),
                        "fab" => (52.0, 52.0).into(),
                        _ => unreachable!(),
                    };
                    ui.move_to(target);
                }
                30 | 108 => ui.down(),
                48 | 120 => {
                    assert!(
                        ui.messages.is_empty(),
                        "holding must not activate {}",
                        example.id
                    );
                    ui.up();
                    let expected = match example.id {
                        "switch" | "checkbox" | "icon_button" | "filter_chip" | "list_item" => {
                            Message::Toggle(!state.active)
                        }
                        "radio" | "tabs" | "navigation_bar" => {
                            Message::Select(u8::from(!state.active))
                        }
                        "segmented_buttons" => Message::Segments(crate::SegmentSelection::Single(
                            Some(u8::from(!state.active)),
                        )),
                        "date_picker" => Message::Date(crate::DateSelection::Single(Some(
                            crate::Date::new(2026, 9, if state.active { 15 } else { 16 }).unwrap(),
                        ))),
                        "time_picker" => Message::Time(
                            crate::Time::new(if state.active { 10 } else { 22 }, 30).unwrap(),
                        ),
                        _ => Message::Action,
                    };
                    assert_eq!(ui.messages, [expected], "{} release", example.id);
                    match ui.messages.pop().unwrap() {
                        Message::Toggle(value) => state.active = value,
                        Message::Select(value) => state.active = value == 1,
                        Message::Segments(crate::SegmentSelection::Single(Some(value))) => {
                            state.active = value == 1
                        }
                        Message::Date(crate::DateSelection::Single(Some(date))) => {
                            state.active = date.day() == 16
                        }
                        Message::Time(time) => state.active = time.hour() == 22,
                        Message::Action => {}
                        _ => unreachable!(),
                    }
                    activations += 1;
                    ui.rebuild(host(example, &state));
                }
                72 | 144 => ui.leave(),
                _ => {}
            }
        } else if matches!(example.id, "slider" | "range_slider") {
            // The labeled slider's track runs between the 24px target insets,
            // inside the host's 24px padding. Its center is 56px below its top.
            let point = |value: f32| (48.0 + (example.size.width - 96.0) * value / 100.0, 80.0);
            match tick {
                18 | 102 => ui.move_to(point(state.value)),
                30 | 108 => ui.down(),
                36..=54 => ui.move_to(point(50.0 + (tick - 36) as f32 / 18.0 * 30.0)),
                114..=132 => ui.move_to(point(80.0 - (tick - 114) as f32 / 18.0 * 30.0)),
                60 | 138 => {
                    assert_eq!(state.value, if tick == 60 { 80.0 } else { 50.0 });
                    ui.up();
                    assert_eq!(
                        ui.messages,
                        [Message::Action],
                        "one commit after the last value"
                    );
                    ui.messages.clear();
                    activations += 1;
                }
                72 | 144 => {
                    ui.move_to((8.0, 8.0));
                    ui.down();
                    ui.up();
                    ui.leave();
                }
                _ => {}
            }
            for message in ui.messages.drain(..) {
                let value = match message {
                    Message::Value(value) => value,
                    Message::Range((lower, upper)) => {
                        assert_eq!(lower, 20.0);
                        upper
                    }
                    _ => panic!("unexpected slider message"),
                };
                assert!((50.0..=80.0).contains(&value));
                assert_eq!(value % 10.0, 0.0);
                state.value = value;
            }
            ui.rebuild(host(example, &state));
        } else if catalog_tick(example, &mut state, &mut ui, tick) {
            // Specialized overlay and controlled-layout timelines.
        } else {
            match (example.id, tick) {
                ("text_field", 30 | 114) | ("dialog", 30 | 90) => {
                    state.active = !state.active;
                    ui.rebuild(host(example, &state));
                }
                _ => {}
            }
        }
        frames.push(ui.frame_at_scale(example.scale as f32));
    }
    if clickable || matches!(example.id, "slider" | "range_slider") {
        assert_eq!(
            activations,
            if repeat { 2 } else { 1 },
            "{} must complete every interaction",
            example.id
        );
    }
    assert!(
        frames.windows(2).any(|p| p[0] != p[1]),
        "{} must animate",
        example.id
    );
    if !matches!(
        example.id,
        "loading_indicator" | "linear_progress" | "circular_progress"
    ) {
        assert!(
            frames[0] == frames[179],
            "{} loop must return to its initial pixels",
            example.id
        );
    }
    frames
}
// All changes here drive the public view parameters or real input events.
fn catalog_tick(
    example: &Example,
    state: &mut State,
    ui: &mut Harness<'_, Message>,
    tick: u64,
) -> bool {
    match example.id {
        "snackbar" | "navigation_rail" | "carousel" | "side_sheet" | "search_bar" | "fab_menu" => {
            if matches!(tick, 30 | 90) {
                state.active = !state.active;
                ui.rebuild(host(example, state));
            }
            if example.id == "snackbar" && tick == 70 {
                assert!(ui.find("Changes saved").height >= 20.0);
                assert!(ui.find("Undo").height >= 20.0);
            }
        }
        "app_bar" => {
            state.progress = match tick {
                30..=60 => (tick - 30) as f32 / 30.0,
                61..=89 => 1.0,
                90..=120 => 1.0 - (tick - 90) as f32 / 30.0,
                _ => 0.0,
            };
            ui.rebuild(host(example, state));
        }
        "menu" | "rich_tooltip" => {
            if tick == 18 {
                ui.click(if example.id == "menu" {
                    "Actions"
                } else {
                    "About"
                });
            }
            if tick == 72 {
                if example.id == "menu" {
                    ui.click("Duplicate");
                    assert_eq!(ui.messages, [Message::Action]);
                    ui.messages.clear();
                } else {
                    ui.move_to((example.size.width - 8.0, example.size.height - 8.0));
                    ui.down();
                    ui.up();
                }
                ui.leave();
            }
        }
        "select" => {
            if matches!(tick, 18 | 102) {
                ui.click(if state.active { "Editor" } else { "Viewer" });
            }
            if matches!(tick, 48 | 120) {
                ui.click(if state.active { "Viewer" } else { "Editor" });
                assert_eq!(ui.messages, [Message::Select(u8::from(!state.active))]);
                let Message::Select(value) = ui.messages.pop().unwrap() else {
                    unreachable!()
                };
                state.active = value == 1;
                ui.rebuild(host(example, state));
            }
            if matches!(tick, 72 | 144) {
                ui.move_to((example.size.width - 8.0, example.size.height - 8.0));
                ui.down();
                ui.up();
                ui.leave();
            }
        }
        "tooltip" => {
            if tick == 18 {
                let point = ui.find("Help").center();
                ui.move_to(point);
            }
            if tick == 100 {
                ui.leave();
            }
        }
        _ => return false,
    }
    true
}
fn beside(left: &Image, right: &Image) -> Image {
    assert_eq!(left.height, right.height);
    let mut pixels = Vec::new();
    for y in 0..left.height as usize {
        for image in [left, right] {
            let start = y * image.width as usize * 4;
            pixels.extend_from_slice(&image.pixels[start..start + image.width as usize * 4]);
        }
    }
    Image {
        width: left.width + right.width,
        height: left.height,
        pixels,
    }
}
fn variants(example: &Example) -> Image {
    let frames: Vec<_> = [false, true]
        .into_iter()
        .map(|dark| {
            let mut ui = Harness::new(
                widget::column![
                    typography(if dark { "Dark" } else { "Light" }, TypeScale::TitleMedium),
                    (example.variants)(),
                ]
                .spacing(16)
                .padding(24)
                .width(Length::Fill)
                .height(Length::Fill),
                example.variants_size,
                theme(dark).expressive(),
            );
            ui.at(0);
            ui.frame();
            match example.id {
                "menu" => ui.click("Actions"),
                "rich_tooltip" => ui.click("About"),
                "tooltip" => {
                    let point = ui.find("Help").center();
                    ui.move_to(point);
                }
                _ => {}
            }
            ui.at(1000);
            ui.frame();
            ui.at(1300);
            if matches!(example.id, "date_picker" | "time_picker") {
                ui.find("OK");
                ui.find("Cancel");
            }
            if example.id == "snackbar" {
                assert!(ui.find("Changes saved").height >= 20.0);
                assert!(ui.find("Undo").height >= 20.0);
            }
            ui.frame_at_scale(example.scale as f32)
        })
        .collect();
    beside(&frames[0], &frames[1])
}

#[test]
#[ignore = "documentation renderer; invoked by cargo xtask doc-media"]
fn generate_component_doc_media() {
    let root = PathBuf::from(
        std::env::var_os("ICED_M3_DOC_OUTPUT")
            .expect("set ICED_M3_DOC_OUTPUT to a fresh output directory"),
    );
    fs::create_dir_all(&root).unwrap();
    assert!(
        fs::read_dir(&root).unwrap().next().is_none(),
        "render output must be empty"
    );
    let mut registry = Vec::new();
    for example in examples::registry() {
        let frames = render(&example, false);
        let bytes = if example.animated {
            encoder::encode_sparse(&frames.iter().collect::<Vec<_>>(), 60).0
        } else {
            let path = root.join(format!("{}.png", example.id));
            frames[0].write(&path);
            fs::read(path).unwrap()
        };
        let mut total = bytes.len();
        fs::write(root.join(format!("{}.png", example.id)), bytes).unwrap();
        frames[0].write(&root.join(format!("{}-poster.png", example.id)));
        variants(&example).write(&root.join(format!("{}-variants.png", example.id)));
        if example.compare {
            let standard = render(&example, true);
            let paired: Vec<_> = standard
                .iter()
                .zip(&frames)
                .map(|(a, b)| beside(a, b))
                .collect();
            paired[0].write(&root.join(format!("{}-comparison-poster.png", example.id)));
            let (bytes, _) = encoder::encode_sparse(&paired.iter().collect::<Vec<_>>(), 60);
            total += bytes.len();
            fs::write(root.join(format!("{}-comparison.png", example.id)), bytes).unwrap();
        }
        assert!(
            total <= example.budget,
            "{}: primary/comparison bytes {total} exceed reviewed budget {}",
            example.id,
            example.budget
        );
        fs::write(root.join(format!("{}.rs", example.id)), example.source).unwrap();
        fs::write(
            root.join(format!("{}-variants.rs", example.id)),
            format!(
                "fn variants() -> Element<'static, Message> {{ ({})() }}",
                example.variant_source
            ),
        )
        .unwrap();
        fs::write(root.join(format!("{}.txt", example.id)), example.caption).unwrap();
        registry.push(serde_json::json!({
            "id": example.id, "title": example.title,
            "scale": example.scale, "width": example.size.width as u32,
            "height": example.size.height as u32, "compare": example.compare,
            "budget": example.budget, "animated": example.animated,
        }));
        eprintln!("{}: {total} primary/comparison bytes", example.id);
    }
    fs::write(
        root.join("registry.json"),
        serde_json::to_vec_pretty(&registry).unwrap(),
    )
    .unwrap();
}
