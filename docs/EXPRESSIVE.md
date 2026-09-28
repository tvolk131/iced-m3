# Expressive actions and motion

Material 3 design guidance is the target. Official platform implementations supply
pinned numerical references where the guidance does not specify the algorithm.
This is the first shared-spring and action-recipe milestone, not a claim of full
Material 3 Expressive conformance.

## Theme defaults and component choices

```rust
use iced_m3::{Theme, MotionScheme, ButtonSize, ButtonShape, Element, button};

let theme = Theme::dark().expressive();
// The same action shapes with more restrained spatial springs:
let restrained = theme.clone().motion_scheme(MotionScheme::standard());
let action: Element<'_, ()> = button("Create")
    .size(ButtonSize::Medium)
    .shape(ButtonShape::Square)
    .on_press(())
    .into();
```

Return the theme from iced's application `.theme(...)` callback. There is no new
wrapper. `Theme::expressive()` selects Expressive springs and enables default
press/selection shape feedback for eligible action buttons. It preserves colors,
fonts, sizes, variants, duration settings and the reduced-motion preference.
It does not make every component larger or turn every progress bar into a wave.
`Theme::motion_scheme(...)` changes only the springs; both Standard and Expressive
use springs, with more spatial overshoot in Expressive.

- A button's `.expressive(true/false)` overrides the theme's shape-feedback default.
  `.shape(...)` also opts into shape feedback unless explicitly disabled.
- `.motion_scheme(...)` on a button overrides the shared spring scheme. Shape
  feedback and group deformation use Expressive springs when no scheme is set.
- Explicit dimensions and padding override recipe defaults. Arbitrary child
  widgets keep their own typography and artwork size.
- `.reduced_motion(true)` wins over shared and component motion choices. It stores
  a preference separately, so `.reduced_motion(false)` restores custom settings.
  Custom widgets should read `theme.effective_motion()`, not `theme.motion`.

Ordinary `Theme::light()` and `Theme::dark()` keep the existing duration-based
motion profile. Opt in with `.expressive()` or `.motion_scheme(...)`. Existing
`.expressive(true)` action compositions now use spring shape feedback.

## Action recipes

`button(label).size(...)` coordinates the container, padding, label typography,
outline width and rounded-square/pressed corners. `Button::new(content)` accepts
custom passive content; it does not rewrite that content's font or artwork.

| Size | Height | Label role | Common-button icon | Icon-button icon |
| --- | ---: | --- | ---: | ---: |
| ExtraSmall | 32 | LabelLarge (14/20) | 20 | 20 |
| Small | 40 | LabelLarge (14/20) | 20 | 24 |
| Medium | 56 | TitleMedium (16/24) | 24 | 24 |
| Large | 96 | HeadlineSmall (24/32) | 32 | 32 |
| ExtraLarge | 136 | HeadlineLarge (32/40) | 40 | 40 |

Dimensions are logical pixels. `ButtonShape::Round` uses rounded ends;
`ButtonShape::Square` uses size-specific rounded corners. Selected toggle actions
use the opposite shape. Explicit radius/corner overrides retain their geometry.
These are visible container sizes, not a new invisible touch-target policy.

```rust
use iced::widget::svg;
use iced_m3::{ButtonSize, IconButtonWidth, icon, icon_button, Element};
let size = ButtonSize::Medium;
let action: Element<'_, ()> = icon_button(
    icon(svg::Handle::from_path("assets/add.svg"))
        .size(size.icon_button_icon_size()),
)
.size(size)
.icon_width(IconButtonWidth::Wide)
.on_press(())
.into();
```

Icon-only actions expose Narrow, Default (square) and Wide widths. Supply artwork
at `ButtonSize::icon_button_icon_size()`; neither the recipe nor its width setting
rescales arbitrary widgets. The component centers the widget's layout bounds,
which need not center a text glyph's visible ink. Prefer `icon(...)` with SVGs.

Button groups retain child state and focus while the pressed button grows and
its immediate neighbors compress. Requested expansion defaults to 15% of resting
width. The target is limited by neighboring padding before applying the spring,
so motion keeps its overshoot and release rebound. Overshoot can consume spare
padding on the opposite side; simultaneous neighbor animations share the total
padding budget without reducing the label's original available space.
`.expanded_ratio(0.0)` disables width deformation. Connected groups use
2px gaps, 8px resting inner corners, 4px pressed inner corners and rounded selected
items; ordinary groups use 12px gaps. Release, cancellation and interruption
retarget the same springs. Groups currently allocate equal resting widths and do
not implement custom weights or automatic overflow menus.

## Motion behavior and coverage

`MotionScheme` has Fast, Default and Slow roles for both spatial movement and
color/opacity effects. Their stiffness and damping fields are customizable through
`Spring::new`. The analytic unit-mass spring evaluates elapsed time independently
of frame rate. Retargeting preserves position and velocity. Preset effects are
critically damped; spatial springs may overshoot. Bounded layout/opacity consumers
clamp their output where negative size or invalid opacity would be inappropriate.

Dialog content and paper use sequential ranges of one reversible effects spring:
content is absent whenever the rounded panel is translucent. This preserves the
existing content-cover compositing approach without exposing its rectangular
mask during exit, and does not add a second spring's settling delay.

Switches use Fast spatial springs for thumb position and diameter, and Fast
effects for selection colors. Spatial movement retains its small overshoot beyond
the resting endpoints; only physical track containment limits extreme custom
springs. Pressing snaps the thumb into its enlarged shape; releasing or cancelling
springs it back, preserving velocity when a transition is retargeted. This follows
the [official Compose switch's motion treatment](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Switch.kt).
The baseline duration-based theme retains its existing press/selection behavior,
and reduced motion settles immediately.

Shared schemes currently drive button state/shape feedback, selection movement,
slider press/label feedback, floating field labels, tab indicators, chip icon
replacement, rail/extended-label reveals, search expansion and overlay presence.
Existing ripple growth/release timing and continuous progress/loading cycles keep
their dedicated algorithms. Not every finite transition has been converted to a
spring, and adopting the scheme does not assert platform-identical motion for
every component. A nonzero legacy duration enables a spring; its exact duration
no longer sets that spring's settling time. A zero duration still disables it.

Run the focused comparison with:

```sh
cargo run --release --example expressive
```

The Standard/Expressive, reduced-motion and light/dark controls stay visible while
you scroll. Each demo owns its state, including separate selections for the spaced
and connected button groups.

- **Button groups:** hold a button, release early, then press its neighbor. Compare
  expansion, neighbor compression, shape changes and release rebound.
- **Tabs:** jump between the first and last tabs and reverse before the moving
  indicator settles.
- **Navigation rail:** expand and collapse while it is moving. The neighboring
  content follows the animated width.
- **Extended FAB:** hide and restore its label. The reveal uses the shared spring,
  but bounds its visible size rather than overshooting the collapsed/expanded ends.
- **Switch and slider:** hold the switch to inspect thumb feedback; drag the slider
  to compare handle and value-bubble feedback. Slider values follow your pointer.
- **Dialog and action recipes:** compare reversible dialog presence and all five
  button/icon sizes, including a button with shape feedback explicitly disabled.

The effect is deliberately different across these demos: a tab indicator can
overshoot, while dialog opacity stays within its valid range. Ripples and looping
progress indicators still use their dedicated timing, so they are not included as
spring comparisons.

Remaining Expressive work includes theme typography/shape schemes, complete FAB
and split-button size families, richer toolbar/FAB-menu behavior, current slider
and navigation variants, and a broader component-by-component fidelity audit.
The existing general accessibility, locale and renderer boundaries still apply.

## Reference profile

Checked September 27, 2026. Design guidance:
[Expressive motion theming](https://m3.material.io/blog/m3-expressive-motion-theming)
and [building with Expressive](https://m3.material.io/blog/building-with-m3-expressive).

Numerical reference: AndroidX Material 3, commit
[`8fd64ac21a546397597caa1527fd2e589ea2915e`](https://github.com/androidx/androidx/tree/8fd64ac21a546397597caa1527fd2e589ea2915e/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3).
Motion tokens are `v0_14_0`; common-button tokens are `v0_11_0`; connected small
group tokens are `14_1_0`. The pin also supplies icon-button tokens, label-role
mapping and group expansion/padding guidance. These are implementation/token
versions, not a version number for the living Material design website.

| Scheme / role | Fast stiffness, damping | Default | Slow |
| --- | --- | --- | --- |
| Standard spatial | 1400, 0.9 | 700, 0.9 | 300, 0.9 |
| Expressive spatial | 800, 0.6 | 380, 0.8 | 200, 0.8 |
| Both effects | 3800, 1.0 | 1600, 1.0 | 800, 1.0 |

The Rust spring solver and iced layout integration are independently implemented.
AndroidX sources are Copyright The Android Open Source Project, Apache-2.0.
