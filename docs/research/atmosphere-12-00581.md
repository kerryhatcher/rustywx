# Polarimetric Radar Characteristics of Tornadogenesis Failure in Supercell Thunderstorms

*Article*

**Matthew Van Den Broeke** (ORCID)

Department of Earth and Atmospheric Sciences, University of Nebraska-Lincoln, Lincoln, NE 68588, USA; mvandenbroeke2@unl.edu

**Citation:** Van Den Broeke, M. Polarimetric Radar Characteristics of Tornadogenesis Failure in Supercell Thunderstorms. *Atmosphere* **2021**, *12*, 581. https://doi.org/10.3390/atmos12050581

**Academic Editor:** Merhala Thurai

**Received:** 20 March 2021 **Accepted:** 27 April 2021 **Published:** 30 April 2021

**Publisher's Note:** MDPI stays neutral with regard to jurisdictional claims in published maps and institutional affiliations.

**Copyright:** © 2021 by the author. Licensee MDPI, Basel, Switzerland. This article is an open access article distributed under the terms and conditions of the Creative Commons Attribution (CC BY) license (https://creativecommons.org/licenses/by/4.0/).

*(Note: the small CC-BY license icon and MDPI/journal logo graphics embedded on page 1 are decorative marks, not data figures, and were not retained in `media/`.)*

**Abstract:** Many nontornadic supercell storms have times when they appear to be moving toward tornadogenesis, including the development of a strong low-level vortex, but never end up producing a tornado. These tornadogenesis failure (TGF) episodes can be a substantial challenge to operational meteorologists. In this study, a sample of 32 pre-tornadic and 36 pre-TGF supercells is examined in the 30 min pre-tornadogenesis or pre-TGF period to explore the feasibility of using polarimetric radar metrics to highlight storms with larger tornadogenesis potential in the near-term. Overall the results indicate few strong distinguishers of pre-tornadic storms. Differential reflectivity (Z<sub>DR</sub>) arc size and intensity were the most promising metrics examined, with Z<sub>DR</sub> arc size potentially exhibiting large enough differences between the two storm subsets to be operationally useful. Change in the radar metrics leading up to tornadogenesis or TGF did not exhibit large differences, though most findings were consistent with hypotheses based on prior findings in the literature.

**Keywords:** supercell; nowcasting; tornadogenesis failure; polarimetric radar

---

## 1. Introduction

Supercell thunderstorms produce most strong tornadoes in North America, motivating study of their radar signatures for the benefit of the operational and research communities. Since the polarimetric upgrade to the national radar network of the United States (2011–2013), polarimetric radar signatures of these storms have become well-known, e.g., [1–5], and many others. Over time, characteristic radar signatures in supercell storms have been quantified, e.g., [5–7]. Quantification facilitates comparison between subsets of storms and allows inferences about storm structural and microphysical changes through the supercell life cycle, including around the time of tornadogenesis and tornado demise, e.g., [8–11]. This study extends prior work by focusing on polarimetric radar signatures during pre-tornadogenesis failure (TGF) analysis periods in nontornadic supercell storms.

TGF is defined by [12] as "the lack of tornado formation within a strong (vertical vorticity ≥ 0.01 s<sup>−1</sup>) low-level mesocyclone whose life cycle is ≥15 min". This mesocyclone must also be produced by a storm that otherwise looked as if it was transitioning into a tornadic phase. The time of maximized low-level vertical vorticity was defined as the point of TGF [12]. The author in [12] notes that many nontornadic supercells may not have a TGF event since low-level rotation may not be present.

Potential reasons for TGF have been documented in the literature. The authors in [13] discuss undercutting of the mesocyclone by cold outflow, which can disrupt mesocyclone maintenance and if occurring at the right time in the storm life cycle may result in TGF. The presence of low-level cold air is not, however, universally detrimental to tornadogenesis, e.g., [14]. The author in [12] speculates that, based on tornadogenesis modes described by [15], TGF may result from large stability in the boundary layer or from a low-level updraft circulation too weak to contract environmental vorticity into a tornado-strength vortex within a reasonable amount of time. The importance of boundary layer stability was further indicated by, e.g., [16], who found that tornadic supercells are more likely to have large low-level instability and smaller convective inhibition in the mesocyclone vicinity.

Numerical supercell simulations have largely agreed with observational studies. The authors in [17] simulated supercells using Rapid Update Cycle (RUC) soundings and completed analyses of vorticity along backward trajectories leading to tornadogenesis and TGF. Trajectories in tornadic simulations experienced larger increases in vertical vorticity due to tilting and especially due to stretching, indicating a rapid increase in upward motion and supporting larger instability and smaller convective inhibition near the mesocyclone in tornadic storms [17]. Using an ensemble of modified soundings observed in tornadic and nontornadic supercell environments, [18] also showed via numerical experiments that tornadic supercells are characterized by a steady, organized updraft at low levels, while nontornadic storms had a less steady low-level updraft. This study also indicated the potential importance of small within-storm details that are not yet well understood [18]. Further numerical experiments indicate that vertical collocation of a strong low-level updraft and intensifying near-surface vorticity maximum is characteristic of tornadic supercells [19], a condition supported by streamwise orientation of horizontal vorticity at low levels.

The work reported here is an extension of that presented by [11], who compared long temporal periods in nontornadic storms to shorter (30 min) pre-tornadic periods in tornadic storms. In that work, the primary findings most important for future work included (1) smaller extent of radar-inferred hailfall at low levels in pre-tornadic supercells. It was hypothesized that storm outflow thermodynamic characteristics may be influenced by hailfall. (2) Lower radar reflectivity (Z<sub>HH</sub>) values in the cores of tornadic supercells, possibly indicating a difference in precipitation production (primarily hail) and/or the vertical temperature profile (e.g., melting hail often has larger Z<sub>HH</sub> than dry hail). (3) Larger and steadier differential reflectivity (Z<sub>DR</sub>) columns in pre-tornadic storms, indicating larger and steadier midlevel updrafts. These findings form a basis for our hypotheses regarding how pre-tornadic and pre-TGF periods should differ.

Polarimetric radar observations may provide additional support for the mechanisms described in prior studies and shed additional light on kinematic and microphysical processes in the time leading up to TGF. When compared with tornadic storms, such observations may indicate operationally-useful differences in radar characteristics between those storms which produce a tornado and those in which tornadogenesis fails. In this work, radar signatures are examined for the first time within equal-length periods prior to tornadogenesis and TGF in a sample of supercell storms. This builds on the work of [11] by only considering nontornadic storms that experienced TGF and by comparing equal-length analysis periods before tornadogenesis and TGF, allowing direct comparison between these two storm subsets. Given the findings of prior studies we here hypothesize that pre-TGF periods will contain weaker and less-steady midlevel updrafts than pre-tornadic periods, will contain larger areas dominated by hail, and will be characterized by larger storm-core Z<sub>HH</sub> values on average. Additional hypotheses are described in the Discussion where findings are related to prior work. Storm sample sizes are sufficient to determine whether any of the radar metrics examined are likely to be useful for this purpose, and to determine which radar metrics may warrant additional work.

## 2. Data and Methods

Sets of tornadic and nontornadic supercells were required. Tornadic storms were taken from [11]; see their Table 1) and included in this study if the algorithm used for radar analysis (described below) identified and tracked the storm well. Table 1 of the present study includes more information about these storms. Tornadogenesis times were those reported in the Storm Events Database [20] which, despite its limitations, e.g., [21], represents the best information available about when a storm is tornadic. All radar scans in the 30 min prior to reported tornadogenesis were retained for the pre-tornadogenesis analysis (Table 1).

**Table 1.** The date, radar, analysis period (UTC), forward flank angle (FFA; degrees), freezing level (FL; meters), and Z<sub>DR</sub> calibration factor (dB) for each of the tornadic cases included in the pre-tornadogenesis analysis. The end of the analysis period is the tornadogenesis time.

| Date | Radar | Analysis Period | FFA | FL | Z<sub>DR</sub> Calibration Factor |
|---|---|---|---|---|---|
| 26–27 April 2012 | KOHX | 2330–0000 | 190 | 3852 | 0.203 |
| 30 April 2012 | KDDC | 2222–2252 | 160 | 3907 | 0.575 |
| 10 May 2012 | KEWX | 1816–1846 | 140 | 4020 | −0.518 |
| 8–9 June 2012 | KMQT | 2343–0013 | 190 | 3999 | −0.012 |
| 18 March 2013 | KFFC | 2140–2210 | 180 | 3278 | −0.100 |
| 31 March 2013 | KINX | 0205–0235 | 190 | 2885 | 0.026 |
| 31 March 2013 | KSRX | 0350–0420 | 200 | 3000 | −0.361 |
| 17 April 2013 | KFDR | 2320–2350 | 180 | 4639 | −0.523 |
| 15 May 2013 | KFWS | 2311–2341 | 190 | 4614 | −0.128 |
| 18–19 May 2013 | KDDC | 2348–0018 | 180 | 3635 | 0.007 |
| 19 May 2013 | KTLX | 2052–2122 | 160 | 4369 | −0.526 |
| 19 May 2013 | KTLX | 2230–2300 | 160 | 4369 | −0.526 |
| 20 May 2013 | KINX | 2021–2051 | 170 | 4167 | 0.353 |
| 20 May 2013 | KEAX | 2127–2157 | 140 | 3563 | −0.261 |
| 30 May 2013 | KINX | 2319–2349 | 190 | 4708 | 0.206 |
| 30 May 2013 | KTLX | 2228–2258 | 200 | 4708 | −0.365 |
| 18 June 2013 | KRAX | 2200–2230 | 180 | 4460 | −0.591 |
| 19 June 2013 | KLBB | 2202–2232 | 190 | 4812 | −0.062 |
| 28 August 2013 | KDTX | 0324–0354 | 215 | 4845 | −0.273 |
| 30–31 August 2013 | KBIS | 2340–0010 | 190 | 4561 | −0.259 |
| 11 May 2014 | KUEX | 1956–2026 | 140 | 3368 | −0.010 |
| 27 April 2015 | KFWS | 0139–0209 | 180 | 3502 | −0.166 |
| 8 May 2015 | KFDR | 2052–2122 | 150 | 4223 | −0.550 |
| 19 May 2015 | KTLX | 1910–1940 | 180 | 4000 | −0.073 |
| 27 May 2015 | KDDC | 1933–2003 | 180 | 3764 | 0.026 |
| 4 June 2015 | KFTG | 2208–2238 | 180 | 4560 | 0.016 |
| 18 September 2015 | KEAX | 2239–2309 | 180 | 4497 | 0.004 |
| 2 February 2016 | KDGX | 2018–2048 | 130 | 3242 | 0.206 |
| 24 February 2016 | KRAX | 2030–2100 | 115 | 2992 | 0.205 |
| 29 April 2016 | KFDR | 1959–2029 | 160 | 3923 | 0.194 |
| 21 October 2017 | KFDR | 2153–2223 | 140 | 4047 | −0.383 |

An initial set of nontornadic supercells was also taken from [11] and extended forward in time through 2020. The additional nontornadic supercells were identified as by [11]: in summary, they were required to contain radar features consistent with supercells as described by [22], to be within 100 km of a polarimetric Weather Surveillance Radar-1988 Doppler (WSR-88D), and to be separate from other convective cells with Z<sub>HH</sub> > 20 dBZ. From an initial list of 94 candidate storms, TGF times were identified using normalized rotation (NROT) in Gibson Ridge GR2 Analyst software, commonly used by operational meteorologists. NROT is described by [23,24]. It is calculated as the magnitude of the maximum inbound velocity plus the magnitude of the maximum outbound velocity, the sum of which is divided by two. For the radar datasets used here, which are all super-resolution [25], the azimuthal gradient of velocity is calculated over a 9 pixel by 9 pixel box and corrected for distance to the radar. After the maximum NROT value was recorded for each nontornadic supercell, storms were removed from the dataset if (1) they were too close to the radar or too far from the radar for a high-quality analysis; (2) they did not remain isolated from other convection; (3) they did not have a low-level vortex and therefore did not exhibit TGF; and (4) they had low-level rotation but never had NROT ≥ 1.0, indicating that low-level rotation was never strong (strong/substantial rotation is defined as NROT ≥ 1.0; [23,24]). This left 36 nontornadic supercells (Table 2). The TGF time for these storms was defined as the time of maximum base-scan NROT. Using a minimum base-scan NROT threshold of 1.0 ensures that storms included possess a strong low-level vortex and are in that way comparable with the tornadic storm subset.

**Table 2.** The date, radar, analysis period (UTC), forward flank angle (FFA; degrees), freezing level (FL; meters), and Z<sub>DR</sub> calibration factor (dB) for each of the nontornadic cases included in the pre-TGF analysis. The end of the analysis period is the TGF time.

| Date | Radar | Analysis Period | FFA | FL | Z<sub>DR</sub> Calibration Factor |
|---|---|---|---|---|---|
| 1 June 2012 | KAMA | 2303–2333 | 150 | 4735 | 0.114 |
| 2 April 2013 | KGRK | 1940–2010 | 150 | 3800 | −0.388 |
| 7–8 April 2013 | KSGF | 2250–2320 | 190 | 3374 | −0.157 |
| 25 May 2013 | KUDX | 2110–2140 | 180 | 4131 | −0.121 |
| 24–25 July 2013 | KUEX | 2336–0006 | 200 | 4275 | −0.269 |
| 6–7 August 2013 | KMPX | 2238–2308 | 180 | 3885 | −0.039 |
| 13 August 2013 | KDIX | 1222–1252 | 180 | 4079 | −0.315 |
| 14–15 August 2013 | KAMA | 2321–2351 | 190 | 4660 | 0.237 |
| 14 October 2013 | KDDC | 1936–2006 | 170 | 3816 | 0.161 |
| 26–27 October 2013 | KFWS | 2338–0008 | 190 | 3567 | −0.440 |
| 3 April 2014 | KICT | 0137–0207 | 140 | 3828 | 0.183 |
| 24 April 2014 | KDYX | 0007–0037 | 170 | 4215 | −0.101 |
| 10 May 2017 | KFDX | 0457–0527 | 170 | 4303 | −0.329 |
| 18 May 2017 | KVNX | 2118–2148 | 130 | 3948 | −0.162 |
| 12 June 2017 | KCYS | 2054–2124 | 140 | 4554 | 0.107 |
| 28 June 2017 | KDMX | 2210–2240 | 200 | 4363 | −0.285 |
| 29 May 2018 | KDDC | 2038–2108 | 170 | 4463 | −0.143 |
| 2 October 2018 | KPBZ | 2056–2126 | 210 | 3919 | 0.109 |
| 24 March 2019 | KLSX | 2234–2304 | 160 | 2733 | −0.035 |
| 30 April 2019 | KSRX | 2310–2340 | 180 | 3950 | −0.188 |
| 7 May 2019 | KAMA | 2048–2118 | 170 | 4093 | 0.032 |
| 20 May 2019 | KLBB | 1902–1932 | 160 | 4549 | 0.022 |
| 23 May 2019 | KSGF | 0215–0245 | 170 | 4491 | 0.150 |
| 25 May 2019 | KLBB | 1853–1923 | 130 | 4530 | 0.033 |
| 27 May 2019 | KIWX | 2141–2211 | 180 | 3903 | 0.315 |
| 30 May 2019 | KLWX | 1832–1902 | 150 | 4027 | 0.894 |
| 24 March 2020 | KGWX | 2328–2358 | 180 | 4048 | 0.128 |
| 22 April 2020 | KPOE | 2204–2234 | 190 | 3887 | 0.153 |
| 20 May 2020 | KPUX | 0031–0101 | 130 | 4767 | 0.032 |
| 23 May 2020 | KDVN | 1708–1738 | 160 | 3619 | 0.087 |
| 23 May 2020 | KFDR | 0029–0059 | 120 | 4253 | −0.160 |
| 23 May 2020 | KSRX | 0213–0243 | 180 | 4266 | 0.046 |
| 23–24 May 2020 | KAMA | 2321–2351 | 200 | 4452 | 0.155 |
| 7–8 June 2020 | KBIS | 2337–0007 | 120 | 4230 | −0.049 |
| 20 June 2020 | KLNX | 2238–2308 | 200 | 3990 | −0.390 |
| 27 June 2020 | KLSX | 2125–2155 | 170 | 4529 | 0.313 |

Radar datasets were analyzed using the Supercell Polarimetric Observation Research Kit (SPORK; [26]) after ensuring quality data, including a lack of depolarization streaks, hail spikes, and areas of anomalously low cross-correlation coefficient (ρ<sub>hv</sub>). Three necessary inputs to SPORK for each dataset are the supercell forward flank angle, environmental freezing level, and Z<sub>DR</sub> calibration factor. The supercell forward flank angle is an average over the analysis period, estimated following the method depicted in Figure 1. The environmental freezing level was determined using observed soundings, which are usually available at 0000 and 1200 UTC and occasionally at other times if special soundings are launched. The freezing level was interpolated between the surrounding vertical levels. Many storms were located very near sounding sites; in that case, soundings from that site were used to estimate the freezing level. If a sounding was available within 2 h of the radar analysis period its freezing level was taken, but if no sounding was available within 2 h, an average was taken of the two soundings temporally surrounding the analysis period. For storms not located near a sounding site, an average freezing level was taken from the surrounding two or three soundings that best represented the storm location. In this case, the same temporal constraints were applied. Finally, a Z<sub>DR</sub> calibration factor was determined for each radar dataset following the procedure of [27], as applied by, e.g., [9]. This is a scatterer-based calibration factor which assumes a 'correct' Z<sub>DR</sub> value of 0.15 dB in ice crystals within thunderstorm anvils at an altitude 1.5 km above the ambient 0 °C level. One Z<sub>DR</sub> calibration factor was derived from the radar scan nearest the midpoint of the analysis period, which is sufficient since Z<sub>DR</sub> calibration does not drift substantially on temporal scales of importance to this analysis. The forward flank angle, freezing level, and Z<sub>DR</sub> calibration factor for each dataset are given in Tables 1 and 2. Finally, a Z<sub>DR</sub> threshold of 3.25 dB was used to identify the Z<sub>DR</sub> arc region, and a K<sub>DP</sub> threshold of 1.5 deg km<sup>−1</sup> was used to identify the K<sub>DP</sub> foot (both as in [26]).

![Figure 1. Method used to estimate the supercell forward flank angle. Base radar reflectivity (Z_HH) image from a tornadic supercell scanned by the Dodge City, Kansas WSR-88D (KDDC) at 0005 UTC on 19 May 2013, colorized on a dBZ scale from -25 to 75. A circular protractor overlay is centered on the storm with tick labels at 0°, 45°, 90°, 135°, 180°, 225°, 270°, and 315° (meteorological convention, north = 0°). A black arrow drawn from the circle's center points toward roughly 180° (south), indicating the direction of the forward-flank Z_HH gradient — the value used as the SPORK forward-flank-angle input (~180° in this example). A "NORTH" label with an upward white arrow appears at lower right for orientation.](media/atmosphere-12-00581/fig1_radar_quadrants.png)

**Figure 1.** Method used to estimate the supercell forward flank angle, a necessary SPORK input. Example is Z<sub>HH</sub> from a tornadic supercell in the domain of the Dodge City, Kansas, WSR-88D (KDDC) at 0005 UTC on 19 May 2013. Legend indicates Z<sub>HH</sub> values (dBZ). Circle with angles marked indicates meteorological direction (north is 0°). Black arrow is the direction of the forward–flank Z<sub>HH</sub> gradient, the value of which is input to SPORK (in this case, ~180°).

SPORK [26] is a Python-based algorithm designed to facilitate the analysis of polarimetric features of supercell storms which have been examined in prior literature. The algorithm identifies and tracks storm objects, then produces tables containing quantified values of the polarimetric radar features for each storm object during each radar scan within the analysis period. A list of SPORK output variables used for this study and references describing those variables in more detail are included in Table 3. The variables primarily describe supercell inflow characteristics (via Z<sub>DR</sub> arcs; e.g., [1,5,28–30]), hailfall properties, e.g., [1,11,31], and updraft characteristics (via Z<sub>DR</sub> columns; e.g., [11,32–34]). Once output variables were produced for all the radar scans in a storm's analysis period, one value was calculated for each variable by taking the average across all radar scans. Scans without a given feature were omitted from the average for that storm.

**Table 3.** SPORK output variables examined in this study and sample references describing them.

| Variable | Units | Sample References |
|---|---|---|
| Storm speed of forward motion | m s<sup>−1</sup> | [35,36] |
| Storm direction of motion | degrees | [35,36] |
| Area of Z<sub>DR</sub> arc | km<sup>2</sup> | [1,9,11,26] |
| Mean pixel value in Z<sub>DR</sub> arc | dB | [11,26] |
| Median pixel value in Z<sub>DR</sub> arc | dB | [26] |
| Standard deviation of pixel values in Z<sub>DR</sub> arc | dB | [26] |
| Avg. of 10 highest pixel values in Z<sub>DR</sub> arc | dB | [26,29] |
| Base-scan polarimetrically-inferred hail area | km<sup>2</sup> | [1,11] |
| K<sub>DP</sub> foot area | km<sup>2</sup> | [26,37] |
| Storm area with Z<sub>HH</sub> > 35 dBZ | km<sup>2</sup> | [9] |
| Avg. of pixels exceeding 95th percentile of Z<sub>HH</sub> | dBZ | [11,26] |
| Separation distance, K<sub>DP</sub> foot/Z<sub>DR</sub> arc centroids | km | [26,37] |
| K<sub>DP</sub> foot/Z<sub>DR</sub> arc separation angle | degrees | [26,37] |
| Area of 1 dB Z<sub>DR</sub> column 1 km above 0 °C level | km<sup>2</sup> | [9] |
| Maximum Z<sub>DR</sub> column depth | km | [6,11] |
| Average Z<sub>DR</sub> column depth | km | [11] |

## 3. Results

Numerous radar metrics were compared between pre-tornadic and pre-TGF storm subsets (Table 3). Some of the radar features being characterized (e.g., Z<sub>DR</sub> arcs and/or columns) were not present in every storm, so comparisons were only made between those storms in each subset that contained the radar feature of interest. Specifically, among pre-tornadic storms two did not have a defined Z<sub>DR</sub> arc, ten did not have a base-scan region of hailfall, and one did not have a defined Z<sub>DR</sub> column. Among pre-TGF storms, twelve did not have a base-scan region of hailfall and two did not have a defined Z<sub>DR</sub> column. Two comparisons were made between the storm subsets: (1) the distributions of each radar metric were compared (Table 4; Figure 2), and (2) the change in each radar metric was compared leading up to the time of tornadogenesis or TGF (Table 5). Tables 4 and 5 contain some variables not discussed below for completeness.

![Figure 2. Eight-panel grid of violin plots comparing pre-tornadogenesis (P-TG, left violin in each panel) and pre-TGF (P-TGF, right violin in each panel) storm subsets for eight radar metrics. Each violin shows the kernel-density shape of the distribution with individual storm values plotted as blue dots, a yellow horizontal line marking the median, and a black/gray horizontal line marking the mean. Panel (a) Z_DR Arc Area (km^2), p = 0.020, P-TG visibly larger/wider-spread than P-TGF. Panel (b) Z_DR Arc Median (dB), p = 0.047, P-TG slightly higher. Panel (c) Z_DR Arc Avg. of 10 max pixel values (dB), p = 0.049, P-TG higher. Panel (d) Hail Area (km^2), p = 0.502, P-TGF somewhat larger/more variable. Panel (e) Z_HH 95th Percentile (dBZ), p = 0.720, distributions similar. Panel (f) Z_DR-K_DP Separation (km), p = 0.148, P-TG somewhat larger. Panel (g) Z_DR Column Area (km^2), p = 0.126, P-TG larger. Panel (h) Z_DR Column Depth (m), p = 0.105, P-TG larger.](media/atmosphere-12-00581/fig_violin_panels.png)

**Figure 2.** Violin plots of some common radar metrics for pre-tornadogenesis (P-TG) and pre-TGF (P-TGF) storms: (**a**) area of the Z<sub>DR</sub> arc (km<sup>2</sup>), (**b**) median pixel value within the Z<sub>DR</sub> arc (dB), (**c**) average of the 10 maximum pixel values within the Z<sub>DR</sub> arc (dB), (**d**) base-scan area of polarimetrically-inferred hail (km<sup>2</sup>), (**e**) average Z<sub>HH</sub> value of pixels in the storm core exceeding the 95th percentile of Z<sub>HH</sub> (dBZ), (**f**) separation distance between Z<sub>DR</sub> arc and K<sub>DP</sub> foot centroids (km), (**g**) area of the 1 dB Z<sub>DR</sub> column above the ambient 0 °C level (km<sup>2</sup>), and (**h**) maximum Z<sub>DR</sub> column depth (m). In each violin, the yellow line is the median and the black line is the mean, and the WMW *p*-value for each comparison is included in each panel. Dots within the violins are the raw values, one for each storm that contributed.

**Table 4.** Average value of each radar metric examined for pre-tornadogenesis (Pre-TG) and pre-TGF storms, and WMW *p*-value comparing these two populations. *p*-values < 0.05 (95% confidence that the storm subsets are distinct) are highlighted in bold.

| Variable | Units | Pre-TG Avg. | Pre-TGF Avg. | *p*-Value |
|---|---|---|---|---|
| Storm speed of forward motion | m s<sup>−1</sup> | 11.1 | 12.8 | 0.060 |
| Storm direction of motion | degrees | 253.6 | 266.8 | 0.432 |
| Area of Z<sub>DR</sub> arc | km<sup>2</sup> | 78.6 | 40.0 | **0.020** |
| Mean pixel value in Z<sub>DR</sub> arc | dB | 3.69 | 3.64 | 0.090 |
| Median pixel value in Z<sub>DR</sub> arc | dB | 3.68 | 3.61 | **0.047** |
| Standard deviation of pixel values in Z<sub>DR</sub> arc | dB | 0.37 | 0.31 | 0.062 |
| Avg. of 10 highest pixel values in Z<sub>DR</sub> arc | dB | 4.54 | 4.26 | **0.049** |
| Base-scan polarimetrically-inferred hail area | km<sup>2</sup> | 27.8 | 59.9 | 0.502 |
| K<sub>DP</sub> foot area | km<sup>2</sup> | 149.4 | 148.7 | 0.768 |
| Storm area with Z<sub>HH</sub> >35 dBZ | km<sup>2</sup> | 1188.0 | 1782.9 | 0.584 |
| Avg. of pixels exceeding 95th percentile of Z<sub>HH</sub> | dBZ | 56.6 | 57.3 | 0.720 |
| Separation distance, K<sub>DP</sub> foot/Z<sub>DR</sub> arc centroids | km | 8.84 | 7.48 | 0.148 |
| K<sub>DP</sub> foot/Z<sub>DR</sub> arc separation angle | degrees | 78.4 | 79.4 | 0.932 |
| Area of 1 dB Z<sub>DR</sub> column 1 km above 0 °C level | km<sup>2</sup> | 52.3 | 34.9 | 0.126 |
| Maximum Z<sub>DR</sub> column depth | km | 2.91 | 2.50 | 0.105 |
| Average Z<sub>DR</sub> column depth | km | 1.42 | 1.30 | 0.285 |

**Table 5.** Average change of each radar metric examined for pre-tornadogenesis (Pre-TG) and pre-TGF storms, and WMW *p*-value comparing these two populations. Change is defined as the average metric value 0–15 min prior to tornadogenesis or TGF minus the average metric value 15–30 min prior.

| Variable | Units | Pre-TG Avg. | Pre-TGF Avg. | *p*-Value |
|---|---|---|---|---|
| Storm speed of forward motion | m s<sup>−1</sup> | −0.40 | 0.29 | 0.215 |
| Storm direction of motion | degrees | 0.73 | 5.70 | 0.382 |
| Area of Z<sub>DR</sub> arc | km<sup>2</sup> | 21.6 | 8.9 | 0.634 |
| Mean pixel value in Z<sub>DR</sub> arc | dB | −0.01 | −0.02 | 0.634 |
| Median pixel value in Z<sub>DR</sub> arc | dB | 0.01 | −0.02 | 0.317 |
| Standard deviation of pixel values in Z<sub>DR</sub> arc | dB | −0.00 | −0.05 | 0.491 |
| Avg. of 10 highest pixel values in Z<sub>DR</sub> arc | dB | 0.04 | −0.14 | 0.302 |
| Base-scan polarimetrically-inferred hail area | km<sup>2</sup> | 2.7 | −13.1 | 0.207 |
| K<sub>DP</sub> foot area | km<sup>2</sup> | 20.4 | −10.1 | 0.213 |
| Storm area with Z<sub>HH</sub> > 35 dBZ | km<sup>2</sup> | −30.9 | 207.8 | 0.205 |
| Avg. of pixels exceeding 95th percentile of Z<sub>HH</sub> | dBZ | −0.12 | −0.61 | 0.120 |
| Separation distance, K<sub>DP</sub> foot/Z<sub>DR</sub> arc centroids | km | 0.36 | 0.52 | 0.736 |
| K<sub>DP</sub> foot/Z<sub>DR</sub> arc separation angle | degrees | −5.30 | 2.20 | 0.277 |
| Area of 1 dB Z<sub>DR</sub> column 1 km above 0 °C level | km<sup>2</sup> | 11.2 | 0.2 | 0.549 |
| Maximum Z<sub>DR</sub> column depth | km | −0.04 | 0.09 | 0.707 |
| Average Z<sub>DR</sub> column depth | km | −0.04 | 0.06 | 0.367 |

### 3.1. Comparisons between Radar Metric Distributions

Quantified values of the radar metrics in the 30 min prior to tornadogenesis were compared to those in the 30 min prior to TGF. Results of this comparison are shown in Table 4, along with Wilcoxon–Mann–Whitney (WMW) *p*-values for each comparison, e.g., [38]. This nonparametric test was used since the distributions of radar metric values were typically not Gaussian. In Table 4 a *p*-value ≤ 0.05 increases confidence that the two storm subsets are separate, with lower *p*-values indicating increasing likelihood of distinctness. Basic storm characteristics including speed and direction of storm motion and storm size were statistically similar between the storm subsets (Table 4), indicating a fair comparison between reasonably similar storms.

Z<sub>DR</sub> arcs are a signature of raindrop size sorting in storm-relative inflow along the supercell forward flank, e.g., [1,30]. A changing low-level wind field leading up to tornadogenesis/TGF could be manifest as a change in the forward-flank sorting process and therefore as a change in Z<sub>DR</sub> arc extent and magnitude. Among the storms examined here, Z<sub>DR</sub> arcs were significantly larger on average among pre-tornadic storms (average area 78.6 km<sup>2</sup> vs. 40.0 km<sup>2</sup> for pre-TGF storms; *p* = 0.020; Table 4; Figure 2a). Median pixel value in the arc region was also significantly larger for pre-tornadic storms (*p* = 0.047; Table 4; Figure 2b). Mean pixel value and standard deviation of pixel values within the Z<sub>DR</sub> arc were not significantly larger for pre-tornadic storms (Table 4). Finally, the average Z<sub>DR</sub> value of the 10 highest-valued pixels within the arc was ~0.28 dB larger for pre-tornadic storms (*p* = 0.049; Figure 2c).

Hail reaching low levels (observed at base scan) has been related to the life cycle of supercell storms including their near-tornado segments, e.g., [9,11]. Melting hail can strongly affect local thermodynamics through locally-increased cooling and potentially affect the near-tornado portion of some supercells via strengthening of the rear-flank outflow. In the storms examined here, no significant differences were found in base-scan hail area (Figure 2d) or in the average of the top 5% of Z<sub>HH</sub> values (Figure 2e). Notably the hail extent was 115% larger in pre-TGF storms, but given the large variability in values of this metric the result was not significant (Table 4).

Recent research has indicated some promising results linking a large Z<sub>DR</sub>-K<sub>DP</sub> separation angle to tornadic potential in supercell storms, e.g., [26,37]. In this study, the separation angle is nearly identical between the pre-tornadic and pre-TGF storms (Table 4). The distance between the Z<sub>DR</sub> arc and K<sub>DP</sub> foot centroids was ~1.36 km (18%) larger among pre-tornadic storms (Table 4; Figure 2f), though this was not significantly different.

Z<sub>DR</sub> columns have long been used as an updraft proxy signature, e.g., [11,34]. Their size and their maximum depth is expected to increase as updrafts become larger, and their maximum depth is expected to increase with updraft intensity. Prior studies have examined changes in updraft characteristics around the time of tornadogenesis, e.g., [39,40]. Among the storms examined in this study, Z<sub>DR</sub> column area (Figure 2g) and maximum depth (Figure 2h) did not show significant differences between storm subsets because variability between storms was large. Column area and depth were, however, notably larger for pre-tornadic storms—area was 50% larger and maximum depth was 16% larger (Table 4).

### 3.2. Comparisons between Radar Metric Change Leading Up to Tornadogenesis/TGF

It was hypothesized that change in the radar metrics leading up to tornadogenesis may differ from change leading up to TGF. For example, prior work suggests deeper, broader, and steadier updrafts in tornadic supercells, e.g., [11], so here it is hypothesized that Z<sub>DR</sub> column characteristics will exhibit less change leading up to tornadogenesis than to TGF. Likewise, the prior finding of larger and steadier base-scan hail areas in nontornadic supercells [11] could be extended here to the hypothesis that hail area will exhibit less change leading toward tornadogenesis than toward TGF. Though prior studies have provided variable results, the connection between Z<sub>DR</sub> arcs and the low-level storm-relative wind profile, e.g., [30,41], may also suggest that Z<sub>DR</sub> arcs should become larger and more defined prior to tornadogenesis.

For this analysis, the change of a radar metric is defined as average metric value 0–15 min prior to tornadogenesis or TGF minus the average metric value 15–30 min prior. For instance, if the Z<sub>DR</sub> arc increases in size as hypothesized for pre-tornadic storms, the change value would be positive. One change value was calculated for each storm, and an average was taken across all storms in the pre-tornadic and pre-TGF subsets. As in the prior section, Wilcoxon–Mann–Whitney statistics were calculated to assess whether the distributions of change were likely different between storm subsets.

No significant differences in the change values were found when comparing pre-tornadic and pre-TGF storm subsets (Table 5). Storm motion slowed by 0.4 m s<sup>−1</sup> in pre-tornadic storms and increased by 0.3 m s<sup>−1</sup> prior to TGF. Z<sub>DR</sub> arc area increased in both storm subsets (by 21.6 km<sup>2</sup> in pre-tornadic storms and 8.9 km<sup>2</sup> in pre-TGF storms), and though this finding was not significant due to large variability, it is expected given the hypothesis of larger Z<sub>DR</sub> arcs prior to tornadogenesis. Pixel values and their variability within the Z<sub>DR</sub> arc did not appear to change much in either storm subset (Table 5). Polarimetrically-inferred hail area and storm-core Z<sub>HH</sub> values at base scan were less variable leading up to tornadogenesis as hypothesized, though these were not significant (Table 5). Finally, the Z<sub>DR</sub> column became larger in pre-tornadic storms while remaining nearly the same size in pre-TGF storms, and Z<sub>DR</sub> column depth decreased slightly in pre-tornadic storms while increasing in pre-TGF storms (Table 5). Though many of these changes are consistent with the hypotheses which reflect the theory of how these radar signatures should behave around the time of tornadogenesis as discussed below, they are not significant. Nevertheless, it is worthwhile to consider whether temporal changes in the radar metrics may indicate increasing tornado potential, and the initial analysis presented here with relatively small storm subsets does not negate the potential value of these signatures in some nowcasting environments.

## 4. Discussion

In this study, polarimetric radar characteristics of 32 pre-tornadic and 36 pre-TGF supercells were examined. This work was motivated by the potential operational value in knowing whether polarimetric radar signatures and their changes might help diagnose pre-tornadic storms. Though large differences were not expected assuming similar storm-scale processes between supercell subsets, this comparison needed to be made given prior findings in the literature and the potential for operationally valuable findings. An analysis of the polarimetric radar signatures leading up to TGF has not yet been published, though it is an extension of similar prior work examining pre-tornadic periods in supercells [11].

The work presented here is subject to numerous limitations. First, the mode of TGF was not considered for those storms (e.g., undercutting of the mesocyclone by cold air [13]; weak low-level instability [12,17]). Mode of TGF is beyond the ability of our current observational network to diagnose for most storms. Nevertheless, it is possible that TGF mode may alter a storm's radar presentation leading up to TGF. For example, cold air in the vicinity of the low-level mesocyclone could originate with the melting and sublimation of hail, so a possible hypothesis is that the low-level hail signature might be larger in some storms exhibiting that TGF mode. In storms experiencing TGF failure due to weak low-level instability in the updraft vicinity, a possible hypothesis is that a smaller and/or shallower Z<sub>DR</sub> column will be observed leading up to TGF.

The sample of storms analyzed here remains relatively small. Since many of the radar signatures examined are best observed within 100 km of the radar (e.g., Z<sub>DR</sub> arcs and low-level hail signatures), the number of available storms is limited especially since they must be sufficiently close to the radar for 30 min prior to maximum low-level rotation. Though the value of similar statistical analysis increases with larger numbers of data points (individual storms), the number of storms used (>30 for each subset) is large enough that any consistent differences with immediate operational value should be evident. Nevertheless, with storm samples this small the results should be seen as a guide to the most worthwhile future research topics rather than as providing an authoritative answer as to which radar signatures are most valuable for nowcasting.

A similar limitation is the low number of contributing radar scans in the 30 min prior to tornadogenesis or TGF. Thirty minutes typically contained four to eight radar scans—while enough to yield reasonable statistics, this is not ideal and is unlikely to yield statistically significant findings except for the strongest associations. This is especially problematic when looking at change in the radar metrics, which uses 15 min periods. The low number of samples (radar scans) in those 15 min periods was likely a key reason why statistical significance was low for the change comparison leading up to tornadogenesis/TGF (Table 5). This is a limitation that could be overcome by rapid-scan radar systems. Mesocyclone cycling was also not considered in this analysis—15 min samples are typically too short to sample an entire mesocyclone cycle, which at minimum tend to be ~20 min in length, e.g., [42]. Future work with different-length analysis periods prior to tornadogenesis/TGF may be insightful, especially if care is taken to normalize the results with respect to mesocyclone cycles.

A final limitation is environmental uncertainty. For this study, the ambient freezing level was need to calculate Z<sub>DR</sub> column depth and to calibrate the Z<sub>DR</sub> field. Any error introduced was thought to be minimal since observed soundings were used which should have captured spatiotemporal changes to the ambient freezing level. Here numerically-derived soundings were not used as in some prior studies, e.g., [9,11] since they would not have clearly produced additional benefit. Future work attempting to improve near-storm environmental characterization would be beneficial.

Given the theory of how Z<sub>DR</sub> arcs are related to the low-level wind field, e.g., [30,41] and observations showing larger and more pronounced arcs in tornadic storms [11,28,29], it was hypothesized that larger and more pronounced Z<sub>DR</sub> arcs would be observed in pre-tornadic periods than in pre-TGF periods. This was the case; arcs were 97% larger in pre-tornadic storms (*p* = 0.020; Figure 2a) and the median Z<sub>DR</sub> value within the arc was 0.07 dB higher (*p* = 0.047; Figure 2b). Though both results were significant, the Z<sub>DR</sub> median value difference between storm subsets was within the Z<sub>DR</sub> error and too small to be operationally useful. Z<sub>DR</sub> arc size differences within the arc were 0.28 dB larger for pre-tornadic storms (*p* = 0.049), which is larger than the typical Z<sub>DR</sub> error and large enough to become evident in some operational data. These findings suggest that the size and maximum values within the Z<sub>DR</sub> arc may distinguish pre-tornadic supercells, and were possibly the most promising results obtained in this study. They support prior literature which suggests that tornadic supercells are associated with large Z<sub>DR</sub> arc pixel values, e.g., [29]. It is also hypothesized that Z<sub>DR</sub> arcs may become larger and more defined pre-tornado compared to pre-TGF (Table 5), and median values of Z<sub>DR</sub> within the arc did very slightly increase (compared to decrease in pre-TGF periods). Though these findings support our hypothesis, they are not significant and likely not to be operationally useful in most cases. Nevertheless, nowcasters may take special note of a storm exhibiting a Z<sub>DR</sub> arc which is rapidly growing and/or rapidly increasing in magnitude.

Prior research indicates that hailfall may be more cyclic in tornadic supercells and cover a larger area in non-tornadic supercells, e.g., [9,11], though prior work also suggests relatively similar hailfall characteristics across the time of tornadogenesis [1]. Thus, for this study it is hypothesized that larger hail area will be present in pre-TGF storms compared to pre-tornadic storms, and that only small changes in hail area will occur leading up to tornadogenesis and TGF. These hypotheses were supported—hail area was 32.1 km<sup>2</sup> (115%) larger in pre-TGF storms and hail area was least variable leading up to tornadogenesis (Tables 4 and 5)—though these were not significant results. Since the hypotheses were supported, this topic may warrant additional work to determine whether there may be some operational value in hail area and its variation among certain subsets of supercells.

K<sub>DP</sub>-Z<sub>DR</sub> separation may be a valuable tool according to prior studies. For example, [29] found that separation between the Z<sub>DR</sub> arc and K<sub>DP</sub> foot was larger in tornadic storms compared to nontornadic storms, and at tornadic times compared to nontornadic times in the same storm. The separation orientation of the line connecting these centroids and the line representing storm motion (separation angle; [26]) is often larger when the storm relative helicity is large, possibly indicating enhanced tornado potential, e.g., [26,43]. Thus, it was hypothesized that separation distance between the Z<sub>DR</sub> arc and K<sub>DP</sub> foot centroids should be larger in the pre-tornadic storms, and that the separation angle should become larger leading up to tornadogenesis than leading up to TGF. The separation distance was 18% larger in pre-tornadic storms (Table 4), supporting the hypothesis but not a significant result. This indicates that separation distance warrants additional work. Separation angle was not different between storm subsets, though given results in prior studies there remains value in further investigation to explore whether this metric may be useful in some nowcasting situations.

Z<sub>DR</sub> columns are a proxy for size and intensity of the supercell updraft. In prior studies they have been observed to be larger, deeper, and steadier in tornadic supercells, e.g., [11]. Prior studies have also indicated that the supercell updraft may weaken around the time of tornadogenesis, e.g., [44,45]. Thus it is hypothesized that pre-tornadic storms will exhibit relatively large, deep columns. If TGF represents the same general storm-scale process, both tornadogenesis and TGF are hypothesized to be associated with small decreases in Z<sub>DR</sub> column depth leading up to the time of maximum low-level rotation. Both column area and depth were substantially but not significantly larger for pre-tornadic storms, yielding only weak support for the hypothesis but indicating that future work is warranted. Column depth was nearly unchanged leading up to tornadogenesis and exhibited small increases leading up to TGF. As neither result was significant and the change values were very small (Table 5), the results are consistent with the hypothesis.

Many comparisons examined in this study were not statistically significant. Though this was generally expected given the large variability in many of the radar signatures examined here over supercell life cycles, it was important to investigate given the potential for operationally-useful findings. Low statistical significance indicates that a given radar metric is not likely to be immediately useful in nowcasting, though the several more significant results (e.g., *p* ≤ 0.05) indicate that those metrics may be worthy of additional future investigation, e.g., in different storm-scale environments and in cases when multiple storms are simultaneously present. Radar metrics which were not found to be significantly different here may, however, still be considered in future studies, e.g., in the development of an AI-informed model based on a large sample of supercells and associated environments.

The severe storms community has already produced many simulations of tornadic and nontornadic supercells, in some cases with the objective of finding a 'tipping point' past which tornadogenesis occurs. Many of these nontornadic storms have likely experienced TGF. It could be worthwhile to examine some of the radar signatures discussed in this study within these prior datasets to see if observations highlighted here are also present in numerical datasets.

This study, though for a relatively small sample of pre-tornadic and pre-TGF storms, indicates that the polarimetric radar signatures by themselves are generally not likely to be useful to diagnose future tornadic potential of a given supercell storm. The results presented in this paper would best be used in a radar domain with multiple supercells present. Even in that case, however, there are not clear indications here that the radar signatures examined are beneficial to diagnose pre-tornadic storms. Z<sub>DR</sub> arc size and intensity may be the most likely exception. This is due to the large variability of the radar signatures examined, particularly on a temporal scale less than one full mesocyclone cycle. Thus, while this study has identified some differences that may warrant additional work particularly once a larger supercell dataset is available, it appears unlikely that the radar signatures examined here by themselves will become highly beneficial in operations for the purpose of distinguishing pre-tornadic and pre-TGF supercells.

**Funding:** This research and APC were funded by NOAA, grant numbers NA18OAR4590307 and NA19OAR4590340. The author is also supported by an academic appointment at the University of Nebraska-Lincoln.

**Institutional Review Board Statement:** Not applicable.

**Informed Consent Statement:** Not applicable.

**Data Availability Statement:** Radar data used in this study are freely available from the National Centers for Environmental Information (https://www.ncdc.noaa.gov/data-access/radar-data (accessed on 28 April 2021)) or at Amazon Web Services (https://s3.amazonaws.com/noaa-nexrad-level2/index.html (accessed on 28 April 20210)). Archived sounding data are freely available, e.g., from the University of Wyoming sounding archive (http://weather.uwyo.edu/upperair/sounding.html (accessed on 28 April 2021)).

**Acknowledgments:** Brennen Darrah is acknowledged for helpful discussions about the project, and Matthew Wilson is acknowledged for helpful information about SPORK. Three anonymous peer reviewers are thanked for comments which strengthened the manuscript.

**Conflicts of Interest:** The author declares no conflict of interest. The funders had no role in the design of the study; in the collection, analyses, or interpretation of data; in the writing of the manuscript, or in the decision to publish the results.

## References

1. Kumjian, M.R.; Ryzhkov, A.V. Polarimetric signatures in supercell thunderstorms. *J. Appl. Meteor. Climatol.* **2008**, *47*, 1940–1961. [CrossRef]
2. Romine, G.S.; Burgess, D.W.; Wilhelmson, R.B. A dual-polarization-radar-based assessment of the 8 May 2003 Oklahoma City area tornadic supercell. *Mon. Weather Rev.* **2008**, *136*, 2849–2870. [CrossRef]
3. Van Den Broeke, M.S.; Straka, J.M.; Rasmussen, E.N. Polarimetric radar observations at low levels during tornado life cycles in a small sample of classic Southern Plains supercells. *J. Appl. Meteor. Climatol.* **2008**, *47*, 1232–1247. [CrossRef]
4. Tanamachi, R.L.; Heinselman, P.L. Rapid-scan, polarimetric observations of central Oklahoma severe storms on 31 May 2013. *Weather Forecast.* **2016**, *31*, 19–42. [CrossRef]
5. Van Den Broeke, M.S. Polarimetric variability of classic supercell storms as a function of environment. *J. Appl. Meteor. Climatol.* **2016**, *55*, 1907–1925. [CrossRef]
6. Snyder, J.C.; Ryzhkov, A.V.; Kumjian, M.R.; Khain, A.P.; Picca, J. A Z<sub>DR</sub> column detection algorithm to examine convective storm updrafts. *Weather Forecast.* **2015**, *30*, 1819–1844. [CrossRef]
7. French, M.M.; Burgess, D.W.; Mansell, E.R.; Wicker, L.J. Bulk hook echo raindrop sizes retrieved using mobile, polarimetric Doppler radar observations. *J. Appl. Meteor. Climatol.* **2015**, *54*, 423–450. [CrossRef]
8. Houser, J.L.; Bluestein, H.B.; Snyder, J.C. Finescale radar examination of the tornadic debris signature and weak-echo reflectivity band associated with a large, violent tornado. *Mon. Weather Rev.* **2016**, *144*, 4101–4130. [CrossRef]
9. Van Den Broeke, M.S. Polarimetric radar metrics related to tornado life cycles and intensity in supercell storms. *Mon. Weather Rev.* **2017**, *145*, 3671–3686. [CrossRef]
10. McKeown, K.E.; French, M.M.; Tuftedal, K.S.; Kingfield, D.M.; Bluestein, H.B.; Reif, D.W.; Wienhoff, Z.B. Rapid-scan and polarimetric radar observations of the dissipation of a violent tornado on 9 May 2016 near Sulphur, Oklahoma. *Mon. Weather Rev.* **2020**, *148*, 3951–3971. [CrossRef]
11. Van Den Broeke, M.S. A preliminary polarimetric radar comparison of pretornadic and nontornadic supercell storms. *Mon. Weather Rev.* **2020**, *148*, 1567–1584. [CrossRef]
12. Trapp, R.J. Observations of nontornadic low-level mesocyclones and attendant tornadogenesis failure during VORTEX. *Mon. Weather Rev.* **1999**, *127*, 1693–1705. [CrossRef]
13. Brooks, H.E.; Doswell III, C.A.; Wilhelmson, R.B. The role of midtropospheric wind in the evolution and maintenance of low-level mesocyclones. *Mon. Weather Rev.* **1994**, *122*, 126–136. [CrossRef]
14. Finley, C.A.; Lee, B.D.; Grzych, M.; Karstens, C.D.; Samaras, T.M. Mobile mesonet observations of the rear-flank downdraft evolution associated with a violent tornado near Bowdle, SD on 22 May 2010. Preprints. In *Proceedings of the 25th Conference on Severe Local Storms*, Denver, CO, USA, 2010. Available online: https://ams.confex.com/ams/25SLS/techprogram/paper_176132.htm (accessed on 28 April 2021).
15. Trapp, R.J.; Davies-Jones, R. Tornadogenesis with and without a dynamic pipe effect. *J. Atmos. Sci.* **1997**, *54*, 113. [CrossRef]
16. Markowski, P.M.; Straka, J.M.; Rasmussen, E.N. Direct surface thermodynamic observations within the rear-flank downdrafts of nontornadic and tornadic supercells. *Mon. Weather Rev.* **2002**, *130*, 1692–1721. [CrossRef]
17. Naylor, J.; Gilmore, M.S. Vorticity evolution leading to tornadogenesis and tornadogenesis failure in simulated supercells. *J. Atmos. Sci.* **2014**, *71*, 1201–1217. [CrossRef]
18. Coffer, B.E.; Parker, M.D.; Dahl, J.M.L.; Wicker, L.J.; Clark, A.J. Volatility of tornadogenesis: An ensemble of simulated nontornadic and tornadic supercells in VORTEX2 environments. *Mon. Weather Rev.* **2017**, *145*, 4605–4625. [CrossRef]
19. Coffer, B.E.; Parker, M.D. Is there a "tipping point" between simulated nontornadic and tornadic supercells in VORTEX2 environments? *Mon. Weather Rev.* **2018**, *146*, 2667–2693. [CrossRef]
20. National Centers for Environmental Information. Storm Events Database. Available online: https://www.ncdc.noaa.gov/stormevents/ (accessed on 18 March 2021).
21. Trapp, R.J.; Wheatley, D.M.; Atkins, N.T.; Przybylinski, R.W.; Wolf, R. Buyer beware: Some words of caution on the use of severe wind reports in postevent assessment and research. *Weather Forecast.* **2006**, *21*, 408–415. [CrossRef]
22. Thompson, R.L.; Edwards, R.; Hart, J.A.; Elmore, K.L.; Markowski, P. Close proximity soundings within supercell environments obtained from the Rapid Update Cycle. *Weather Forecast.* **2003**, *18*, 1243–1261. [CrossRef]
23. Cooper, D.T.; Vorst, A.B. Assessing the utility of normalized rotation in detecting tornado development along the Allegheny front. In *Proceedings of the Northeast Regional Operational Workshop XVII*, Albany, NY, USA, 3 November 2016.
24. Gibson, M. 2017: FAQ: NROT and GR-MDA products. In *GRlevelx User Forums*; Gibson Ridge Software, LLC: Suwanee, GA, USA. Available online: http://www.grlevelx.com/owners/ (accessed on 10 March 2021).
25. Torres, S.; Curtis, C. Initial implementation of super-resolution data on the NEXRAD network. In *Proceedings of the 23rd International Conference on Interactive Information Processing Systems for Meteorology, Oceanography, and Hydrology*, American Meteor Society, San Antonio, TX, USA, 2007. Available online: https://ams.confex.com/ams/87ANNUAL/techprogram/paper_116240.htm (accessed on 28 April 2021).
26. Wilson, M.B.; Van Den Broeke, M.S. An automated Python algorithm to quantify Z<sub>DR</sub> arcs and K<sub>DP</sub>-Z<sub>DR</sub> separation signatures in supercells. *J. Atmos. Ocean. Technol.* **2021**, *38*, 371–386. [CrossRef]
27. Picca, J.; Ryzhkov, A. A dual-wavelength polarimetric analysis of the 16 May 2010 Oklahoma City extreme hailstorm. *Mon. Weather Rev.* **2012**, *140*, 1385–1403. [CrossRef]
28. Palmer, R.D.; Bodine, D.; Kumjian, M.; Cheong, B.; Zhang, G.; Cao, Q.; Bluestein, H.; Ryzhkov, A.; Yu, T.; Wang, Y. Observations of the 10 May 2010 tornado outbreak using OU-PRIME: Potential for new science with high-resolution polarimetric radar. *Bull. Am. Meteorol. Soc.* **2011**, *92*, 871–891. [CrossRef]
29. Crowe, C.; Schultz, C.; Kumjian, M.; Carey, L.; Petersen, W. Use of dual-polarization signatures in diagnosing tornadic potential. *Electron. J. Oper. Meteorol.* **2012**, *13*, 57–78.
30. Dawson, D.T.; Mansell, E.R.; Jung, Y.; Wicker, L.J.; Kumjian, M.R.; Xue, M. Low-level Z<sub>DR</sub> signatures in supercell forward flanks: The role of size sorting and melting of hail. *J. Atmos. Sci.* **2014**, *71*, 276–299. [CrossRef]
31. Zrnić, D.S.; Bringi, V.N.; Balakrishnan, N.; Aydin, K.; Chandrasekar, V.; Hubbert, J. Polarimetric measurements in a severe hailstorm. *Mon. Weather Rev.* **1993**, *121*, 2223–2238. [CrossRef]
32. Illingworth, A.J.; Goddard, J.W.F.; Cherry, S.M. Polarization radar studies of precipitation development in convective storms. *Quart. J. R. Meteorol. Soc.* **1987**, *113*, 469–489. [CrossRef]
33. Brandes, E.A.; Vivekanandan, J.; Tuttle, J.D.; Kessinger, C.J. A study of thunderstorm microphysics with multiparameter radar and aircraft observations. *Mon. Weather Rev.* **1995**, *123*, 3129–3143. [CrossRef]
34. Kumjian, M.R.; Ryzhkov, A.V.; Melnikov, V.M.; Schuur, T.J. Rapid-scan super-resolution observations of a cyclic supercell with a dual-polarization WSR-88D. *Mon. Weather Rev.* **2010**, *138*, 3762–3786. [CrossRef]
35. Bunkers, M.J.; Klimowski, B.A.; Zeitler, J.W.; Thompson, R.L.; Weisman, M.L. Predicting supercell motion using a new hodograph technique. *Weather Forecast.* **2000**, *15*, 61–79. [CrossRef]
36. Bunkers, M.J. Observations of right-moving supercell motion forecast errors. *Weather Forecast.* **2018**, *33*, 145–159. [CrossRef]
37. Loeffler, S.D.; Kumjian, M.R.; Jurewicz, M.; French, M.M. Differentiating between tornadic and nontornadic supercells using polarimetric radar signatures of hydrometeor size sorting. *Geophys. Res. Lett.* **2020**, *47*, e2020GL088242. [CrossRef]
38. Corder, G.W.; Foreman, D.I. *Nonparametric Statistics: A Step-by-Step Approach*, 2nd ed.; Wiley: Hoboken, NJ, USA, 2014; p. 288.
39. Lemon, L.R.; Doswell III, C.A. Severe thunderstorm evolution and mesocyclone structure as related to tornadogenesis. *Mon. Weather Rev.* **1979**, *107*, 1184–1197. [CrossRef]
40. Fischer, J.; Dahl, J.M.L. The relative importance of updraft and cold pool characteristics in supercell tornadogenesis using highly idealized simulations. *J. Atmos. Sci.* **2020**, *77*, 4089–4107. [CrossRef]
41. Kumjian, M.R.; Ryzhkov, A.V. Storm-relative helicity revealed from polarimetric radar measurements. *J. Atmos. Sci.* **2009**, *66*, 667–685. [CrossRef]
42. Adlerman, E.J.; Droegemeier, K.K. The sensitivity of numerically simulated cyclic mesocyclogenesis to variations in model physical and computational parameters. *Mon. Weather Rev.* **2002**, *130*, 2671–2691. [CrossRef]
43. Loeffler, S.D.; Kumjian, M.R. Quantifying the separation of enhanced Z<sub>DR</sub> and K<sub>DP</sub> regions in nonsupercell tornadic storms. *Weather Forecast.* **2018**, *33*, 1143–1157. [CrossRef]
44. Brandes, E.A. Mesocyclone evolution and tornadogenesis: Some observations. *Mon. Weather Rev.* **1978**, *106*, 995–1011. [CrossRef]
45. Adlerman, E.J.; Droegemeier, K.K.; Davies-Jones, R.P. A numerical simulation of cyclic mesocyclogenesis. *J. Atmos. Sci.* **1999**, *56*, 2045–2069. [CrossRef]

---

## Reference / Citation

Van Den Broeke, M. Polarimetric Radar Characteristics of Tornadogenesis Failure in Supercell Thunderstorms. *Atmosphere* **2021**, 12(5), 581.

**DOI:** [10.3390/atmos12050581](https://doi.org/10.3390/atmos12050581)

**Journal URL:** https://www.mdpi.com/2073-4433/12/5/581

**Open access** under Creative Commons Attribution (CC BY 4.0) license, MDPI, Basel, Switzerland.
