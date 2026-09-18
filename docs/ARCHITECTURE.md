# Architecture and decisions

Current update: [adaptive navigation and catalog completion](COMPLETION.md) supersedes earlier milestone notes about keyboard support, HCT palettes, pickers, sheet dragging, FAB expansion and Expressive previews.

## Boundary

One library crate, two executable examples, and integration tests. The gallery is deliberately ordinary application code consuming the same public API as a downstream user. No application state store, router, background service, animation subscription, or upstream fork is introduced.

The search/filtering milestone adds an anchored Search widget with application-owned
visibility and results, an automatically focused native input, and a persistent
scrollable overlay. Its container interpolates from the bar to docked/full-window
bounds; a separate renderer layer makes the surface opaque to underlying text/SVG.
New chip builders reuse Button with private selection styling and independent
child removal. RangeSlider shares the single slider's scale/geometry calculations
and tracks the active endpoint internally while publishing complete value pairs.
See [SEARCH.md](SEARCH.md) for behavior and scope.

`Element<'a, Message>` is an alias for `iced::Element<'a, Message, iced_m3::Theme>`. Components expose typed variants and builders. Persistent business values remain in the application's model. Transient interaction state belongs to iced's widget `Tree`; `diff` preserves it across view rebuilds. Composition uses iced elements and native catalogs. The initial fixed layout scale is shared through `tokens`, with local builder overrides where they make sense. Runtime theme changes affect colors, elevation and motion; typography/spacing/shape measurements come from the shared scale.

Rendering uses upstream iced's wgpu backend by default, retaining Tiny Skia as a fallback. Software-only builds are available through `--no-default-features`. Release profiling showed that small text changes still incur substantial software painting work when overlapping large shadows and surfaces; dependency optimization alone did not solve native dialog typing lag. No renderer fork or application animation loop is needed for the default GPU path. See [the measurements and native verification](VALIDATION.md).

Tabs and lists default to transparent containers. Tab fills wrap the focus/scroll
group so colors and gradients cover the viewport without moving with labels;
list fills remain ordinary native container styles. Arbitrary-child rounded
clipping and opacity require a different renderer capability. The test-only
band-replay and CPU offscreen experiments establish their cost and blending limits;
neither is linked into production. See [COMPOSITION.md](COMPOSITION.md) for the
native renderer-group proposal and the current carousel/fade compromise.

The loading indicator reads small, baked cubic morph pairs exported by the pinned
Material/AndroidX shape code. Wavy progress measures and trims cubic paths, then
caches the current SVG geometry. Both use widget redraw timestamps and draw-time
color/opacity, preserving pause, reduced motion and background animation beneath
modals. Java is an asset-generation tool only; the Rust runtime adds no dependency.
See [LOADING_PROGRESS.md](LOADING_PROGRESS.md) for source pins and drawing limits.

## Milestone 1 findings

- **Button:** a custom widget accepts passive content, draws rounded quads within the viewport, emits on left release only after a matching press, and cancels on drag away/release or window focus loss. A brief click gets immediate press feedback and an eased fade to hover/idle. Disabled state clears pending presses during reconciliation. Child window events and operations are forwarded; interactive child input events are intentionally not forwarded.
- **Text field:** the actual editing widget is upstream `TextInput`. It handles selection, clipboard, secure entry, key handling, and input method events. The wrapper forwards native operations/overlays and reads native focus to animate the label. Its own cached paragraphs support shaping, a bounded floating-label notch, and wrapping supporting text. Focus and selection survive normal rebuilds, including a validation-message change. Making a field disabled clears focus.
- **Dialog:** `modal(background, dialog, open)` is a persistent root host that retains both child trees while animating visibility. The overlay covers the window, captures input and receives operations instead of the background until closing finishes. The host also gates input as defense in depth. A synthetic window-unfocus event cancels in-flight background gestures; cancellation messages are discarded, while subsequent redraws can finish visual feedback. Child content may expose nested native overlays. `dialog::stack` retains multiple dialogs and restores focus when the top one closes.
- **Sizing:** dialog content scrolls within the viewport with 24px outer margins (smaller on tiny windows). Actions remain reachable when content is tall. A slim scrollbar reserves its own width so it does not obscure the right edge of fields or cards. The gallery uses a persistent wrapping-row structure with responsive child widths, so narrow layouts remain scrollable and app values survive resizing.
- **Scheduling:** a small finite transition primitive retargets from its current interpolated value. Buttons, checkboxes, field labels, and switch thumbs request frames only during active transitions. There is no global timer. Theme motion settings are cached from the last draw in widget-tree cells because iced does not pass a theme into widget event updates. State starts with the same default timings before the first draw.

The available released APIs were sufficient for these interactions. No upstream patch was required.

## Deliberate visual compromises

Material 3 is the visual reference; Material UI informs naming, variants and composition rather than default visuals. This is a Material-inspired preview, not a certified pixel-perfect implementation of every specification.

A pointer-origin circular ripple clipped by an arbitrary rounded shape would require a more complex path/mesh or canvas rendering implementation: iced's renderer layer clipping is rectangular. This release uses a rounded tonal state layer matching the button silhouette. It gives a visible animated press/release effect without leaking through corners, and the same action widget serves icon buttons and chips. A radial ripple can be added behind this API later without requiring application animation messages or a framework fork.

The accent generator targets Material-style luminance tones using sRGB interpolation, rather than importing an HCT implementation. Foreground/background role contrast is checked for extreme and saturated accent inputs. HCT generation can replace this function later; semantic role names remain stable.

The shape, spacing and typography scales are fixed centrally in 0.1. Full runtime density/typography replacement and per-component class catalogs can be added after actual downstream use demonstrates the needed variation. Current overrides cover sizes, padding, radius, typography font/color, field background and public semantic colors.

## Milestone 2

Typography helpers return native text. All 15 baseline M3 roles select bundled Roboto Regular or Medium, registered once through iced’s public shared font system before layout. Fields use the same font for native input and cached label/supporting paragraphs. Native iced text can opt in with the application default font. Tracking is documented but cannot be applied through iced 0.14’s native text APIs. Surfaces return styled native containers. Dividers use theme catalogs. Icon buttons and chips reuse Button. Badges are compact styled containers. Switches originally embedded a drawing widget in Button; the navigation milestone moves them to the shared selection-control widget described below. Disabling controls consistently omits the application action. The enabled switch/checkbox values remain controlled by the application.

Checkboxes now have a dedicated widget: an 18px box, 40px circular state layer,
48px target, native text label, and independently retargetable selection, mixed
mark, hover and press transitions. Label and target clicks share release/cancel
semantics. Disabling snaps visual state and cancels the gesture. Indeterminate and
error appearance are controlled builder inputs. A compatibility catalog remains
for applications composing native iced checkboxes, but that native path does not
have the new animation.

Checkbox and radio press feedback separates expansion from release opacity.
The centered ripple uses Standard easing over `motion.ripple_expand` (450 ms),
remains visible while held, and fades using `motion.short` (150 ms). A quick
release retains feedback until half the expansion interval has elapsed; message
delivery is never delayed. Cancellation bypasses that minimum and disabling
clears the effect. The 8% hover circle remains visible beneath the separate 12%
press ripple, following Material Web's independent hover and press layers. Release
fades only the ripple while the pointer remains over the control. Completed or
invisible ripples stop requesting frames. Switch thumb feedback uses its existing
timings.

Checkmarks and gallery icons use cached SVG handles through iced’s SVG renderer.
This path handles ancestor scroll transforms and clipping consistently at 2×
scale. Tiny Skia 0.14’s canvas geometry path applies a group clipping transform
twice and composes display-scale translation incorrectly under scrolling. Using
SVG avoids that path without modifying upstream iced. Checkmark paths are rebuilt
only while their shape changes; colors and opacity remain draw-time parameters. The application does
not receive animation messages. See [the visual audit](MATERIAL_AUDIT.md) for the
reference comparison and remaining fidelity work.

## Everyday workflow milestone

Menus and plain tooltips share `anchored` placement. It converts ancestor scroll
translations to window coordinates, flips when the opposite side has more room,
and clamps to the viewport. Menus reuse Button rows and iced's native scrollable;
opening state belongs to the trigger's tree. A private trigger message opens the
popup without adding application messages. Actions close it and publish ordinary
application messages. Select maps typed options onto the same menu. Right-click
context menus wrap interactive content; action-menu triggers accept passive
content. Escape closes a nested menu before its containing dialog.

Plain tooltips forward input to their content. They schedule one delayed redraw,
then remain idle while visible. Press/Escape suppression prevents a dismissed
hint immediately reopening under a stationary pointer. Arrow icons use a cached
SVG, following the earlier icon alignment/scrolling fix.

Radio buttons reuse the checkbox control's gesture handling, label layout, hover
and press transitions with a 20px circular ring and animated 10px dot. They suppress
an action on an already selected value. Groups only arrange controlled radios;
there is no independent hidden group selection.

The snackbar host preserves its background tree. A notice identity (ID, text and
duration) resets the timer; ordinary application rebuilds do not. The host requests
a future deadline and pauses elapsed time while hovered or unfocused. Actions
and timeouts publish once. Pointer events over the bar are forwarded to the
background with an unavailable cursor and discarded messages, so covered content
clears an earlier gesture without clicking through. Keyboard editing remains
functional while hovering the bar. The outer modal host suspends the snackbar
with its existing synthetic Unfocused event and now sends a matching Focused event
when closing in an active window. This resumes deadlines without restoring field
focus or replaying input.

The host draws the complete snackbar in a separate foreground renderer layer.
This preserves ordering between the background's text batches and the notice's
surface; call order within a shared layer alone does not guarantee occlusion.
A pixel regression verifies that adding text behind the bar cannot change its
interior on either tested backend.

Inverse surface/on-surface/primary roles support tooltip and snackbar colors in
both themes and are included in contrast tests. No dependency or upstream fork
was added. These components retain the agreed mouse-first scope. Flat menus,
plain tooltips, a single application-owned notice and immediate popup entrance/
exit keep this milestone bounded. See [WORKFLOWS.md](WORKFLOWS.md).

## Navigation and structured content milestone

Tabs compose action buttons inside a small custom strip widget. The strip owns
only indicator geometry and finite transitions; selected values stay in the app.
It forwards operations and overlays and stores positions relative to its layout,
so indicator drawing survives native scroll translations. Primary indicators are
3px with rounded top corners; secondary indicators span the tab at 2px. Native
horizontal scrolling handles natural-width overflow. Initial selection snaps;
subsequent selection/geometry changes animate with `motion.medium`.

List rows use a private interactive-content mode in Button. Children receive
input first; a child capture/message or interactive pointer target suppresses the
row action. Crossing between the row and its trailing control cancels a gesture.
Child overlays and native operations are forwarded. Disabling a row rebuilds its
child tree to cancel transient input and blocks child events and overlays; custom
children still own their visual disabled state. Public Button content remains
passive. A static list row keeps its children enabled. Headline/supporting/overline
slots select a 56/72/88px minimum height and can grow for wrapped text.

App bars use native layout with a clipped Title Large slot that leaves actions
reachable under a long title. Scroll elevation is an explicit app-owned Boolean.
The 80px rail uses native vertical scrolling and a separate footer. Its action
feedback is drawn over the 56×32 icon container so selected items retain visible
hover feedback. The gallery owns routing and swaps between a rail and top tabs at
700px; page values and preferences survive view rebuilds and resizing.

Switches now share the checkbox/radio widget's label layout and gesture handling.
Their 40px halo follows the moving thumb, leaving labels untinted. Selection uses
`motion.medium`; hover and pressure use `short`, with a 28px pressed thumb. All
transitions stop scheduling frames when settled. Icon buttons still return Button,
but use their own color rules for standard/outlined/filled/tonal actions and
toggles, including `surface_container_highest` for unselected filled toggles.

No dependency, global animation subscription or upstream fork was added. See
[NAVIGATION.md](NAVIGATION.md) for reference tokens and bounded behavior.

## Visual regression testing

The private `motion::now` function reads the system clock in production. Library
unit tests can scope a thread-local clock override with automatic restoration.
The shared headless harness supplies explicit event timestamps, preserves the
real widget cache across rebuilds and draws frames without dispatching an extra
wall-clock redraw. This makes mouse-down and intermediate transition frames
reproducible without exposing test controls in the public library API.

Reference comparison uses a development-only PNG dependency. Exact software
pixels are checked in a dedicated canonical-environment CI job, while ordinary
behavioral and rendering-property tests remain cross-platform. The complete
coverage, review process and limits are in [VISUAL_TESTS.md](VISUAL_TESTS.md).

## Value controls and progress milestone

Sliders own only drag state, message deduplication and hover/press transitions.
The application owns their values. Label paragraphs use native text with reserved
bubble space. Finite ranges are calculated in f64 before returning f32 values to
avoid overflow for large but valid ranges; invalid ranges remain inert.

Segmented buttons reuse Button's input handling, with private corner and animated
selection-color settings. A shared foreground layer draws the outline/dividers
after the child button layers. Selection is a typed single/multiple value owned
by the application. Passive icons are replaced by a fixed-size check slot.

Progress indicators use redraw timestamps and the same private test clock as
finite motion. Determinate progress settles; indeterminate progress schedules
frames only while visible, focused and unpaused. Paused intervals are omitted
when resumed. Linear drawing uses quads; circular arcs use cached SVG handles,
following the existing scroll-safe glyph path. No timer dependency, runtime
subscription or upstream fork was introduced. Gallery work is a separate,
cancellable simulation with generation-checked application messages.

See [scope, reference sources and usage](VALUES.md). The existing mouse-first
accessibility boundary remains in effect.

## Reuse review (2026-09-12)

| Project | Inspected source/license | Decision |
| --- | --- | --- |
| iced | Released 0.14.0; MIT. Local registry sources for `iced_core` 0.14.0, `iced_widget` 0.14.2, `iced_test` 0.14.0. | Direct dependency. Reuse native editing, layout, rendering, scrolling and headless testing. Pin iced itself to 0.14.0 and retain Cargo.lock. |
| iced_aw | [Main Cargo.toml](https://raw.githubusercontent.com/iced-rs/iced_aw/main/Cargo.toml) declares MIT and iced 0.15.0-dev dependencies/patches. | Useful component organization reference; inspected main is not a compatible dependency for this release. No code copied. This is not a claim that every older iced_aw release is incompatible. |
| libcosmic | [Cargo.toml](https://raw.githubusercontent.com/pop-os/libcosmic/master/Cargo.toml) uses `./iced`; [LICENSE](https://raw.githubusercontent.com/pop-os/libcosmic/master/LICENSE) is MPL-2.0. | Its fork/platform integration exceeds this project's upstream-only scope. No code copied. |

`Cargo.lock` records exact transitive versions. The package is `iced-m3` (Rust import `iced_m3`); see [release preparation](RELEASING.md) for publication status and compatibility expectations. Original code is MIT licensed. The unmodified bundled Roboto font is SIL OFL 1.1 licensed; its license and provenance are retained in `assets/fonts/`.

## Future focus support

Custom widgets already participate in iced's tree and operation traversal. The modal host distinguishes foreground and background operations. This leaves a place for explicit focus IDs, keyboard activation and focus management, without adding a partial accessibility framework now. The preview does not claim screen-reader support, complete keyboard reachability, focus trapping/restoration, dedicated high-contrast or reduced-motion modes.

## Sheets and primary actions milestone

`sheet::host` retains background and content trees even while closed. Standard
side sheets lay out beside the background and animate its reserved width; modal
side/bottom sheets use a root overlay and animate translation plus scrim opacity.
Both use the existing retargetable `Transition` with standard easing and dedicated
theme entrance/exit durations. Closing disables content input immediately while
retaining its drawing until exit completes. Modal background input stays blocked
through exit. Explicit cancel/resume flags prevent covered gestures from firing
and suspend/resume background timers. Focus events from an enclosing dialog also
reach the sheet's content even when its overlay is hidden. Renderer layers isolate
the opaque sheet surface from background text/SVG batching.

`Fab` is a typed builder over `Button`, with size, semantic color and optional
label. The button adds a private elevated treatment, retaining the same gesture
cancellation, finite hover/release and passive-child contract. Its shadow extends
outside the hit box but remains clipped to the ancestor viewport. Added tertiary
and surface-container-low roles use the existing sRGB tonal approximation; no
runtime dependency, framework fork or public animation messages were added.
See [SHEETS.md](SHEETS.md) for the API contract and intentional limits.
