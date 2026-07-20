# CORS Quick-Reference — rustywx Data Sources

**Date:** 2025-07-19
**Purpose:** Quick lookup for which data sources work from WASM (browser) vs. require a relay proxy.

---

## CORS Status Table

| Data Source | URL | CORS? | WASM-Ready? | Notes |
|---|---|---|---|---|
| NEXRAD S3 Archive | `s3://unidata-nexrad-level2` | ❌ No | No | S3 buckets don't serve CORS for anonymous access |
| NEXRAD S3 Real-time | `s3://unidata-nexrad-level2-chunks` | ❌ No | No | Same S3 limitation |
| NWS Alerts API | `api.weather.gov/alerts/active` | ✅ Yes | Yes | `Access-Control-Allow-Origin: *`. Don't set custom `User-Agent`. |
| NHC CurrentStorms.json | `www.nhc.noaa.gov/CurrentStorms.json` | ❌ No | No | Served via CloudFront CDN. No CORS headers. |
| NHC GIS MapServer | `mapservices.weather.noaa.gov/.../MapServer` | ✅ Yes | Yes | Echoes origin. `Access-Control-Allow-Credentials: true`. |
| Natural Earth GeoJSON | `raw.githubusercontent.com/...` | ✅ Yes | Yes | GitHub serves `Access-Control-Allow-Origin: *` |

---

## WASM Relay Proxy Requirements

For WASM (browser) builds, a relay proxy (Cloudflare Worker) must handle:

| Route | Proxied URL | Reason |
|---|---|---|
| `/api/nexrad/*` | `s3://unidata-nexrad-level2/*` | S3 has no CORS |
| `/api/nhc/storms` | `www.nhc.noaa.gov/CurrentStorms.json` | CloudFront has no CORS |

**Not needed:**
- NWS Alerts — direct fetch from WASM
- NHC GIS overlays — direct fetch from WASM
- Natural Earth borders — direct fetch from WASM

---

## Desktop vs WASM Data Flow

```
Desktop (native):
  nexrad-data crate ──→ S3 (sigv4) ──→ decode ──→ render
  Ply net ──→ NWS / NHC / GeoJSON ──→ render

WASM (browser):
  Ply net ──→ Cloudflare Worker ──→ S3 ──→ decode ──→ render
  Ply net ──→ Cloudflare Worker ──→ NHC CurrentStorms.json ──→ render
  Ply net ──→ NWS / NHC GIS / GeoJSON ──→ render  (direct)
```

---

## Test Commands (reproducible)

```bash
# NHC CurrentStorms.json — NO CORS
curl -sI -H "Origin: http://localhost:8080" \
  "https://www.nhc.noaa.gov/CurrentStorms.json" \
  | grep -i access-control
# → (no output)

# NHC GIS MapServer — CORS OK
curl -sI -H "Origin: http://localhost:8080" \
  "https://mapservices.weather.noaa.gov/tropical/rest/services/tropical/NHC_tropical_weather_summary/MapServer?f=json" \
  | grep -i access-control
# → access-control-allow-origin: http://localhost:8080
# → access-control-allow-credentials: true

# NWS Alerts — CORS OK
curl -sI -H "Origin: http://localhost:8080" \
  "https://api.weather.gov/alerts/active" \
  | grep -i access-control
# → access-control-allow-origin: *
```
