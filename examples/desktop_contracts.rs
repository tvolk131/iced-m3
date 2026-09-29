//! Focused native smoke test: `cargo run --example desktop_contracts`.
//! Tab through hints and picker fields; use the theme controls to compare motion.
use iced::{Length, widget};
use iced_m3::{
    ButtonVariant, Element, SelectOption, Theme, Time, TimePart, TypeScale, button, button_group,
    checkbox, circular_progress, dialog, focus, rich_tooltip, select, time_picker, tooltip,
    typography,
};

struct App {
    dark: bool,
    reduced: bool,
    selected: u8,
    role: u8,
    hour: String,
    minute: String,
    pm: bool,
    open: bool,
    last: String,
    events: usize,
}
impl Default for App {
    fn default() -> Self {
        Self {
            dark: false,
            reduced: false,
            selected: 0,
            role: 0,
            hour: "10".into(),
            minute: "30".into(),
            pm: false,
            open: false,
            last: "No actions yet".into(),
            events: 0,
        }
    }
}
#[derive(Debug, Clone)]
enum Message {
    Dark(bool),
    Reduced(bool),
    Select(u8),
    Role(u8),
    Hour(String),
    Minute(String),
    Period(bool),
    Confirm(Time),
    Dialog(bool),
    Action(&'static str),
}
impl App {
    fn update(&mut self, message: Message) {
        self.events += 1;
        self.last = format!("{message:?}");
        match message {
            Message::Dark(v) => self.dark = v,
            Message::Reduced(v) => self.reduced = v,
            Message::Select(v) => self.selected = v,
            Message::Role(v) => self.role = v,
            Message::Hour(v) => self.hour = v,
            Message::Minute(v) => self.minute = v,
            Message::Period(v) => self.pm = v,
            Message::Dialog(v) => self.open = v,
            Message::Confirm(time) => self.last = format!("Confirmed {time}"),
            Message::Action(label) => self.last = label.into(),
        }
    }
    fn theme(&self) -> Theme {
        (if self.dark {
            Theme::dark()
        } else {
            Theme::light()
        })
        .expressive()
        .reduced_motion(self.reduced)
    }
    fn view(&self) -> Element<'_, Message> {
        let hints = widget::row![
            tooltip(
                button("Plain hint").on_press(Message::Action("Plain action")),
                "Shown after keyboard focus or hover"
            ),
            rich_tooltip(
                widget::text("Rich hint"),
                "Keyboard help",
                widget::column![
                    widget::text("Tab exits after the last action. Escape returns to the trigger."),
                    button("First action").on_press(Message::Action("First hint action")),
                    button("Last action").on_press(Message::Action("Last hint action")),
                ]
                .spacing(8)
            ),
            button("After hint").on_press(Message::Action("After hint")),
        ]
        .spacing(16);
        let groups = button_group(["Create", "Review", "Share"].into_iter().enumerate().map(
            |(i, label)| {
                button(label)
                    .variant(ButtonVariant::Tonal)
                    .selected(self.selected == i as u8)
                    .on_press(Message::Select(i as u8))
            },
        ))
        .connected(true);
        let background = widget::container(
            widget::column![
                typography("Desktop interaction check", TypeScale::HeadlineMedium),
                widget::row![
                    checkbox(self.dark)
                        .label("Dark theme")
                        .on_toggle(Message::Dark),
                    checkbox(self.reduced)
                        .label("Reduced motion")
                        .on_toggle(Message::Reduced)
                ]
                .spacing(24),
                hints,
                groups,
                widget::row![
                    time_picker(Time::new(10, 30).unwrap(), TimePart::Hour)
                        .input_mode(true)
                        .hour_input(&self.hour, Message::Hour)
                        .minute_input(&self.minute, Message::Minute)
                        .input_period(self.pm, Message::Period)
                        .on_confirm(Message::Confirm),
                    widget::column![
                        select(
                            "Access",
                            [
                                SelectOption::new(0, "Viewer"),
                                SelectOption::new(1, "Editor"),
                                SelectOption::new(2, "Locked").disabled(true)
                            ],
                            Some(self.role)
                        )
                        .on_select(Message::Role),
                        button("Open dialog").on_press(Message::Dialog(true)),
                        circular_progress(0.).indeterminate(true),
                        widget::text(
                            "The spinner stays active behind dialogs and in an unfocused window."
                        ),
                    ]
                    .spacing(24)
                    .width(Length::Fill),
                ]
                .spacing(24),
                widget::text(format!("{} events · {}", self.events, self.last)),
            ]
            .spacing(24),
        )
        .padding(32)
        .width(Length::Fill)
        .height(Length::Fill);
        let panel = dialog(widget::text(
            "Close and reopen to check interrupted motion.",
        ))
        .actions(button("Close").on_press(Message::Dialog(false)))
        .on_dismiss(Message::Dialog(false));
        focus::scope(dialog::modal(background, panel, self.open))
    }
}
fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .theme(App::theme)
        .default_font(iced_m3::fonts::REGULAR)
        .title("iced-m3 · Desktop contracts")
        .window_size((940., 700.))
        .run()
}
