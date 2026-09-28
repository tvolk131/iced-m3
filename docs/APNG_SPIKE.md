# Component documentation APNG size spike

Measured 2026-09-27 against component source at `3eb015c`, with the added
original APNG probe. This document preserves historical feasibility measurements.
The spike runner and Python report have been retired; the production renderer,
encoder round-trip tests, and Rust workflow now live in the
[executable documentation pipeline](DOC_MEDIA.md).

## Result

Lossless APNGs are practical for compact component examples. Large animated
surfaces need an explicit budget: the dialog accounts for about 61% of this
five-example sample at 2×/30 fps. Rendering every component, variant, and theme
as a separate high-resolution animation would not be cheap.

Each animation lasts three seconds. Sizes below are KiB (1,024 bytes), light
theme; 1× and 2× are actual renderer output scales, not resized screenshots.

| Example | Logical dimensions | 1× / 30 fps | 1× / 60 fps | 2× / 30 fps | 2× / 60 fps |
| --- | --- | ---: | ---: | ---: | ---: |
| Switch | 300×96 | 37.5 | 63.9 | 75.9 | 130.7 |
| Button group | 440×104 | 102.1 | 183.1 | 236.5 | 435.1 |
| Floating-label field | 360×144 | 61.3 | 89.3 | 153.2 | 222.8 |
| Dialog open/close | 480×360 | 473.7 | 836.2 | 1192.9 | 2175.3 |
| Loading indicator + circular progress | 200×112 | 151.9 | 303.2 | 294.9 | 588.2 |

Five-example light-theme totals:

| Configuration | Raw APNGs | Gzipped tar containing the APNGs |
| --- | ---: | ---: |
| 1× / 30 fps | 0.81 MiB | 0.77 MiB |
| 1× / 60 fps | 1.44 MiB | 1.38 MiB |
| 2× / 30 fps | 1.91 MiB | 1.81 MiB |
| 2× / 60 fps | 3.47 MiB | 3.30 MiB |

These are media-only sizes, not the resulting crate's total size. Our existing
package checker has a project-specific 5,000,000-byte compressed limit; a full
catalog would need a measured budget before changing that limit.

Dark-theme 2×/30 fps sizes were similar: switch 79.5 KiB, group 223.9 KiB,
field 154.9 KiB, dialog 1102.1 KiB, loading pair 319.9 KiB. No general claim
about all themes/components is implied by these five samples.

## Method

- Real iced `UserInterface`, TinySkia, bundled Roboto, Expressive motion, and the
  existing virtual animation clock. Rust 1.92.0 on this Apple Silicon host.
- Capture at 60 Hz; the 30 fps encodings use every other sample from that same
  timeline. The underlying widget/event simulation is identical at both rates.
- Switch and group receive hover at 300 ms, press at 500 ms, release at 800 ms,
  and pointer departure at 1200 ms. Switch resets at 1900 ms.
- Field gets a programmatic value at 500 ms and clears at 1900 ms. This tests its
  floating-label transition without an independent native caret clock.
- Dialog opens at 500 ms and closes at 1900 ms, including its scrim and shadow.
- Loading sample runs continuously; its three-second clip is not selected to
  match a seamless natural animation cycle.
- PNG 0.18.1 `Balanced`, full RGBA8. No quantization or lossy processing.
- Optimized encoding stores the bounding rectangle of changed pixels using APNG
  SOURCE/NONE composition, and merges identical consecutive frames by extending
  their duration. Idle time does not cost repeated full images.
- Both optimized and full-frame encodings are decoded and independently
  composited. Every original sampled pixel and frame duration must round-trip.

For comparison, the switch at 2×/30 fps was 939.0 KiB with full-frame encoding
versus 75.9 KiB optimized. Dialog was 3398.2 versus 1192.9 KiB. A continuously
changing loading sample benefits less: 369.9 versus 294.9 KiB.

A data-URI HTML fragment for each 2×/30 fps image totals 2.54 MiB before
compression and approximately 1.84 MiB when each fragment is gzipped. This
measures embedding overhead only; docs.rs/rustdoc integration was not tested in
this size spike. Avoid shipping both separate APNGs and embedded copies without
a reason.

## Recommendation

Use one compact animation for the behavior each component needs to demonstrate,
with static images for additional themes and states. Keep 60 fps available for
spring-motion examples, where temporal detail matters. Use a smaller viewport,
1× output, or a deliberately reviewed larger budget for dialogs/sheets. Choose
30 versus 60 fps after inspecting the comparison page, not by file size alone.

The sizes support proceeding with the proposed example-driven docs pipeline.
They favor release-generated media if avoiding repeated binary additions to Git
is the priority, but compact selected animations could also reasonably be
committed. This sample is not enough to extrapolate a reliable full-catalog size.

## Current workflow

The original runner was retired after the production pipeline demonstrated byte-
identical image output. Use `cargo xtask doc-media check` for current rendering,
repeatability, packaging, and documentation checks. Historical size figures above
are not the current full-catalog package size or budget; see [DOC_MEDIA.md](DOC_MEDIA.md).

## Validation

Both independent runs passed all APNG pixel/timing round-trip assertions; all
25 animations and 25 posters were byte-identical across runs. The warm-build
probe took 28–33 seconds on this machine, including the full-frame comparison
encodings and their verification. Formatting, Clippy with warnings denied, and
the existing selection-animation visual references also passed.

Before using these scenarios in published docs, choose meaningful first-frame
posters and natural loop boundaries. The prototype intentionally keeps the
entire interaction timeline visible, including the dialog's closed state.
