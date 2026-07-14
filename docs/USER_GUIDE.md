# rustywx User Guide

rustywx is a desktop radarscope for the Macon, Georgia area. It downloads live
weather radar data from the National Weather Service's KJGX station (Robins
Air Force Base) and displays it as a classic circular radar screen — the kind
you'd see in a TV weather forecast, showing rain, storms, and wind motion in
real time.

This guide covers installing, running, and reading the app. If you want to
modify the code, see [`CONTRIBUTING.md`](../CONTRIBUTING.md) instead.

## What you need

- **A network connection.** rustywx downloads radar data from a public
  Amazon Web Services archive. No account, sign-up, or API key is required —
  the data is free and open to anyone.
- **Rust's `cargo` build tool**, if you're running from source (see
  [Installing](#installing)). rustywx isn't currently distributed as a
  pre-built app, so you'll build it once with a single command.

## Installing

From a checkout of the project:

```
cargo run --release
```

The first build compiles all dependencies and takes a few minutes. After
that, `cargo run --release` starts instantly. A window titled
"rustywx — KJGX radarscope (Macon, GA)" opens automatically.

> **Why `--release`?** The radar image is redrawn by rasterizing hundreds of
> thousands of data points into a texture. Without `--release`, this happens
> in unoptimized debug code and can make the app feel sluggish. Release mode
> is not optional for comfortable use.

## Your first launch

When rustywx starts, it immediately reaches out to the archive to find the
most recent volume scan for KJGX. You'll see:

1. The status bar (bottom of the window) reads *"Starting up — fetching
   latest KJGX volume…"*
2. After a few seconds, the radar screen fills in and the status bar shows
   the scan's timestamp, e.g. *"Scan 2026-07-13 20:14:12 UTC (16:14:12
   local)"*.

If nothing appears after 30–60 seconds, see [Troubleshooting](#troubleshooting).

## Reading the display

rustywx draws a **PPI scope** (Plan Position Indicator) — a top-down view
centered on the radar, with north pointing up, exactly like a map.

| Element | What it means |
|---|---|
| **White dot at center** | The KJGX radar antenna itself (Robins AFB). |
| **Range rings** | Concentric circles every 50 km, labeled along the north spoke, showing distance from the radar. |
| **N / E / S / W spokes** | Compass directions, for orientation. |
| **Small circles labeled "Macon" / "Warner Robins"** | City markers, positioned by real latitude/longitude. |
| **Colored regions** | The radar data itself — precipitation or wind, depending on the selected product (see below). |
| **Color legend, bottom-left** | The color scale for whichever product is currently displayed. |
| **Timestamp, top-left** | When the currently displayed scan was captured, in both UTC and your local time. |

Areas with no color (transparent/background) mean no significant echo was
detected there — clear air, or a value too weak to show.

## Products: Reflectivity vs. Velocity

rustywx displays two different measurements from the same radar sweep. Switch
between them using the buttons in the top control bar.

### Reflectivity

Reflectivity shows **where precipitation is** and roughly how intense it is,
measured in dBZ (decibels of radar reflectivity). The color scale runs from
cool colors at light intensity to hot colors at severe intensity:

- **Light blue/teal** — very light rain or drizzle
- **Green** — light-to-moderate rain
- **Yellow/orange** — heavy rain
- **Red** — very heavy rain or a thunderstorm core
- **Pink/purple/white** — extreme returns, often hail

### Velocity

Velocity shows **how fast precipitation or debris is moving toward or away
from the radar**, measured in meters per second. This is the product
meteorologists use to spot rotation in storms.

- **Green** — motion *toward* the radar (inbound)
- **Red** — motion *away from* the radar (outbound)
- **Muted gray-green tones near the middle of the scale** — little to no
  motion relative to the radar

Velocity is only meaningful where there's also precipitation to reflect the
radar beam — don't expect to see anything in areas that are blank on
Reflectivity.

## Controls

The control bar at the top of the window has three parts:

- **Product buttons** ("Reflectivity" / "Velocity") — click to switch which
  measurement is displayed. Your tilt selection is preserved across product
  switches where possible.
- **Tilt dropdown** — the radar doesn't scan flat; it sweeps at several
  elevation angles ("tilts"), from nearly flat (closer to the ground, better
  for distant weather) to steep (better for weather directly overhead).
  Select a tilt angle (e.g. `0.5°`, `1.5°`) to view that sweep. Velocity
  sometimes has fewer tilts available than Reflectivity for a given scan —
  rustywx automatically falls back to the nearest valid tilt if needed.
- **Status bar** (bottom of window) — shows what the background updater is
  currently doing (see next section).

## Auto-refresh and status messages

rustywx checks for a new volume scan every **2 minutes** in the background,
without interrupting what you're looking at. You'll see one of these in the
status bar:

| Message | Meaning |
|---|---|
| `Checking KJGX for new data…` | A background check is in progress. |
| `Up to date` | Checked, and the scan on screen is still the latest available — nothing to redraw. |
| `Scan <timestamp> UTC (<timestamp> local)` | A new scan was just downloaded and is now displayed. |
| `Error: … — retrying in Ns` | The last check failed (see below); it will automatically retry. |

A new volume scan typically becomes available roughly every 4–10 minutes,
depending on the radar's operating mode, so it's normal to see several
consecutive "Up to date" checks between updates.

## Troubleshooting

**The window stays on "Starting up — fetching latest KJGX volume…" for a long time.**
This usually means the network request is slow or blocked. Check your
internet connection. If you're on a restrictive network (corporate firewall,
some VPNs), outbound HTTPS access to Amazon S3 may be blocked.

**I see "Error: … — retrying in Ns" in the status bar.**
rustywx couldn't reach the archive or couldn't decode what it downloaded. It
retries automatically with increasing delays (30s, 60s, 120s, … up to a
10-minute cap) so it won't hammer the network if there's a real outage. No
action is needed — leave it running and it will recover once the network
issue clears.

**The screen is blank / mostly transparent.**
This is often correct, not a bug — it means little or no precipitation is
within range of KJGX right now. Try a different tilt angle, or check back
later. If you have reason to believe there should be weather showing (e.g.
you can see rain out the window), see the note below about coverage.

**Velocity shows nothing even though Reflectivity has data.**
Velocity data can be genuinely absent below a certain range near the radar,
or for tilts flagged in a way rustywx doesn't yet display. Try switching
tilts.

## Frequently asked questions

**Why only Macon, GA / KJGX?**
rustywx is currently built around a single radar site. Robins AFB (KJGX) was
chosen because its coverage area includes Macon and Warner Robins.

**Do I need an AWS account or API key?**
No. NEXRAD Level II archive data is published to a public, unauthenticated
S3 bucket by NOAA/Unidata. rustywx reads it anonymously.

**Why only Reflectivity and Velocity, not other products (like Spectrum Width)?**
These are the two most commonly used products for general storm-watching.
Support for additional products is a possible future enhancement — see
[`CONTRIBUTING.md`](../CONTRIBUTING.md) if you'd like to add one.

**Does rustywx store or cache old scans?**
No. It always shows the most recently downloaded scan; nothing is written to
disk.
