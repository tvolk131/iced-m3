//! Run with `cargo run --example gallery`.
#[path = "gallery/icons.rs"]
mod icons;
#[cfg(test)]
#[allow(dead_code)]
#[path = "../tests/visual/reference.rs"]
mod reference;
#[cfg(test)]
#[allow(dead_code)] // Shared with the component suite, which uses interaction helpers too.
#[path = "../tests/visual/harness.rs"]
mod visual_harness;
#[cfg(test)]
fn with_time<T>(_: std::time::Instant, f: impl FnOnce() -> T) -> T {
    // Gallery references are settled compositions. Precise animation timing is
    // covered by the library unit suite, where its private test clock is available.
    f()
}
use iced::{
    Alignment, Color, Length, Size,
    widget::{self, column, container, responsive, row, scrollable, space},
};
use iced_m3::{
    AppBar, Button, ButtonVariant, Element, FabColor, FabSize, MenuItem, NavigationItem,
    NavigationLayout, RadioOption, Segment, SegmentSelection, SelectOption, SurfaceVariant, Tab,
    TabVariant, Theme, TypeScale, app_bar, assist_chip, badge, bottom_sheet, button, checkbox,
    chip, circular_progress, context_menu, dialog::dialog, divider, extended_fab, fab, filter_chip,
    icon_button, input_chip, linear_progress, list, list_item, menu, navigation_bar,
    navigation_rail, radio_group, range_slider, search_bar, segmented_buttons, select, sheet,
    side_sheet, slider, snackbar, suggestion_chip, surface, switch, tabs, text_field, tooltip,
    typography,
};
use icons::Icon;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Delivery {
    Immediately,
    Daily,
    Weekly,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Access {
    Viewer,
    Editor,
    Owner,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Page {
    #[default]
    Workspace,
    Activity,
    Settings,
}
impl Page {
    fn title(self) -> &'static str {
        match self {
            Self::Workspace => "Workspace",
            Self::Activity => "Activity",
            Self::Settings => "Settings",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum WorkspaceTab {
    #[default]
    Overview,
    Files,
    Components,
    Export,
    Schedule,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum ActivityTab {
    #[default]
    All,
    Unread,
    Archived,
}
#[derive(Clone)]
struct Gallery {
    navigation_modal_open: bool,
    rail_expanded: Option<bool>,
    editor_open: bool,
    editor_discard_open: bool,
    calendar_input: bool,
    editor_name: String,
    editor_note: String,
    dialog_kind: u8,
    reduced_motion: bool,
    high_contrast: bool,
    carousel_index: usize,
    fab_menu_open: bool,
    extended_action: bool,
    drag_height: f32,
    calendar_month: iced_m3::Date,
    calendar_selection: iced_m3::DateSelection,
    calendar_years: bool,
    calendar_range: bool,
    calendar_text: String,
    calendar_end: String,
    clock_time: iced_m3::Time,
    clock_part: iced_m3::TimePart,
    clock_text: String,
    clock24: bool,
    clock_input: bool,
    clock_hour: String,
    clock_minute: String,
    clock_pm: bool,
    amount: String,
    password: String,
    show_password: bool,
    card_dragged: bool,
    card_disabled: bool,
    schedule_input: bool,
    schedule_saved: String,
    name: String,
    email: String,
    note: String,
    long_value: String,
    autosave: bool,
    notifications: bool,
    required_choice: bool,
    selected: [bool; 3],
    dark: bool,
    accent: Color,
    accent_hex: String,
    outside: bool,
    open: bool,
    saved: bool,
    status: String,
    clicks: u32,
    compact: bool,
    delivery: Delivery,
    access: Access,
    notice: Option<(u64, String, bool)>,
    exiting_notice: Option<(u64, String, bool)>,
    notice_id: u64,
    undo_name: Option<String>,
    page: Page,
    workspace_tab: WorkspaceTab,
    activity_tab: ActivityTab,
    read: [bool; 3],
    pinned: bool,
    export_format: u8,
    export_quality: f32,
    export_options: Vec<u8>,
    export_tick: Option<u8>,
    export_id: u64,
    export_cancel: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    preview_value: f32,
    feedback_loading: bool,
    loading_paused: bool,
    traveling_wave: bool,
    feedback_selected: bool,
    feedback_range: (f32, f32),
    details_open: bool,
    details_index: usize,
    details_modal: bool,
    creating: bool,
    new_name: String,
    file_notes: [String; 2],
    search_open: bool,
    query: String,
    recent_searches: Vec<String>,
    filter_shared: bool,
    filter_pinned: bool,
    file_age: (f32, f32),
    sample_input: bool,
}
impl Default for Gallery {
    fn default() -> Self {
        Self {
            navigation_modal_open: false,
            rail_expanded: None,
            editor_open: false,
            editor_discard_open: false,
            calendar_input: false,
            editor_name: String::new(),
            editor_note: String::new(),
            dialog_kind: 0,
            reduced_motion: false,
            high_contrast: false,
            carousel_index: 0,
            fab_menu_open: false,
            extended_action: true,
            drag_height: 520.0,
            calendar_month: iced_m3::Date::new(2026, 9, 1).unwrap(),
            calendar_selection: iced_m3::DateSelection::Single(Some(
                iced_m3::Date::new(2026, 9, 18).unwrap(),
            )),
            calendar_years: false,
            calendar_range: false,
            calendar_text: "2026-09-18".into(),
            calendar_end: String::new(),
            clock_time: iced_m3::Time::new(10, 30).unwrap(),
            clock_part: iced_m3::TimePart::Hour,
            clock_text: "10:30".into(),
            clock24: false,
            clock_input: false,
            clock_hour: "10".into(),
            clock_minute: "30".into(),
            clock_pm: false,
            amount: "125.00".into(),
            password: "my-workspace".into(),
            show_password: false,
            card_dragged: false,
            card_disabled: false,
            schedule_input: false,
            schedule_saved: "No review scheduled yet".into(),
            name: "Studio workspace".into(),
            email: "hello@example.com".into(),
            note: String::new(),
            long_value: "Labels stay within the outline".into(),
            autosave: true,
            notifications: true,
            required_choice: false,
            selected: [true, false, true],
            dark: false,
            accent: Color::from_rgb8(103, 80, 164),
            accent_hex: "#6750A4".into(),
            outside: true,
            open: false,
            saved: true,
            status: "Ready when you are".into(),
            clicks: 3,
            compact: false,
            delivery: Delivery::Daily,
            access: Access::Editor,
            notice: None,
            exiting_notice: None,
            notice_id: 0,
            undo_name: None,
            page: Page::Workspace,
            workspace_tab: WorkspaceTab::Overview,
            activity_tab: ActivityTab::All,
            read: [false, true, false],
            pinned: false,
            export_format: 0,
            export_quality: 80.0,
            export_options: vec![0],
            export_tick: None,
            export_id: 0,
            export_cancel: None,
            preview_value: 0.4,
            feedback_loading: true,
            loading_paused: false,
            traveling_wave: false,
            feedback_selected: false,
            feedback_range: (48.0, 52.0),
            details_open: false,
            details_index: 0,
            details_modal: false,
            creating: false,
            new_name: String::new(),
            file_notes: [String::new(), String::new()],
            search_open: false,
            query: String::new(),
            recent_searches: vec!["Design".into(), "Checklist".into()],
            filter_shared: false,
            filter_pinned: false,
            file_age: (0.0, 30.0),
            sample_input: true,
        }
    }
}
#[derive(Clone, Debug)]
enum Message {
    NavigationModal(bool),
    RailExpanded(bool),
    EditorOpen,
    EditorCancel,
    EditorDiscard(bool),
    EditorKeep,
    CalendarInput,
    EditorSave,
    EditorName(String),
    EditorNote(String),
    ReducedMotion(bool),
    HighContrast(bool),
    Carousel(usize),
    FabMenu,
    ExtendedAction(bool),
    SheetHeight(f32),
    Calendar(iced_m3::DateSelection),
    CalendarMonth(iced_m3::Date),
    CalendarYears,
    CalendarRange(bool),
    CalendarText(String),
    CalendarEnd(String),
    Clock(iced_m3::Time),
    ClockPart(iced_m3::TimePart),
    ClockText(String),
    Clock24(bool),
    ClockInput,
    ClockHour(String),
    ClockMinute(String),
    ClockPeriod(bool),
    Amount(String),
    Password(String),
    ShowPassword,
    CardDragged(bool),
    CardDisabled(bool),
    ScheduleInput(bool),
    SaveSchedule,
    Page(Page),
    WorkspaceTab(WorkspaceTab),
    ActivityTab(ActivityTab),
    Read(usize, bool),
    Pin,
    Name(String),
    Email(String),
    Note(String),
    LongValue(String),
    Autosave(bool),
    Notifications(bool),
    RequiredChoice(bool),
    SelectAll(bool),
    Select(usize),
    Dark(bool),
    Accent(Color, &'static str),
    AccentHex(String),
    Outside(bool),
    Open,
    Close,
    Save,
    Action(&'static str),
    Compact,
    Delivery(Delivery),
    Access(Access),
    Duplicate,
    UndoDuplicate,
    ShowNotice,
    DismissNotice(u64),
    ExportFormat(SegmentSelection<u8>),
    ExportOptions(SegmentSelection<u8>),
    ExportQuality(f32),
    StartExport,
    CancelExport,
    ExportTick(u64, u8),
    PreviewValue(f32),
    LoadingPaused(bool),
    TravelingWave(bool),
    FeedbackLoading(bool),
    FeedbackSelected(bool),
    FeedbackRange((f32, f32)),
    FileDetails(usize),
    CloseDetails,
    ModalDetails,
    NewWorkspace,
    NewName(String),
    CreateWorkspace,
    CancelNew,
    FileNote(String),
    SearchOpen,
    SearchClose,
    Query(String),
    SearchSubmit,
    Suggest(String),
    FilterShared,
    FilterPinned,
    FileAge((f32, f32)),
    ClearFilters,
    SampleInput(bool),
}
impl Gallery {
    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::NavigationModal(open) => self.navigation_modal_open = open,
            Message::RailExpanded(expanded) => self.rail_expanded = Some(expanded),
            Message::EditorOpen => {
                self.dialog_kind = 2;
                self.editor_name = self.name.clone();
                self.editor_note = self.note.clone();
                self.editor_open = true;
                self.editor_discard_open = false;
            }
            Message::EditorCancel => {
                if self.editor_name != self.name || self.editor_note != self.note {
                    self.editor_discard_open = true;
                } else {
                    self.editor_open = false;
                }
            }
            Message::EditorDiscard(discard) => {
                self.editor_discard_open = false;
                if discard {
                    self.editor_open = false;
                }
            }
            Message::EditorKeep => self.editor_discard_open = false,
            Message::CalendarInput => self.calendar_input = !self.calendar_input,
            Message::EditorName(name) => self.editor_name = name,
            Message::EditorNote(note) => self.editor_note = note,
            Message::EditorSave => {
                if !self.editor_name.trim().is_empty() {
                    self.name = self.editor_name.trim().into();
                    self.note = self.editor_note.clone();
                    self.editor_open = false;
                    self.notify("Workspace details saved", false);
                }
            }
            Message::ReducedMotion(value) => self.reduced_motion = value,
            Message::HighContrast(value) => self.high_contrast = value,
            Message::Carousel(index) => self.carousel_index = index,
            Message::FabMenu => self.fab_menu_open = !self.fab_menu_open,
            Message::ExtendedAction(extended) => self.extended_action = extended,
            Message::SheetHeight(height) => self.drag_height = height,

            Message::Calendar(selection) => {
                self.calendar_selection = selection;
                match selection {
                    iced_m3::DateSelection::Single(date) => {
                        self.calendar_text = date.map(|d| d.to_string()).unwrap_or_default();
                        self.calendar_end.clear();
                    }
                    iced_m3::DateSelection::Range { start, end } => {
                        self.calendar_text = start.map(|d| d.to_string()).unwrap_or_default();
                        self.calendar_end = end.map(|d| d.to_string()).unwrap_or_default();
                    }
                }
            }
            Message::CalendarMonth(month) => self.calendar_month = month,
            Message::CalendarYears => self.calendar_years = !self.calendar_years,
            Message::CalendarRange(range) => {
                self.calendar_range = range;
                self.calendar_selection = if range {
                    iced_m3::DateSelection::Range {
                        start: self.calendar_text.parse().ok(),
                        end: None,
                    }
                } else {
                    iced_m3::DateSelection::Single(self.calendar_text.parse().ok())
                };
                self.calendar_end.clear();
            }
            Message::CalendarText(value) => {
                self.calendar_text = value;
                self.calendar_selection = if self.calendar_range {
                    iced_m3::DateSelection::Range {
                        start: self.calendar_text.parse().ok(),
                        end: self.calendar_end.parse().ok(),
                    }
                } else {
                    iced_m3::DateSelection::Single(self.calendar_text.parse().ok())
                };
                if let Ok(date) = self.calendar_text.parse::<iced_m3::Date>() {
                    self.calendar_month = date.first_of_month();
                }
            }
            Message::CalendarEnd(value) => {
                self.calendar_end = value;
                self.calendar_selection = iced_m3::DateSelection::Range {
                    start: self.calendar_text.parse().ok(),
                    end: self.calendar_end.parse().ok(),
                };
            }
            Message::Clock(time) => {
                self.clock_time = time;
                self.clock_text = time.to_string();
                self.sync_clock_draft();
                self.clock_input = false;
            }
            Message::ClockPart(part) => self.clock_part = part,
            Message::ClockText(value) => {
                self.clock_text = value;
                if let Ok(time) = self.clock_text.parse() {
                    self.clock_time = time;
                    self.sync_clock_draft();
                }
            }
            Message::Clock24(format) => {
                self.clock24 = format;
                self.sync_clock_draft();
            }
            Message::ClockInput => {
                self.clock_input = !self.clock_input;
            }
            Message::ClockHour(value) => self.clock_hour = value,
            Message::ClockMinute(value) => self.clock_minute = value,
            Message::ClockPeriod(pm) => self.clock_pm = pm,
            Message::Amount(value) => self.amount = value,
            Message::Password(value) => self.password = value,
            Message::ShowPassword => self.show_password = !self.show_password,
            Message::CardDragged(value) => self.card_dragged = value,
            Message::CardDisabled(value) => self.card_disabled = value,
            Message::ScheduleInput(input) => self.schedule_input = input,
            Message::SaveSchedule => {
                if self.schedule_valid() {
                    self.schedule_saved = format!(
                        "Review scheduled for {}{} at {}",
                        self.calendar_text,
                        if self.calendar_range {
                            format!(" – {}", self.calendar_end)
                        } else {
                            String::new()
                        },
                        self.clock_text
                    );
                    let message = self.schedule_saved.clone();
                    self.notify(&message, false);
                }
            }

            Message::SampleInput(shown) => self.sample_input = shown,
            Message::SearchOpen => self.search_open = true,
            Message::SearchClose => self.search_open = false,
            Message::Query(query) => self.query = query,
            Message::SearchSubmit => {
                self.remember_search();
                self.search_open = false;
            }
            Message::Suggest(query) => {
                self.query = query;
                self.workspace_tab = WorkspaceTab::Files;
                self.search_open = true;
            }
            Message::FilterShared => self.filter_shared = !self.filter_shared,
            Message::FilterPinned => self.filter_pinned = !self.filter_pinned,
            Message::FileAge(values) => self.file_age = values,
            Message::ClearFilters => {
                self.query.clear();
                self.filter_shared = false;
                self.filter_pinned = false;
                self.file_age = (0.0, 30.0);
            }
            Message::FileNote(value) => self.file_notes[self.details_index] = value,
            Message::FileDetails(index) => {
                self.remember_search();
                self.search_open = false;
                self.details_index = index.min(1);
                self.details_modal = false;
                self.details_open = true;
            }
            Message::ModalDetails => {
                self.details_modal = true;
                self.details_open = true;
            }
            Message::CloseDetails => self.details_open = false,
            Message::NewWorkspace => {
                self.dialog_kind = 1;
                self.new_name.clear();
                self.creating = true;
            }
            Message::NewName(value) => self.new_name = value,
            Message::CreateWorkspace => {
                if !self.new_name.trim().is_empty() {
                    self.name = self.new_name.trim().to_owned();
                    self.creating = false;
                    self.details_open = false;
                    self.saved = false;
                    self.notify("Workspace created for this session", false);
                }
            }
            Message::CancelNew => self.creating = false,
            Message::PreviewValue(value) => self.preview_value = value,
            Message::LoadingPaused(value) => self.loading_paused = value,
            Message::TravelingWave(value) => self.traveling_wave = value,
            Message::FeedbackLoading(value) => self.feedback_loading = value,
            Message::FeedbackSelected(value) => self.feedback_selected = value,
            Message::FeedbackRange(value) => self.feedback_range = value,
            Message::ExportFormat(SegmentSelection::Single(Some(value)))
                if self.export_tick.is_none() =>
            {
                self.export_format = value
            }
            Message::ExportOptions(SegmentSelection::Multiple(values))
                if self.export_tick.is_none() =>
            {
                self.export_options = values
            }
            Message::ExportQuality(value) if self.export_tick.is_none() => {
                self.export_quality = value
            }
            Message::ExportFormat(_) | Message::ExportOptions(_) | Message::ExportQuality(_) => {}
            Message::StartExport if self.export_tick.is_none() => {
                self.export_id += 1;
                self.export_tick = Some(0);
                let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
                self.export_cancel = Some(cancelled.clone());
                return export_task(self.export_id, cancelled);
            }
            Message::StartExport => {}
            Message::CancelExport => {
                if let Some(cancelled) = self.export_cancel.take() {
                    cancelled.store(true, std::sync::atomic::Ordering::Relaxed);
                }
                if self.export_tick.take().is_some() {
                    self.notify("Export cancelled", false);
                }
            }
            Message::ExportTick(id, tick) => {
                if id == self.export_id
                    && let Some(current) = self.export_tick
                    && tick > current
                {
                    if tick >= 40 {
                        self.export_tick = None;
                        self.export_cancel = None;
                        self.notify("Export complete — demo finished", false);
                    } else {
                        self.export_tick = Some(tick);
                    }
                }
            }
            Message::Page(page) => {
                self.navigation_modal_open = false;
                self.page = page;
                self.search_open = false;
                self.details_open = false;
            }
            Message::WorkspaceTab(tab) => {
                self.workspace_tab = tab;
                self.search_open = false;
                self.details_open = false;
            }
            Message::ActivityTab(tab) => self.activity_tab = tab,
            Message::Read(index, read) => self.read[index] = read,
            Message::Pin => {
                self.pinned = !self.pinned;
                self.notify(
                    if self.pinned {
                        "Workspace pinned"
                    } else {
                        "Workspace unpinned"
                    },
                    false,
                );
            }
            Message::Delivery(value) => {
                self.delivery = value;
                self.saved = false;
            }
            Message::Access(value) => {
                self.access = value;
                self.saved = false;
            }
            Message::Duplicate => {
                self.undo_name = Some(self.name.clone());
                self.name = format!("{} copy", self.name);
                self.saved = false;
                self.notify("Workspace duplicated", true);
            }
            Message::UndoDuplicate => {
                if let Some(name) = self.undo_name.take() {
                    self.name = name;
                }
                self.notify("Duplication undone", false);
            }
            Message::ShowNotice => self.notify("All changes are up to date", false),
            Message::DismissNotice(id) => {
                if self
                    .notice
                    .as_ref()
                    .is_some_and(|(current, _, _)| *current == id)
                {
                    self.exiting_notice = self.notice.take();
                }
            }
            Message::Name(name) => {
                self.name = name;
                self.saved = false;
            }
            Message::Email(email) => {
                self.email = email;
                self.saved = false;
            }
            Message::Note(note) => self.note = note,
            Message::LongValue(value) => self.long_value = value,
            Message::Autosave(value) => {
                self.autosave = value;
                self.saved = false;
            }
            Message::Notifications(value) => {
                self.notifications = value;
                self.saved = false;
            }
            Message::RequiredChoice(value) => self.required_choice = value,
            Message::SelectAll(value) => {
                self.selected.fill(value);
                self.saved = false;
            }
            Message::Select(index) => {
                self.selected[index] = !self.selected[index];
                self.saved = false;
            }
            Message::Dark(value) => self.dark = value,
            Message::Accent(color, hex) => {
                self.accent = color;
                self.accent_hex = hex.into();
            }
            Message::AccentHex(value) => {
                if let Some(color) = parse_hex(&value) {
                    self.accent = color;
                }
                self.accent_hex = value;
            }
            Message::Outside(value) => self.outside = value,
            Message::Open => {
                self.dialog_kind = 0;
                self.open = true;
            }
            Message::Close => self.open = false,
            Message::Save => {
                self.open = false;
                self.saved = true;
                self.status = "Preferences saved for this session".into();
                self.notify("Preferences saved for this session", false);
            }
            Message::Action(label) => {
                self.fab_menu_open = false;
                self.clicks += 1;
                self.status = format!("{label} · {} actions", self.clicks);
            }
            Message::Compact => {
                self.compact = !self.compact;
                let size = if self.compact {
                    Size::new(390.0, 844.0)
                } else {
                    Size::new(1160.0, 920.0)
                };
                return iced::window::latest().and_then(move |id| iced::window::resize(id, size));
            }
        }
        iced::Task::none()
    }
    fn notify(&mut self, text: &str, undo: bool) {
        self.notice_id += 1;
        self.notice = Some((self.notice_id, text.into(), undo));
    }
    fn theme(&self) -> Theme {
        Theme::from_accent_with_contrast(
            self.accent,
            self.dark,
            if self.high_contrast { 1.0 } else { 0.0 },
        )
        .reduced_motion(self.reduced_motion)
    }
    fn valid(&self) -> bool {
        !self.name.trim().is_empty() && self.email.contains('@') && self.email.contains('.')
    }
    fn view(&self) -> Element<'_, Message> {
        let content = responsive(move |size| {
            let background = responsive(move |size| {
                let navigation_layout = NavigationLayout::for_width(size.width);
                let narrow = navigation_layout == NavigationLayout::Bar;
                let unread = self.read.iter().filter(|read| !**read).count() as u32;
                let rail: Element<'_, Message> = if narrow {
                    space().width(0).into()
                } else {
                    navigation_rail(
                        [
                            NavigationItem::new(Page::Workspace, "Workspace", Icon::Workspace),
                            NavigationItem::new(Page::Activity, "Activity", Icon::Activity)
                                .badge(unread),
                            NavigationItem::new(Page::Settings, "Settings", Icon::Settings),
                        ],
                        Some(self.page),
                    )
                    .on_select(Message::Page)
                    .expanded(
                        self.rail_expanded
                            .unwrap_or(navigation_layout == NavigationLayout::ExpandedRail),
                    )
                    .header(
                        container(typography("m", TypeScale::TitleLarge))
                            .center_x(40)
                            .center_y(40)
                            .style(|theme: &Theme| widget::container::Style {
                                background: Some(theme.colors.primary.into()),
                                text_color: Some(theme.colors.on_primary),
                                border: iced::Border {
                                    radius: 12.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }),
                    )
                    .into()
                };
                let navigation: Element<'_, Message> = if narrow {
                    navigation_bar(
                        [
                            NavigationItem::new(Page::Workspace, "Workspace", Icon::Workspace),
                            NavigationItem::new(Page::Activity, "Activity", Icon::Activity)
                                .badge(unread),
                            NavigationItem::new(Page::Settings, "Settings", Icon::Settings),
                        ],
                        Some(self.page),
                    )
                    .on_select(Message::Page)
                    .into()
                } else {
                    space().height(0).into()
                };
                let gutter = if narrow { 16.0 } else { 32.0 };
                let floating_action =
                    self.page == Page::Workspace && self.workspace_tab == WorkspaceTab::Files;
                let main_padding = iced::Padding {
                    bottom: gutter + if floating_action { 88.0 } else { 0.0 },
                    ..iced::Padding::from(gutter)
                };
                let width = (size.width
                    - if narrow {
                        0.0
                    } else if self
                        .rail_expanded
                        .unwrap_or(navigation_layout == NavigationLayout::ExpandedRail)
                    {
                        280.0
                    } else {
                        80.0
                    }
                    - gutter * 2.0)
                    .clamp(0.0, 1040.0);
                let body = match self.page {
                    Page::Workspace => self.workspace_page(width),
                    Page::Activity => self.activity_page(),
                    Page::Settings => column![
                        typography("Make it yours", TypeScale::HeadlineMedium),
                        muted(
                            "Appearance, preferences, and delivery options.",
                            TypeScale::BodyLarge
                        ),
                        self.appearance(),
                        self.settings(),
                        self.switch_states(),
                    ]
                    .spacing(24)
                    .into(),
                };
                let main = scrollable(
                    container(
                        widget::column![
                            body,
                            divider(),
                            muted(
                                "Upstream iced 0.14 · Material 3 inspired · Keyboard navigation",
                                TypeScale::Supporting
                            )
                        ]
                        .spacing(24)
                        .width(width),
                    )
                    .padding(main_padding)
                    .center_x(Length::Fill),
                )
                .height(Length::Fill);
                let main: Element<'_, Message> = if floating_action {
                    let action: Element<'_, Message> = if narrow {
                        tooltip(
                            fab(Icon::Add).on_press(Message::NewWorkspace),
                            "New workspace",
                        )
                        .into()
                    } else {
                        extended_fab(Icon::Add, "New workspace")
                            .on_press(Message::NewWorkspace)
                            .into()
                    };
                    widget::stack![
                        main,
                        container(action)
                            .align_right(Length::Fill)
                            .align_bottom(Length::Fill)
                            .padding(24),
                    ]
                    .into()
                } else {
                    main.into()
                };
                row![
                    rail,
                    column![self.app_header(navigation_layout), main, navigation]
                        .height(Length::Fill)
                ]
                .height(Length::Fill)
                .into()
            });
            let panel = if size.width < 900.0 {
                bottom_sheet(self.file_details())
                    .height(self.drag_height)
                    .on_height(Message::SheetHeight)
                    .snap_points([240.0, 520.0, 720.0])
            } else if self.details_modal {
                side_sheet(self.file_details()).modal()
            } else {
                side_sheet(self.file_details())
            }
            .open(self.details_open)
            .on_dismiss(Message::CloseDetails);
            sheet::host(
                iced_m3::snackbar::host(
                    background,
                    self.notice.as_ref().or(self.exiting_notice.as_ref()).map(
                        |(id, text, undo)| {
                            let notice = snackbar(text.clone())
                                .id(*id)
                                .visible(self.notice.is_some())
                                .on_dismiss(Message::DismissNotice(*id));
                            if *undo {
                                notice.action("Undo", Message::UndoDuplicate)
                            } else {
                                notice
                            }
                        },
                    ),
                ),
                panel,
            )
        });
        let content = iced_m3::modal_navigation_rail(
            content,
            navigation_rail(
                [
                    NavigationItem::new(Page::Workspace, "Workspace", Icon::Workspace),
                    NavigationItem::new(Page::Activity, "Activity", Icon::Activity)
                        .badge(self.read.iter().filter(|read| !**read).count() as u32),
                    NavigationItem::new(Page::Settings, "Settings", Icon::Settings),
                ],
                Some(self.page),
            )
            .on_select(Message::Page)
            .header(
                row![
                    icon_button(Icon::Close).on_press(Message::NavigationModal(false)),
                    typography("Navigate", TypeScale::TitleMedium)
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            ),
            self.navigation_modal_open,
            Message::NavigationModal(false),
        );
        iced_m3::focus::scope(iced_m3::dialog::stack(
            content,
            [
                (
                    if self.editor_open || (!self.open && !self.creating && self.dialog_kind == 2) {
                        self.workspace_editor()
                    } else if self.creating || (!self.open && self.dialog_kind == 1) {
                        dialog(column![
                    typography("New workspace", TypeScale::HeadlineSmall),
                    muted("Give your team's next workspace a name. This demo keeps changes for this session.", TypeScale::BodyMedium),
                    text_field("Workspace name", &self.new_name).on_input(Message::NewName),
                    iced_m3::dialog::actions(row![
                        button("Cancel").variant(ButtonVariant::Text).on_press(Message::CancelNew),
                        button("Create workspace").on_press(Message::CreateWorkspace).disabled(self.new_name.trim().is_empty()),
                    ].spacing(8).wrap()),
                ].spacing(24)).on_dismiss(Message::CancelNew)
                    } else {
                        {
                            dialog(
                        column![
                            typography("Make it yours", TypeScale::HeadlineSmall),
                            muted(
                                "Save these workspace preferences for this gallery session?",
                                TypeScale::BodyMedium
                            ),
                            surface(
                                column![
                                    typography(self.name.clone(), TypeScale::Title),
                                    muted(self.email.clone(), TypeScale::Body),
                                    muted(
                                        if self.autosave {
                                            "Autosave is on"
                                        } else {
                                            "Autosave is off"
                                        },
                                        TypeScale::Label
                                    )
                                ]
                                .spacing(8)
                            )
                            .variant(SurfaceVariant::Outlined)
                            .padding(16),
                            self.access_select(),
                            text_field("Optional note", &self.note)
                                .id("gallery-note")
                                .on_input(Message::Note)
                                .supporting_text("Native text entry works inside the dialog too."),
                            muted(
                                if self.outside {
                                    "Escape or click the scrim to cancel."
                                } else {
                                    "Scrim dismissal is off. Escape still cancels."
                                },
                                TypeScale::Supporting
                            ),
                        ]
                        .spacing(20),
                    )
                    .actions(
                        row![
                            button("Cancel")
                                .variant(ButtonVariant::Text)
                                .on_press(Message::Close),
                            button("Save preferences")
                                .variant(ButtonVariant::Text)
                                .on_press(Message::Save)
                        ]
                        .spacing(8)
                        .wrap()
                    )
                    .max_height(600)
                    .on_dismiss(Message::Close)
                    .dismiss_on_outside(self.outside)
                        }
                    },
                    self.open || self.creating || self.editor_open,
                ),
                (
                    dialog(
                        column![
                            typography("Discard changes?", TypeScale::HeadlineSmall),
                            muted(
                                "Your workspace name and note have unsaved changes.",
                                TypeScale::BodyMedium
                            ),
                            iced_m3::dialog::actions(
                                row![
                                    button("Keep editing")
                                        .variant(ButtonVariant::Text)
                                        .on_press(Message::EditorKeep),
                                    button("Discard")
                                        .variant(ButtonVariant::Text)
                                        .on_press(Message::EditorDiscard(true)),
                                ]
                                .spacing(8)
                                .wrap()
                            )
                        ]
                        .spacing(20),
                    )
                    .width(400.0)
                    .on_dismiss(Message::EditorKeep),
                    self.editor_discard_open,
                ),
            ],
        ))
    }
    fn app_header(&self, layout: NavigationLayout) -> AppBar<'_, Message> {
        app_bar(self.page.title()).action(menu(
            "More",
            [
                MenuItem::new("Review preferences", Message::Open),
                MenuItem::new("Edit workspace details", Message::EditorOpen),
                MenuItem::submenu(
                    "Navigation",
                    [
                        MenuItem::new("Open navigation menu", Message::NavigationModal(true)),
                        MenuItem::new(
                            if self
                                .rail_expanded
                                .unwrap_or(layout == NavigationLayout::ExpandedRail)
                            {
                                "Collapse rail"
                            } else {
                                "Expand rail"
                            },
                            Message::RailExpanded(
                                !self
                                    .rail_expanded
                                    .unwrap_or(layout == NavigationLayout::ExpandedRail),
                            ),
                        )
                        .disabled(layout == NavigationLayout::Bar),
                    ],
                ),
                MenuItem::new("New workspace", Message::NewWorkspace),
                MenuItem::new("File details in modal sheet", Message::ModalDetails),
                MenuItem::new(
                    if self.compact {
                        "Wide view"
                    } else {
                        "Narrow view"
                    },
                    Message::Compact,
                ),
                MenuItem::separator(),
                MenuItem::new("Show confirmation", Message::ShowNotice),
            ],
        ))
    }
    fn workspace_editor(&self) -> iced_m3::Dialog<'_, Message> {
        iced_m3::full_screen_dialog(
            "Edit workspace",
            column![
                muted(
                    "Changes stay in this draft until you save them.",
                    TypeScale::BodyLarge
                ),
                text_field("Workspace name", &self.editor_name)
                    .on_input(Message::EditorName)
                    .supporting_text("Choose a name your team will recognize."),
                text_field("Optional note", &self.editor_note).on_input(Message::EditorNote),
                divider(),
                typography("Workspace access", TypeScale::TitleLarge),
                muted(
                    "Your current sharing and delivery preferences remain available in Settings.",
                    TypeScale::BodyLarge
                ),
            ]
            .spacing(24),
        )
        .on_dismiss(Message::EditorCancel)
        .action(
            button("Save")
                .variant(ButtonVariant::Text)
                .on_press(Message::EditorSave)
                .disabled(self.editor_name.trim().is_empty()),
        )
        .initial_focus(1 + usize::from(!self.editor_name.trim().is_empty()))
        .into()
    }
    fn workspace_page(&self, width: f32) -> Element<'_, Message> {
        let wide = width >= 760.0;
        let left = if wide { (width - 24.0) * 0.55 } else { width };
        let right = if wide { width - left - 24.0 } else { width };
        let content = match self.workspace_tab {
            WorkspaceTab::Overview => column![self.workflow(), self.files(width)]
                .spacing(24)
                .into(),
            WorkspaceTab::Files => self.files(width),
            WorkspaceTab::Export => self.export_page(),
            WorkspaceTab::Schedule => self.schedule_page(),
            WorkspaceTab::Components => column![
                row![
                    container(self.actions()).width(left),
                    container(self.type_and_surfaces()).width(right)
                ]
                .spacing(24)
                .wrap(),
                self.checkbox_states(),
                self.value_controls(),
                self.loading_progress(),
                self.primary_actions(),
                self.chip_variants(),
                self.modern_components(),
                self.switch_states(),
                self.edge_cases(),
                self.desktop_variants(),
                self.desktop_fidelity(),
                self.baseline_completion(),
            ]
            .spacing(24)
            .into(),
        };
        column![
            typography(self.name.clone(), TypeScale::HeadlineMedium),
            muted(
                "A place for your team's everyday work.",
                TypeScale::BodyLarge
            ),
            tabs(
                [
                    Tab::new(WorkspaceTab::Overview, "Overview"),
                    Tab::new(WorkspaceTab::Files, "Files").badge(3),
                    Tab::new(WorkspaceTab::Components, "Components"),
                    Tab::new(WorkspaceTab::Export, "Export"),
                    Tab::new(WorkspaceTab::Schedule, "Schedule")
                ],
                Some(self.workspace_tab)
            )
            .variant(TabVariant::Secondary)
            .scrollable(true)
            .on_select(Message::WorkspaceTab),
            content,
        ]
        .spacing(20)
        .into()
    }

    fn sync_clock_draft(&mut self) {
        self.clock_hour = format!(
            "{:02}",
            if self.clock24 {
                self.clock_time.hour()
            } else {
                self.clock_time.hour12()
            }
        );
        self.clock_minute = format!("{:02}", self.clock_time.minute());
        self.clock_pm = self.clock_time.is_pm();
    }
    fn baseline_completion(&self) -> Element<'_, Message> {
        surface(column![
            typography("Feedback in motion", TypeScale::TitleLarge),
            muted("Move the handles close together to compare both values. Switch from loading to measured progress, or change direction while it moves.", TypeScale::BodyLarge),
            range_slider(0.0..=100.0, self.feedback_range)
                .value_labels(format!("{:.0}%",self.feedback_range.0),format!("{:.0}%",self.feedback_range.1))
                .on_change(Message::FeedbackRange),
            row![
                checkbox(self.feedback_selected).label("Selected").on_toggle(Message::FeedbackSelected),
                filter_chip("Selected filter",self.feedback_selected).on_press(Message::FeedbackSelected(!self.feedback_selected)),
                input_chip("Avery").avatar(container(typography("A",TypeScale::LabelLarge)).center_x(24).center_y(24).style(|theme: &Theme| widget::container::Style {
                    background:Some(theme.colors.tertiary_container.into()),text_color:Some(theme.colors.on_tertiary_container),
                    border:iced::Border{radius:12.0.into(),..Default::default()},..Default::default()
                })).selected(self.feedback_selected).on_press(Message::FeedbackSelected(!self.feedback_selected)),
            ].spacing(16).align_y(Alignment::Center).wrap(),
            row![
                button(if self.feedback_loading { "Show progress" } else { "Start loading" }).on_press(Message::FeedbackLoading(!self.feedback_loading)),
                button("Open dialog").variant(ButtonVariant::Text).on_press(Message::Open),
                menu("Commands", [MenuItem::new("Select",Message::FeedbackSelected(true)),MenuItem::new("Clear",Message::FeedbackSelected(false))])
            ].spacing(12).wrap(),
            slider(0.0..=1.0,self.preview_value).on_change(Message::PreviewValue),
            linear_progress(self.preview_value).indeterminate(self.feedback_loading),
            circular_progress(self.preview_value).indeterminate(self.feedback_loading),
        ].spacing(24)).variant(SurfaceVariant::Outlined).padding(24).into()
    }

    fn desktop_fidelity(&self) -> Element<'_, Message> {
        let c = self.theme().colors;
        let mut elevations = row![].spacing(24);
        for level in 0..=5 {
            elevations = elevations.push(iced_m3::elevated(
                container(typography(format!("Level {level}"), TypeScale::LabelLarge))
                    .center_x(136)
                    .center_y(80)
                    .style(|theme: &Theme| widget::container::Style {
                        background: Some(theme.colors.surface_container_low.into()),
                        border: iced::Border {
                            radius: 12.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                level as f32,
                12.0,
            ));
        }
        let mut accents = row![].spacing(12);
        for (label, color, ink) in [
            ("Primary fixed", c.primary_fixed, c.on_primary_fixed),
            ("Secondary fixed", c.secondary_fixed, c.on_secondary_fixed),
            ("Tertiary fixed", c.tertiary_fixed, c.on_tertiary_fixed),
            ("Secondary", c.secondary, c.on_secondary),
            ("Tertiary", c.tertiary, c.on_tertiary),
        ] {
            accents = accents.push(
                container(typography(label, TypeScale::LabelLarge))
                    .padding(16)
                    .width(180)
                    .style(move |_| widget::container::Style {
                        background: Some(color.into()),
                        text_color: Some(ink),
                        border: iced::Border {
                            radius: 12.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
            );
        }
        surface(
            column![
                typography("Finishing details", TypeScale::TitleLarge),
                muted(
                    "Light, depth and feedback across the desktop controls.",
                    TypeScale::BodyLarge
                ),
                accents.wrap(),
                container(elevations.wrap()).padding([24, 0]),
                typography("Hover or focus to see the value", TypeScale::LabelLarge),
                slider(0.0..=1.0, self.preview_value)
                    .labeled(true)
                    .value_label(format!("{:.0}%", self.preview_value * 100.0))
                    .on_change(Message::PreviewValue),
                linear_progress(self.preview_value),
                row![
                    circular_progress(self.preview_value),
                    circular_progress(0.0).indeterminate(true),
                    container(linear_progress(0.0).indeterminate(true)).center_y(48)
                ]
                .spacing(24)
                .align_y(Alignment::Center)
            ]
            .spacing(24),
        )
        .variant(SurfaceVariant::Outlined)
        .padding(24)
        .into()
    }

    fn desktop_variants(&self) -> Element<'_, Message> {
        let colors = [
            iced::Color::from_rgb8(66, 133, 244),
            iced::Color::from_rgb8(234, 67, 53),
            iced::Color::from_rgb8(251, 188, 5),
            iced::Color::from_rgb8(52, 168, 83),
        ];
        let cards = [
            SurfaceVariant::Filled,
            SurfaceVariant::Outlined,
            SurfaceVariant::Elevated,
        ]
        .into_iter()
        .map(|variant| {
            iced_m3::card(
                column![
                    typography(format!("{variant:?} card"), TypeScale::TitleMedium),
                    typography("Workspace collection", TypeScale::BodyMedium)
                ]
                .spacing(8),
            )
            .variant(variant)
            .width(240)
            .on_press(Message::Action("Collection opened"))
            .disabled(self.card_disabled)
            .dragged(self.card_dragged)
            .into()
        });
        surface(
            column![
                typography("More ways to work", TypeScale::TitleLarge),
                row![
                    text_field("Budget", &self.amount)
                        .on_input(Message::Amount)
                        .prefix("$")
                        .suffix("USD")
                        .width(280)
                        .supporting_text("Set a budget for this workspace"),
                    text_field("Password", &self.password)
                        .on_input(Message::Password)
                        .variant(iced_m3::TextFieldVariant::Filled)
                        .secure(!self.show_password)
                        .leading(Icon::Workspace)
                        .trailing_action(Icon::Visibility, Message::ShowPassword)
                        .supporting_text("Use the icon to show or hide the password")
                        .width(320),
                ]
                .spacing(24)
                .wrap(),
                self.access_select()
                    .variant(iced_m3::TextFieldVariant::Filled)
                    .leading(Icon::Workspace)
                    .supporting_text("Choose a role for new members"),
                row![
                    checkbox(self.card_disabled)
                        .label("Disable cards")
                        .on_toggle(Message::CardDisabled),
                    checkbox(self.card_dragged)
                        .label("Preview dragged state")
                        .on_toggle(Message::CardDragged)
                ]
                .spacing(24)
                .wrap(),
                widget::Row::with_children(cards).spacing(24).wrap(),
                row![
                    fab(Icon::Add).on_press(Message::Action("Standard action")),
                    typography("Standard", TypeScale::BodyMedium),
                    fab(Icon::Add)
                        .lowered(true)
                        .on_press(Message::Action("Lowered action")),
                    typography("Lowered", TypeScale::BodyMedium),
                ]
                .spacing(16)
                .align_y(Alignment::Center)
                .wrap(),
                typography("Buffered transfer", TypeScale::LabelLarge),
                slider(0.0..=1.0, self.preview_value).on_change(Message::PreviewValue),
                linear_progress(self.preview_value).buffer((self.preview_value + 0.25).min(1.0)),
                row![
                    circular_progress(0.0).indeterminate(true).colors(colors),
                    typography("Multicolor loading", TypeScale::BodyMedium)
                ]
                .spacing(16)
                .align_y(Alignment::Center),
                linear_progress(0.0).indeterminate(true).colors(colors),
            ]
            .spacing(24),
        )
        .variant(SurfaceVariant::Outlined)
        .into()
    }
    fn schedule_valid(&self) -> bool {
        let min = iced_m3::Date::new(2026, 1, 1).unwrap();
        let max = iced_m3::Date::new(2030, 12, 31).unwrap();
        self.calendar_text
            .parse::<iced_m3::Date>()
            .is_ok_and(|start| {
                start >= min
                    && start <= max
                    && (!self.calendar_range
                        || self
                            .calendar_end
                            .parse::<iced_m3::Date>()
                            .is_ok_and(|end| end >= start && end <= max))
            })
            && self.clock_text.parse::<iced_m3::Time>().is_ok()
    }
    fn schedule_page(&self) -> Element<'_, Message> {
        let min = iced_m3::Date::new(2026, 1, 1).unwrap();
        let max = iced_m3::Date::new(2030, 12, 31).unwrap();
        let selection: Element<'_, Message> = if self.schedule_input {
            let mut inputs = column![
                iced_m3::date_input(
                    if self.calendar_range {
                        "Start date"
                    } else {
                        "Review date"
                    },
                    &self.calendar_text
                )
                .on_input(Message::CalendarText)
            ]
            .spacing(16);
            if self.calendar_range {
                inputs = inputs.push(
                    iced_m3::date_input("End date", &self.calendar_end)
                        .on_input(Message::CalendarEnd),
                );
            }
            inputs
                .push(
                    iced_m3::time_input("Review time", &self.clock_text)
                        .on_input(Message::ClockText),
                )
                .into()
        } else {
            row![
                iced_m3::date_picker(self.calendar_month, self.calendar_selection)
                    .today(iced_m3::Date::new(2026, 9, 13).unwrap())
                    .bounds(min, max)
                    .years(self.calendar_years)
                    .on_toggle_years(Message::CalendarYears)
                    .on_select(Message::Calendar)
                    .on_month(Message::CalendarMonth)
                    .input_mode(self.calendar_input)
                    .on_toggle_input(Message::CalendarInput)
                    .input(&self.calendar_text, Message::CalendarText)
                    .end_input(&self.calendar_end, Message::CalendarEnd),
                iced_m3::time_picker(self.clock_time, self.clock_part)
                    .format24(self.clock24)
                    .on_change(Message::Clock)
                    .on_part(Message::ClockPart)
                    .input_mode(self.clock_input)
                    .on_toggle_input(Message::ClockInput)
                    .hour_input(&self.clock_hour, Message::ClockHour)
                    .minute_input(&self.clock_minute, Message::ClockMinute)
                    .input_period(self.clock_pm, Message::ClockPeriod)
            ]
            .spacing(24)
            .wrap()
            .into()
        };
        column![
            typography("Schedule a review", TypeScale::HeadlineSmall),
            muted(
                "Choose a date or range and a local time. This example accepts dates in 2026–2030.",
                TypeScale::BodyLarge
            ),
            row![
                filter_chip("Date range", self.calendar_range)
                    .on_press(Message::CalendarRange(!self.calendar_range)),
                filter_chip("24-hour clock", self.clock24)
                    .on_press(Message::Clock24(!self.clock24)),
                filter_chip("Type date and time", self.schedule_input)
                    .on_press(Message::ScheduleInput(!self.schedule_input)),
                iced_m3::docked_date_picker(
                    typography("Open calendar", TypeScale::LabelLarge),
                    iced_m3::date_picker(self.calendar_month, self.calendar_selection)
                        .bounds(min, max)
                        .years(self.calendar_years)
                        .on_toggle_years(Message::CalendarYears)
                        .on_month(Message::CalendarMonth)
                        .on_select(Message::Calendar)
                        .input_mode(self.calendar_input)
                        .on_toggle_input(Message::CalendarInput)
                        .input(&self.calendar_text, Message::CalendarText)
                        .end_input(&self.calendar_end, Message::CalendarEnd)
                )
            ]
            .spacing(8)
            .wrap(),
            selection,
            button("Schedule review")
                .on_press(Message::SaveSchedule)
                .disabled(!self.schedule_valid()),
            typography(self.schedule_saved.clone(), TypeScale::BodyLarge)
        ]
        .spacing(20)
        .into()
    }

    fn modern_components(&self) -> Element<'_, Message> {
        use iced_m3::{
            CarouselVariant, FabMenuItem, TextFieldVariant, button_group, fab_menu, lazy_carousel,
            loading_indicator, rich_tooltip, split_button, toolbar,
        };
        let items = |i: usize| {
            let svg = format!(
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 280 200'><rect width='280' height='200' fill='{}'/><circle cx='{}' cy='65' r='70' fill='{}'/><path d='M0 160 L95 65 L220 200 H0Z' fill='{}'/><circle cx='225' cy='150' r='60' fill='{}'/></svg>",
                ["#d8c9f3", "#bee5df", "#f8d5b5", "#c8daef", "#edcfe0"][i % 5],
                70 + (i % 5) * 25,
                ["#aa89d4", "#68bdb0", "#e99d65", "#82add8", "#c88eaf"][i % 5],
                ["#6e519c", "#36796e", "#b56e3a", "#466f9c", "#925975"][i % 5],
                ["#e7def6", "#cde9d6", "#fae2b9", "#e2ecf9", "#f4e1eb"][i % 5]
            );
            let art: Element<'_, Message> =
                widget::svg(widget::svg::Handle::from_memory(svg.into_bytes()))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .content_fit(iced::ContentFit::Cover)
                    .into();
            let caption: Element<'_, Message> = if i == self.carousel_index {
                container(typography(
                    format!("Collection {}", i + 1),
                    TypeScale::TitleLarge,
                ))
                .padding(12)
                .width(Length::Fill)
                .style(|theme: &Theme| widget::container::Style {
                    background: Some(theme.colors.surface_container.into()),
                    text_color: Some(theme.colors.on_surface),
                    ..Default::default()
                })
                .into()
            } else {
                space().height(0).into()
            };
            Button::new(widget::stack![
                art,
                container(caption).align_bottom(Length::Fill)
            ])
            .padding(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .on_press(Message::Action("Open collection"))
            .into()
        };
        surface(
            column![
                typography("More ways to work", TypeScale::TitleLarge),
                row![
                    checkbox(self.reduced_motion)
                        .label("Reduce motion")
                        .on_toggle(Message::ReducedMotion),
                    checkbox(self.high_contrast)
                        .label("Increase contrast")
                        .on_toggle(Message::HighContrast)
                ]
                .spacing(16)
                .wrap(),
                lazy_carousel(10_000, self.carousel_index, items)
                    .variant(CarouselVariant::MultiBrowse)
                    .on_select(Message::Carousel),
                row![
                    button("Previous collection")
                        .on_press(Message::Carousel(self.carousel_index.saturating_sub(1)))
                        .disabled(self.carousel_index == 0),
                    button("Next collection")
                        .on_press(Message::Carousel((self.carousel_index + 1).min(9_999)))
                        .disabled(self.carousel_index == 9_999)
                ]
                .spacing(8)
                .wrap(),
                text_field("Filled workspace field", &self.name)
                    .variant(TextFieldVariant::Filled)
                    .on_input(Message::Name),
                row![
                    button("Elevated action")
                        .variant(ButtonVariant::Elevated)
                        .on_press(Message::Action("Elevated action")),
                    assist_chip("Elevated chip")
                        .elevated(true)
                        .on_press(Message::Action("Elevated chip")),
                    input_chip("Studio")
                        .avatar(Icon::Workspace)
                        .dropdown(true)
                        .on_press(Message::Action("Studio chip")),
                    switch(self.notifications)
                        .icons(true)
                        .label("Notifications")
                        .on_toggle(Message::Notifications)
                ]
                .spacing(12)
                .wrap(),
                button_group([
                    button("Copy").on_press(Message::Action("Copy")),
                    button("Move").on_press(Message::Action("Move")),
                    button("Share").on_press(Message::Action("Share"))
                ])
                .connected(true),
                split_button(
                    "Save",
                    Message::Action("Save"),
                    [
                        MenuItem::new("Save a copy", Message::Action("Save a copy"))
                            .supporting_text("Keep the original workspace"),
                        MenuItem::new("Save as template", Message::Action("Save as template"))
                    ]
                ),
                toolbar([
                    icon_button(Icon::Add)
                        .on_press(Message::Action("Add"))
                        .into(),
                    icon_button(Icon::Star)
                        .on_press(Message::Action("Star"))
                        .into(),
                    icon_button(Icon::Settings)
                        .on_press(Message::Action("Settings"))
                        .into()
                ])
                .floating(true),
                rich_tooltip(
                    typography("About collections", TypeScale::LabelLarge),
                    "Keep related work together",
                    column![
                        typography(
                            "Collections organize your files and shared resources.",
                            TypeScale::BodyMedium
                        ),
                        button("Got it")
                            .variant(ButtonVariant::Text)
                            .on_press(Message::Action("Collection help"))
                    ]
                    .spacing(12)
                ),
                checkbox(self.extended_action)
                    .label("Show action label")
                    .on_toggle(Message::ExtendedAction),
                extended_fab(Icon::Add, "Create collection")
                    .extended(self.extended_action)
                    .on_press(Message::Action("Create collection")),
                fab_menu(
                    Icon::Add,
                    self.fab_menu_open,
                    Message::FabMenu,
                    [
                        FabMenuItem::new(
                            "New document",
                            Icon::Workspace,
                            Message::Action("New document")
                        ),
                        FabMenuItem::new("New folder", Icon::Add, Message::Action("New folder"))
                    ]
                ),
                row![
                    loading_indicator().contained(true),
                    typography("Expressive loading", TypeScale::BodyMedium)
                ]
                .spacing(12)
                .align_y(Alignment::Center)
            ]
            .spacing(24),
        )
        .variant(SurfaceVariant::Outlined)
        .into()
    }
    fn loading_progress(&self) -> Element<'_, Message> {
        let speed = if self.traveling_wave { 20.0 } else { 0.0 };
        surface(column![
            typography("Loading with expression",TypeScale::TitleLarge),
            muted("Seven rounded shapes flow into one another. Try waves while loading, then switch to measured progress.",TypeScale::BodyLarge),
            row![iced_m3::loading_indicator().paused(self.loading_paused),
                iced_m3::loading_indicator().contained(true).paused(self.loading_paused),
                iced_m3::loading_indicator().contained(true).size(72.).paused(self.loading_paused),
            ].spacing(24).align_y(Alignment::Center),
            row![
                button(if self.feedback_loading { "Show measured progress" } else { "Show loading" })
                    .on_press(Message::FeedbackLoading(!self.feedback_loading)),
                checkbox(self.loading_paused).label("Pause motion").on_toggle(Message::LoadingPaused),
                checkbox(self.traveling_wave).label("Traveling waves").on_toggle(Message::TravelingWave),
            ].spacing(16).align_y(Alignment::Center).wrap(),
            slider(0.0..=1.0,self.preview_value).on_change(Message::PreviewValue),
            typography(format!("{:.0}% complete", self.preview_value*100.),TypeScale::LabelLarge),
            linear_progress(self.preview_value).wavy(true).indeterminate(self.feedback_loading)
                .paused(self.loading_paused).wave_speed(speed),
            row![circular_progress(self.preview_value).wavy(true).indeterminate(self.feedback_loading)
                    .paused(self.loading_paused).wave_speed(speed),
                circular_progress(self.preview_value).wavy(true).size(80.).indeterminate(self.feedback_loading)
                    .paused(self.loading_paused).wave_speed(speed),
                muted("Waves settle into a flat track near completion.",TypeScale::BodyMedium),
            ].spacing(24).align_y(Alignment::Center).wrap(),
        ].spacing(24)).variant(SurfaceVariant::Outlined).padding(24).into()
    }
    fn value_controls(&self) -> Element<'_, Message> {
        surface(
            column![
                typography("Values and progress", TypeScale::TitleLarge),
                muted(
                    "Drag continuously or in steps. Progress follows the value below.",
                    TypeScale::BodyMedium
                ),
                typography(
                    format!("Continuous · {:.0}%", self.preview_value * 100.0),
                    TypeScale::LabelLarge
                ),
                slider(0.0..=1.0, self.preview_value).on_change(Message::PreviewValue),
                typography("Disabled slider", TypeScale::LabelLarge),
                slider(0.0..=1.0, 0.6).step(0.1).ticks(true),
                linear_progress(self.preview_value),
                row![
                    circular_progress(self.preview_value),
                    typography("Determinate", TypeScale::BodyMedium),
                    circular_progress(0.0).indeterminate(true),
                    typography("Indeterminate", TypeScale::BodyMedium)
                ]
                .spacing(12)
                .align_y(Alignment::Center)
                .wrap(),
                linear_progress(0.0).indeterminate(true),
                muted(
                    "For stepped sliders and segmented selection, open Workspace → Export.",
                    TypeScale::BodySmall
                ),
            ]
            .spacing(16),
        )
        .variant(SurfaceVariant::Outlined)
        .into()
    }
    fn export_page(&self) -> Element<'_, Message> {
        let busy = self.export_tick.is_some();
        let preparing = self.export_tick.is_some_and(|tick| tick < 8);
        let progress = self
            .export_tick
            .map_or(0.0, |tick| (tick.saturating_sub(8) as f32 / 32.0).min(1.0));
        let status = if preparing {
            "Preparing your export…".into()
        } else if busy {
            format!("Exporting · {:.0}%", progress * 100.0)
        } else {
            "Ready to export".into()
        };
        surface(
            column![
                typography("Export workspace", TypeScale::TitleLarge),
                muted(
                    "Try a simulated export. Your files stay where they are.",
                    TypeScale::BodyMedium
                ),
                typography("Output format", TypeScale::LabelLarge),
                segmented_buttons(
                    [
                        Segment::new(0, "PNG"),
                        Segment::new(1, "JPEG"),
                        Segment::new(2, "WebP")
                    ],
                    SegmentSelection::Single(Some(self.export_format))
                )
                .width(Length::Fill)
                .on_change(Message::ExportFormat)
                .disabled(busy),
                row![
                    typography("Quality", TypeScale::LabelLarge),
                    space::horizontal(),
                    typography(
                        format!("{:.0}%", self.export_quality),
                        TypeScale::BodyMedium
                    )
                ],
                slider(0.0..=100.0, self.export_quality)
                    .step(5.0)
                    .ticks(true)
                    .value_label(format!("{:.0}%", self.export_quality))
                    .on_change(Message::ExportQuality)
                    .disabled(busy),
                typography("Include", TypeScale::LabelLarge),
                segmented_buttons(
                    [Segment::new(0, "Notes"), Segment::new(1, "Tags")],
                    SegmentSelection::Multiple(self.export_options.clone())
                )
                .width(Length::Fill)
                .on_change(Message::ExportOptions)
                .disabled(busy),
                divider(),
                row![
                    circular_progress(progress).indeterminate(preparing),
                    typography(status, TypeScale::BodyLarge)
                ]
                .spacing(12)
                .align_y(Alignment::Center),
                linear_progress(progress).indeterminate(preparing),
                row![
                    button(if busy { "Exporting…" } else { "Start export" })
                        .on_press(Message::StartExport)
                        .disabled(busy),
                    button("Cancel")
                        .variant(ButtonVariant::Text)
                        .on_press(Message::CancelExport)
                        .disabled(!busy)
                ]
                .spacing(8)
                .wrap(),
            ]
            .spacing(16),
        )
        .variant(SurfaceVariant::Outlined)
        .into()
    }
    fn file_details(&self) -> Element<'_, Message> {
        let title = if self.details_index == 0 {
            "Design notes"
        } else {
            "Component checklist"
        };
        column![
            row![
                typography("File details", TypeScale::TitleLarge),
                space::horizontal(),
                tooltip(icon_button(Icon::Close).on_press(Message::CloseDetails), "Close details"),
            ].align_y(Alignment::Center).spacing(12),
            typography(title, TypeScale::HeadlineSmall),
            muted("Shared with your team", TypeScale::LabelLarge),
            divider(),
            typography("About this file", TypeScale::TitleMedium),
            muted(if self.details_index == 0 {
                "Working notes for the workspace: decisions, visual references, and ideas for the next iteration."
            } else {
                "A shared checklist for reviewing buttons, choices, navigation, and feedback across the app."
            }, TypeScale::BodyMedium),
            list([
                list_item("Location").supporting_text(self.name.clone()).into(),
                list_item("Last updated").supporting_text("Just now · Local demo").into(),
                list_item("Access").supporting_text("Your team can edit").into(),
            ]),
            text_field("File note", &self.file_notes[self.details_index])
                .on_input(Message::FileNote).supporting_text("Kept when you close and reopen this panel."),
            divider(),
            row![
                button("Close details").variant(ButtonVariant::Text).on_press(Message::CloseDetails),
                button("Export workspace").on_press(Message::WorkspaceTab(WorkspaceTab::Export)),
            ].spacing(8).wrap(),
        ].spacing(20).into()
    }
    fn primary_actions(&self) -> Element<'_, Message> {
        surface(column![
            typography("Floating action buttons", TypeScale::TitleLarge),
            muted("One prominent action for a screen. Compare sizes and colors here; try the floating New workspace action on the Files tab.", TypeScale::BodyMedium),
            row![
                tooltip(fab(Icon::Add).size(FabSize::Small).on_press(Message::NewWorkspace), "Small FAB"),
                tooltip(fab(Icon::Add).on_press(Message::NewWorkspace), "Regular FAB"),
                tooltip(fab(Icon::Add.sized(36.0)).size(FabSize::Large).on_press(Message::NewWorkspace), "Large FAB"),
                extended_fab(Icon::Add, "New workspace").on_press(Message::NewWorkspace),
            ].align_y(Alignment::Center).spacing(24).wrap(),
            row![
                tooltip(fab(Icon::Add).color(FabColor::Primary).on_press(Message::NewWorkspace), "Primary"),
                tooltip(fab(Icon::Add).color(FabColor::Secondary).on_press(Message::NewWorkspace), "Secondary"),
                tooltip(fab(Icon::Add).color(FabColor::Tertiary).on_press(Message::NewWorkspace), "Tertiary"),
                tooltip(fab(Icon::Add).color(FabColor::Surface).on_press(Message::NewWorkspace), "Surface"),
                tooltip(fab(Icon::Add).disabled(true), "Unavailable action"),
            ].spacing(24).wrap(),
            button("Show modal side sheet").variant(ButtonVariant::Outlined).on_press(Message::ModalDetails),
        ].spacing(24)).variant(SurfaceVariant::Outlined).into()
    }
    fn remember_search(&mut self) {
        let query = self.query.trim().to_owned();
        if !query.is_empty() {
            self.recent_searches
                .retain(|old| !old.eq_ignore_ascii_case(&query));
            self.recent_searches.insert(0, query);
            self.recent_searches.truncate(5);
        }
    }
    fn matching_files(&self) -> Vec<usize> {
        let query = self.query.trim().to_lowercase();
        [
            (
                "Design notes Working notes and decisions",
                false,
                self.pinned,
                0.0,
            ),
            (
                "Component checklist Buttons choices navigation and feedback",
                true,
                false,
                7.0,
            ),
            (
                "Archive export Unavailable in this preview",
                false,
                false,
                21.0,
            ),
        ]
        .iter()
        .enumerate()
        .filter_map(|(index, (text, shared, pinned, age))| {
            (text.to_lowercase().contains(&query)
                && (!self.filter_shared || *shared)
                && (!self.filter_pinned || *pinned)
                && *age >= self.file_age.0
                && *age <= self.file_age.1)
                .then_some(index)
        })
        .collect()
    }
    fn file_results(&self, width: f32, filtered: bool) -> Element<'_, Message> {
        let indices = if filtered {
            self.matching_files()
        } else {
            vec![0, 1, 2]
        };
        if indices.is_empty() {
            return column![
                typography("No matching files", TypeScale::TitleMedium),
                muted(
                    "Try another search or clear your filters.",
                    TypeScale::BodyMedium
                ),
                button("Clear filters")
                    .variant(ButtonVariant::Text)
                    .on_press(Message::ClearFilters),
            ]
            .spacing(12)
            .into();
        }
        let mut items = Vec::new();
        for index in indices {
            if !items.is_empty() {
                items.push(divider().into());
            }
            let item = match index {
                0 => list_item("Design notes")
                    .supporting_text("Updated today · Workspace")
                    .leading(Icon::Workspace)
                    .trailing(tooltip(
                        icon_button(Icon::Star)
                            .selected(self.pinned)
                            .on_press(Message::Pin),
                        "Pin design notes",
                    ))
                    .on_press(Message::FileDetails(0))
                    .into(),
                1 => {
                    let actions = menu(
                        "Actions",
                        [
                            MenuItem::new("Duplicate checklist", Message::Duplicate),
                            MenuItem::new("Save preferences", Message::Save),
                        ],
                    );
                    let checklist = list_item("Component checklist")
                        .overline("SHARED WITH YOUR TEAM")
                        .supporting_text(
                            "Updated 7 days ago · Buttons, choices, navigation, and feedback.",
                        )
                        .leading(Icon::Activity)
                        .on_press(Message::FileDetails(1));
                    if width < 500.0 {
                        column![checklist, container(actions).padding([8, 16])].into()
                    } else {
                        checklist.trailing(actions).into()
                    }
                }
                _ => list_item("Archive export")
                    .supporting_text("21 days ago · Unavailable in this preview")
                    .leading(Icon::Workspace)
                    .disabled(true)
                    .into(),
            };
            items.push(item);
        }
        list(items).into()
    }
    fn files(&self, width: f32) -> Element<'_, Message> {
        if self.workspace_tab != WorkspaceTab::Files {
            return surface(
                column![
                    typography("Recent files", TypeScale::TitleLarge),
                    self.file_results(width, false),
                    muted(self.status.clone(), TypeScale::BodySmall)
                ]
                .spacing(16),
            )
            .variant(SurfaceVariant::Outlined)
            .into();
        }
        let mut results = column![typography(
            if self.query.trim().is_empty() {
                "Recent searches"
            } else {
                "Search results"
            },
            TypeScale::LabelLarge
        )]
        .spacing(16);
        if self.query.trim().is_empty() {
            results = results.push(
                widget::Row::with_children(self.recent_searches.iter().map(|query| {
                    suggestion_chip(query)
                        .on_press(Message::Suggest(query.clone()))
                        .into()
                }))
                .spacing(8)
                .wrap(),
            );
        }
        results = results.push(self.file_results(width.min(720.0) - 32.0, true));
        let mut filters = row![
            filter_chip("Shared", self.filter_shared).on_press(Message::FilterShared),
            filter_chip("Pinned", self.filter_pinned).on_press(Message::FilterPinned),
        ]
        .spacing(8);
        if !self.query.is_empty() {
            filters = filters.push(
                input_chip(format!("Search: {}", self.query))
                    .on_remove(Message::Query(String::new())),
            );
        }
        surface(
            column![
                typography("Find a file", TypeScale::TitleLarge),
                search_bar("Search files", &self.query)
                    .open(self.search_open)
                    .on_open(Message::SearchOpen)
                    .on_close(Message::SearchClose)
                    .on_input(Message::Query)
                    .on_submit(Message::SearchSubmit)
                    .results(results),
                filters.wrap(),
                row![
                    typography(
                        format!(
                            "Updated {}–{} days ago",
                            self.file_age.0 as u32, self.file_age.1 as u32
                        ),
                        TypeScale::BodyMedium
                    ),
                    button("Reset")
                        .variant(ButtonVariant::Text)
                        .on_press(Message::ClearFilters)
                ]
                .align_y(Alignment::Center)
                .spacing(8)
                .wrap(),
                range_slider(0.0..=30.0, self.file_age)
                    .step(1.0)
                    .ticks(true)
                    .labeled(true)
                    .value_labels(
                        format!("{} days", self.file_age.0 as u32),
                        format!("{} days", self.file_age.1 as u32)
                    )
                    .on_change(Message::FileAge),
                divider(),
                typography(
                    format!(
                        "{} matching {}",
                        self.matching_files().len(),
                        if self.matching_files().len() == 1 {
                            "file"
                        } else {
                            "files"
                        }
                    ),
                    TypeScale::TitleMedium
                ),
                self.file_results(width, true),
            ]
            .spacing(16),
        )
        .variant(SurfaceVariant::Outlined)
        .into()
    }
    fn chip_variants(&self) -> Element<'_, Message> {
        surface(
            column![
                typography("Chips", TypeScale::TitleLarge),
                muted(
                    "Actions, suggestions, filters, and removable inputs.",
                    TypeScale::BodyMedium
                ),
                row![
                    assist_chip("Add to calendar")
                        .leading(Icon::Add.sized(18.0))
                        .on_press(Message::Action("Added to calendar · Demo")),
                    suggestion_chip("Design").on_press(Message::Suggest("Design".into())),
                    filter_chip("Shared", self.filter_shared).on_press(Message::FilterShared),
                    if self.sample_input {
                        Element::from(
                            input_chip("Team: Studio").on_remove(Message::SampleInput(false)),
                        )
                    } else {
                        Element::from(
                            assist_chip("Restore input").on_press(Message::SampleInput(true)),
                        )
                    },
                ]
                .spacing(12)
                .wrap(),
                row![
                    assist_chip("Unavailable"),
                    filter_chip("Selected", true).disabled(true),
                    input_chip("Managed")
                        .on_remove(Message::Action("Managed"))
                        .disabled(true)
                ]
                .spacing(12)
                .wrap(),
            ]
            .spacing(16),
        )
        .variant(SurfaceVariant::Outlined)
        .into()
    }
    fn activity_page(&self) -> Element<'_, Message> {
        let unread = self.read.iter().filter(|read| !**read).count() as u32;
        let rows = [
            ("Workspace created", "Your team's shared space is ready."),
            (
                "Preferences saved",
                "Delivery and appearance preferences were updated.",
            ),
            ("Design notes updated", "A new version is ready to review."),
        ]
        .into_iter()
        .enumerate()
        .filter(|(index, _)| self.activity_tab != ActivityTab::Unread || !self.read[*index])
        .map(|(index, (headline, supporting))| {
            list_item(headline)
                .supporting_text(supporting)
                .leading(Icon::Activity)
                .trailing(
                    checkbox(self.read[index]).on_toggle(move |read| Message::Read(index, read)),
                )
                .selected(!self.read[index])
                .on_press(Message::Read(index, !self.read[index]))
                .into()
        })
        .collect::<Vec<_>>();
        let empty = rows.is_empty();
        column![
            typography("Keep up with your workspace", TypeScale::HeadlineMedium),
            muted(
                "Click a row or its checkbox to mark it read. Each click changes it once.",
                TypeScale::BodyLarge
            ),
            tabs(
                [
                    Tab::new(ActivityTab::All, "All updates"),
                    Tab::new(ActivityTab::Unread, "Unread").badge(unread),
                    Tab::new(ActivityTab::Archived, "Archived").disabled(true)
                ],
                Some(self.activity_tab)
            )
            .on_select(Message::ActivityTab)
            .scrollable(true),
            list(rows),
            muted(
                if empty {
                    "You're all caught up."
                } else {
                    "Checked items are read. Unread items have a tinted surface."
                },
                TypeScale::BodyMedium
            ),
            code("tabs(items, selected).on_select(Message::ActivityTab)"),
        ]
        .spacing(24)
        .into()
    }
    fn switch_states(&self) -> Element<'_, Message> {
        surface(
            column![
                typography("Switches and icon actions", TypeScale::TitleLarge),
                list([
                    list_item("Automatic saving")
                        .supporting_text("Hover or press the switch to see its thumb feedback.")
                        .trailing(switch(self.autosave).on_toggle(Message::Autosave))
                        .on_press(Message::Autosave(!self.autosave))
                        .into(),
                    list_item("Workspace notifications")
                        .trailing(switch(self.notifications).on_toggle(Message::Notifications))
                        .into(),
                    list_item("Managed by your team")
                        .trailing(switch(true))
                        .disabled(true)
                        .into(),
                ]),
                row![
                    tooltip(
                        icon_button(Icon::Star)
                            .selected(self.pinned)
                            .on_press(Message::Pin),
                        "Standard toggle"
                    ),
                    tooltip(
                        icon_button(Icon::Star)
                            .variant(ButtonVariant::Outlined)
                            .selected(self.pinned)
                            .on_press(Message::Pin),
                        "Outlined toggle"
                    ),
                    tooltip(
                        icon_button(Icon::Star)
                            .variant(ButtonVariant::Filled)
                            .selected(self.pinned)
                            .on_press(Message::Pin),
                        "Filled toggle"
                    ),
                    tooltip(
                        icon_button(Icon::Star)
                            .variant(ButtonVariant::Tonal)
                            .selected(self.pinned)
                            .on_press(Message::Pin),
                        "Tonal toggle"
                    ),
                    icon_button(Icon::Star).variant(ButtonVariant::Filled),
                ]
                .spacing(16)
                .wrap(),
            ]
            .spacing(16),
        )
        .variant(SurfaceVariant::Outlined)
        .into()
    }
    fn appearance(&self) -> Element<'_, Message> {
        let palette = row![
            chip("Violet", self.accent_hex.eq_ignore_ascii_case("#6750A4"))
                .on_press(Message::Accent(Color::from_rgb8(103, 80, 164), "#6750A4")),
            chip("Teal", self.accent_hex.eq_ignore_ascii_case("#006A6A"))
                .on_press(Message::Accent(Color::from_rgb8(0, 106, 106), "#006A6A")),
            chip("Clay", self.accent_hex.eq_ignore_ascii_case("#9C4233"))
                .on_press(Message::Accent(Color::from_rgb8(156, 66, 51), "#9C4233"))
        ]
        .spacing(8)
        .wrap();
        let mut accent = text_field("Accent color", &self.accent_hex)
            .on_input(Message::AccentHex)
            .width(180);
        if parse_hex(&self.accent_hex).is_none() {
            accent = accent.error("Use #RRGGBB");
        }
        surface(
            row![
                column![
                    typography("Your palette", TypeScale::Label),
                    muted("Every component, one theme", TypeScale::Supporting)
                ]
                .spacing(4),
                palette,
                accent,
                switch(self.dark)
                    .label("Dark theme")
                    .on_toggle(Message::Dark)
            ]
            .spacing(24)
            .align_y(Alignment::Center)
            .wrap(),
        )
        .variant(SurfaceVariant::Outlined)
        .padding(20)
        .into()
    }
    fn settings(&self) -> Element<'_, Message> {
        let mut name = text_field("Workspace name", &self.name)
            .on_input(Message::Name)
            .supporting_text("A name for your next good idea.");
        if self.valid() {
            name = name.on_submit(Message::Open);
        }
        if self.name.trim().is_empty() {
            name = name.error("Give your workspace a name.");
        }
        let mut email = text_field("Email address", &self.email)
            .on_input(Message::Email)
            .supporting_text("Used for workspace updates.");
        if !self.email.contains('@') || !self.email.contains('.') {
            email = email.error("Enter an email address, like hello@example.com.");
        }
        let tags = row![
            chip("Design", self.selected[0]).on_press(Message::Select(0)),
            chip("Development", self.selected[1]).on_press(Message::Select(1)),
            chip("Research", self.selected[2]).on_press(Message::Select(2))
        ]
        .spacing(8)
        .wrap();
        surface(
            column![
                eyebrow("01  /  REAL-WORLD COMPOSITION"),
                typography("Workspace settings", TypeScale::Headline),
                muted(
                    "A small form with a little room to breathe.",
                    TypeScale::Body
                ),
                name,
                email,
                divider(),
                switch(self.autosave)
                    .label("Save work automatically")
                    .on_toggle(Message::Autosave),
                checkbox(self.notifications)
                    .label("Send me workspace updates")
                    .on_toggle(Message::Notifications),
                typography("Your interests", TypeScale::Label),
                tags,
                row![
                    button("Review changes")
                        .on_press(Message::Open)
                        .disabled(!self.valid()),
                    muted(
                        if self.saved {
                            "Up to date"
                        } else {
                            "Unsaved changes"
                        },
                        TypeScale::Supporting
                    )
                ]
                .spacing(16)
                .align_y(Alignment::Center)
                .wrap(),
                code("text_field(\"Workspace name\", &name)\n    .on_input(Message::Name)")
            ]
            .spacing(16),
        )
        .into()
    }
    fn actions(&self) -> Element<'_, Message> {
        surface(column![eyebrow("02  /  INTERACTION LAB"),typography("Small actions, clear intent",TypeScale::Title),
            muted("Hover, press, drag away, and release. State layers settle when you're done.",TypeScale::Body),
            row![button("Filled").on_press(Message::Action("Filled button")),button("Outlined").variant(ButtonVariant::Outlined).on_press(Message::Action("Outlined button"))].spacing(8).wrap(),
            row![button("Text").variant(ButtonVariant::Text).on_press(Message::Action("Text button")),button("Tonal").variant(ButtonVariant::Tonal).on_press(Message::Action("Tonal button"))].spacing(8).wrap(),
            row![button("Disabled"),button("Disabled").variant(ButtonVariant::Outlined),button("Disabled").variant(ButtonVariant::Text)].spacing(8).wrap(),
            code("button(\"Outlined\")\n    .variant(ButtonVariant::Outlined)\n    .on_press(Message::Action)"),
            divider(),typography("Icons & indicators",TypeScale::Label),
            row![icon_button(Icon::Add).on_press(Message::Action("Add")),
                icon_button(Icon::Star).selected(true).on_press(Message::Action("Favorite")),
                icon_button(Icon::Close),typography("Inbox",TypeScale::Body),badge(self.clicks),badge(120)
            ].spacing(12).align_y(Alignment::Center).wrap(),
            muted(self.status.clone(),TypeScale::Supporting),divider(),typography("Dialog behavior",TypeScale::Label),
            checkbox(self.outside).label("Click outside to dismiss").on_toggle(Message::Outside),
            button("Open dialog").variant(ButtonVariant::Outlined).on_press(Message::Open)
        ].spacing(16)).variant(SurfaceVariant::Outlined).into()
    }
    fn type_and_surfaces(&self) -> Element<'_, Message> {
        surface(
            column![
                eyebrow("03  /  FOUNDATION"),
                typography("One visual language", TypeScale::Title),
                typography("Headline small · 24 / 32", TypeScale::HeadlineSmall),
                typography("Title medium · 16 / 24 · Medium", TypeScale::TitleMedium),
                typography("Body large · 16 / 24 · Roboto Regular", TypeScale::Body),
                muted(
                    "Body small · 12 / 16 · Roboto Regular",
                    TypeScale::Supporting
                ),
                divider(),
                row![
                    surface(typography("Filled", TypeScale::Label))
                        .width(120)
                        .padding(16),
                    surface(typography("Outlined", TypeScale::Label))
                        .variant(SurfaceVariant::Outlined)
                        .width(120)
                        .padding(16),
                    surface(typography("Elevated", TypeScale::Label))
                        .variant(SurfaceVariant::Elevated)
                        .width(120)
                        .padding(16)
                ]
                .spacing(12)
                .wrap(),
                code("surface(content)\n    .variant(SurfaceVariant::Elevated)")
            ]
            .spacing(16),
        )
        .variant(SurfaceVariant::Outlined)
        .into()
    }
    fn access_select(&self) -> iced_m3::Select<'_, Access, Message> {
        select(
            "Workspace access",
            [
                SelectOption::new(Access::Viewer, "Viewer"),
                SelectOption::new(Access::Editor, "Editor"),
                SelectOption::new(Access::Owner, "Owner (managed)").disabled(true),
            ],
            Some(self.access),
        )
        .on_select(Message::Access)
        .width(260.0)
    }
    fn workflow(&self) -> Element<'_, Message> {
        let choices = column![
            typography("Delivery schedule", TypeScale::TitleMedium),
            radio_group(
                [
                    RadioOption::new(Delivery::Immediately, "As things happen"),
                    RadioOption::new(Delivery::Daily, "Daily summary"),
                    RadioOption::new(Delivery::Weekly, "Weekly summary").disabled(true),
                ],
                Some(self.delivery)
            )
            .on_select(Message::Delivery),
            self.access_select(),
        ]
        .spacing(8)
        .width(280);
        let actions = column![
            typography("Actions where you need them", TypeScale::TitleMedium),
            row![
                menu(
                    "Workspace actions",
                    [
                        MenuItem::new("Duplicate workspace", Message::Duplicate).leading(Icon::Add),
                        MenuItem::new("Save preferences", Message::Save),
                        MenuItem::separator(),
                        MenuItem::new("Share workspace", Message::Action("Share")).disabled(true),
                    ]
                ),
                tooltip(
                    icon_button(Icon::Star).on_press(Message::Action("Pinned workspace")),
                    "Pin this workspace"
                ),
            ]
            .spacing(12)
            .align_y(Alignment::Center)
            .wrap(),
            context_menu(
                surface(
                    column![
                        typography("Workspace card", TypeScale::LabelLarge),
                        muted("Right-click for contextual actions.", TypeScale::BodyMedium),
                    ]
                    .spacing(4)
                )
                .variant(SurfaceVariant::Outlined)
                .padding(16),
                [
                    MenuItem::new("Duplicate workspace", Message::Duplicate),
                    MenuItem::new("Save preferences", Message::Save),
                ]
            ),
            tooltip(
                button("Show confirmation")
                    .variant(ButtonVariant::Text)
                    .on_press(Message::ShowNotice),
                "A snackbar appears at the bottom of the window"
            ),
        ]
        .spacing(16)
        .width(400);
        surface(column![
            eyebrow("NEW  /  EVERYDAY WORKFLOWS"),
            typography("Choices, actions, and feedback", TypeScale::TitleLarge),
            muted("Choose an option, open a menu, or duplicate the workspace and undo it.", TypeScale::BodyMedium),
            row![choices, actions].spacing(32).wrap(),
            code("menu(\"Workspace actions\", items)\nradio_group(options, selected).on_select(Message::Delivery)"),
        ].spacing(16)).variant(SurfaceVariant::Outlined).into()
    }
    fn checkbox_states(&self) -> Element<'_, Message> {
        let all = self.selected.iter().all(|value| *value);
        let any = self.selected.iter().any(|value| *value);
        surface(column![
            eyebrow("05  /  SELECTION STATES"),
            typography("A little feedback, at every step", TypeScale::TitleLarge),
            muted("Hover either the box or its label. Press, release, and watch the mark settle.", TypeScale::BodyMedium),
            row![
                checkbox(self.notifications).label("Workspace updates").on_toggle(Message::Notifications),
                checkbox(all).indeterminate(any && !all).label("All interests").on_toggle(Message::SelectAll),
                checkbox(self.required_choice).error(!self.required_choice).label("Required choice").on_toggle(Message::RequiredChoice),
            ].spacing(24).wrap(),
            row![checkbox(false).label("Disabled"), checkbox(true).label("Disabled selected")].spacing(24).wrap(),
            muted("18 px box · 40 px circular state layer · 48 px target · Roboto 16 / 24", TypeScale::BodySmall),
            code("checkbox(all_selected)\n    .indeterminate(some_selected && !all_selected)\n    .on_toggle(Message::SelectAll)"),
        ].spacing(12)).variant(SurfaceVariant::Outlined).into()
    }
    fn edge_cases(&self) -> Element<'_, Message> {
        surface(column![eyebrow("04  /  THE DETAILS"),typography("Built for real content",TypeScale::Title),
            text_field::<Message>("Unavailable field","Managed by your organization").supporting_text("Disabled inputs retain their value."),
            text_field("A deliberately long floating label to check clipping at narrow widths",&self.long_value)
                .on_input(Message::LongValue).error("Supporting and error text wrap, even when a message takes more than a single line."),
            checkbox::<Message>(true).label("Disabled, checked"),switch::<Message>(false).label("Disabled switch"),
            Button::new(row![typography("+",TypeScale::Title),typography("Custom child content",TypeScale::Label)].spacing(8).align_y(Alignment::Center))
                .variant(ButtonVariant::Tonal).on_press(Message::Action("Custom content")),
            code("Button::new(row![icon, label])\n    .on_press(Message::Action)"),
            muted("This preview is mouse-first. Screen-reader integration and full keyboard navigation are future work.",TypeScale::Supporting)
        ].spacing(16)).variant(SurfaceVariant::Outlined).into()
    }
}
fn muted<'a>(
    value: impl widget::text::IntoFragment<'a>,
    scale: TypeScale,
) -> widget::Text<'a, Theme> {
    typography(value, scale).style(|t: &Theme| widget::text::Style {
        color: Some(t.colors.on_surface_variant),
    })
}
fn eyebrow(value: &str) -> widget::Text<'_, Theme> {
    typography(value, TypeScale::Supporting).style(|t: &Theme| widget::text::Style {
        color: Some(t.colors.primary),
    })
}
fn code(value: &'static str) -> Element<'static, Message> {
    container(typography(value, TypeScale::Supporting).font(iced::Font::MONOSPACE))
        .padding(12)
        .width(Length::Fill)
        .style(|t: &Theme| widget::container::Style {
            background: Some(t.colors.surface_container_high.into()),
            text_color: Some(t.colors.on_surface_variant),
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}
fn parse_hex(value: &str) -> Option<Color> {
    let value = value.strip_prefix('#').unwrap_or(value);
    if value.len() != 6 || !value.is_ascii() {
        return None;
    }
    let number = u32::from_str_radix(value, 16).ok()?;
    Some(Color::from_rgb8(
        (number >> 16) as u8,
        (number >> 8) as u8,
        number as u8,
    ))
}
// Simulated work is application-owned. The controls never publish animation ticks.
fn export_task(
    id: u64,
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> iced::Task<Message> {
    let (sender, receiver) = iced::futures::channel::mpsc::unbounded();
    std::thread::spawn(move || {
        for tick in 1..=40 {
            std::thread::sleep(std::time::Duration::from_millis(100));
            if cancelled.load(std::sync::atomic::Ordering::Relaxed)
                || sender.unbounded_send((id, tick)).is_err()
            {
                break;
            }
        }
    });
    iced::Task::run(receiver, |(id, tick)| Message::ExportTick(id, tick))
}
fn main() -> iced::Result {
    iced::application(Gallery::default, Gallery::update, Gallery::view)
        .title("Material Gallery")
        .theme(Gallery::theme)
        .default_font(iced_m3::fonts::REGULAR)
        .window(iced::window::Settings {
            size: Size::new(1160.0, 920.0),
            min_size: Some(Size::new(320.0, 480.0)),
            ..Default::default()
        })
        .run()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn export_demo_selects_options_disables_changes_and_completes() {
        let mut gallery = Gallery::default();
        click_and_update(&mut gallery, 390.0, "Export");
        assert_eq!(click_and_update(&mut gallery, 390.0, "JPEG"), 1);
        assert_eq!(gallery.export_format, 1);
        assert_eq!(click_and_update(&mut gallery, 390.0, "Tags"), 1);
        assert_eq!(gallery.export_options, [0, 1]);
        assert_eq!(click_and_update(&mut gallery, 390.0, "Start export"), 1);
        assert_eq!(gallery.export_tick, Some(0));
        assert_eq!(click_and_update(&mut gallery, 390.0, "PNG"), 0);
        let _ = gallery.update(Message::ExportQuality(15.0));
        assert_eq!(gallery.export_quality, 80.0);
        let _ = gallery.update(Message::ExportTick(gallery.export_id, 20));
        assert_eq!(gallery.export_tick, Some(20));
        let _ = gallery.update(Message::ExportTick(gallery.export_id, 40));
        assert!(gallery.export_tick.is_none());
        assert!(
            gallery
                .notice
                .as_ref()
                .unwrap()
                .1
                .contains("Export complete")
        );
    }
    #[test]
    fn cancelling_and_restarting_export_ignores_old_worker_updates() {
        let mut gallery = Gallery::default();
        let _ = gallery.update(Message::StartExport);
        let old_id = gallery.export_id;
        let cancel = gallery.export_cancel.clone().unwrap();
        let _ = gallery.update(Message::CancelExport);
        assert!(cancel.load(std::sync::atomic::Ordering::Relaxed));
        let _ = gallery.update(Message::ExportTick(old_id, 40));
        assert!(gallery.export_tick.is_none());
        let _ = gallery.update(Message::StartExport);
        let _ = gallery.update(Message::ExportTick(old_id, 40));
        assert_eq!(gallery.export_tick, Some(0));
        let _ = gallery.update(Message::ExportTick(gallery.export_id, 20));
        let _ = gallery.update(Message::ExportTick(gallery.export_id, 10));
        assert_eq!(gallery.export_tick, Some(20));
        let _ = gallery.update(Message::CancelExport);
    }
    #[test]
    #[ignore = "reference suite; run with --ignored"]
    fn visual_references_export_gallery() {
        for dark in [false, true] {
            for width in [320.0, 390.0, 1160.0] {
                for tick in [None, Some(0), Some(24)] {
                    let gallery = Gallery {
                        dark,
                        workspace_tab: WorkspaceTab::Export,
                        export_tick: tick,
                        ..Default::default()
                    };
                    let mut ui = visual_harness::Harness::new(
                        gallery.view(),
                        Size::new(width, 1000.0),
                        gallery.theme(),
                    );
                    ui.at(0);
                    reference::check(
                        &format!(
                            "gallery-export/{}/{}/{}",
                            if dark { "dark" } else { "light" },
                            width as u32,
                            match tick {
                                None => "idle",
                                Some(0) => "preparing",
                                _ => "running",
                            }
                        ),
                        &ui.frame(),
                    );
                }
            }
        }
    }
    #[test]
    fn file_details_keep_per_file_notes_and_navigation_closes_the_panel() {
        let mut gallery = Gallery {
            workspace_tab: WorkspaceTab::Files,
            ..Default::default()
        };
        assert_eq!(click_and_update(&mut gallery, 1160.0, "Design notes"), 1);
        assert!(gallery.details_open);
        let _ = gallery.update(Message::FileNote("Review the new motion".into()));
        let _ = gallery.update(Message::CloseDetails);
        assert_eq!(
            click_and_update(&mut gallery, 1160.0, "Component checklist"),
            1
        );
        assert_eq!(gallery.details_index, 1);
        assert!(gallery.file_notes[1].is_empty());
        let _ = gallery.update(Message::Page(Page::Activity));
        assert!(!gallery.details_open);
        assert_eq!(gallery.file_notes[0], "Review the new motion");
    }
    #[test]
    fn new_workspace_fab_creates_a_named_workspace_and_rejects_empty_names() {
        let mut gallery = Gallery {
            workspace_tab: WorkspaceTab::Files,
            ..Default::default()
        };
        assert_eq!(click_and_update(&mut gallery, 1160.0, "New workspace"), 1);
        assert!(gallery.creating);
        assert_eq!(
            click_and_update(&mut gallery, 1160.0, "Create workspace"),
            0
        );
        assert!(gallery.creating);
        let _ = gallery.update(Message::NewName("  Illustration studio  ".into()));
        assert_eq!(
            click_and_update(&mut gallery, 1160.0, "Create workspace"),
            1
        );
        assert_eq!(gallery.name, "Illustration studio");
        assert!(!gallery.creating);
        assert!(gallery.notice.as_ref().unwrap().1.contains("created"));
    }
    #[test]
    #[ignore = "reference suite; run with --ignored"]
    fn visual_references_sheet_gallery() {
        for dark in [false, true] {
            for width in [320.0, 390.0, 1160.0] {
                for case in ["files", "details", "modal-details", "new-workspace"] {
                    let gallery = Gallery {
                        dark,
                        workspace_tab: WorkspaceTab::Files,
                        details_open: case.contains("details"),
                        details_modal: case == "modal-details",
                        creating: case == "new-workspace",
                        new_name: "Illustration studio".into(),
                        ..Default::default()
                    };
                    let mut theme = gallery.theme();
                    theme.motion.sheet_enter = std::time::Duration::ZERO;
                    let mut ui = visual_harness::Harness::new(
                        gallery.view(),
                        Size::new(width, 844.0),
                        theme,
                    );
                    ui.frame();
                    ui.at(0);
                    ui.at(1);
                    reference::check(
                        &format!(
                            "gallery-sheets/{}/{}/{case}",
                            if dark { "dark" } else { "light" },
                            width as u32
                        ),
                        &ui.frame(),
                    );
                    if case == "details" {
                        ui.move_to((width - 60.0, 720.0));
                        ui.event(iced::Event::Mouse(iced::mouse::Event::WheelScrolled {
                            delta: iced::mouse::ScrollDelta::Lines { x: 0.0, y: -100.0 },
                        }));
                        ui.leave();
                        reference::check(
                            &format!(
                                "gallery-sheets/{}/{case}-{}/scrolled",
                                if dark { "dark" } else { "light" },
                                width as u32
                            ),
                            &ui.frame(),
                        );
                    }
                }
            }
            let gallery = Gallery {
                dark,
                ..Default::default()
            };
            let mut ui = visual_harness::Harness::new(
                container(gallery.primary_actions()).padding(24),
                Size::new(760.0, 580.0),
                gallery.theme(),
            );
            ui.at(0);
            reference::check(
                &format!(
                    "gallery-fabs/{}/overview",
                    if dark { "dark" } else { "light" }
                ),
                &ui.frame(),
            );
        }
    }
    #[test]
    fn preferences_dialog_select_matches_its_host_surface() {
        for dark in [false, true] {
            let gallery = Gallery {
                dark,
                open: true,
                ..Default::default()
            };
            let mut ui = visual_harness::Harness::new(
                gallery.view(),
                Size::new(1160.0, 844.0),
                gallery.theme(),
            );
            ui.at(0);
            ui.frame();
            ui.at(600);
            let field = ui.find("Workspace access");
            let image = ui.frame();
            let pixel = |x: f32, y: f32| {
                let index = (((y * 2.0) as u32 * image.width + (x * 2.0) as u32) * 4) as usize;
                &image.pixels[index..index + 4]
            };
            let outside = pixel(field.x + 8.0, field.y + 2.0);
            assert_eq!(outside, pixel(field.x + 13.0, field.y + 2.0));
            assert_eq!(outside, pixel(field.x + field.width - 40.0, field.y + 45.0));
            image.write(&std::path::PathBuf::from(format!(
                "target/visual-report/dialog-fix-gallery/{}.png",
                if dark { "dark" } else { "light" },
            )));
        }
    }
    #[test]
    #[ignore = "profiles real dialog animation frames at 2x display scale"]
    fn dialog_animation_profile() {
        let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
        let closed = Gallery::default();
        let opened = Gallery {
            open: true,
            ..Default::default()
        };
        let theme = opened.theme();
        let mut ui = visual_harness::Harness::with_backend(
            closed.view(),
            Size::new(1160.0, 844.0),
            theme,
            &backend,
        );
        ui.at(0);
        ui.frame();
        ui.rebuild(opened.view());
        let mut costs = Vec::new();
        for ms in (0..=512).step_by(16) {
            let start = std::time::Instant::now();
            ui.at(ms);
            ui.frame();
            costs.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        let mean = costs.iter().sum::<f64>() / costs.len() as f64;
        costs.sort_by(f64::total_cmp);
        eprintln!(
            "{backend}: mean={mean:.2}ms median={:.2}ms p95={:.2}ms max={:.2}ms (includes screenshot readback)",
            costs[costs.len() / 2],
            costs[costs.len() * 95 / 100],
            costs.last().unwrap()
        );
    }
    #[test]
    #[ignore = "measures software redraw cost at 2x display scale"]
    fn software_dialog_redraw_profile() {
        use iced::advanced::renderer::Headless;
        use iced_test::renderer::graphics::{Viewport, damage};
        use iced_test::runtime::{UserInterface, user_interface};
        use std::time::{Duration, Instant};

        let mut renderer: iced::Renderer = iced_test::futures::futures::executor::block_on(
            <iced::Renderer as Headless>::new(iced::Font::DEFAULT, 16.0.into(), Some("tiny-skia")),
        )
        .unwrap();
        let mut gallery = Gallery {
            open: true,
            ..Default::default()
        };
        let size = Size::new(1160.0, 844.0);
        let bounds = iced::Rectangle::with_size(size);
        let viewport = Viewport::with_physical_size(Size::new(2320, 1688), 2.0);
        let mut pixels = tiny_skia::Pixmap::new(2320, 1688).unwrap();
        let mut mask = tiny_skia::Mask::new(2320, 1688).unwrap();
        let mut cache = user_interface::Cache::default();
        let mut previous = Vec::new();
        for frame in 0..12 {
            gallery.note.push('a');
            let start = Instant::now();
            let mut ui = UserInterface::build(gallery.view(), size, cache, &mut renderer);
            let cursor = iced::mouse::Cursor::Unavailable;
            let mut messages = Vec::new();
            if frame == 0 {
                ui.operate(
                    &renderer,
                    &mut iced::advanced::widget::operation::focusable::focus::<()>(
                        iced::widget::Id::new("gallery-note"),
                    ),
                );
            }
            ui.update(
                &[iced::Event::Window(iced::window::Event::RedrawRequested(
                    iced::time::Instant::now() + Duration::from_secs(1),
                ))],
                cursor,
                &mut renderer,
                &mut iced::advanced::clipboard::Null,
                &mut messages,
            );
            ui.draw(&mut renderer, &gallery.theme(), &Default::default(), cursor);
            cache = ui.into_cache();
            let layout_time = start.elapsed();
            #[cfg(feature = "wgpu")]
            let iced::Renderer::Secondary(software) = &mut renderer else {
                panic!("software renderer required")
            };
            #[cfg(not(feature = "wgpu"))]
            let software = &mut renderer;
            let layers = software.layers().to_vec();
            let regions = damage::group(
                damage::diff(
                    &previous,
                    &layers,
                    |layer| vec![layer.bounds],
                    iced_tiny_skia::Layer::damage,
                ),
                bounds,
            );
            let start = Instant::now();
            software.draw(
                &mut pixels.as_mut(),
                &mut mask,
                &viewport,
                &regions,
                gallery.theme().colors.surface,
            );
            eprintln!(
                "frame={frame} layout={layout_time:?} paint={:?} regions={regions:?}",
                start.elapsed()
            );
            previous = layers;
        }
    }
    #[test]
    fn workflow_gallery_duplicates_and_undoes_through_component_messages() {
        let mut gallery = Gallery::default();
        let original = gallery.name.clone();
        let mut ui = iced_test::Simulator::new(gallery.view());
        ui.click("Workspace actions").unwrap();
        ui.click("Duplicate workspace").unwrap();
        for message in ui.into_messages() {
            let _ = gallery.update(message);
        }
        assert_eq!(gallery.name, format!("{original} copy"));
        let stale_id = gallery.notice_id;
        let mut ui = iced_test::Simulator::new(gallery.view());
        ui.click("Undo").unwrap();
        for message in ui.into_messages() {
            let _ = gallery.update(message);
        }
        assert_eq!(gallery.name, original);
        let _ = gallery.update(Message::DismissNotice(stale_id));
        assert!(
            gallery.notice.is_some(),
            "old timeout cannot dismiss a new notice"
        );
    }

    #[test]
    fn accent_hex_parser_handles_partial_invalid_and_unicode_input() {
        assert_eq!(parse_hex("#006A6A"), Some(Color::from_rgb8(0, 106, 106)));
        for value in ["#", "#zzzzzz", "#FFF", "🦀abc"] {
            assert!(parse_hex(value).is_none());
        }
    }
    fn click_and_update(gallery: &mut Gallery, width: f32, label: &str) -> usize {
        let mut ui = iced_test::Simulator::with_size(
            Default::default(),
            Size::new(width, 1800.0),
            gallery.view(),
        );
        ui.click(label).unwrap();
        let messages = ui.into_messages().collect::<Vec<_>>();
        let count = messages.len();
        for message in messages {
            let _ = gallery.update(message);
        }
        count
    }
    #[test]
    fn gallery_navigation_preserves_preferences_across_wide_and_narrow_pages() {
        let mut gallery = Gallery::default();
        let autosave = gallery.autosave;
        assert_eq!(click_and_update(&mut gallery, 1160.0, "Settings"), 1);
        assert_eq!(gallery.page, Page::Settings);
        assert_eq!(click_and_update(&mut gallery, 390.0, "Automatic saving"), 1);
        assert_eq!(gallery.autosave, !autosave);
        assert_eq!(click_and_update(&mut gallery, 390.0, "Workspace"), 1);
        assert_eq!(click_and_update(&mut gallery, 1160.0, "Files"), 1);
        assert_eq!(gallery.workspace_tab, WorkspaceTab::Files);
        assert_eq!(click_and_update(&mut gallery, 1160.0, "Activity"), 1);
        assert_eq!(click_and_update(&mut gallery, 390.0, "Workspace"), 1);
        assert_eq!(gallery.workspace_tab, WorkspaceTab::Files);
        assert_eq!(gallery.autosave, !autosave);
    }
    #[test]
    fn activity_row_updates_once_and_unread_filter_removes_it_after_rebuild() {
        let mut gallery = Gallery::default();
        click_and_update(&mut gallery, 1160.0, "Activity");
        click_and_update(&mut gallery, 390.0, "Unread");
        assert!(!gallery.read[0]);
        assert_eq!(
            click_and_update(&mut gallery, 390.0, "Workspace created"),
            1
        );
        assert!(gallery.read[0]);
        let mut ui = iced_test::Simulator::new(gallery.view());
        assert!(ui.find("Workspace created").is_err());
        assert!(ui.find("Design notes updated").is_ok());
        drop(ui);
        click_and_update(&mut gallery, 1160.0, "Design notes updated");
        let mut ui = iced_test::Simulator::new(gallery.view());
        assert!(ui.find("You're all caught up.").is_ok());
    }
    #[test]
    #[ignore = "writes gallery renders for manual visual inspection"]
    fn gallery_snapshots() {
        // Software-only is the documented default for this artifact generator.
        // Set ICED_TEST_BACKEND=wgpu explicitly to inspect the GPU renderer too.
        let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_else(|_| "tiny-skia".into());
        for (name, width, height, dark, open) in [
            ("light", 1160.0, 1800.0, false, false),
            ("dark", 1160.0, 1800.0, true, false),
            ("narrow", 390.0, 3600.0, false, false),
            ("narrow-dark", 390.0, 3600.0, true, false),
            ("dialog", 1160.0, 844.0, false, true),
            ("dialog-dark", 390.0, 844.0, true, true),
            ("overview", 1160.0, 920.0, false, false),
            ("minimum", 320.0, 560.0, false, true),
            ("narrow-bottom", 320.0, 760.0, true, false),
            ("activity", 1160.0, 920.0, false, false),
            ("activity-dark", 390.0, 844.0, true, false),
            ("settings", 1160.0, 1400.0, false, false),
            ("settings-narrow", 320.0, 2200.0, false, false),
            ("components", 1160.0, 2200.0, false, false),
            ("checkboxes", 960.0, 440.0, false, false),
            ("checkboxes-dark", 960.0, 440.0, true, false),
        ] {
            let gallery = Gallery {
                dark,
                open,
                page: if name.starts_with("activity") {
                    Page::Activity
                } else if name.starts_with("settings") {
                    Page::Settings
                } else {
                    Page::Workspace
                },
                workspace_tab: if name == "narrow-bottom" || name == "components" {
                    WorkspaceTab::Components
                } else {
                    WorkspaceTab::Overview
                },
                ..Default::default()
            };
            let mut ui = iced_test::Simulator::with_size(
                iced::Settings::default(),
                Size::new(width, height),
                if name.starts_with("checkboxes") {
                    container(gallery.checkbox_states()).padding(24).into()
                } else {
                    gallery.view()
                },
            );
            if name.starts_with("checkboxes") {
                ui.point_at((72.0, 172.0));
                let now = iced::time::Instant::now();
                ui.simulate([
                    iced::Event::Window(iced::window::Event::RedrawRequested(now)),
                    iced::Event::Window(iced::window::Event::RedrawRequested(
                        now + std::time::Duration::from_secs(1),
                    )),
                ]);
            }
            if name == "narrow-bottom" {
                ui.point_at((160.0, 400.0));
                ui.simulate([iced::Event::Mouse(iced::mouse::Event::WheelScrolled {
                    delta: iced::mouse::ScrollDelta::Lines { x: 0.0, y: -1000.0 },
                })]);
            }
            let destination = format!("target/visuals/gallery-{name}-{backend}.png");
            if std::path::Path::new(&destination).exists() {
                std::fs::remove_file(destination).unwrap();
            }
            ui.snapshot(&gallery.theme())
                .unwrap()
                .matches_image(format!("target/visuals/gallery-{name}.png"))
                .unwrap();
        }
    }

    #[test]
    #[ignore = "reference suite; run with --ignored"]
    fn visual_references_gallery() {
        for (name, page, width, height, dark, bottom) in [
            (
                "workspace-light",
                Page::Workspace,
                1160.0,
                920.0,
                false,
                false,
            ),
            (
                "activity-dark-narrow",
                Page::Activity,
                390.0,
                844.0,
                true,
                false,
            ),
            (
                "settings-dark-snackbar",
                Page::Settings,
                1160.0,
                740.0,
                true,
                true,
            ),
            (
                "settings-light-fields",
                Page::Settings,
                1160.0,
                740.0,
                false,
                false,
            ),
            (
                "settings-dark-fields",
                Page::Settings,
                1160.0,
                740.0,
                true,
                false,
            ),
        ] {
            let gallery = Gallery {
                page,
                dark,
                notice: bottom.then(|| (1, "Workspace unpinned".into(), false)),
                ..Default::default()
            };
            let mut ui = visual_harness::Harness::new(
                gallery.view(),
                Size::new(width, height),
                gallery.theme(),
            );
            ui.at(0);
            if bottom {
                ui.move_to((width - 20.0, height / 2.0));
                ui.event(iced::Event::Mouse(iced::mouse::Event::WheelScrolled {
                    delta: iced::mouse::ScrollDelta::Lines { x: 0.0, y: -1000.0 },
                }));
                ui.leave();
                ui.at(1000);
            }
            reference::check(&format!("gallery/{name}"), &ui.frame());
        }
    }
    #[test]
    fn file_search_combines_query_chips_and_range_and_can_reset() {
        let mut gallery = Gallery {
            workspace_tab: WorkspaceTab::Files,
            ..Default::default()
        };
        assert_eq!(gallery.matching_files(), [0, 1, 2]);
        let _ = gallery.update(Message::Query("  CHECKLIST  ".into()));
        assert_eq!(gallery.matching_files(), [1]);
        let _ = gallery.update(Message::FilterShared);
        assert_eq!(gallery.matching_files(), [1]);
        let _ = gallery.update(Message::FileAge((0.0, 3.0)));
        assert!(gallery.matching_files().is_empty());
        let _ = gallery.update(Message::ClearFilters);
        assert_eq!(gallery.matching_files(), [0, 1, 2]);
        let _ = gallery.update(Message::FilterPinned);
        assert!(gallery.matching_files().is_empty());
        let _ = gallery.update(Message::Pin);
        assert_eq!(gallery.matching_files(), [0]);
    }
    #[test]
    fn search_result_opens_details_and_history_is_bounded_and_deduplicated() {
        let mut gallery = Gallery {
            workspace_tab: WorkspaceTab::Files,
            ..Default::default()
        };
        assert_eq!(click_and_update(&mut gallery, 1160.0, "Search files"), 1);
        assert!(gallery.search_open);
        let _ = gallery.update(Message::Query("design".into()));
        // The file title also exists behind the overlay; target the visible result region.
        let messages = {
            let mut ui = visual_harness::Harness::new(
                gallery.view(),
                Size::new(1160.0, 844.0),
                gallery.theme(),
            );
            ui.at(0);
            let heading = ui.find("Search results");
            ui.move_to((heading.x + 40.0, heading.y + 70.0));
            ui.down();
            ui.up();
            ui.messages
        };
        assert!(matches!(messages.as_slice(), [Message::FileDetails(0)]));
        for message in messages {
            let _ = gallery.update(message);
        }
        assert!(gallery.details_open);
        assert!(!gallery.search_open);
        assert_eq!(gallery.details_index, 0);
        assert_eq!(gallery.recent_searches, ["design", "Checklist"]);
        for i in 0..8 {
            let _ = gallery.update(Message::Query(format!("Query {i}")));
            let _ = gallery.update(Message::SearchSubmit);
        }
        assert_eq!(gallery.recent_searches.len(), 5);
        assert_eq!(gallery.recent_searches[0], "Query 7");
    }
    #[test]
    fn gallery_filters_and_clear_action_emit_once_through_components() {
        let mut gallery = Gallery {
            workspace_tab: WorkspaceTab::Files,
            ..Default::default()
        };
        assert_eq!(click_and_update(&mut gallery, 1160.0, "Shared"), 1);
        assert_eq!(gallery.matching_files(), [1]);
        let _ = gallery.update(Message::Query("no match".into()));
        assert_eq!(click_and_update(&mut gallery, 1160.0, "Clear filters"), 1);
        assert_eq!(gallery.matching_files(), [0, 1, 2]);
        let _ = gallery.update(Message::SampleInput(false));
        assert!(!gallery.sample_input);
        let _ = gallery.update(Message::SampleInput(true));
        assert!(gallery.sample_input);
    }
    #[test]
    #[ignore = "reference suite; run with --ignored"]
    fn visual_references_search_gallery() {
        for dark in [false, true] {
            for width in [320.0, 390.0, 1160.0] {
                for case in ["files", "search", "query", "filtered", "empty"] {
                    let gallery = Gallery {
                        dark,
                        workspace_tab: WorkspaceTab::Files,
                        search_open: matches!(case, "search" | "query"),
                        query: if case == "query" {
                            "Design".into()
                        } else if case == "empty" {
                            "No match".into()
                        } else {
                            String::new()
                        },
                        filter_shared: case == "filtered",
                        file_age: if case == "filtered" {
                            (0.0, 14.0)
                        } else {
                            (0.0, 30.0)
                        },
                        ..Default::default()
                    };
                    let mut ui = visual_harness::Harness::new(
                        gallery.view(),
                        Size::new(width, 844.0),
                        gallery.theme(),
                    );
                    ui.at(0);
                    ui.operate(&mut iced::advanced::widget::operation::focusable::unfocus::<()>());
                    reference::check(
                        &format!(
                            "gallery-search/{}/{}/{case}",
                            if dark { "dark" } else { "light" },
                            width as u32
                        ),
                        &ui.frame(),
                    );
                }
            }
        }
    }
    #[test]
    fn schedule_validates_typed_input_and_saves_current_selection() {
        let mut gallery = Gallery {
            workspace_tab: WorkspaceTab::Schedule,
            schedule_input: true,
            ..Default::default()
        };
        let _ = gallery.update(Message::CalendarText("2026-02-30".into()));
        assert!(!gallery.schedule_valid());
        assert_eq!(click_and_update(&mut gallery, 1160.0, "Schedule review"), 0);
        let _ = gallery.update(Message::CalendarText("2026-09-21".into()));
        let _ = gallery.update(Message::ClockText("14:45".into()));
        assert_eq!(click_and_update(&mut gallery, 1160.0, "Schedule review"), 1);
        assert!(gallery.schedule_saved.contains("2026-09-21"));
        assert!(gallery.schedule_saved.contains("14:45"));
        let _ = gallery.update(Message::CalendarRange(true));
        assert!(!gallery.schedule_valid());
        let _ = gallery.update(Message::CalendarEnd("2026-09-20".into()));
        assert!(!gallery.schedule_valid());
        let _ = gallery.update(Message::CalendarEnd("2026-09-25".into()));
        assert!(gallery.schedule_valid());
    }
    #[test]
    #[ignore = "reference suite; run with --ignored"]
    fn visual_references_completion_gallery() {
        for dark in [false, true] {
            for width in [320.0, 390.0, 840.0, 1280.0] {
                for input in [false, true] {
                    let gallery = Gallery {
                        dark,
                        workspace_tab: WorkspaceTab::Schedule,
                        schedule_input: input,
                        ..Default::default()
                    };
                    let mut ui = visual_harness::Harness::new(
                        gallery.view(),
                        Size::new(width, 1000.0),
                        gallery.theme(),
                    );
                    ui.at(0);
                    reference::check(
                        &format!(
                            "gallery-completion/{}/{}/schedule-{}",
                            if dark { "dark" } else { "light" },
                            width as u32,
                            if input { "input" } else { "calendar" }
                        ),
                        &ui.frame(),
                    );
                }
            }
            let gallery = Gallery {
                dark,
                ..Default::default()
            };
            let mut ui = visual_harness::Harness::new(
                gallery.modern_components(),
                Size::new(1040.0, 1450.0),
                gallery.theme(),
            );
            ui.at(0);
            reference::check(
                &format!(
                    "gallery-completion/{}/components/more",
                    if dark { "dark" } else { "light" }
                ),
                &ui.frame(),
            );
        }
    }

    #[test]
    #[ignore = "reference suite; run with --ignored"]
    fn visual_references_loading_progress_gallery() {
        for dark in [false, true] {
            for width in [390., 1280.] {
                let gallery = Gallery {
                    dark,
                    loading_paused: true,
                    ..Default::default()
                };
                let mut ui = visual_harness::Harness::new(
                    gallery.loading_progress(),
                    Size::new(width, 780.),
                    gallery.theme(),
                );
                ui.at(0);
                reference::check(
                    &format!(
                        "gallery-loading-progress/{}/{}/paused",
                        if dark { "dark" } else { "light" },
                        width as u32
                    ),
                    &ui.frame(),
                );
            }
        }
    }
    #[test]
    fn full_screen_editor_commits_on_save_and_discards_cancelled_drafts() {
        let mut gallery = Gallery::default();
        let original = gallery.name.clone();
        let _ = gallery.update(Message::EditorOpen);
        let _ = gallery.update(Message::EditorName("Unsaved draft".into()));
        let _ = gallery.update(Message::EditorCancel);
        assert_eq!(gallery.name, original);
        assert!(gallery.editor_open && gallery.editor_discard_open);
        let _ = gallery.update(Message::EditorKeep);
        assert!(gallery.editor_open && !gallery.editor_discard_open);
        assert_eq!(gallery.editor_name, "Unsaved draft");
        let _ = gallery.update(Message::EditorCancel);
        let _ = gallery.update(Message::EditorDiscard(true));
        assert!(!gallery.editor_open && !gallery.editor_discard_open);
        let _ = gallery.update(Message::EditorOpen);
        assert_eq!(gallery.editor_name, original);
        let _ = gallery.update(Message::EditorName("  Desktop workspace  ".into()));
        let _ = gallery.update(Message::EditorNote("Review the next milestone".into()));
        assert_eq!(click_and_update(&mut gallery, 840.0, "Save"), 1);
        assert_eq!(gallery.name, "Desktop workspace");
        assert_eq!(gallery.note, "Review the next milestone");
        assert!(!gallery.editor_open);
        let _ = gallery.update(Message::EditorOpen);
        let _ = gallery.update(Message::EditorName(" ".into()));
        assert_eq!(click_and_update(&mut gallery, 390.0, "Save"), 0);
        assert!(gallery.editor_open);
    }
    #[test]
    fn modal_navigation_selects_a_page_and_closes() {
        let mut gallery = Gallery {
            navigation_modal_open: true,
            ..Default::default()
        };
        let messages = {
            let mut ui = visual_harness::Harness::new(
                gallery.view(),
                Size::new(840.0, 700.0),
                gallery.theme(),
            );
            ui.at(0);
            ui.frame();
            ui.at(500);
            ui.click("Settings");
            ui.messages.clone()
        };
        assert_eq!(messages.len(), 1);
        for message in messages {
            let _ = gallery.update(message);
        }
        assert_eq!(gallery.page, Page::Settings);
        assert!(!gallery.navigation_modal_open);
    }
    #[test]
    #[ignore = "reference suite; run with --ignored"]
    fn visual_references_desktop_gallery() {
        for dark in [false, true] {
            for width in [390.0, 1280.0] {
                let mut gallery = Gallery {
                    dark,
                    ..Default::default()
                };
                let _ = gallery.update(Message::EditorOpen);
                let mut ui = visual_harness::Harness::new(
                    gallery.view(),
                    Size::new(width, 780.0),
                    gallery.theme(),
                );
                ui.at(0);
                ui.operate(&mut iced::advanced::widget::operation::focusable::unfocus::<()>());
                reference::check(
                    &format!(
                        "gallery-desktop/{}/{}/full-screen-editor",
                        if dark { "dark" } else { "light" },
                        width as u32
                    ),
                    &ui.frame(),
                );
                let gallery = Gallery {
                    dark,
                    navigation_modal_open: true,
                    ..Default::default()
                };
                let mut ui = visual_harness::Harness::new(
                    gallery.view(),
                    Size::new(width, 780.0),
                    gallery.theme(),
                );
                ui.at(0);
                ui.frame();
                ui.at(500);
                reference::check(
                    &format!(
                        "gallery-desktop/{}/{}/modal-navigation",
                        if dark { "dark" } else { "light" },
                        width as u32
                    ),
                    &ui.frame(),
                );
                let gallery = Gallery {
                    dark,
                    ..Default::default()
                };
                let mut ui = visual_harness::Harness::new(
                    gallery.view(),
                    Size::new(width, 780.0),
                    gallery.theme(),
                );
                ui.click("More");
                ui.click("Navigation");
                ui.at(1000);
                reference::check(
                    &format!(
                        "gallery-desktop/{}/{}/submenu",
                        if dark { "dark" } else { "light" },
                        width as u32
                    ),
                    &ui.frame(),
                );
                let gallery = Gallery {
                    dark,
                    workspace_tab: WorkspaceTab::Schedule,
                    schedule_input: true,
                    ..Default::default()
                };
                let mut ui = visual_harness::Harness::new(
                    gallery.view(),
                    Size::new(width, 1000.0),
                    gallery.theme(),
                );
                ui.click("Open calendar");
                ui.at(1000);
                reference::check(
                    &format!(
                        "gallery-desktop/{}/{}/docked-calendar",
                        if dark { "dark" } else { "light" },
                        width as u32
                    ),
                    &ui.frame(),
                );
            }
        }
    }
    #[test]
    #[ignore = "reference suite; run with --ignored"]
    fn visual_references_desktop_completion_gallery() {
        for dark in [false, true] {
            for width in [390.0, 1280.0] {
                for name in ["nested-discard", "calendar-input", "compact-calendar"] {
                    let mut gallery = Gallery {
                        dark,
                        ..Default::default()
                    };
                    if name == "nested-discard" {
                        let _ = gallery.update(Message::EditorOpen);
                        let _ = gallery
                            .update(Message::EditorNote("An unsaved note worth keeping".into()));
                        let _ = gallery.update(Message::EditorCancel);
                    } else {
                        gallery.workspace_tab = WorkspaceTab::Schedule;
                        gallery.calendar_input = name == "calendar-input";
                    }
                    let mut ui = visual_harness::Harness::new(
                        gallery.view(),
                        Size::new(width, 1000.0),
                        gallery.theme(),
                    );
                    ui.at(0);
                    if name == "compact-calendar" {
                        ui.click("Open calendar");
                    }
                    ui.frame();
                    ui.at(600);
                    ui.operate(&mut iced::advanced::widget::operation::focusable::unfocus::<()>());
                    reference::check(
                        &format!(
                            "gallery-desktop-completion/{}/{}/{name}",
                            if dark { "dark" } else { "light" },
                            width as u32
                        ),
                        &ui.frame(),
                    );
                }
            }
        }
    }
    #[test]
    fn manual_clock_drafts_commit_only_after_apply_and_sync_formats() {
        let mut gallery = Gallery::default();
        let _ = gallery.update(Message::ClockInput);
        let original = gallery.clock_time;
        let _ = gallery.update(Message::ClockHour("12".into()));
        let _ = gallery.update(Message::ClockMinute("05".into()));
        let _ = gallery.update(Message::ClockPeriod(true));
        assert_eq!(gallery.clock_time, original);
        let _ = gallery.update(Message::Clock(iced_m3::Time::new(12, 5).unwrap()));
        assert!(!gallery.clock_input);
        assert_eq!(gallery.clock_text, "12:05");
        let _ = gallery.update(Message::Clock24(true));
        assert_eq!(gallery.clock_hour, "12");
        let _ = gallery.update(Message::Clock(iced_m3::Time::new(0, 5).unwrap()));
        let _ = gallery.update(Message::Clock24(false));
        assert_eq!(gallery.clock_hour, "12");
        assert!(!gallery.clock_pm);
    }
    #[test]
    #[ignore = "reference suite; run with --ignored"]
    fn visual_references_variants_gallery() {
        for dark in [false, true] {
            for width in [390.0, 1280.0] {
                for (name, disabled, dragged) in [
                    ("controls", false, false),
                    ("disabled", true, false),
                    ("dragged", false, true),
                ] {
                    let gallery = Gallery {
                        dark,
                        card_disabled: disabled,
                        card_dragged: dragged,
                        ..Default::default()
                    };
                    let mut ui = visual_harness::Harness::new(
                        container(gallery.desktop_variants()).padding(24),
                        Size::new(width, if width < 500.0 { 1450.0 } else { 1000.0 }),
                        gallery.theme(),
                    );
                    ui.at(0);
                    ui.frame();
                    ui.at(1750);
                    reference::check(
                        &format!(
                            "gallery-variants/{}/{}/{name}",
                            if dark { "dark" } else { "light" },
                            width as u32
                        ),
                        &ui.frame(),
                    );
                }
                let gallery = Gallery {
                    dark,
                    workspace_tab: WorkspaceTab::Schedule,
                    clock_input: true,
                    ..Default::default()
                };
                let mut ui = visual_harness::Harness::new(
                    gallery.view(),
                    Size::new(width, 1000.0),
                    gallery.theme(),
                );
                ui.at(0);
                ui.frame();
                ui.at(600);
                reference::check(
                    &format!(
                        "gallery-variants/{}/{}/manual-clock",
                        if dark { "dark" } else { "light" },
                        width as u32
                    ),
                    &ui.frame(),
                );
            }
        }
    }
    #[test]
    #[ignore = "reference suite; run with --ignored"]
    fn visual_references_fidelity_gallery() {
        for dark in [false, true] {
            for width in [400.0, 1100.0] {
                let gallery = Gallery {
                    dark,
                    ..Default::default()
                };
                let mut ui = visual_harness::Harness::new(
                    container(gallery.desktop_fidelity()).padding(24),
                    Size::new(width, if width < 500.0 { 1650.0 } else { 900.0 }),
                    gallery.theme(),
                );
                ui.at(0);
                ui.frame();
                ui.at(667);
                reference::check(
                    &format!(
                        "gallery-finish/{}/{}/details",
                        if dark { "dark" } else { "light" },
                        width as u32
                    ),
                    &ui.frame(),
                );
            }
        }
    }
    #[test]
    fn feedback_preview_keeps_controlled_values_when_modes_change() {
        let mut gallery = Gallery::default();
        let _ = gallery.update(Message::FeedbackRange((49.0, 51.0)));
        let _ = gallery.update(Message::FeedbackSelected(true));
        let _ = gallery.update(Message::FeedbackLoading(false));
        assert_eq!(gallery.feedback_range, (49.0, 51.0));
        assert!(gallery.feedback_selected);
        assert!(!gallery.feedback_loading);
        assert_eq!(gallery.preview_value, 0.4);
    }
    #[test]
    #[ignore = "reference suite; run with --ignored"]
    fn visual_references_baseline_gallery() {
        for dark in [false, true] {
            for width in [390.0, 1100.0] {
                let gallery = Gallery {
                    dark,
                    ..Default::default()
                };
                let mut ui = visual_harness::Harness::new(
                    container(gallery.baseline_completion()).padding(24),
                    Size::new(width, if width < 500.0 { 1000.0 } else { 650.0 }),
                    gallery.theme(),
                );
                ui.at(0);
                ui.frame();
                ui.at(450);
                reference::check(
                    &format!(
                        "gallery-baseline/{}/{}/feedback",
                        if dark { "dark" } else { "light" },
                        width as u32
                    ),
                    &ui.frame(),
                );
            }
        }
    }
}
