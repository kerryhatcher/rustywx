# Stage 5: "Tropical" — NHC Hurricane Data

**Status:** 🔲 Not started
**Tag:** `v0.3.0-stage5`

## Goal

NHC tropical cyclone data: GIS overlays on scope + detail panel.

## Scope

- `nhc.rs` — port fetch + parse. Replace `ureq` with Ply `net` for
  `CurrentStorms.json` and GIS MapServer requests. Use `net::get()` with
  unique IDs per endpoint, poll each frame. Keep `image` crate for
  decoding graphics product thumbnails into RGBA bytes for Ply textures.
- **NHC CORS status (known from R1):** `CurrentStorms.json` has **no CORS**
  (requires relay proxy for WASM). GIS MapServer **has CORS** (direct fetch
  from WASM works). Desktop builds are unaffected — native HTTP clients
  don't require CORS headers.
- Draw NHC GIS overlays on scope: forecast cone, track, points, watches/warnings
- Draw wind probability contours
- Draw arrival time contours
- NHC slide-in panel with:
  - Storm selector dropdown
  - Storm stats (intensity, pressure, position, movement)
  - Graphics products grid (thumbnail + link)
  - Map overlay toggles
  - Text products (collapsible sections from Stage 3 widget)
- NHC toggle button in control bar with storm count badge

## Dependencies to add

- `image = { version = "0.25", default-features = false, features = ["png", "jpeg"] }` — decode NHC graphics product thumbnails
- `webbrowser = "0.8"` — open external links from NHC panel

## Deliverable

Full NHC tropical cyclone data — overlays on scope, detail panel with all
products.

## Validation

- [ ] NHC button shows storm count badge when storms are active
- [ ] NHC panel slides in from right on click
- [ ] Storm selector switches between active storms
- [ ] Storm stats display correctly
- [ ] Graphics products show thumbnails (or placeholders while loading)
- [ ] Map overlay toggles work (wind probs, arrival times)
- [ ] Text products expand/collapse
- [ ] GIS overlays visible on scope (cone, track, points)
- [ ] Wind probability contours render as colored lines
- [ ] "No active storms" state when season is quiet
- [ ] Data refreshes every 5 minutes
- [ ] NHC CORS status confirmed: CurrentStorms.json proxied, GIS MapServer direct
- [ ] `git push` → CI passes → `git tag v0.3.0-stage5` → `git push --tags`
