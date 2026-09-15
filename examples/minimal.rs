use iced::widget::{column, container};
use iced_m3::{Element, Theme, button, text_field};

#[derive(Default)]
struct App {
    name: String,
    saved: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Name(String),
    Save,
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::Name(name) => {
                self.name = name;
                self.saved = false;
            }
            Message::Save => self.saved = true,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        iced_m3::focus::scope(
            container(
                column![
                    text_field("Your name", &self.name).on_input(Message::Name),
                    button(if self.saved { "Saved" } else { "Save" })
                        .on_press(Message::Save)
                        .disabled(self.name.trim().is_empty()),
                ]
                .spacing(24),
            )
            .padding(32),
        )
    }
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .theme(|_: &App| Theme::light())
        .default_font(iced_m3::fonts::REGULAR)
        .title("Material example")
        .run()
}
