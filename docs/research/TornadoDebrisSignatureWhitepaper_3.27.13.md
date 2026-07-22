# Tornado Debris Data

**April 2014**

*WSI Corporation — Professional Division of The Weather Company*

---

WSI Corporation has recently developed a Tornado Debris Signature (TDS) algorithm that automatically detects the existence of debris lofted by tornadoes in real-time. The goal of this algorithm is to enable improved decision-making for deployment of insurance CAT team resources.

The success of the TDS algorithm is dependent upon the incorporation of data from the new dual-polarization (dual-pol) capability of the National Weather Service (NWS) radar network. These dual-pol data complement the more traditional radar-based Tornado Vortex Signature (TVS) detections to provide a better assessment of whether debris is lofted from a tornado.

## Tornado Debris Data

### About Dual-Polarization Radar

Before the advent of dual-polarization (dual-pol) capabilities in recent years, the Weather Surveillance Radar – 1988 Doppler (WSR-88D) operated by the NWS was a non-polarimetric radar that only transmitted horizontally-polarized electromagnetic waves. This limited the measurement of reflected energy to only the horizontal direction. Dual-pol radar sends simultaneous electromagnetic waves in both the horizontal and vertical directions. This enables the reflected energy to have not only a horizontal measurement but also a vertical measurement. This two-dimensional representation of returning wave energy offers more detailed insight into the types and characteristics of the particles in the atmosphere (NSSL website).

![Example of conventional versus dual-pol radar, showing a single horizontally-polarized wave for conventional radar and simultaneous horizontal and vertical waves for dual-pol radar](media/TornadoDebrisSignatureWhitepaper_3.27.13/page1-1.png)

**Figure (p.1):** Example of conventional versus dual-pol radar. Image courtesy of NOAA. *Description:* A diagram comparing a conventional radar (single red horizontal sine wave emitted toward a sphere/target) against dual-polarization radar (the same horizontal red wave plus an additional blue vertical sine wave emitted simultaneously toward the target), illustrating that dual-pol radar measures reflected energy in two dimensions instead of one.

---

### About Dual-Polarization Radar (Continued)

With the completion of the dual-pol WSR-88D upgrade, each of the radars now produce 14 new products that help to not only differentiate precipitation types and intensities but also non-meteorological particles. Some of the new radar products include Differential Reflectivity (Zdr), Correlation Coefficient (CC), Specific Differential Phase (Kdp) and Hydrometeor Classification Product (HC). Of these products, the CC is most important to the TDS algorithm because it reflects the similarity between horizontal and vertical returns within a radar bin (Istok et al 2009). Since tornado debris is typically less uniform in shape than meteorological targets, CC can be a clue that a tornado is on the ground, provided a vortex signature is simultaneously present.

![Radar reflectivity image showing a circled area of low correlation coefficient near Shawnee, OK caused by lofted tornado debris](media/TornadoDebrisSignatureWhitepaper_3.27.13/page2-2.png)

**Figure (p.2):** Example of low correlation coefficient as a result of lofted debris from May 19, 2013 Shawnee, OK tornado. Image courtesy of NWS Tulsa. *Description:* A color radar reflectivity map of the Shawnee/Chandler/Oklahoma City, OK region with a supercell storm shown in reds/oranges/greens. A distinct area of low correlation coefficient (marked by a white/pink circular outline) sits within the storm core near Shawnee, indicating a debris ball caused by a tornado.

---

## Characteristics of Tornado-Producing Storms

Tornado-producing storms have several identifiable characteristics in radar data. None of these characteristics alone can demonstrate a tornado is present. When more than one is present, we can be more confident that a tornado is on the ground, lofting debris into the air.

A weak echo region (WER) is an area of markedly lower reflectivity, resulting from an increase in updraft strength (Glickman, 2000) that can develop during the growth phase of a tornadic thunderstorm. A WER may also be surrounded on the top and sides by an area of higher reflectivity (Glockman, 2000), referred to as a bounded weak echo region (BWER). The BWER is suggestive of a strong updraft that is preventing precipitation from reaching the ground, generally associated with a mesocyclone that is capable of producing a tornado.

Couplets in the velocity fields are another indication that a tornado is either occurring or imminent. A couplet occurs when significant velocities towards and away from the radar are found in close proximity to one another. A couplet detected at the lowest elevation angles of the radar volume is evidence of rotation close to the ground. These couplets seen in the radar velocity data are the primary triggers for a TVS.

A hook echo is also a good indication that a tornado is either occurring or imminent. A hook echo is caused by the precipitation being pulled into the mesocyclone (Glickman, 2000). This typically shallow hook-shaped area of reflectivity is often associated with a tornado.

A debris ball is an area of extremely high reflectivity that is generally located near a hook echo. Large (compared to precipitation), irregularly-shaped non-meteorological debris is often lofted into the atmosphere during a tornado (Bodine et al, 2013). The characteristics of this debris ball are the basis for using the correlation coefficient in the calculation of the TDS.

![Side-by-side radar images: reflectivity showing a hook echo (left) and velocity showing a couplet (right) for the May 20, 2013 Avant, OK tornado](media/TornadoDebrisSignatureWhitepaper_3.27.13/page3-3.png)

**Figure (p.3):** Example of a hook echo (left) and velocity couplet (right) from May 20, 2013 Avant, OK tornado. Image courtesy of NWS Tulsa. *Description:* Two side-by-side radar panels. The left panel is a reflectivity display (rainbow color scale) showing a supercell thunderstorm with a hook-shaped appendage circled in yellow on its southwest flank. The right panel is the corresponding velocity display (red/green color scale for outbound/inbound velocities) showing a tight red-green velocity couplet circled in yellow at the same location, indicating rotation.

---

## Raw Calculations

The calculations for the TDS are based on the correlation coefficient and the corresponding reflectivity value at or near the location of a TVS. To have a TDS, a sufficiently low CC, sufficiently high reflectivity and a location within 130 km of a radar site are needed. TDS values can range from 0-10, but there are only a subset of discrete values that are produced in the current version of the algorithm (below).

A TDS detection is only triggered for a value of 10; lower non-zero values represent "failure modes", or those TVS detections that do not have TDS detections associated with them.

| TDS Value | Criteria |
|---|---|
| 0.0 | No data or location beyond range |
| 1.0 | Reflectivity of area below threshold |
| 2.0 | Reflectivity is above threshold, but no CC data available for the area |
| 3.0 | Reflectivity is above threshold, but CC is above threshold |
| 7.0 | Reflectivity is above threshold, but CC is near but slightly above threshold |
| 10.0 | Reflectivity is above threshold, CC is below threshold. Tornado debris likely |

---

## Examples

A few recent examples of TDS detections are now described in more detail.

### Moore, OK

On May 20, 2013, an EF5 tornado with estimated winds of 200-210 mph devastated the community of Moore, OK. The tornado developed at 2:45 PM CST, 4.4 miles west of Newcastle, OK. The tornado produced EF0 and EF1 damage for the first 10 minutes. The first TDS was detected at 2:55 PM CDT (below). During the 50 minutes the tornado was on the ground, it traveled 17 miles and had a maximum width of 1.3 miles. It destroyed homes and businesses in Moore and the surrounding communities (from NWS Norman). The tornado caused 24 fatalities. The evolution of the TDS during the event is shown below.

![Three sequential TDS product maps tracking the tornado debris signature from Newcastle to Moore, OK on May 20, 2013](media/TornadoDebrisSignatureWhitepaper_3.27.13/page5-5.png)

**Figure (p.5), three panels:**
1. *"The first tornado debris signature is detected near Newcastle, OK at 2:55 CDT on 5/20/13."* — Map showing the TDS product (light-to-heavy color scale) with an arrow labeled "Tornado Debris First Detected" pointing to a small debris signature southwest of Moore near Newcastle, OK.
2. *"A tornado debris signature continues to be indicated near the end of the hook echo on the south side of Moore, OK at 3:15 CDT on 5/20/13."* — Map showing the signature has moved north/east, arrow labeled "Tornado Debris south side of Moore, OK."
3. *"The tornado debris signature passes Moore, OK at 3:25 CDT on 5/20/13."* — Map with a white circular outline labeled "Suspended Debris in the air" over Moore, OK, and an arrow labeled "Tornado Debris Passing Moore."

### Cleburne, TX

On May 15, 2013, an EF3 tornado with estimated winds of 140 mph hit the town of Cleburne, TX. The tornado developed in the late evening, 9 miles south-southwest of Cleburne. The tornado produced pockets of EF3 damage during the 8.5 miles it traveled, and had a maximum width of 1 mile. It caused significant damage to dozens of homes in Cleburne (from NWS Dallas/Fort Worth). An image of the TDS during the event is shown below.

![Map showing the TDS "DEBRIS" marker over Cleburne, TX overlaid on a satellite/radar composite with surrounding towns labeled](media/TornadoDebrisSignatureWhitepaper_3.27.13/page6-6.png)

**Figure (p.6):** The tornado debris signature near Cleburne, OK [sic — TX] at 8:49 PM CDT on 5/15/13. *Description:* A composite map (satellite base layer with radar reflectivity overlay in green/yellow/orange/red and a county/warning polygon outline in yellow) centered on Cleburne, TX, with surrounding towns labeled (Granbury, Burleson, DeSoto, Glen Rose, Blum, Italy, Waxahachie). A red downward-pointing triangle marker labeled "DEBRIS" sits directly over Cleburne, indicating the TDS detection location.

---

## Performance

The TDS has shown to be extremely effective at detecting tornadoes on the ground. During a test in late winter and spring 2011-12, 18 tornadoes were confirmed by spotters within 130 km (81 miles) of WSR-88D radars that had completed the dual-pol upgrade. In all 18 cases, a TDS was detected during part of the lifespan of the tornado. The tornadoes ranged in strength from EF0 to EF4. During the test period, TDS signatures occurred in 47 WSR-88D volume scans. Confirmed tornadoes correlated with 45 (96%) of those volume scans with detections.

**The strength breakdown of the 18 tornadoes is as follows:**

| Strength | Count |
|---|---|
| EF0 | 3 |
| EF1 | 2 |
| EF2 | 5 |
| EF3 | 7 |
| EF4 | 1 |

**The breakdown of TDS detection is as follows:**

| Metric | Count |
|---|---|
| # of TDS | 47 |
| # of TDS with a Tornado | 45 |
| # of False TDS | 2 |

## Shortcomings

The data above clearly show that the TDS algorithm is quite useful in the detection of damaging tornadoes; however, there are a few shortcomings. The most obvious is that the storm needs to be sufficiently resolved by a WSR-88D dual-pol radar. Storms that are far from a radar site may not be well resolved due to the curvature of the Earth. Thus, the radar beam will intercept the storm at a higher altitude, making it more likely that the tornadic debris is underneath the beam. For this reason, the algorithm caps any potential debris signatures to within 130 km (or 81 miles) of the radar location. The second shortcoming is that a TDS relies on the detection of a TVS. While many tornadoes, especially strong, long-lasting ones, coincide with a TVS, weaker tornadoes have a lower tendency to trigger TVS. Finally, the TDS will not detect all tornadoes near the radar since many do not loft enough debris to trigger the algorithm, especially those tornadoes over open grasslands or other areas without significant population.

## Summary

The Tornado Debris Signature algorithm represents an evolutionary step in real-time tornado detection. While there are the typical limitations that radar imposes -- such as the tornado's distance from the radar and how high the debris is lofted -- it aids those who need to make quick resource allocation decisions with greater confidence than ever before. When users receive notification of a tornado debris signature, it is likely damaging weather is occurring close to or over the specified location and that action is more likely to be required. Our findings also suggest the false alarm rate is considerably lower than National Weather Service Tornado Warnings; however, limitations in the algorithm and the radar itself means some tornadoes will go undetected when merely considering the algorithm alone. Additional information, such as spotter and chaser reports and confirmations from trained meteorologists, are still required to definitively say that a tornado has or has not touched down.

## References

Bodine, David J., Matthew R. Kumjian, Robert D. Palmer, Pamela L. Heinselman, Alexander V. Ryzhkov, 2013: Tornado Damage Estimation Using Polarimetric Radar. *Weather Forecasting*, 28, 139–158. [Available online at http://dx.doi.org/10.1175/WAF-D-11-00158.1]

Glickman, Todd S. (ed.) (2000). *Glossary of Meteorology* (2nd ed.). American Meteorological Society.

Istok, Michael J., M. A. Fresch, S. D. Smith, Z. Jing, R. Murnan, A. V. Ryzhkov, J. Krause, M. H. Jain, J. T. Ferree, P. T. Schlatter, B. Klein, D. J. Stein, G. S. Cate, and R. E. Saffle. "WSR-88D Dual Polarization Initial Operational Capabilities." *25th Conference on International Interactive Information and Processing Systems (IIPS) for Meteorology, Oceanography, and Hydrology* (2009). [Available online at https://ams.confex.com/ams/pdfpapers/148927.pdf]

"NWS Damage Survey for 5/20/2013 Newcastle/Moore Tornado Event - Update 3". National Weather Service Norman, Oklahoma. May 21, 2013. [Available online at http://mesonet.agron.iastate.edu/wx/afos/p.php?pil=PNSOUN&e=201305211950]

"RESEARCH TOOLS: DUAL-POLARIZED RADAR." NSSL Research Tools: Dual Polarized Radar. Web. 09 Sept. 2013. [Available online at http://www.nssl.noaa.gov/tools/radar/dualpol/]

---

## Reference / Citation

**Title:** Tornado Debris Data (Tornado Debris Signature Whitepaper)
**Issuing organization:** WSI Corporation, Professional Division of The Weather Company
**Publication date:** April 2014 (source file dated 3/27/13 in the InDesign export/filename; PDF metadata creation date 2014-03-27)
**Canonical/mirrored URL:** https://www.carriermanagement.com/assets/TornadoDebrisSignatureWhitepaper_3.27.13.pdf

This is an industry whitepaper (not a peer-reviewed journal article) describing WSI Corporation's proprietary Tornado Debris Signature (TDS) algorithm, built on NWS WSR-88D dual-polarization radar products, intended to support insurance catastrophe (CAT) team resource-deployment decisions. It has no DOI. The concept it documents overlaps with the peer-reviewed literature on polarimetric tornadic debris signatures, notably Bodine et al. (2013), *Weather Forecasting* 28, 139–158, doi:10.1175/WAF-D-11-00158.1 (cited within this whitepaper), and is referenced in the Wikipedia article "Tornado debris signature" (https://en.wikipedia.org/wiki/Tornado_debris_signature).
