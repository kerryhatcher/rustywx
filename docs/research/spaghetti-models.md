# Spaghetti models (TC track/intensity guidance): data sources

Research date: 2026-10-09. All live checks below were run that day with `curl`.
No code was changed.

## TL;DR

- Cyclocane does not own this data. Its spaghetti images are credited to **UWM** (`web.uwm.edu/hurricane-models`), which renders them from the **NHC/ATCF a-deck**. Cyclocane itself says the paths are "experimental path data based on weather models" and links to NHC for official info.
- Use the a-deck directly: `https://ftp.nhc.noaa.gov/atcf/aid_public/a{basin}{NN}{YYYY}.dat.gz`. It is HTTPS, public domain, about 350 KB gzipped for an active storm, and one fetch gives every model and every cycle.
- The `CurrentStorms.json` id maps straight to the filename: `al092026` becomes `aal092026.dat.gz`.

## 1. Authoritative sources

| Source | URL | Notes |
|---|---|---|
| NHC ATCF a-decks (**primary**) | `https://ftp.nhc.noaa.gov/atcf/aid_public/` | Public guidance files. Verified HTTP 200 over HTTPS. |
| HTTP variant | `http://ftp.nhc.noaa.gov/atcf/aid_public/...` | Verified 301 redirect to HTTPS. Use HTTPS directly. |
| Full (non-public) a-deck dir | `.../atcf/aid/` | Includes licence-restricted models. Not for us. |
| Best track (b-deck) | `https://ftp.nhc.noaa.gov/atcf/btk/bal092026.dat` | Verified 200. Plain text, not gzipped. Observed track, useful for a past-track line. |
| Tech descriptor list | `https://ftp.nhc.noaa.gov/atcf/docs/nhc_techlist.dat` | 54 KB table of tech id, description, and NHC's own default colour index. |
| ATCF README | `https://ftp.nhc.noaa.gov/atcf/README` | Directory layout, naming, and the data notice. |
| Format spec (NRL) | `https://www.nrlmry.navy.mil/atcf_web/docs/database/new/abdeck.txt` | Referenced by the README. It returned "Access Denied" to curl and WebFetch from this machine. Field list in section 2 was reconstructed from the live data, the README, and the techlist. Check it by hand in a browser before relying on the rarer columns. |
| Model summary | `https://www.nhc.noaa.gov/modelsummary.shtml` | Model catalogue, early vs late, disclaimers. |
| Active storm ids | `https://www.nhc.noaa.gov/CurrentStorms.json` | Already used in `app/src/nhc.rs:26`. |
| NCAR/RAL TC Guidance Project | `https://hurricanes.ral.ucar.edu/realtime/` | The page loads, but I found no machine-readable feed. It is a plot viewer built from the same ATCF data. I did not pursue it. |

**File details (live, `aal092026.dat.gz`)**

- Compression: gzip only (`Content-Type: application/x-gzip`). The uncompressed `.dat` path returns 404.
- Size: 352,802 bytes gzipped, 2,935,171 bytes uncompressed, 26,380 lines, for a storm with about 4 days of cycles. Other files in the index range from 17 KB to 2.7 MB.
- Headers: `Last-Modified` and `ETag` are present, so conditional GETs (`If-None-Match`) are available. The index showed the file touched at 14:54 UTC for a 12Z cycle.
- Cadence: rows are appended per synoptic cycle (00, 06, 12, 18Z). The file is rewritten as model runs finish, so a new cycle fills in over several hours. In the 12Z cycle at 15:36 UTC, only 437 of the rows for that init time existed, against 1,284 and 1,535 for the two earlier cycles. A 5 to 15 minute poll is plenty. `CurrentStorms.json` itself is `cache-control: max-age=300`.
- Coverage: the directory holds **all storms of the season**, not only active ones. This includes numbered storms (01 to 30), invests (90 to 99), and the Eastern and Central Pacific (`aep`, `acp`). Per the README, 80 to 89 are training storms to ignore. Old storms stay in the directory with their last-modified date. The directory index (~100+ rows) is HTML; do not scrape it.
- Naming: `a<basin><NN><YYYY>.dat.gz`, with basin `al`, `ep`, or `cp`.
- Not covered: West Pacific, Indian Ocean, and other basins. Those would come from NRL/JTWC, which I did not evaluate.

## 2. A-deck record format

Comma-separated ASCII, fixed-pad with spaces. Always `trim()` every field. Verified against live rows.

| # | Field | Example | Need? | Notes |
|---|---|---|---|---|
| 1 | BASIN | `AL` | yes | `AL`, `EP`, `CP` |
| 2 | CY | `09` | yes | Storm number |
| 3 | YYYYMMDDHH | `2026100912` | yes | Init time (UTC) of the run. The "cycle". |
| 4 | TECHNUM/MIN | `03` | no | `00` CARQ/WRNG, `01` CARQ in practice, `03` for the guidance rows seen |
| 5 | TECH | `AVNI` | yes | Model id (section 3) |
| 6 | TAU | `12` | yes | Forecast hour. Can be **negative** (CARQ past positions) and **non-multiples of 6** (OFCL has `3`). |
| 7 | LatN/S | `270N` | yes | Tenths of a degree plus hemisphere letter: `270N` is 27.0 N. Southern hemisphere is `S`. |
| 8 | LonE/W | `876W` | yes | Tenths plus `E`/`W`: `876W` is -87.6. **Not zero padded** (`  0W`). |
| 9 | VMAX | `105` | yes | Knots. `0` means "no value" for some techs, not calm. |
| 10 | MSLP | `959` | optional | mb. Mostly `0` (missing) for models. Populated for CARQ and some OFCL rows. |
| 11 | TY | `HU` | optional | `DB`, `TD`, `TS`, `HU`, `EX`, `SS`, `SD`, `LO`, `WV`, `XX`. Blank for most models. |
| 12 | RAD | `34` | row key | Wind radii threshold in kt: 34, 50, 64, or `0` (none). |
| 13 | WINDCODE | `NEQ` | no | Quadrant code. `AAA` = full circle. |
| 14 to 17 | RAD1..RAD4 | `180,110,80,160` | no | NE, SE, SW, NW radii in nm. |
| 18+ | pressure of outer closed isobar, radius, RMW, gusts, eye, subregion, max seas, forecaster initials, direction, speed | `1008,250,20,...` | no | Only on OFCL/CARQ. Rows reach 46 fields. |
| ~28 | STORMNAME | `ISAIAS` | no | On CARQ/OFCL rows. `INVEST` for invests. |

Field counts observed in the live file: 18 (21,829 rows), 23, 28, 30, 36, 44, 46. Most model rows are short, so a parser must read only the leading columns and tolerate missing ones.

**Quirks that matter**

1. **Duplicate rows per (tech, tau).** One physical forecast point is emitted once per wind-radii threshold (34/50/64). The same position and vmax repeat. Dedupe on `(init, tech, tau)`, keeping the first row. Example, `OFCL` tau 12 appears 3 times with identical lat/lon (see sample).
2. **Intensity-only techs have no position.** `IVCN`, `SHIP`, `DSHP`, `LGEM`, `SHF5`, `OCD5`, `NNIC`, `NNIB` and similar emit lat/lon like `0N, 0W` (or the tech's own track). **Never plot a 0N/0W vertex.** Route them to the intensity chart only. `SHIP`'s rows carried real positions in the sample, `IVCN` did not. Do not assume either way, so filter on `lat == 0 && lon == 0`.
3. **Negative taus.** `CARQ` rows at tau -24..0 are the past 24 h track and the current fix. They are not a forecast.
4. `OFCL` has an extra tau 3. Filter to multiples of 6 or 12 if drawing uniform markers.
5. Lat/lon parse: strip the last letter, divide the integer by 10, negate for `S` and `W`.
6. Records for the same cycle are not in a guaranteed order, and the file mixes cycles. Group by init time.
7. **Late vs interpolated.** Models without an `I` or `2` suffix (`AVNO`, `HWRF`, `CTCX`) are *late*: they finish after the official forecast. Their interpolated twins (`AVNI`, `HWFI`, `CTCI`) are what forecasters saw. Use the interpolated ones. The ones ending in `2` are the 12-hour-interpolated variants (older run).
8. Names can be reused across years (this file's storm is `ISAIAS` again in 2026), so key storms by id plus year, not name.

## 3. Model catalogue

Source for descriptions: NHC model summary (<https://www.nhc.noaa.gov/modelsummary.shtml>) and `nhc_techlist.dat`. The model summary marks entries with `*` as restricted: **public access is limited by provider agreements** (ECMWF/UKMET/some Google). Those do not appear in `aid_public` as plain ids, though `UKX2` does appear as the GFS-tracker version of UKMET.

| Tech | What | Type | Early/Late | Group |
|---|---|---|---|---|
| OFCL | NHC official forecast | track+int | n/a | **Official** |
| OFCI / OFC2 | OFCL interpolated 6 / 12 h | track+int | interp | Official (hide by default) |
| CARQ | Combined ARQ: past positions and the current fix (negative taus, tau 0) | best-track-ish | n/a | **Observed** |
| WRNG | Warning position | obs | n/a | Observed |
| OCD5 / BCD5 | CLP5 plus Decay-SHIFOR | skill baseline | n/a | Baseline |
| AVNI | GFS interpolated | dynamical, track+int | early | Global |
| CMC2 / CMCI | Canadian global (12 h / 6 h interp) | dynamical | early | Global |
| NVG2 / NVGI | NAVGEM (12 h / 6 h interp) | dynamical | early | Global |
| UKX2 / UKXI | UKMET via GFS tracker | dynamical | early | Global |
| HWFI / HWF2 | HWRF interp | regional, track+int+radii | early | Hurricane models |
| HFAI / HFBI | HAFS-A / HAFS-B interp | regional | early | Hurricane models |
| HMNI | HMON interp | regional | early | Hurricane models |
| CTCI | COAMPS-TC interp | regional | early | Hurricane models |
| AEMI | GEFS ensemble mean interp | ensemble mean | early | Ensemble |
| CEM2 / CEMI | Canadian ensemble mean | ensemble mean | early | Ensemble |
| GDMI | Google DeepMind ensemble mean interp | AI/ML | early | AI |
| TVCN | Track variable consensus (>=2 members) | consensus, track | early | **Consensus** |
| HCCA | HFIP corrected consensus | corrected consensus | early | Consensus |
| RYOC | Forecaster-created consensus (any combination) | consensus | early | Consensus |
| IVCN | Intensity variable consensus | consensus, intensity only | early | Consensus (intensity) |
| NNIC / NNIB | Neural-net intensity consensus (bias-corrected / raw) | consensus, intensity only | early | Consensus (intensity) |
| RVCN | Wind-radii consensus | radii only | early | Skip (radii) |
| SHIP / DSHP | SHIPS / Decay SHIPS | stat-dynamical, intensity only | early | Statistical (intensity) |
| LGEM | Logistic growth equation model | statistical, intensity only | early | Statistical (intensity) |
| SHF5 | SHIFOR5 | statistical baseline, intensity | early | Baseline |
| TCLP | Trajectory-CLIPER | statistical, track+int | early | Baseline |
| CLP5 | CLIPER5 | statistical baseline, track | early | Baseline |
| DRCL | Wind-radii CLIPER | radii only | early | Skip (radii) |
| XTRP | Extrapolation of last 12 h motion | simple | early | Baseline |
| TABS/TABM/TABD | Trajectory-and-beta (shallow/medium/deep) | track only | late (GFS run) | Trajectory |
| AP01..AP30 | GFS ensemble members | ensemble member | late | Ensemble members (**30 extra lines, hide**) |
| AVNO, HWRF, CTCX, HMON, HFSA, HFSB, NVGM, AEMN, CEMN, GDMN | late (non-interpolated) versions | | late | Hide, duplicate their `I` twins |
| RI25 | Rapid-intensification probability, not a track | prob | n/a | Skip |

The cyclocane codes I looked up in `nhc_techlist.dat` were all found there. The 12Z cycle in the live file carried these ids (it is still filling in, so other cycles have more): `AEMI AVNI CARQ CEM2 CLP5 CMC2 CTCI DRCL DSHP GDMI HCCA HFAI HFBI HMNI HWFI IVCN LGEM NNIB NNIC NVG2 OCD5 OFCI OFCL RVCN RYOC SHF5 SHIP TABD TABM TABS TCLP TVCN UKX2 XTRP`.

**Suggested defaults for a track overlay**

- Always on: `OFCL` (thick, white or black), `CARQ` past track (thin grey).
- On by default (the "spaghetti"): `AVNI`, `HWFI`, `HFAI`, `HFBI`, `HMNI`, `CTCI`, `NVG2`, `CMC2`, `UKX2`, `AEMI`, `CEM2`, `GDMI`, `TVCN`, `HCCA`.
- Off by default behind a "more" toggle: `TABS/M/D`, `XTRP`, `CLP5`, `TCLP`, `AP01..AP30`, `OFCI`, and the late non-`I` models.
- Intensity chart only: `OFCL`, `HWFI`, `HFAI`, `HFBI`, `HMNI`, `CTCI`, `AVNI`, `IVCN`, `NNIC`, `DSHP`, `LGEM`, `SHIP`, `HCCA`, `RYOC`. Those with `0N 0W` rows belong here only.
- Colours: group by family, not per model. For example, hurricane models (HWFI/HFAI/HFBI/HMNI/CTCI) in warm hues, global models (AVNI/NVG2/CMC2/UKX2) in cool hues, ensembles in greens, consensus (TVCN/HCCA/RYOC/IVCN) bold purple. The alternative is NHC's own colour index: the `COLOR` column in `nhc_techlist.dat` (OFCL is 28, AVNI 47, TVCN 21) is an ATCF palette index, which could be mapped to RGB, but the table gives numbers only. Cyclocane colours the track by wind, which is a rendering choice we could copy using the `VMAX` field with the Saffir-Simpson thresholds (34/64/83/96/113/137 kt).

## 4. Licensing, attribution, disclaimers

- NHC site terms (<https://www.nhc.noaa.gov/disclaimer.shtml>): government server information is **public domain** "unless specifically annotated otherwise". Content "shall not be modified in content" and then presented as official government material. **Attribution is not required**, but crediting is good practice.
- The ATCF README notice (<https://ftp.nhc.noaa.gov/atcf/README>) is the important one for us. The data are "provided 'as-is' in a realtime automated fashion", users "are REQUIRED to perform quality control", and the files "can differ from information issued in official NHC products". Official forecasts are the WTNT2x/WTPZ2x messages.
- NHC model summary: "On average, NHC official forecasts have smaller errors than any individual model. Users should consult the official NHC and local National Weather Service forecasts rather than relying solely on model output."
- Models marked `*` have public access restricted by the data provider. Stay in `aid_public` and we avoid them.
- Cyclocane's own disclaimer is "experimental path data based on weather models", linking to official info.

**Suggested in-app text** (shown whenever the overlay is on): "Model guidance from NHC ATCF (aid_public). Individual models are not official forecasts. The NHC forecast (OFCL) is the official track. Data are automated and may contain errors."

## 5. Practical concerns

- **Rate limits:** none documented. Neither the disclaimer page nor the README states one, and `nhcwebmaster@noaa.gov` is the contact for questions. Be polite: one fetch per active storm, conditional GET with `ETag`/`Last-Modified`, refresh no faster than every 10 to 15 minutes, and only while the overlay is open. A 350 KB gzip per storm is small.
- **User-Agent:** `nhc.rs:1062` already sends `rustywx/0.3` for text products. Reuse that header.
- **gzip in Rust:** `flate2 1.1.9` (with `miniz_oxide 0.8.9`) is **already in `Cargo.lock`**, but only transitively. `app/Cargo.toml` lists `zip = "2"` and no direct `flate2`, so we would add `flate2 = "1"` to `app/Cargo.toml`. It will resolve to the version already locked. Decode with `flate2::read::GzDecoder` into a `String`. Expect about 3 MB uncompressed, so decode on a worker and parse streaming by line, not through the UI thread. Check whether `Ply net::get` hands back raw bytes. KMZ handling at `nhc.rs:489` already deals with bytes, so it probably does.
- **CORS:** irrelevant for the desktop app. For a future web build, `ftp.nhc.noaa.gov` is an Apache server and I did not test for `Access-Control-Allow-Origin` (a response header check was cheap, but the point is moot here). See `docs/cors-quick-reference.md`.
- **Storm id to filename:** `CurrentStorms.json` gives `id` like `al092026` (verified live: `al092026` Isaias, `ep182026` Rachel, `ep202026` Simon). Filename is `format!("a{id}.dat.gz")`, which gives `aal092026.dat.gz`, `aep182026.dat.gz`. Verified: both `aal092026.dat.gz` and `aep*` exist in the directory. Prefix with `https://ftp.nhc.noaa.gov/atcf/aid_public/`.
  - Check the existing `StormMeta` id field: `nhc.rs` already has `binNumber` (e.g. `AT4`) and `id`. Use `id`, not `binNumber`.
  - Invests (`al90`..`al99`) use the same pattern and may appear in `CurrentStorms.json` too. Per README, ids 80 to 89 should be dropped.
- **Latest init cycle:** parse `YYYYMMDDHH` from column 3 and take the max. But the newest cycle is incomplete for hours (see cadence above), so a robust approach is: take the newest cycle that has `OFCL` rows, and for each model take its newest init that is no older than about 12 to 18 h, or simply fall back to the previous cycle for any model absent in the newest one. Mixing cycles is what the interpolated `I`/`2` models are designed to avoid, so prefer the single newest cycle that contains `OFCL`, and show its init time in the UI.
- **Time to first paint:** `just run` smoke test per `AGENTS.md` applies once code lands.

## 6. Live sample (`aal092026.dat.gz`, fetched 2026-10-09 15:36 UTC, 12Z cycle)

Lines are trimmed to the leading 15 columns only for readability (`OFCL` and `CARQ` rows are wider in the file).

```text
# CARQ: past positions (negative tau), then current fix with wind radii
AL, 09, 2026100912, 01, CARQ, -24, 234N,  906W,  75,    0, HU,  34, AAA,    0,    0,    0,    0, ...
AL, 09, 2026100912, 01, CARQ, -12, 248N,  890W,  85,    0, HU,  34, AAA,    0,    0,    0,    0, ...
AL, 09, 2026100912, 01, CARQ,   0, 270N,  876W, 105,  959, HU,  34, NEQ,  180,  110,   80,  160, ...
# OFCL: three rows per tau (34/50/64 kt radii), same position
AL, 09, 2026100912, 03, OFCL,   0, 270N,  876W, 105,    0, HU,  34, NEQ,  180,  110,   80,  160, ...
AL, 09, 2026100912, 03, OFCL,   0, 270N,  876W, 105,    0, HU,  50, NEQ,   60,   70,   40,   60, ...
AL, 09, 2026100912, 03, OFCL,   3, 277N,  873W, 105,  959, HU,  34, NEQ,  180,  110,   80,  160, ...
AL, 09, 2026100912, 03, OFCL,  12, 297N,  868W, 100,    0, HU,  34, NEQ,  140,  100,   60,  110, ...
# Models
AL, 09, 2026100912, 03, AVNI,  12, 301N,  870W,  87,    0,   ,  34, NEQ,  110,   71,   59,   91
AL, 09, 2026100912, 03, HWFI,   0, 270N,  876W, 105,    0,   ,  34, NEQ,  180,  110,   80,  160
AL, 09, 2026100912, 03, CTCI,   0, 270N,  876W, 105,    0,   ,  34, NEQ,  180,  110,   80,  160
AL, 09, 2026100912, 03, AEMI,  24, 322N,  880W,  69,    0,   ,  34, NEQ,    0,    0,    0,    0
AL, 09, 2026100912, 03, CMC2,  24, 325N,  877W,  85,    0,   ,  34, NEQ,   24,   13,   26,    7
AL, 09, 2026100912, 03, NVG2,  24, 325N,  871W,  85,    0,   ,  34, NEQ,   74,    9,   37,   91
AL, 09, 2026100912, 03, RYOC,  12, 297N,  868W,  86,    0,   ,   0,    ,    0,    0,    0,    0
AL, 09, 2026100912, 03, SHIP,  24, 322N,  871W,  70,    0, EX,   0,    ,    0,    0,    0,    0
# Intensity-only consensus: NO position (0N, 0W). Do not plot.
AL, 09, 2026100912, 03, IVCN,  12,   0N,    0W,  89,    0,   ,   0,    ,    0,    0,    0,    0
AL, 09, 2026100912, 03, IVCN,  24,   0N,    0W,  47,    0,   ,   0,    ,    0,    0,    0,    0
# Invest-era CARQ rows for the same storm (note INVEST name, DB type)
AL, 09, 2026100512, 01, CARQ, -24, 220N,  965W,  15,    0, DB,  34, AAA, ... INVEST
```

Stats from the same file: 26,380 lines, 17 init cycles (2026100512 to 2026100912), about 80 distinct tech ids, `OFCL` taus 0, 3, 12, 24, 36, 48, 60 in the latest cycle (this storm is forecast to dissipate or go extratropical by 60 h, so horizons vary by storm and cycle).

## Open items / caveats

- The NRL `abdeck.txt` spec was unreachable here, so columns 18 onward are described from observed data, not the spec. Only the first 17 columns are needed for this feature and those were checked against live rows.
- I did not test: conditional-GET behaviour (`If-None-Match` returning 304), CORS headers, or whether the guidance files go away when a storm ends (the index shows entries back to June, so they appear to persist for the season).
- Colour recommendation is a suggestion. The techlist's colour numbers are unmapped ATCF palette indices.
- NCAR/RAL has no obvious data feed. I left it as a viewer, not a source.
- Interpolation detail (the `I` and `2` suffix meaning 6 h and 12 h) is from `nhc_techlist.dat` descriptions. The model summary page describes the concept but does not list suffixes.
