Handler, S., V. Lakshmanan, T. Schuur, and M. V. D. Broeke, 2014: Determining the likelihood of observing the tornado debris signature at different geographic locations throughout the United States. *J. Operational Meteor.*, in press.

# Determining the Likelihood of Observing the Tornado Debris Signature at Different Geographic Locations throughout the United States

**SHAWN L. HANDLER**
National Weather Center Research Experience for Undergraduates, Norman, Oklahoma
Plymouth State University Department of Atmospheric Science/Chemistry, Plymouth, New Hampshire

**VALLIAPPA LAKSHMANAN**
NOAA/National Severe Storms Laboratory, Norman, Oklahoma
Cooperative Institute for Mesoscale Meteorological Studies, University of Oklahoma, Norman, Oklahoma

**TERRY J. SCHUUR**
NOAA/National Severe Storms Laboratory, Norman, Oklahoma
Cooperative Institute for Mesoscale Meteorological Studies, University of Oklahoma, Norman, Oklahoma

**MATTHEW VAN DEN BROEKE**
Department of Earth and Atmospheric Sciences, University of Nebraska-Lincoln, Lincoln, Nebraska

Corresponding author address: Shawn L. Handler, 67 Langdon Street, Plymouth, NH, 03264
E-mail: slhandler@plymouth.edu

---

## ABSTRACT

With the upgrade of the National Weather Service network of weather radars to dual-polarization, it has become possible to use the new radar moments to detect tornado debris. This study investigates the likelihood of observing the tornado debris signature (TDS) at different geographic locations throughout the United States given that an ongoing tornado is present. The likelihood of observing a TDS varies according to radar geometry and the presence of materials that can be lofted by a tornado. To estimate the likelihood of observing a TDS at different geographic locations, we employed datasets of range from the nearest radar, lowest unblocked height of the radar beam, population density, and a normalized differenced vegetation index (NDVI). We also modeled the relationship of tornado intensity and the vertical extent of the debris signature. Maps for three distinct seasons in 2012 (spring, summer, fall) were generated identifying areas where TDS detection would or would not be likely for tornadoes of EF0-EF2 and EF3+ intensities.

The study indicates that a tornado is likely to be depicted by a TDS on radar if it occurs in regions of close proximity to the radar site, high population density or rich vegetation, and if the tornado itself is strong. The signature is less likely to be seen for weak tornadoes, rural areas that have little vegetation, and regions that experience beam blockage. Tornadoes of EF0 or EF1 intensities are unlikely to exhibit a TDS, and in some areas, like the Gulf Coast, the TDS may only be observed for tornadoes of EF3+ intensity. The range of TDS detection was also found to be limited in areas susceptible to tornadoes which included portions of the Central Plains, Midwest, and Mississippi Valley.

---

## 1. Introduction

With the nationwide upgrade of the National Weather Service's network of weather surveillance radars to dual-polarization capability, dual-polarization radar products are increasingly being used in forecasting, especially for severe weather. The tornado debris signature (TDS) is a dual-polarization signature widely used by forecasters to detect where lofted debris may be present due to a tornado. Polarimetric radar products allow forecasters to differentiate meteorological from non-meteorological scatter which is important for tornado detection.

The TDS can be diagnosed from several polarimetric radar fields: horizontal reflectivity (Z), radial velocity (Vr), correlation coefficient (ρhv), and differential reflectivity (ZDR). The criteria for TDS detection were initially specified by Ryzhkov et al. (2005) as (1.) the presence of a hook echo, (2.) ρhv < 0.8, (3.) a pronounced radial velocity vortex signature, (4.) ZDR < 0.5 dB, and (5.) Z > 45 dBZ. Since then, other studies have suggested modifications to some of the thresholds used to identify a TDS (see WDTB 2011; Bodine et al. 2012; Schultz et al. 2012a). Of the five criteria, ρhv and ZDR are dual-polarization products, with ρhv proven to be the most powerful in TDS detection when collocated with a radial velocity vortex signature (Ryzhkov et al. 2005).

Since meteorological and non-meteorological scatterers have different polarimetric radar signatures (Ryzhkov et al. 2005; Schultz et al. 2012a; Bodine et al. 2012; Van Den Broeke and Jauernic 2014), lofted debris can be depicted through the use of dual-polarization radar. At S band wavelength, ρhv values from 0.8-1.0 typically represent meteorological hydrometeors while values less than 0.8 indicate non-meteorological scatterers (WDTB 2011; Schultz et al. 2012a; NWS Louisville, KY). Irregularly shaped and tumbling debris lofted by a tornado results in anomalously low ρhv values that are well below those of meteorological scatterers (Schultz et al. 2012a) and near-zero ZDR values that are typically much smaller than surrounding areas (WDTB 2011).

There are limitations and caveats to the signature. The correlation coefficient of large hailstones could be within the TDS threshold values of non-meteorological scatterers resulting in false representation of debris (Van Den Broeke and Jauernic 2014). Correlation coefficient values may also be affected by the variability of the differential phase within the radar resolution volume; therefore, ρhv may be dependent on range and ΦDP (Ryzhkov et al. 2005; Schultz et al. 2012a). ZDR bias could occur based on radar scan angles resulting in quality degradation (Ryzhkov et al. 2005; Schultz et al. 2012a; Van Den Broeke and Jauernic 2014).

Debris could also remain suspended in the atmosphere for durations exceeding the life of the tornado, resulting in a false TDS detection depending on the tornado strength (Bunkers and Baxter 2011; Schultz et al. 2012b). Sometimes, there may be a confirmed tornado without a TDS indicating that the vertical extent of the TDS is not within the range of the lowest unblocked height of the radar beam. In some cases, a TDS may have been detected without any reports of associated damage or debris. This might occur due to a low signal to noise ratio (SNR) in the inflow region of the storm (Schultz et al. 2012b). Lastly, the areal width of the TDS may be wider than the actual tornado due to debris centrifuging (Dowell et al. 2005; Van Den Broeke and Jauernic 2014) which might lead to damage paths being off center by 1-4 km in some areas (Carey et al. 2011; Schultz et al. 2012b).

Criteria to determine new TDS detection thresholds have been researched extensively, yet little research has been conducted as to how likely it would be for a weather forecaster to observe a TDS given that a tornado is occurring at a specific location. This is because different geographic areas and land classification zones around the CONUS have different likelihoods due to their being different distances from the radar, the radar potentially being blocked by closer-in obstacles, and varying heights or the radar beam above the ground. In addition, differences in land surfaces can impact the evolution of mesoscale convective systems (Pielke et al. 2011; Kellner and Niyogi 2013). Kellner and Niyogi (2013) suggested that changes in land cover might influence tornado touchdown location because of the generation of local vorticity boundaries. Recently, Van Den Broeke and Jauernic (2014) examined spatial and temporal characteristics of the TDS and discovered that the range from the radar and different land classification zones influence TDS detection.

Determining the likelihood of observing a TDS in different geographic locations of the CONUS should be of importance to forecasters since different environmental and societal variables can influence TDS detection on radar. This study expands upon the Van Den Broeke and Jauernic (2014) study, but focuses on determining the likelihood of being able to detect a TDS based on the distance to the nearest radar, lowest unblocked height of a radar beam, vegetation and land cover, population density (used to define the location of urban areas), tornado intensity, and radar beam blockage due to terrain, or built up infrastructure. This study also assesses the relationship of the lowest unblocked height of the radar beam, tornado intensity and the vertical extent of the TDS to the overall detectability of the TDS since tornadoes of greater intensities tend to loft debris to higher altitudes and feature debris fields of greater areal width (Schultz et al. 2012a; Van Den Broeke and Jauernic 2014).

## 2. Data and methods

### a. Radar variables

Data used in this study were drawn from Van Den Broeke and Jauernic (2014), who presented statistics describing certain spatial and temporal characteristics related to TDS observations (see Van Den Broeke and Jauernic (2014) Tables 1, 2, 7, and 8).

The range from radar for TDS detection was 10-160 km for this study (Van Den Broeke and Jauernic 2014, Schultz et al. 2012a). The midpoint of each range group and the percentage of reported tornado events that presented a TDS (see Table 7 in Van Den Broeke and Jauernic (2014)) were used to create a regression model representing the likelihood of observing a TDS based on range to the nearest radar. The linear regression curve (Fig. 1) yielded the equation below used to calculate the likelihood of TDS detection for each distance within range of the radar.

```
l = 17.4964 + 354.9634/d - 0.0838d     (1)
```

Where *l* represents the likelihood of observing the TDS, and *d* represents the distance from the radar.

![Figure 1. Likelihood of TDS detection based on range from radar. Circles represent data points found from Van Den Broeke and Jaurenic (2014). Range from radar (x-axis) has units of km.](media/Handler-finalpaper/fig1_likelihood_vs_range.png)

*Figure 1 description: A scatter/line plot titled "Likelihood of observing TDS." The y-axis is "Fraction of tornadoes with TDS" (0 to 1.0) and the x-axis is "Range from radar" (0 to 150+ km). The fitted curve decays sharply from near 1.0 at very close range (~10 km) down to roughly 0.1-0.2 by 150-160 km, following the hyperbolic/linear-combination form of Eq. (1). Circles mark the empirical midpoint data points from Van Den Broeke and Jauernic (2014).*

Radar variable datasets computed using the Warning Decision Support System Integrated Information (WDSS-II; Lakshmanan et al. 2007) software included the distance to the nearest radar, lowest unblocked height of the radar beam, and lowest elevation scan angle corresponding to the lowest unblocked height.

The lowest unblocked height represented the lowest elevation of the radar beam above ground level (AGL) in the volume scan that was not affected by beam blockage. This variable was used to determine which radar site would be used for determining the distance to the nearest radar, including situations where overlapping coverage between two or more radars occurred. The midpoint height of the radar beam was computed using the technique of Maddox et al. (2002). Half a degree of beam width was then subtracted in order to yield the lowest unblocked height not affected by beam blockage.

The tornado intensity and vertical extent of the TDS relationship was assessed by creating a linear regression model based off the vertical extent of the TDS data provided in Table 2 by Van Den Broeke and Jauernic (2014) and the lowest unblocked height of a radar beam dataset (Fig. 2). The linear model featured mean and one standard deviation values to create best fit curves (Fig. 3). The minus-one standard deviation curve was used as it best represented almost all tornadoes whose debris field reached the given height before the TDS would be detectable at a given location. Using the lowest unblocked height as the input variable for the equation governing the minus-one curve (Equation 2 below), a map displaying the tornado intensity needed to observe a TDS was generated.

```
c(s-1) = (16.8054v - 2.809) / 10     (2)
```

Where c(s-1) represents the minus-one standard deviation curve and v represents the input variable of the lowest unblocked height.

![Figure 2. Lowest unblocked height of a radar beam. Units of elevation are in km AGL. Black circles represent WSR-88D radar sites. Range of each radar site is 10-160 km.](media/Handler-finalpaper/fig2_lowest_unblocked_height.png)

*Figure 2 description: A CONUS map titled "Lowest Unblocked Height Above Ground Level." Each WSR-88D radar site is shown as a set of concentric colored rings (rainbow color scale from purple/low to red/high, per the legend "Height AGL") extending out to the 10-160 km detection range, visualizing how the lowest unblocked beam height increases with distance from each radar and is elevated further by terrain/beam blockage in mountainous areas.*

![Figure 3. Tornado intensity (y-axis) and vertical extent of the TDS (x-axis) relationship. Units for the vertical extent of the TDS are in km. Tornado intensity represents the EF scale.](media/Handler-finalpaper/fig3_intensity_vs_vertical_extent.png)

*Figure 3 description: A plot titled "Tornado Rating vs. Vertical Extent of TDS," y-axis "Tornado Intensity" (EF0 to EF4+) and x-axis "Vert. Ext. of TDS" (0 to 7 km). Three fitted curves are shown: green = "-1 S.D.", red = "Mean", and blue = "+1 S.D.", each rising from the origin with increasing steepness in the order green (steepest/leftmost), red (middle), then blue (shallowest/rightmost) — i.e., for a given vertical extent, the -1 S.D. curve implies the lowest tornado intensity needed, while +1 S.D. implies the highest.*

### b. Vegetation, land cover, and population density influence

Percentages of tornado events that featured a TDS for different land classification types were used in this study (see Table 8 in Van Den Broeke and Jauernic 2014). Similar land classification types were combined and a weighted average was used to determine new percentage values representing tornado events that featured a TDS (Table 1).

The 2012 Normalized Difference Vegetation Index (NDVI), 2010 North American population density dataset, and USGS 1992 United States conterminous land cover dataset were used to determine how land cover affected TDS detection. The vegetation index was provided as a raster by NASA's Earth Observatory Group (NEO) for three months in 2012: April, July, and November each representing a different season (Fig. 4a-c). Vegetation values were classified into three groups and their respective thresholds (Table 2) by comparing index values with the 200-meter resolution 1992 USGS Conterminous United States land cover map. These sources were used to identify regions of different land classification types. Once the index values were determined, they were matched with the respective land classification type and the percentages were used in generation of the TDS detection maps.

**Table 1.** Land classification groups and newly determined weighted average of events that exhibit a TDS.

| Land Classification | % TDS |
|---|---|
| Water | 19 |
| Conif/Decid Forest | 17.9 |
| Urban | 25.4 |
| Grass/Soil/Crop | 14.4 |

**Table 2.** NDVI Table. Vegetation index values based on season and land classification groups.

| Season / Land Class. | Spring (April) | Summer (July) | Fall (November) |
|---|---|---|---|
| Water | -0.1-0.1 | -0.1-0.1 | -0.1-(-0.016) |
| Grass/Soil/Crops | 0.11-0.67 | 0.11-0.69 | -0.015-0.64 |
| Decid/Conif Forests | 0.68+ | 0.7+ | 0.65+ |

![Figure 4a-c. NDVI images for spring, summer, and fall (respectively) used to represent vegetation values. Index values range from -1 to 1 with more "greenness" representing values close to 1.](media/Handler-finalpaper/fig4a_ndvi_spring.png)

*Figure 4a description: Satellite-derived NDVI composite of North America for April (spring) 2012, colored in shades of tan/brown to green — greener shading over the eastern/southeastern U.S. and Pacific Northwest, browner/tan over the arid Southwest and Great Plains still in early greenup.*

![](media/Handler-finalpaper/fig4b_ndvi_summer.png)

*Figure 4b description: Same NDVI product for July (summer) 2012 — deep green covers most of the eastern half of the continent and the corn/soy belt of the Midwest, with the Southwest, Great Basin, and High Plains remaining tan/brown.*

![](media/Handler-finalpaper/fig4c_ndvi_fall.png)

*Figure 4c description: Same NDVI product for November (fall) 2012 — greenness has receded relative to summer, with more tan/brown across the central and northern U.S. as vegetation senesces, while some green remains in the Southeast.*

The raster population density dataset was provided by Columbia University's Center for International Earth Science Information Network (CIESIN) and used to determine if an area was to be considered an urban region (Fig. 5). Grid points with a population density greater than 2,500 persons per square kilometer, were considered urban and were assigned a likelihood percentage value of 25.4 as discussed by Van Den Broeke and Jauernic (2014).

![Figure 5. 2010 North American population density dataset. Population density is measured in persons per square km. Darker shades of orange represent regions of population density greater than 2,500.](media/Handler-finalpaper/fig5_population_density.png)

*Figure 5 description: CONUS map titled "2010 Population Density," with a yellow-to-dark-orange color ramp (legend "NA 2010 population, Persons per sq. km": 1-2,500 / 2,500-10,000 / 10,000-50,000 / 50,000-500,000 / 500,000+). Dark orange/red clusters mark major metropolitan areas (e.g., Northeast corridor, Chicago, California cities, Texas Triangle) against a pale-yellow rural background; black dots mark WSR-88D radar sites.*

### c. Determining overall likelihoods

Overall likelihood maps were generated for determining the likelihood of observing a TDS for the spring, summer, and fall seasons for two different tornado intensity groups. The first sets of maps were for tornadoes of EF0-EF2 strength, and the second set featured tornadoes of EF3+ intensity. The raster datasets were all rescaled to having matching degree resolution and map projections and fused to generate the overall coverage maps. These maps were generated by combining the likelihood of TDS detection based on range from radar, NDVI vegetation index, population density, and the tornado intensity needed to observe a TDS using Equation 2. The minimum percentage value (a fuzzy logic AND operation) between the four input variables at a given location was tagged as the overall likelihood of observing a TDS at that location given there would be an ongoing tornado. It should be emphasized that this is conditional on there being a tornado on the ground, i.e., tornado climatology is not taken into account – it is the likelihood that a tornado at that location presents a TDS on the NEXRAD network of radars.

## 3. Analysis and Discussion

The likelihood products can be visualized as map products on any geographic information system (GIS). The following section describes the results found.

### a. Coverage maps

Determining the distance to the nearest radar was important for showing how well the TDS would be detected. Ranges far from the radar would result in the beam being too wide to observe the debris signature while being too close to the radar would require higher scan elevation angles being used, or may result in no TDS detection due to the "cone of silence". It has been noted in a study by Maddox et al. (2002) that WSR-88D coverage is limited below 2 km AGL over much of the CONUS. However, at 3 km AGL much of the East Coast and Southeast United States have coverage by multiple radars.

![Figure 6. Range of TDS detection from the nearest radar. Black circles represent WSR-88D radar sites. Each individual radar site has a range of detection of 10-160 km.](media/Handler-finalpaper/fig6_distance_to_nearest_radar.png)

*Figure 6 description: CONUS map titled "Distance to the Nearest Radar," each WSR-88D site shown as concentric rainbow rings (legend "Distance to the nearest radar," dark blue near = short range, progressing through green/yellow/orange/red at greater range) out to 160 km, tiled across the country with black dots marking radar sites.*

Figure 6 displays the radar coverage for TDS detection throughout the CONUS. Over portions of the East Coast and southeastern U.S., coverage to observe a TDS is fairly complete out to 160 km. However, tornado prone locations including the northern Plains, central Minnesota, portions of the mid-Mississippi Valley, and western Texas, experienced areas of reduced to no coverage. Further west, reduced coverage continued in close proximity of the Rocky Mountains. This map illustrates that it would be unlikely to observe the debris signature given an ongoing tornado was present in an area which surpassed the 160 km range of detection.

Based on the regression model (Fig. 1) and Eq. 1 discussed earlier, the likelihood of TDS detection based on range from radar map was generated (Fig. 7). Maximum TDS detection occurred when tornadoes were within close range of the radar, and detection rate decreased as range increased. The maximum value generated was 52 percent representing a distance of 10 km away from the radar. Most of the coverage values ranged between 10 to 30 percent. It's important to note that this map was not influenced by tornado intensity since weak tornadoes may not reveal a signature at close range.

![Figure 7. Likelihood of observing a TDS given an ongoing tornado based on range from the nearest radar. Likelihood of detection is shown as a percentage. Range from radar is the same as Fig. 6.](media/Handler-finalpaper/fig7_likelihood_tds_range.png)

*Figure 7 description: CONUS map titled "Likelihood of TDS Detection Based on Range from Radar," each radar site shown with concentric rings shaded on a blue (low, ~0%) to orange/red (high, up to 52%) scale ("Likelihood of detection" legend), highest likelihood concentrated in small circles right at each radar site and dropping off with distance, giving a "bullseye" pattern repeated at every radar location across the CONUS.*

### b. Vertical extent of the TDS, tornado intensity, and lowest unblocked height relationship

Van Den Broeke and Jauernic (2014) reported that as tornado intensity increased, the potential for tornado debris to be lofted to greater aerial extents increased. This seems plausible given that stronger tornadoes have more vigorous updrafts compared to weaker tornadoes. Figures 3 and Eq. 2, displayed earlier, showed the regression model and equation used. Incorporating the lowest unblocked height of the radar beam (Fig. 2) as the input variable in Eq. 2, the map featuring the tornado intensity needed to observe a TDS was generated (Fig. 8). If the vertical extent of the lofted debris for a specific intensity was below the lowest unblocked height elevation of the radar beam, the TDS would not be detected at the given location for that intensity. However, if the vertical extent of the lofted debris surpassed the lowest unblocked height of the radar beam, a TDS could possibly be detected for the given intensity.

Based on Eq. 2, Fig. 3, and Fig. 8, a TDS could be detected at most locations when an ongoing tornado was of EF2 intensity or greater. TDS detection for tornadoes of EF0 and EF1 intensity would be unlikely due to the vertical extent of the debris field not exceeding the lowest unblocked height of the radar beam AGL. Terrain blockage is prevalent around the mountainous zones of the U.S. as well as portions of the Central Plains where elevation increases. This would require debris fields to be lofted to higher elevations since the lowest unblocked height of the radar beam AGL has increased, resulting in the need for a stronger tornado. Along the Gulf Coast and portions of the Eastern seaboard, an ongoing tornado must be of EF3 or greater intensity to observe the TDS. This may be caused due to the topography the region exhibits, but also to the radars position being elevated above ground level more compared to inland locations.

As range from the radar increases, the lowest unblocked height increases; therefore the tornado intensity needed to observe a TDS would also have to increase. This is noticeable by the red, and light blue colors encompassing the perimeters of certain radar coverage areas.

![Figure 8. Ongoing tornado intensity needed for TDS detection based on the model of Equation 2. Range of radar detection is the same as Fig. 6. Color scale shown depicts EF scale tornado intensities.](media/Handler-finalpaper/fig8_intensity_needed_map.png)

*Figure 8 description: CONUS map titled "Tornado Intensity Needed to Observe TDS," each radar site shown as concentric rings colored by a categorical EF-scale legend ("EF Scale Intensity": 2=dark blue center rings, 3=lighter blue, 4=red outer rings). Most of the area close to each radar needs only EF2, while red (needing EF4) appears at the far perimeters of radar coverage and in mountainous/terrain-blocked regions, indicating higher tornado intensity is required to produce a detectable TDS with increasing range/beam blockage.*

Figures 9a-c show incomplete coverage for an ongoing tornado of EF2 intensity or less for certain radars around the U.S. This is due to beam blockage or increased terrain resulting in the lowest unblocked height to be higher AGL which meant a tornado of stronger intensity would be needed to observe the TDS. However, given that a tornado is ongoing and lofting debris, it would be likely to observe the TDS in the southeastern U.S. for most of the year, except for the Gulf Coast which would require a stronger tornado. This may be due to more densely vegetated areas as the Southeast has abundant vegetation in the form of deciduous and coniferous forests.

During the spring and fall months for portions of the Great Plains, and Midwest, detecting a TDS on radar would be somewhat likely (10-20 percent). This may be due to the available vegetation the region has which would serve as debris for the tornado. Portions of the Northeast may also be more likely to observe a TDS for tornadoes of EF2 intensity or weaker for all seasons compared to the Central and Southern Plains, especially during the summer. Highly vegetated areas around the Northeast are prevalent compared to the Central and Southern Plains which feature mainly agricultural lands.

A TDS would also be more detectable in areas with high population density. Urban environments have the highest likelihood of observing a TDS (see Van Den Broeke and Jauernic 2014). WSR-88D radar sites are positioned within close proximity to urban centers (Fig. 5) increasing the chances of TDS detection given the presence of anthropogenic material urban centers have readily available to be lofted by a tornado.

Figures 9d-f feature the likelihood of observing a TDS for an ongoing tornado of EF3+ intensity, given that the ongoing tornado is of at least EF2 strength. If it's possible to detect a TDS for an EF2 tornado at the given location, detecting a TDS for a tornado of greater intensity is assured since the debris field of the EF2 tornado is exceeding the elevation of the lowest unblocked height of the radar beam.

From these maps, the trend is similar for tornadoes of EF0-EF2 intensity except that the coverage is complete for each radar site throughout the United States. Ongoing tornadoes of EF3+ intensity are more likely to be seen in areas of dense vegetation and near urban areas close to radar sites. It would be likely (20-25.4 percent) to observe a TDS for ongoing tornadoes of EF3+ intensity in portions of the Central Plains during the spring. However, during the summer and fall seasons, it would be somewhat likely (10-20 percent) to observe a TDS in the Southern and Central Plains unless a tornado was to travel through an urban center. This is due to the available vegetation and anthropogenic material that would be lofted by an ongoing tornado.

The Midwest and Northeast exhibit seasons where TDS detection is more likely to occur for one season than during other seasons mostly due to the increase and decrease of vegetation sources.

### c. Overall likelihood maps

Overall likelihood maps were generated for April, July, and November representing three distinct seasons (Fig. 9a-f). The left column featured tornadoes of weak intensity (EF0-EF2) and the right column featured tornadoes of strong intensity (EF3+). However, based on Fig. 8 above, tornadoes of EF0-EF1 intensity were unlikely to exhibit a TDS. Therefore, Fig. 9a-c would best represent tornadoes of EF2 intensity only.

![Figure 9a. Overall likelihood of TDS detection: EF0-2 Likelihood, Spring 2012.](media/Handler-finalpaper/fig9a_ef02_spring.png)

![Figure 9b. Overall likelihood of TDS detection: EF0-2 Likelihood, Summer 2012.](media/Handler-finalpaper/fig9b_ef02_summer.png)

![Figure 9c. Overall likelihood of TDS detection: EF0-2 Likelihood, Fall 2012.](media/Handler-finalpaper/fig9c_ef02_fall.png)

![Figure 9d. Overall likelihood of TDS detection: EF3+ Likelihood, Spring 2012.](media/Handler-finalpaper/fig9d_ef3plus_spring.png)

![Figure 9e. Overall likelihood of TDS detection: EF3+ Likelihood, Summer 2012.](media/Handler-finalpaper/fig9e_ef3plus_summer.png)

![Figure 9f. Overall likelihood of TDS detection: EF3+ Likelihood, Fall 2012.](media/Handler-finalpaper/fig9f_ef3plus_fall.png)

*Figure 9a-f description: Six CONUS maps arranged in two columns (left = EF0-EF2, right = EF3+) by three rows (spring/April, summer/July, fall/November 2012), each titled accordingly (e.g., "EF0-2 Likelihood Spring 2012," "EF3+ Likelihood Summer 2012"). Each radar site is shown as concentric rings colored on a blue (low, near 0) to red (high, up to 25.4) "Likelihood of Detection" percentage scale, overlaid on a light state-boundary basemap. Red centers cluster tightly around radar sites/urban areas; blue/gray dominates the outer perimeters and low-vegetation, low-population rural zones. The EF3+ (right column) maps show visibly larger areas of red/orange coverage than the EF0-2 (left column) maps for the same season, and coverage is noticeably more complete (fewer detection gaps) on the EF3+ side. Caption states: "Figures 9a-f. Overall likelihood of TDS detection. Figures 9a-c feature likelihood of TDS detection for EF0-EF2 intensities during spring, summer, and fall months respectively. Figures 9d-f are formatted the same, except for tornadoes of EF3+ intensities. Red values depict most likely areas, while dark blues represent less likely to unlikely areas. Range of detection is the same as previous images. Percentage values represent the overall likelihood, not probability."*

Figures 9a-c show incomplete coverage for an ongoing tornado of EF2 intensity or less for certain radars around the U.S. This is due to beam blockage or increased terrain resulting in the lowest unblocked height to be higher AGL which meant a tornado of stronger intensity would be needed to observe the TDS. However, given that a tornado is ongoing and lofting debris, it would be likely to observe the TDS in the southeastern U.S. for most of the year, except for the Gulf Coast which would require a stronger tornado.

## 4. Conclusions

Using data drawn from Van Den Broeke and Jauernic (2014), coverage maps were generated to best represent the likelihood of detecting a TDS for an ongoing tornado at different geographic locations throughout the United States. These maps were generated from different environmental and societal variables that influence TDS detection.

Based on the maps analyzed (i.e., Figs. 2, 7, 8, 9a-f), region's most likely to observe a TDS from an ongoing tornado would need to satisfy the following criteria: close proximity to the radar which is not affected by beam blockage, an ongoing tornado of at least EF2 strength, and a region that is rich in vegetation (i.e. forests) or within an urban setting with readily available anthropogenic materials that could be lofted by the tornado. Regions less likely or unlikely to detect a TDS include locations at a greater distance away from the radar, tornadoes of EF2 or less intensity, and an area with little (i.e. crops, grass) to no vegetation (i.e. soil) or anthropogenic materials readily available to be lofted by a tornado.

Given the range of radar used for TDS detection in this study, it can be shown that there is little to no coverage in some areas that are nevertheless susceptible to tornadoes. This includes portions of the Central Plains, western Texas, central Minnesota, Missouri, and regions encompassing the lower Mississippi River Valley. However, a majority of the East Coast features complete coverage as does the Southeast.

Another notable result from this study involved the elevation of the lowest unblocked height of the radar beam along the Gulf Coast. The lowest unblocked was noticeably higher along the Gulf Coast compared to neighboring areas (readers should realize that because the height is AGL, it is counterintuitively higher in low-lying areas and lower in hilly areas). Therefore, tornadoes would need to loft debris to a greater vertical extent in order for a TDS to be observed. This would mean that a tornado of greater intensity, an EF3 in this study, would have to be present to loft debris to a higher vertical extent. This would become important during periods of Atlantic hurricane activity as hurricanes making landfall can potentially have tornadoes disguised in the spiral rainbands. These tornadoes are not usually of EF3 intensity, and so are unlikely to present a TDS on radar, but may still cause substantial damage.

Future research regarding this project could continue which would include incorporating case study examples and different methods of analyzing land cover and land use. Using different criteria defining the TDS could also be applied, which may yield a different dataset and different results. A study analyzing how transition zones affect tornado strength and TDS appearance could be conducted. Although the available dataset is small due to recent completion of upgrades to dual-polarization radar products, in future time a larger dataset would be useful. Our modeling in the form of regression equations would be made more accurate with a larger dataset featuring more events. Applying tornado climatology as an input variable would also make these maps less of a theoretical approach and more based on factual data. Lastly, future studies analyzing different polarimetric values for different land classification types could result in new criteria used to differentiate and possibly determine what form of debris (i.e., grass, crops, building pieces) is being lofted by a tornado.

It's important for forecasters to remember that the TDS detection infers that there may or may not be an ongoing tornado, and that visual confirmation should be obtained. There are limitations to the signature which should be taken into account during the forecast process. The vegetation index and land cover parameters used in this study are also subject to change year to year. Therefore, forecasters should be aware that a change in vegetation and land cover will change the overall likelihood of observing a TDS for an ongoing tornado. Forecasters should also take into consideration tornado climatology for the United States and also realize that observing a TDS for an ongoing tornado only occurs about a quarter of the time (Van Den Broeke and Jauernic 2014). Although the likelihood of seeing a TDS given an ongoing tornado is high in the southeastern U.S. in summer, this likelihood number should be tempered by the fact that tornadoes very infrequently happen during that time of year for in the region. Lastly, the figures provided should not be used as a way of early detection or warning process, but as part of the verification process for confirming an ongoing tornado.

**Acknowledgements:** A sincere thank you is given to Dr. Daphne LaDue and the National Weather Center for making this research project and experience possible. The authors would also like to thank Steve Fletcher for installing modules on the computer machine used for performing analyses for this project.

This material is based upon work supported by the National Science Foundation under Grant No. AGS-1062932. Funding for Schuur and Lakshmanan was provided by NOAA/Office of Oceanic and Atmospheric Research under NOAA-OU Cooperative Agreement NA11OAR4320072, U.S. Department of Commerce.

## REFERENCES

Bodine, D., M. R. Kumjian, R. D. Palmer, P. L. Heinselman, A. V. Ryzhkov, 2012: Tornado damage estimation using polarimetric radar. *Wea. Forecasting.*, 28, 139-158, doi: 10.1175/WAF-D-11-00158.1.

Bunkers, M. J., and M. A. Baxter, 2011: Radar tornadic debris signatures on 27 April 2011. *Electronic J. Operational Meteor.* 12(7), 1-6.

Carey, L., C. Schultz, E. Schultz, W. Petersen, P. Gatlin, K. Knupp, A. Molthan, G. Jedlovec, B. Carcione, C. Darden, and C. Crowe, 2011: Dual-polarimetric radar-based tornado debris paths associated with EF-4 and EF-5 tornadoes over Northern Alabama during the historic outbreak of 27 April 2011. Preprints, 36th Nat. Wea. Assoc. Annual Meeting, Birmingham, AL, National Weather Association.

Center for International Earth Science Information Network (CIESIN), Columbia University; and Centro Internacional de Agricultura Tropical (CIAT). 2005. Gridded Population of the World Version 3 (GPWv3), cited 2014: Population Density Grids. Palisades, NY: Socioeconomic Data and Applications Center (SEDAC), CIESIN, Columbia University. [Available online at http://sedac.ciesin.columbia.edu/gpw]

Kellner, O., and D. Niyogi, 2014: Land surface heterogeneity signature in tornado climatology? An illustrative analysis over Indiana, 1950-2012. *Earth Interactions.*, 18 (10), 1-32, doi: 10.1175/2013EI000548.1.

Lakshmanan, V., T. Smith, G. Stumpf, K. Hondl, 2007: The Warning Decision Support System–Integrated Information. *Wea. Forecasting*, 22, 596-612.

Maddox, R. A., J. Zhang, J. J. Gourley, and K. W. Howard, 2002: Weather radar coverage over the contiguous United States. *Wea. Forecasting*, 17, 927-934.

NASA Earth Observatory Group (NEO), MODIS Land Science Team, cited 2014: Vegetation index [NDVI] (1 month – terra/MODIS) [Available online at http://neo.sci.gsfc.nasa.gov/view.php?datasetId=MOD13A2_M_NDVI]

National Weather Service, Louisville, KY, cited 2014: What is dual-polarization radar and what can it do for me? [Available at http://www.crh.noaa.gov/images/lmk/pdf/Dual_Pol_Overview.pdf]

Pielke, R. A., A. Pitman, D. Niyogi, R. Mahmood, C. McAlpine, F. Hossain, K. K. Goldewijk, U. Nair, R. Betts, S. Fall, M. Reichstein, P. Kabat, and N. Noblet, 2011: Land use/land cover changes and climate: modeling analysis and observational evidence. *WIREs Clim Change*, 2, 828-850, doi: 10.1002/wcc.144.

Ryzhkov, A. V., T. J. Schuur, D. W. Burgess, and D. S. Zrnic, 2005: Polarimetric tornado detection. *J. Appl. Meteor.*, 44, 557-570.

USGS, cited 2014: Map layer info: Conterminous United States land cover 1992 – 200 meter resolution. [Available online at http://nationalatlas.gov/mld/landcovi.html]

Schultz, C.J., L.D. Carey, E.V. Schultz, B.C. Carcione, C.B. Darden, C.C. Crowe, P.N. Gatlin, D.J. Nadler, W.A. Peterson, and K.R. Knupp, 2012a: Dual-polarization tornadic debris signatures part I: Examples and utility in an operational setting. *Electronic J. Operational Meteor.*, 13 (9), 120-137.

____, S.E. Nelson, L.D. Carey, L. Belanger, B.C. Carcione, T. Johnstone, A.L. Molthan, G.J. Jedlovec, E.V. Schultz, C.C. Crowe, and K.R. Knupp, 2012b: Dual-polarization tornadic debris signatures part II: comparisons and caveats. *Electronic J. Operational Meteor.*, 13 (10), 138-150.

Van Den Broeke, M. S., and S. T. Jauernic, 2014: Spatial and temporal characteristics of polarimetric debris signatures. *J. Appl. Meteor. Climatol.*, in press.

WDTB, cited 2014: Dual-polarization radar training for NWS partners: tornado debris signature. [Available online at http://www.wdtb.noaa.gov/courses/dualpol/outreach/index.html]

____, ROC, cited 2014: Dual-polarization pre-deployment operational assessment: Differential Reflectivity (ZDR). [Available online at http://www.erh.noaa.gov/rah/downloads/Dual_Pol/ZDR_v1.pdf]

---

## Reference / Citation

Handler, S. L., V. Lakshmanan, T. J. Schuur, and M. Van Den Broeke, 2014: Determining the likelihood of observing the tornado debris signature at different geographic locations throughout the United States. *Journal of Operational Meteorology* (submitted as "in press" per the paper's own masthead; original source is a National Weather Center Research Experience for Undergraduates (NWC REU) 2014 final paper).

- Original PDF (OU CAPS REU 2014 archive): https://caps.ou.edu/reu/reu14/finalpapers/Handler-finalpaper.pdf
- Project abstract page: https://caps.ou.edu/reu/reu14/finalpapers/Handler-finalpaper.html

No DOI could be located for this article; it appears to have circulated as an REU final paper / "in press" manuscript rather than through a formal DOI-indexed journal record. If a DOI-bearing published version exists in the *Journal of Operational Meteorology* (nwafiles.nwas.org / NWA), it was not found via web search at the time of this conversion.
