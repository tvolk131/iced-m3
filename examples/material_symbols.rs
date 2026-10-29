//! cargo run --locked --example material_symbols --features material-symbols
use iced::widget::{column, container, row};
use iced::{Length, Size};
use iced_m3::material_symbols::{Fill, IconName, OpticalSize, Style, Symbol};
use iced_m3::{Element, NavigationItem, Theme, TypeScale, navigation_rail, typography};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Page {
    #[default]
    Home,
    Inbox,
    Settings,
}

#[derive(Default)]
struct App {
    page: Page,
}

#[derive(Debug, Clone)]
enum Message {
    Select(Page),
}

impl App {
    fn update(&mut self, Message::Select(page): Message) {
        self.page = page;
    }

    fn view(&self) -> Element<'_, Message> {
        let items = [
            (Page::Home, "Home", IconName::Home),
            (Page::Inbox, "Inbox", IconName::Inbox),
            (Page::Settings, "Settings", IconName::Settings),
        ]
        .map(|(page, label, name)| {
            let symbol = Symbol::new(name).style(Style::Rounded);
            NavigationItem::new(page, label, symbol).selected_icon(symbol.fill(Fill::Filled))
        });
        let title = match self.page {
            Page::Home => "Home",
            Page::Inbox => "Inbox",
            Page::Settings => "Settings",
        };
        iced_m3::focus::scope(
            row![
                navigation_rail(items, Some(self.page)).on_select(Message::Select),
                container(
                    column![
                        Symbol::new(IconName::Home)
                            .style(Style::Rounded)
                            .optical_size(OpticalSize::Size48)
                            .icon()
                            .size(48),
                        typography(title, TypeScale::HeadlineMedium),
                        typography(
                            "Official Material Symbols, available offline.",
                            TypeScale::BodyLarge
                        ),
                    ]
                    .spacing(16)
                )
                .padding(32)
                .width(Length::Fill),
            ]
            .height(Length::Fill),
        )
    }
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .theme(|_: &App| Theme::light())
        .default_font(iced_m3::fonts::REGULAR)
        .window_size(Size::new(640.0, 400.0))
        .title("Material Symbols navigation")
        .run()
}
