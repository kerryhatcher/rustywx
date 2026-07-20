# Stage 3: "Custom Widgets" — Dropdown, Toggle, Collapsing

**Status:** 🔲 Not started
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
- The dropdown's filter input uses raw macroquad `get_char_pressed()` —
  there's no Ply-native text input widget (see Ply issue #3).
- The dropdown iterates all 160+ sites for click detection each frame.
  Stage 3 should optimize to only check visible options.
- The product toggle buttons (Reflectivity/Velocity) are already in the
  top bar with active-state highlighting. Stage 3 extracts these into a
  proper toggle widget.

## Deliverable

Clickable site selector with type-to-filter, product toggle buttons,
working tilt dropdown.

## Validation

- [ ] Site dropdown opens on click, shows list of sites
- [ ] Typing in site dropdown filters the list
- [ ] Selecting a site triggers data fetch
- [ ] Product toggle shows active state with accent color
- [ ] Tilt dropdown shows available elevations from current scan
- [ ] All controls work with mouse and keyboard
- [ ] Dropdowns close on outside click
- [ ] `git push` → CI passes → `git tag v0.2.0-stage3` → `git push --tags`
