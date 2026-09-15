# Search and filtering

This milestone adds a search bar/view, all four baseline chip families, and a
two-handle range slider. It uses released iced 0.14 APIs. Queries, results,
selection, visibility and ranges remain application-owned; no search service,
global timer, or application animation messages are required.

## Search

`search_bar(placeholder, query)` returns a `Search` builder. Supply `.on_open`,
`.on_close`, `.open(bool)`, `.on_input`, and `.results(content)`. Optional
`.on_submit(message)` handles Enter. Keep the widget mounted when closed so its
native input/scroll state and exit animation survive rebuilds.

The closed bar is 56px high, with a 28px radius, 24px search icon, Body Large
typography and `surface_container_high`. It expands into a docked view on windows
at least 600px wide and fills smaller windows. `.full_screen(bool)` overrides
that choice. The bar is capped at 720px; docked views are at least 360px when the
viewport allows it and stay within an 8px window margin. `.height(...)` changes
the desired 400px docked height. Near the bottom edge, the view shifts upward.

The view focuses its native iced text input on opening, supports normal editing,
and exposes back and clear actions. Clearing publishes an empty query; it does
not close the view. The application decides what submitting or choosing a
result does. Results can include suggestions, lists, chips and nested menus, and
scroll independently. Escape closes a nested menu before requesting search
dismissal. Outside dismissal requires a press and release outside. Search captures
pointer, wheel and keyboard input through its exit transition.

Expansion uses Standard easing over `theme.motion.search_enter` (300ms), and
collapse uses `search_exit` (250ms). Both can be zero. Reversing a transition starts
at its current geometry. Motion stops requesting frames once settled; a focused
native text input can still schedule its own caret blink. Timing is a library
choice informed by baseline Material motion, not a claim of a single mandatory
M3 duration across platforms. This version animates the container geometry; icon
morphing, staggered content motion and Expressive springs are not implemented.

Search is an anchored widget, not a root modal host. It does not provide a complete
focus trap or restore a previously focused background field. Avoid opening it
programmatically during an unrelated background drag. Screen-reader semantics,
keyboard result traversal and asynchronous search/cancellation remain application
or future accessibility work.

## Chips

| Builder | Purpose |
| --- | --- |
| `assist_chip(label)` | An action, with an optional leading icon |
| `suggestion_chip(label)` | A suggested query or response |
| `filter_chip(label, selected)` | A controlled filter, with a selected checkmark |
| `input_chip(label)` | An input token with optional activation, selection and removal |

All use 32px containers, 8px corners, Label Large typography, an outline-variant
border and the shared button state layers. Selected filter/input chips transition
to the secondary container and lose their outline over `motion.medium` (200ms).
Disabled chips suppress actions, cancel pending gestures and use disabled colors.
The compatibility `chip(label, selected)` helper is unchanged.

`.leading(content)` accepts a passive 18px icon. Filter chips reserve an 18px
check slot, even without a leading icon, to keep neighboring chips stationary
when selection changes. This is a deliberate layout choice. `.on_press(message)`
activates the body. `.on_remove(message)` gives input chips an independent 32px
remove target; it never activates the body. Removal-only chips work without
`.on_press`. The application removes the token on that message. Use `.disabled`
to disable the whole chip, and keyed parent widgets when dynamically reordering
interactive children. Wrap chip rows with iced's `.wrap()`.

These are baseline outlined chips. Elevated assist/suggestion treatments, avatar
slots, dropdown indicators, touch-target expansion and animated insertion/removal
are outside this milestone. Buttons/chips retain the existing shape-matched tonal
press effect rather than a pointer-origin radial ripple.

## Range slider

`range_slider(domain, (lower, upper))` uses the existing baseline slider geometry:
a 4px track, two 20px circular handles and a 48px interaction area. Only the track
between handles is active. `.on_change` publishes the complete pair; `.on_release`
publishes once after the final change of a completed drag. Cancellation does not
commit. `.step`, `.ticks`, `.labeled`, `.value_labels`, `.width` and `.disabled`
parallel the single-value slider.

Values are ordered, clamped and snapped from the domain minimum. Both endpoints
remain reachable even when the step does not divide the domain evenly. Invalid
domains are inert; nonfinite and reversed input values are normalized. Closest
handle selection is based on pointer distance. Handles meet without crossing;
when exactly coincident, the first meaningful drag direction chooses which handle
to separate. Dense ticks are omitted when they cannot be visually separated.

Labeled sliders reserve 32px above the track and show only the active handle's
value, avoiding overlapping bubbles. Hover and held feedback use `motion.short`
(150ms), preserve the active handle across rebuilds, and settle without ongoing
animation frames. Vertical layout, keyboard operation, minimum separation and
the newer Expressive handle shape are outside this baseline milestone.

## Gallery and validation

Workspace → Files combines case-insensitive text matching, Shared/Pinned filters,
and a stepped age range. Suggestions and at most five recent queries are local
session data. Enter keeps the filtered file list; selecting a result opens its
detail sheet. Clearing a query preserves the other filters; Reset clears all.
The archive row intentionally remains unavailable. Components also includes a
four-family chip example with working removal and restoration.

`tests/visual/search.rs` exercises actual iced input, overlays and rendering:
focus/typing/clear/submit, outside and nested Escape handling, scrolling/reopening,
reversible/zero-duration motion, opaque surfaces, independent chip removal,
range clamping/snapping/cancellation and coincident handles. Reference sequences
cover search expansion/collapse, chip hover/press/selection/removal, and slider
hover/held/drag/release in light and dark themes. Gallery references cover three
window widths, suggestions, queries, combined filters and empty results.

Native iced caret timing is independent of the private component clock. Reference
tests explicitly unfocus the input through an operation before capturing images;
separate behavioral tests exercise autofocus and editing without pixel baselines.

## References

- [Official Material search documentation](https://github.com/material-components/material-components-android/blob/master/docs/components/Search.md)
- [Official Material chip documentation](https://github.com/material-components/material-components-android/blob/master/docs/components/Chip.md)
- [Official Material slider documentation](https://github.com/material-components/material-components-android/blob/master/docs/components/Slider.md)

These document the search bar/view split, four chip families, and two-thumb range
selection. The library follows the established baseline visual system and does
not claim complete platform or Expressive parity.
