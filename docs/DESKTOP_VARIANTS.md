# Desktop variants pass

Current follow-up: [desktop fidelity](DESKTOP_FIDELITY.md) supersedes the historical shadow, color-role, picker-action, slider-label and progress limitations below.


Completed 2026-09-14. This pass adds field adornments, filled/outlined dropdown
fields, numeric entry inside the time picker, disabled/dragged cards, lowered
FABs, and buffered/multicolor progress. Review the
[component frames](http://127.0.0.1:8766/?filter=variants-) and
[gallery compositions](http://127.0.0.1:8766/?filter=gallery-variants).

## Fields and dropdowns

`TextField` now has `leading(icon)`, `trailing(icon)`, `trailing_action(icon,
message)`, `prefix(text)` and `suffix(text)`. Passive icon slots are 24px; the
trailing action uses the shared 40px icon button with keyboard focus and ripple.
Disabling the field disables that action regardless of which builder came first.

Affixes are display content, separate from the native editor's value, selection,
clipboard and input callbacks. They appear with the floating label; empty,
unfocused fields show their label without affixes. Padding reserves the slots,
and outlined labels move back toward the outline's leading edge when floating.
Long affixes are clipped to leave space for editing on narrow fields. Supporting
text wraps below the field. Icons should be passive content that inherits its
parent foreground; caller-owned explicit colors retain their own styling.

```rust,ignore
text_field("Budget", amount)
    .on_input(Message::Amount)
    .prefix("$")
    .suffix("USD")
    .supporting_text("Budget for this workspace");

text_field("Password", password)
    .variant(TextFieldVariant::Filled)
    .on_input(Message::Password)
    .secure(!show_password)
    .leading(lock_icon)
    .trailing_action(visibility_icon, Message::TogglePassword);
```

The native iced editor and its widget tree remain intact during field rebuilds.
Caret placement, selection, secure entry and IME handling still belong to iced.

`Select` adds `variant(TextFieldVariant)`, `leading(icon)`, `supporting_text(text)`
and `error(text)`. It uses the same field drawing recipe with its own focus target;
it does not expose a text editor or start an IME. Its menu retains arrow/Home/End
navigation, typeahead, disabled options, Escape and focus restoration. The popup
anchors at the field bottom and can cover supporting text while open. Empty
selects show the resting label. Editable/autocomplete comboboxes remain outside
this API; the search component is available for controlled query workflows.

## Clock and numeric entry

A time picker can now switch between its dial and numeric entry in place:

```rust,ignore
time_picker(time, part)
    .format24(use_24_hour)
    .on_change(Message::Time)
    .on_part(Message::Part)
    .input_mode(typing)
    .on_toggle_input(Message::ToggleTimeEntry)
    .hour_input(hour_draft, Message::HourDraft)
    .minute_input(minute_draft, Message::MinuteDraft)
    .input_period(pm_draft, Message::PeriodDraft);
```

Keep the mode, hour/minute strings and AM/PM draft in application state. Hour
entry accepts 0–23 in 24-hour mode or 1–12 in 12-hour mode; minutes accept 0–59.
One or two ASCII digits are accepted, preserving incomplete text without
silently changing the selected time. Midnight/noon convert correctly between
12-hour display and the underlying 24-hour `Time`.

**Apply** or Enter in a native input emits `on_change` only for a valid draft.
`input_time()` exposes the same validation for an application's own confirmation
action. Changing AM/PM emits the draft callback; it does not commit the time.
Switching modes alone never applies a draft. The gallery syncs drafts when a
dial value is selected or a valid draft is applied, and returns to the dial after
Apply. It also converts drafts when changing the 12/24-hour format.

The numeric view retains Display Large 57/64 typography, 80px input height,
96/114px input widths, helper labels and a separate period selector. Dial/input
variants follow the [official time-picker guidance](https://developer.android.com/develop/ui/compose/components/time-pickers).
The library's inline Apply arrangement is a desktop composition; exact dialog
footer/period-border choreography and locale-aware input remain separate work.

## Cards and FABs

`Card::disabled(true)` suppresses the parent action and child input/focus targets,
and cancels a held child action. Passive cards without an action remain enabled
containers. Explicit styles on arbitrary child widgets remain caller-owned;
set child disabled appearances too when composing independently styled controls.

`Card::dragged(true)` applies the dragged state layer and elevation and suppresses
activation while dragging. It is a controlled appearance: the application owns
pointer drag recognition, positioning and drop handling. Disabled wins over
dragged. The baseline recipes are:

| Card | Disabled container/outline | Dragged elevation |
| --- | --- | --- |
| Filled | Surface variant at 38% | 3 |
| Outlined | Surface, outline at 12% | 3 |
| Elevated | Surface at 38%, elevation 1 | 4 |

Dragged cards use an on-surface state layer at 16%. `ColorScheme::surface_variant`
is now available as the semantic role needed by the disabled filled recipe.

`Fab::lowered(true)` uses elevation 1 at rest/focus/press and 2 on hover, including
extended FABs. The surface-colored lowered FAB also uses surface-container-low;
regular surface FABs retain surface-container-high. Other color and size choices
continue to work. These state choices follow the pinned Material Web v0_192
[card](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/tokens/versions/v0_192/_md-comp-filled-card.scss)
and [FAB](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/tokens/versions/v0_192/_md-comp-fab-surface.scss)
tokens. The renderer's existing single-shadow approximation remains.

## Buffered and multicolor progress

`linear_progress(done).buffer(available)` adds an independently animated buffered
fraction behind completed work. Fractions are clamped, NaN means zero, and the
buffer never draws behind the completed fraction. Circular and indeterminate
indicators ignore buffer values without scheduling extra work.

Both indicator kinds support `.colors([color1, color2, color3, color4])` (or any
nonempty color list). Determinate indicators use the first color. Indeterminate
indicators smoothly blend to the next color during the last quarter of each
existing motion cycle. An empty list uses theme primary. Pause freezes both
geometry and color, reduced motion uses a stable first-color pose, and hidden
indicators retain the existing suspension behavior.

These are documented library variants: the buffer is a translucent secondary
bar (24% indicator color), and color handoffs reuse the existing 2-second linear
and 1.333-second circular cycles. They are not an exact port of every Android
four-color delegate or the older dotted-buffer choreography. Existing default
progress geometry and animation curves are unchanged. Current Expressive wavy
progress remains deferred; [Android's progress documentation](https://github.com/material-components/material-components-android/blob/master/docs/components/ProgressIndicator.md)
describes the broader set of styles.

## Coverage and remaining scope

Runtime tests cover native editing state across field rebuilds, independent
trailing actions, disabled gesture cancellation, selection-only dropdowns,
keyboard menu navigation, time validation/commit rules, disabled/dragged card
children, independent buffering, and paused/reduced color motion. Scalar tests
check invalid fractions and continuous color-cycle wraparound. Gallery tests
exercise draft commits and format conversion.

Timed references cover light/dark fields (including narrow, error and disabled
states), floating labels, held/released trailing actions, dropdown focus/popups,
12/24-hour numeric pickers, card states, lowered FABs, buffer interpolation and
multicolor cycles. Gallery compositions cover 390px and 1280px layouts.

This completes the scoped variants pass. Native screen-reader semantics, exact
tracking/caret colors, locale/RTL adapters, cross-platform native validation,
full shadow recipes, carousel physics and complete Expressive motion remain in
[the fidelity matrix](FIDELITY.md). See [validation results](VALIDATION.md) for the
executed checks and current reference totals.
