# Hands-on Training on Weather Radar QC

**Author:** Junji Hotta
**Organization:** Japan Meteorological Agency (JMA), Office of Observation Systems Operation, Observation Department
**Workshop:** WMO/ASEAN Training Workshop on Weather Radar Data Quality and Standardization
**Date:** 7–8 February 2018
**Location:** Bangkok, Thailand

**Source / Citation:**
- JMA Workshop Materials: https://www.jma.go.jp/jma/en/photogallery/WMO-ASEAN_Radar_Workshop_Feb2018/2-5_Hands-on%20training%20on%20weather%20radar%20QC.pdf
- WMO Final Report: https://www.jma.go.jp/jma/en/photogallery/WMO-ASEAN_Radar_Workshop_Feb2018/Final%20report%20-%20WMO_ASEAN_Training_Workshop_on_Weather_Radar.pdf

---

## Workshop Overview

This training material was presented at the WMO/ASEAN Training Workshop on Weather Radar Data Quality and Standardization, held 5–13 February 2018 in Bangkok, Thailand. The workshop covered a broad range of topics across multiple days:

**Workshop Guide Map:**
- Day 1: Weather radar basics and operation (hardware, observation, installation, observation scheduling)
- Day 1–2: Quality control, calibration, maintenance
- Day 2–4: QPE & QPF, sites composites, advanced techniques
- Day 4: Advanced techniques (solid-state transmitter, Doppler velocity, detecting mesocyclone, products from dual-pol observations, accurate observations using dual-pol, weather summary)
- Day 5: Regional radar network, capacity development, data exchange, regional cooperation
- Day 9: Japan Meteorological Agency

**Training Session Topics (90 min total):**
- Introduction of JMA Operational system (15 min)
- Quality control algorithms:
  - Characteristics of non-precipitation echo (10 min)
  - JMA methods of Pseudo CAPPI process (15 min)
  - Statistical approach for QC (10 min)
- Hands-on training (90 min):
  - Adjustment of elevation angle composite table
  - Making PCAPPI and Statistical data
  - Verification of the results

---

## JMA Weather Radar Network

### Network Overview

- **20 C-band Doppler radars** operated by JMA across Japan (klystron transmitter, 250 kW peak power)
- **26 MLIT C-band radars** (Ministry of Land, Infrastructure, Transport and Tourism)
- C-band radar echo data (JMA and MLIT) are collected to the center system and integrated into a nationwide echo intensity composite map every 5 minutes
- Used for: Monitoring/Nowcasting, Quantitative Precipitation Estimation/Forecast (QPE/QPF)

### JMA Radar Specifications

| Parameter | Value |
|---|---|
| Frequency | 5300–5370 MHz (C-band) |
| TX type | Klystron |
| Peak Power | 250 kW |
| Pulse Width | 2.5 μs |
| Pulse Repetition Frequency (PRF) | 330 Hz (low), 940/752 Hz, 600/480 Hz (dual high) |
| Antenna Diameter | 4 m (Beam Width < 1.2°) |
| Maximum Range | Rainfall intensity: 400 km; Doppler velocity: 250 km |

### Radar Network System Architecture

The radar network consists of:
- **Radar Observatory:** Antenna unit with radome, transmitter, receiver, signal processor, data processor, antenna controller, communication line
- **Center System (Tokyo):** Control and monitor unit, central operation center at Headquarters
- **Backup Center (Osaka)**
- **Remote Control and Monitor Units** at local meteorological offices connected via JMA network

### Automated QC on Radar Systems — Data Flow

**At Radar Sites:**
1. Raw data collection (Reflectivity, Velocity)
2. QC methods applied: Side lobe removal, Selective MTI, MTI map processing, Interference removal, Sweep correlation, 2nd-trip echo removal

**At Center System:**
1. Raw data → QC processing → Making products
2. QC on the PCAPPI process
3. Forecasting product systems (QPE, QPF)
4. QC at every product stage
5. Quality control for radar composite

---

## Scan Strategy

### Basic Concept of Scanning Schedule

- **Precipitation mode:** Low PRF (330 Hz), long pulse (2.6 μs) — long distance, mainly precipitation
- **Velocity mode:** Dual high PRF (940/752 Hz, 600/480 Hz), short pulse (1.1 μs) — short distance, large velocity range
- Range unfolding used for extended velocity coverage

### Observation Scan Sequence (Tokyo Radar Example)

10-minute volume scanning with the following elevation angles and PRF modes:

| Elevation (°) | Range (km) | PRF Mode |
|---|---|---|
| 0.0 | 400 | Low-PRF (330Hz, 2.6μs) |
| 0.3 | 400 | Low-PRF (330Hz, 2.6μs) |
| 0.7 | 150 | High-PRF (940/752Hz, 1.1μs) |
| 1.0 | 250 | Middle-PRF (600/480Hz, 1.1μs) |
| 1.1 | 250 | High-PRF (940/752Hz, 1.1μs) |
| 1.7 | 250 | Middle-PRF (600/480Hz, 1.1μs) |
| 2.0 | 150 | High-PRF (940/752Hz, 1.1μs) |
| 2.5 | 250 | Middle-PRF (600/480Hz, 1.1μs) |
| 3.5 | 150 | High-PRF (940/752Hz, 1.1μs) |
| 3.8 | 250 | High-PRF (940/752Hz, 1.1μs) |
| 4.8 | — | — |
| 6.7 | — | — |
| 9.3 | — | — |
| 12.9 | — | — |
| 17.9 | — | — |
| 25.0 | — | — |

*Range unfolding is applied for the 250km-range High-PRF observations.*

### Primary Data Specifications

| Data Type | Unit | Coordinates | Mesh Size | Area | Number of Mesh | Data Size | Format | Period |
|---|---|---|---|---|---|---|---|---|
| Echo Intensity | dBZ | Polar | 250m × 0.7° | 400 km radius | 8,192,000 | 1 byte | GRIB2 | 10 min |
| Doppler Velocity | m/s | Polar | 250m × 0.7° | 250 or 150 km radius | 512,000 or 307,200 | 1 byte | GRIB2 | 10 min |

---

## CAPPI and Pseudo CAPPI (PCAPPI)

### CAPPI (Constant Altitude Plan Position Indicator)

A horizontal cross-section display of a variable at a specified altitude. Different elevation scans are used to construct the cross-section at the target altitude.

### Pseudo CAPPI (PCAPPI)

The "no data" regions in CAPPI (close to and away from the radar relative to the selected altitude) are filled with data from the highest and lowest elevations, respectively. This produces a more complete composite display.

**JMA's PCAPPI method:** Uses several PPIs at low elevation angles to create data at approximately 2 km altitude. This approach can remove both sea clutter and ground clutter.

### Elevation Angle Composite Table

The elevation angle composite table is the key parameter for making quality-controlled CAPPI data:
- **Purpose:** Selecting an optimal elevation angle located near 2 km altitude in each direction from the radar
- **Goal:** Avoiding the effect of ground clutter
- **Configuration:** Per-radar, per-direction (azimuth) parameter selecting which elevation angle PPI to use at which range
- **Available elevation angles:** (1) 0.0°, (2) 0.3°, (3) 0.7°, (4) 1.1°, (5) 1.7°, (6) 2.5°

### PCAPPI Data Specifications

| Data Type | Unit | Coordinates | Mesh Size | Area | Number of Mesh | Data Size | Format | Period |
|---|---|---|---|---|---|---|---|---|
| Pseudo CAPPI | mm/hr | xy | 1 × 1 km | 500 × 500 km | 250,000 | 1 byte | Radar IO | 5 min |
| CAPPI | dBZ | xy | 1 × 1 km | 500 × 500 km | 250,000 | 1 byte | GRIB2 | 10 min |

### Nationwide Radar Composite Processing

1. Low-altitude reflectivity of each radar → Z-R Conversion (Z = 200R^1.6) → Quality control
2. Radar-raingauge processing → Calibration factor of each radar
3. Calibration → Combining (maximum method) → Intercompare before calibration
4. All radar echo data collected to center system → Nationwide radar echo composite data

---

## Non-Precipitation Echo Types (Clutter)

### 1. Ground Clutter

Echoes due to non-precipitation targets including buildings, hills, mountains, aircraft, and chaff. Suppressed by MTI (Moving Target Indicator). However, with the passage of a low pressure system, storms may cause ground clutters that are not completely removed. Careful monitoring is required because they do not indicate actual precipitation.

### 2. Sea Clutter

Caused by sea wave or sea spray. Because of sea wave motion, suppression by MTI does not work well. For elimination of usual sea clutters, radar scans with high elevation angles are employed to produce the PCAPPI. In windy situations (e.g., typhoons), sea spray may be observed at low elevation angles.

### 3. Anomalous Propagation (AP)

The variation of refractive index in the air refracts the radar beam downward. This kind of refraction is known as anomalous propagation. AP produces false echoes, in most cases appearing as sea clutter. AP can be caused by inversion layers.

### 4. Upper-Air Echoes

Radar echoes sometimes observed only at upper altitudes. In this case, the liquid drop is completely vaporized during its fall, so precipitation doesn't reach the ground.

### 5. Electromagnetic Interference

- **Sun noise:** The sun generates noise that can be detected by the radar
- **Interference from other radars:** Artifactual electromagnetic sources
- **Interference from moving bodies:** Aircraft, ships, etc.

### Importance of Case Accumulation

It is important to accumulate cases of anomalous echoes. It will be very useful for QC. The materials should include meteorological information such as weather charts, various observations (AWS, sonde, satellite, radar).

---

## QC Processing Flow for PCAPPI (EIL Process)

### Data Processing Flow

1. **Radar data in GRIB2** (polar coordinates r, θ) by elevation angle
2. **Coordinate transformation:** Polar → X-Y coordinates (1 km resolution, 500 × 500 km area)
3. **Elevation angle composite table:** Selecting optimal elevation angles
4. **Compositing PPIs** at several elevation angles
5. **Isolated echo removal:** Removing isolated echoes from ground clutter, ships, aircraft
6. **Noise cut:** Setting echo intensities below threshold to zero
7. **Clutter map subtraction:** Removing residual clutter
8. **Minimum rain rate filter:** Cutting low-level values after clutter map processing
9. **Smoothing** around elevation angle borders
10. Output: JMA PCAPPI in X-Y coordinates (Echo Intensity at the Lowest level — EIL)
11. **Compositing XY data** of all sites → Nationwide echo intensity composite map

### Key Parameters (sitelowmake.ini)

| Parameter | Description | Example Value |
|---|---|---|
| `elangles` | Elevation angles list | 3.8,2,1.0,0.3,1.7,1.1,0.7,0.3,0.0,... |
| `use_angle_10a` | Angle usage flags | 0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,1,1,1,1 |
| `code` | Station code | A5 |
| `ename` | Station name | tokyo |
| `offx` / `offy` | Offset coordinates | 20, 20 |
| `n0` | Noise level | 51.1 |
| `noise_cut` | Threshold (dBZ × 100) | 704 |
| `rain_cut` | Minimum rain rate (mm/h × 100) | 33 |
| `iso_window` | Isolated echo removal window size | 5 |
| `iso_count` | Isolated echo count threshold | 5 |
| `smooth_r` | Smoothing range (km × 10) | 100 |
| `smooth_t` | Smoothing azimuth (deg × 10) | 10 |
| `clut1_file` | Clutter map file | CLUT¥"aa"¥_00_1 |
| `clut1_type` | Clutter map type | 3 |
| `clut1_wx` / `clut1_wy` | Clutter map window x/y | 3, 3 |
| `clut1_count` | Clutter map count | 0 |
| `B` / `beta` | Z-R relationship parameters (Z = B·R^beta) | 200, 1.6 |

### Smoothing Around Elevation Angle Borders

Smoothing is applied around borders of the elevation angle composite table to avoid discontinuity at connection points. The smoothing area is 10 km around the border. The number of meshes at each angle is counted and used as weight for averaging intensities.

**Example calculation:**
- Elevation 1: 77 meshes at 30 dBZ
- Elevation 2: 154 meshes at 50 dBZ
- Elevation 3: 210 meshes at 35 dBZ
- Smoothed intensity = (77×10^(30/10) + 154×10^(50/10) + 210×10^(35/10)) / 441 = 36601.08
- Logarithmic value: 10 × log(36601.08) = 45.63 dBZ

### Isolated Echo Removal

In a 5×5 mesh window, the number of meshes with intensity > 0 is counted (excluding the center mesh). If the count is below the threshold, the center mesh intensity is set to zero.

### Noise Cut

Echo intensities below a threshold are set to zero (No Echo). The threshold depends on each radar. Example: `noise_cut=704` means threshold of 7.04 dBZ (dBZ × 100).

### Clutter Map

Used when MTI processing cannot eliminate clutter. A clutter map has thresholds to delete echoes or subtract values from observed echo intensities.

**Formula:** Nr = Ns + 10 × log(1 − 10^((Ng − Ns)/10))

Where:
- Ng = clutter map value (dBZ)
- Ns = reflectivity before subtraction (dBZ)
- Nr = reflectivity after subtraction (dBZ)

**Cases:**
- If Ns >> Ng: Nr ≈ Ns (no effect, precipitation passes through)
- If Ns ≈ Ng: Nr is significantly reduced
- If Ns < Ng: Nr is set to zero (cut)

### Minimum Rain Rate

If the reflectivity after clutter subtraction (Nr) is below a minimum rain rate threshold, Nr is set to zero (No Echo).

### Summary of QC Algorithm Features

| Algorithm | Type | Advantage | Disadvantage |
|---|---|---|---|
| Elevation angle composite table | Selected angle, Area: (r,θ) | Removes sea clutter, ground clutter | — |
| Noise cut | Low level cut, Area: all | Removes low level noise | Removes low level echo |
| Minimum rain-rate | Low level cut, Area: all | Removes low level echo | — |
| Clutter map | Level cut (set), Area: mesh | Removes ground clutter (enable to remove by MTI) | Removes precipitation echo (labor for setting) |

### EIL Process Summary

The EIL (Echo Intensity at Lowest level) process contains many quality control methods. To create Cartesian data with good quality (less clutter and less noise), various parameters need to be set adequately. Removing non-precipitation echo also has the possibility of removing precipitation echo.

---

## Statistical Approach for QC

Statistical methods are effective for understanding the quality of radar data. JMA uses statistics for quality control.

### Statistical Methods

1. **Appearance Count:** Count over 1mm/h precipitation intensity calculated from dBZ, B, and beta. Clarifies continuous weak echoes.
2. **Summation (Average):** Sum up precipitation from radar data. While appearance count can't detect clutter or high intensity echo, summation can. The summation of precipitation from clutter affects QPE.

*Note: Average is used instead of summation (equivalent with using data count).*

### Statistical Targets

- **Every elevation's observation:** To understand characteristics of observation at each elevation
- **EIL (Echo Intensity at the Lowest Level):** To understand characteristics of products — shadowed area, observable area, low quality area

### How to Detect Clutter

Two types of suspicious patterns:
1. **Merely but strong:** High average but low appearance count — suspicious (clutter caught merely but strong)
2. **Weak but continuous:** Low average but high appearance count — suspicious (clutter caught weak but continuous)

Precipitation cases are not suspicious (moderate appearance count and moderate average).

### Summation Test Example (Murotomisaki Radar, May 2014)

- 0.2° is the operational angle (observational product)
- 0.6° test decreases the clutter
- 1.2° test also decreases the clutter, but decreases real precipitation echo

---

## Hands-on Training Exercises

### Tools Used

- **radar-library.jar:** Executable binary written in Java, compressed in JAR format. Contains decoding, encoding, data processing, coordinate transforming, and data viewing programs. Runnable via command line for every purpose.
- **Windows batch files** used for simplicity.

### Training Data

- **Butterworth site** RAW data (IRIS format)
- **Data period:** 1 day (17 Dec 2014)
- **Elevation angles for PPI:** 0.0, 0.7, 1.5, 2.5

### Composite Table (CSV file)

The composite table shows angles used in each area. Format: `Azimuth(deg), Distance(km), Angle(deg)`

**Example (simple CAPPI):**
```
360, 0, 2.5, 60, 1.5, 85, 0.7, 150, 0, 300
```
This means: for 0–360 azimuth (all around), use 2.5° PPI for 0–60 km, 1.5° PPI for 60–85 km, 0.7° PPI for 85–150 km, 0.0° PPI for 150–300 km.

### Training Workflow

1. **Operation check** of batch files
2. **Adjust composite table** using Radar Beam Visibility map and Cross Section Chart
3. **Make PCAPPI** using MakePCAPPI.bat
4. **Make statistics** using Statistics_pcappi.bat
5. **Verification** of results

### Exercise 1: Avoiding Beam Blockage

Using the Radar Beam Visibility Map and Cross Section Chart, identify azimuths where the lowest elevation beams are blocked by terrain. Edit the composite table to use higher elevation angles in those directions.

**Example:** For azimuth 100–120°, the 0.0° beam is blocked. Modified composite table:
```
100, 0, 2.5, 60, 1.5, 300
120, 0, 2.5, 60, 1.5, 300
360, 0, 2.5, 60, 1.5, 85, 0.7, 150, 0, 300
```
This stops using 0.7° and 0.0° angle data for 100–120° azimuth.

### Exercise 2: Southwestern Beam Blockage

Similar exercise for azimuth 230–260° where 0.0° beams are blocked.

### Exercise 3: Avoiding Mirror Image

Mirror images occur when radar beams reflect off buildings, creating false echoes at a symmetric angle. Statistical data reveals PCAPPI contamination by mirror images. Adjust the composite table to use higher elevation angles in affected directions to avoid the mirror image.

### Verification Process

Verify the results of statistics from remade PCAPPI. It is important to verify the adjusted data because there might be cases where the adjustment affects data quality negatively.

### Batch File Details

**1. MakeTablepng.bat:** Converts composite table CSV to PNG visualization
```
java -cp radar-library.jar PCAPPITableToPNG ./tbl/agButterworth.csv
```

**2. MakePCAPPI.bat:** Generates PCAPPI data from raw radar data
```
java -cp radar-library.jar JMAPCappiMain -envfile=./sitelowmake.ini -tbldir=./tbl -destdir=./pcappi/. -clutdir=./clut ./RawData/%%i -gz
java -cp radar-library.jar PCAPPIView ./pcappi/%%i -dest=./pcappiView/. -colorPallet=./cp/dbz_color.txt
```

**3. Statistics_pcappi.bat:** Generates statistical data from PCAPPI
```
java -cp radar-library.jar StatisticsMain -ini=statistics.json -name=pcappi -start=201412170000 -end=201412180000
java -cp radar-library.jar StatisticsView ./statistics/pcappi
```

Statistics parameter file (statistics.json):
```json
{
  "filename": "./pcappi/RCAP.*'yyyyMMddHHmmss'N302N.gz",
  "sek": "10",
  "calculation": "DBZtoRAIN",
  "outdir": "./statistics/pcappi/."
}
```

---

## Composite Maps

### National Composite Map

Quality of the composite map depends on quality of each radar's EIL. The national composite integrates all radar sites' PCAPPI data.

### International Radar Composite Imagery

JMA also participates in international radar composite efforts, showing composite imagery with satellite image overlays.

---

## Summary

- In order to create Cartesian data with good quality (less clutter and less noise), various processes are needed
- It is impossible to completely eliminate anomalous echoes by automatic processing
- Radar data quality control should be done through whole radar systems
- Accumulation and careful investigation of radar data will improve QC results
- Anomalous echoes include: ground clutter, sea clutter, anomalous propagation, upper-air echoes, electromagnetic interference
- The elevation angle composite table is the key parameter for PCAPPI quality
- Statistical methods (appearance count, summation/average) are effective for detecting clutter
- Case accumulation contributes to QC improvement

---

## Relevance to rustywx

Provides practical QC techniques for radar data: ground clutter identification, AP detection, isolated echo removal, and the PCAPPI compositing approach. The elevation angle composite table concept is relevant to rustywx's tilt selection and future CAPPI display modes. The clutter map and noise cut techniques could inform rustywx's own clutter filtering in `scope.rs`. Key takeaways:
- The Z-R relationship Z = 200R^1.6 used by JMA
- Multi-elevation compositing at ~2 km altitude to avoid ground clutter
- Isolated echo removal using 5×5 mesh window with count threshold
- Statistical QC via appearance count and summation/average patterns
- The importance of beam visibility analysis for tilt selection