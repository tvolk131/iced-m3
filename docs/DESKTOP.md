# Desktop behavior and variants

Latest visual/state corrections and remaining catalog gaps: [FIDELITY.md](FIDELITY.md).

This milestone prioritizes mouse and keyboard use, as requested. It adds desktop
variants on released upstream iced 0.14 and preserves the default visual theme.
Expressive styling remains opt-in. This is not a claim of complete M3 conformance.

Current follow-up: [motion and desktop interactions](MOTION_POLISH.md) adds surface
lifecycles, pointer-origin button ripples, menu type-ahead/pointer intent and
calendar navigation across months and years.

## Navigation and focus

`focus::scope` handles Tab/Shift+Tab across enabled controls. Navigation bars,
rails, tabs, radio groups and segmented/button groups expose one Tab stop;
arrows move within them and Enter/Space activate on release. Home/End move to
the first/last enabled action. Traversal remembers the most recently focused
item; a new selection changes the next entry point without stealing focus.
Use `focus::group_with_active(content, index)` for custom groups, counting enabled
controls. Ordinary iced ID-based focus operations remain available.

Keyboard traversal reveals controls through nested horizontal and vertical
scrollables. Scrollable groups also reveal the selected item on first layout or
selection changes when the group is not already focused. Rail headers/footers
have their own Tab stops, separate from destinations. Calendar arrows move
between days and rows; Home/End use the current week. Calendar traversal crosses
month boundaries; Page Up/Down change months and Shift+Page Up/Down change
years. Locale-specific input remains future work.

`navigation_rail(...).expanded(bool)` animates between 80 and 280 logical pixels.
Labels retain their target layout while the width changes. Reversal starts at the
current width; shared reduced motion settles immediately, with no idle frame loop.
This is a width reveal, not a port of every Expressive navigation morph.

`modal_navigation_rail(background, rail, open, on_dismiss)` presents the expanded
rail from the leading edge over a scrim. Keep the host mounted during exit. It
uses the existing sheet lifecycle, cancels covered gestures, contains keyboard
focus and blocks background input until exit finishes. Destination selection and
closing are separate app-owned messages. Put a close button in `rail.header(...)`
when wanted; the gallery does so. Pointer drag/fling is intentionally deferred.

## Cascading menus

`MenuItem::submenu(label, items)` supports multiple nested levels. Hover opens a
branch after 200ms; clicking or Right/Enter/Space opens it immediately. This hover
delay is our desktop interaction choice, not a universal Material timing token.
Root triggers also support Down/Up to enter at the selected/first or last enabled
item. Context menus support the context-menu key and Shift+F10 on a focused region.

Up/Down and Home/End navigate a panel. Left or Escape returns one level, restoring
the parent branch's focus. Escape on the root closes it; a subsequent Escape can
then dismiss an enclosing dialog. A leaf action closes the complete chain and
publishes once. Parent rows remain available to pointer input, siblings close the
previous branch, and panels flip horizontally at window edges. Outside dismissal
requires a matching press/release. Only the active panel draws keyboard focus;
ancestor branches retain an active background and preserve focus for restoration.

Type-ahead, bounded diagonal pointer intent and popup reveal/exit motion are now
implemented in the motion follow-up. Native screen-reader semantics and full
Material opacity/shape choreography remain unfinished.

## Full-screen dialogs

`full_screen_dialog(title, body)` returns a builder with `on_dismiss`, `action`,
`dismiss_on_escape` and `initial_focus`. Convert it with `.into()` when passing it
to `dialog::modal`. Its title/close/action bar stays fixed while the body scrolls.
The full window is opaque, input is isolated, and Tab remains inside the dialog.
`initial_focus(index)` counts enabled traversal stops; the full-screen default is
the close action. Basic dialogs also offer this explicit initial-focus setting.

The gallery's **More → Edit workspace details** keeps a separate draft. Closing an edited draft now opens a discard confirmation; Keep editing preserves it.
Save validates the name and commits both fields.
Basic/full-screen dialog entrance/exit animation is available through the retained
`dialog::modal` API. The subsequent [desktop completion pass](DESKTOP_COMPLETION.md)
adds `dialog::stack` for retained nested dialogs, including focus/caret restoration.
Very long app-bar titles now ellipsize within the space left by actions.

## Docked calendars

`docked_date_picker(trigger, date_picker(...))` anchors the calendar to a passive
trigger. It flips above when needed and scrolls in a short viewport. Month/year
messages keep it open across view rebuilds. A single date closes on selection;
a range closes after its second endpoint. Escape/outside dismiss the popup only:
previous selection messages are not rolled back. Applications can retain drafts
when they need transactional confirmation. The gallery exposes this under
**Workspace → Schedule → Open calendar**.

This reuses the existing calendar, bounds, disabled-date predicate and keyboard
grid. The subsequent [compact desktop profile and manual-entry toggle](DESKTOP_COMPLETION.md)
are now included; localized formatting and alternative calendar systems remain future work.

## Validation and references

Behavior and precise reference frames live in `tests/visual/desktop.rs`, with
complete app compositions and draft/selection workflow tests in the gallery.
Canonical references use Tiny Skia, bundled Roboto and a fixed 2× scale. GPU tests
use the same headless iced runtime with wgpu/Metal; remote Windows/Linux and native
screen-reader support remain unverified. Current totals are in [VALIDATION.md](VALIDATION.md).

Primary design references:

- [M3 navigation rail](https://github.com/material-components/material-components-android/blob/master/docs/components/NavigationRail.md)
- [M3 menus](https://github.com/material-components/material-components-android/blob/master/docs/components/Menu.md)
- [Basic/full-screen dialogs](https://github.com/material-components/material-components-android/blob/master/docs/components/Dialog.md)
- [Docked/modal date pickers](https://developer.android.com/develop/ui/compose/components/datepickers)
