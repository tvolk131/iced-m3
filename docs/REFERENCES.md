# References and licensing

The implementation targets the [iced 0.14.0 release](https://github.com/iced-rs/iced/releases/tag/0.14.0) and its [versioned API documentation](https://docs.rs/iced/0.14.0/iced/), not the 0.15 development documentation. See the [value controls milestone](VALUES.md), [navigation milestone](NAVIGATION.md) and [workflow milestone](WORKFLOWS.md) for components and their reference tokens. See the [Material UI / Material 3 visual audit](MATERIAL_AUDIT.md) for the comparison, changes, screenshots and remaining gaps. Visual references: [Material 3 buttons](https://m3.material.io/components/buttons/specs), [text fields](https://m3.material.io/components/text-fields/specs), and [dialogs](https://m3.material.io/components/dialogs/specs).

[iced_aw](https://github.com/iced-rs/iced_aw) and [libcosmic](https://github.com/pop-os/libcosmic) were evaluated for compatibility and licensing. The former's inspected main branch targets iced 0.15 development APIs; the latter uses its own iced tree. No code was copied from either. The library depends directly on released iced; details are in [docs/ARCHITECTURE.md](ARCHITECTURE.md).

The bundled Roboto font is redistributed unmodified under [SIL OFL 1.1](../assets/fonts/OFL.txt). See [font provenance](../assets/fonts/README.md); `fonts::LICENSE` exposes its notice for application acknowledgments.

Canonical loading shape/morph data is generated from Apache-2.0 Material/AndroidX
sources. Preserve its [license and attribution](../assets/loading/README.md);
`loading::{LICENSE, NOTICE}` exposes these for application acknowledgments.
