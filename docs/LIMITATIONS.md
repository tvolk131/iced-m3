# Support and limitations

This is a desktop beta with an evolving API. Full M3 conformance
and native accessibility are not claimed. See [beta validation](BETA_READINESS.md)
for the minimum Rust version and platform checks actually completed.

- Native screen-reader integration is unavailable through iced 0.14's public widget API. Keyboard support is separate: wrap the root in `focus::scope` for Tab/Shift+Tab, focus rings and Enter/Space activation. Navigation groups support arrows; sliders support arrows, Page Up/Down and Home/End. Open dialog/menu/search/modal-sheet panels contain Tab traversal. This is still not an accessibility-complete library. The [feasibility assessment](ACCESSIBILITY.md) explains the runtime hooks needed for a native bridge.
- Pointer interaction is tested on desktop. Full touch/gesture support, RTL and localized calendar/date input remain future work. Navigation groups now use one Tab stop, selected-item entry and automatic scrolling to focused controls.
- Buttons have bounded pointer-origin ripples over a separate hover layer. Menus and tooltips reveal/close; retained dialogs and snackbars enter/exit with translation. These are desktop adaptations, not a port of every Material opacity/shape transition. Shared motion settings include reduced motion.
- Date/time values are app-owned, timezone-free values. Calendar arrows cross months; Page Up/Down move months and Shift+Page Up/Down move years, preserving focus without selecting. Locale-aware formatting and swiping remain future work. Date validators disable endpoints; apps validate the interior of a selected range when necessary.
- Carousels use fitted keylines and bounded velocity-aware snapping. `lazy_carousel(count, selected, builder)` constructs only nearby items; keep durable values in application state. `HeroCenter` adds a centered hero arrangement. Masks use a surrounding surface color (`.background(...)` for custom parents). Exact Android touch/fling physics, RTL and vertical strategies remain adaptations. See [the API and motion details](DESKTOP_FINISH.md).
- Software repainting of large layered surfaces remains expensive in iced 0.14. The dialog shadow optimization reduces some of that cost; the default GPU renderer performs substantially better in the measured scenes. See [measurements and the upstream reproduction](PERFORMANCE.md).
- Rounded clipping of arbitrary children and true group opacity remain renderer integration work. The [tested composition experiments](COMPOSITION.md) preserve editing and live content, but expose performance and color-space costs; those experimental paths are not enabled in the library or gallery.
- Canonical loading shapes and wavy progress are implemented. Expressive action compositions remain previews: button-group width/neighbor deformation, complete size/shape tokens and the broader spring system remain unfinished. [Loading/progress scope and numerical adaptations](LOADING_PROGRESS.md).
- Text tracking is exposed as token data but cannot be applied through iced's public text API. IME/font fallback remains platform-dependent. Retained nested dialogs are supported through `dialog::stack`; keep their entries mounted in stable order.

See [desktop behavior and variants](DESKTOP.md), [the implementation and remaining gaps](COMPLETION.md) and [validation](VALIDATION.md).
