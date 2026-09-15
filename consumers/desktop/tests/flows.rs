//! Tests use the consumer's real view/update and public iced APIs only.
use iced::advanced::{renderer::Headless, widget::Operation};
use iced::{Event, Point, Rectangle, Renderer, Size, keyboard, mouse, window};
use iced_test::{
    Selector,
    runtime::{UserInterface, user_interface},
    simulator,
};
use material_desktop_consumer::{Access, Digest, Message, Page, Studio};
use std::{path::Path, time::Instant};

struct Client {
    app: Studio,
    cache: user_interface::Cache,
    renderer: Renderer,
    size: Size,
    cursor: mouse::Cursor,
    delivered: Vec<Message>,
}

impl Client {
    fn new(dark: bool, size: Size) -> Self {
        let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
        let renderer = iced::futures::executor::block_on(<Renderer as Headless>::new(
            iced_material::fonts::REGULAR,
            16.0.into(),
            Some(&backend),
        ))
        .expect("consumer renderer");
        Self {
            app: Studio {
                dark,
                reduced_motion: true,
                ..Default::default()
            },
            cache: Default::default(),
            renderer,
            size,
            cursor: mouse::Cursor::Unavailable,
            delivered: Vec::new(),
        }
    }

    // Rebuild exactly where an application does, after delivering the previous
    // event's messages. The widget cache survives; the view cannot borrow private
    // library state or the repository's test clock.
    fn with_ui<T>(
        &mut self,
        f: impl FnOnce(
            &mut UserInterface<'_, Message, iced_material::Theme, Renderer>,
            &mut Renderer,
            mouse::Cursor,
            &mut Vec<Message>,
        ) -> T,
    ) -> T {
        let theme = self.app.theme();
        let mut ui = UserInterface::build(
            self.app.view(),
            self.size,
            std::mem::take(&mut self.cache),
            &mut self.renderer,
        );
        let mut messages = Vec::new();
        ui.update(
            &[Event::Window(
                window::Event::RedrawRequested(Instant::now()),
            )],
            self.cursor,
            &mut self.renderer,
            &mut iced::advanced::clipboard::Null,
            &mut messages,
        );
        ui.draw(
            &mut self.renderer,
            &theme,
            &iced::advanced::renderer::Style {
                text_color: theme.colors.on_surface,
            },
            self.cursor,
        );
        let result = f(&mut ui, &mut self.renderer, self.cursor, &mut messages);
        self.cache = ui.into_cache();
        for message in messages {
            self.delivered.push(message.clone());
            self.app.update(message);
        }
        result
    }

    fn event(&mut self, event: Event) {
        self.with_ui(|ui, renderer, cursor, messages| {
            ui.update(
                &[event],
                cursor,
                renderer,
                &mut iced::advanced::clipboard::Null,
                messages,
            );
        });
    }

    fn find(&mut self, label: &str) -> Option<Rectangle> {
        self.with_ui(|ui, renderer, _, _| {
            let mut selector = Selector::find(label);
            ui.operate(
                renderer,
                &mut iced::advanced::widget::operation::black_box(&mut selector),
            );
            match selector.finish() {
                iced::advanced::widget::operation::Outcome::Some(Some(target)) => {
                    target.visible_bounds()
                }
                _ => None,
            }
        })
    }

    fn find_id(&mut self, id: &str) -> Rectangle {
        self.with_ui(|ui, renderer, _, _| {
            let mut selector = iced_test::selector::id(id.to_owned()).find();
            ui.operate(
                renderer,
                &mut iced::advanced::widget::operation::black_box(&mut selector),
            );
            match selector.finish() {
                iced::advanced::widget::operation::Outcome::Some(Some(target)) => {
                    target.visible_bounds().expect("visible consumer id")
                }
                _ => panic!("missing consumer id: {id}"),
            }
        })
    }

    fn click_at(&mut self, position: Point) {
        self.cursor = mouse::Cursor::Available(position);
        self.event(Event::Mouse(mouse::Event::CursorMoved { position }));
        for event in simulator::click() {
            self.event(event);
        }
    }

    fn click(&mut self, label: &str) {
        let bounds = self
            .find(label)
            .unwrap_or_else(|| panic!("missing consumer control: {label}"));
        self.click_at(bounds.center());
    }

    fn key(&mut self, key: keyboard::key::Named) {
        for event in simulator::tap_key(key, None) {
            self.event(event);
        }
    }

    fn type_text(&mut self, value: &str) {
        for event in simulator::typewrite(value) {
            self.event(event);
        }
    }

    fn replace(&mut self, label: &str, value: &str) {
        self.click(label);
        self.event(Event::Keyboard(keyboard::Event::ModifiersChanged(
            keyboard::Modifiers::COMMAND,
        )));
        for mut event in simulator::tap_key(keyboard::Key::Character("a".into()), None) {
            if let Event::Keyboard(
                keyboard::Event::KeyPressed { modifiers, .. }
                | keyboard::Event::KeyReleased { modifiers, .. },
            ) = &mut event
            {
                *modifiers = keyboard::Modifiers::COMMAND;
            }
            self.event(event);
        }
        self.event(Event::Keyboard(keyboard::Event::ModifiersChanged(
            keyboard::Modifiers::empty(),
        )));
        if value.is_empty() {
            self.key(keyboard::key::Named::Backspace);
        } else {
            self.type_text(value);
        }
    }

    fn frame(&mut self, scale: f32) -> Vec<u8> {
        let size = Size::new(
            (self.size.width * scale) as u32,
            (self.size.height * scale) as u32,
        );
        let background = self.app.theme().colors.surface;
        self.with_ui(|_, renderer, _, _| renderer.screenshot(size, scale, background))
    }

    fn capture(&mut self, path: &Path, scale: f32) {
        let pixels = self.frame(scale);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut encoder = png::Encoder::new(
            std::io::BufWriter::new(std::fs::File::create(path).unwrap()),
            (self.size.width * scale) as u32,
            (self.size.height * scale) as u32,
        );
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&pixels)
            .unwrap();
    }
}

#[test]
fn editing_validation_native_input_and_save_use_the_consumer_message_loop() {
    let mut client = Client::new(false, Size::new(1000.0, 900.0));
    client.click("Edit workspace");
    let name_position = client.find("Northstar studio").unwrap().center();
    client.replace("Northstar studio", "");
    client.click("Save changes");
    assert_eq!(client.app.save_count, 0);
    client.click_at(name_position);
    client.type_text("Juniper studio");
    assert_eq!(client.app.draft.name, "Juniper studio");
    client.replace("studio@example.com", "juniper@example.org");
    client.replace("Weekly planning on Monday", "Planning moved to Tuesday");
    client.click("Editor");
    client.click("Viewer");
    client.click("Save changes");
    assert!(!client.app.editing);
    assert_eq!(client.app.save_count, 1);
    assert_eq!(client.app.saved.access, Access::Viewer);
    assert_eq!(client.app.saved.note, "Planning moved to Tuesday");
    assert_eq!(client.app.saved.email, "juniper@example.org");
    assert!(client.find("Workspace changes saved").is_some());
    assert_eq!(
        client
            .delivered
            .iter()
            .filter(|m| matches!(m, Message::Save))
            .count(),
        1
    );
}

#[test]
fn foreign_theme_popup_and_buttons_keep_their_actions_inside_the_modal() {
    for dark in [false, true] {
        let mut client = Client::new(dark, Size::new(1000.0, 900.0));
        let background = client.find("Refresh activity").unwrap().center();
        client.click("Edit workspace");
        let digest = client.find_id("digest-selector");
        client.click_at(digest.center());
        // Native PickList paints menu labels directly rather than exposing text
        // operations. This fixture opens its two-row menu above the anchor.
        client.click_at(Point::new(
            digest.center_x(),
            digest.y - 1.5 * digest.height,
        ));
        assert_eq!(client.app.draft.digest, Digest::Daily);
        client.click("Use team defaults");
        assert_eq!(client.app.draft.digest, Digest::Weekly);
        client.click_at(background);
        assert_eq!(
            client.app.refreshes, 0,
            "covered native button cannot activate"
        );
        client.key(keyboard::key::Named::Escape);
        assert!(!client.app.editing);
        client.click("Refresh activity");
        assert_eq!(client.app.refreshes, 1);
    }
}

#[test]
fn nested_discard_restores_the_native_editor_focus_and_preserves_drafts() {
    let mut client = Client::new(true, Size::new(1000.0, 900.0));
    client.click("Edit workspace");
    client.replace("Weekly planning on Monday", "Draft");
    client.key(keyboard::key::Named::Escape);
    assert!(client.app.confirming_discard);
    client.type_text("must not reach the editor");
    assert_eq!(client.app.draft.note, "Draft");
    client.key(keyboard::key::Named::Escape);
    assert!(!client.app.confirming_discard);
    client.type_text(" restored");
    assert_eq!(client.app.draft.note, "Draft restored");
    client.key(keyboard::key::Named::Escape);
    client.click("Discard changes");
    assert!(!client.app.editing);
    assert_eq!(client.app.draft, client.app.saved);
    assert_eq!(client.app.save_count, 0);
}

#[test]
fn navigation_theme_changes_and_custom_surfaces_preserve_application_values() {
    let mut client = Client::new(false, Size::new(1000.0, 800.0));
    client.click("Preferences");
    client.replace("Explore a new direction", "A new collection");
    client.click("Dark appearance");
    assert!(client.app.dark);
    assert_eq!(client.app.scratch, "A new collection");
    let dark = client.frame(1.25);
    client.click("Overview");
    client.click("Preferences");
    assert_eq!(client.app.page, Page::Preferences);
    assert!(client.find("A new collection").is_some());
    client.click("Dark appearance");
    assert_ne!(dark, client.frame(1.25));
    assert_eq!(client.app.scratch, "A new collection");
}

#[test]
fn keyboard_traversal_activates_material_controls_across_the_consumer_boundary() {
    let mut client = Client::new(false, Size::new(1000.0, 900.0));
    client.key(keyboard::key::Named::Tab);
    client.key(keyboard::key::Named::Enter);
    assert!(client.app.editing);
    client.type_text(" plus");
    assert!(client.app.draft.name.contains("plus"));
    client.key(keyboard::key::Named::Tab);
    client.type_text("test");
    assert!(client.app.draft.email.contains("test"));
    client.key(keyboard::key::Named::Escape);
    assert!(client.app.confirming_discard);
    client.key(keyboard::key::Named::Enter);
    assert!(
        !client.app.confirming_discard,
        "initial confirmation focus keeps editing"
    );
    assert!(client.app.editing);
}

#[test]
#[ignore = "consumer review captures; no golden replacement or private test clock"]
fn capture_consumer_review() {
    let root = std::env::var_os("CONSUMER_CAPTURES")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("target/review"));
    for dark in [false, true] {
        for (width, scale) in [(420, 1.0), (1000, 1.25), (1000, 2.0)] {
            let mut client = Client::new(dark, Size::new(width as f32, 900.0));
            let name = format!("{}-{width}-{scale}", if dark { "dark" } else { "light" });
            client.capture(&root.join(format!("{name}-overview.png")), scale);
            client.click("Preferences");
            client.capture(&root.join(format!("{name}-preferences.png")), scale);
            client.click("Edit workspace");
            client.capture(&root.join(format!("{name}-editor.png")), scale);
            client.app.update(Message::Name("A new studio".into()));
            client.key(keyboard::key::Named::Escape);
            client.capture(&root.join(format!("{name}-discard.png")), scale);
        }
    }
}
