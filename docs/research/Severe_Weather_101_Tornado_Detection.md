# NSSL — NOAA National Severe Storms Laboratory

# SEVERE WEATHER 101

## Tornado Detection

*Source: Severe Weather 101 — Tornadoes, page "Detection". Retrieved 7/22/26, 2:46 PM (per page footer timestamps). Original URL: https://www.nssl.noaa.gov/education/svrwx101/tornadoes/detection/*

![Severe Weather 101 — Tornadoes hero banner: a photo of a tornado touching down over open farmland at dusk, silhouetted against a dramatic storm-lit sky, with a barbed-wire fence in the foreground.](media/Severe_Weather_101_Tornado_Detection/tornado-hero.png)

Forecasters and storm spotters have learned to recognize certain thunderstorm features and structure that make tornado formation more likely. Some of these are visual cues, like the rear-flank downdraft, and others are particular patterns in radar images, like the tornadic vortex signature (TVS).

**Storm spotters** have been trained to recognize tornado conditions and report what they see to the National Weather Service. Storm spotters can be emergency managers or even local people with a keen interest in severe weather who have taken formal storm spotter training in their community.

Computer programs, called algorithms, analyze **Doppler radar** data and display it in ways that make it easier for forecasters to identify dangerous weather. A storm with a tornado observed by radar has certain distinguishing features and forecasters are trained to recognize them.

When a Doppler radar detects a large rotating updraft that occurs inside a supercell, it is called a **mesocyclone**. The mesocyclone is usually 2-6 miles in diameter, and is much larger than the tornado that may develop within it.

![Mesocyclone Detection Algorithm Display: a paired radar display. Left panel, titled "Reflectivity (dBZ) Image," shows a color-coded precipitation intensity map (blues/greens/yellows/reds) of a supercell thunderstorm over a map with town labels (Belk, Kennedy, Lipscomb/Ilport, Millport, Tuscaloosa, Northport, Holt) and a circled hook-shaped core marked "703." Right panel, titled "Storm Relative Velocity Image," shows the same storm's Doppler velocity field in reds (outbound) and greens (inbound) with a small triangle marker indicating a detected mesocyclone/TVS near Reform/Gordo. Header row across the top of the display shows algorithm output values (e.g., 789, 254, 185, 76, CPLT, 5, 2545, 4, 3, 6, 13, 15, 4, 22, 13%, 28%). Below each panel are radar metadata fields: date/time 04/09/98 00:12:01 UTC, Vol: 195, CtrAz: 204.3°, CtrRn: 89.0km, Sw: 1, El: 0.5deg, VCP: 11, Mag: 8X, Nyquist: 23m/s, and for the velocity panel Direction 235.40°, Speed 21.40, SelAz 204.4°, SelRn 89.2km.](media/Severe_Weather_101_Tornado_Detection/mesocyclone-display.png)

*Caption: Mesocyclone Detection Algorithm Display [+]*

**What we do:** *NSSL developed the WSR-88D Mesoscale Detection Algorithm to analyze radar data and look for a rotation pattern meeting specific criteria for size, strength, vertical depth, and duration. A mesocyclone is usually 2-6 miles in diameter, and is much larger than the tornado that may develop within it.*

*NSSL researchers discovered the Tornado Vortex Signature (TVS), a Doppler radar velocity pattern that indicates a region of intense concentrated rotation. The TVS appears on radar several kilometers above the ground before a tornado touches ground. It has smaller, tighter rotation than a mesocyclone. While the existence of a TVS does not guarantee a tornado, it does strongly increase the probability of a tornado occurring.*

![Radar reflectivity image of two separate thunderstorm cells over a county-map background (magenta county boundary lines). The upper-left cell shows a broad multicolor (blue/green/yellow/orange/red) reflectivity core; the lower-center cell shows a compact intense high-reflectivity core (red/orange) with a small hook-like extension, surrounded by scattered cyan/blue ground-clutter-like returns along the radar's outer range.](media/Severe_Weather_101_Tornado_Detection/hook-echo-radar.png)

*Caption: Hook echo [+]*

A **"hook echo"** describes a pattern in radar reflectivity images that looks like a hook extending from the radar echo, usually in the right-rear part of the storm (relative to the motion of the storm). A hook is often associated with a mesocyclone and indicates favorable conditions for tornado formation. The hook is caused by the rear flank downdraft and is the result of precipitation wrapping around the back side of the updraft.

Dual-polarization radar technology, installed on NWS radars, can detect the presence of random shaped and sized targets like leaves, insulation or other **debris**. This gives meteorologists a high degree of confidence that a damaging tornado is on the ground, and is especially helpful at night when tornadoes are difficult to see with the human eye.

**What we do:** *NSSL engineers and scientists have adapted phased array technology, formerly used on Navy ships for surveillance, for use in weather forecasting. Phased array technology can scan an entire storm in less than one minute, allowing forecasters to see signs of developing tornadoes well ahead of current radar technology. NSSL uses a mobile Doppler radar to position close to tornadic storms to scan the entire lifecycle of a tornado. This helps us understand atmospheric processes to help improve forecasts of significant weather events.*

*Researchers at NSSL are developing the New Tornado Detection Algorithm, or NTDA, to help NWS forecasters better detect tornadoes and hail. The NTDA provides an operations update to the Tornado Detection Algorithm, also developed at NSSL, which is currently in use. The NTDA uses machine learning to evaluate storm criteria and calculates the probability of whether a tornado is present with each detection. The algorithm takes into account multiple storm aspects, including information available from dual-polarization radar, and reviews the statistics related to each evaluated element. All of these factors are then combined by the NTDA to yield a probability of a tornado presence. The NTDA is currently being tested in [NOAA's Hazardous Weather Testbed](https://hwt.nssl.noaa.gov/) on its performance and how NWS forecasters like the look and feel of the product.*

*NSSL's On-Demand web-based tool helped confirm when and where tornadoes occurred by mapping circulations on satellite images. NWS forecasters could quickly review warnings and check their accuracy with this system. Emergency responders and damage surveyors also used On-Demand to produce high-resolution street maps of potentially damaged areas so they can more effectively begin rescue and recovery efforts. Today, circulation*

> Page 3 of the source document ends mid-sentence ("Today, circulation …"); the PDF export provided is a 3-page excerpt (page footer shows "3 of 7") and does not include the remainder of the article (pages 4–7 of the live web page).

---

## Reference / Citation

NOAA National Severe Storms Laboratory (NSSL). "Severe Weather 101 — Tornadoes: Tornado Detection." *Severe Weather 101*.
URL: https://www.nssl.noaa.gov/education/svrwx101/tornadoes/detection/

## Notes on conversion

- Source PDF: `Severe_Weather_101_Tornado_Detection.pdf` (3 pages), unmodified.
- Images extracted with `pdfimages -png`; two duplicate raw-image extractions (byte-identical, from the same source bitmap rendered twice across the page-break in the print-to-PDF export) were removed, and one hero photo plus two figure images were kept and renamed for clarity:
  - `media/Severe_Weather_101_Tornado_Detection/tornado-hero.png` — page banner photo (1200×360)
  - `media/Severe_Weather_101_Tornado_Detection/mesocyclone-display.png` — "Mesocyclone Detection Algorithm Display" dual radar panel (800×589)
  - `media/Severe_Weather_101_Tornado_Detection/hook-echo-radar.png` — "Hook echo" radar reflectivity image (800×622)
- Six tiny icon/glyph images (social-media icons, 12×12–64×44 px slivers) extracted by `pdfimages` were discarded as UI chrome, not article content.
- No page-level `pdftoppm` screenshots were needed — the article's actual figures were already recovered cleanly via `pdfimages`.
