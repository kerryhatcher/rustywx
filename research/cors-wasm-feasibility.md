# CORS & WASM Feasibility — Research Findings

**Date:** 2025-07-19
**Source:** AWS Open Data docs, NWS API GitHub discussions, NHC docs, StackOverflow

---

## 1. NEXRAD S3 Bucket — ❌ NO CORS

**Bucket:** `unidata-nexrad-level2` (moved from `noaa-nexrad-level2`, old bucket deprecated Sept 2025)

AWS S3 buckets do NOT support CORS for anonymous cross-origin requests by default. The bucket is configured for public read access via AWS API (sigv4 signing or `--no-sign-request` CLI flag), but browser-based WASM apps cannot make direct requests without CORS headers.

**Evidence:**
- StackOverflow question confirms CORS errors when accessing S3 from Blazor WASM
- The `nexrad-mapbox-backend` project uses a **Python Flask proxy** to relay S3 data to the browser frontend
- The `level2-browser` project fetches NEXRAD files by having users provide URLs — it doesn't access S3 directly from the browser

**Impact on WASM target:** A relay/proxy is REQUIRED for radar data on WASM. Options:
- Cloudflare Worker proxy in front of the S3 bucket
- Server-side component that the WASM app calls
- Accept that WASM is desktop-data-only or synthetic-data mode

**Note:** The `nexrad` crate has a `wasm` feature that enables `aws` (reqwest-based S3 access). This may work if reqwest's WASM backend handles S3 requests, but CORS is still a server-side concern.

## 2. NWS API (api.weather.gov) — ✅ CORS SUPPORTED

**Evidence from GitHub discussion #312:**
- Response headers include: `access-control-allow-origin: *`
- Confirmed in multiple curl traces from the NWS API GitHub discussions
- The API explicitly supports browser-based access

**Caveat:** Setting a custom `User-Agent` header triggers a CORS preflight request, and the preflight response does NOT allow `User-Agent` as an accepted header. The NWS maintainer clarified:
> "The suggestion to make [User-Agent] unique is for server-side applications (or proxies). Using the default UA in a client-side application is fine."

**For WASM:** The NWS alerts API (`api.weather.gov/alerts/active`) should work from WASM as long as you don't set a custom User-Agent header. The browser's default User-Agent is fine.

## 3. NHC CurrentStorms.json — ⚠️ LIKELY CORS-SUPPORTED

**URL:** `https://www.nhc.noaa.gov/CurrentStorms.json`

The Home Assistant community uses this endpoint via REST API (server-side), not browser-side. No direct evidence of CORS headers was found in the research.

However, since it's served from `www.nhc.noaa.gov` (same domain as the NHC website) and NOAA generally supports open data access, it's likely CORS-enabled. But this needs **verification** before committing to WASM support.

**Fallback:** If CORS is not supported, the same relay/proxy used for NEXRAD data can proxy this endpoint.

## 4. NHC GIS MapServer — ⚠️ UNKNOWN

**URL:** `https://mapservices.weather.noaa.gov/tropical/rest/services/tropical/NHC_tropical_weather_summary/MapServer`

This is an ArcGIS MapServer endpoint. ArcGIS servers can be configured with CORS but it's not guaranteed. No evidence found either way.

**Fallback:** Same relay/proxy approach.

## 5. Natural Earth GeoJSON (borders) — ⚠️ UNKNOWN

**URL:** `https://raw.githubusercontent.com/nvkelso/natural-earth-vector/master/geojson/...`

GitHub's raw.githubusercontent.com serves files with CORS headers (`Access-Control-Allow-Origin: *`). This should work from WASM.

---

## Summary Table

| Data Source | CORS? | WASM-Ready? | Mitigation |
|---|---|---|---|
| NEXRAD S3 | ❌ No | No | Relay/proxy required |
| NWS Alerts API | ✅ Yes | Yes | Don't set custom User-Agent |
| NHC CurrentStorms.json | ⚠️ Unknown | Maybe | Verify; fallback to proxy |
| NHC GIS MapServer | ⚠️ Unknown | Maybe | Verify; fallback to proxy |
| Natural Earth GeoJSON | ✅ Yes | Yes | Works directly |

## Recommendation

**For Stage 8 (WASM):**
1. The WASM target can work for the UI, rendering, and overlays
2. Radar data will need a relay proxy — scope this into Stage 8 or descope live radar on WASM
3. NWS alerts should work directly from WASM
4. NHC data needs CORS verification — test early
5. Consider a Cloudflare Worker as a simple relay for all NOAA data sources

**The plan's "CORS permitting" caveat is accurate but understated** — it's not "permitting," it's "requires a proxy" for the primary data source (NEXRAD).
