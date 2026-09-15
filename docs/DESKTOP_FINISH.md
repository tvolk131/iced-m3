# Overlay choreography and carousel fitting

This pass extends the baseline desktop profile on released iced 0.14. It does
not declare full M3 conformance. Expressive changes and native accessibility are
separate from these baseline changes.

## Retained overlays

`dialog::actions(row)` gives a basic dialog's actions their own timeline without
changing their layout. At the default durations, actions wait 150ms and fade for
150ms on entry, and fade for 100ms on exit. Title/body retain their 50ms delay and
200ms fade. The wrapper is optional for arbitrary existing dialog content; use
it when composing the action row. Single dialogs and lower dialogs in a stack
advance the same timelines. The gallery uses it for creation, preferences and
discard confirmation.

Plain menus now fade their rows individually: 250ms per row, with start times
distributed across the first 250ms. Opening above the trigger reverses the row
order. Closing reverses that order again, with 50ms fades starting between 50ms
and 100ms. Separators follow the surface reveal. Reduced motion removes delays;
interrupted entry/exit retargets each retained opacity without resetting it.
These intervals follow the pinned [Material Web dialog animation](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/dialog/internal/animations.ts)
and [menu animation](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/menu/internal/menu.ts).

Rich hints and docked calendars use separate rounded surfaces, a bounded cached
shadow, and a body fade. Their shared desktop popup profile delays the body by
10% of entry duration and fades it over 50%; exit fades it over two-thirds of
exit duration. This is a documented adaptation of the retained popup profile,
not a claim that Android calendars and tooltips share this exact animation.
Closing content gets animation updates with input suppressed, preserving actual
window activity. Picker validation, selection and month/year navigation remain
controlled by application messages.

Search lays out its editor and results at their final dimensions throughout the
surface expansion. The editor moves with the surface without rewrapping at each
frame. A full-screen header is 72px high; a docked header is 56px. Results/divider
wait 75ms then fade for 150ms; the clear action waits 250ms then fades for 50ms.
On exit these fade for 83ms and 42ms respectively. Custom search durations scale
these intervals. The source is the pinned [Android search animation helper](https://github.com/material-components/material-components-android/blob/d12048664f383e88148afb18e971aa6dd24ed42e/lib/java/com/google/android/material/search/SearchViewAnimationHelper.java).
The native editor and arbitrary results are not transformed by Android's 0.95
content scale or its individual icon translation paths. Closing search captures
input without re-focusing the editor or publishing duplicate actions.

Fades are painted against the component's known surface color: iced's public
renderer has no general subtree-opacity operation. Caller-provided opaque child
colors remain caller-owned. Exact mobile choreography, full-screen dialog
subcomponent choreography, and arbitrary nested media effects remain adaptations.

## Carousels

The carousel fits a set of large focal items and smaller edge previews to the
viewport, then interpolates each item's position and mask as it crosses those
reference positions (keylines). Start/end arrangements shift previews out of
the way so the first and last items can be fully revealed. It supports:

- `MultiBrowse`: large items, a flexible medium preview and small previews.
- `Hero`: large items at the start, sized up to twice the carousel height.
- `HeroCenter`: focal items between small previews on both sides.
- `Uncontained`: constant item width with viewport clipping.
- `FullScreen`: one viewport-wide item.

Fitting equations and end-state interpolation are based on Android's pinned
[Arrangement](https://github.com/material-components/material-components-android/blob/d12048664f383e88148afb18e971aa6dd24ed42e/lib/java/com/google/android/material/carousel/Arrangement.java),
[strategies](https://github.com/material-components/material-components-android/blob/d12048664f383e88148afb18e971aa6dd24ed42e/lib/java/com/google/android/material/carousel/MultiBrowseCarouselStrategy.java)
and [keyline shifting](https://github.com/material-components/material-components-android/blob/d12048664f383e88148afb18e971aa6dd24ed42e/lib/java/com/google/android/material/carousel/KeylineStateList.java).
This implementation is horizontal and start-aligned except `HeroCenter`. It
does not implement every Android padding, RTL or vertical strategy option.

Recent pointer velocity affects the snap destination. A stopped/held drag snaps
to the nearest position. Velocity older than 100ms is discarded; projection is
bounded to three additional items. This is a desktop adaptation using a 200ms
settling curve, not Android RecyclerView fling physics or a velocity-preserving
spring. Home/End, arrows, horizontal wheel input and controlled selection remain
available. A fractional end offset maps to the last item when released there.

`carousel(items, selected)` retains eager construction. For large collections:

```rust
use iced_m3::{lazy_carousel, button, Element};
#[derive(Clone)]
enum Message { Select(usize), Open(usize) }
fn collection(selected: usize) -> Element<'static, Message> {
    lazy_carousel(10_000, selected, |index| {
        button(format!("Open item {index}"))
            .on_press(Message::Open(index)).into()
    }).on_select(Message::Select).into()
}
```

The factory builds only the visible range plus a small overscan region. Trees
for overlapping indices retain focus/editor state as the mounted range moves.
Offscreen trees outside overscan are discarded. Store durable values in the
application, and keep indices stable across rebuilds. The factory must be a
side-effect-free view builder: it can be called multiple times. Visible items'
nested overlays are forwarded; invisible items cannot expose overlays. Rounded
masks still require the surrounding surface color through `.background(...)`
when the parent is not the theme surface. The gallery now uses 10,000 lazily
constructed collection items.

## Disabled foreground correction

The dark-mode "Create workspace" report exposed a renderer-specific compositing
bug. The fill used translucent on-surface color, but the label used an opaque
sRGB mix against the page surface. With GPU linear compositing over the raised
dialog, fill and label were only four red-channel levels apart (96–100).
Buttons now pass the actual 38% on-surface foreground to the renderer, matching
the 12% disabled container's compositing context. This applies to labels and
inherited icons across the shared button implementation. The action remains
disabled. [Pinned disabled button tokens](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/tokens/versions/v0_192/_md-comp-filled-button.scss).

See [VALIDATION.md](VALIDATION.md) for executed checks and [ACCESSIBILITY.md](ACCESSIBILITY.md)
for the separate runtime feasibility assessment.
