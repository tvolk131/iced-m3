// Shared token registry: compiled by the renderer and the metadata-only xtask.
doc_examples! {
    switch {
        title: "Switch", size: (320.0, 96.0), scale: 2, budget: 500_000,
        compare: true, animated: true, variants_size: (360.0, 484.0),
        caption: "Hover, hold, release to toggle, then toggle back. Expressive motion. The comparison below uses the same input sequence with Standard and Expressive springs.",
        view: fn view(checked: bool, icons: bool, disabled: bool) -> Element<'static, Message> {
            switch(checked)
                .label("Workspace notifications")
                .icons(icons)
                .disabled(disabled)
                .on_toggle(Message::Toggle)
                .into()
        },
        make: |s| view(s.active, false, false),
        variants: || widget::column![
            typography("Without icons", TypeScale::TitleSmall),
            view(false, false, false), view(true, false, false),
            typography("With icons", TypeScale::TitleSmall),
            view(false, true, false), view(true, true, false),
            typography("Disabled", TypeScale::TitleSmall),
            view(false, false, true), view(true, true, true),
        ].spacing(8).into(),
    }
    button_group {
        title: "Button group", size: (440.0, 104.0), scale: 1, budget: 1_500_000,
        compare: true, animated: true, variants_size: (440.0, 274.0),
        caption: "Hover and hold Review, then release. Standard and Expressive are compared on the same clock; both have spring motion, with different responses.",
        view: fn view(connected: bool) -> Element<'static, Message> {
            button_group([
                button("Create").on_press(Message::Action),
                button("Review").on_press(Message::Action),
                button("Share").on_press(Message::Action),
            ]).connected(connected).into()
        },
        make: |_| view(false),
        variants: || widget::column![
            typography("Spaced", TypeScale::TitleSmall), view(false),
            typography("Connected", TypeScale::TitleSmall), view(true),
        ].spacing(16).into(),
    }
    button {
        title: "Button", size: (200.0, 96.0), scale: 2, budget: 350_000,
        compare: false, animated: true, variants_size: (360.0, 424.0),
        caption: "Hover, hold, and release Save. Expressive shape and press feedback; release publishes an action.",
        view: fn view(label: &'static str, variant: ButtonVariant, disabled: bool) -> Element<'static, Message> {
            button(label).variant(variant).disabled(disabled).on_press(Message::Action).into()
        },
        make: |_| view("Save", ButtonVariant::Filled, false),
        variants: || widget::column![
            view("Filled", ButtonVariant::Filled, false), view("Tonal", ButtonVariant::Tonal, false),
            view("Elevated", ButtonVariant::Elevated, false), view("Outlined", ButtonVariant::Outlined, false),
            view("Text", ButtonVariant::Text, false), view("Disabled", ButtonVariant::Filled, true),
        ].spacing(12).into(),
    }
    icon_button {
        title: "Icon button", size: (128.0, 96.0), scale: 2, budget: 150_000,
        compare: false, animated: true, variants_size: (360.0, 424.0),
        caption: "Hold and release a filled toggle twice. Selection is application state; the SVG inherits the button's foreground.",
        view: fn view(selected: bool, variant: ButtonVariant, disabled: bool) -> Element<'static, Message> {
            let add = widget::svg::Handle::from_memory(br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M11 4h2v7h7v2h-7v7h-2v-7H4v-2h7z"/></svg>"#);
            icon_button(icon(add)).variant(variant).selected(selected)
                .disabled(disabled).on_press(Message::Toggle(!selected)).into()
        },
        make: |s| view(s.active, ButtonVariant::Filled, false),
        variants: || widget::column![
            typography("Standard / outlined", TypeScale::TitleSmall),
            widget::row![view(false, ButtonVariant::Text, false), view(false, ButtonVariant::Outlined, false)].spacing(16),
            typography("Filled: unselected / selected", TypeScale::TitleSmall),
            widget::row![view(false, ButtonVariant::Filled, false), view(true, ButtonVariant::Filled, false)].spacing(16),
            typography("Tonal / disabled", TypeScale::TitleSmall),
            widget::row![view(true, ButtonVariant::Tonal, false), view(true, ButtonVariant::Filled, true)].spacing(16),
        ].spacing(12).into(),
    }
    fab {
        title: "Floating action button", size: (176.0, 112.0), scale: 2, budget: 350_000,
        compare: false, animated: true, variants_size: (360.0, 424.0),
        caption: "Hover, hold, and release the primary action. The icon uses centered SVG artwork at the component's expected size.",
        view: fn view(extended: bool, size: FabSize, disabled: bool) -> Element<'static, Message> {
            let add = widget::svg::Handle::from_memory(br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M11 4h2v7h7v2h-7v7h-2v-7H4v-2h7z"/></svg>"#);
            let action = if extended {
                extended_fab(icon(add), "New note")
            } else {
                fab(icon(add).size(if size == FabSize::Large { 36 } else { 24 })).size(size)
            };
            action.disabled(disabled).on_press(Message::Action).into()
        },
        make: |_| view(false, FabSize::Regular, false),
        variants: || widget::column![
            typography("Small / regular / large", TypeScale::TitleSmall),
            widget::row![view(false, FabSize::Small, false), view(false, FabSize::Regular, false), view(false, FabSize::Large, false)].spacing(16),
            typography("Extended", TypeScale::TitleSmall), view(true, FabSize::Regular, false),
            typography("Disabled", TypeScale::TitleSmall), view(true, FabSize::Regular, true),
        ].spacing(12).into(),
    }
    checkbox {
        title: "Checkbox", size: (280.0, 96.0), scale: 2, budget: 125_000,
        compare: false, animated: true, variants_size: (360.0, 484.0),
        caption: "Hover, hold, release to check, then uncheck. Hover feedback stays visible while the centered press ripple expands.",
        view: fn view(checked: bool, indeterminate: bool, error: bool, disabled: bool) -> Element<'static, Message> {
            checkbox(checked).label("Workspace updates").indeterminate(indeterminate)
                .error(error).disabled(disabled).on_toggle(Message::Toggle).into()
        },
        make: |s| view(s.active, false, false, false),
        variants: || widget::column![
            view(false, false, false, false), view(true, false, false, false),
            view(false, true, false, false), view(false, false, true, false),
            view(false, false, false, true), view(true, false, false, true),
        ].spacing(8).into(),
    }
    radio {
        title: "Radio", size: (280.0, 152.0), scale: 2, budget: 200_000,
        compare: false, animated: true, variants_size: (360.0, 424.0),
        caption: "Select Team, then Personal, using actual pointer press/release events. A selected radio stays selected until another option is chosen.",
        view: fn view(selected: u8, disabled: bool) -> Element<'static, Message> {
            widget::column![
                radio("Personal", 0, Some(selected)).disabled(disabled).on_select(Message::Select),
                radio("Team", 1, Some(selected)).disabled(disabled).on_select(Message::Select),
            ].spacing(8).into()
        },
        make: |s| view(u8::from(s.active), false),
        variants: || widget::column![
            typography("Enabled", TypeScale::TitleSmall), view(0, false),
            typography("Disabled", TypeScale::TitleSmall), view(1, true),
        ].spacing(16).into(),
    }
    slider {
        title: "Slider", size: (360.0, 144.0), scale: 2, budget: 350_000,
        compare: false, animated: true, variants_size: (360.0, 464.0),
        caption: "Drag from 50 to 80, release, then drag back to 50. Values snap to steps of 10; each release commits once. The surrounding focus::scope clears focus on an outside click, hiding the label.",
        view: fn view(value: f32, discrete: bool, disabled: bool) -> Element<'static, Message> {
            let control = slider(0.0..=100.0, value).labeled(true).disabled(disabled)
                .on_change(Message::Value).on_release(Message::Action);
            if discrete { control.step(10.0).ticks(true).into() } else { control.into() }
        },
        make: |s| view(s.value, true, false),
        variants: || widget::column![
            typography("Continuous", TypeScale::TitleSmall), view(35.0, false, false),
            typography("Discrete", TypeScale::TitleSmall), view(70.0, true, false),
            typography("Disabled", TypeScale::TitleSmall), view(50.0, true, true),
        ].spacing(8).into(),
    }
    text_field {
        title: "Text field", size: (360.0, 144.0), scale: 2, budget: 500_000,
        compare: false, animated: true, variants_size: (360.0, 604.0),
        caption: "A programmatic value change raises the floating label, then clearing the value returns it. This demonstrates label motion rather than typing or caret blinking.",
        view: fn view(value: &str, error: bool, disabled: bool) -> Element<'_, Message> {
            let field = text_field("Workspace name", value)
                .on_input(Message::Input)
                .disabled(disabled)
                .supporting_text("Your team's shared space");
            if error { field.error("Enter a workspace name").into() } else { field.into() }
        },
        make: |s| view(if s.active { "Studio workspace" } else { "" }, false, false),
        variants: || widget::column![
            view("", false, false), view("Studio workspace", false, false),
            view("", true, false), view("Managed workspace", false, true),
        ].spacing(16).into(),
    }
    dialog {
        title: "Dialog", size: (480.0, 360.0), scale: 1, budget: 1_100_000,
        compare: false, animated: true, variants_size: (480.0, 424.0),
        caption: "The retained modal begins open, closes, and opens again. Body and fixed actions are separate. This larger preview uses 1× output at 60 fps.",
        view: fn view(open: bool) -> Element<'static, Message> {
            dialog::modal(
                typography("Workspace", TypeScale::HeadlineSmall),
                dialog(widget::column![
                    typography("Workspace settings", TypeScale::HeadlineSmall),
                    typography("Save these changes?", TypeScale::BodyMedium),
                    text_field("Workspace name", "Studio workspace").on_input(Message::Input),
                ].spacing(16))
                .width(380.0)
                .actions(widget::row![
                    button("Cancel").on_press(Message::Dismiss),
                    button("Save").on_press(Message::Action),
                ].spacing(8))
                .on_dismiss(Message::Dismiss),
                open,
            )
        },
        make: |s| view(s.active),
        variants: || view(true),
    }
    loading_indicator {
        title: "Loading indicator", size: (200.0, 112.0), scale: 2, budget: 350_000,
        compare: false, animated: true, variants_size: (360.0, 304.0),
        caption: "A three-second excerpt of the Expressive shape-morphing indicator. This preview restarts at the loop boundary; it is not a complete natural motion cycle.",
        view: fn view(contained: bool) -> Element<'static, Message> {
            loading_indicator().contained(contained).into()
        },
        make: |_| view(false),
        variants: || widget::column![
            typography("Uncontained", TypeScale::TitleSmall), view(false),
            typography("Contained", TypeScale::TitleSmall), view(true),
        ].spacing(16).into(),
    }
    typography {
        title: "Typography", size: (400.0, 180.0), scale: 1, budget: 100_000,
        compare: false, animated: false, variants_size: (400.0, 740.0),
        caption: "Static type specimens rendered with the bundled Roboto fonts. Type roles control size, weight, and line height.",
        view: fn view() -> Element<'static, Message> {
            widget::column![typography("Your workspace", TypeScale::HeadlineLarge),
                typography("A place for your next idea", TypeScale::BodyLarge),
                typography("SHARED WITH YOUR TEAM", TypeScale::LabelSmall)].spacing(12).into()
        },
        make: |_| view(),
        variants: || widget::Column::with_children([
            ("Display large", TypeScale::DisplayLarge), ("Display medium", TypeScale::DisplayMedium),
            ("Display small", TypeScale::DisplaySmall), ("Headline large", TypeScale::HeadlineLarge),
            ("Headline medium", TypeScale::HeadlineMedium), ("Headline small", TypeScale::HeadlineSmall),
            ("Title large", TypeScale::TitleLarge), ("Title medium", TypeScale::TitleMedium),
            ("Title small", TypeScale::TitleSmall), ("Body large", TypeScale::BodyLarge),
            ("Body medium", TypeScale::BodyMedium), ("Body small", TypeScale::BodySmall),
            ("Label large", TypeScale::LabelLarge), ("Label medium", TypeScale::LabelMedium),
            ("Label small", TypeScale::LabelSmall),
        ].map(|(label, role)| typography(label, role).into())).spacing(12).into(),
    }
    surface {
        title: "Surface", size: (320.0, 120.0), scale: 2, budget: 100_000,
        compare: false, animated: false, variants_size: (360.0, 360.0),
        caption: "A static surface containing ordinary iced-compatible content. The variants use the same content with three surface treatments.",
        view: fn view(variant: SurfaceVariant) -> Element<'static, Message> {
            surface(typography("Project overview", TypeScale::TitleMedium)).variant(variant).padding(20).into()
        },
        make: |_| view(SurfaceVariant::Filled),
        variants: || widget::column![view(SurfaceVariant::Filled), view(SurfaceVariant::Outlined), view(SurfaceVariant::Elevated)].spacing(24).into(),
    }
    card {
        title: "Card", size: (300.0, 140.0), scale: 1, budget: 400_000,
        compare: false, animated: true, variants_size: (360.0, 490.0),
        caption: "Hover, hold, and release an interactive card. Card content is composed from real typography widgets.",
        view: fn view(variant: SurfaceVariant, disabled: bool) -> Element<'static, Message> {
            card(widget::column![typography("Design notes", TypeScale::TitleMedium),
                typography("Updated just now", TypeScale::BodyMedium)].spacing(8))
                .variant(variant).disabled(disabled).on_press(Message::Action).into()
        },
        make: |_| view(SurfaceVariant::Filled, false),
        variants: || widget::column![view(SurfaceVariant::Filled, false), view(SurfaceVariant::Outlined, false),
            view(SurfaceVariant::Elevated, false), view(SurfaceVariant::Filled, true)].spacing(16).into(),
    }
    divider {
        title: "Divider", size: (280.0, 150.0), scale: 2, budget: 100_000,
        compare: false, animated: false, variants_size: (320.0, 260.0),
        caption: "Static horizontal and vertical separators inherit the theme's outline-variant color.",
        view: fn view() -> Element<'static, Message> {
            widget::column![typography("Workspace", TypeScale::TitleMedium), divider(),
                widget::row![typography("Personal", TypeScale::BodyMedium), vertical_divider(),
                    typography("Team", TypeScale::BodyMedium)].spacing(16).height(32)].spacing(16).into()
        },
        make: |_| view(), variants: || view(),
    }
    filter_chip {
        title: "Chips", size: (200.0, 96.0), scale: 2, budget: 300_000,
        compare: false, animated: true, variants_size: (360.0, 400.0),
        caption: "Toggle the filter twice. Its check slot is reserved so neighboring chips do not move during selection.",
        view: fn view(selected: bool, disabled: bool) -> Element<'static, Message> {
            filter_chip("Unread", selected).disabled(disabled).on_press(Message::Toggle(!selected)).into()
        },
        make: |s| view(s.active, false),
        variants: || widget::column![assist_chip("Assist").on_press(Message::Action),
            suggestion_chip("Suggestion").on_press(Message::Action), view(true, false),
            input_chip("Teammate").on_press(Message::Action).on_remove(Message::Dismiss), view(false, true)].spacing(16).into(),
    }
    badge {
        title: "Badges", size: (240.0, 96.0), scale: 2, budget: 100_000,
        compare: false, animated: false, variants_size: (320.0, 230.0),
        caption: "Static unread counts and a dot indicator. Large counts display as 99+.",
        view: fn view() -> Element<'static, Message> {
            widget::row![badge_dot(), badge(3), badge(120)].spacing(24).align_y(iced::Alignment::Center).into()
        },
        make: |_| view(),
        variants: || widget::column![view(), badged(button("Inbox").on_press(Message::Action), Some(8)),
            badged(button("Updates").on_press(Message::Action), None)].spacing(20).into(),
    }
    tabs {
        title: "Tabs", size: (360.0, 120.0), scale: 1, budget: 400_000,
        compare: false, animated: true, variants_size: (360.0, 320.0),
        caption: "Select Activity, then Overview. The indicator follows the controlled selection with the component's own motion.",
        view: fn view(selected: u8, variant: TabVariant) -> Element<'static, Message> {
            tabs([Tab::new(0, "Overview"), Tab::new(1, "Activity").badge(3), Tab::new(2, "Archive").disabled(true)], Some(selected))
                .variant(variant).on_select(Message::Select).into()
        },
        make: |s| view(u8::from(s.active), TabVariant::Primary),
        variants: || widget::column![typography("Primary", TypeScale::TitleSmall), view(0, TabVariant::Primary),
            typography("Secondary", TypeScale::TitleSmall), view(1, TabVariant::Secondary)].spacing(20).into(),
    }
    list_item {
        title: "Lists", size: (340.0, 152.0), scale: 1, budget: 350_000,
        compare: false, animated: true, variants_size: (360.0, 410.0),
        caption: "Click to select and clear the row. Leading and trailing content can be composed independently.",
        view: fn view(selected: bool, disabled: bool) -> Element<'static, Message> {
            list_item("Design notes").supporting_text("Shared with the team").overline("WORKSPACE")
                .selected(selected).disabled(disabled).on_press(Message::Toggle(!selected)).into()
        },
        make: |s| view(s.active, false),
        variants: || list([list_item("One-line item").into(),
            list_item("Two-line item").supporting_text("Supporting text").trailing(badge(2)).into(),
            view(true, false), view(false, true)]).into(),
    }
    menu {
        title: "Menu", size: (320.0, 260.0), scale: 1, budget: 600_000,
        compare: false, animated: true, variants_size: (340.0, 340.0),
        caption: "Open Actions, choose Duplicate, then let the popup close. Shortcut text is a visual hint; the application owns key bindings.",
        view: fn view() -> Element<'static, Message> {
            menu("Actions", [MenuItem::new("Duplicate", Message::Action).shortcut("Ctrl+D"),
                MenuItem::new("Share", Message::Action), MenuItem::separator(),
                MenuItem::new("Delete", Message::Dismiss).disabled(true)]).into()
        },
        make: |_| view(), variants: || view(),
    }
    select {
        title: "Select", size: (340.0, 300.0), scale: 1, budget: 700_000,
        compare: false, animated: true, variants_size: (360.0, 400.0),
        caption: "Open the field to choose Editor, then return to Viewer. The value changes only through the selection callback.",
        view: fn view(selected: u8, filled: bool, disabled: bool) -> Element<'static, Message> {
            select("Access", [SelectOption::new(0, "Viewer"), SelectOption::new(1, "Editor"),
                SelectOption::new(2, "Owner").disabled(true)], Some(selected))
                .variant(if filled { TextFieldVariant::Filled } else { TextFieldVariant::Outlined })
                .disabled(disabled).on_select(Message::Select).into()
        },
        make: |s| view(u8::from(s.active), false, false),
        variants: || widget::column![view(0, false, false), view(1, true, false), view(0, false, true)].spacing(24).into(),
    }
    tooltip {
        title: "Tooltip", size: (300.0, 148.0), scale: 1, budget: 250_000,
        compare: false, animated: true, variants_size: (320.0, 190.0),
        caption: "Hover over Help for the default 500ms delay, then move away to dismiss the hint.",
        view: fn view() -> Element<'static, Message> {
            tooltip(button("Help").on_press(Message::Action), "Learn about workspaces").placement(tooltip::Placement::Bottom).into()
        },
        make: |_| view(), variants: || view(),
    }
    rich_tooltip {
        title: "Rich tooltip", size: (340.0, 260.0), scale: 1, budget: 600_000,
        compare: false, animated: true, variants_size: (360.0, 340.0),
        caption: "Click About to open a rich hint, then click outside. Rich hints support a title, descriptive body, and optional actions.",
        view: fn view() -> Element<'static, Message> {
            rich_tooltip(typography("About", TypeScale::LabelLarge), "Shared spaces",
                typography("Keep notes and decisions together.", TypeScale::BodyMedium)).into()
        },
        make: |_| view(), variants: || view(),
    }
    snackbar {
        title: "Snackbar", size: (360.0, 190.0), scale: 1, budget: 700_000,
        compare: false, animated: true, variants_size: (360.0, 240.0),
        caption: "Application state shows and hides a retained notice. Passing visible(false) keeps its exit animation; removing the notice immediately would skip it.",
        view: fn view(visible: bool) -> Element<'static, Message> {
            snackbar::host(widget::container(typography("Workspace saved", TypeScale::TitleMedium))
                .width(iced::Length::Fill).height(iced::Length::Fill),
                Some(snackbar("Changes saved").action("Undo", Message::Action).persistent().visible(visible)))
        },
        make: |s| view(s.active), variants: || view(true),
    }
    app_bar {
        title: "App bar", size: (400.0, 220.0), scale: 1, budget: 600_000,
        compare: false, animated: true, variants_size: (400.0, 540.0),
        caption: "The application drives collapse from 0 to 1 and back, as it would from a scroll offset. This is controlled layout motion, not an internal scroll animation.",
        view: fn view(variant: AppBarVariant, collapse: f32) -> Element<'static, Message> {
            app_bar("Workspace").variant(variant).collapse(collapse).scrolled(collapse > 0.0)
                .action(button("Edit").variant(ButtonVariant::Text).on_press(Message::Action)).into()
        },
        make: |s| view(AppBarVariant::Large, s.progress),
        variants: || widget::column![view(AppBarVariant::Small, 0.0), view(AppBarVariant::CenterAligned, 0.0),
            view(AppBarVariant::Medium, 0.0), view(AppBarVariant::Large, 0.0)].spacing(16).into(),
    }
    navigation_bar {
        title: "Navigation bar", size: (400.0, 140.0), scale: 1, budget: 500_000,
        compare: false, animated: true, variants_size: (400.0, 320.0),
        caption: "Select Notes, then Home. Destinations publish a value; the application owns routing and page content.",
        view: fn view(selected: u8, disabled: bool) -> Element<'static, Message> {
            let symbol = || icon(widget::svg::Handle::from_memory(br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M4 4h16v16H4z"/></svg>"#));
            navigation_bar([NavigationItem::new(0, "Home", symbol()), NavigationItem::new(1, "Notes", symbol()).badge(3),
                NavigationItem::new(2, "Archive", symbol()).disabled(true)], Some(selected)).disabled(disabled).on_select(Message::Select).into()
        },
        make: |s| view(u8::from(s.active), false),
        variants: || widget::column![view(0, false), view(1, true)].spacing(24).into(),
    }
    navigation_rail {
        title: "Navigation rail", size: (340.0, 280.0), scale: 1, budget: 600_000,
        compare: false, animated: true, variants_size: (400.0, 370.0),
        caption: "Application state expands the rail from 80 to 280 logical pixels and collapses it again. Selection remains stable during the transition.",
        view: fn view(expanded: bool) -> Element<'static, Message> {
            let symbol = || icon(widget::svg::Handle::from_memory(br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M4 4h16v16H4z"/></svg>"#));
            navigation_rail([NavigationItem::new(0, "Home", symbol()), NavigationItem::new(1, "Notes", symbol()).badge(3),
                NavigationItem::new(2, "Archive", symbol()).disabled(true)], Some(0)).expanded(expanded).on_select(Message::Select).into()
        },
        make: |s| view(s.active), variants: || widget::row![view(false), view(true)].spacing(8).into(),
    }
    segmented_buttons {
        title: "Segmented buttons", size: (360.0, 100.0), scale: 1, budget: 450_000,
        compare: false, animated: true, variants_size: (360.0, 240.0),
        caption: "Change a single selection to Week and back to Day. The appearance grid also shows multiple selection and a disabled segment.",
        view: fn view(selection: SegmentSelection<u8>) -> Element<'static, Message> {
            segmented_buttons([Segment::new(0, "Day"), Segment::new(1, "Week"), Segment::new(2, "Month").disabled(true)], selection)
                .on_change(Message::Segments).into()
        },
        make: |s| view(SegmentSelection::Single(Some(u8::from(s.active)))),
        variants: || widget::column![view(SegmentSelection::Single(Some(0))), view(SegmentSelection::Multiple(vec![0, 1]))].spacing(24).into(),
    }
    range_slider {
        title: "Range slider", size: (360.0, 144.0), scale: 1, budget: 400_000,
        compare: false, animated: true, variants_size: (360.0, 350.0),
        caption: "Drag the upper handle from 50 to 80 and back, keeping the lower value at 20. The surrounding focus::scope clears focus on an outside click.",
        view: fn view(upper: f32, disabled: bool) -> Element<'static, Message> {
            range_slider(0.0..=100.0, (20.0, upper)).step(10.0).ticks(true).labeled(true)
                .disabled(disabled).on_change(Message::Range).on_release(Message::Action).into()
        },
        make: |s| view(s.value, false), variants: || widget::column![view(70.0, false), view(80.0, true)].spacing(24).into(),
    }
    linear_progress {
        title: "Linear progress", size: (320.0, 100.0), scale: 1, budget: 400_000,
        compare: false, animated: true, variants_size: (360.0, 290.0),
        caption: "A three-second excerpt of indeterminate progress. The recording restarts; it does not imply a three-second component cycle.",
        view: fn view(wavy: bool, indeterminate: bool) -> Element<'static, Message> {
            linear_progress(0.4).buffer(0.7).wavy(wavy).indeterminate(indeterminate).into()
        },
        make: |_| view(false, true),
        variants: || widget::column![typography("Buffered", TypeScale::TitleSmall), view(false, false),
            typography("Wavy", TypeScale::TitleSmall), view(true, false),
            typography("Indeterminate", TypeScale::TitleSmall), view(false, true)].spacing(20).into(),
    }
    circular_progress {
        title: "Circular progress", size: (128.0, 110.0), scale: 2, budget: 450_000,
        compare: false, animated: true, variants_size: (320.0, 300.0),
        caption: "A three-second excerpt of an indeterminate arc growing and contracting. The recording restarts at its boundary.",
        view: fn view(wavy: bool, indeterminate: bool) -> Element<'static, Message> {
            circular_progress(0.65).wavy(wavy).indeterminate(indeterminate).into()
        },
        make: |_| view(false, true), variants: || widget::column![
            typography("Determinate / loading", TypeScale::TitleSmall), widget::row![view(false, false), view(false, true)].spacing(32),
            typography("Wavy", TypeScale::TitleSmall), widget::row![view(true, false), view(true, true)].spacing(32)].spacing(24).into(),
    }
    date_picker {
        title: "Date picker", size: (408.0, 720.0), scale: 1, budget: 650_000,
        compare: false, animated: true, variants_size: (408.0, 800.0),
        caption: "Choose September 16, then return to September 15. The month is fixed for repeatable documentation; application state owns the selected date.",
        view: fn view(day: u8, range: bool) -> Element<'static, Message> {
            let date = Date::new(2026, 9, day).unwrap();
            date_picker(date, if range { DateSelection::Range { start: Some(date), end: Some(Date::new(2026, 9, 19).unwrap()) } }
                else { DateSelection::Single(Some(date)) }).on_select(Message::Date).on_confirm(Message::Date).on_cancel(Message::Dismiss).into()
        },
        make: |s| view(if s.active { 16 } else { 15 }, false), variants: || view(15, true),
    }
    time_picker {
        title: "Time picker", size: (408.0, 600.0), scale: 1, budget: 650_000,
        compare: false, animated: true, variants_size: (380.0, 360.0),
        caption: "Change 10:30 AM to PM and back through the period buttons. The appearance grid shows numeric entry instead of the clock dial.",
        view: fn view(pm: bool, input: bool) -> Element<'static, Message> {
            let picker = time_picker(Time::new(if pm { 22 } else { 10 }, 30).unwrap(), TimePart::Hour).input_mode(input)
                .on_change(Message::Time).on_part(Message::Part).on_confirm(Message::Time).on_cancel(Message::Dismiss);
            if input { picker.hour_input("10", Message::Input).minute_input("30", Message::Input).input_period(pm, Message::Toggle).into() }
            else { picker.into() }
        },
        make: |s| view(s.active, false), variants: || view(false, true),
    }
    carousel {
        title: "Carousel", size: (400.0, 200.0), scale: 1, budget: 600_000,
        compare: false, animated: true, variants_size: (400.0, 480.0),
        caption: "Application state advances the selected item and returns to the first. The component performs its own snapping animation and crops the SVG artwork to its keylines.",
        view: fn view(selected: usize, variant: CarouselVariant) -> Element<'static, Message> {
            let tiles = ["#6750a4", "#625b71", "#7d5260"].map(|color| {
                let artwork = format!(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 180 128"><rect width="180" height="128" fill="{color}"/><circle cx="90" cy="64" r="38" fill="white" fill-opacity="0.35"/><circle cx="90" cy="64" r="18" fill="white" fill-opacity="0.7"/></svg>"#);
                widget::svg(widget::svg::Handle::from_memory(artwork.into_bytes())).width(iced::Length::Fill).height(iced::Length::Fill).content_fit(iced::ContentFit::Cover).into()
            });
            carousel(tiles, selected)
                .variant(variant).height(128.0).item_width(180.0).on_select(Message::Index).into()
        },
        make: |s| view(usize::from(s.active), CarouselVariant::MultiBrowse),
        variants: || widget::column![view(0, CarouselVariant::MultiBrowse), view(1, CarouselVariant::Hero), view(1, CarouselVariant::Uncontained)].spacing(16).into(),
    }
    side_sheet {
        title: "Sheets", size: (400.0, 280.0), scale: 1, budget: 800_000,
        compare: false, animated: true, variants_size: (400.0, 340.0),
        caption: "Application state opens and closes a retained modal side sheet. The appearance grid shows the bottom-sheet composition.",
        view: fn view(open: bool, bottom: bool) -> Element<'static, Message> {
            let body = widget::column![typography("Details", TypeScale::TitleLarge),
                typography("Everything about this workspace.", TypeScale::BodyMedium)].spacing(16);
            let panel = if bottom { bottom_sheet(body).height(160.0) } else { side_sheet(body).width(240.0) };
            sheet::host(typography("Workspace", TypeScale::HeadlineSmall), panel.modal().open(open).on_dismiss(Message::Dismiss))
        },
        make: |s| view(s.active, false), variants: || view(true, true),
    }
    search_bar {
        title: "Search", size: (400.0, 300.0), scale: 1, budget: 800_000,
        compare: false, animated: true, variants_size: (400.0, 260.0),
        caption: "Application state opens and closes search results for a fixed query. Query editing and result selection remain application-owned. The capture leaves the input unfocused so native caret blinking does not affect this motion preview.",
        view: fn view(open: bool) -> Element<'static, Message> {
            search_bar("Search workspace", "design").open(open).on_open(Message::Toggle(true)).on_close(Message::Toggle(false))
                .on_input(Message::Input).on_submit(Message::Action).height(200.0)
                .results(list([list_item("Design notes").supporting_text("Updated today").on_press(Message::Action).into(),
                    list_item("Design decisions").on_press(Message::Action).into()])).into()
        },
        make: |s| view(s.active), variants: || view(false),
    }
    split_button {
        title: "Split button", size: (340.0, 240.0), scale: 1, budget: 500_000,
        compare: false, animated: true, variants_size: (360.0, 190.0),
        caption: "Hover, hold, and release the primary action. The adjacent arrow owns the separate menu of alternate actions.",
        view: fn view() -> Element<'static, Message> {
            split_button("Create", Message::Action, [MenuItem::new("From template", Message::Action), MenuItem::new("Blank note", Message::Action)])
        },
        make: |_| view(), variants: || view(),
    }
    toolbar {
        title: "Toolbar", size: (360.0, 150.0), scale: 1, budget: 500_000,
        compare: false, animated: true, variants_size: (360.0, 450.0),
        caption: "Hold and release Bold in a floating toolbar. The toolbar groups focus without taking editing keys away from editable children.",
        view: fn view(floating: bool, vertical: bool) -> Element<'static, Message> {
            toolbar([button("Bold").variant(ButtonVariant::Text).on_press(Message::Action).into(),
                button("Italic").variant(ButtonVariant::Text).on_press(Message::Action).into()]).floating(floating).vertical(vertical).into()
        },
        make: |_| view(true, false), variants: || widget::column![view(false, false), view(true, false), view(true, true)].spacing(24).into(),
    }
    fab_menu {
        title: "FAB menu", size: (300.0, 300.0), scale: 1, budget: 700_000,
        compare: false, animated: true, variants_size: (320.0, 360.0),
        caption: "Application state expands and collapses a retained FAB menu. Keep it mounted while closed so its exit can finish.",
        view: fn view(open: bool) -> Element<'static, Message> {
            let add = || icon(widget::svg::Handle::from_memory(br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M11 4h2v7h7v2h-7v7h-2v-7H4v-2h7z"/></svg>"#));
            fab_menu(add(), open, Message::Toggle(!open), [FabMenuItem::new("New note", add(), Message::Action), FabMenuItem::new("New folder", add(), Message::Action)])
        },
        make: |s| view(s.active), variants: || view(true),
    }
    full_screen_dialog {
        title: "Full-screen dialog", size: (440.0, 300.0), scale: 1, budget: 100_000,
        compare: false, animated: false, variants_size: (440.0, 364.0),
        caption: "A static view of the full-screen dialog's app bar, dismiss action, body, and save action. Retain it in dialog::modal for animated open/close transitions.",
        view: fn view() -> Element<'static, Message> {
            dialog::modal(widget::space(), full_screen_dialog("Edit workspace", widget::column![
                text_field("Name", "Studio").on_input(Message::Input),
                typography("Changes stay in this workspace.", TypeScale::BodyMedium)].spacing(16))
                .on_dismiss(Message::Dismiss).action(button("Save").variant(ButtonVariant::Text).on_press(Message::Action)).into(), true)
        },
        make: |_| view(), variants: || view(),
    }
}
