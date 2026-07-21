# rustywx User Guide

## Starting the App

Run the app using the Justfile:

```bash
just run
```

Or manually:

```bash
cd ply-spike && cargo run --release
```

The app will start on the default site (KJGX by default, or your last selected site if you've saved settings).

## Main Display

The main radar scope shows a Plan Position Indicator (PPI) view of the current weather radar volume.

### Status Bar

The bottom status bar displays:
- **Current Site:** The NEXRAD radar site (e.g., KJGX)
- **VCP / Scan Mode:** The active Volume Coverage Pattern (e.g., "VCP 12 — Precipitation") and operational mode, giving context about data quality and scan strategy
- **Elevation / Tilt:** The currently selected elevation angle
- **Nyquist Velocity:** The unambiguous velocity range for the current tilt. Values near ±Nyquist are at the edge of the unambiguous range and may be aliased.

### Radar Rings and Markers

The scope displays:
- **Range Rings:** Concentric circles at 10 nmi intervals to measure distance from the radar
- **Cardinal Spokes:** North, South, East, West reference lines
- **City Markers:** Reference points for geographic orientation
- **State and County Borders:** Political boundaries overlay

## Products

Select between three radar products by clicking the product selector or using keyboard shortcuts:

### Reflectivity

Shows the intensity of precipitation. Higher values (red/magenta) indicate heavier rain or hail. Lower values (green/blue) indicate light precipitation or cloud tops.

### Velocity

Shows the radial velocity (movement toward or away from the radar). Red indicates motion away (outbound), green indicates motion toward (inbound). This helps identify rotation, wind shear, and storm motion.

### Spectrum Width

Shows the velocity dispersion in the radar sample volume (standard deviation of the velocity spectrum). High values indicate turbulence, wind shear, or mixed target types.

## Elevation Tilt Selection

Use the elevation selector to view different tilt angles through the storm. Each tilt provides a different vertical "slice" of the weather system. The status bar updates to show the Nyquist velocity for the selected tilt.

## Overlays

Toggle overlays on or off using the controls or settings:

- **Borders:** State, county, and international borders for geographic reference
- **Alerts:** NWS Warning (tornado warnings, severe thunderstorm warnings) and Watch (areas of potential severe weather) polygons
- **Tropical (NHC):** National Hurricane Center storm track and forecast cone overlays for tropical systems

## Settings Panel

Click the gear icon to open the Settings panel. Configure:

### Site Selection
- **Default Site:** Choose the radar site the app opens on. Select from available NEXRAD stations. Your choice is saved.

### Poll Interval
- **Auto-Refresh Frequency:** How often the app checks for new volume scans (default 2 minutes). Lower values get fresher data but use more bandwidth.

### NHC Refresh
- **Tropical Data Update:** How often the National Hurricane Center overlay is refreshed.

### Overlay Defaults
- **Borders:** Toggle borders on/off at startup
- **Alerts:** Toggle NWS alerts on/off at startup
- **Tropical (NHC):** Toggle tropical system overlay on/off at startup

### Animation Level
- **Full:** Animate between stored radar volumes smoothly
- **Subtle:** Light animation between frames
- **None:** No animation; display only the latest volume

### TDBZ Clutter Filter Sensitivity
The TDBZ (Time Difference Between Tilts) clutter filter removes wind turbine and other static/ground clutter from the display.

- **Sensitive (5×5):** Removes subtle clutter but may suppress weak precipitation at storm edges
- **Default (9×9):** Balanced filtering; recommended for most situations
- **Aggressive (13×13):** Removes strong clutter but may over-filter weak signals

Settings are automatically saved and restored on app startup.

## Keyboard Shortcuts

Press **?** to display the keyboard shortcuts overlay. Available shortcuts include:

- **Product Selection:** Switch between Reflectivity, Velocity, and Spectrum Width
- **Elevation Control:** Increase/decrease tilt angle
- **Overlay Toggles:** Show/hide Borders, Alerts, and Tropical overlays
- **Settings:** Open the Settings panel
- **Quit:** Exit the application

## Tips

- **Nyquist Aliasing:** Velocity values that appear purple or wrapped are near the Nyquist velocity and may be aliased (folded). This is especially common in strong mesocyclones.
- **Reflectivity Interpretation:** The NWS color table uses specific thresholds: >60 dBZ typically indicates hail or very heavy rain.
- **Spectrum Width:** Rapidly increasing spectrum width often precedes rotation and is a useful shear indicator.
- **Data Freshness:** Check the poll interval setting if you want near-real-time data. The public AWS archive updates every few minutes.
- **Network Errors:** The app handles network failures gracefully. If a data fetch fails, it will retry on the next scheduled poll.
