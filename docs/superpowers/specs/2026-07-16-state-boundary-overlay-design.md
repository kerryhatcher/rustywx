# rustywx — State Boundary Overlay Design

**Date:** 2026-07-16
**Status:** Approved design, pending implementation plan

## Purpose

Draw US state border lines on the radarscope for geographic orientation,
alongside the existing range rings, cardinal spokes, and city markers. A
viewer unfamiliar with the exact shape of central Georgia can tell at a
glance which side of a storm is still in-state versus crossing into Alabama,
South Carolina, or Florida.

## Decisions

| Question | Decision |
|---|---|
| Geographic scope | Only border segments within the 230 km display radius around KJGX (`scope::MAX_RANGE_KM`) — not a national or pannable map |
| States involved | Georgia plus its neighbors whose border could plausibly fall within range: Alabama, South Carolina, Florida |
| Data source | US Census Bureau TIGERweb REST API, `TIGERweb/State_County/MapServer/0` (States layer), queried with `f=geojson` |
| Data delivery | Loaded from disk at runtime, not embedded in the binary |
| Cache location | `~/.rustywx/state_borders.geojson` |
| Missing-file behavior | Auto-download once on first run, then read the local cache on every later launch |
| Fetch failure behavior | Log a status note and run without borders for this session; retry on the *next* launch (no in-session retry loop) |
| Fetch mechanism | A dedicated one-shot thread doing a single blocking HTTP GET — separate from the existing tokio-based NEXRAD worker |
| Parsing | `serde_json::Value`, walked manually for `geometry.coordinates` — no dedicated `geojson` crate |

### Why TIGERweb over a shapefile

TIGERweb's State layer (verified live against the service, not from memory)
supports `f=geojson` directly:
`https://tigerweb.geo.census.gov/arcgis/rest/services/TIGERweb/State_County/MapServer/0/query?where=STUSAB+IN+('GA','AL','SC','FL')&outFields=STUSAB,NAME&f=geojson`
returns ready-to-use GeoJSON polygons with a `STUSAB` field for state postal
abbreviation. This avoids adding a shapefile parser, a zip-extraction step, or
a geometry-simplification library for what is ultimately background
reference lines — consistent with the project's existing minimal-dependency
approach (see `CONTRIBUTING.md`'s invariants section).

### Why disk-cached rather than embedded

The boundary data is fetched once and doesn't change at a pace that matters
for this app, but embedding it in the binary would mean recompiling to
update it and would bloat the executable with data most builds won't need to
regenerate. A cache file under `~/.rustywx/` is the standard convention for
this kind of app-managed local data, keeps the binary itself small, and lets
the app self-bootstrap on first run without any manual setup step.

## Architecture

A third thread joins the existing UI thread and NEXRAD worker thread:

- **UI thread** — unchanged; still never performs I/O.
- **NEXRAD worker thread** (existing, `data.rs`) — unchanged.
- **Border-fetch thread** (new) — runs once, not a loop:
  1. Check whether `~/.rustywx/state_borders.geojson` exists.
  2. If yes, read and parse it.
  3. If no, GET it from TIGERweb, write it to that path, then parse it.
  4. Send the parsed rings (or an error) over its own `mpsc` channel and wake
     the UI via `request_repaint()`, mirroring the existing worker's
     wake-up pattern.

This is a separate channel and message type from `WorkerMessage`
(`data.rs`), not a reuse of it — the two data sources have unrelated
lifecycles (one polls forever, the other runs once) and unrelated failure
modes, and conflating them would make `data.rs` responsible for a concern
it doesn't otherwise have.

## Modules

- `borders.rs` (new) — owns the cache-check/fetch/parse logic described
  above. Exposes something like
  `pub fn load_or_fetch(cache_path: &Path) -> Result<Vec<Vec<(f64, f64)>>>`
  (one `Vec<(f64, f64)>` per polygon ring, lat/lon pairs) plus the
  spawn-a-thread wrapper that calls it and reports over a channel.
- `geo.rs` (unchanged) — `range_bearing` and `polar_to_offset` are reused
  as-is for projecting border vertices, the same functions already used for
  city markers.
- `scope.rs` — `draw_scope` gains a borders-drawing step: for each ring,
  project every vertex to a screen offset and connect them with line
  segments, in a color distinct from the range-ring grid color.
- `app.rs` — owns the border-fetch channel receiver alongside the existing
  NEXRAD receiver, drains both, and holds the parsed rings in app state
  once available.

## Data Flow

```
~/.rustywx/state_borders.geojson (cache)
      │  present? read it   │  absent? GET from TIGERweb, write cache, then read it
      ▼
serde_json::Value
      │  walk geometry.coordinates per feature
      ▼
Vec<Vec<(f64, f64)>>  (rings, lat/lon)
      │  channel → UI thread
      ▼
app state
      │  geo::range_bearing + geo::polar_to_offset per vertex (scope.rs)
      ▼
line segments drawn by the existing egui painter, alongside rings/spokes/cities
```

## Error Handling

- Fetch fails (offline, TIGERweb unreachable, non-200 response): the border
  thread sends an error message; the UI logs a one-line status note and
  simply doesn't draw borders for this session. This does not block or
  degrade any existing functionality (radar display, product/tilt controls
  all work identically with or without borders loaded).
- Cache file exists but is malformed or unreadable: treated the same as a
  fetch failure — log and skip, don't crash, don't delete the file
  automatically (a corrupt file is left for a human to investigate or
  delete, rather than being silently overwritten every launch).
- No retry within a single run: if the fetch fails, the app doesn't attempt
  it again until the next launch. This avoids adding a second polling/backoff
  schedule for what is fundamentally a one-time bootstrap.

## Testing

- Unit tests: GeoJSON-to-rings parsing against a small hand-written synthetic
  `FeatureCollection` fixture (a couple of features, a couple of rings each)
  — no network access required.
- Integration test: an opt-in (`#[ignore]`-tagged) live test that hits the
  real TIGERweb endpoint and asserts it returns at least one ring for
  Georgia, mirroring the existing pattern in `tests/network.rs`.
- Manual/visual verification: run the app and confirm border lines appear in
  plausible positions relative to the existing city markers (e.g., the
  Georgia/Alabama line should appear west of Macon, roughly along the
  expected bearing).

## Dependencies (anticipated)

`ureq` (new — small blocking HTTP client for the one-shot border fetch) and
`serde_json` (new — GeoJSON parsing). No change to existing dependencies
(`eframe`, `egui`, `nexrad-data`, `nexrad-model`, `tokio`, `chrono`,
`anyhow`). Exact versions verified against docs during implementation
planning, per this project's existing convention.

## Out of Scope

- County lines, congressional districts, or any TIGERweb layer other than
  State.
- Panning/zooming the scope independent of the radar's own 230 km range.
- Configurable state list — the GA/AL/SC/FL set is fixed for the KJGX site;
  changing the radar site (see `CONTRIBUTING.md`'s "point at a different
  radar site" recipe) would also mean updating this list by hand.
- Re-fetching the cached file automatically (e.g., on a schedule, or if
  Census data changes) — a human can delete the cache file to force a
  re-fetch.
