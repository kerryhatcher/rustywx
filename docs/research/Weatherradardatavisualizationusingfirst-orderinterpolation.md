# Weather Radar Data Visualization Using First-Order Interpolation

**Authors:** Roman Kvasov¹, Sandra Cruz-Pol², José Colom-Ustáriz², Leyda León Colón², Paula Rees³

¹ University of Puerto Rico at Mayagüez, Department of Mathematical Sciences, PR 00681 — roman.kvasov@upr.edu
² University of Puerto Rico at Mayagüez, Electrical Engineering Department, PR 00681 — SandraCruzPol@ieee.org, colom@ece.uprm.edu, leyda.leon@upr.edu
³ University of Massachusetts at Amherst, Civil & Environmental Engineering, MA 01003 — rees@ecs.umass.edu

**Source / Citation:**
- ResearchGate: https://www.researchgate.net/publication/269329708_Weather_radar_data_visualization_using_first-order_interpolation
- NASA ADS: https://ui.adsabs.harvard.edu/abs/2013igar.conf..924K/abstract
- Also referenced in: 2013 IEEE International Geoscience and Remote Sensing Symposium (IGARSS)

---

## Abstract

In this article we present the visualization of NEXRAD reflectivity data by means of first-order (bilinear) interpolation. We provide the comparison of the raw and interpolated data with the corresponding values obtained from TropiNet radar for weather events occurred in Puerto Rico. The analysis of the relative error of the interpolated data confirms that bilinear interpolation gives better reflectivity estimates than nearest-neighbor interpolation. The efficiency of the bilinear interpolation algorithm makes it suitable for real-time radar data visualization.

**Index Terms:** weather radar, NEXRAD, radar reflectivity, bilinear interpolation, dual-polarized Doppler X-band radar

---

## 1. Introduction

Detection of the atmospheric phenomena is a task of high priority, especially when the observed events may imply an immediate danger to the country and its citizens. In the island of Puerto Rico the examples of such natural hazards include hurricanes, tropical storms and depressions, intense rainfall events and heavy flooding. Weather radars provide data about weather events, which is further visualized and analyzed to determine their structure and magnitude.

The raw reflectivity data provided by weather radars is usually given as a matrix of the discrete averaged values corresponding to the particular gates within the range of the radar. In order to visualize the radar data one needs to convert between the indices in the reflectivity matrix (radius and azimuth) and the rectangular output image coordinates. Due to the discrete nature of the obtained data, the output of such conversion represents the grid of discrete reflectivity values at certain fixed locations of the image. These values are further used to visualize the data by yielding a nearest-neighbor interpolation.

This approach has two potential major limitations. First, since the radar data is a piecewise-constant (zero-order) approximation of the real weather events, it does not provide a smooth and realistic output image. Second, for continuous weather events, the estimation of the reflectivity at a particular location within the range of the radar is expected to also depend on the values at neighboring gates, which is not the case for the zero-order type of interpolation.

Weather data over Puerto Rico is provided by NEXRAD (Next-Generation Radar), – a WSR-88D unit located in Cayey (18.12°N, 66.08°W, 886.63m elevation). The radar scans the entire island every 6 minutes with the maximum range coverage of 462.5 km [1]. The limitations mentioned above become somewhat tedious for the data visualization since the horizontal area covered by a single gate can reach up to 4 km² and thus an additional numerical technique is needed for the proper reflectivity estimation and visualization (Fig. 1).

> **Figure 1.** Current NEXRAD reflectivity data visualization using orthogonal pixels and nearest-neighbor interpolation.

In this paper we propose to use the first-order (bilinear) interpolation estimate of the reflectivity values from the raw NEXRAD data for rendering the two-dimensional images. The algorithm provides accurate estimation for the tropical weather systems and is sufficiently efficient for live visualization of the data.

---

## 2. Proposed Bilinear Interpolation

Bilinear interpolation is one of the most frequently used resampling techniques for the two-dimensional distributed data and is based on the linear interpolation in the direction of each variable: radius and azimuth.

Let R be the range of the radar; N_φ and N_r be the dimensions of the reflectivity Z (N_φ = 360 and N_r = 230 for NEXRAD). Let (φ, r) be the polar coordinates of the point of interest that lies inside the curvilinear trapezoid with four neighboring vertices Z_ij, Z_i(j+1), Z_(i+1)j and Z_(i+1)(j+1) (Fig. 2). The bilinear interpolation estimate of the reflectivity value is given as:

```
        ┌                              ┐ ┌         ┐
Z = [1-ξ  ξ]  │  1-η   η  │ │ Z_ij       Z_i(j+1)   │
        └                              ┘ │ Z_(i+1)j  Z_(i+1)(j+1) │
                                          └         ┘
```

where:

```
            2π(i-1)                    R(j-1)
ξ = φ - —————————    and    η = r - —————————
              N_φ                        N_r
```

are the relative azimuth and relative radius respectively. The expression for the interpolation estimate represents a matrix multiplication and thus can be efficiently computed, which makes it suitable for live data visualization.

For continuous functions the error of the bilinear interpolation is known to be quadratic (proportional to the square of the distance between the data points) and directly depends on the linear combination of the largest absolute values of its second derivatives. This in particular implies that the bilinear interpolation of the linearly distributed data is exact, while for the nonlinear data the error is smaller for functions that are less "oscillating".

> **Figure 2.** Curvilinear trapezoid formed by neighboring reflectivity values and the relative radius and relative azimuth of the point of interest (top view).

> **Figure 4.** An example of the NEXRAD raw reflectivity data and the bilinear interpolation along radius and azimuth.

---

## 3. Validation of the Method

In order to use the bilinear interpolation on NEXRAD reflectivity data we need to make sure that the interpolation does not contribute largely to the error for weather systems formed particularly in Puerto Rico. For this purpose we compare the raw and interpolated NEXRAD data with the reflectivity values obtained from the dual-polarized Doppler X-band radar (known as TropiNet) located at (18.16°N, 67.18°W). The data was collected for the elevation angles of 0.5° for NEXRAD and 3.0° for TropiNet respectively.

Since the centers and the elevation angles of NEXRAD and TropiNet do not coincide, they generally do not observe the same part of the event throughout the whole range of TropiNet. The majority of the TropiNet data corresponds to the lower levels of atmosphere, while NEXRAD even at low elevation angles observes much higher altitudes. Therefore the comparison of the data is valid only for the part of the event that is observed by both radars (Fig. 3).

In order to find the common data we run through all TropiNet gates calculating its Cartesian coordinates (x, y, z), taking into account the elevation of the radar and the curvature of the earth. From the planar coordinates (x, y) we identify which NEXRAD gate they correspond to and compare the altitude of its center with z. The difference in altitudes serves as a threshold for determining the region where the radar beams intersect.

> **Figure 3.** Diagram depicting region of common data from TropiNet and NEXRAD (side view).

The common area observed by both radars consists of approximately 4000 TropiNet gates, which serve as a sample set of true reflectivity values. Whether the TropiNet reflectivity is defined, we calculate the relative error for both raw and interpolated data obtained from NEXRAD and compare their values.

The comparison of the data was performed for the events occurring between October 1 and 2, 2012. Example of the event that has been analyzed is given in Fig. 5.

> **Figure 5.** Event occurred in Puerto Rico at 18:41 on October 1, 2012: TropiNet reflectivity data, NEXRAD raw reflectivity data and the bilinear interpolation along radius and azimuth.

The positive improvement in reflectivity estimation was registered on average for 90.1% of TropiNet gates (see Table 1 for details). The contribution to relative error can be due to beam blockage, ground clutter, or hardware calibration [5].

### Table 1. Comparison of TropiNet and NEXRAD data: improvement achieved after bilinear interpolation of NEXRAD data

| Date of the Event | Number of gates | Gates with improved estimate (%) | Maximum Error (%) |
|---|---|---|---|
| October 1, 2012 | 3,917 | 91.4 | 5.3 |
| October 1, 2012 | 4,031 | 88.2 | 6.1 |
| October 2, 2012 | 4,012 | 87.3 | 6.7 |
| October 2, 2012 | 3,844 | 93.3 | 4.3 |
| October 2, 2012 | 3,983 | 90.1 | 4.8 |

The bilinear interpolation can be efficiently computed and thus is suitable for live image rendering. The interpolated image is obtained by employing additional points and estimating the value at each one of them similar to how it is performed in [6]. The smooth and realistic image is achieved with just 16 additional points per gate (Fig. 6).

> **Figure 6.** Raw NEXRAD data and its bilinear interpolation with 4, 9, 16, 25 and 36 additional points. When the number of additional points per gate is greater than 16, the results are visually indistinguishable.

---

## 4. Conclusion

We presented the visualization of NEXRAD reflectivity data by means of first-order (bilinear) interpolation. The validation of the proposed interpolation is performed via comparison of the nearest-neighbor and interpolated reflectivity data with the corresponding values obtained from TropiNet radar for real weather events occurring in Puerto Rico. The analysis of the relative error of the interpolated data confirmed that bilinear interpolation gives better reflectivity estimates than nearest-neighbor interpolation. The bilinear interpolation algorithm is efficient comparing to higher order interpolation, which makes it suitable for real-time radar data visualization.

---

## 5. Acknowledgements

This work is funded by the Collaborative Adaptive Sensing of the Atmosphere (CASA) Engineering Research Center (NSF AN0313747).

---

## 6. References

1. Ramírez-Beltrán, N.D., Kuligowski, R.J., Harmsen, E., Cruz-Pol, S., Castro, J.M., and Matos, I., "Validation of Hydro-Estimator and NEXRAD over Puerto Rico", 87th American Meteorological Society Annual Meeting, 2008.
2. Keeler J., Hubbert J., Lutz J., "NEXRAD data quality by spectral processing. Spectral processing on NCAR's S-Pol radar" Geoscience and Remote Sensing IEEE International Symposium - IGARSS, 2003.
3. Gao Ying; Fei Yi; Zheng Tao; Peng Yu-xin, "Visualization of 3D spatial data sets applied in radar imaging", International Conference on Microwave and Millimeter Wave Technology, ICMMT, 2059–2062, 2008.
4. Ian G. Cumming, Frank H. Wong, "Digital processing of synthetic aperture radar data", Artech House, London, 2005.
5. Wardah, T., Sharifah Nurul Huda, S. Y., Deni, S.M., Nur Azwa, B. "Radar Rainfall Estimates Comparison with Kriging. Interpolation of Gauged Rain", IEEE Colloquium on Humanities, Science and Engineering Research, 2011.
6. Kuzano S., Sato M., Yokota Y., "Data Interpolation and Resampling for a Synthetic Aperture Radar Data", Geoscience and Remote Sensing IEEE International Symposium - IGARSS, 2011.
7. "Radar Navigation and Maneuvering Board Manual (Pub. 1310)", 7th Edition, 2001.
8. R. J. Doviak, D. S. Zrnic, "Doppler radar and weather observations", 2nd ed. San Diego, CA. Academic Press, 1993.

---

## Relevance to rustywx

- **Problem:** Raw NEXRAD reflectivity data is a discrete matrix of averaged values per gate. Nearest-neighbor (zero-order) interpolation produces blocky, unrealistic images and doesn't account for neighboring gate values.
- **Bilinear interpolation** estimates reflectivity at any point using four neighboring vertices in the curvilinear trapezoid formed by adjacent radials and gates. The estimate is a matrix multiplication, making it efficient for live rendering.
- **Validation:** Compared NEXRAD data against TropiNet (dual-polarized Doppler X-band radar) for events in October 2012. Found positive improvement in reflectivity estimation for ~90% of TropiNet gates on average.
- **Error analysis:** Maximum relative error ranged from 4.3% to 6.7% across tested events.
- **Visual quality:** 16 additional interpolation points per gate produces results visually indistinguishable from higher-order interpolation.
