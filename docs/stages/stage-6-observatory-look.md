# Stage 6: "Observatory Look" — Visual Design

**Status:** ✅ Complete (tagged `v0.4.0-stage6`)
**Tag:** `v0.4.0-stage6`

## Goal

The full "Observatory" visual identity — frosted glass, animations,
typography, responsive layout.

## Scope

- `widgets/glass_panel.rs` — reusable frosted glass panel wrapper
- Frosted glass styling on all panels. Ply's `built-in-shaders` has GLOW but
  no Gaussian blur — **author a custom GLSL ES 1.00 fragment shader** for the
  blur effect. Apply via `.shader(&BLUR_SHADER, |s| s.uniform("u_radius", 8.0))`.
  Validated by Spike S1.
- Dark gradient background with noise texture (use `GRADIENT_RADIAL` built-in
  or a custom shader)
- Teal/cyan accent color on all interactive elements
- Inter font for UI chrome, JetBrains Mono for scope/data labels.
  Use `FontAsset::Path("assets/fonts/Inter.ttf")` — verify font loading
  works on all platforms. Bundle fonts in `assets/fonts/`.
- Rich animations:
  - Panel slide-in/out with spring physics
  - Hover glow on buttons and interactive elements (use `GLOW` built-in shader)
  - Pulse animation on new data arrival
  - Staggered entrance on app launch
  - Radar sweep line (optional visual flourish)
- Auto-hiding control bar (fades after 3s inactivity)
- Responsive layout:
  - Desktop: top control bar, side NHC panel, bottom status
  - Mobile (<900px): bottom control bar, full-screen NHC panel, larger touch targets
- Loading skeleton while data loads
- Empty states ("No active storms", "No alerts", etc.)
- **Spline-based color tables** — replace the current step-based color lookup
  in `colors.rs` with cubic Hermite spline (Cardinal Spline) interpolation for
  smoother, more professional-looking color ramps. The thesis by Yi Ru (2007)
  uses this approach for interactive transfer functions; the same technique
  applies to static NWS-style color tables. See
  `docs/post-v1-multi-site-animation.md` for the thesis reference.
- **Bilinear range interpolation** — replace the current single-gate range
  lookup in `scope.rs` rasterization with bilinear interpolation across both
  azimuth and range. The current code already interpolates between two
  nearest radials (azimuth axis) but uses a hard gate index for range,
  producing blocky gate-aligned artifacts. Kvasov et al. show that 4-gate
  bilinear interpolation (weighted blend of the curvilinear trapezoid
  formed by 2 adjacent radials × 2 adjacent range gates) gives ~90%
  improvement over nearest-neighbor, with max error only 4.3–6.7%. The
  formula is a simple weighted blend:
  `Z = (1-ξ)(1-η)·Z_ij + (1-ξ)η·Z_i(j+1) + ξ(1-η)·Z_(i+1)j + ξη·Z_(i+1)(j+1)`
  where ξ = relative azimuth, η = relative range. See
  `docs/research/Weatherradardatavisualizationusingfirst-orderinterpolation.md`
  for the full paper.

## Notes from Spike S1

- The blur shader was validated during spike S1 and removed from Stage 1
  as a spike artifact. The shader file (`assets/shaders/blur.frag`) remains
  in the repo. Re-add the `BLUR_SHADER` static and glass panel UI in this
  stage.
- **GLSL ES 1.00** is the target version (not 3.00 as docs suggest — see
  Ply issue #4). Use `#version 100` with `attribute`/`varying`/`gl_FragColor`.
- Custom shaders **must** declare `u_resolution` (vec2) and `u_position`
  (vec2) auto-uniforms even if unused (see Ply issue #5).
- Constant loop bounds are required in GLSL ES 1.00 on some drivers.

### Shader loading: `ShaderAsset::Source` + `include_str!` (settled)

The loading mechanism is **decided, not open**. Use `ShaderAsset::Source`
with `include_str!`; do **not** use `ShaderAsset::Path`. This was the
approach validated by Spike S1 (see `research/spike-s1-report.md`) and is
the same idiom every Ply built-in shader uses (`GLOW`, `GRADIENT_RADIAL`,
etc. are all `ShaderAsset::Source { fragment: include_str!(...) }`).

Rationale, confirmed against the `ply-engine-1.1.1` source:

- `ShaderAsset::Path` reads the file at runtime via `std::fs::read_to_string`
  and **panics** if the file is missing (`unwrap_or_else(|e| panic!(...))`).
  It also resolves relative to the process CWD, which is arbitrary when a
  user launches the desktop app from a dock, launcher, or `.desktop` file.
  That is a flaky release bug, not a dev convenience.
- `ShaderAsset::Source` embeds the GLSL in the binary at compile time
  (`Cow::Borrowed`, zero-copy at runtime) with no filesystem or CWD
  dependency.

**Keep `assets/shaders/blur.frag` in the repo.** It is not redundant — it
is the *input* to `include_str!`, read at build time and baked into the
binary. This mirrors the built-in shaders keeping their `.frag.glsl`
source files next to `mod.rs`.

**Do not introduce a `ShaderAsset::Path` variant alongside `Source`.**
There is no reason to load the shader two ways; if dev hot-reload is ever
wanted, swap `Source` for `Path` as a temporary dev-time change, never both
at once.

**`include_str!` path is relative to the source file containing the call,**
not the crate root or CWD. Get the `../` count right or it fails to
compile (a loud, immediate error — not a silent risk):

- If `BLUR_SHADER` lives in `main.rs`: `include_str!("../assets/shaders/blur.frag")`
- If it lives in `widgets/glass_panel.rs` (recommended — co-locate the
  shader with its consumer): `include_str!("../../assets/shaders/blur.frag")`

Recommended declaration:

```rust
use ply_engine::shaders::ShaderAsset;

pub const BLUR_SHADER: ShaderAsset = ShaderAsset::Source {
    file_name: "rustywx_blur",
    fragment: include_str!("../../assets/shaders/blur.frag"),
};
```

## Dependencies to add

- `ply-engine` features: add `built-in-shaders` and `text-styling`:
  ```toml
  ply-engine = { version = "1.1", features = ["net", "net-json", "storage", "built-in-shaders", "text-styling"] }
  ```
  Both are **zero-cost API gates** — in `ply-engine`'s `Cargo.toml` they
  are declared as `built-in-shaders = []` and `text-styling = []` with no
  optional dependencies, so enabling them adds no compile-time
  dependencies, only code paths.
- `built-in-shaders` is **required for this stage regardless of the blur
  shader decision**: the stage uses `GLOW` (hover glow on buttons) and
  `GRADIENT_RADIAL` (dark gradient background), both of which live behind
  `#[cfg(feature = "built-in-shaders")]`. The custom blur shader itself
  does not need the feature (the `shaders` module and `.shader()` are
  always available), but the stage cannot ship without it anyway.

## Delivered

- `widgets/glass_panel.rs` — reusable frosted-glass panel module:
  - `BLUR_SHADER` — custom GLSL ES 1.00 Gaussian blur fragment shader,
    embedded via `include_str!("../../assets/shaders/blur.frag")` as
    `ShaderAsset::Source` (loading mechanism settled — see "Notes from
    Spike S1" above).
  - Observatory theme constants: `ACCENT` (teal `#0dc5b8`), `ACCENT_GLOW`,
    `GLASS_BG`, `GLASS_BORDER`, `PANEL_BG`, `CARD_BG`, `TEXT_PRIMARY`,
    `TEXT_MUTED` — mirroring `docs/observatory-mockup.html`.
  - `glass()` — styling helper applying semi-transparent frosted background +
    subtle border to any element builder (blur shader NOT applied — see Lessons
    Learned).
- `colors.rs` — replaced stepped `banded()` lookup with Catmull-Rom
  (Cardinal Spline, tension 0) interpolation. Passes exactly through every
  NWS anchor at its threshold; smooth in between. 10 unit tests (was 4).
- `scope.rs` — replaced hard gate index with bilinear interpolation across
  azimuth (ξ) and range (η). New testable `bilinear_sample()` helper
  renormalises by available weight so missing gates don't bias the sample.
  6 new unit tests.
- `state.rs` — new animation fields: `start_time`, `nhc_anim_start`,
  `nhc_anim_from`, `pulse_time`, `sweep_angle`, `hovered_ids`,
  `last_click_time`, `last_click_pos` (double-click site selection).
- `main.rs` — visual identity wiring:
  - Fonts: Inter (Regular + Bold) as default UI font, JetBrains Mono for
    scope/data labels (status bar, zoom/pan readout, dBZ legend).
  - Dark radial-gradient observatory background (`draw_observatory_background`).
  - Radar sweep line with trailing teal fade (`draw_radar_sweep`), fading
    in on launch via the entrance animation.
  - Frosted glass on control bar, NHC panel, status bar, and modal.
  - Teal accent on active overlay buttons; hover-glow tint via
    previous-frame pointer-over state (`hover_tint`).
  - Control bar always visible (auto-hide removed per user request).
  - NHC panel spring slide-in (`ease_out_elastic`) from off-screen right;
    full-screen on mobile (<900px).
  - Pulse-on-new-data: status colour blends toward accent for ~1.2s.
  - Staggered entrance: radar sweep fades in over 0.6s on launch.
  - Loading skeleton: pulsing "◌ Loading radar data…" while first scan loads.
  - Empty states: "No active storms" (NHC), alerts count badge shows `(0)`.
  - Responsive: <900px → NHC full-screen, control bar 48px, touch targets
    ≥44px on overlay buttons. (Bottom-docked controls deferred to post-v1.)
- `Cargo.toml` — added `built-in-shaders` and `text-styling` features to
  `ply-engine` (both zero-cost API gates, no optional dependencies).
- `assets/fonts/` — added Inter-Regular.ttf, Inter-Bold.ttf, JetBrainsMono.ttf
  (SIL OFL, from Google Fonts gstatic).
- `lib.rs` — added `pub mod glass_panel` via `widgets/mod.rs`, and `pub mod cities`.

### Additional features (added during Stage 6, beyond original scope)

- **Radar site markers + double-click selection** — all radar sites drawn as
  teal accent markers at their geographic positions (culled to screen bounds,
  visible while panning). Double-clicking a marker selects it as the active
  site. New `scope::project_site()` helper shared by the draw and hit-test.
- **City data from Natural Earth GeoJSON** (`cities.rs` + `assets/cities.geojson`)
  — replaced the hand-curated `geo::CITIES` list (1,194 entries, no population)
  with Natural Earth 10m populated places trimmed to North America (1,369
  cities, 174 KB), embedded via `include_str!`. Progressive disclosure: a
  `min_population_for_zoom()` formula gates which cities appear (biggest
  when zoomed out, progressively smaller as you zoom in). Greedy label
  placement with rectangle collision avoidance so names never overlap.
- **Default site + persistence** — first launch defaults to KFFC (Atlanta);
  the last-selected site is persisted via Ply storage and restored on next
  launch. New `cache::save_site()` / `load_site()`.


## Implementation Data

| Item | Result |
|---|---|
| New modules | 1 (`widgets/glass_panel.rs`) |
| New unit tests | 12 (colors: +6, scope bilinear: +6) → total 69 |
| Custom shader | 1 (`BLUR_SHADER` via `include_str!`) |
| Ply features added | 2 (`built-in-shaders`, `text-styling`) |
| Fonts added | 3 (Inter Regular, Inter Bold, JetBrains Mono) |
| Theme constants | 8 (accent, glass, panel, card, text) |
| Animations | sweep line, entrance fade, NHC spring slide, data pulse, auto-hide |
| Responsive breakpoint | 900px (mobile: full-screen NHC, 48px bar, 44px touch targets) |

## Lessons Learned

### Font loading: CWD-sensitive `FontAsset::Path`

`FontAsset::Path` resolves relative to the process CWD, not the crate
root. The workspace `Justfile` runs `cd ply-spike && cargo run` so
`assets/fonts/...` resolves correctly, but `cargo run -p rustywx` from the
workspace root fails. This is the same class of fragility that motivated
`ShaderAsset::Source` + `include_str!` for shaders (see above). Fonts are
loaded once at startup with a panic on failure; a future stage should
switch to `FontAsset::Bytes { data: include_bytes!(...) }` for the same
robustness, or rely on Stage 8 packaging to fix the CWD.

### Catmull-Rom overshoot on sharp NWS transitions

The NWS colour table has inherently sharp transitions (e.g. green→yellow
is a ~250-unit R jump over 5 dBZ). Catmull-Rom interpolation overshoots
near these transitions, occasionally clamping a channel to 255. This is
expected spline behaviour, not a bug — the result is still smooth and
banding-free. Tests were written to verify continuity and bound the
per-step jump rather than assume linearity.

### Bilinear with missing-gate renormalisation

Real NEXRAD sweeps have `None` gates (below threshold / range folded).
The bilinear sampler excludes missing corners and renormalises by
available weight, so partial data doesn't bias the sample toward zero.
This gracefully degrades to azimuth-only or single-gate interpolation
when range neighbours are missing.

### Blur shader on content panels degrades text

The custom GLSL blur shader (`BLUR_SHADER`) was validated by Spike S1 and
compiles/runs cleanly. However, Ply's `.shader()` API captures an element
**and its children** to an offscreen buffer and applies the fragment shader
to that capture. Applied to a content-bearing panel, this blurs the panel's
own text — making it unreadable — and in practice caused the NHC sidebar to
render empty (the offscreen capture + blur of large floating panels is
fragile).

**Decision:** the frosted-glass look is achieved via the semi-transparent
dark background (`GLASS_BG`) composited over the gradient scope, plus the
subtle light border — NOT via the blur shader. `BLUR_SHADER` remains defined
and validated for potential use on non-text decorative elements, but is not
applied to any content panel. The `glass()` helper applies only
`background_color` + `corner_radius` + `border` (no `.shader()`).


## Deliverable

App matches the `observatory-mockup.html` look and feel.

## Validation

- [x] Frosted glass panels (semi-transparent dark background + border; blur shader
  defined but not applied to content panels — see Lessons Learned)
- [~] ~~Control bar auto-hides~~ — auto-hide removed per user request (always visible)
- [x] Buttons glow on hover with accent color
- [x] NHC panel slides with spring animation
- [x] New data triggers subtle pulse
- [x] Inter font used in UI, JetBrains Mono on scope
- [~] Responsive <900px: NHC full-screen + ≥44px touch targets ✓;
  ~~controls move to bottom~~ deferred to [post-v1](../post-v1-multi-site-animation.md#4-mobile-bottom-control-bar)
- [x] Touch targets ≥44px on mobile
- [x] Loading state shows skeleton/spinner
- [x] Empty states show helpful messages
- [x] Spline-based color tables produce smooth gradients (no visible banding)
- [x] Bilinear range interpolation eliminates gate-aligned blockiness in scope
- [x] `git push` → CI passes → `git tag v0.4.0-stage6` → `git push --tags`  ✅
