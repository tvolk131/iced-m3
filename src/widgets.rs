//! Explicit theme-bound construction for built-in component typography.
use crate::{
    AppBar, Button, Checkbox, Chip, Date, DatePicker, DateSelection, Element, Fab, FabMenuItem,
    FullScreenDialog, ListItem, Menu, MenuItem, NavigationBar, NavigationItem, NavigationRail,
    Radio, RadioGroup, RadioOption, RangeSlider, Search, Segment, SegmentSelection,
    SegmentedButtons, Select, SelectOption, Slider, Snackbar, Switch, Tab, Tabs, TextField, Theme,
    Time, TimePart, TimePicker, Tooltip, TypeScale, Typography, app_bar, assist_chip, button,
    checkbox, date_input, date_picker, extended_fab, fab, filter_chip, full_screen_dialog,
    input_chip, list_item, navigation_bar, navigation_rail, radio, radio_group, range_slider,
    search_bar, segmented_buttons, select, slider, snackbar, suggestion_chip, switch, tabs,
    text_field, time_input, time_picker,
};
use iced::widget;
use std::ops::RangeInclusive;

/// An owned typography snapshot for constructing a view. Obtain it from
/// [`Theme::widgets`], and recreate it when the application's theme changes.
/// Custom child widgets keep their own typography. Colors and shape roles still
/// resolve from the theme passed to iced at drawing time.
#[derive(Debug, Clone, Default)]
pub struct Widgets {
    pub(crate) typography: Typography,
}

impl Theme {
    /// Construct built-in labels and editors with this theme's type recipes.
    /// iced 0.14 only supplies its application theme during drawing, too late
    /// to change native text layout. This factory resolves metrics up front.
    pub fn widgets(&self) -> Widgets {
        Widgets {
            typography: self.typography.clone(),
        }
    }
}

impl Widgets {
    pub fn full_screen_dialog<'a, Message>(
        &self,
        title: impl Into<String>,
        body: impl Into<Element<'a, Message>>,
    ) -> FullScreenDialog<'a, Message> {
        full_screen_dialog(title, body).with_typography(self.typography.clone())
    }

    pub fn chip<'a, Message: 'a>(
        &self,
        label: impl widget::text::IntoFragment<'a>,
        selected: bool,
    ) -> Button<'a, Message> {
        crate::components::chip_with_typography(label, selected, &self.typography)
    }

    pub fn badged<'a, Message: 'a>(
        &self,
        content: impl Into<Element<'a, Message>>,
        count: Option<u32>,
    ) -> Element<'a, Message> {
        crate::variants::badged_with_typography(content, count, &self.typography)
    }
    pub fn split_button<'a, Message: Clone + 'a>(
        &self,
        label: impl widget::text::IntoFragment<'a>,
        action: Message,
        items: impl IntoIterator<Item = MenuItem<'a, Message>>,
    ) -> Element<'a, Message> {
        crate::expressive::split_button_with_typography(
            label,
            action,
            items,
            self.typography.clone(),
        )
    }

    pub fn fab_menu<'a, Message: Clone + 'a>(
        &self,
        icon: impl Into<Element<'a, Message>>,
        open: bool,
        toggle: Message,
        items: impl IntoIterator<Item = FabMenuItem<'a, Message>>,
    ) -> Element<'a, Message> {
        crate::expressive::fab_menu_with_typography(
            icon,
            open,
            toggle,
            items,
            self.typography.clone(),
        )
    }

    pub fn rich_tooltip<'a, Message: Clone + 'a>(
        &self,
        trigger: impl Into<Element<'a, Message>>,
        title: impl Into<String>,
        body: impl Into<Element<'a, Message>>,
    ) -> Menu<'a, Message> {
        crate::variants::rich_tooltip_with_typography(trigger, title, body, &self.typography)
    }
    pub fn menu<'a, Message: Clone + 'a>(
        &self,
        label: impl widget::text::IntoFragment<'a>,
        items: impl IntoIterator<Item = MenuItem<'a, Message>>,
    ) -> Menu<'a, Message> {
        self.menu_button(self.typography(label, TypeScale::LabelLarge), items)
    }

    pub fn menu_button<'a, Message: Clone + 'a>(
        &self,
        content: impl Into<Element<'a, Message>>,
        items: impl IntoIterator<Item = MenuItem<'a, Message>>,
    ) -> Menu<'a, Message> {
        crate::menu::menu_button_with_typography(content, items, self.typography.clone())
    }

    pub fn context_menu<'a, Message: Clone + 'a>(
        &self,
        content: impl Into<Element<'a, Message>>,
        items: impl IntoIterator<Item = MenuItem<'a, Message>>,
    ) -> Menu<'a, Message> {
        crate::menu::context_menu_with_typography(content, items, self.typography.clone())
    }

    pub fn tooltip<'a, Message: 'a>(
        &self,
        content: impl Into<Element<'a, Message>>,
        hint: impl widget::text::IntoFragment<'a>,
    ) -> Tooltip<'a, Message> {
        crate::tooltip::with_typography(content, hint, &self.typography)
    }

    pub fn badge<'a, Message: 'a>(&self, count: u32) -> widget::Container<'a, Message, Theme> {
        crate::components::badge_with_typography(count, &self.typography)
    }

    pub fn date_input<'a, Message: Clone + 'a>(
        &self,
        label: &str,
        value: &str,
    ) -> TextField<'a, Message> {
        date_input(label, value).with_typography(self.typography.clone())
    }

    pub fn time_input<'a, Message: Clone + 'a>(
        &self,
        label: &str,
        value: &str,
    ) -> TextField<'a, Message> {
        time_input(label, value).with_typography(self.typography.clone())
    }
    pub fn typography<'a>(
        &self,
        text: impl widget::text::IntoFragment<'a>,
        role: TypeScale,
    ) -> widget::Text<'a, Theme> {
        self.typography.text(text, role)
    }

    pub fn emphasized<'a>(
        &self,
        text: impl widget::text::IntoFragment<'a>,
        role: TypeScale,
    ) -> widget::Text<'a, Theme> {
        self.typography.emphasized(role).text(text)
    }

    pub fn button<'a, Message: 'a>(
        &self,
        label: impl widget::text::IntoFragment<'a>,
    ) -> Button<'a, Message> {
        button(label).with_typography(self.typography.clone())
    }

    pub fn checkbox<'a, Message>(&self, checked: bool) -> Checkbox<'a, Message> {
        checkbox(checked).with_typography(self.typography.clone())
    }

    pub fn radio<'a, Value: Clone + PartialEq + 'a, Message: 'a>(
        &self,
        label: impl widget::text::IntoFragment<'a>,
        value: Value,
        selected: Option<Value>,
    ) -> Radio<'a, Value, Message> {
        radio(label, value, selected).with_typography(self.typography.clone())
    }

    pub fn radio_group<'a, Value, Message>(
        &self,
        options: impl IntoIterator<Item = RadioOption<Value>>,
        selected: Option<Value>,
    ) -> RadioGroup<'a, Value, Message> {
        radio_group(options, selected).with_typography(self.typography.clone())
    }

    pub fn switch<'a, Message>(&self, checked: bool) -> Switch<'a, Message> {
        switch(checked).with_typography(self.typography.clone())
    }

    pub fn text_field<'a, Message: Clone + 'a>(
        &self,
        label: &str,
        value: &str,
    ) -> TextField<'a, Message> {
        text_field(label, value).with_typography(self.typography.clone())
    }

    pub fn assist_chip<'a, Message>(&self, label: impl Into<String>) -> Chip<'a, Message> {
        assist_chip(label).with_typography(self.typography.clone())
    }

    pub fn suggestion_chip<'a, Message>(&self, label: impl Into<String>) -> Chip<'a, Message> {
        suggestion_chip(label).with_typography(self.typography.clone())
    }

    pub fn filter_chip<'a, Message>(
        &self,
        label: impl Into<String>,
        selected: bool,
    ) -> Chip<'a, Message> {
        filter_chip(label, selected).with_typography(self.typography.clone())
    }

    pub fn input_chip<'a, Message>(&self, label: impl Into<String>) -> Chip<'a, Message> {
        input_chip(label).with_typography(self.typography.clone())
    }

    pub fn fab<'a, Message: 'a>(&self, icon: impl Into<Element<'a, Message>>) -> Fab<'a, Message> {
        fab(icon).with_typography(self.typography.clone())
    }

    pub fn extended_fab<'a, Message: 'a>(
        &self,
        icon: impl Into<Element<'a, Message>>,
        label: impl widget::text::IntoFragment<'a>,
    ) -> Fab<'a, Message> {
        extended_fab(icon, label).with_typography(self.typography.clone())
    }

    pub fn list_item<'a, Message>(&self, headline: impl Into<String>) -> ListItem<'a, Message> {
        list_item(headline).with_typography(self.typography.clone())
    }

    pub fn tabs<'a, Value, Message>(
        &self,
        items: impl IntoIterator<Item = Tab<'a, Value, Message>>,
        selected: Option<Value>,
    ) -> Tabs<'a, Value, Message> {
        tabs(items, selected).with_typography(self.typography.clone())
    }

    pub fn navigation_bar<'a, Value, Message>(
        &self,
        items: impl IntoIterator<Item = NavigationItem<'a, Value, Message>>,
        selected: Option<Value>,
    ) -> NavigationBar<'a, Value, Message> {
        navigation_bar(items, selected).with_typography(self.typography.clone())
    }

    pub fn navigation_rail<'a, Value, Message>(
        &self,
        items: impl IntoIterator<Item = NavigationItem<'a, Value, Message>>,
        selected: Option<Value>,
    ) -> NavigationRail<'a, Value, Message> {
        navigation_rail(items, selected).with_typography(self.typography.clone())
    }

    pub fn segmented_buttons<'a, Value, Message>(
        &self,
        items: impl IntoIterator<Item = Segment<'a, Value, Message>>,
        selection: SegmentSelection<Value>,
    ) -> SegmentedButtons<'a, Value, Message> {
        segmented_buttons(items, selection).with_typography(self.typography.clone())
    }

    pub fn slider<'a, Message>(
        &self,
        range: RangeInclusive<f32>,
        value: f32,
    ) -> Slider<'a, Message> {
        slider(range, value).with_typography(self.typography.clone())
    }

    pub fn range_slider<'a, Message>(
        &self,
        range: RangeInclusive<f32>,
        values: (f32, f32),
    ) -> RangeSlider<'a, Message> {
        range_slider(range, values).with_typography(self.typography.clone())
    }

    pub fn app_bar<'a, Message>(&self, title: impl Into<String>) -> AppBar<'a, Message> {
        app_bar(title).with_typography(self.typography.clone())
    }

    pub fn search_bar<'a, Message: 'a>(
        &self,
        placeholder: impl Into<String>,
        value: &str,
    ) -> Search<'a, Message> {
        search_bar(placeholder, value).with_typography(self.typography.clone())
    }

    pub fn select<'a, Value, Message>(
        &self,
        label: impl Into<String>,
        options: impl IntoIterator<Item = SelectOption<Value>>,
        selected: Option<Value>,
    ) -> Select<'a, Value, Message> {
        select(label, options, selected).with_typography(self.typography.clone())
    }

    pub fn date_picker<'a, Message>(
        &self,
        month: Date,
        selection: DateSelection,
    ) -> DatePicker<'a, Message> {
        date_picker(month, selection).with_typography(self.typography.clone())
    }

    pub fn time_picker<'a, Message>(&self, value: Time, part: TimePart) -> TimePicker<'a, Message> {
        time_picker(value, part).with_typography(self.typography.clone())
    }

    pub fn snackbar<Message>(&self, text: impl Into<String>) -> Snackbar<Message> {
        snackbar(text).with_typography(self.typography.clone())
    }
}
