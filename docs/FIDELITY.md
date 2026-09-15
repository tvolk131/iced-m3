# Material fidelity audit

Audited 2026-09-13. This pass compares the library's component recipes with
Google's published tokens and reviews the resulting light/dark renders. It closes
specific visual/state gaps; it does not certify every pixel, gesture or platform
as fully conformant. See [the review page](http://127.0.0.1:8766/fidelity-audit/)
and [timed state frames](http://127.0.0.1:8766/?filter=fidelity).

Follow-up 2026-09-14: [desktop completion](DESKTOP_COMPLETION.md) adds app-bar ellipsis,
retained dialog stacks and compact calendars with manual entry. The matrix below
reflects those additions; the baseline correction table remains historical.

Latest: [desktop fidelity](DESKTOP_FIDELITY.md) completes baseline color roles, two-layer shadows, picker actions/period borders, slider labels and the documented current progress profile.

Earlier follow-up: [baseline desktop motion/layout](BASELINE_COMPLETION.md) refines range labels, progress handoff/springs, selection feedback, overlay surfaces and picker headers.

Latest follow-up: [overlay choreography and fitted/lazy carousels](DESKTOP_FINISH.md), plus a [native accessibility feasibility assessment](ACCESSIBILITY.md).

## Reference profile

The default profile remains **baseline Material 3**, with Expressive opt-in.
The baseline token source is Material Web's `tokens/versions/v0_192`, pinned to
revision [`a300d043d2e11886b0be3c0ee429c6bd336c41e9`](https://github.com/material-components/material-web/tree/a300d043d2e11886b0be3c0ee429c6bd336c41e9/tokens/versions/v0_192).
This supplies explicit component dimensions, typography and semantic state roles.
The same revision's public text-button wrapper supplies 12px horizontal padding.
This version is an auditable baseline, not a claim that it includes every current
Material catalog update.

Current Material Web wrappers override some baseline values: for example, their
text-field focus strokes are 3px while v0.192 specifies 2px. This library retains
its baseline 2px field focus stroke. The newer Android progress drawing profile is now implemented by the
[desktop fidelity follow-up](DESKTOP_FIDELITY.md). The earlier claim that the
preceding square-ended implementation already used that geometry was incorrect.

Measurements below are logical pixels. Stock typography is bundled Roboto 400/500;
letters are shaped by iced. Supplied icons, custom content and custom palettes
remain application-owned.

## Corrections made

| Component/state | Previous behavior | Corrected behavior |
| --- | --- | --- |
| Outlined field hover | Outline changed, label stayed neutral; error hover stayed error | Normal hovered label uses on-surface; error hover uses on-error-container for outline/label, while supporting text stays error |
| Filled field idle/hover | Idle indicator used outline; no hover layer | Idle indicator uses on-surface-variant; an 8% on-surface hover layer fades in/out over the field alone |
| Filled field disabled | Kept its normal opaque fill; indicator used outlined-field opacity | On-surface at 4% for container and 38% for indicator, with no hover response |
| Floating label typography | Fixed 20px line height during size changes | Label size/line height interpolate from Body Large 16/24 to Body Small 12/16, preserving centered and floating alignment |
| Button focus | Focus ring alone | Baseline focus state layer plus the existing desktop focus ring; focused outlined-button stroke uses primary |
| Filled/tonal button hover | Flat at all times | Elevation 0 → 1 on hover, back to 0 while held; elevated buttons keep their 1 → 2 recipe |
| Text button spacing | 24px horizontal padding shared with filled buttons | 12px default horizontal padding; explicit application padding remains authoritative in either builder order |
| Switch | Unselected track used surface-container-high; thumb/icon colors stayed at idle roles | Highest surface track; hovered/focused/pressed thumb uses on-surface-variant or primary-container; icon roles and disabled unselected track corrected |
| Checkbox/radio focus and press | Focus outline without inner state layer; press tint reused hover tint | Focus layer and stronger unselected outline; baseline selected/unselected press color roles, with hover still visible underneath |
| Slider/range-slider focus | Focus outline alone | 12% primary thumb halo as well as the existing desktop focus outline |
| Chips | All unselected labels used on-surface; outline-variant borders; unselected elevated disabled chips were transparent | Assist label uses on-surface, other labels use on-surface-variant; baseline outline/focus and leading-icon roles; disabled elevated fill at 12%; trailing dropdown spacing corrected |
| Legacy chip helper | Generic outlined/selected button styling | Shared chip state recipe, including selected outline removal |
| Cards and surfaces | Filled tones too light; cards used generic outline and centered shrink content | Filled uses highest surface, elevated uses low surface, outlined uses surface/outline-variant; card content starts at its padding; card hover uses the corresponding elevation recipe |
| Navigation bar | Shared rail's 56px indicator | 64×32 bar indicator; rail remains 56×32. Indicator feedback also renders a held press ripple |
| Date picker | 24px headline, generic year-button typography, neutral today outline, oversized range band | 32/40 headline; Body Large years in 72×36 indicators; primary today text/outline; range text uses on-secondary-container with a 40px range band |
| Time picker | 45px digits in 88px fields; generic tonal AM/PM; 40px clock selector | Display Large 57/64 digits, 96px or 114px fields; Title Medium AM/PM in a joined 52×80 group with tertiary selection; 48px clock selector |
| Sheets | Bottom elevation 3; standard bottom surface differed from modal bottom | Baseline elevation 1 and low surface for bottom/modal side sheets; standard side stays surface/elevation 0 |

`ColorScheme` now exposes `error_container` and `on_error_container`, generated by
the same cached HCT scheme as the other roles. Applications that construct a full
`ColorScheme` literal must supply the new roles; editing a generated theme remains
unchanged.

The press expansion duration, minimum quick-tap interval, release timing, and
separate hover/ripple layering are preserved. Ordinary renderer tests continue to
check that pressing a checkbox or radio does not erase its hover circle.

Follow-up 2026-09-14: [desktop variants](DESKTOP_VARIANTS.md) adds adorned fields, filled/outlined dropdowns, clock/numeric switching, card states, lowered FABs and buffered/multicolor progress.

## Catalog review matrix

This table distinguishes source comparisons from known adaptations. Existing
reference groups cover every family; the added `fidelity-*` groups concentrate on
the corrected states. A row is not a promise of every possible application layout.

| Family | Compared in this pass | Remaining fidelity or variant limits |
| --- | --- | --- |
| Typography / theme | All 15 size, line-height, weight and recorded tracking roles; HCT-derived semantic role assignments | Tracking cannot be applied through iced's public text API; baseline color roles are complete; density/font-scaling adapters remain incomplete |
| Buttons / icon buttons | Five button variants, 40px height, label role, container/foreground/outline roles, disabled opacity, hover/focus/press recipes | Arbitrary leading/trailing content requires caller sizing; explicit desktop focus rings are a library addition; outlined icon focus opacity follows the source's 8% token |
| FABs / extended FABs | 40/56/96px sizes, 12/16/28px corners, 24/36px icon slots, four color choices, elevation 3/4 | Lowered FABs now use elevation 1/2 and the surface-low color; full shape choreography remains incomplete; disabled FABs are a library convenience |
| Checkboxes / radios / switches | Box, ring, track, thumb, icon and halo geometry; selected, mixed, error, disabled, hover, focus and held-state color recipes | Checkbox bar morph, separate opacity and asymmetric timing are implemented; centered ripple edge remains an approximation; keyboard focus ring is a desktop addition |
| Text fields | Transparent outlined fields with a moving border gap; filled label/underline, error/disabled/hover/focus roles; leading/trailing slots, actions and prefix/suffix layout | Native caret color/tracking remain upstream limits; long affixes clip in constrained widths; custom child styling remains caller-owned |
| Chips / segmented buttons | 32/40px heights, 18px icons, type roles, selection containers, outlines and shared state feedback | Filter chips deliberately reserve their check slot; input avatars now remain visible during selection with the rounded/inset recipe; filter icon scaling and detailed per-family/trailing-action choreography remain adaptations |
| Sliders / range sliders | Baseline 4px track, 20px handle, 40px halo, 28px value label, ticks, disabled and focus roles | Level-1 thumb shadows, pointed labels and hover/focus scaling are implemented; colliding labels now separate as a documented desktop adaptation; newer/Expressive slider size variants remain separate work |
| Linear / circular progress | Baseline advance/Web motion plus opt-in wavy linear/disjoint and circular/retreat profiles; buffered/multicolor APIs, amplitude transitions and completion handoffs | Wave arc lengths use numerical cubic measurements; buffered rendering and the baseline Web linear profile remain documented choices; not every alternate delegate/show-hide behavior is ported; see LOADING_PROGRESS.md |
| Cards / surfaces | 12px corners, filled/outlined/elevated tones and outline roles, hover elevation, left-aligned default card content | Two-layer cached key/ambient shadows; disabled/dragged states have controlled APIs; drag/drop behavior and media-specific compositions remain caller-owned |
| Badges / dividers | 6px dot, 16px count height, Label Small, error/on-error; 1px outline-variant dividers | Anchor placement and list insets remain compositional; no automatic icon-specific badge offset |
| Lists | Transparent grouping with optional native container fill; 56/72/88px minimum heights, 16px horizontal space, 16/24 headline, 14/20 supporting text, 11/16 overline | Caller supplies icon/avatar/image sizing; selected row treatment is a desktop library extension; wrapping can increase height |
| Tabs / app bars | Transparent tabs with explicit color/gradient viewport fill; primary/secondary label roles and 3/2px indicators; 48/64px tabs; 64/112/152px app bars with 22/28/32px titles | Long titles now ellipsize; collapse is controlled; precise content spacing remains dependent on caller actions |
| Navigation | Bar/rail indicators, icon/label roles, 80px bar and collapsed rail; expanded/modal rail implementation reviewed against its documented newer profile | Baseline token set predates expanded rails; full Expressive geometry/morph is not claimed; focus ring and adaptive breakpoints are documented desktop choices |
| Menus / selects / tooltips | Menu surface/elevation 2, 4px corners, 48px rows, type hierarchy; inverse plain tooltip; rich tooltip surface/elevation 2 and 12px corners | Select now shares filled/outlined field styling, adornments, errors and supporting text; editable/autocomplete comboboxes remain separate; rich body/actions are caller content; plain-menu surface/content fades and rounded size transitions are implemented; per-row staggering and rounded rich-popup/body fades are implemented; rich-popup timing remains a documented desktop adaptation |
| Snackbars / dialogs | Inverse snackbar roles, action typography, 48/68px content-driven sizing; dialog high surface, 28px corners, 24px title/padding | Dialog title/body/action roles require appropriate content composition; basic surfaces now have rounded height/translation, linear scrim, surface and content fades; semantic action staggering is available through `dialog::actions`; custom content must opt into that slot; nested stacks isolate input and restore focus/carets |
| Bottom / side sheets | Surface/elevation recipes, 28/16px modal corners, 32×4px handle | Widths use the documented desktop/newer Android profile; standard separators are a desktop treatment; full gesture physics remain deferred |
| Search | 56px bar, Body Large, high surface, 28px docked view corners and overlay occlusion | 72px full-screen header, stable editor/results layout, staged results/divider and clear action; Android content-scale/icon paths and complete mobile choreography remain adaptations |
| Date / time pickers | Calendar/date/year role assignments and indicator sizes; time display, period colors, dial and selector geometry | Compact calendars and validated date/time entry toggles are now available; baseline modal header spacing is implemented; docked surface/body choreography is implemented as a desktop adaptation; locale/RTL formatting remains unfinished; explicit Cancel/OK actions and continuous period borders are implemented |
| Carousel | Fitted keylines, shifting start/end arrangements, five layouts including centered hero, bounded lazy construction, recent-velocity snapping | Horizontal desktop implementation; masks require the parent surface color; full Android padding/RTL/vertical strategy and native fling/spring physics remain adaptations |
| Expressive previews | Groups, split buttons, toolbars and FAB menus retained as opt-in; canonical loading outlines/morph matching and wavy progress implemented | Width/neighbor deformation, complete size/shape tokens and the broader spring system remain incomplete; loader spring timing uses a continuous analytical response |

## Rendering and platform boundaries

- Efficient rounded child clipping and true group opacity need renderer
  integration. [Composition prototypes](COMPOSITION.md) exercise both backends
  and document performance/color-space failures of widget-only workarounds.
  Existing carousel color patches and known-surface fades remain adaptations.
- Built-in surfaces now use cached key/ambient shadows at levels 0–5.
  `Theme::elevation` remains a single-key-shadow compatibility helper for native
  iced containers; `elevated` supplies the complete recipe. Browser and CPU/GPU
  rasterization can still differ.
- The native text-input style has no separate caret-color field. The caret follows
  the input value color, so primary/error caret color is still an upstream text
  rendering gap. Replacing the editor merely to recolor the caret would also put
  native editing and IME behavior at risk.
- Rounded state layers, fonts and shadows may rasterize differently across CPU
  and GPU backends. Timed golden comparisons use the established canonical
  software environment; property tests also exercise Metal.
- Native screen-reader semantics still require upstream support or an evaluated
  bridge. This pass adds no platform accessibility integration.
- RTL, locale adapters, Windows/Linux native interaction, touch/swipe/fling and
  native window gestures are not newly validated by this headless review.

## Verification

Six ordinary tests check actual renderer output against independent reference
swatches or spatial properties: field roles/state precedence, disabled opacity,
hover bounds/reduced motion, button hover elevation and held behavior, card surface
roles, switch colors, and keyboard focus without activation (the field role test
covers several properties together). Existing interaction, gallery and timed
reference tests run alongside them. The clock hand regression now locates the dial
from its rendered labels, rather than assuming a header offset; the clock exposes
its label children to ordinary iced widget operations.

Three new reference functions add 88 frames in 28 light/dark review groups. Counts,
executed backends and final checks are recorded in [VALIDATION.md](VALIDATION.md).
The before/after page compares the old and new library, not screenshots of an
external implementation. The source links below supply the Material reference.

## Primary source anchors

All baseline links refer to the same pinned revision:

- [Outlined field](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/tokens/versions/v0_192/_md-comp-outlined-text-field.scss), [filled field](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/tokens/versions/v0_192/_md-comp-filled-text-field.scss)
- [Filled button](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/tokens/versions/v0_192/_md-comp-filled-button.scss), [text button spacing](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/tokens/_md-comp-text-button.scss)
- [Switch](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/tokens/versions/v0_192/_md-comp-switch.scss), [checkbox](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/tokens/versions/v0_192/_md-comp-checkbox.scss), [radio](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/tokens/versions/v0_192/_md-comp-radio-button.scss)
- [Filter chip](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/tokens/versions/v0_192/_md-comp-filter-chip.scss), [filled card](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/tokens/versions/v0_192/_md-comp-filled-card.scss)
- [Navigation bar](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/tokens/versions/v0_192/_md-comp-navigation-bar.scss), [date picker](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/tokens/versions/v0_192/_md-comp-date-picker-modal.scss), [time picker](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/tokens/versions/v0_192/_md-comp-time-picker.scss)
- [Bottom sheet](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/tokens/versions/v0_192/_md-comp-sheet-bottom.scss), [side sheet](https://github.com/material-components/material-web/blob/a300d043d2e11886b0be3c0ee429c6bd336c41e9/tokens/versions/v0_192/_md-comp-sheet-side.scss)
