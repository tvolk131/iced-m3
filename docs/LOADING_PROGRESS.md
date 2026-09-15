# Canonical loading and wavy progress

This milestone replaces the loading indicator's procedural approximations with
the canonical Material shape morphs and adds opt-in wavy linear/circular progress.
The existing flat progress profile stays available. No renderer fork, offscreen
readback or runtime dependency was added.

## Use

```rust
use iced_m3::{Element, linear_progress, circular_progress, loading_indicator};

let bar: Element<'static, ()> = linear_progress(0.45).wavy(true).into();
let ring: Element<'static, ()> = circular_progress(0.0)
    .wavy(true).indeterminate(true).into();
let loading: Element<'static, ()> = loading_indicator().contained(true).into();
```

The application still owns the measured value and loading state. Values are
clamped in the same way as flat progress. `buffer` works with measured linear
waves; `colors` and `track_color` work with both kinds. `.wavy(false)` returns to
the baseline drawing/motion profile. Configuration order does not change which
wave defaults apply.

| Setting | Default and behavior |
| --- | --- |
| `wavy(true)` | Enable the Expressive drawing and loading profile |
| `size(px)` | Linear stroke thickness, or circular allocated diameter; defaults 4 / 48px |
| `wave_amplitude(px)` | 3px linear / 1.6px circular; zero flattens the wave; circular amplitude is bounded inside its radius |
| `wavelength(px)` | 40px measured linear / 20px loading linear / 15px circular; fitted to a whole number of waves |
| `wave_speed(px_per_second)` | 0, matching the reference default; positive and negative values enable opposite travel directions |
| `paused(true)` | Freeze loading and wave travel; application changes to measured progress still take effect |

Linear layout reserves `thickness + 2 × amplitude`: the default wave is 10px
high. Its inactive track stays flat and its stop marker stays at the end. A
circular wave bends inward from the track radius, preserving its allocated size.
Circular wavy strokes are 4px (bounded for extremely small diameters); changing
the diameter does not scale the stroke. The wavy profile uses `primary` for the
active stroke and `secondary_container` for the inactive track, following the
pinned token file. Both can still be overridden.

Wave shape near completion follows the source's amplitude range: full amplitude
between 10% and 90%, transitioning flat outside it over 500ms with the shared
motion scaling. At 100%, the settled indicator is a full flat bar/ring. Stationary
measured indicators stop requesting frames. Optional traveling waves continue
only while visible, active, unpaused, permitted by reduced-motion settings and
showing a nonzero wave.

## Shape and motion references

Reference revision: Material Components for Android
`d12048664f383e88148afb18e971aa6dd24ed42e`, token version 34.0.0. This is a pinned,
auditable profile, not a claim to follow future changes automatically.

The loading sequence is **soft burst → cookie 9 → pentagon → pill → sunny →
cookie 4 → oval**. The original shapes and pairwise feature matching are exported
by the actual AndroidX graphics-shapes 1.0.1 code. The runtime interpolates matched
cubic controls, retaining rounded contours through each morph. Its shape size is
34px inside a 48px container, and its initial rotation is -90°. Contained loading
uses `on_primary_container` on `primary_container`; uncontained uses `primary`.
`.size(...)` scales that entire composition.

Every 650ms the morph spring advances its target by one, with stiffness 200 and
damping ratio 0.6. Rotation combines 50° of constant movement per interval with
90° per spring-driven morph. An analytical spring response preserves velocity
between targets and remains independent of frame frequency. Android's spring
driver has discrete frame/settling thresholds; this implementation evaluates the
continuous response instead. Shape provenance, generation and Apache-2.0 notices
are in [assets/loading/README.md](../assets/loading/README.md).

Wavy linear loading uses Android's **1800ms disjoint** profile and its four
independent endpoint easing curves. Each segment grows, then contracts; two may
be visible during the handoff. It finishes the remainder of that cycle before
switching to measured progress. Multicolor changes happen at the empty boundary.
The baseline flat variant retains its existing 2000ms Material Web profile.

Wavy circular loading uses the **6000ms retreat** profile: one active section
grows from 10% to 87% over three seconds and retreats over the following three,
with continuous rotation and four accelerated spins. The inactive track remains
visible. Its completion handoff contracts over 500ms, then lets the existing
measured spring settle. Flat circular progress retains its advance profile.

Primary sources at that revision:

- [Loading geometry](https://github.com/material-components/material-components-android/blob/d12048664f383e88148afb18e971aa6dd24ed42e/lib/java/com/google/android/material/loadingindicator/LoadingIndicatorDrawingDelegate.java), [spring/rotation](https://github.com/material-components/material-components-android/blob/d12048664f383e88148afb18e971aa6dd24ed42e/lib/java/com/google/android/material/loadingindicator/LoadingIndicatorAnimatorDelegate.java), [loading tokens](https://github.com/material-components/material-components-android/blob/d12048664f383e88148afb18e971aa6dd24ed42e/lib/java/com/google/android/material/loadingindicator/res/values/tokens.xml).
- [Linear drawing](https://github.com/material-components/material-components-android/blob/d12048664f383e88148afb18e971aa6dd24ed42e/lib/java/com/google/android/material/progressindicator/LinearDrawingDelegate.java), [circular drawing](https://github.com/material-components/material-components-android/blob/d12048664f383e88148afb18e971aa6dd24ed42e/lib/java/com/google/android/material/progressindicator/CircularDrawingDelegate.java), [progress tokens](https://github.com/material-components/material-components-android/blob/d12048664f383e88148afb18e971aa6dd24ed42e/lib/java/com/google/android/material/progressindicator/res/values/tokens.xml).
- [Linear endpoints](https://github.com/material-components/material-components-android/blob/d12048664f383e88148afb18e971aa6dd24ed42e/lib/java/com/google/android/material/progressindicator/LinearIndeterminateDisjointAnimatorDelegate.java), [circular retreat](https://github.com/material-components/material-components-android/blob/d12048664f383e88148afb18e971aa6dd24ed42e/lib/java/com/google/android/material/progressindicator/CircularIndeterminateRetreatAnimatorDelegate.java), [measured amplitude transitions](https://github.com/material-components/material-components-android/blob/d12048664f383e88148afb18e971aa6dd24ed42e/lib/java/com/google/android/material/progressindicator/DeterminateDrawable.java).

## Rendering, scheduling and practical limits

The wave path uses cubic halves with the reference 0.48 tangent control length.
Curves are measured with 32 subdivisions per cubic, then split at the requested
arc-length endpoints; the final draw retains cubic curves. Circular anchors use
analytic circle positions instead of measuring Android's cubic circle. These
are small numerical drawing adaptations, not a port of Android's entire PathMeasure
or stroke renderer. Extremely narrow tracks retain a whole wave, and very short
sections use a shrinking rounded mark. Custom pathological amplitudes/wavelengths
are bounded to avoid inverted circular radii and unbounded path counts.

Each widget caches its current SVG geometry; color/alpha are applied at draw
time. Updates use actual widget redraw timestamps. Pausing or window inactivity
discards the inactive interval on resume. Covered widgets continue advancing
behind a dialog/sheet because modal input isolation does not mark the window
inactive. Native text editing and overlay focus behavior are unchanged.

Reduced motion holds a static loading pose, suppresses wave travel and settles
measured transitions immediately. This milestone does not add native screen-reader
semantics, all progress show/hide animations, every alternative indeterminate
delegate, or the remaining Expressive button/group/slider size and shape system.

## Review and tests

The gallery's **Workspace → Components → Loading with expression** card offers
loading/measured switching, pause, wave travel, a progress slider, loader sizes
and both wave kinds. Tests compare all seven shapes and morph midpoints against
independently exported AndroidX SVGs, and cover wave layout/gaps, endpoints,
clock-origin independence, pause/resume, reduced motion, modal coverage and
completion to idle. Exact timed reference frames cover light/dark motion,
completion handoffs, endpoint values, custom waves and narrow/wide gallery layouts.

```sh
cargo test --locked --lib loading_progress
ICED_TEST_BACKEND=wgpu cargo test --locked --lib loading_progress
cargo test --locked --no-default-features --all-targets visual_references -- --ignored
cargo test --locked --no-default-features --lib capture_loading_progress_preview -- --ignored
python3 tests/visual/loading-preview.py
```

The local motion player replays 30fps captures from the actual renderer; it does
not interpolate missing frames. Reproduce performance measurements separately
with `profile_loading_progress` in release mode. They include screenshot readback,
so they compare paths on this host rather than promise native-window frame rates.
Executed results and image counts are recorded in [VALIDATION.md](VALIDATION.md).
