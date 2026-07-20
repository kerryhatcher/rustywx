# rustywx → Ply Port — De-Risking Plan

**Goal:** Answer the open questions and validate the riskiest assumptions in
`ply-port-plan.md` before committing to full-stage execution. Each item below
is self-contained — pick it up, run it, report back.

**Total effort:** ~1.5 days (10–12 hours)

---

## Quick Wins (Research — read docs, run a command, report)

These are binary questions. Each takes 15–30 minutes.

### R1: NHC CORS Support

**Blocks:** Stage 5 (Tropical), Stage 8 (WASM)

**Question:** Do `www.nhc.noaa.gov` and `mapservices.weather.noaa.gov` serve
`Access-Control-Allow-Origin` headers?

**Steps:**

```bash
# Test 1: CurrentStorms.json
curl -sI -H "Origin: http://localhost:8080" \
  "https://www.nhc.noaa.gov/CurrentStorms.json" \
  | grep -i access-control

# Test 2: GIS MapServer (pick any layer)
curl -sI -H "Origin: http://localhost:8080" \
  "https://mapservices.weather.noaa.gov/tropical/rest/services/tropical/NHC_tropical_weather_summary/MapServer?f=json" \
  | grep -i access-control
```

**Success:** Response includes `Access-Control-Allow-Origin: *` (or echoes the
origin). Either means WASM can fetch directly.

**Failure:** No CORS header. WASM will need the same relay proxy as NEXRAD
(see Stage 8). Note this in the report — it changes the WASM architecture.

**Report template:**

```
### R1: NHC CORS
- CurrentStorms.json: [CORS confirmed / NO CORS — header was: ___]
- GIS MapServer:      [CORS confirmed / NO CORS — header was: ___]
- Implication:        [no change needed / WASM needs relay proxy for NHC too]
```

---

### R2: nexrad-data Bucket Migration

**Blocks:** Stage 2 (Live Data)

**Question:** Does `nexrad-data` 1.0.0-rc.7 point to the new
`unidata-nexrad-level2` bucket, or the deprecated `noaa-nexrad-level2`?

The old bucket is scheduled for shutdown September 2025. If the crate hasn't
migrated yet, we need to pin a version, fork, or find a workaround.

**Steps:**

1. Check the crate's source:
   ```bash
   grep -r "noaa-nexrad-level2\|unidata-nexrad-level2" \
     ~/.cargo/registry/src/*/nexrad-data-*/src/
   ```

2. Check the upstream repo for recent commits about the migration:
   ```bash
   # Open in browser or use gh CLI
   gh api repos/danielway/nexrad/commits --jq '.[0:5].[].commit.message'
   ```

3. If the crate still points to the old bucket, test whether the new bucket
   works with the same API by swapping the bucket name in a local test.

**Report template:**

```
### R2: Bucket Migration
- Crate version tested: ___
- Bucket used:          [unidata-nexrad-level2 / noaa-nexrad-level2]
- Migration status:     [already migrated / not yet migrated / unclear]
- Action needed:        [none / pin version / file upstream issue / fork]
```

---

### R3: `plyx web` with Manual Scaffold

**Blocks:** Stage 8 (Cross-Platform)

**Question:** The spike was scaffolded manually (not `plyx init`). Does
`plyx web` work on a non-init project, or does it expect a specific layout?

**Steps:**

```bash
cd ply-spike
cargo install plyx          # if not already installed
plyx web                     # try building for WASM
```

If it fails, compare the ply-spike directory structure against what
`plyx init` produces in a temp directory.

**Report template:**

```
### R3: plyx web compatibility
- plyx web result:     [builds successfully / failed with: ___]
- plyx apk result:     [builds successfully / failed with: ___ / not tested]
- Action needed:       [none / restructure project to match plyx init layout]
```

---

### R4: `webbrowser` Crate on WASM

**Blocks:** Stage 5 (NHC panel external links), Stage 8 (WASM)

**Question:** Does `webbrowser::open("https://...")` work when compiled to
WASM, or does it need JS interop?

**Steps:**

1. Check the crate's platform support:
   ```bash
   # Look at the crate docs or source for WASM support
   grep -r "wasm\|target_arch" ~/.cargo/registry/src/*/webbrowser-*/src/
   ```

2. If unclear, add a test button to the ply-spike that calls
   `webbrowser::open("https://example.com")` and build for WASM. Check
   browser console for errors.

**Report template:**

```
### R4: webbrowser WASM
- WASM support:        [yes, works / no, needs JS interop / unclear]
- Fallback if no:      [window.open via JS interop / remove external links on WASM]
```

---

## Spikes (Write code, validate, report)

These are the riskiest assumptions. Each takes 1–3 hours. Do them in the
ply-spike directory. Commit the spike code to a branch so we can reference
it later — it doesn't need to be production quality.

### S1: Custom GLSL Blur Shader — HIGHEST PRIORITY

**Blocks:** Stage 6 (Observatory Look)

**Risk:** The frosted glass effect is the visual centerpiece of Stage 6.
Ply has no built-in blur shader. If a custom GLSL blur shader doesn't work
or tanks performance, Stage 6 needs a redesign.

**What to build:**

A minimal two-pass Gaussian blur in GLSL ES 1.00:

1. **Horizontal pass shader** — samples neighboring pixels horizontally,
   weights by a Gaussian kernel, outputs intermediate result
2. **Vertical pass shader** — samples the horizontal pass output vertically
3. Apply to a semi-transparent panel via Ply's `.shader()` API
4. Place the panel over moving content (e.g., the radar scope) so the blur
   is visible

**Starter code (horizontal pass):**

```glsl
// blur_horizontal.frag — GLSL ES 1.00
varying vec2 v_tex_coord;
uniform sampler2D u_texture;
uniform float u_radius;      // blur radius in pixels
uniform vec2 u_texel_size;   // 1.0 / texture_size

void main() {
    vec4 sum = vec4(0.0);
    float total = 0.0;
    int radius = int(u_radius);

    for (int x = -radius; x <= radius; x++) {
        float weight = exp(-float(x * x) / (2.0 * u_radius * u_radius));
        sum += texture2D(u_texture, v_tex_coord + vec2(float(x) * u_texel_size.x, 0.0)) * weight;
        total += weight;
    }

    gl_FragColor = sum / total;
}
```

**Steps:**

1. Write both shader files, place in `ply-spike/assets/shaders/`
2. Load and apply via Ply's shader API (check Ply docs for exact syntax —
   likely `.shader(&blur_shader, |s| s.uniform("u_radius", 8.0))`)
3. Render a panel with the shader over the radar scope
4. Measure frame time with and without the shader (target: <16ms at 60fps)
5. Test different radius values (4, 8, 12, 16) for visual quality vs perf

**Success criteria:**
- [ ] Blur effect visible on panel (content behind panel is blurred)
- [ ] Frame time stays under 16ms on desktop
- [ ] Works on both desktop (OpenGL) and WASM (WebGL) — test WASM if possible
- [ ] No visual artifacts at panel edges

**Fallback if it fails:** Use Ply's `GLOW` built-in shader + reduced panel
opacity (e.g., 0.85 alpha). This gives a "glowing glass" look instead of
true frosted glass. Still attractive, just different.

**Report template:**

```
### S1: GLSL Blur Shader
- Approach:            [two-pass Gaussian / single-pass / other]
- Visual quality:      [excellent / good / acceptable / poor]
- Frame time impact:   [___ms with shader vs ___ms without]
- Radius sweet spot:   [___px — best balance of blur vs perf]
- WASM tested:         [yes, works / yes, but ___fps / not tested]
- Verdict:             [ready for Stage 6 / needs optimization / use GLOW fallback]
- Code location:       [branch: ___, path: ___]
```

---

### S2: nexrad-data + Ply Integration

**Blocks:** Stage 2 (Live Data)

**Risk:** The plan assumes the existing `data.rs` pattern (tokio on a
background thread + `mpsc::channel`) works with Ply/macroquad. The research
notes "some users report runtime crashes." This must be validated before
writing any Stage 2 code.

**What to build:**

1. Add `nexrad-data`, `nexrad-model`, `tokio`, `chrono` to ply-spike's
   `Cargo.toml`
2. Port the worker pattern from `src/data.rs` into the ply-spike:
   - Spawn `std::thread` with a tokio `Runtime`
   - Call `nexrad_data::aws::archive::list_files` + `download_file`
   - Decode via `nexrad_model` → `ScanData`
   - Send over `mpsc::channel` to main thread
3. In the Ply game loop, `rx.try_recv()` each frame
4. On receipt, rasterize and display (reuse the existing `scope::rasterize`)
5. Test: site switch triggers new fetch, error handling works, clean shutdown
   (drop tx, join thread)

**Steps:**

```bash
cd ply-spike

# Add dependencies
cargo add nexrad-data --version "1.0.0-rc.7"
cargo add nexrad-model --version "1.0.0-rc.2"
cargo add tokio --features "rt,time"
cargo add chrono --features "serde"
cargo add anyhow

# Build and test
cargo run
```

**Success criteria:**
- [ ] Real radar data from at least one site (e.g., KJGX) displays in the Ply window
- [ ] Site switching (arrow keys) fetches new data
- [ ] Error displayed gracefully if network is unavailable
- [ ] App shuts down cleanly (no panic on close, thread joins)
- [ ] Works for 10+ fetch cycles without crash or memory growth

**Report template:**

```
### S2: nexrad-data + Ply
- Sites tested:        [KJGX, ___]
- Fetch latency:       [___ms from request to display]
- Thread pattern:      [stable / crashes after ___ cycles / other]
- Shutdown:            [clean / panics with: ___]
- Memory:              [stable / grows ___MB per cycle]
- Verdict:             [ready for Stage 2 / needs ___ fix]
- Code location:       [branch: ___, path: ___]
```

---

### S3: Ply Composite Widget — Searchable Dropdown

**Blocks:** Stage 3 (Custom Widgets)

**Risk:** The spike only has simple `.id("btn-refl")` buttons. A searchable
dropdown with 160+ sites, type-to-filter, and outside-click-to-close is a
different order of complexity. It's unclear whether Ply's declarative UI
supports this or if we need raw macroquad draw calls.

**What to build:**

ONE widget — the site selector dropdown — end-to-end:

1. **Closed state:** A button showing current site ID + name
2. **Open state:** A floating panel with a text input at top and a
   scrollable list of sites below
3. **Type-to-filter:** Typing filters the list in real time
4. **Selection:** Click a site → close dropdown, trigger site change
5. **Outside click:** Clicking outside the dropdown closes it
6. **Keyboard:** Arrow keys navigate list, Enter selects, Escape closes

**Approach A (Ply-native):** Compose with `.element()` chains, use
`ply.is_just_pressed()` for clicks, capture keyboard via macroquad's
`get_char_pressed()`.

**Approach B (hybrid):** Use Ply for the button, raw macroquad
`draw_rectangle` + `draw_text` for the floating dropdown panel, manual
hit-testing for clicks.

Try Approach A first. Fall back to B if Ply can't express the interaction.

**Steps:**

1. Create `ply-spike/src/widgets/dropdown.rs`
2. Build the site dropdown using `geo::RADAR_SITES` (160+ entries)
3. Wire it into `main.rs` — replace the arrow-key site switching
4. Test: click, type, select, outside-click, keyboard nav

**Success criteria:**
- [ ] Dropdown opens on click
- [ ] Typing filters the 160+ site list in real time (<1 frame lag)
- [ ] Clicking a site selects it and closes the dropdown
- [ ] Clicking outside the dropdown closes it
- [ ] Arrow keys + Enter work for keyboard-only navigation
- [ ] Escape closes the dropdown
- [ ] Dropdown renders correctly at window edges (doesn't clip offscreen)

**Report template:**

```
### S3: Composite Widget
- Approach used:       [Ply-native / hybrid Ply+macroquad / pure macroquad]
- Filter perf:         [instant / ___ms lag with 160 sites]
- Edge cases found:    [none / ___]
- Verdict:             [ready for Stage 3 / needs ___ / use simpler widget]
- Code location:       [branch: ___, path: ___]
```

---

### S4: Texture Lifecycle & Memory

**Blocks:** Stage 1–2 (long-running app stability)

**Risk:** The spike creates a new `Texture2D` on every rerasterization
without explicitly deleting the old one. Over hours of auto-refresh (every
2 minutes = 30 textures/hour), this could leak GPU memory.

**What to build:**

A stress test in the ply-spike:

1. Add a counter that forces rerasterization every frame for 1000 frames
2. Create a new `Texture2D` each time (simulating the current pattern)
3. Monitor: does the old texture get dropped? Does GPU memory grow?
4. Test explicit cleanup: call `std::mem::replace` on the
   `Option<Texture2D>` before creating a new one — does the old one Drop?
5. Check if Ply/macroquad has a `delete_texture` function

**Steps:**

```bash
cd ply-spike
# Add a stress test mode: hold 'T' to rerasterize every frame
cargo run
# Watch GPU memory: nvidia-smi -l 1  (NVIDIA) or radeontop (AMD)
```

**Success criteria:**
- [ ] GPU memory stable after 1000 rerasterizations
- [ ] No frame time degradation over the test
- [ ] Explicit drop/delete pattern identified (or confirmed unnecessary)

**Report template:**

```
### S4: Texture Lifecycle
- Pattern tested:      [replace Option / explicit delete / rely on Drop]
- GPU memory after 1000 cycles: [stable at ___MB / grew from ___ to ___MB]
- Frame time:          [stable at ___ms / degraded from ___ to ___ms]
- Cleanup mechanism:   [Drop works / need explicit delete_texture / unclear]
- Verdict:             [no issue / need to add explicit cleanup in Stage 1]
```

---

### S5: Ply `storage` Async Ergonomics

**Blocks:** Stage 2 (cache), Stage 7 (settings)

**Risk:** The `storage` API is async (`.await`). Called inside the game loop,
it could block the frame. We need to understand the ergonomics before
designing the cache and settings layers.

**What to build:**

A minimal test in the ply-spike:

1. Call `storage.save_bytes("test.bin", &[0u8; 1024*1024])` (1MB write)
   inside the game loop
2. Measure frame time before/after
3. Call `storage.load_bytes("test.bin")` inside the game loop
4. Measure frame time
5. Test: does it need to be spawned? Does Ply's async runtime handle it?

**Steps:**

```bash
cd ply-spike
# Add ply-engine storage feature if not already enabled
# Add a test key that triggers save/load
cargo run
```

**Success criteria:**
- [ ] Save 1MB: frame time impact <2ms
- [ ] Load 1MB: frame time impact <2ms
- [ ] No visible frame drop during save/load
- [ ] Pattern documented: call directly vs spawn

**Report template:**

```
### S5: Storage Async
- Save 1MB frame time: [___ms]
- Load 1MB frame time: [___ms]
- Blocks frame:        [yes / no / only for >___MB]
- Recommended pattern: [call directly / spawn / batch writes]
- Verdict:             [ready / needs workaround]
```

---

### S6: Ply `net` Concurrent Requests

**Blocks:** Stage 4 (Borders & Alerts), Stage 5 (NHC)

**Risk:** Stages 4–5 fire multiple `net::get()` calls (borders, alerts, NHC
JSON, NHC GIS). The plan assumes you can have multiple in-flight requests
with different IDs. This needs validation.

**What to build:**

1. Fire 3 `net::get()` calls simultaneously with different IDs:
   - `"test-a"`: a fast-responding URL
   - `"test-b"`: a slow-responding URL
   - `"test-c"`: a URL that returns an error
2. Poll all three each frame via `net::request("test-a")`, etc.
3. Verify: all three complete independently, none are dropped, errors are
   reported per-request
4. Test: calling `net::get("test-a", ...)` while a previous `"test-a"`
   request is still in flight — does it cancel, queue, or error?

**Steps:**

```bash
cd ply-spike
# Add a test mode that fires 3 concurrent requests
cargo run
```

**Success criteria:**
- [ ] 3 concurrent requests all complete independently
- [ ] Error response reported correctly for the failing URL
- [ ] Re-firing with same ID: [cancels old / queues / errors / other: ___]
- [ ] No panic or hang

**Report template:**

```
### S6: Net Concurrent
- Concurrent requests: [all 3 completed / only ___ completed]
- Same-ID behavior:    [cancels old / queues / errors with: ___]
- Error handling:       [works / error response not delivered]
- Verdict:              [ready / need to serialize requests / need unique IDs per fetch]
```

---

## Execution Order

```
Session 1 (2–3 hours) — Binary questions + core pipeline:
  R1 (NHC CORS)         15 min
  R2 (bucket migration) 15 min
  S2 (nexrad-data+Ply)  1–2 hours   ← validates Stage 2

Session 2 (2–3 hours) — Visual & interaction risk:
  S1 (blur shader)      2–3 hours   ← highest technical risk

Session 3 (2–3 hours) — Widget & infrastructure:
  S3 (dropdown widget)  2–3 hours   ← validates Stage 3

Session 4 (2–3 hours) — Remaining items:
  S4 (texture lifecycle) 30 min
  S5 (storage async)     30 min
  S6 (net concurrent)    30 min
  R3 (plyx web)          30 min
  R4 (webbrowser WASM)   15 min
```

**Total: ~10–12 hours (1.5 days)**

If time is tight, the must-do items are: **R1, R2, S2, S1, S3**.
Everything else can be done just-in-time before its respective stage.

---

## Reporting Back

For each item, fill in its report template. Collect all results into a
single response. Flag anything that changes the plan — especially:

- **Architecture changes** (e.g., "WASM needs relay proxy for NHC too")
- **Schedule changes** (e.g., "blur shader needs 2 extra days")
- **Fallback activations** (e.g., "using GLOW instead of custom blur")
- **New dependencies** (e.g., "need a JS interop crate for webbrowser")

Commit all spike code to a branch (`spike/de-risk`) so it's referenceable
during the actual stages.
