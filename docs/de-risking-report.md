# De-Risking Report — rustywx → Ply Port

**Date:** 2025-07-19
**Status:** Research items R1–R4 complete; spikes S1–S6 pending

This report answers the binary research questions from `de-risking-plan.md`.
Each item maps to a specific risk in `ply-port-plan.md`.

---

## R1: NHC CORS Support

**Blocks:** Stage 5 (Tropical), Stage 8 (WASM)

### Results

| Endpoint | CORS? | Response Headers |
|---|---|---|
| `www.nhc.noaa.gov/CurrentStorms.json` | **❌ NO** | No `Access-Control-Allow-Origin` header returned |
| `mapservices.weather.noaa.gov/.../MapServer?f=json` | **✅ YES** | `Access-Control-Allow-Origin: http://localhost:8080` (echoes origin), `Access-Control-Allow-Credentials: true` |

### Implication

**Architecture change required for WASM.** The NHC `CurrentStorms.json` endpoint does NOT serve CORS headers. WASM builds will need the same relay proxy used for NEXRAD data to fetch storm metadata. The GIS MapServer endpoint works from WASM — overlays (forecast cone, track, wind probabilities) can be fetched directly.

**Action:** The WASM relay proxy (Stage 8) must proxy `www.nhc.noaa.gov/CurrentStorms.json` in addition to the NEXRAD S3 bucket. This is a single additional route on the same Cloudflare Worker — minimal extra effort.

**Desktop builds:** Unaffected. Native HTTP clients (reqwest/ureq) don't require CORS headers.

---

## R2: nexrad-data Bucket Migration

**Blocks:** Stage 2 (Live Data)

### Results

| Crate Version | Archive Bucket | Status |
|---|---|---|
| `nexrad-data 0.2.0` | `noaa-nexrad-level2` | **Deprecated** — shutdown September 2025 |
| `nexrad-data 1.0.0-rc.7` | `unidata-nexrad-level2` | **Migrated** ✅ |

The project's `Cargo.toml` already pins `nexrad-data = "1.0.0-rc.7"`, which points to the new `unidata-nexrad-level2` bucket. Both the archive and real-time buckets are on the new Unidata infrastructure.

### Implication

**No action needed.** The crate version already in use has completed the migration. The deprecated `noaa-nexrad-level2` bucket is not referenced anywhere in the dependency tree.

**Verification:** The real-time bucket (`unidata-nexrad-level2-chunks`) was already on Unidata in both versions — only the archive bucket changed.

---

## R3: `plyx web` Compatibility with Manual Scaffold

**Blocks:** Stage 8 (Cross-Platform)

### Results

| Test | Result |
|---|---|
| `plyx` installed? | Yes — v0.2.3 at `~/.cargo/bin/plyx` |
| `plyx web` in ply-spike | **Failed** — `wasm32-unknown-unknown` target not installed |
| Root cause | Missing Rust target, not a plyx/project issue |

The error was:
```
error[E0463]: can't find crate for `std`
  = note: the `wasm32-unknown-unknown` target may not be installed
```

### Implication

**Environment issue, not a project structure issue.** The ply-spike's manual scaffold does not appear to conflict with `plyx web`. The failure is purely a missing `rustup target add wasm32-unknown-unknown`.

**Action:** Install the WASM target and re-test:
```bash
rustup target add wasm32-unknown-unknown
cd ply-spike && plyx web
```

This should be re-tested before Stage 8, but there's no indication the manual scaffold is incompatible. The `plyx web` command is essentially a wrapper around `cargo build --target wasm32-unknown-unknown` with Ply-specific post-processing — it doesn't require a specific project layout.

---

## R4: `webbrowser` Crate on WASM

**Blocks:** Stage 5 (NHC panel external links), Stage 8 (WASM)

### Results

| Crate Version | WASM Support | Mechanism |
|---|---|---|
| `webbrowser 0.8.15` | ✅ Yes | `web_sys::window().open_with_url_and_target()` |
| `webbrowser 1.2.1` | ✅ Yes | Same mechanism, updated `web_sys` |

Both versions have a dedicated `src/wasm.rs` module. The WASM implementation:
- Ignores the `Browser` parameter (always opens in the same browser)
- Uses `window.open(url, target_hint)` via `web_sys`
- Validates that URLs are http/https before opening
- Supports `dry_run` mode (returns Ok if window exists)
- Logs to browser console with the `wasm-console` feature
- Handles popup blocking gracefully (returns an error if `window.open` returns null)

### Implication

**No action needed.** The `webbrowser` crate works on WASM out of the box. External links in the NHC panel (graphics products, text products) will open in a new browser tab. The crate handles the edge cases (popup blockers, invalid URLs) with proper error returns.

**Note:** The project currently uses `webbrowser = "0.8"`. Consider updating to `"1.2"` for the latest WASM improvements, but 0.8 is sufficient.

---

## Summary: Plan Impact

| Item | Finding | Plan Change |
|---|---|---|
| **R1** | NHC CurrentStorms.json has NO CORS; GIS MapServer has CORS | WASM relay proxy must also proxy `www.nhc.noaa.gov/CurrentStorms.json` |
| **R2** | Bucket already migrated in rc.7 | None — crate version is correct |
| **R3** | `plyx web` fails on missing WASM target | Install `wasm32-unknown-unknown` target; re-test before Stage 8 |
| **R4** | `webbrowser` has full WASM support | None — works as-is |

### Architecture Changes

1. **WASM relay proxy scope expanded.** The Stage 8 relay proxy (Cloudflare Worker) now needs to proxy two additional endpoints beyond NEXRAD S3:
   - `www.nhc.noaa.gov/CurrentStorms.json` (no CORS)
   - NHC GIS MapServer is fine — direct fetch from WASM works

   This is a minor scope increase — adding routes to an existing Worker is trivial.

### Schedule Impact

None of these findings change the stage schedule. The WASM relay proxy was already scoped for Stage 8; it just needs two extra routes.

### New Dependencies

None. All findings confirm existing dependency choices.

---

## Next Steps: Spikes S1–S6

The research items are complete. The remaining de-risking work is the six code spikes (S1–S6) in `de-risking-plan.md`. These require active development in the `ply-spike` directory:

| Priority | Spike | Effort | Blocks |
|---|---|---|---|
| 🔴 **Highest** | S1: Custom GLSL Blur Shader | 2–3h | Stage 6 | ✅ **DONE** — see `research/spike-s1-report.md` |
| 🔴 **High** | S2: nexrad-data + Ply Integration | 1–2h | Stage 2 | ✅ **DONE** — see `research/spike-s2-report.md` |
| 🟡 **High** | S3: Composite Widget (Dropdown) | 2–3h | Stage 3 | ✅ **DONE** — see `research/spike-s3-report.md` |
| 🟢 **Medium** | S4: Texture Lifecycle & Memory | 30m | Stage 1–2 | ✅ **DONE** — see `research/spike-s4-s5-s6-report.md` |
| 🟢 **Medium** | S5: Storage Async Ergonomics | 30m | Stage 2, 7 | ✅ **DONE** — see `research/spike-s4-s5-s6-report.md` |
| 🟢 **Medium** | S6: Net Concurrent Requests | 30m | Stage 4–5 | ✅ **DONE** — see `research/spike-s4-s5-s6-report.md` |

**Recommended execution order** (from the plan):
1. Session 1: S2 (nexrad-data + Ply) — ✅ **COMPLETE**
2. Session 2: S1 (blur shader) — ✅ **COMPLETE**
3. Session 3: S3 (dropdown widget) — ✅ **COMPLETE**
4. Session 4: S4, S5, S6 — ✅ **COMPLETE**

All spike code should be committed to the `spike/de-risk` branch for reference during the actual stages.
