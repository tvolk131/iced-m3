# Motion and desktop interaction polish

Current follow-up: [baseline desktop motion/layout](BASELINE_COMPLETION.md) supersedes the historical range-label, progress timing/completion, selection animation, basic-overlay and picker-header limitations below.

Latest visual/state corrections and remaining catalog gaps: [FIDELITY.md](FIDELITY.md).

This pass keeps baseline styling and prioritizes mouse/keyboard behavior. It builds
on [the desktop variants](DESKTOP.md), with Expressive styling still opt-in.

## Press feedback

Buttons, icon buttons and the action components that use them now paint a bounded
press ripple from the pointer position. Keyboard activation starts at the center.
The origin moves toward the center while the ripple grows. A separate 8% hover
layer remains beneath the 12% press layer. Rounded/asymmetric corners and ancestor
scroll clipping are preserved. The soft edge is an approximation of Material Web's
gradient, not an exact shader port.

Expansion uses Standard easing over 450ms. A quick click retains feedback until
at least 225ms after pressing, then fades over 150ms; its action publishes on
release without waiting. Holds remain visible. Cancellation skips the minimum
interval, disabling clears the gesture, and settled effects stop requesting
frames. Checkbox/radio centered ripples and switch thumb feedback are unchanged.
These timings follow [Material Web's ripple source](https://github.com/material-components/material-web/blob/main/ripple/internal/ripple.ts).

## Surface lifetimes

| Surface | Enter / exit | Behavior |
| --- | --- | --- |
| Menu/select/docked calendar/rich hint | 500 / 150ms | Reveals from the anchored edge; keeps content geometry stable |
| Basic/full-screen dialog | 300 / 200ms | Translates from/to the bottom of the window; scrim opacity follows progress |
| Plain tooltip | 150 / 100ms | Reveals after the existing 500ms hover delay; reverses on pointer return |
| Snackbar | 250 / 200ms | Moves from/to the bottom edge; identity replacement reverses from the current position |

These are configurable `theme.motion` defaults. Menu durations follow
[Material Web's menu implementation](https://github.com/material-components/material-web/blob/main/menu/internal/menu.ts).
Other surface timings and the translation/reveal treatment are library choices.
The released iced renderer does not expose group opacity for arbitrary widget
content, so we do not claim Material Web's full opacity/stagger choreography.
Standard easing is used for these transitions; Expressive spring/shape motion is
still separate work.

Use `dialog::modal(background, dialog, open)` to retain the dialog through exit.
Closing children cannot publish actions, and background input remains blocked
until exit finishes. The host restores the retained invoker and resumes suspended
background timers. Reopening reverses from the current position.
A host initially mounted open is already settled; changes to its flag animate.

Keep outgoing snackbar data and pass `.visible(false)` to animate removal.
Timeout/actions also begin exit internally and publish once. Passing `None`
removes a notice immediately. New IDs reset timeout/action state. A hidden notice
does not consume its timeout; hover and window deactivation still pause it.
The gallery retains its outgoing notice and current dialog for these lifetimes.

Menus stop accepting child actions as soon as they close, while their painted area
continues to intercept pointer input during exit. Their invoker still receives
redraw events so its ripple can finish. Tooltips remain passive; activating their
control, Escape, disabling or deactivating the window clears the hint immediately.
Hover departure uses the exit transition. Reduced motion removes animation but
preserves the tooltip's intentional hover delay. Switching to zero-duration
motion also settles an already active transition.

## Keyboard and pointer behavior

- Menu type-ahead is case-insensitive, supports Unicode text, skips disabled rows
  and moves focus without selecting. Repeated letters cycle matches; prefixes
  reset after 200ms, following Material Web's default buffer. Modifier shortcuts
  are left alone. Rich popups containing inputs do not intercept their typing or
  calendar arrow keys.
- Diagonal movement toward a submenu has a bounded 300ms grace period, including
  panels flipped to the left. Entering the child, moving away, clicking or using
  the keyboard ends/bypasses the delay. This pointer-intent heuristic is a desktop
  choice, not a Material timing token.
- Calendar arrows cross month/year boundaries. Page Up/Down change month;
  Shift+Page Up/Down change year. Month-end days clamp correctly, disabled days
  are skipped and bounds apply. Focus is restored after the application's month
  update; selection changes only on activation. An entirely disabled destination
  month does not trigger an unbounded search. These work inline and docked.

## Verification and remaining limits

The real iced runtime tests input isolation, interrupted/reversed animation,
reduced motion, calendar focus, menu typing, pointer intent and idle scheduling.
Pixel assertions check ripple opacity/origin/rounding, the menu's animated edge,
and an invoker finishing its ripple while its popup remains open. Timed reference
frames cover light/dark themes for each new surface lifecycle and button feedback.

Visual review caught two renderer-specific details: SVG tint alpha was not
applying ripple opacity, and Tiny Skia shadows ignored the parent clip. Ripple
opacity now lives in the SVG; revealing menus draw a shadow at their animated
size and suppress the full-height shadow until settled. Complete test counts and
platform coverage are in [VALIDATION.md](VALIDATION.md).

Long-title ellipsis and retained nested modal dialogs were added in the
[desktop completion pass](DESKTOP_COMPLETION.md). Localized/RTL input, native
screen-reader semantics and full Expressive fidelity remain follow-ups. Exact
Material conformance still needs the planned component/state audit. New native OS
window gestures have not been manually rechecked in this pass.
