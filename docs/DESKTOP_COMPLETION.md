# Desktop completion pass

Current follow-up: [desktop fidelity](DESKTOP_FIDELITY.md) supersedes the historical shadow, color-role, picker-action, slider-label and progress limitations below.


Completed 2026-09-14. This pass adds title ellipsis, a retained dialog stack, and
compact calendars with integrated text entry. Review the
[component frames](http://127.0.0.1:8766/?filter=desktop-completion) and
[gallery examples](http://127.0.0.1:8766/?filter=gallery-desktop-completion).

## App-bar titles

All four app-bar variants ellipsize long titles within the space left by leading
and trailing actions. Medium/large bars recompute the available width as their
controlled collapse value changes. Center-aligned bars keep symmetric title
insets. Titles use the actual Roboto paragraph shaper and truncate at Unicode
grapheme boundaries, preserving combining marks and emoji sequences. If even the
ellipsis does not fit, the title draws nothing. Widening the bar restores text.

Measurements are cached in the widget tree and reused until the title, typography
or available width changes. No character-count width heuristic is used. Full
title text remains available to iced text operations.

## Dialog stack

Use `dialog::stack(background, [(parent, parent_open), (child, child_open)])`.
Entries are ordered bottom to top. Keep entries mounted in stable order and
change their flags; removing/reordering them discards their positional state and
cannot preserve an exit animation. Basic and full-screen dialogs can be mixed.
Use `dialog::modal(background, dialog, open)` for a single retained dialog.

- Only the top visible dialog accepts input or participates in focus operations.
- Tab/Shift+Tab wrap in that dialog. Escape and scrim dismissal affect it alone.
  A menu opened within it receives Escape before its containing dialog.
- Covered pointer/key gestures are cancelled; their releases cannot activate a
  covered action. Input stays blocked through exit animations.
- Each dialog defaults to its first enabled control; `initial_focus(index)`
  overrides that choice. Closing restores the prior focus target beneath it,
  preferring its widget ID and otherwise its traversal position. If the target
  disappears, restoration uses the nearest remaining traversal position.
- Native input caret/selection state is preserved using iced's public input
  state API. A window that loses focus is not synthetically reactivated when a
  dialog closes; background restoration waits for window focus to return.
- Surfaces use separate renderer layers so upper dialog backgrounds occlude
  text below. Covered popups are cancelled instead of remaining interactive.

The gallery's **More → Edit workspace details** demonstrates this with a discard
confirmation over the full-screen editor. **Keep editing** preserves the draft;
**Discard** closes both dialogs; **Save** commits it.

## Compact calendars and manual entry

`DatePicker::compact(true)` provides a desktop profile with a 328px preferred
width, 16px padding/corners, 40px day rows and the existing 40px day indicators.
It omits the large modal headline. `docked_date_picker` selects this profile
automatically, retains month/year navigation and flips/scrolls at viewport edges.
These dimensions are a documented desktop adaptation, not a claim of a complete
port of Google's docked picker tokens or separate month/year dropdown menus.

Enable calendar/text switching on either profile:

```rust,ignore
let picker = date_picker(month, selection)
    .on_month(Message::Month)
    .on_select(Message::Date)
    .input_mode(typing)
    .on_toggle_input(Message::ToggleDateEntry)
    .input(&date_text, Message::DateText)
    .end_input(&end_text, Message::EndText)
    .bounds(first_allowed, last_allowed);
```

Keep raw strings and the mode in application state. Incomplete edits survive mode
switches. Manual fields use the same bounds and disabled-date predicate as the
calendar, and reject reversed ranges. **Apply** or Enter in a field publishes the
existing `on_select` message only for a complete valid selection. A docked popup
then closes. Invalid input keeps it open. `input_selection()` exposes the same
validation for application-owned Save/OK actions. Applications still validate
range interiors if they have restrictions beyond endpoint availability.

The edit/calendar icon has a tooltip and participates in keyboard traversal. Date
input fields use the picker's themed surface, including their floating-label
cutout, through the new reusable `TextField::background_with` builder.

The calendar/text distinction and toggle follow the
[official date-picker anatomy and input variants](https://github.com/material-components/material-components-android/blob/master/docs/components/DatePicker.md).
Formatting remains ISO input and English calendar labels. Locale/RTL adapters,
time-picker inline mode switching and the remaining exact modal header/action
recipes are separate work.

## Validation

Nine new ordinary tests cover shaped title fitting, three dialog levels, focus
wrapping/restoration, native caret preservation, window deactivation, held-gesture
cancellation, popup/scrim dismissal priority, opaque layering, date restrictions,
mode changes, and manual Apply/Enter. The existing gallery draft test now checks
the discard-confirmation workflow.

120 new reference images cover light/dark, 320/720px app bars, nested dialog
opening/closing frames, compact/modal calendars, single/range text entry, invalid
and incomplete values, and 12 gallery compositions at 390/1280px widths.
The earlier fidelity field fixtures now select text for focused snapshots:
iced's native blinking caret uses wall time independently of the private Material
motion clock, so a fixed selection avoids inconsistent reference frames.

This completes the scoped desktop pass. It does not add native screen-reader
semantics, locale support, remaining component variants, or full Expressive motion.
See [the fidelity matrix](FIDELITY.md) for those boundaries and
[validation results](VALIDATION.md) for the executed checks.
