# Radar side panel — design

**Date:** 2026-07-21
**Branch:** port/ply-engine

## Goal

Add a `Radar` button to the controls bar (left of `Borders`) that toggles a
right-hand slide-in side panel, mirroring the existing Tropical panel. Move the
radar controls — site dropdown, product toggle, tilt dropdown, and the site-name
readout — out of the controls bar and into that panel. The bar collapses to:

```
Radar | Borders | Alerts | Tropical | Location | [window controls]
```

## Existing patterns being reused

- **Tropical panel** (`app/src/main.rs` ~1060–1150): right-anchored
  `glass_panel::glass`, `floating(...).offset((panel_x, y)).z_index(50).attach_root()`,
  spring slide-in driven by `nhc_anim_start` + `animation_level`
  (`ease_out_elastic` / `ease_out_cubic` / instant). Gated by `state.nhc_show_panel`.
- **Tropical toggle** (`btn-nhc`, main.rs ~933–973 draw, ~1977–1983 + 2010 handler):
  hover-tinted button, flips `nhc_show_panel`, persists `settings.show_nhc`.
- **Dropdown widget** (`app/src/widgets/dropdown.rs`): `DropdownConfig` is `Copy`;
  both `draw` and `handle_input` take it by value. The popup list floats at a
  fixed absolute screen offset (`config.panel_offset`) via `attach_root()`.
- **Settings persistence** (`app/src/settings.rs`): `show_nhc: bool`, default
  `false`, JSON round-trip covered by tests at lines ~160/174/213.

## State changes

`AppState` (`app/src/state.rs`):
- `show_radar: bool` — panel visibility (runtime), initialized from `settings.show_radar`.
- `radar_anim_start: f64` — slide-in animation clock (mirrors `nhc_anim_start`).

`Settings` (`app/src/settings.rs`):
- `show_radar: bool`, **default `true`** (panel open on first launch — the radar
  controls left the main bar, so make them immediately discoverable).
- Add to serialize/deserialize + the existing round-trip test JSON.

Initialization in `main.rs` mirrors line 542
(`state.nhc_show_panel = state.settings.show_nhc;`):
`state.show_radar = state.settings.show_radar;`.

## Controls-bar changes

Remove from the bar (main.rs ~847–873): `site_dropdown.draw`, the `— {name}`
text, `toggle::draw(PRODUCT_OPTIONS)`, `tilt_dropdown.draw`.

Add a `Radar` toggle button as the **first** control (before `Borders`),
copied from the `btn-nhc` button pattern:
- id `btn-radar`, hover-tinted, active color `0x0dc5b8` when `show_radar`.
- label `"Radar ✓"` / `"Radar"`, glyph `nf::RADAR` (optional, matches header).
- `.accessibility(|a| a.button(label).checked(state.show_radar))`.

## Radar panel rendering

Clone the Tropical panel block, gated by `state.show_radar`, using
`radar_anim_start`. Same right-edge anchor, width (`320.0` desktop /
`screen_width()` mobile), z-index, glass styling. Header: `nf::RADAR` +
`"Radar"` (bold font, honoring `active_dyslexic`).

Body (moved verbatim from the controls bar, stacked TopToBottom):
1. `— {site.name}` readout text.
2. `state.site_dropdown.draw(ui, site_cfg, site.id, &site_options, Some(state.site_index))`.
3. `toggle::draw(ui, state.product, &PRODUCT_OPTIONS)`.
4. `state.tilt_dropdown.draw(ui, tilt_cfg, tilt_label, &tilt_options, ...)`.

Where `site_cfg` / `tilt_cfg` are runtime copies of `SITE_DROPDOWN` / `TILT_DROPDOWN`
(see next section). `handle_input` (main.rs ~1805/1812) must be passed the *same*
runtime configs.

## Dropdown popup positioning (flyout-left)

The dropdown popup floats at `config.panel_offset` in absolute screen space. With
the buttons now in the right sidebar, build a per-frame runtime `DropdownConfig`
so the popup opens **to the left of** the panel, near its button:

```
x = screen_width - panel_w - popup_w - gap   // popup_w = config.width, gap ~8, panel_w+8 edge
y = approximate control position within the fixed panel layout
```

`DropdownConfig` is `Copy`, so: `let site_cfg = DropdownConfig { panel_offset: (x, y), ..SITE_DROPDOWN };`
Pass `site_cfg` to both `draw` and `handle_input`. Same for tilt.

The `y` values are hardcoded estimates because the panel layout is fixed and
known (header 28 + gaps + control heights). This is a deliberate ceiling:

```
// ponytail: hardcoded popup y per fixed panel layout; compute from measured
// element rects if the panel layout becomes dynamic.
```

On mobile (`panel_w == screen_width()`) there is no room to the left; fall back
to opening the popup at the left edge (`x = 8`) over the panel.

## Mutual exclusion

`show_radar`, `nhc_show_panel`, and `show_settings_panel` all anchor to the same
right edge. When opening any one, close the other two so panels never stack.
Add the guard in each toggle handler (reset the opened panel's `*_anim_start` to
`0.0` so its slide-in replays).

## Error handling

None new — this is UI wiring over existing state. Dropdown open/close, site
selection, product/tilt changes keep their current handlers; only the
`DropdownConfig` passed in changes.

## Testing

One runnable check: extend the existing `settings.rs` round-trip test to include
`show_radar` (serialize true/false, deserialize, assert). No new framework.
`cargo test` must stay green (107 tests).

## Out of scope

- No change to Borders / Alerts / Tropical / Location buttons (stay in bar).
- No dropdown-widget rewrite (flyout via runtime offset, not inline expansion).
- No keyboard-shortcut changes (F/N site nav, etc. unchanged).
