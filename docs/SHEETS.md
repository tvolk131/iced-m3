# Sheets and primary actions

Current update: [adaptive navigation and catalog completion](COMPLETION.md) supersedes earlier milestone notes about keyboard support, HCT palettes, pickers, sheet dragging, FAB expansion and Expressive previews.

This milestone adds standard and modal side sheets, modal bottom sheets, and
floating action buttons. It follows the same baseline Material 3 system as the
existing library. The app owns visibility, field values and business actions;
the widget tree owns motion, scroll state and pointer gestures.

## Material references and chosen behavior

References checked on 2026-09-13:

- [Material side sheets](https://github.com/material-components/material-components-android/blob/master/docs/components/SideSheet.md): standard sheets coexist with the main UI, while modal sheets block it with a scrim.
- [Material bottom sheets](https://github.com/material-components/material-components-android/blob/master/docs/components/BottomSheet.md): supplementary content presented from the bottom, including modal use.
- [Baseline Material Web FAB tokens](https://github.com/material-components/material-web/blob/main/tokens/_md-comp-fab.scss) and [large FAB measurements](https://github.com/material-components/material-web/blob/main/tokens/versions/v0_192/_md-comp-fab-surface-large.scss): size, color, icon, state-layer and elevation roles.

The implementation is original Rust on released upstream iced 0.14. Numeric
design tokens inform the layout; no Android or Material Web implementation was
copied. This is not an implementation of all newer Expressive variants.

| Component | Defaults |
| --- | --- |
| Standard side sheet | Right side; 360px width, full height, surface color, square edges and a 1px divider; reserves space beside content |
| Modal side sheet | Right side; 360px width, full height, surface-container-low, 16px leading corners, 32% scrim at full expansion |
| Modal bottom sheet | Centered; maximum 640px width, requested height 480px, surface-container-low, 28px upper corners, elevation 3 |
| Small FAB | 40×40px container, 12px corners, 24px icon slot |
| Regular FAB | 56×56px container, 16px corners, 24px icon slot |
| Large FAB | 96×96px container, 28px corners, 36px icon slot |
| Extended FAB | 56px height, natural width, 16px corners, 24px icon, 12px icon/label gap, LabelLarge type, 16px leading and 20px trailing padding |

Sheet dimensions clamp to their host. Modal side/bottom sheets leave up to 56px
of outside space on their entrance axis; tiny windows reduce that margin.
Content has 24px padding and a vertical scrollbar with a reserved gutter. Lists
return ordinary containers: use `.style(...)` to make a list transparent when
placing it on a different sheet surface. Fields should receive the same surface
color through `.background(...)` for their floating-label notch.

FAB colors are Primary, Secondary, Tertiary and Surface. Container/on-container
pairs and the added surface-container-low role follow the existing approximate
sRGB theme generation. This does not add HCT. New role pairs are included in the
contrast checks. FABs reuse the shared button's 8% hover and 12% held state layer,
150ms release, and matching press/release action semantics. Elevation moves from
3 at rest/press to 4 on hover; the shadow stays within the ancestor viewport.
Omitting the callback disables the FAB as a library convenience; normally an app
should hide an unavailable primary action. This remains a mouse-first preview,
so the small FAB's hit box is its 40px container.

## Persistent visibility and motion

Construct `sheet::host(background, side_sheet(content).open(is_open))` on every
view rebuild, including when closed. Use `.modal()` for a modal side sheet, or
`bottom_sheet(content)` for a modal bottom sheet. Constructors start closed.
`on_dismiss` reports a request; the app applies it by changing its Boolean state.
An app can instead close through any button/message in its arbitrary content.

The host retains both child trees throughout closing and reopening. Standard
sheets animate the reserved background width; modal sheets translate over the
background while the scrim fades. The default theme chooses 300ms entrance and
200ms exit with Material standard easing, `cubic-bezier(0.2, 0, 0, 1)`. These are
the library's consistent timing choices, not a claim that every Material platform
uses identical durations. Set `theme.motion.sheet_enter` / `sheet_exit` to adjust
them; zero durations settle immediately. Reversing retargets from the current
position, without a discontinuity. Settled or window-deactivated sheets request
no animation frames; elapsed motion settles when the window becomes active again.

The full-window host belongs at the root, inside any dialog host. Place snackbar
hosts inside the sheet host so modal opening suspends their timers. A modal sheet
captures pointer, wheel, keyboard, touch and IME input through its exit. Closing
turns its own children inert immediately. Opening cancels a covered background
gesture; closing/resuming never replays it. Nested menus get first refusal of
Escape; otherwise configured Escape or a matching outside left press/release
requests dismissal. Missing/disabled dismissal callbacks do not permit input to
pass through the scrim. Standard sheets leave the main UI usable.

Enclosing dialogs hide sheet popups and cancel their active gestures. Closing a
sheet cancels child focus/press activity while retaining values in the app and
the scrollbar offset in the tree. Reopening resumes child timers without restoring
text focus automatically. The same host can switch from a standard side sheet
to a modal bottom sheet when the application changes its responsive layout.

## Gallery and scope

Workspace → Files opens details for the two available files. Windows below 900px
use the modal bottom sheet; wider windows use the standard side sheet. More →
File details in modal sheet demonstrates the modal side variant at wide sizes.
Per-file notes survive closing. The floating New workspace action opens a naming
dialog; an empty name cannot submit, and a successful action changes the session
workspace name and posts a snackbar. This demo does not write files or persist data.

Components → Floating action buttons shows all sizes and colors. On narrow
windows, file-row actions move below the row to preserve readable text, and the
Files scroll area reserves room for its floating action.

This milestone intentionally uses close buttons, Escape and outside-click
dismissal. Drag/swipe dismissal, snap points, left-edge/detached sheets, animated
extended-FAB expansion, FAB menus, focus trapping/restoration and screen-reader
integration remain outside scope. The sheet host is not intended inside an
unbounded scroll region. For FAB content, prefer `icon(svg_handle)` with centered
artwork in a square SVG view box: 24×24 logical pixels for small, regular and
extended FABs, or `.size(36)` for large FABs. Custom children are passive and sized
by the caller. The FAB centers their layout box without scaling the artwork or
adjusting its optical alignment; a text glyph's baseline and line height can make
its visible ink look off-center.

## Validation

The ordinary tests cover reversible motion, idle scheduling, zero duration,
window deactivation, invalid dimensions, modal/standard changes, native typing,
nested-menu Escape, outside gesture origins, covered presses, blocking throughout
exit, long-content scrolling, retained scroll offsets, snackbar suspension, and
a dialog over a sheet. FAB tests check hover, held-state pixels, one action on
release, release outside and disabling while pressed. Gallery tests cover file
notes and creation through component messages.

The reference suite adds 358 images: 102 sheet frames, 224 FAB states and 32 gallery
compositions. Sheet captures include entrance at 0/50/100/150/225/300/500ms, exit at
0/50/100/150/200ms, reversal and 320/390px layouts. FAB captures include intermediate
hover/release and a 500ms hold in both themes. These use the private test clock and
real renderer; gallery images use zero sheet entrance duration to capture settled
compositions. See [VISUAL_TESTS.md](VISUAL_TESTS.md) and [VALIDATION.md](VALIDATION.md)
for executed checks and platform limits.
