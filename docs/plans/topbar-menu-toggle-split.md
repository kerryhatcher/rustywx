# Top-bar split: Panels group vs Layers group (decouple panel-open from data-visible)

Status: PLAN ONLY — do not implement outside the assigned task. Design "Option A".

## Goal

Split the flat top-bar control row into two visually distinct groups, separated
by a vertical divider:

- **Panels group** — menu-openers that open/close the right-edge side panels.
  Buttons: `Radar ▸` and `Tropical ▸`. Chevron affordance, **no** `✓`.
- **Layers group** — pure visibility toggles (`✓` idiom). Buttons: `Radar`,
  `Tropical`, `Borders`, `Watches`, `Warnings`, `Location`.

Core requirement: **panel-open state must be a separate flag from data-visible
state.** Today `AppState.show_radar` is double-duty (opens the Radar panel AND
gates the radar site markers via the `show_sites` arg). We split that, and we
add a real "radar data on/off" gate (which currently does not exist — the radar
texture draws unconditionally) plus a "tropical data on/off" master gate.

## Naming decision (justified)

- **Rename** `AppState.show_radar` → `AppState.radar_panel_open`.
  Rationale: the field's *only* remaining meaning after the split is
  "is the Radar side panel open". Keeping the old name would be a live trap
  (every future reader assumes it gates data). The rename is mechanical
  (13 call sites, all in `main.rs` + 1 decl in `state.rs`) and the compiler
  finds every miss. `nhc_show_panel` already has a good name — leave it.
- **Add** `AppState.show_radar_data: bool` — gates the radar data texture AND
  the radar site markers (`show_sites` arg). This is the new "radar layer" flag.
- **Add** `AppState.show_nhc_data: bool` — master gate that ANDs with the
  existing `nhc_overlays` sub-toggles to control `draw_nhc_overlays`.
- **Persisted Settings keys keep their current names** (`show_radar`,
  `show_nhc`) meaning "panel open at startup" — renaming them would force a
  serde migration for zero user benefit. We **add** two new persisted keys
  `show_radar_data` and `show_nhc_data` (both `#[serde(default = "default_true")]`
  for back-compat with existing config blobs).

## Defaults decision (justified)

| Flag                         | Default | Why |
|------------------------------|---------|-----|
| `show_radar_data`            | `true`  | Radar is the whole app; data on by default. |
| `show_nhc_data`              | `true`  | Tropical overlays draw when a storm bundle exists; master gate on. |
| `settings.show_radar` (panel)| **flip `true`→`false`** | The panel auto-opened before purely for *discoverability* of the radar controls. Now there is a dedicated `Radar ▸` opener button in the top bar, so the panel no longer needs to auto-open. Panels closed on first launch is the cleaner default. |
| `settings.show_nhc` (panel)  | `false` (unchanged) | Already closed by default. |

The `settings.show_radar` flip is a one-line change plus three test-assertion
updates, all specified in Task 1. If the maintainer prefers panels-open, revert
that single literal — nothing else depends on it.

## Distinction to keep straight (naming is confusing here)

- **"radar data texture"** = `state.radar_texture` (the rasterised reflectivity/
  velocity image), drawn at `scope.rs:419`. THIS is gated by `show_radar_data`.
- **"radar sweep line"** = the decorative rotating beam, `draw_radar_sweep(...)`
  at `main.rs:831-833`, gated by `settings.show_sweep`. **Do NOT touch this.**

## Line references

All line numbers below are as of the current HEAD of `port/ply-engine`. Edits
in earlier tasks WILL shift later line numbers. Implementers must match on the
quoted code, not on the raw line number, and must run tasks in dependency order.

---

# Task 0 — Add `CHEVRON_RIGHT` glyph constant  [tier: Haiku]

**File:** `app/src/widgets/mod.rs`

The `nf` module (lines 11-23) has `CHEVRON_DOWN`/`CHEVRON_UP` but no
right-pointing chevron for the `▸` panel-opener affordance. Add one.

**Edit** — inside `pub mod nf { ... }`, after the `CHEVRON_UP` line (line 20):

```rust
    pub const CHEVRON_UP: &str = "\u{f077}"; // fa-chevron-up
    pub const CHEVRON_RIGHT: &str = "\u{f054}"; // fa-chevron-right
```

`\u{f054}` is Font Awesome `fa-chevron-right`, in the same PUA range as the
existing chevrons, so it renders with `SYMBOL_FONT` (the icon font already used
for `CLOSE`, `GEAR`, etc.). No new dependency.

**Dependencies:** none. **Parallel:** yes (independent file/region).
**Verify:** `cargo check -p rustywx` (constant is `unused` until Task 3 — that
is fine; `cargo check` still passes, `pub const` does not warn).

---

# Task 1 — Settings: add persisted data-visibility fields  [tier: Haiku]

**File:** `app/src/settings.rs`

### 1a. Add two struct fields

After the `show_radar` field block (ends line 123) add:

```rust
    #[serde(default = "default_show_radar")]
    pub show_radar: bool,
    /// Whether the radar data layer (texture + site markers) is shown at startup.
    #[serde(default = "default_true")]
    pub show_radar_data: bool,
    /// Whether the tropical (NHC) data layer master-gate is on at startup.
    #[serde(default = "default_true")]
    pub show_nhc_data: bool,
```

(`default_true` already exists at line 87.)

### 1b. Default impl (lines 154-177)

Add the two new fields (both `true`) and **flip `show_radar` to `false`**:

```rust
            show_nhc: false,
            show_radar: false,        // was: true — see Defaults decision
            show_radar_data: true,
            show_nhc_data: true,
```

### 1c. Update tests

- `default_matches_existing_startup_behaviour` (line 191):
  change `assert!(settings.show_radar);` → `assert!(!settings.show_radar);`
  and add `assert!(settings.show_radar_data);` + `assert!(settings.show_nhc_data);`.
- `serde_round_trip` (lines 201-220): add `show_radar_data: false,` and
  `show_nhc_data: true,` (any values — this test just round-trips) to the
  constructed struct so it still compiles.
- `deserializes_settings_without_location_fields` (lines 247-263): the legacy
  JSON blob omits the new keys; add assertions
  `assert!(s.show_radar_data);` and `assert!(s.show_nhc_data);` (they must
  default `true`). The existing `assert!(s.show_radar);` on line 259 asserts the
  serde *default fn* (`default_show_radar` → still `true`) — that is the
  back-compat default for OLD blobs, which is independent of the `Default` impl
  flip in 1b. **Leave line 259 as-is** (`default_show_radar()` unchanged).

**Dependencies:** none. **Parallel:** yes.
**Verify:** `cargo test -p rustywx settings::` and `cargo fmt`.

---

# Task 2 — Core decouple: state fields, global rename, render gating, restore  [tier: Sonnet]

Depends on **Task 1** (reads `settings.show_radar_data` / `show_nhc_data`).
This is the semantic heart. Do it as one atomic change so `main.rs` compiles.

**Files:** `app/src/state.rs`, `app/src/main.rs`.

### 2a. `state.rs` — rename + add fields

Line 99-100, rename and re-doc:

```rust
    /// Whether the Radar controls side panel is open.
    pub radar_panel_open: bool,
```

Immediately after `radar_anim_start` (line 102) add:

```rust
    /// Toggle: draw the radar data texture and radar site markers.
    pub show_radar_data: bool,
    /// Master gate: draw NHC tropical overlays (ANDs with `nhc_overlays`).
    pub show_nhc_data: bool,
```

### 2b. `main.rs` — global rename `show_radar` → `radar_panel_open`

Rename **every** `state.show_radar` and the struct-init `show_radar:` to
`radar_panel_open`, at these sites: **428, 582, 895, 898, 910, 1144, 2159,
2340, 2344, 2350, 2360, 2366**. (Line **825** is the ONE exception — handled in
2d, it becomes `show_radar_data`.) A grep for `show_radar` in `main.rs` after
this task must return only the new `show_radar_data` uses.

Note: `state.settings.show_radar` (the persisted field, e.g. line 582 RHS and
2350) is NOT renamed — only the `AppState` field is.

### 2c. `main.rs` — struct init (~line 428)

```rust
        radar_panel_open: false,
        radar_anim_start: 0.0,
        show_radar_data: true,
        show_nhc_data: true,
```

(Init defaults; the real values are seeded from settings in 2e.)

### 2d. `main.rs` — settings-apply seeding (~lines 578-583)

Current:
```rust
            state.nhc_show_panel = state.settings.show_nhc;
            state.show_radar = state.settings.show_radar;
```
After:
```rust
            state.nhc_show_panel = state.settings.show_nhc;
            state.radar_panel_open = state.settings.show_radar;
            state.show_radar_data = state.settings.show_radar_data;
            state.show_nhc_data = state.settings.show_nhc_data;
```

### 2e. `main.rs` — render gating at the scope call (lines 812-827)

This is the actual data decouple. Current:
```rust
        scope::draw_scope_to_texture(
            state.radar_texture.as_ref(),
            site,
            state.pan_km,
            state.zoom,
            Some((&state.borders, state.show_borders)),
            Some((&state.alerts, state.show_watches, state.show_warnings)),
            state.nhc_bundle.as_ref().map(|b| (b, &state.nhc_overlays)),
            if state.show_location {
                state.user_location
            } else {
                None
            },
            state.show_radar,
            state.settings.show_scope_rings,
        );
```
After:
```rust
        scope::draw_scope_to_texture(
            if state.show_radar_data {
                state.radar_texture.as_ref()
            } else {
                None
            },
            site,
            state.pan_km,
            state.zoom,
            Some((&state.borders, state.show_borders)),
            Some((&state.alerts, state.show_watches, state.show_warnings)),
            if state.show_nhc_data {
                state.nhc_bundle.as_ref().map(|b| (b, &state.nhc_overlays))
            } else {
                None
            },
            if state.show_location {
                state.user_location
            } else {
                None
            },
            state.show_radar_data, // show_sites arg — radar markers follow data layer
            state.settings.show_scope_rings,
        );
```

No change needed in `scope.rs`: passing `None` for `radar_texture` skips the
texture draw (scope.rs:419), passing `None` for `nhc` skips
`draw_nhc_overlays` (scope.rs:587), and `show_sites` already gates the markers
(scope.rs:537). Update the stale comment at `scope.rs:536`
("Shown only while the Radar panel is open") → "Shown while the radar data
layer is on".

### 2f. Behavior notes (call out in the PR, no code)

- Radar site markers now appear whenever `show_radar_data` is on (default), not
  only when the panel is open. Double-click-to-select-site therefore works any
  time the radar layer is visible. Intended.
- `radar_panel_open`/`nhc_show_panel` mutual exclusivity (2344-2345, 2360, 2366)
  is preserved by the rename — no logic change.

**Dependencies:** Task 1. **Parallel:** no (owns state.rs + broad main.rs).
**Verify:** `cargo check -p rustywx` (expect: compiles; the two panel buttons
still show `✓` — that is fixed in Task 3), `cargo fmt`.

---

# Task 3 — Top-bar UI split + click handlers + shortcut repurpose  [tier: Sonnet]

Depends on **Task 0** (CHEVRON_RIGHT) and **Task 2** (fields renamed/added).
Owns the new element-ID contract, so UI + handlers are ONE task to prevent
ID mismatch. **Files:** `app/src/main.rs`.

### New element IDs

| ID                | Group   | Affordance | Flag toggled              |
|-------------------|---------|-----------|---------------------------|
| `btn-radar`       | Panels  | `Radar ▸` | `radar_panel_open` (reuse existing id) |
| `btn-tropical`    | Panels  | `Tropical ▸` | `nhc_show_panel` (renamed from `btn-nhc`) |
| `btn-radar-data`  | Layers  | `Radar ✓` | `show_radar_data`         |
| `btn-tropical-data`| Layers | `Tropical ✓` (+storm badge) | `show_nhc_data` |
| `btn-borders`     | Layers  | `Borders ✓` | `show_borders` (unchanged) |
| `btn-watches`     | Layers  | `Watches ✓` (+count) | `show_watches` (unchanged) |
| `btn-warnings`    | Layers  | `Warnings ✓` (+count) | `show_warnings` (unchanged) |
| `btn-location`    | Layers  | `Location ✓` | `show_location` (unchanged) |

Keep `btn-radar` as the id for the Radar **panel opener** (it already maps to
the panel via `radar_panel_open`, minimising handler churn). The old `btn-nhc`
panel button is renamed to `btn-tropical`.

### 3a. Rewrite the button row (main.rs 888-1085)

Replace the flat sequence of six buttons with: Panels group → divider → Layers
group. Keep the existing spacer (1090-1093) + window controls (1095+)
**unchanged**. Preserve `is_mobile` heights (`44.0` else `24.0`) and the outer
glass panel's `.wrap()` behavior.

**Panel-opener button pattern** (chevron, NO `✓`). Two children: label in Inter,
chevron in `SYMBOL_FONT`. Sketch for `Radar ▸`:

```rust
// ── Panels group ──────────────────────────────────────────
// Radar panel opener (chevron affordance, not a checkmark).
let radar_panel_bg = hover_tint(
    &state.hovered_ids,
    "btn-radar",
    if state.radar_panel_open { 0x0dc5b8 } else { 0x1E1B1B },
    0x1E1B1B,
);
ui.element()
    .id("btn-radar")
    .width(fit!())
    .height(fixed!(if is_mobile { 44.0 } else { 24.0 }))
    .background_color(radar_panel_bg)
    .corner_radius(4.0)
    .layout(|l| l.direction(LeftToRight).gap(6).padding((0, 8, 0, 8)).align(CenterX, CenterY))
    .accessibility(|a| a.button("Open radar panel").checked(state.radar_panel_open))
    .children(|ui| {
        ui.text("Radar", |t| t.font_size(12).color(0xE8E0DC));
        ui.text(nf::CHEVRON_RIGHT, |t| {
            t.font_size(10).font(&SYMBOL_FONT).color(0xE8E0DC)
        });
    });
```

`Tropical ▸` is the same shape with `id("btn-tropical")`, gated on
`state.nhc_show_panel`, accessibility `"Open tropical panel"`. Use `hover_tint`
with id `"btn-tropical"`. (Confirm `SYMBOL_FONT` is in scope in `main.rs`; it is
re-exported via `widgets` — the window controls already render `nf::COMPRESS`
etc. with a symbol font at 1098+, follow that exact import/qualification.)

**Divider element** between the groups:

```rust
// Divider between Panels and Layers groups.
ui.element()
    .width(fixed!(1.0))
    .height(fixed!(if is_mobile { 28.0 } else { 16.0 }))
    .background_color((1.0f32, 1.0f32, 1.0f32, 40.0f32)) // faint rule
    .empty();
```

**Layer-toggle button pattern** — this is exactly the *existing* `✓` idiom
(background flips `0x0dc5b8`/`0x1E1B1B`, label gets ` ✓` suffix when active,
`hover_tint`, `.checked(...)`). The existing Borders/Watches/Warnings/Location
buttons (925-1085) move verbatim into the Layers group. **Add two new ones:**

`Radar ✓` (new data toggle), id `btn-radar-data`:
```rust
// ── Layers group ──────────────────────────────────────────
let radar_data_bg = hover_tint(
    &state.hovered_ids, "btn-radar-data",
    if state.show_radar_data { 0x0dc5b8 } else { 0x1E1B1B }, 0x1E1B1B,
);
let radar_data_label = if state.show_radar_data { "Radar ✓" } else { "Radar" };
ui.element()
    .id("btn-radar-data")
    .width(fit!())
    .height(fixed!(if is_mobile { 44.0 } else { 24.0 }))
    .background_color(radar_data_bg)
    .corner_radius(4.0)
    .layout(|l| l.padding((0, 8, 0, 8)).align(CenterX, CenterY))
    .accessibility(|a| a.button(radar_data_label).checked(state.show_radar_data))
    .children(|ui| { ui.text(radar_data_label, |t| t.font_size(12).color(0xE8E0DC)); });
```

`Tropical ✓` (new data toggle) id `btn-tropical-data`, **carries the storm-count
badge** that currently lives on the old `btn-nhc`. Move the `storm_count` /
`nhc_badge` computation (currently lines 1033-1044) here and append the badge to
the label exactly as before:
```rust
let tropical_data_bg = hover_tint(
    &state.hovered_ids, "btn-tropical-data",
    if state.show_nhc_data { 0x0dc5b8 } else { 0x1E1B1B }, 0x1E1B1B,
);
let tropical_data_label = if state.show_nhc_data { "Tropical ✓" } else { "Tropical" };
let storm_count = state.nhc_bundle.as_ref().map(|b| b.metas.len()).unwrap_or(0);
let nhc_badge = if storm_count > 0 {
    format!(" ({storm_count})")
} else if state.nhc_fetch_fired { " (…)".to_string() } else { String::new() };
ui.element()
    .id("btn-tropical-data")
    .width(fit!())
    .height(fixed!(if is_mobile { 44.0 } else { 24.0 }))
    .background_color(tropical_data_bg)
    .corner_radius(4.0)
    .layout(|l| l.padding((0, 8, 0, 8)).align(CenterX, CenterY))
    .accessibility(|a| a.button(tropical_data_label).checked(state.show_nhc_data))
    .children(|ui| {
        ui.text(&format!("{tropical_data_label}{nhc_badge}"), |t| t.font_size(12).color(0xE8E0DC));
    });
```

Watches/Warnings keep their `count_suffix` badges (938-953 logic is reused
verbatim). Final Layers order: `Radar`, `Tropical`, `Borders`, `Watches`,
`Warnings`, `Location`.

### 3b. Click handlers (main.rs ~2295-2368)

- **`btn-radar`** (existing block 2338-2352): unchanged logic — it already
  toggles `radar_panel_open` (post-rename), keeps exclusivity
  (`nhc_show_panel = false`), and persists `settings.show_radar`. Keep as-is.
- **Rename** the `btn-nhc` handler (2355-2362) id string to `"btn-tropical"`.
  Logic unchanged: toggles `nhc_show_panel`, sets `radar_panel_open = false` on
  open. (It does not persist today — leave that behavior; note it in the PR.)
- **Add** two new layer-toggle handlers near the other overlay toggles
  (after `btn-warnings` at 2304):
```rust
    if ply.is_just_pressed("btn-radar-data") {
        state.show_radar_data = !state.show_radar_data;
        state.settings.show_radar_data = state.show_radar_data;
        state.cache.save_settings(&state.settings);
    }
    if ply.is_just_pressed("btn-tropical-data") {
        state.show_nhc_data = !state.show_nhc_data;
        state.settings.show_nhc_data = state.show_nhc_data;
        state.cache.save_settings(&state.settings);
    }
```

### 3c. Keyboard shortcut for the tropical data toggle (main.rs 2363-2368)

Repurpose the existing **`N`** key from "toggle tropical panel" to "toggle
tropical **data**", matching the B/W/A data-toggle family. Replace 2363-2368:
```rust
    if !dropdown_open && !state.location_input_focused && is_key_pressed(KeyCode::N) {
        state.show_nhc_data = !state.show_nhc_data;
        state.settings.show_nhc_data = state.show_nhc_data;
        state.cache.save_settings(&state.settings);
    }
```

**Radar data toggle: mouse-only (no keyboard).** Justification: every clean
mnemonic is taken — `R`/`V`/`W` are the product selectors (2248-2256), `B`/`A`
are borders/warnings, `T`=tilt, `F`=fullscreen. There is no non-confusing free
key for "radar data", so we leave `btn-radar-data` mouse-only rather than
overload a product key. Panels (`Radar ▸`/`Tropical ▸`) are also mouse-only
openers by design; `N` no longer opens the tropical panel.

> Pre-existing quirk, do not "fix" here: `W` is bound twice inside the same
> `!dropdown_open && !modal_open` block — SpectrumWidth product (2254) AND
> watches toggle (2283). Out of scope; leave both.

**Dependencies:** Task 0, Task 2. **Parallel:** no (broad main.rs UI region).
**Verify:** `cargo check -p rustywx`, `cargo fmt`, and confirm no lingering
`btn-nhc` string in main.rs (`grep -n 'btn-nhc' app/src/main.rs` → empty).

---

# Task 4 — Update shortcuts modal text  [tier: Haiku]

**File:** `app/src/widgets/shortcuts.rs` (OVERLAYS section, lines 143-147).

Reflect the `N` repurpose and the mouse-only data/panel controls. Change:
```rust
                            shortcut_row(ui, "N", "Toggle tropical panel");
```
to:
```rust
                            shortcut_row(ui, "N", "Toggle tropical data");
```
Leave `B`/`W`/`A` rows as-is. (Do not add rows for the mouse-only Radar data
toggle or the panel openers.)

**Dependencies:** logically depends on the Task 3 key decision, but touches no
shared code — can be done **in parallel** with Task 3 (the key mapping is fixed
by this plan). **Verify:** `cargo check -p rustywx` (compiles; text-only).

---

# Task 5 — Integration + full verification  [tier: Sonnet]

Depends on **all** prior tasks. Run after they land.

1. `cargo fmt --all` — no diff after.
2. `cargo clippy -p rustywx --all-targets` (or `cargo check`) — zero errors;
   resolve any `unused`/rename fallout (esp. leftover `show_radar` references,
   dead `storm_count`/`nhc_badge` at the old btn-nhc site, unused
   `CHEVRON_RIGHT` if Task 3 slipped).
3. `cargo test -p rustywx` — settings tests green.
4. Grep gates (all must be empty):
   - `grep -rn 'state.show_radar\b' app/src` (only `show_radar_data` should match — verify no bare `show_radar` field access remains)
   - `grep -n 'btn-nhc' app/src/main.rs`
5. Manual smoke (`cargo run -p rustywx`, or the project `run` skill):
   - Top bar shows `Radar ▸` `Tropical ▸` │ `Radar` `Tropical` `Borders`
     `Watches` `Warnings` `Location`, divider visible, no `✓` on the two
     `▸` panel buttons.
   - `Radar ▸` opens/closes the radar panel WITHOUT hiding radar data; `Radar`
     (layers) hides the radar texture + site markers while the panel stays put.
   - `Tropical ▸` opens the tropical panel; `Tropical` (layers) toggles the
     overlays; storm-count badge sits on the `Tropical` layer toggle.
   - `N` toggles tropical data (not the panel); `B`/`W`/`A` unchanged.
   - Narrow the window < 900px: buttons wrap, heights = 44, divider intact.
   - Window controls still on the right edge.
6. Confirm persistence: toggle `Radar`/`Tropical` data + open a panel, restart,
   state restores (data flags from `show_radar_data`/`show_nhc_data`, radar
   panel default closed).

---

## Task graph

```
Task 0 (Haiku)  ─┐
Task 1 (Haiku)  ─┼─► Task 2 (Sonnet) ─► Task 3 (Sonnet) ─► Task 5 (Sonnet, integration)
                 │                        ▲
Task 4 (Haiku) ──┴────────────────────────┘  (parallel; no code dep)
```

- **Parallel at start:** Task 0, Task 1, Task 4.
- **Task 2** needs Task 1. **Task 3** needs Task 0 + Task 2. **Task 5** last.
