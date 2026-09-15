# Gallery and sample applications

Run these commands from the source checkout.

```sh
cargo run --locked --example gallery
```

The gallery is a small app with **Workspace, Activity and Settings** pages. Bottom navigation switches pages below 600px; an 80px rail appears from 600px and a 280px expanded rail from 1200px. Workspace includes Files, Components, Export and Schedule tabs, menus, tooltips and a working Duplicate/Undo flow. Activity has an unread filter and interactive list rows. Settings contains light/dark and accent controls, the editable form and selection controls. Open **More → Narrow view** to resize or **More → Review preferences** for the dialog. Changes are held in memory for the session.

**Workspace → Export** demonstrates segmented format/options selection, a stepped
quality slider, preparation and export progress, cancellation and a completion
snackbar. This is a four-second simulation; it does not create or modify files.
**Components → Values and progress** also demonstrates a continuous slider,
disabled steps and both progress shapes.

**Workspace → Files** now opens editable file details: a standard side sheet on
wide windows and a modal bottom sheet below 900px. Notes survive closing and
reopening. **More → File details in modal sheet** demonstrates a modal side sheet
on wide windows. The floating **New workspace** action opens a naming dialog and
updates the gallery's session workspace. **Components → Floating action buttons**
shows all sizes and colors.

**Workspace → Files** also supports search, recent queries, Shared/Pinned filters,
and a two-handle age range. Search expands into a docked panel or a full-window
view on narrow windows. Selecting a result opens its detail sheet. **Components →
Chips** demonstrates assist, suggestion, filter and removable input chips.

**Workspace → Schedule** demonstrates calendar and year selection, date ranges,
12/24-hour clock entry, text entry and validation. **Components → More ways to
work** demonstrates carousels, filled fields, elevated actions, switch icons,
button groups, split buttons, toolbars, rich hints and expandable FABs. It also
contains **Reduce motion** and **Increase contrast** controls. Bottom file-detail
sheets expose a drag handle with three snap heights.

**Components → Loading with expression** demonstrates canonical loading shapes
and wavy linear/circular progress, with loading/measured switching, pause and
optional wave travel. See [usage and reference timings](LOADING_PROGRESS.md).

```sh
cargo run --locked --example minimal
cargo run --locked --example gallery --no-default-features
```

See [renderer configuration and development profiles](GETTING_STARTED.md#rendering-and-development-builds).

The **More** menu now includes a full-screen workspace editor and a **Navigation**
submenu for rail expansion and modal navigation. **Schedule → Open calendar**
demonstrates a compact docked date picker with an integrated calendar/text toggle. Edited workspace drafts now show a nested discard confirmation. See [desktop behavior and variants](DESKTOP.md).

## Independent consumer app

The repository also contains **Northstar Studio**, a separate package with its own
manifest, lockfile and application model. It imports only public APIs and mixes
Material controls with native iced widgets and a generic application component.
Its editor supports validation, save/cancel, nested discard confirmation and native
popups. All changes stay in memory.

```sh
cargo run --locked --manifest-path consumers/desktop/Cargo.toml
cargo run --locked --manifest-path consumers/desktop/Cargo.toml --no-default-features
```

This fixture lives in the source checkout, outside the distributable crate. Its
tests also run against an extracted `.crate` archive. See [the integration and
release checks](BETA_READINESS.md).
