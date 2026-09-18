use super::{harness::Harness, reference, theme};
use crate::{Element, SelectOption, Theme, TypeScale, typography};
use iced::advanced::{Renderer as _, renderer::Headless};
use iced::{Length, Rectangle, Renderer, Size, widget};

fn backend() -> String {
    std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into())
}

#[test]
fn resizing_dialog_shadow_reuses_a_small_raster_and_preserves_the_outline() {
    let mut renderer = iced::futures::executor::block_on(<Renderer as Headless>::new(
        iced::Font::DEFAULT,
        16.0.into(),
        Some(&backend()),
    ))
    .unwrap();
    let cache = crate::elevation::Cache::default();
    let viewport = Rectangle::with_size(Size::new(640., 640.));
    let mut id = None;
    for height in [120., 257.5, 540., 350.] {
        let bounds = Rectangle {
            x: 40.25,
            y: 40.25,
            width: 560.,
            height,
        };
        for clipped in [false, true] {
            let clip = if clipped {
                Rectangle {
                    x: 50.,
                    y: 70.,
                    width: 480.,
                    height: 260.,
                }
            } else {
                viewport
            };
            for dark in [false, true] {
                let mut theme = theme(dark);
                theme.colors.shadow.a = if clipped { 0.5 } else { 1.0 };
                renderer.reset(viewport);
                crate::elevation::draw_resizing(
                    &mut renderer,
                    &theme,
                    bounds,
                    540.,
                    28.,
                    3.,
                    clip,
                    &cache,
                );
                let tiled = renderer.screenshot(Size::new(1280, 1280), 2.0, theme.colors.surface);
                let cached = cache.borrow();
                let (size, handle) = cached.as_ref().unwrap();
                assert!(
                    size[0] <= 160. && size[1] <= 160.,
                    "Blur work must stay bounded for a large dialog"
                );
                assert_eq!(
                    *id.get_or_insert(handle.id()),
                    handle.id(),
                    "Height, position and opacity changes must reuse the same raster"
                );
                drop(cached);
                renderer.reset(viewport);
                renderer.with_layer(clip, |renderer| {
                    crate::elevation::draw(
                        renderer,
                        &theme,
                        bounds,
                        28.0.into(),
                        3.,
                        clip,
                        &Default::default(),
                    )
                });
                let exact = renderer.screenshot(Size::new(1280, 1280), 2.0, theme.colors.surface);
                // Tiny Skia truncates each SVG raster's translated origin to
                // device pixels (including toward zero at negative positions).
                // Independently translated patches may differ by one edge pixel
                // at fractional positions. Require every color to remain within
                // the original outline's one-pixel neighborhood, not a broad
                // whole-image percentage tolerance.
                for (index, (&actual, &expected)) in tiled.iter().zip(&exact).enumerate() {
                    if actual.abs_diff(expected) <= 2 {
                        continue;
                    }
                    let (x, y, channel) = (index / 4 % 1280, index / 4 / 1280, index % 4);
                    let mut low = u8::MAX;
                    let mut high = u8::MIN;
                    for row in y.saturating_sub(1)..=(y + 1).min(1279) {
                        for col in x.saturating_sub(1)..=(x + 1).min(1279) {
                            let v = exact[(row * 1280 + col) * 4 + channel];
                            low = low.min(v);
                            high = high.max(v);
                        }
                    }
                    assert!(
                        (low.saturating_sub(2)..=high.saturating_add(2)).contains(&actual),
                        "Shadow pixel {x},{y}={actual} differs from Gaussian outline {low}..{high} (height={height}, clipped={clipped}, dark={dark})"
                    );
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Message {
    Select(u8),
    Open,
    Close,
}

fn ripple_dialog(open: bool, stacked: bool, nested: bool) -> Element<'static, Message> {
    ripple_dialog_with_paused_progress(open, stacked, nested, false)
}
fn ripple_dialog_with_paused_progress(
    open: bool,
    stacked: bool,
    nested: bool,
    paused: bool,
) -> Element<'static, Message> {
    let launcher = widget::column![
        crate::fab(widget::container(super::icon::add().size(36)).id("launcher-icon"))
            .size(crate::FabSize::Large)
            .on_press(Message::Open),
        crate::circular_progress(0.)
            .indeterminate(true)
            .paused(paused),
    ]
    .spacing(24);
    let content = widget::column![
        typography("New workspace", TypeScale::HeadlineSmall),
        crate::button("Cancel").on_press(Message::Close),
    ]
    .spacing(24);
    let dialog = crate::dialog(content).width(280.);
    if nested {
        crate::dialog::stack(
            widget::space().width(Length::Fill).height(Length::Fill),
            [
                (
                    crate::dialog(widget::container(launcher).height(340)).width(560.),
                    true,
                ),
                (dialog, open),
            ],
        )
    } else {
        let background = widget::container(launcher)
            .padding(24)
            .width(Length::Fill)
            .height(Length::Fill);
        if stacked {
            crate::dialog::stack(background, [(dialog, open)])
        } else {
            crate::dialog::modal(background, dialog, open)
        }
    }
}

fn ripple_ui(open: bool, stacked: bool, nested: bool, dark: bool) -> Harness<'static, Message> {
    let mut ui = Harness::with_backend(
        ripple_dialog(open, stacked, nested),
        Size::new(640., 520.),
        theme(dark),
        &backend(),
    );
    ui.at(0);
    ui.frame();
    ui
}

fn launcher_center(ui: &mut Harness<'_, Message>) -> iced::Point {
    use iced::advanced::widget::{Operation, operation};
    let mut query = iced_test::Selector::find(widget::Id::new("launcher-icon"));
    ui.operate(&mut iced::advanced::widget::operation::black_box(
        &mut query,
    ));
    match query.finish() {
        operation::Outcome::Some(Some(found)) => found.visible_bounds().unwrap().center(),
        _ => panic!("Missing FAB icon"),
    }
}

#[test]
fn fab_ripple_finishes_under_a_dialog_without_reactivating_the_background() {
    for (stacked, nested) in [(false, false), (true, false), (true, true)] {
        for dark in [false, true] {
            let mut ui = ripple_ui(false, stacked, nested, dark);
            let center = launcher_center(&mut ui);
            let crop = |image: reference::Image| {
                image.crop(
                    ((center.x - 48.) * 2.) as u32,
                    ((center.y - 48.) * 2.) as u32,
                    192,
                    192,
                )
            };
            ui.move_to((center.x + 28., center.y + 28.));
            ui.at(200);
            ui.down();
            ui.at(210);
            ui.up();
            assert_eq!(
                ui.messages,
                [Message::Open],
                "A quick tap must open immediately"
            );
            ui.messages.clear();
            ui.rebuild(ripple_dialog(true, stacked, nested));
            ui.at(210);
            let opening = crop(ui.frame());
            ui.at(285);
            assert!(
                opening != crop(ui.frame()),
                "Feedback must keep moving while covered"
            );
            ui.at(900);
            let finished = crop(ui.frame());
            let mut idle = ripple_ui(true, stacked, nested, dark);
            idle.at(900);
            assert!(
                finished == crop(idle.frame()),
                "The invoker must return to its idle appearance while the dialog is still open (stacked={stacked}, nested={nested}, dark={dark})"
            );
            assert_eq!(
                ui.at(901),
                iced::window::RedrawRequest::NextFrame,
                "Covered loading indicators keep animating after finite feedback settles"
            );
            ui.down();
            ui.up();
            assert!(
                ui.messages.is_empty(),
                "The background must remain noninteractive"
            );
        }
    }
}

#[test]
fn covering_a_held_fab_finishes_its_cancellation_without_replaying_the_release() {
    for (stacked, nested) in [(false, false), (true, false), (true, true)] {
        let mut ui = ripple_ui(false, stacked, nested, false);
        let center = launcher_center(&mut ui);
        ui.move_to(center);
        ui.down();
        ui.at(20);
        ui.rebuild(ripple_dialog(true, stacked, nested));
        ui.at(20);
        ui.at(900);
        ui.up();
        assert!(ui.messages.is_empty());
        ui.rebuild(ripple_dialog(false, stacked, nested));
        ui.at(1000);
        ui.at(1200);
        ui.up();
        assert!(
            ui.messages.is_empty(),
            "Covering a held FAB cancels its gesture"
        );
    }
}

#[test]
fn instant_dialog_keeps_scheduling_until_the_invoker_feedback_finishes() {
    for stacked in [false, true] {
        let mut theme = theme(false);
        theme.motion.dialog_enter = std::time::Duration::ZERO;
        theme.motion.dialog_exit = std::time::Duration::ZERO;
        let mut ui = Harness::with_backend(
            ripple_dialog_with_paused_progress(false, stacked, false, true),
            Size::new(640., 520.),
            theme,
            &backend(),
        );
        ui.at(0);
        ui.frame();
        let center = launcher_center(&mut ui);
        ui.move_to(center);
        ui.down();
        ui.at(10);
        ui.up();
        ui.messages.clear();
        ui.rebuild(ripple_dialog_with_paused_progress(
            true, stacked, false, true,
        ));
        ui.at(10);
        assert_eq!(
            ui.at(100),
            iced::window::RedrawRequest::NextFrame,
            "The invoker's ripple needs its own frames after the dialog has settled"
        );
        ui.at(600);
        assert_eq!(ui.at(601), iced::window::RedrawRequest::Wait);
        assert!(ui.messages.is_empty());
    }
}

#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_modal_ripple() {
    for (stacked, nested, host) in [
        (false, false, "single"),
        (true, false, "stack"),
        (true, true, "nested"),
    ] {
        for dark in [false, true] {
            let mut ui = ripple_ui(false, stacked, nested, dark);
            let center = launcher_center(&mut ui);
            ui.move_to((center.x + 28., center.y + 28.));
            ui.at(200);
            ui.down();
            ui.at(210);
            ui.up();
            ui.rebuild(ripple_dialog(true, stacked, nested));
            for ms in [0, 75, 225, 375, 690] {
                ui.at(210 + ms);
                reference::check(
                    &format!(
                        "modal-ripple/{}/{host}/after-open-{ms:03}ms",
                        if dark { "dark" } else { "light" }
                    ),
                    &ui.frame(),
                );
            }
        }
    }
}

fn dialog(open: bool, stacked: bool) -> Element<'static, Message> {
    let dialog = crate::dialog(
        widget::column![
            typography("Make it yours", TypeScale::HeadlineSmall),
            crate::select(
                "Workspace access",
                [
                    SelectOption::new(0, "Viewer"),
                    SelectOption::new(1, "Editor")
                ],
                Some(1)
            )
            .on_select(Message::Select)
            .background_with(|theme| theme.colors.surface_container_high),
            typography(
                "The field and label share the dialog surface.",
                TypeScale::BodyMedium
            ),
            crate::button("Cancel").on_press(Message::Close),
        ]
        .spacing(24),
    )
    .on_dismiss(Message::Close)
    .initial_focus(0);
    let background = widget::container(widget::space())
        .width(Length::Fill)
        .height(Length::Fill);
    if stacked {
        crate::dialog::stack(background, [(dialog, open)])
    } else {
        crate::dialog::modal(background, dialog, open)
    }
}
fn ui(open: bool, stacked: bool, theme: Theme) -> Harness<'static, Message> {
    let mut ui = Harness::with_backend(
        dialog(open, stacked),
        Size::new(640., 480.),
        theme,
        &backend(),
    );
    ui.at(0);
    ui.frame();
    ui
}
fn pixel(image: &reference::Image, x: f32, y: f32) -> &[u8] {
    let index = (((y * 2.) as u32 * image.width + (x * 2.) as u32) * 4) as usize;
    &image.pixels[index..index + 4]
}

#[test]
fn dialog_select_label_and_fill_match_the_surface_in_both_hosts_and_themes() {
    for dark in [false, true] {
        for stacked in [false, true] {
            let mut ui = ui(true, stacked, theme(dark));
            ui.at(600);
            let field = ui.find("Workspace access");
            let image = ui.frame();
            // Floating label starts 16px into the field; its cutout extends 4px
            // left. Sample the empty notch margin above the top border.
            let outside = pixel(&image, field.x + 8., field.y + 2.);
            assert_eq!(
                outside,
                pixel(&image, field.x + 13., field.y + 2.),
                "The label must not have a page-colored patch"
            );
            assert_eq!(
                outside,
                pixel(&image, field.x + field.width - 40., field.y + 45.),
                "The trigger fill must match its host"
            );
            ui.click("Workspace access");
            ui.at(1200);
            ui.click("Viewer");
            assert!(ui.messages.contains(&Message::Select(0)));
        }
    }
}

#[test]
#[ignore = "reference suite; run with --ignored"]
fn visual_references_dialog_regressions() {
    for dark in [false, true] {
        for stacked in [false, true] {
            let mut ui = ui(false, stacked, theme(dark));
            ui.rebuild(dialog(true, stacked));
            let name = format!(
                "dialog-fix/{}/{}",
                if dark { "dark" } else { "light" },
                if stacked { "stack" } else { "single" }
            );
            for ms in [0, 50, 100, 250, 500, 600] {
                ui.at(ms);
                reference::check(&format!("{name}/open-{ms:03}ms"), &ui.frame());
            }
            ui.rebuild(dialog(false, stacked));
            for ms in [0, 50, 100, 150] {
                ui.at(600 + ms);
                reference::check(&format!("{name}/close-{ms:03}ms"), &ui.frame());
            }
        }
    }
}
