use iced::{Length, font::Weight, widget};
use iced_m3::{
    ButtonShape, ButtonSize, ButtonVariant, Element, ShapeScheme, Tab, Theme, TypeScale,
};

#[derive(Default)]
struct App {
    dark: bool,
    custom_type: bool,
    larger_text: bool,
    compact_corners: bool,
    name: String,
    notifications: bool,
    selected: u8,
    value: f32,
}

#[derive(Debug, Clone)]
enum Message {
    Dark(bool),
    CustomType(bool),
    LargerText(bool),
    CompactCorners(bool),
    Name(String),
    Notifications(bool),
    Select(u8),
    Value(f32),
}

impl App {
    fn theme(&self) -> Theme {
        let mut theme = if self.dark {
            Theme::dark()
        } else {
            Theme::light()
        };
        if self.larger_text {
            theme.typography = theme.typography.scaled(1.25);
        }
        if self.custom_type {
            theme.typography[TypeScale::HeadlineMedium].font.weight = Weight::Bold;
            theme.typography[TypeScale::LabelLarge].font.weight = Weight::Bold;
            theme.typography[TypeScale::TitleSmall].font.weight = Weight::Bold;
        }
        if self.compact_corners {
            theme.shapes = ShapeScheme {
                extra_small: 0.0,
                small: 4.0,
                medium: 4.0,
                large: 8.0,
                large_increased: 8.0,
                extra_large: 12.0,
                extra_large_increased: 20.0,
                extra_extra_large: 24.0,
            };
        }
        theme
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Dark(value) => self.dark = value,
            Message::CustomType(value) => self.custom_type = value,
            Message::LargerText(value) => self.larger_text = value,
            Message::CompactCorners(value) => self.compact_corners = value,
            Message::Name(value) => self.name = value,
            Message::Notifications(value) => self.notifications = value,
            Message::Select(value) => self.selected = value,
            Message::Value(value) => self.value = value,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let theme = self.theme();
        let material = theme.widgets();
        let controls = widget::row![
            material
                .switch(self.dark)
                .label("Dark")
                .on_toggle(Message::Dark),
            material
                .switch(self.custom_type)
                .label("Custom type")
                .on_toggle(Message::CustomType),
            material
                .switch(self.larger_text)
                .label("125% text")
                .on_toggle(Message::LargerText),
            material
                .switch(self.compact_corners)
                .label("Compact corners")
                .on_toggle(Message::CompactCorners),
        ]
        .spacing(16);
        let actions = widget::row![
            material
                .button("Save changes")
                .on_press(Message::Notifications(true)),
            material
                .button("Square recipe")
                .size(ButtonSize::Medium)
                .shape(ButtonShape::Square)
                .on_press(Message::Notifications(false)),
            material
                .button("Explicit 4px corners")
                .radius(4.0)
                .variant(ButtonVariant::Outlined)
                .on_press(Message::Select(0)),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center);
        let form = iced_m3::surface(
            widget::column![
                material.emphasized("Workspace settings", TypeScale::TitleLarge),
                material
                    .text_field("Workspace name", &self.name)
                    .on_input(Message::Name)
                    .supporting_text("Editing state survives typography changes"),
                material
                    .select(
                        "Visibility",
                        [
                            iced_m3::SelectOption::new(0, "Private"),
                            iced_m3::SelectOption::new(1, "Team")
                        ],
                        Some(self.selected)
                    )
                    .on_select(Message::Select),
                material
                    .checkbox(self.notifications)
                    .label("Notify me about changes")
                    .on_toggle(Message::Notifications),
                actions,
            ]
            .spacing(16),
        );
        iced_m3::focus::scope(widget::column![
            controls,
            widget::scrollable(widget::column![
                material.typography("Typography and shape schemes", TypeScale::HeadlineMedium),
                material.typography("Built-in labels share an explicit theme snapshot. Shapes resolve from the drawing theme.", TypeScale::BodyLarge),
                material.tabs([Tab::new(0, "Overview"), Tab::new(1, "Details")], Some(self.selected)).on_select(Message::Select),
                form,
                material.list_item("A themed list row").overline("TYPOGRAPHY").supporting_text("Headline, overline, and supporting roles stay independent."),
                material.slider(0.0..=100.0, self.value).labeled(true).value_label("A larger value label").on_change(Message::Value),
                material.menu("Open a themed menu", [iced_m3::MenuItem::new("First action", Message::Select(0)).supporting_text("Supporting text has its own role"), iced_m3::MenuItem::submenu("More actions", [iced_m3::MenuItem::new("Nested action", Message::Select(1))])]),
            ].spacing(24).width(Length::Fill)).height(Length::Fill),
        ].spacing(24).padding(24))
    }
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title("Material typography and shapes")
        .theme(App::theme)
        .default_font(iced_m3::fonts::REGULAR)
        .window_size((1040, 820))
        .run()
}
