# Stage 3: "Custom Widgets" — Dropdown, Toggle, Collapsing

**Status:** 🟡 Implemented and locally validated — CI/tag pending
**Tag:** `v0.2.0-stage3`

## Goal

Proper clickable controls replacing keyboard-only hacks.

## Scope

- `widgets/dropdown.rs` — searchable dropdown (site selector, tilt selector)
- `widgets/toggle.rs` — product toggle (Reflectivity ⇄ Velocity)
- `widgets/collapsing.rs` — collapsible section (for NHC text later)
- Wire widgets into control bar
  (Note: `glass_panel.rs` lives in Stage 6 — it's not needed until visual theming.)
- Site dropdown with search/filter for 160+ sites
- Tilt dropdown populated from actual sweep data

## Notes from Stage 1

- A working searchable dropdown is already inline in `main.rs` (from Spike
  S3). It uses Ply-native elements: button + floating panel + keyboard
  input + outside-click detection. Stage 3 extracts this into a reusable
  `widgets/dropdown.rs` module.
- The dropdown's filter input continues to use raw macroquad
  `get_char_pressed()`. Ply 1.1.1 now has a text-input primitive, but its
  callback model is not needed for this compact type-to-filter control.
- The dropdown iterates all 160+ sites for click detection each frame.
  Stage 3 should optimize to only check visible options.
- The product toggle buttons (Reflectivity/Velocity) are already in the
  top bar with active-state highlighting. Stage 3 extracts these into a
  proper toggle widget.

## Deliverable

Clickable site selector with type-to-filter, product toggle buttons,
working tilt dropdown.

## Validation

- [x] Site dropdown opens on click and shows visible site rows
- [x] Typing in site dropdown filters the list
- [x] Selecting a site triggers data fetch
- [x] Product toggle shows active state with accent color
- [x] Tilt dropdown shows elevations from the current scan
- [x] Mouse and keyboard paths are implemented (click/type/arrows/Enter/Escape)
- [x] Dropdowns close on outside click
- [x] `cargo fmt`, clippy, check, and tests pass
- [x] Mandatory `just run` smoke test stays alive for 3 seconds
- [ ] `git push` → CI passes → `git tag v0.2.0-stage3` → `git push --tags`
