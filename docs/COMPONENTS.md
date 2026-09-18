# Component catalog

| Component | Public API | Behavior |
| --- | --- | --- |
| Buttons | `button(label)`, `Button::new(content)` | Filled, elevated, outlined, text, tonal; disabled, hover and pressed states; animated tonal release |
| Text fields | `text_field(label, value)` | Outlined/filled fields, floating label; supporting/error text; secure entry; native selection, clipboard, editing, focus, submit and IME forwarding |
| Dialogs | `dialog(content)`, `dialog::modal(background, dialog, open)`, `dialog::stack`, `full_screen_dialog` | Retained entrance/exit, nested focus restoration, scrim, Escape/outside dismissal and scrollable long content |
| Typography | `typography(text, TypeScale)` | All 15 baseline M3 roles; bundled Roboto Regular/Medium; returns an iced `Text` |
| Surfaces/cards | `surface(content)`, `card(content)` | Filled, outlined, elevated; arbitrary children, width and padding; interactive cards support disabled/dragged states |
| Dividers | `divider()` | Semantic outline-variant role |
| Icon buttons | `icon_button(content)` | Circular 40px action; standard, outlined, filled and tonal colors; controlled toggle selection |
| Checkboxes | `checkbox(checked).label(...)` | Circular hover/press feedback; animated check/mixed marks; error and disabled states; 48px target |
| Switches | `switch(checked).label(...)` | 52×32 track; 40px hover halo around the thumb, pressed thumb growth and finite selection motion |
| Chips | `assist_chip`, `suggestion_chip`, `filter_chip`, `input_chip` | Four baseline families; leading icons, animated selection, independent input removal; legacy `chip` helper retained |
| Search | `search_bar(placeholder, query)` | Controlled query/results/open state; expanding docked or full-window view, autofocus, clear/back/submit, scrolling and outside/Escape dismissal |
| Radio buttons/groups | `radio(label, value, selected)`, `radio_group(options, selected)` | Controlled single selection; animated dot and circular hover/press feedback; disabled options |
| Menus/selects | `menu(label, items)`, `menu_button(content, items)`, `context_menu(content, items)`, `select(label, options, selected)` | Anchored, scrollable popups; disabled/selected items, separators and shortcut hints; Escape/outside dismissal |
| Tooltips | `tooltip(content, hint)` | Plain text hint after 500ms; fits/flips within the window; does not intercept input |
| Snackbars | `snackbar(text)`, `snackbar::host(background, notice)` | Bottom feedback with optional action; four-second timeout; pauses on hover, window deactivation and root dialogs |
| Badges | `badge(count)`, `badge_dot()`, `badged(content, count)` | Count pill including `99+`; pass `None` to `badged` for a positioned dot |
| Tabs | `tabs(items, selected)` | Primary/secondary; icons, badges, disabled items, animated indicator and optional horizontal scrolling |
| Lists | `list(items)`, `list_item(headline)` | Supporting text/overline, leading/trailing content, row selection and independently interactive child controls |
| App bar | `app_bar(title)` | Small, center-aligned, medium and large bars; controlled collapse and optional scrolled elevation |
| Navigation rail | `navigation_rail(items, selected)` | 80/280px collapsed/expanded rail with animated width, roving focus and overflow scrolling; modal host available |
| Navigation bar/adaptive layout | `navigation_bar`, `adaptive_navigation` | Compact bar and app-owned selection; switches navigation layout based on available width |
| Sliders | `slider(range, value)`, `range_slider(range, (lower, upper))` | Single or two-handle selection; continuous/stepped values, ticks/value bubble, hover/drag feedback, release callback and disabled states |
| Segmented buttons | `segmented_buttons(items, selection)` | Controlled single/multiple selection, optional icons replaced by checks, connected outlines and animated selection colors |
| Progress | `linear_progress(value)`, `circular_progress(value)` | Fractions in 0..=1, buffering, color cycles, indeterminate motion, wavy variants and pause; hidden indicators stop requesting frames |
| Loading | `loading_indicator()` | Canonical seven-shape morph sequence, optional container and explicit pause |
| Date/time pickers | `date_picker`, `docked_date_picker`, `date_input`, `time_picker`, `time_input` | App-owned dates/times, single/range dates, calendar/text entry, clock/numeric entry and validation |
| Carousels | `carousel`, `lazy_carousel` | Fitted keylines, snapping and lazy construction of nearby items; durable item state belongs to the app |
| Expressive previews | `button_group`, `split_button`, `toolbar`, `fab_menu`, `rich_tooltip` | Action compositions and rich hints; see the documented variant and motion limitations |
| Sheets | `side_sheet(content)`, `bottom_sheet(content)`, `sheet::host(background, sheet)` | Standard/modal side and bottom sheets, optional bottom-sheet handle resizing; reversible entrance/exit motion, scrolling, configurable Escape/outside dismissal and modal input blocking |
| Floating action buttons | `fab(icon)`, `extended_fab(icon, label)` | Small/regular/large, extended label, four semantic colors, elevated hover and held/release states; controlled action and optional disabled treatment |

`Button::new` and icon buttons accept passive custom content such as an icon/text row. Interactive child controls are not supported inside a button. Icon buttons provide a centered 24×24px content area. The gallery uses small vector drawings that inherit the button foreground, including disabled/selected colors; the library does not impose an icon pack. Text symbols can appear off-center because their font baseline and line height differ from their visible shape.

Icon buttons keep the `ButtonVariant` API: `Text` means standard. Calling
`.selected(bool)` opts into toggle styling; omitting it creates an ordinary action.
Filled/tonal toggles use a neutral container when unselected and their respective
accent container when selected. Outlined selection uses inverse surface colors.

See the [cookbook](COOKBOOK.md) for composed examples and the
[documentation index](README.md) for implementation notes and scope.
