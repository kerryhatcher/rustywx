# Demo mode — historical severe-weather scenes

**Date:** 2026-07-23
**Status:** Approved (design discussed in session; user selected CLI-flag activation)

## Purpose

Load a fixed historical NEXRAD Level II volume instead of live data, so rendering
quality can be evaluated and tuned against canonical severe-weather scenes —
and compared side-by-side with reference viewers (GRLevel3/GRLevelX) showing the
same archived volume. Two curated events ship initially:

| Key | Event | Volume | Why |
|-----|-------|--------|-----|
| `moore2013` | Moore, OK EF5 tornado, 2013-05-20 20:16 UTC | `KTLX20130520_201643_V06.gz` | Hook echo, debris ball, velocity couplet; dual-pol debris signature. Tornado on the ground over Moore at this scan. |
| `harvey2017` | Hurricane Harvey Cat-4 landfall, 2017-08-26 03:04 UTC | `KCRP20170826_030439_V06` | Symmetric eye + eyewall near the radar; broad stratiform shield. |

Both keys verified present in the `unidata-nexrad-level2` S3 archive (the same
bucket the app's `nexrad-data` crate already reads). KTLX and KCRP both exist in
the app's site table (`geo.rs`), so map centering, city overlays, and the site
readout work unchanged.

## Activation

```
rustywx --demo moore2013
rustywx --demo harvey2017
rustywx --demo /path/to/KXXX20250101_000000_V06[.gz]
```

Unknown event key → print the available events to stderr and exit non-zero.
No UI changes for activation (a site-picker entry may follow later).

## Components

### `demo.rs` (new module)

- `DemoEvent { key, label, site, date, volume_name }` — static registry of the
  two events.
- `lookup(key) -> Option<&DemoEvent>` and `available() -> impl Iterator` for the
  error listing.
- `maybe_gunzip(bytes: Vec<u8>) -> Result<Vec<u8>>` — sniffs the gzip magic
  (`1f 8b`) and decompresses via `flate2` (new direct dependency; already in the
  tree transitively through `zip`). 2013-era archive volumes are gzip-wrapped;
  `nexrad_data::volume::File` cannot decode them raw (documented by an existing
  test in `data.rs`).

### `data.rs`

- Factor the decode tail of `fetch_latest_scan` (volume file → `ScanData` with
  Nyquist map and KDP derivation) into a shared helper so the demo path and the
  live path cannot drift.
- `fetch_demo_scan(event) -> Result<ScanData>`:
  1. Raw volume bytes from Ply storage cache key `demo_volume_<volume_name>`,
     else `nexrad_data::aws::archive` download (match the identifier by name in
     `list_files(site, date)`), then cache the raw bytes.
  2. `maybe_gunzip` → `VolumeFile::new` → shared decode helper.
- Local-path variant: read the file, same gunzip + decode, no caching.

### `main.rs` / `state.rs`

- Parse `--demo <arg>` from `std::env::args` at startup.
- Demo startup: set the site to the event's site, load the demo scan, and mark
  `state.demo: Option<DemoInfo>`.
- Polling suppression: while demo is active, the poll worker does not replace
  the scan (otherwise live KTLX/KCRP data overwrites the demo volume within one
  poll cycle). Selecting a different site in the UI clears demo mode and
  resumes normal live behavior for that site.
- Status line prefixes `DEMO — <label>` so screenshots are unambiguous.

## Error handling

- Download/decode failures surface through the existing scan-error status path
  (same as live fetch failures); the app stays up.
- Corrupt cached demo bytes: decode failure removes the cache entry (mirrors
  the existing corrupt-scan-cache self-healing) so the next run re-downloads.

## Testing

- Unit: registry lookup (known/unknown keys), gzip sniff/passthrough of
  `maybe_gunzip` (gzip bytes round-trip, non-gzip bytes unchanged).
- The shared decode helper is exercised by existing live-path tests.
- Network download + full decode: manual verification by running both events
  (the same path live fetching already uses in production).

## Out of scope (deferred)

- Site-picker demo entries.
- Multi-volume animation of an event.
- Rendering-quality gaps identified against GRLevel3 (baked range floor hiding
  10–20 dBZ echo, effective display resolution, palette range) — separate work,
  evaluated *using* this demo mode.
