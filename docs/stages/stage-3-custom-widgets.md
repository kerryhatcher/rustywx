# Stage 3: "Custom Widgets" — Dropdown, Toggle, Collapsing

**Status:** 🟡 Implementation complete; pushed; CI green; tag pending

**Implementation commit:** `05e4fb7` (`feat(ui): implement Stage 3 custom widgets`)

**Target tag:** `v0.2.0-stage3`

## Goal

Replace keyboard-only control hacks and one-off inline UI with reusable,
clickable Ply controls for site, product, and radar tilt selection.

## Delivered

- `widgets/dropdown.rs`
  - Reusable dropdown state and configuration.
  - Optional type-to-filter search.
  - Mouse selection and wheel navigation.
  - Up/Down, Enter, Escape, and Backspace handling.
  - Outside-click closing.
  - Configurable panel width, position, and visible-row count.
  - Hit-testing only for rendered rows rather than the full option set.
- `widgets/toggle.rs`
  - Reusable mutually exclusive button group.
  - Active product receives accent-color highlighting.
- `widgets/collapsing.rs`
  - Reusable header/body section with persistent open/closed state.
  - Ready for NHC text products in Stage 5.
- Control-bar integration
  - Searchable radar-site selector.
  - Reflectivity/Velocity product toggle.
  - Tilt selector populated from the current product's decoded sweeps.
  - Status line includes the selected elevation.
- State transition cleanup
  - Site, product, and tilt changes now use centralized helper functions.
  - Product changes reset the tilt index to prevent stale selections.
  - Site changes clear the current scan, start cache loading, and notify the worker.
  - Only one dropdown can remain open at a time.
- Input conflict fixes
  - Open dropdowns consume wheel/navigation input instead of zooming or panning the scope.
  - Keyboard shortcuts now match their labels: `R` = Reflectivity and `V` = Velocity.

## Implementation Data

| Item | Result |
|---|---|
| Radar sites in `geo::RADAR_SITES` | 143 |
| Site rows rendered/hit-tested at once | 12 |
| Tilt rows rendered/hit-tested at once | Up to 10 |
| Product options | 2 |
| Reusable widget modules added | 3 |
| Workspace unit tests after Stage 3 | 15 |
| Stage 3 implementation commit | `05e4fb7` |

Tilt labels are not hard-coded. They are generated every frame from
`scan.sweeps(state.product)` and `SweepData::elevation_deg`, so the selector
reflects the actual elevations available for the current scan and product.

## Lessons Learned

### Preserve Ply's two-phase frame lifecycle

Ply interaction queries cannot be mixed freely with declaration because
`Ui` holds a mutable borrow of `Ply`. Reusable controls therefore follow a
two-phase contract:

1. Declare elements through `DropdownState::draw(...)` or `toggle::draw(...)`.
2. Call `ui.show(...)`.
3. Query presses and process semantic events through `handle_input(...)`.

Widgets return source indices or values; they do not fetch data or mutate
unrelated application state. This avoids borrow conflicts and keeps data-flow
logic in `main.rs`.

### Use stable source IDs for dynamic options

Ply supports indexed IDs with `(&str, u32)`. Dropdown rows use their stable
source index, not their position in the filtered/visible slice. Filtering and
scrolling therefore do not change an option's identity between frames.

### Render and hit-test the same visible slice

The Stage 1 spike checked every site ID each frame even though only a small
set was rendered. Stage 3 computes one filtered index list and checks only the
visible slice. For the current 143-site data set, at most 12 site option IDs
are queried per frame.

### Dropdown input must gate scope input

Mouse-wheel and pointer input are global macroquad state. Without explicit
gating, scrolling a dropdown also zooms the radar and dragging a control can
pan the scope. The integration now suppresses scope pan/zoom while either
dropdown is open.

### Product and tilt are coupled state

Reflectivity and Velocity can expose different sweep sets. Changing product
must reset or clamp `tilt_index`; otherwise rendering may clamp internally
while the control displays stale state. Stage 3 resets to the lowest tilt and
updates the status line.

### Ply 1.1.1 changes the text-input finding

The original Spike S3 report and draft Ply issue #3 were based on an earlier
API review and state that Ply had no text-input primitive. Ply 1.1.1 exposes
text-input support and focus/value APIs. Stage 3 still uses
`get_char_pressed()` for the compact type-to-filter behavior because it needs
no cursor, selection, clipboard, or IME support. The draft issue should be
reframed around text-input ergonomics/documentation rather than absence.

### UI validation remains partly interactive

Pure filtering, cursor/scroll, and collapsing-state logic have unit coverage,
but Ply still lacks a convenient headless end-to-end UI test path. Stage 3 was
also validated by launching the real app and driving the controls under X11
with `xdotool`, with screenshots captured in `/tmp` during local validation.

## Validation

- [x] Site dropdown opens and renders site rows.
- [x] Typing filters sites case-insensitively by ID or name.
- [x] Mouse and Enter selection use stable source indices.
- [x] Site selection follows the existing worker/cache switch path.
- [x] Product toggle displays an accent active state.
- [x] Tilt dropdown uses elevations from the current scan/product.
- [x] Up/Down, wheel, Enter, Escape, and Backspace paths are implemented.
- [x] Dropdowns close on outside click and close each other when opened.
- [x] Scope pan/zoom is gated while a dropdown is open.
- [x] `cargo fmt`, clippy, check, tests, audit, and deny pass locally.
- [x] Mandatory `just run` smoke test remains alive for at least 3 seconds.
- [x] Desktop interaction validated with the running release binary.
- [x] Commit `05e4fb7` pushed to `origin/port/ply-engine`.
- [x] GitHub Actions run `29716438284` for `05e4fb7` completed successfully.
- [ ] Create and push `v0.2.0-stage3` after CI is green.

## Follow-ups / Deferred Polish

- Use the collapsing widget for NHC text products in Stage 5.
- Consider adopting Ply's native text input if the filter later needs focus,
  cursor, selection, clipboard, accessibility, or IME behavior.
- Add a scrollbar/position indicator for long site result sets if usability
  testing shows it is needed.
- Replace fixed dropdown offsets with anchor-based or available-space-aware
  placement during Stage 6 responsive layout work.
- Revisit `docs/ply-issues/03-text-input-widget.md` before filing; its current
  claim that no Ply text input exists is stale for Ply 1.1.1.
