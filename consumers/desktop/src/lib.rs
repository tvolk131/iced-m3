//! A standalone client of iced-m3: application state stays in this crate.
mod metrics;

use iced::{Alignment, Color, Length, widget};
use iced_m3::{
    ButtonVariant, Element, SelectOption, SurfaceVariant, Tab, Theme, TypeScale, app_bar, button,
    card, checkbox, dialog, focus, linear_progress, loading_indicator, select, snackbar, switch,
    tabs, text_field, typography,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Overview,
    Preferences,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    Viewer,
    Editor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Digest {
    Daily,
    Weekly,
}
impl std::fmt::Display for Digest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Daily => "Daily digest",
            Self::Weekly => "Weekly digest",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub name: String,
    pub email: String,
    pub note: String,
    pub access: Access,
    pub digest: Digest,
    pub notifications: bool,
}
impl Default for Workspace {
    fn default() -> Self {
        Self {
            name: "Northstar studio".into(),
            email: "studio@example.com".into(),
            note: "Weekly planning on Monday".into(),
            access: Access::Editor,
            digest: Digest::Weekly,
            notifications: true,
        }
    }
}
impl Workspace {
    pub fn valid(&self) -> bool {
        !self.name.trim().is_empty()
            && self.email.split_once('@').is_some_and(|(user, host)| {
                !user.trim().is_empty() && host.contains('.') && !host.ends_with('.')
            })
    }
}

pub struct Studio {
    pub page: Page,
    pub saved: Workspace,
    pub draft: Workspace,
    pub editing: bool,
    pub confirming_discard: bool,
    pub dark: bool,
    pub reduced_motion: bool,
    pub scratch: String,
    pub refreshes: u32,
    pub save_count: u32,
    pub notice: String,
    pub notice_visible: bool,
    pub notice_id: u64,
}
impl Default for Studio {
    fn default() -> Self {
        Self {
            page: Page::Overview,
            saved: Workspace::default(),
            draft: Workspace::default(),
            editing: false,
            confirming_discard: false,
            dark: false,
            reduced_motion: false,
            scratch: "Explore a new direction".into(),
            refreshes: 0,
            save_count: 0,
            notice: String::new(),
            notice_visible: false,
            notice_id: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Page(Page),
    Edit,
    Name(String),
    Email(String),
    Note(String),
    Access(Access),
    Digest(Digest),
    Notifications(bool),
    TeamDefaults,
    Cancel,
    KeepEditing,
    Discard,
    Save,
    Dark(bool),
    ReducedMotion(bool),
    Scratch(String),
    Refresh,
    HideNotice,
}

impl Studio {
    pub fn theme(&self) -> Theme {
        Theme::from_accent(Color::from_rgb8(28, 112, 109), self.dark)
            .reduced_motion(self.reduced_motion)
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Page(page) => self.page = page,
            Message::Edit => {
                self.draft = self.saved.clone();
                self.editing = true;
                self.confirming_discard = false;
            }
            Message::Name(value) => self.draft.name = value,
            Message::Email(value) => self.draft.email = value,
            Message::Note(value) => self.draft.note = value,
            Message::Access(value) => self.draft.access = value,
            Message::Digest(value) => self.draft.digest = value,
            Message::Notifications(value) => self.draft.notifications = value,
            Message::TeamDefaults => {
                self.draft.digest = Digest::Weekly;
                self.draft.notifications = true;
            }
            Message::Cancel => {
                if self.draft != self.saved {
                    self.confirming_discard = true;
                } else {
                    self.editing = false;
                }
            }
            Message::KeepEditing => self.confirming_discard = false,
            Message::Discard => {
                self.confirming_discard = false;
                self.editing = false;
                self.draft = self.saved.clone();
            }
            Message::Save if self.editing && self.draft.valid() => {
                self.saved = self.draft.clone();
                self.save_count += 1;
                self.editing = false;
                self.show_notice("Workspace changes saved");
            }
            Message::Save => {}
            Message::Dark(value) => self.dark = value,
            Message::ReducedMotion(value) => self.reduced_motion = value,
            Message::Scratch(value) => self.scratch = value,
            Message::Refresh => {
                self.refreshes += 1;
                self.show_notice("Activity is up to date");
            }
            Message::HideNotice => self.notice_visible = false,
        }
    }

    fn show_notice(&mut self, value: &str) {
        self.notice = value.into();
        self.notice_visible = true;
        self.notice_id += 1;
    }

    pub fn view(&self) -> Element<'_, Message> {
        let navigation = tabs(
            [
                Tab::new(Page::Overview, "Overview"),
                Tab::new(Page::Preferences, "Preferences"),
            ],
            Some(self.page),
        )
        .on_select(Message::Page);
        let content = match self.page {
            Page::Overview => self.overview(),
            Page::Preferences => self.preferences(),
        };
        let base = widget::column![
            app_bar("Northstar Studio").action(
                button("Edit workspace")
                    .variant(ButtonVariant::Text)
                    .on_press(Message::Edit),
            ),
            navigation,
            widget::scrollable(widget::container(content).padding(24).max_width(1100))
                .height(Length::Fill),
        ]
        .height(Length::Fill);
        let notices = snackbar::host(
            base,
            Some(
                snackbar(self.notice.clone())
                    .id(self.notice_id)
                    .visible(self.notice_visible)
                    .on_dismiss(Message::HideNotice),
            ),
        );
        focus::scope(dialog::stack(
            notices,
            [
                (self.editor(), self.editing),
                (self.discard_dialog(), self.confirming_discard),
            ],
        ))
    }

    fn overview(&self) -> Element<'_, Message> {
        let native_action: iced::Element<'_, Message> = widget::button("Refresh activity")
            .padding([10, 16])
            .on_press(Message::Refresh)
            .into();
        widget::column![
            typography("A little space for good work.", TypeScale::HeadlineMedium),
            typography(
                "Your projects and people, together in one studio.",
                TypeScale::Body
            ),
            card(
                widget::column![
                    typography(self.saved.name.clone(), TypeScale::TitleLarge),
                    typography(self.saved.email.clone(), TypeScale::Body),
                    metrics::summary(self.refreshes),
                ]
                .spacing(20)
            )
            .variant(SurfaceVariant::Filled),
            card(
                widget::column![
                    widget::row![
                        typography("Website refresh", TypeScale::TitleMedium),
                        widget::space().width(Length::Fill),
                        typography("In review", TypeScale::LabelLarge),
                    ]
                    .align_y(Alignment::Center),
                    typography(
                        "Four of six milestones are ready for the team.",
                        TypeScale::Body
                    ),
                    linear_progress(0.67).wavy(true),
                ]
                .spacing(16)
            )
            .variant(SurfaceVariant::Outlined),
            widget::row![
                loading_indicator().size(32.0),
                typography("Keeping your studio connected", TypeScale::BodyMedium),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            widget::themer(Some(self.theme().iced()), native_action),
            typography(
                "This demo keeps all changes in memory.",
                TypeScale::BodySmall
            ),
        ]
        .spacing(24)
        .into()
    }

    fn preferences(&self) -> Element<'_, Message> {
        // An intentional custom parent, independent of standard surface tokens.
        let panel = widget::container(
            widget::column![
                typography("Make room for your next idea", TypeScale::TitleLarge),
                text_field("Working idea", &self.scratch)
                    .on_input(Message::Scratch)
                    .supporting_text("This draft stays here as you move between pages."),
                switch(self.dark)
                    .label("Dark appearance")
                    .on_toggle(Message::Dark),
                checkbox(self.reduced_motion)
                    .label("Reduce motion")
                    .on_toggle(Message::ReducedMotion),
            ]
            .spacing(24),
        )
        .padding(24)
        .width(Length::Fill)
        .style(|theme: &Theme| widget::container::Style {
            background: Some(
                iced::gradient::Linear::new(iced::Radians(0.5))
                    .add_stop(0.0, theme.colors.surface_container_highest)
                    .add_stop(1.0, theme.colors.secondary_container)
                    .into(),
            ),
            text_color: Some(theme.colors.on_surface),
            border: iced::Border {
                radius: 16.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });
        widget::column![
            typography("Your studio, your rhythm.", TypeScale::HeadlineMedium),
            panel,
            typography("Workspace details", TypeScale::TitleMedium),
            typography(
                format!("{} · {}", self.saved.name, self.saved.digest),
                TypeScale::Body
            ),
        ]
        .spacing(24)
        .into()
    }

    fn editor(&self) -> iced_m3::Dialog<'_, Message> {
        let email = text_field("Contact email", &self.draft.email).on_input(Message::Email);
        let email = if self.draft.valid() || self.draft.email == self.saved.email {
            email.supporting_text("Used for studio updates.")
        } else {
            email.error("Enter an address such as studio@example.com.")
        };
        // Controls requiring iced::Theme live inside a small native-themed subtree.
        // Its popup, operations and application messages still cross the boundary.
        let native: iced::Element<'_, Message> = widget::row![
            widget::container(
                widget::pick_list(
                    [Digest::Daily, Digest::Weekly],
                    Some(self.draft.digest),
                    Message::Digest
                )
                .width(Length::Fill)
            )
            .id("digest-selector")
            .width(Length::Fill),
            widget::button("Use team defaults")
                .on_press(Message::TeamDefaults)
                .padding(10),
        ]
        .spacing(12)
        .wrap()
        .into();
        dialog(
            widget::column![
                typography("Edit your workspace", TypeScale::HeadlineSmall),
                typography(
                    "Give your team a place that feels like yours.",
                    TypeScale::BodyMedium
                ),
                text_field("Workspace name", &self.draft.name).on_input(Message::Name),
                email,
                select(
                    "Workspace access",
                    [
                        SelectOption::new(Access::Viewer, "Viewer"),
                        SelectOption::new(Access::Editor, "Editor"),
                    ],
                    Some(self.draft.access)
                )
                .on_select(Message::Access),
                widget::column![
                    typography("Team note", TypeScale::LabelLarge),
                    widget::text_input("Add a note", &self.draft.note)
                        .on_input(Message::Note)
                        .padding(12),
                ]
                .spacing(8),
                widget::themer(Some(self.theme().iced()), native),
                checkbox(self.draft.notifications)
                    .label("Workspace notifications")
                    .on_toggle(Message::Notifications),
            ]
            .spacing(20),
        )
        .actions(
            widget::row![
                button("Cancel")
                    .variant(ButtonVariant::Text)
                    .on_press(Message::Cancel),
                button("Save changes")
                    .on_press(Message::Save)
                    .disabled(!self.draft.valid()),
            ]
            .spacing(12)
            .wrap(),
        )
        .width(560.0)
        .max_height(600)
        .on_dismiss(Message::Cancel)
    }

    fn discard_dialog(&self) -> iced_m3::Dialog<'_, Message> {
        dialog(
            widget::column![
                typography("Discard your changes?", TypeScale::HeadlineSmall),
                typography(
                    "Your saved workspace will stay as it was.",
                    TypeScale::BodyMedium
                ),
                dialog::actions(
                    widget::row![
                        button("Keep editing")
                            .variant(ButtonVariant::Text)
                            .on_press(Message::KeepEditing),
                        button("Discard changes").on_press(Message::Discard),
                    ]
                    .spacing(12)
                    .wrap()
                ),
            ]
            .spacing(24),
        )
        .width(420.0)
        .on_dismiss(Message::KeepEditing)
    }
}
