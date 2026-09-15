# Native accessibility feasibility on iced 0.14

Assessment date: 2026-09-14. The dependency stays at released iced 0.14.0.
Keyboard navigation, focus rings and reduced motion are implemented, but this
library does **not** currently expose a native screen-reader tree.

## What the pinned release provides

The installed `iced_core` Widget/Operation interfaces can traverse containers,
focusable controls, text inputs, text and custom state. A library can use
`Operation::custom` to collect its own semantic metadata. There is no standard
role/name/value/action tree, native accessibility action dispatch or AccessKit
integration in this release's widget/runtime paths; Cargo.lock has no AccessKit
dependency. Focus traversal alone does not supply those semantics.

`iced::window::run` does expose a callback with `&dyn Window`, but this is a trait
limited to raw window/display handles. It does not expose the `winit::Window`,
active event loop, or pre-dispatch native window-event hook. A raw handle is a
potential platform integration point, not a complete accessibility bridge.

These findings come from the installed sources: `iced_core-0.14.0/src/widget.rs`,
`iced_core-0.14.0/src/widget/operation.rs`, `iced_runtime-0.14.0/src/window.rs`
(`Window` and `run`), and `iced_winit-0.14.0/src/lib.rs` (`Action::Run` dispatch).
They describe the pinned release; they do not make a claim about every newer
iced branch or proposal.

## Available approaches

The [AccessKit winit adapter](https://docs.rs/accesskit_winit/0.34.0/accesskit_winit/struct.Adapter.html)
requires a winit window plus active event loop at initialization, before the
window is first shown, and native window events before the application handles
them. Its callbacks also need synchronized action routing and semantic-tree
updates. Merely adding this crate to iced-m3 does not expose those hooks.
AccessKit also offers separate platform adapters, so an application-specific
bridge remains technically possible. See [AccessKit's integration overview](https://accesskit.dev/).

| Approach | Feasibility and cost |
| --- | --- |
| Widget-library-only metadata | Feasible with custom operations for our widgets, but cannot by itself deliver OS integration, native text semantics or support for arbitrary third-party widgets. |
| Runtime integration / an upstream-supported release | Preferred path: give the runtime native adapter lifecycle/event hooks, standardized semantic traversal and action dispatch. Requires changes outside this component library; evaluate an upstream release or explicit runtime work separately. |
| Separate platform bridge using raw handles | Possible investigation path, especially for a macOS prototype; requires native lifecycle/thread safety, adapter initialization, actions, IME/text mapping, focus and shutdown handling. No working cross-platform prototype has been validated here. |

The raw-handle route may require unsafe/native integration code. This crate
currently forbids unsafe code, so a bridge would need a separately reviewed
integration boundary rather than weakening the component crate's policy.
The preferred next step is a small runtime integration proof with one button,
one checkbox and one text input, before adding metadata to every component.

## Acceptance gates for that proof

1. VoiceOver discovers the first window's tree immediately, with stable IDs,
   labels, roles, bounds, enabled/selected state and focus.
2. Screen-reader activate/toggle/edit actions dispatch exactly one application
   message and update the announced value.
3. Native text input exposes value, selection, editing and IME behavior; visual
   text alone is insufficient.
4. Dialog and nested-popup focus order matches the visible modal layer, with
   covered content excluded from accessible interaction and correct restoration.
5. Lazy carousel recycling exposes the visible items with stable logical IDs,
   correct position/count and a route to reveal requested offscreen items.
6. Window close/reopen, multiple windows, resize, scale factor and focus changes
   remain safe. Repeat the core flow with NVDA on Windows and Orca on Linux.

No screen-reader test has been run in this milestone, and no native
accessibility implementation is being claimed. This gap does not block the
completed adaptive navigation or desktop visual/interaction work.
