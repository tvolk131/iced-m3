# Beta.3 release review

This release adds opt-in Expressive spring/action support, desktop interaction
fixes, and generated component documentation. It does not claim complete Material
3 Expressive conformance; remaining coverage is listed in [EXPRESSIVE.md](EXPRESSIVE.md).

## Motion review

The review exercises actual iced layout, events, redraws and rendered pixels:

- Group hold/release rebound, rapid release followed by a neighbor press,
  interruption continuity, compact/narrow layouts and idle redraw termination.
- Switch position/diameter overshoot, cancellation, reversal, bounded colors,
  keyboard activation, reduced motion and frame-cadence independence.
- Tab-indicator movement and rail width overshoot/reversal; reversible dialog
  presence and the end-of-dismissal rectangular-content regression.
- Contact sheets for Standard, Expressive and reduced motion in light/dark themes.
  These make the actual rendered transitions reviewable alongside numeric tests.

Release checks now opt into the requested headless backend explicitly. Previously,
several Expressive tests constructed a TinySkia harness regardless of
`ICED_TEST_BACKEND`; fixed-reference tests remain deliberately pinned to TinySkia.

```sh
ICED_TEST_BACKEND=tiny-skia cargo +1.92.0 test --locked --no-default-features --lib visual_tests::
ICED_TEST_BACKEND=tiny-skia cargo +1.92.0 test --locked --no-default-features --lib expressive_release_review -- --ignored
ICED_TEST_BACKEND=wgpu cargo +1.92.0 test --locked --lib visual_tests::expressive:: -- --include-ignored --skip visual_references
ICED_TEST_BACKEND=wgpu cargo +1.92.0 test --locked --lib switch_spring_
ICED_TEST_BACKEND=wgpu cargo +1.92.0 test --locked --lib spring_dialog_exit
ICED_TEST_BACKEND=wgpu cargo +1.92.0 test --locked --lib expressive_rail_width
cargo +1.92.0 test --locked --no-default-features --lib visual_references -- --ignored
```

Review captures are under `target/beta3-motion-review/<backend>/<scheme>-<theme>/`.
Each row reads left to right. Group frames show initial, 40ms press, 1000ms hold,
20ms release, 140ms release and settled state. Controls show initial, 40/140ms
selection, 20/140ms reversal and settled state. Dialogs show open, 40/100/140/200ms
closing and fully closed. All captures come directly from component rendering.
They supplement the existing pixel/geometry assertions and immutable reference
comparisons; generating a new image does not approve its appearance automatically.

## Documentation and package checks

The production pipeline replaces the retired APNG spike runner and Python package
checker. [DOC_MEDIA.md](DOC_MEDIA.md) describes the compiled catalog, lossless
encoder, freshness/orphan checks, deterministic generation and package budget.
The encoder retains independent pixel/timing round-trip tests, including its
full-frame comparison path.

CI checks the generated archive on macOS and the docs.rs configuration on Linux,
plus ordinary Windows/Linux/macOS, MSRV and consumer checks. Publication follows
[RELEASING.md](RELEASING.md) only after checks pass on the merged master commit.
Local checks do not certify native platform accessibility, IME, or complete M3
fidelity. The generated archive is a candidate, not an automatic publication.
