use super::{harness::Harness, input::key, reference::Image};
use crate::{Element, ShapeScale, ShapeScheme, Theme, TypeScale, Typography};
use iced::{Color, Rectangle, Size, font::Weight, keyboard::key::Named, widget};

fn ui<Message: 'static>(
    content: impl Into<Element<'static, Message>>,
    theme: Theme,
) -> Harness<'static, Message> {
    Harness::configured(
        crate::focus::scope(widget::container(content).padding(24)),
        Size::new(640.0, 480.0),
        theme,
    )
}

fn pixel(image: &Image, x: u32, y: u32) -> &[u8] {
    let offset = ((y * 2 * image.width + x * 2) * 4) as usize;
    &image.pixels[offset..offset + 4]
}

fn region(image: &Image, bounds: Rectangle) -> Image {
    image.crop(
        (bounds.x * 2.0).round() as u32,
        (bounds.y * 2.0).round() as u32,
        (bounds.width * 2.0).round() as u32,
        (bounds.height * 2.0).round() as u32,
    )
}

#[test]
fn select_popup_tracks_the_scaled_field_without_including_supporting_text() {
    for variant in [
        crate::TextFieldVariant::Outlined,
        crate::TextFieldVariant::Filled,
    ] {
        let mut theme = Theme::light().reduced_motion(true);
        theme.typography = theme.typography.scaled(1.5);
        let mut view = ui(
            theme
                .widgets()
                .select("Choice", [crate::SelectOption::new(1, "Option")], None)
                .variant(variant)
                .supporting_text("Supporting text stays outside the anchor")
                .on_select(|_| ()),
            theme,
        );
        view.frame();
        let field = view.find("Choice");
        view.click("Choice");
        view.at(0);
        view.at(300);
        view.frame();
        assert_eq!(view.find("Option").y, field.y + field.height + 24.0);
    }
}

#[test]
fn enlarged_rich_popup_corners_do_not_clip_the_padded_title_or_body() {
    let mut original = Theme::light().reduced_motion(true);
    original.shapes.medium = 16.0;
    let mut rounder = original.clone();
    rounder.shapes.medium = 28.0;
    let popup = || {
        crate::rich_tooltip(
            widget::text("Trigger"),
            "Readable title",
            crate::typography("Readable body", TypeScale::BodyMedium),
        )
    };
    let mut before = ui::<()>(popup(), original);
    let mut after = ui::<()>(popup(), rounder);
    for view in [&mut before, &mut after] {
        view.frame();
        view.click("Trigger");
        view.at(0);
        view.at(300);
        view.frame();
    }
    for label in ["Readable title", "Readable body"] {
        let bounds = before.find(label);
        assert_eq!(bounds, after.find(label));
        assert!(region(&before.frame(), bounds) == region(&after.frame(), bounds));
    }
}

#[test]
fn chip_families_grow_to_fit_their_captured_label_metrics() {
    for line_height in [25.0, 40.0] {
        let mut theme = Theme::light();
        theme.typography[TypeScale::LabelLarge].line_height = line_height;
        let widgets = theme.widgets();
        let chips: Vec<Element<'static, ()>> = vec![
            widgets.chip("Chip", true).on_press(()).into(),
            widgets.assist_chip("Chip").on_press(()).into(),
            widgets.suggestion_chip("Chip").on_press(()).into(),
            widgets.filter_chip("Chip", true).on_press(()).into(),
            widgets.input_chip("Chip").on_remove(()).into(),
        ];
        for chip in chips {
            let mut view = ui(chip, theme.clone());
            assert_eq!(view.find("Chip").height, line_height);
        }
    }
}

#[test]
fn field_label_uses_each_endpoint_font_and_respects_explicit_overrides() {
    let mut theme = Theme::light().reduced_motion(true);
    theme.typography[TypeScale::BodyLarge].font = iced::Font::MONOSPACE;
    let widgets = theme.widgets();
    let field = || widgets.text_field("Role font", "").on_input(|_| ());
    let mut themed = ui(field(), theme.clone());
    let mut body = ui(field().font(iced::Font::MONOSPACE), theme.clone());
    let mut small = ui(field().font(crate::fonts::REGULAR), theme);
    assert!(themed.frame() == body.frame());
    assert!(themed.frame() != small.frame());
    for view in [&mut themed, &mut small] {
        view.tap_key(key(Named::Tab));
        view.at(0);
        view.at(300);
    }
    assert!(themed.frame() == small.frame());
}

#[test]
fn period_group_outline_and_fill_share_the_half_button_radius_limit() {
    for numeric in [false, true] {
        let mut original = Theme::light().reduced_motion(true);
        original.shapes.small = 20.0;
        let mut rounder = original.clone();
        rounder.shapes.small = 24.0;
        let picker = |theme: &Theme| {
            theme
                .widgets()
                .time_picker(crate::Time::new(3, 15).unwrap(), crate::TimePart::Hour)
                .input_mode(numeric)
                .on_change(|_| ())
        };
        let mut before = ui(picker(&original), original);
        let mut after = ui(picker(&rounder), rounder);
        let am = before.find("AM");
        let bounds = Rectangle {
            x: am.center_x() - 26.0,
            y: am.center_y() - 20.0,
            width: 52.0,
            height: 80.0,
        };
        assert!(region(&before.frame(), bounds) == region(&after.frame(), bounds));
    }
}

#[test]
fn full_screen_dialog_factory_propagates_its_builtin_title_recipe() {
    let mut theme = Theme::light().reduced_motion(true);
    let dialog = |widgets: crate::Widgets| {
        crate::dialog::modal(
            widget::space()
                .width(iced::Length::Fill)
                .height(iced::Length::Fill),
            widgets
                .full_screen_dialog("Dialog title", widget::space())
                .into(),
            true,
        )
    };
    let mut before = ui::<()>(dialog(theme.widgets()), theme.clone());
    theme.typography[TypeScale::TitleLarge].font.weight = Weight::Bold;
    let mut after = ui::<()>(dialog(theme.widgets()), theme);
    for view in [&mut before, &mut after] {
        view.frame();
        view.at(0);
        view.at(300);
    }
    assert!(before.frame() != after.frame());
}

#[test]
fn typography_defaults_preserve_baseline_metrics_and_reference_emphasized_weights() {
    let typography = Typography::default();
    let expected = [
        (57.0, 64.0, Weight::Normal, Weight::Medium, -0.25),
        (45.0, 52.0, Weight::Normal, Weight::Medium, 0.0),
        (36.0, 44.0, Weight::Normal, Weight::Medium, 0.0),
        (32.0, 40.0, Weight::Normal, Weight::Medium, 0.0),
        (28.0, 36.0, Weight::Normal, Weight::Medium, 0.0),
        (24.0, 32.0, Weight::Normal, Weight::Medium, 0.0),
        (22.0, 28.0, Weight::Normal, Weight::Medium, 0.0),
        (16.0, 24.0, Weight::Medium, Weight::Bold, 0.15),
        (14.0, 20.0, Weight::Medium, Weight::Bold, 0.1),
        (16.0, 24.0, Weight::Normal, Weight::Medium, 0.5),
        (14.0, 20.0, Weight::Normal, Weight::Medium, 0.25),
        (12.0, 16.0, Weight::Normal, Weight::Medium, 0.4),
        (14.0, 20.0, Weight::Medium, Weight::Bold, 0.1),
        (12.0, 16.0, Weight::Medium, Weight::Bold, 0.5),
        (11.0, 16.0, Weight::Medium, Weight::Bold, 0.5),
    ];
    for (role, (size, line, weight, emphasized, tracking)) in
        TypeScale::ALL.into_iter().zip(expected)
    {
        let style = typography.style(role);
        assert_eq!(
            (
                style.size,
                style.line_height,
                style.font.weight,
                style.tracking
            ),
            (size, line, weight, tracking)
        );
        let strong = typography.emphasized(role);
        assert_eq!(
            (strong.size, strong.line_height, strong.font.weight),
            (size, line, emphasized)
        );
    }
}

#[test]
fn complete_shape_scale_and_asymmetric_edges_match_pinned_tokens() {
    let shapes = ShapeScheme::default();
    for (role, radius) in [
        (ShapeScale::None, 0.0),
        (ShapeScale::ExtraSmall, 4.0),
        (ShapeScale::Small, 8.0),
        (ShapeScale::Medium, 12.0),
        (ShapeScale::Large, 16.0),
        (ShapeScale::LargeIncreased, 20.0),
        (ShapeScale::ExtraLarge, 28.0),
        (ShapeScale::ExtraLargeIncreased, 32.0),
        (ShapeScale::ExtraExtraLarge, 48.0),
    ] {
        assert_eq!(shapes.radius(role), radius);
    }
    assert_eq!(
        shapes.top(ShapeScale::ExtraLarge),
        iced::border::Radius {
            top_left: 28.0,
            top_right: 28.0,
            bottom_left: 0.0,
            bottom_right: 0.0
        }
    );
    assert_eq!(
        shapes.left(ShapeScale::Large),
        iced::border::Radius {
            top_left: 16.0,
            bottom_left: 16.0,
            top_right: 0.0,
            bottom_right: 0.0
        }
    );
    assert_eq!(
        shapes.right(ShapeScale::Large),
        iced::border::Radius {
            top_right: 16.0,
            bottom_right: 16.0,
            top_left: 0.0,
            bottom_left: 0.0
        }
    );
}

#[test]
fn scheme_edits_are_isolated_and_invalid_metrics_fall_back_without_global_state() {
    let original = Typography::default();
    let mut edited = original.clone();
    edited[TypeScale::LabelLarge].size = 24.0;
    edited.emphasized_mut(TypeScale::TitleLarge).font.weight = Weight::Black;
    assert_eq!(original.style(TypeScale::LabelLarge).size, 14.0);
    assert_eq!(
        original.emphasized(TypeScale::TitleLarge).font.weight,
        Weight::Medium
    );
    assert_eq!(edited.style(TypeScale::LabelLarge).size, 24.0);
    edited[TypeScale::BodyLarge].size = f32::NAN;
    edited[TypeScale::BodyLarge].line_height = -1.0;
    edited[TypeScale::BodyLarge].tracking = f32::INFINITY;
    assert_eq!(
        edited.style(TypeScale::BodyLarge),
        original.style(TypeScale::BodyLarge)
    );
    let shapes = ShapeScheme {
        small: f32::NAN,
        medium: -1.0,
        ..Default::default()
    };
    assert_eq!(shapes.radius(ShapeScale::Small), 8.0);
    assert_eq!(shapes.radius(ShapeScale::Medium), 12.0);
    let scaled = original
        .clone()
        .scaled(1.5)
        .family(iced::font::Family::Monospace);
    assert_eq!(scaled.style(TypeScale::BodyLarge).line_height, 36.0);
    assert_eq!(
        scaled.style(TypeScale::LabelLarge).font.weight,
        Weight::Medium
    );
    assert_eq!(
        scaled.style(TypeScale::BodyLarge).font.family,
        iced::font::Family::Monospace
    );
    assert_eq!(original.clone().scaled(f32::NAN), original);
}

#[test]
fn typography_cache_keys_track_all_recipes_and_respect_signed_zero_equality() {
    use std::hash::{DefaultHasher, Hash, Hasher};

    let hash = |typography: &Typography| {
        let mut state = DefaultHasher::new();
        typography.hash(&mut state);
        state.finish()
    };
    let original = Typography::default();
    let mut edited = original.clone();
    edited[TypeScale::BodyLarge].font.weight = Weight::Bold;
    assert_ne!(hash(&original), hash(&edited));
    edited = original.clone();
    edited.emphasized_mut(TypeScale::LabelLarge).line_height += 1.0;
    assert_ne!(hash(&original), hash(&edited));
    edited = original.clone();
    edited[TypeScale::DisplayMedium].tracking = -0.0;
    assert_eq!(original, edited);
    assert_eq!(hash(&original), hash(&edited));
}

#[test]
fn composite_labels_use_resolved_recipes_without_clipping_taller_action_rows() {
    let mut theme = Theme::light().reduced_motion(true);
    theme.typography[TypeScale::LabelLarge].line_height = 56.0;
    theme.typography[TypeScale::LabelMedium].line_height = 28.0;
    theme.typography[TypeScale::LabelSmall].line_height = 24.0;
    theme.typography[TypeScale::BodyLarge].line_height = 32.0;
    theme.typography[TypeScale::TitleSmall].line_height = 56.0;
    let widgets = theme.widgets();
    let cases: Vec<(Element<'static, ()>, &str, f32)> = vec![
        (
            widgets
                .tabs([crate::Tab::new(1, "Tab")], Some(1))
                .on_select(|_| ())
                .into(),
            "Tab",
            56.0,
        ),
        (
            widgets
                .segmented_buttons(
                    [crate::Segment::new(1, "Segment")],
                    crate::SegmentSelection::Single(Some(1)),
                )
                .on_change(|_| ())
                .into(),
            "Segment",
            56.0,
        ),
        (
            widgets
                .navigation_bar(
                    [crate::NavigationItem::new(
                        1,
                        "Destination",
                        widget::space(),
                    )],
                    Some(1),
                )
                .on_select(|_| ())
                .into(),
            "Destination",
            28.0,
        ),
        (
            widgets
                .navigation_rail(
                    [crate::NavigationItem::new(
                        1,
                        "Destination",
                        widget::space(),
                    )],
                    Some(1),
                )
                .on_select(|_| ())
                .into(),
            "Destination",
            28.0,
        ),
        (
            widgets
                .date_picker(
                    crate::Date::new(2026, 11, 2).unwrap(),
                    crate::DateSelection::default(),
                )
                .into(),
            "November 2026",
            56.0,
        ),
        (widgets.badge(42).into(), "42", 24.0),
    ];
    for (content, label, height) in cases {
        let mut view = ui(content, theme.clone());
        assert_eq!(view.find(label).height, height, "Clipped {label}");
    }
}

#[test]
fn popup_and_notice_factories_keep_the_snapshot_in_overlay_content() {
    let mut theme = Theme::light().reduced_motion(true);
    theme.typography[TypeScale::BodySmall].line_height = 24.0;
    theme.typography[TypeScale::BodyMedium].line_height = 28.0;
    theme.typography[TypeScale::LabelLarge].line_height = 32.0;
    let widgets = theme.widgets();
    let mut tooltip = ui(
        widgets.tooltip(crate::button("Trigger").on_press(()), "Hint"),
        theme.clone(),
    );
    tooltip.frame();
    let bounds = tooltip.find("Trigger");
    tooltip.move_to(bounds.center());
    tooltip.at(600);
    tooltip.frame();
    tooltip.at(900);
    tooltip.frame();
    assert_eq!(tooltip.find("Hint").height, 24.0);

    let mut notice = ui(
        crate::snackbar::host(
            widget::space().height(300).width(iced::Length::Fill),
            Some(widgets.snackbar("Notice").action("Undo", ()).persistent()),
        ),
        theme,
    );
    notice.frame();
    notice.at(0);
    notice.at(300);
    notice.frame();
    assert_eq!(notice.find("Notice").height, 28.0);
    assert_eq!(notice.find("Undo").height, 32.0);
}

#[test]
fn theme_widget_factory_preserves_default_component_pixels() {
    for theme in [
        Theme::light().reduced_motion(true),
        Theme::dark().reduced_motion(true),
    ] {
        let widgets = theme.widgets();
        let pairs: Vec<(Element<'static, ()>, Element<'static, ()>)> = vec![
            (
                crate::button("Action").on_press(()).into(),
                widgets.button("Action").on_press(()).into(),
            ),
            (
                crate::checkbox(true)
                    .label("Choice")
                    .on_toggle(|_| ())
                    .into(),
                widgets
                    .checkbox(true)
                    .label("Choice")
                    .on_toggle(|_| ())
                    .into(),
            ),
            (
                crate::radio("Radio", 1, Some(1)).on_select(|_| ()).into(),
                widgets.radio("Radio", 1, Some(1)).on_select(|_| ()).into(),
            ),
            (
                crate::switch(true).label("Switch").on_toggle(|_| ()).into(),
                widgets
                    .switch(true)
                    .label("Switch")
                    .on_toggle(|_| ())
                    .into(),
            ),
            (
                crate::text_field("Name", "Ada")
                    .on_input(|_| ())
                    .supporting_text("Support")
                    .into(),
                widgets
                    .text_field("Name", "Ada")
                    .on_input(|_| ())
                    .supporting_text("Support")
                    .into(),
            ),
            (
                crate::assist_chip("Assist").on_press(()).into(),
                widgets.assist_chip("Assist").on_press(()).into(),
            ),
            (
                crate::filter_chip("Filter", true).on_press(()).into(),
                widgets.filter_chip("Filter", true).on_press(()).into(),
            ),
            (
                crate::list_item("Headline")
                    .supporting_text("Supporting")
                    .overline("Overline")
                    .into(),
                widgets
                    .list_item("Headline")
                    .supporting_text("Supporting")
                    .overline("Overline")
                    .into(),
            ),
            (
                crate::app_bar("A title").into(),
                widgets.app_bar("A title").into(),
            ),
            (
                crate::extended_fab(widget::space(), "Create")
                    .on_press(())
                    .into(),
                widgets
                    .extended_fab(widget::space(), "Create")
                    .on_press(())
                    .into(),
            ),
            (
                crate::slider(0.0..=1.0, 0.5)
                    .labeled(true)
                    .on_change(|_| ())
                    .into(),
                widgets
                    .slider(0.0..=1.0, 0.5)
                    .labeled(true)
                    .on_change(|_| ())
                    .into(),
            ),
            (
                crate::range_slider(0.0..=1.0, (0.2, 0.8))
                    .labeled(true)
                    .on_change(|_| ())
                    .into(),
                widgets
                    .range_slider(0.0..=1.0, (0.2, 0.8))
                    .labeled(true)
                    .on_change(|_| ())
                    .into(),
            ),
            (
                crate::tabs([crate::Tab::new(1, "Tab")], Some(1))
                    .on_select(|_| ())
                    .into(),
                widgets
                    .tabs([crate::Tab::new(1, "Tab")], Some(1))
                    .on_select(|_| ())
                    .into(),
            ),
            (
                crate::menu("Menu", [crate::MenuItem::new("Item", ())]).into(),
                widgets
                    .menu("Menu", [crate::MenuItem::new("Item", ())])
                    .into(),
            ),
            (
                crate::select("Select", [crate::SelectOption::new(1, "Value")], Some(1))
                    .on_select(|_| ())
                    .into(),
                widgets
                    .select("Select", [crate::SelectOption::new(1, "Value")], Some(1))
                    .on_select(|_| ())
                    .into(),
            ),
            (
                crate::search_bar("Search", "query").on_input(|_| ()).into(),
                widgets
                    .search_bar("Search", "query")
                    .on_input(|_| ())
                    .into(),
            ),
        ];
        for (index, (baseline, themed)) in pairs.into_iter().enumerate() {
            let mut before = ui(baseline, theme.clone());
            let mut after = ui(themed, theme.clone());
            before.at(0);
            after.at(0);
            assert!(
                before.frame() == after.frame(),
                "Default factory changed fixture {index}"
            );
        }
    }
}

#[test]
fn factory_snapshot_and_explicit_native_overrides_are_resolved_before_first_draw() {
    let mut theme = Theme::light();
    let old = theme.widgets();
    theme.typography[TypeScale::LabelLarge].size = 24.0;
    theme.typography[TypeScale::LabelLarge].line_height = 32.0;
    let widgets = theme.widgets();
    let mut small = ui::<()>(old.button("Label"), theme.clone());
    let mut large = ui::<()>(widgets.button("Label"), theme.clone());
    assert_eq!(small.find("Label").height, 20.0);
    assert_eq!(large.find("Label").height, 32.0);
    let mut explicit = ui::<()>(
        widgets
            .typography("Override", TypeScale::LabelLarge)
            .size(14)
            .line_height(widget::text::LineHeight::Absolute(20.0.into()))
            .font(crate::fonts::MEDIUM),
        theme.clone(),
    );
    let mut baseline = ui::<()>(crate::typography("Override", TypeScale::LabelLarge), theme);
    assert!(explicit.frame() == baseline.frame());
}

#[test]
fn typed_button_sizes_resolve_the_snapshot_role_and_padding() {
    let mut theme = Theme::light();
    theme.typography[TypeScale::HeadlineSmall].size = 30.0;
    theme.typography[TypeScale::HeadlineSmall].line_height = 44.0;
    let mut view = ui::<()>(
        theme
            .widgets()
            .button("Large action")
            .size(crate::ButtonSize::Large),
        theme.clone(),
    );
    assert_eq!(view.find("Large action").height, 44.0);
    let mut custom = ui::<()>(
        crate::Button::new(crate::typography("Custom", TypeScale::BodySmall))
            .size(crate::ButtonSize::Large),
        theme,
    );
    assert_eq!(custom.find("Custom").height, 16.0);
}

#[test]
fn nested_menu_labels_and_supporting_text_receive_the_same_snapshot() {
    let mut theme = Theme::light().reduced_motion(true);
    theme.typography[TypeScale::BodyLarge].line_height = 32.0;
    theme.typography[TypeScale::BodySmall].line_height = 24.0;
    let menu = theme.widgets().menu(
        "Open",
        [
            crate::MenuItem::new("Parent", ()).supporting_text("Support"),
            crate::MenuItem::submenu("More", [crate::MenuItem::new("Nested", ())]),
        ],
    );
    let mut view = ui(menu, theme);
    view.frame();
    view.click("Open");
    view.at(0);
    view.frame();
    assert_eq!(view.find("Parent").height, 32.0);
    assert_eq!(view.find("Support").height, 24.0);
    view.click("More");
    view.at(300);
    view.frame();
    assert_eq!(view.find("Nested").height, 32.0);
}

#[test]
fn app_bar_font_only_changes_invalidate_retained_ellipsis_measurements() {
    let mut theme = Theme::light();
    let title = "Long retained title with graphemes 👩🏽‍💻";
    let mut view = ui::<()>(theme.widgets().app_bar(title), theme.clone());
    let original = view.frame();
    theme.typography[TypeScale::TitleLarge].font.weight = Weight::Bold;
    view.set_theme(theme.clone());
    view.rebuild(crate::focus::scope(
        widget::container(theme.widgets().app_bar(title)).padding(24),
    ));
    let rebuilt = view.frame();
    let mut fresh = ui::<()>(theme.widgets().app_bar(title), theme);
    assert!(
        rebuilt == fresh.frame(),
        "Retained title reused the previous font paragraph"
    );
    assert!(rebuilt != original);
}

#[derive(Default)]
struct Editor {
    bounds: Option<Rectangle>,
    select_all: bool,
}
impl iced::advanced::widget::Operation for Editor {
    fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn iced::advanced::widget::Operation)) {
        visit(self);
    }
    fn text_input(
        &mut self,
        _: Option<&widget::Id>,
        bounds: Rectangle,
        state: &mut dyn iced::advanced::widget::operation::TextInput,
    ) {
        self.bounds = Some(bounds);
        if self.select_all {
            state.select_all();
        }
    }
}

#[test]
fn text_field_theme_rebuild_grows_layout_without_losing_selection_or_focus() {
    let mut theme = Theme::light();
    let field = |theme: &Theme| {
        theme
            .widgets()
            .text_field("Name", "Ada")
            .on_input(|text| text)
            .supporting_text("Supporting")
            .prefix("$")
            .suffix("USD")
            .width(320)
    };
    let mut view = ui(field(&theme), theme.clone());
    view.frame();
    let mut editor = Editor::default();
    view.operate(&mut editor);
    let before = editor.bounds.unwrap();
    view.move_to(before.center());
    view.down();
    view.up();
    editor.select_all = true;
    view.operate(&mut editor);
    theme.typography[TypeScale::BodyLarge].size = 24.0;
    theme.typography[TypeScale::BodyLarge].line_height = 36.0;
    theme.typography[TypeScale::BodySmall].line_height = 24.0;
    view.set_theme(theme.clone());
    view.rebuild(crate::focus::scope(
        widget::container(field(&theme)).padding(24),
    ));
    let mut after = Editor::default();
    view.operate(&mut after);
    assert_eq!(after.bounds.unwrap().height, before.height + 12.0);
    view.tap_key(key(iced::keyboard::Key::Character("z".into())).text("z"));
    assert_eq!(view.messages, ["z"]);
    view.tap_key(key(Named::End));
}

#[test]
fn themed_surface_shadow_matches_an_independent_explicit_radius() {
    for radius in [0.0, 24.0] {
        let mut theme = Theme::light();
        theme.shapes.medium = radius;
        let mut actual = ui::<()>(
            crate::surface(widget::space().width(180).height(100))
                .padding(0)
                .width(180)
                .variant(crate::SurfaceVariant::Elevated),
            theme.clone(),
        );
        let mut expected = ui::<()>(
            crate::elevated(
                widget::container(widget::space().width(180).height(100))
                    .width(180)
                    .style(move |theme: &Theme| widget::container::Style {
                        background: Some(theme.colors.surface_container_low.into()),
                        text_color: Some(theme.colors.on_surface),
                        border: iced::Border {
                            radius: radius.into(),
                            color: theme.colors.outline_variant,
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                1.0,
                radius,
            ),
            theme,
        );
        assert!(
            actual.frame() == expected.frame(),
            "Surface and shadow disagree for radius {radius}"
        );
    }
}

#[test]
fn theme_shape_roles_change_corners_while_explicit_radii_remain_explicit() {
    let original = Theme::light();
    let mut square = original.clone();
    square.shapes.medium = 0.0;
    square.colors.shadow = Color::TRANSPARENT;
    let surface = || {
        crate::surface(widget::space().width(180).height(100))
            .padding(0)
            .width(180)
    };
    let mut rounded = ui::<()>(surface(), original.clone());
    let mut squared = ui::<()>(surface(), square.clone());
    let before = rounded.frame();
    let after = squared.frame();
    assert_ne!(pixel(&before, 25, 25), pixel(&before, 80, 60));
    assert_eq!(pixel(&after, 25, 25), pixel(&after, 80, 60));
    let fixed = || crate::button("Fixed").radius(12.0).on_press(());
    let mut baseline = ui(fixed(), original);
    let mut themed = ui(fixed(), square);
    assert!(baseline.frame() == themed.frame());
}
