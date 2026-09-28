//! Interactive comparison of shared motion and explicit action recipes.
use iced::{Length, widget};
use iced_m3::{
    ButtonShape, ButtonSize, ButtonVariant, Element, IconButtonWidth, MotionScheme, Theme,
    TypeScale, button, button_group, checkbox, dialog, icon, icon_button, typography,
};

struct App {
    dark: bool,
    reduced: bool,
    expressive: bool,
    selected: u8,
    open: bool,
    activations: u32,
}
impl Default for App {
    fn default() -> Self {
        Self {
            dark: false,
            reduced: false,
            expressive: true,
            selected: 1,
            open: false,
            activations: 0,
        }
    }
}
#[derive(Debug, Clone)]
enum Message {
    Dark(bool),
    Reduced(bool),
    Expressive(bool),
    Select(u8),
    Action,
    Dialog(bool),
}
impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::Dark(value) => self.dark = value,
            Message::Reduced(value) => self.reduced = value,
            Message::Expressive(value) => self.expressive = value,
            Message::Select(value) => self.selected = value,
            Message::Action => self.activations += 1,
            Message::Dialog(value) => self.open = value,
        }
    }
    fn theme(&self) -> Theme {
        (if self.dark {
            Theme::dark()
        } else {
            Theme::light()
        })
        .expressive()
        .motion_scheme(if self.expressive {
            MotionScheme::expressive()
        } else {
            MotionScheme::standard()
        })
        .reduced_motion(self.reduced)
    }
    fn view(&self) -> Element<'_, Message> {
        let settings = widget::row![
            checkbox(self.expressive)
                .label("Expressive springs")
                .on_toggle(Message::Expressive),
            checkbox(self.reduced)
                .label("Reduced motion")
                .on_toggle(Message::Reduced),
            checkbox(self.dark)
                .label("Dark theme")
                .on_toggle(Message::Dark),
        ]
        .spacing(24);
        let sizes = [
            ButtonSize::ExtraSmall,
            ButtonSize::Small,
            ButtonSize::Medium,
            ButtonSize::Large,
            ButtonSize::ExtraLarge,
        ];
        let recipes = widget::Column::with_children(sizes.into_iter().map(|size| {
            widget::row![
                button(format!("{size:?}"))
                    .size(size)
                    .on_press(Message::Action),
                button("Square")
                    .size(size)
                    .shape(ButtonShape::Square)
                    .variant(ButtonVariant::Tonal)
                    .on_press(Message::Action),
                widget::Row::with_children(
                    [
                        IconButtonWidth::Narrow,
                        IconButtonWidth::Default,
                        IconButtonWidth::Wide
                    ]
                    .into_iter()
                    .map(|width| {
                        icon_button(
                            icon(widget::svg::Handle::from_memory(PLUS))
                                .size(size.icon_button_icon_size()),
                        )
                        .size(size)
                        .icon_width(width)
                        .variant(ButtonVariant::Tonal)
                        .on_press(Message::Action)
                        .into()
                    })
                )
                .spacing(12),
            ]
            .spacing(20)
            .align_y(iced::Alignment::Center)
            .into()
        }))
        .spacing(24);
        let group = |connected| {
            button_group((0..3).map(|i| {
                button(["Create", "Review", "Share"][i as usize])
                    .selected(self.selected == i)
                    .variant(ButtonVariant::Tonal)
                    .on_press(Message::Select(i))
            }))
            .connected(connected)
        };
        let content = widget::column![
            typography("Material 3 Expressive", TypeScale::HeadlineLarge),
            typography(
                "Hold an action, release early, or switch motion schemes to compare the response.",
                TypeScale::BodyLarge
            ),
            settings,
            typography(
                "Shared springs · explicit sizes and shapes",
                TypeScale::TitleLarge
            ),
            group(false),
            group(true),
            widget::row![
                button("Open dialog").on_press(Message::Dialog(true)),
                button("No shape feedback")
                    .expressive(false)
                    .on_press(Message::Action),
                typography(
                    format!("{} activations", self.activations),
                    TypeScale::BodyMedium
                ),
            ]
            .spacing(20)
            .align_y(iced::Alignment::Center),
            recipes,
        ]
        .spacing(24)
        .max_width(1100);
        let background =
            widget::scrollable(widget::container(content).padding(32).width(Length::Fill));
        let panel = dialog(widget::column![
            typography("Shared surface motion", TypeScale::HeadlineSmall),
            typography("This dialog uses the selected theme scheme. Close and reopen it quickly to test reversal.", TypeScale::BodyLarge),
        ].spacing(16)).actions(button("Close").on_press(Message::Dialog(false)))
            .on_dismiss(Message::Dialog(false)).width(480.0);
        iced_m3::focus::scope(dialog::modal(background, panel, self.open))
    }
}
const PLUS: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M12 4v16M4 12h16" fill="none" stroke="black" stroke-width="2"/></svg>"#;
fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .theme(App::theme)
        .default_font(iced_m3::fonts::REGULAR)
        .title("iced-m3 · Expressive actions and motion")
        .window_size((1160., 900.))
        .run()
}
