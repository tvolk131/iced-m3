//! Interactive comparison of shared springs, independent groups and action recipes.
use iced::{Length, widget};
use iced_m3::{
    ButtonShape, ButtonSize, ButtonVariant, Element, IconButtonWidth, MotionScheme, NavigationItem,
    Tab, Theme, TypeScale, button, button_group, checkbox, dialog, extended_fab, icon, icon_button,
    navigation_rail, slider, switch, tabs, typography,
};

struct App {
    dark: bool,
    reduced: bool,
    expressive: bool,
    spaced_selected: u8,
    connected_selected: u8,
    tab: u8,
    destination: u8,
    rail_expanded: bool,
    fab_extended: bool,
    notifications: bool,
    volume: f32,
    open: bool,
    activations: u32,
}
impl Default for App {
    fn default() -> Self {
        Self {
            dark: false,
            reduced: false,
            expressive: true,
            spaced_selected: 1,
            connected_selected: 1,
            tab: 0,
            destination: 0,
            rail_expanded: false,
            fab_extended: true,
            notifications: false,
            volume: 40.0,
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
    SelectSpaced(u8),
    SelectConnected(u8),
    Tab(u8),
    Destination(u8),
    ExpandRail(bool),
    ExtendFab(bool),
    Notifications(bool),
    Volume(f32),
    Action,
    Dialog(bool),
}
impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::Dark(value) => self.dark = value,
            Message::Reduced(value) => self.reduced = value,
            Message::Expressive(value) => self.expressive = value,
            Message::SelectSpaced(value) => self.spaced_selected = value,
            Message::SelectConnected(value) => self.connected_selected = value,
            Message::Tab(value) => self.tab = value,
            Message::Destination(value) => self.destination = value,
            Message::ExpandRail(value) => self.rail_expanded = value,
            Message::ExtendFab(value) => self.fab_extended = value,
            Message::Notifications(value) => self.notifications = value,
            Message::Volume(value) => self.volume = value,
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
    fn group(&self, connected: bool) -> Element<'_, Message> {
        let selected = if connected {
            self.connected_selected
        } else {
            self.spaced_selected
        };
        button_group((0..3).map(|i| {
            button(["Create", "Review", "Share"][i as usize])
                .selected(selected == i)
                .variant(ButtonVariant::Tonal)
                .on_press(if connected {
                    Message::SelectConnected(i)
                } else {
                    Message::SelectSpaced(i)
                })
        }))
        .connected(connected)
        .into()
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
        let header = widget::column![
            typography("Material 3 Expressive", TypeScale::HeadlineLarge),
            typography(
                "Hold, release early, or reverse a motion. Compare the spring schemes as you explore.",
                TypeScale::BodyLarge
            ),
            settings,
        ]
        .spacing(16);
        let rail = navigation_rail(
            [
                NavigationItem::new(0, "Home", icon(widget::svg::Handle::from_memory(HOME))),
                NavigationItem::new(1, "Notes", icon(widget::svg::Handle::from_memory(NOTE))),
                NavigationItem::new(2, "Saved", icon(widget::svg::Handle::from_memory(STAR))),
            ],
            Some(self.destination),
        )
        .expanded(self.rail_expanded)
        .on_select(Message::Destination);
        let content = widget::column![
            typography("Button groups", TypeScale::TitleLarge),
            typography(
                "Each row has its own selection. Hold a button to expand it and compress its neighbors.",
                TypeScale::BodyMedium
            ),
            widget::column![
                typography("Spaced", TypeScale::LabelLarge),
                self.group(false),
                typography("Connected", TypeScale::LabelLarge),
                self.group(true),
            ].spacing(12),
            typography("Moving tab indicator", TypeScale::TitleLarge),
            typography(
                "Jump between the first and last tabs, then reverse before the indicator settles.",
                TypeScale::BodyMedium
            ),
            tabs(
                [Tab::new(0, "Overview"), Tab::new(1, "Activity"), Tab::new(2, "History")],
                Some(self.tab),
            ).on_select(Message::Tab),
            typography("Expanding navigation rail", TypeScale::TitleLarge),
            checkbox(self.rail_expanded).label("Expand rail").on_toggle(Message::ExpandRail),
            widget::row![
                rail,
                widget::container(widget::column![
                    typography(["Home", "Notes", "Saved"][self.destination as usize], TypeScale::HeadlineSmall),
                    typography(
                        "Toggle the rail again while it moves. This content follows its animated width.",
                        TypeScale::BodyLarge
                    ),
                ].spacing(12)).padding(24).width(Length::Fill),
            ].height(260),
            typography("Extended FAB reveal", TypeScale::TitleLarge),
            typography(
                "Collapse and restore the label. Its reveal is bounded, so the width stays within its endpoints.",
                TypeScale::BodyMedium
            ),
            checkbox(self.fab_extended).label("Show FAB label").on_toggle(Message::ExtendFab),
            extended_fab(icon(widget::svg::Handle::from_memory(PLUS)), "New note")
                .extended(self.fab_extended)
                .on_press(Message::Dialog(true)),
            typography("Selection and press feedback", TypeScale::TitleLarge),
            typography(
                "Hold the switch to see its thumb react. Drag the slider to compare its handle and value bubble.",
                TypeScale::BodyMedium
            ),
            switch(self.notifications).label("Notifications").on_toggle(Message::Notifications),
            slider(0.0..=100.0, self.volume)
                .step(1.0)
                .value_label(format!("{}%", self.volume as u32))
                .on_change(Message::Volume),
            typography("Dialog presence and action recipes", TypeScale::TitleLarge),
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
        // Keep the controls visible while scrolling, and keep every demo mounted
        // across state changes so interrupted animations retain their velocity.
        let background = widget::column![
            widget::container(header).padding(32).width(Length::Fill),
            widget::scrollable(
                widget::container(content)
                    .padding([16, 32])
                    .width(Length::Fill)
            )
            .height(Length::Fill),
        ];
        let panel = dialog(widget::column![
            typography("Shared surface motion", TypeScale::HeadlineSmall),
            typography("This dialog uses the selected theme scheme. Close and reopen it quickly to test reversal.", TypeScale::BodyLarge),
        ].spacing(16)).actions(button("Close").on_press(Message::Dialog(false)))
            .on_dismiss(Message::Dialog(false)).width(480.0);
        iced_m3::focus::scope(dialog::modal(background, panel, self.open))
    }
}
const PLUS: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M12 4v16M4 12h16" fill="none" stroke="black" stroke-width="2"/></svg>"#;
const HOME: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="m3 10 9-7 9 7v11h-6v-8H9v8H3Z" fill="none" stroke="black" stroke-width="2" stroke-linejoin="round"/></svg>"#;
const NOTE: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M5 3h14v18H5ZM8 8h8M8 12h8M8 16h5" fill="none" stroke="black" stroke-width="2" stroke-linejoin="round"/></svg>"#;
const STAR: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="m12 3 2.8 5.7 6.2.9-4.5 4.4 1.1 6.2-5.6-3-5.6 3 1.1-6.2L3 9.6l6.2-.9Z" fill="none" stroke="black" stroke-width="2" stroke-linejoin="round"/></svg>"#;
fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .theme(App::theme)
        .default_font(iced_m3::fonts::REGULAR)
        .title("iced-m3 · Expressive actions and motion")
        .window_size((1160., 900.))
        .run()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn click(view: Element<'_, Message>, label: &str) -> Vec<Message> {
        let mut ui = iced_test::Simulator::with_size(
            Default::default(),
            iced::Size::new(1160.0, 3000.0),
            view,
        );
        ui.click(label).unwrap();
        let messages: Vec<_> = ui.into_messages().collect();
        assert_eq!(messages.len(), 1, "one activation for {label}");
        messages
    }

    #[test]
    fn group_clicks_only_change_their_own_row_selection() {
        let mut app = App::default();
        for (connected, label, expected) in [
            (false, "Create", (0, 1)),
            (true, "Share", (0, 2)),
            (false, "Review", (1, 2)),
            (true, "Create", (1, 0)),
        ] {
            // Use the same row builder as the full view to exercise its actual
            // callbacks, including a rebuild after each interaction.
            for message in click(app.group(connected), label) {
                app.update(message);
            }
            assert_eq!((app.spaced_selected, app.connected_selected), expected);
        }
    }

    #[test]
    fn playground_controls_work_independently_through_view_rebuilds() {
        let mut app = App::default();
        for label in [
            "Expand rail",
            "Notes",
            "History",
            "Show FAB label",
            "Notifications",
            "Expressive springs",
            "Reduced motion",
            "Dark theme",
        ] {
            for message in click(app.view(), label) {
                app.update(message);
            }
        }
        assert!(app.rail_expanded);
        assert_eq!(app.destination, 1);
        assert_eq!(app.tab, 2);
        assert!(!app.fab_extended);
        assert!(app.notifications);
        assert!(!app.expressive);
        assert!(app.reduced);
        assert!(app.dark);
        assert_eq!((app.spaced_selected, app.connected_selected), (1, 1));

        for label in ["Expand rail", "Show FAB label", "Overview"] {
            for message in click(app.view(), label) {
                app.update(message);
            }
        }
        assert!(!app.rail_expanded);
        assert!(app.fab_extended);
        assert_eq!(app.tab, 0);
        assert_eq!(app.destination, 1);
        assert!(app.notifications);

        for message in click(app.view(), "New note") {
            app.update(message);
        }
        assert!(app.open);
        for message in click(app.view(), "Close") {
            app.update(message);
        }
        assert!(!app.open);
    }
}
