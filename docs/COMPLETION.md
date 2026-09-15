# Adaptive navigation and catalog completion pass

Current follow-up: [baseline desktop motion/layout](BASELINE_COMPLETION.md) supersedes the historical range-label, progress timing/completion, selection animation, basic-overlay and picker-header limitations below.

Current follow-up: [desktop fidelity](DESKTOP_FIDELITY.md) supersedes the historical shadow, color-role, picker-action, slider-label and progress limitations below.


The [desktop variants pass](DESKTOP_VARIANTS.md) adds field slots/affixes, dropdown fields, clock/numeric switching, card states, lowered FABs and buffered/multicolor progress.

Latest visual/state corrections and remaining catalog gaps: [FIDELITY.md](FIDELITY.md).

This pass follows the authorized sequence: adaptive navigation, date/time
pickers, variants/carousel, then visual and Expressive refinement. It extends the
released upstream iced 0.14 implementation; it does not establish full M3 or
accessibility conformance.

Current desktop follow-up: [DESKTOP.md](DESKTOP.md).
Current motion/interaction follow-up: [MOTION_POLISH.md](MOTION_POLISH.md).

## Navigation and keyboard

`NavigationLayout::for_width` chooses Bar below 600 logical pixels, Rail below
1200, then ExpandedRail. `navigation_bar` and `navigation_rail.expanded(true)` use
the same typed destination values. `adaptive_navigation` accepts a view closure
that receives this layout choice. The gallery retains its app-owned page and
form state across changes in layout.

`AppBarVariant` supports Small, CenterAligned, Medium and Large. `collapse(0..1)`
is app-owned scroll progress. Title layout measures the actual actions and ellipsizes
text to protect them. The leading title moves horizontally before it reaches
the controls during collapse.

Wrap an app root in `focus::scope`. Enabled Material buttons, selection controls,
sliders and native inputs participate in Tab/Shift+Tab. Enter/Space activates
buttons and selection controls on release; changing focus cancels a held key.
Navigation, tabs, radio/segmented/button groups use arrows to move focus, with
explicit activation. Sliders use arrows, Page Up/Down and Home/End; range sliders
have two separate focus stops. Menus support Up/Down from an initially unfocused
panel. Open dialogs, menus, search views and modal sheets trap Tab in their panel.
Background focus is retained by the dialog host and restored on closing while
the corresponding tree still exists.

Groups now use one Tab stop and remember their active item. Keyboard traversal
scrolls focused controls into view, including nested scrollables. Selected tabs
and rail destinations are revealed on entry; calendar arrows navigate date rows.
Calendar arrows and Page Up/Down now cross month/year boundaries. Full touch/RTL
and native screen-reader support remain unfinished. See DESKTOP.md and
MOTION_POLISH.md for exact behavior.

## Pickers

`Date` is a validated Gregorian date in years 1–9999, with month/day arithmetic,
weekday calculation, strict ISO parsing and no timezone conversion.
`DateSelection` supports a single date or start/end range. A second range click
orders the endpoints; a subsequent click starts a new range.

`date_picker(month, selection)` exposes `on_select`, `on_month`, `today`, `bounds`,
`date_enabled`, `first_weekday`, and a controlled year-grid mode. Bounds also
apply to partial year pages. Disabled-date predicates apply to selected
endpoints; callers should separately validate interior dates if their booking
rules require it. `date_input` retains incomplete strings through the normal
text-field callback and displays ISO parsing errors.

`Time` is a validated 00:00–23:59 wall-clock value. `time_picker(time, part)` has
12/24-hour dials, AM/PM, pointer dragging, minute precision, keyboard increments,
and an `on_part` callback to advance from hours to minutes after release.
`time_input` provides strict numeric 24-hour entry. These controls compose into
`dialog(...)`; a compact docked calendar and validated in-picker text entry are now included.
Retained dialog stacks are supported. See [desktop completion](DESKTOP_COMPLETION.md); locale adapters remain future work.

The gallery's Schedule tab keeps drafts in application state and disables Save
until the entire date/range/time combination is valid. The example uses explicit
2026–2030 bounds, not an implicit system clock.

## Variants and motion

Added filled fields, elevated buttons/chips, input-chip avatars/dropdown marks,
interactive cards with independent child actions, dot/anchored badges, vertical
dividers, supporting text in menus, rich tooltips with actions, and switch icons.
Rich hints deliberately open on activation to keep their actions reachable.

Standard bottom sheets reserve vertical space. Opt-in `on_height` exposes a
handle and controlled live resizing; `snap_points` chooses the nearest stop on
release. A short downward drag can dismiss through the existing callback;
window/pointer cancellation does not dismiss. Pointer dragging is implemented;
finger/swipe physics remain future work. Extended FABs have an `extended(bool)`
transition, and FAB menus retain their child trees through expansion/collapse.

Carousels provide MultiBrowse, Hero, HeroCenter, Uncontained and FullScreen
treatments, fitted keylines, rounded masks, horizontal wheel navigation, drag
cancellation, keyboard selection and bounded velocity-aware snapping.
`lazy_carousel` constructs nearby items; durable values remain application-owned.
Android's full strategy/physics are not ported. Use the `background` setter when
the surrounding opaque surface differs from `theme.colors.surface`. Patterned
parents and general child opacity still need renderer integration; the tested
workarounds and proposed contract are in [COMPOSITION.md](COMPOSITION.md).

Tabs and lists now preserve the parent by default, with explicit color/gradient
fills available for their container. This closes the ordinary background mismatch
without changing selection, hover/ripple or indicator behavior.

Expressive previews include connected/separated button groups, press shape
changes, split buttons, floating/docked horizontal/vertical toolbars, FAB menus,
and a canonical loading indicator. Its seven shapes and matched morph curves are
exported by the pinned Material/AndroidX shape code; its cadence is 650ms with the
Android spring stiffness/damping and rotation parameters. Wavy linear/circular
progress, amplitude transitions and completion handoffs are implemented. See
[LOADING_PROGRESS.md](LOADING_PROGRESS.md) for API details and numerical drawing
adaptations. Expressive width/neighbor deformation, every new size/shape token
and the broader spring system remain unfinished.

## Theme and upstream boundaries

`Theme::from_accent` now uses the HCT tonal-spot scheme from the pinned
`material-colors` 0.4.2 Rust port. `from_accent_with_contrast` accepts -1..1;
`reduced_motion` sets shared durations to zero and pauses indeterminate motion.
A bounded cache avoids recomputing schemes during normal view rebuilding.
Reference colors therefore change throughout the library.

The installed released iced Widget/Operation APIs expose focus operations, but
no native accessibility-tree integration or AccessKit dependency. This is a
blocker for native screen-reader semantics through those APIs, **not** for
adaptive layout or keyboard navigation. No iced fork or silent platform bridge
has been added. Native letter tracking is also absent from the public text API. The newer desktop
work adds full-screen dialogs, cascading menus and modal/animated rails without
changing these upstream boundaries.

## Primary references

- [Navigation bar](https://github.com/material-components/material-components-android/blob/master/docs/components/BottomNavigation.md)
- [Collapsed/expanded rails](https://github.com/material-components/material-components-android/blob/master/docs/components/NavigationRail.md)
- [Date pickers](https://github.com/material-components/material-components-android/blob/master/docs/components/DatePicker.md)
- [Time pickers](https://github.com/material-components/material-components-android/blob/master/docs/components/TimePicker.md)
- [Carousel layouts](https://github.com/material-components/material-components-android/blob/master/docs/components/Carousel.md)
- [HCT scheme guidance](https://github.com/material-foundation/material-color-utilities/blob/main/dev_guide/creating_color_scheme.md)
- [Rust port and license](https://docs.rs/material-colors/0.4.2/material_colors/)
- [Loading shape sequence](https://github.com/material-components/material-components-android/blob/master/lib/java/com/google/android/material/loadingindicator/LoadingIndicatorDrawingDelegate.java)
- [Loading timing and spring parameters](https://github.com/material-components/material-components-android/blob/master/lib/java/com/google/android/material/loadingindicator/LoadingIndicatorAnimatorDelegate.java)
