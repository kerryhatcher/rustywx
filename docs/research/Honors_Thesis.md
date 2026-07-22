# Correlation of Tornado Intensity with Dual-Polarization Radar Information

**Undergraduate Research Thesis**

Presented in Partial Fulfillment of the Requirements for graduation "with honors research distinction in Atmospheric Sciences" in the undergraduate colleges of The Ohio State University

by

**John Banghoff**

The Ohio State University
December 2015

Project Advisor: Professor Jay S. Hobgood, Department of Geography

---

## Table of Contents

| Section | Page |
|---|---|
| List of Tables | iii |
| List of Figures | iv |
| Abstract | 1 |
| I. Introduction | 2 |
| II. Literature Review | 4 |
| III. Methods | 8 |
| IV. Results | 12 |
| V. Discussion | 21 |
| VI. Acknowledgements | 25 |
| VII. References | 25 |

### List of Tables

| Table | | Page |
|---|---|---|
| 1 | Breakdown of tornadoes investigated by EF-Scale | 13 |
| 2 | TDS and rotational velocity criteria for tornado warning issuance | 22 |

### List of Figures

| Figure | | Page |
|---|---|---|
| 1 | El Reno, OK tornado debris signature on 31 May 2013 | 9 |
| 2 | Hattiesburg, MS (2/10/13) TDS demonstrating TDS height | 11 |
| 3 | Minimum correlation coefficient vs. maximum rated wind speed | 13 |
| 4 | Maximum TDS height vs. maximum rated wind speed | 15 |
| 5 | Maximum TDS diameter vs. tornado diameter | 17 |
| 6 | Moore, OK tornado on 20 May 2013 qualitative analysis | 19 |
| 7 | Time tendencies of TDS height and rotational velocity | 20 |

---

## ABSTRACT

Analysis of National Weather Service radar data associated with tornadoes documented by the Storm Prediction Center through May 2014 indicates that 332 of all tornadoes and 126 significant tornadoes exhibited a polarimetric tornado debris signature (TDS). This study documented confirmed TDS events associated with significant tornadoes (EF2 or greater) from May 2010 through May 2014 and characterized multiple characteristics throughout the life cycle of each tornado. Where available, wind speeds throughout the life of the tornado corresponding to the TDS at a given time were documented. A correlation was also determined between maximum wind speed of a tornado and maximum altitude of the TDS.

Results from this study documented a positive correlation between max rated wind speed and TDS height with an R² value of 0.6302. Some outliers from the predicted value were studied with distinct differences in land type noted. The strong correlation between tornado intensity and TDS height has led to a recent change in guidance and policy on how forecasters respond to tornado threats at the NWS Peachtree City WFO. Concurrently, an investigation of the life cycle of a TDS was conducted. It was noted that the diameter of a TDS is not a good indicator of the diameter of the tornado during its mature and dissipating phase. This knowledge is beneficial for forecasters as they communicate specific threats based only on verification from radar of a tornado on the ground.

---

## I. INTRODUCTION

In severe weather situations, individuals look to TV stations, NOAA weather radio, tornado sirens, and, more recently, portable electronic devices to receive relevant information. Weather information is more easily accessible than ever before and the specificity of the forecasts therein is unmatched in history. As individuals become more familiar with weather information, they develop confidence in their understanding and often take decision-making into their own hands. Because of the relative infrequency of severe weather in many locations, individuals think they will not be affected by severe weather. Psychological distance, defined as "a subjective experience that something is close or far away from the self, here, and now" (Trope et al. 2010), provides an explanation for this perceived immunity to severe weather. When severe weather hasn't affected individuals before, it is not perceived to be harmful and doesn't promote a response. The National Weather Service's service assessment of the Joplin, MO tornado (5/22/2011) found that after receiving a tornado warning, individuals looked for additional information to clarify the threat. Because of psychological distance, individuals consult phone apps or TV meteorologists to determine if the tornado will actually affect them. This research seeks to decrease the psychological distance of tornadoes by communicating an imminent threat and promoting a sufficient response using tornado intensity information observed in real-time.

Before discussing the technicalities of research methodology, a basic knowledge of the tools used to collect data is of utmost importance. The National Weather Service relies on over 100 Next-generation Radars (NEXRAD) to monitor current weather conditions and make decisions regarding issuance of watches and warnings. These radars are of the type WSR-88D which has several operational applications as outlined by Crum et al. (1993). As a supplement to the WSR-88D, dual-Polarization radar technology was first implemented on March 8, 2011 at Vance Air Force Base near Enid, OK. Since then, the National Weather Service has upgraded 151 operational radars which are now equipped with dual-polarization capabilities. Dual-pol radars are an improvement over the previous NEXRAD WSR-88D as they provide not only a horizontal scan of the atmosphere, but also a vertical component. This allows forecasters to both see that there is precipitation falling and determine its type based on the vertical span or orientation. Hail, for example, is much larger than a rain drop and the multi-dimensional picture as demonstrated by the radar allows forecasters to differentiate between the two in a thunderstorm.

Of particular benefit is the ability of dual-polarization radar products to identify the presence of debris in a tornado. The three dimensional picture of the objects in the air shed light on the scattered, inconsistent, and large objects lofted by a tornado. In the presence of a velocity couplet which is the traditional way forecasters identify a possible tornado, significant inconsistency throughout the layer and spherical orientation of objects differentiate debris from rain in a tornado-producing supercell. A tornado debris signature (TDS) is demonstrated by the presence of particular characteristics of dual-polarization products which are collocated. These characteristics are discussed at length below.

The presence of a TDS serves as verification of a tornado on the ground with as much weight as a storm spotter calling in a report to the National Weather Service. When the radar shows debris being lofted, there is a confirmed tornado on the ground. One of the first verifications of tornadoes by a National Weather Service Forecast Office occurred in Jackson, MS on February 10, 2013 – shortly after the Jackson radar had dual-pol technology installed. An EF-4 tornado ripped through Hattiesburg, MS but no lives were lost. Forecasters utilized the presence of a TDS as verification for a tornado on the ground and used "CONFIRMED TORNADO" instead of "DOPPLER RADAR INDICATED TORNADO" in their warning text in order to convey their message more effectively.

In what follows, a review of previous work regarding TDS applications and implications will lay the framework for a discussion of research methodology and results. While TDSs have been used to verify the presence of a tornado, it was hypothesized that TDS characteristics could also give an indication of tornado intensity. The height of lofted debris, degree of uniformity of debris within the tornado, and horizontal expanse of debris were compared to the estimated maximum wind speed of each tornado. It was hypothesized that the height of lofted debris would be a good indicator of tornado intensity and could be used by forecasters to convey information about the tornado strength in warning text to improve response and decrease psychological distance. A discussion of results and an analysis of operational significance are followed by suggestions for continued investigation of the relationship between tornado impacts and TDS characteristics.

---

## II. LITERATURE REVIEW

Previous research regarding tornado debris signatures is limited in scope compared to many other facets of weather forecasting (e.g. storm structure, winter weather forecasting, precipitation type determination, etc.) but the potential for life saving findings is very high. Research linking tornado debris signature characteristics to tornado intensity has been on the rise seeking to provide real-time tornado information to forecasters which can be translated to warning text, communications with emergency management, and increased preparedness for the general public. A tornado debris signature has been defined based on four dual-polarization radar products: horizontal radar reflectivity (Z<sub>HH</sub>), storm-relative or base velocity (V), correlation coefficient (ρ<sub>hv</sub>), and differential reflectivity (Z<sub>DR</sub>) (Ryzhkov et al. 2005, Schultz et al. 2012, Entremont 2013, Bodine et al. 2013).

Historically, tornado warning issuance has been dictated by environmental factors, radar monitoring, and recognition of a tornado vortex signature (TVS). Brown et al. (1978) defined TVSs as "signatures with extreme Doppler velocity values of opposite sign". Horizontal radar reflectivity is an indication of how many scatterers are in the air at a particular height. Higher values indicate a high amount of energy reflected off of objects and returned to the radar while lower values indicate less material in the atmosphere. The term "debris signature" has been used numerous times by broadcasters and researchers alike to identify the ball-like appearance on a horizontal radar reflectivity scan when a large tornado is on the ground (Bunkers and Baxter 2011, Forbes 2011). The collocation of a TVS and horizontal radar reflectivity is necessary for the presence of a TDS because if debris is lofted, it will return high levels of energy to the radar. Before the introduction of dual-polarization radar beginning in 2011, operational forecasters could only rely on base velocity and horizontal radar reflectivity to indicate the presence of a tornado and identify potential debris.

Dual-polarization radar provides horizontal and vertical scans of the atmosphere giving a three-dimensional picture of atmospheric behavior. Dual-pol products have been used for numerous applications including differentiating hydrometeor types and meteorological vs. non-meteorological scatterers (Zrnic and Ryzhkov 1999; Vivekanandan et al. 1999). As noted above, the two dual-pol products used to identify a TDS are correlation coefficient (ρ<sub>hv</sub>) and differential reflectivity (Z<sub>DR</sub>). The NWS Warning Decision Training Branch Dual-Polarization Radar Training defines correlation coefficient as the "measure of how similarly the horizontally and vertically polarized pulses are behaving within a pulse volume" (WDTB). Put another way, it measures the homogeneity or heterogeneity of a column of air. A rain band or any hydrometeors will have a ρ<sub>hv</sub> of 1 because they are homogeneous but non-meteorological scatterers such as bats, birds, debris, or insects will have a much lower correlation coefficient value because the column of air is more heterogeneous (Balakrishnan and Zrnic 1990). Differential reflectivity is defined as the "difference between the horizontal and vertical reflectivity factors" (WDTB). This product gives an indication of the overall shape of the scatterers being detected by radar. Spherical particles are assumed to have a Z<sub>DR</sub> near zero because the horizontal reflectivity will be equal to the vertical reflectivity. Ryzhkov notes that randomly oriented scatterers, such as debris, have a differential reflectivity near zero but if they possess some sort of common orientation, Z<sub>DR</sub> may be slightly positive or negative based on their orientation (Ryzhkov et al. 2005).

The first tornado debris signature was identified using dual-polarization radar on 3 May 1999 by the Cimarron polarimetric radar at the National Severe Storms Laboratory in Norman, OK (Ryzhkov et al 2005). Ryzhkov notes that "after tornado touchdown, the signature at the tip of the hook echo for that storm was identified by a Z<sub>DR</sub> that was close to zero and anomalously low values of ρ<sub>hv</sub> (less than 0.5)." Since then, tornado debris signatures have been identified by a valid velocity couplet, high horizontal reflectivity, a lowering of correlation coefficient, and differential reflectivity values near zero. It has been noted that ρ<sub>hv</sub> is better than Z<sub>DR</sub> for determining the presence of a TDS (Ryzhkov et al. 2005, Bluestein et al. 2007 and Kumjian and Ryzhkov 2008). Threshold values for each product vary based on sample size, personal preference, and previous research being modeled after, but the thresholds used in this paper are outlined in the next section.

Tornado debris signatures have been discussed as a tool for providing real-time weather information by numerous studies (Ryzhkov et al. 2005, Schultz Pt I. 2012, Bodine et al. 2012, Entremont 2013, Bodine 3013, and Van den Broeke 2014). The ability of a TDS to identify tornadoes has proven difficult for weaker tornadoes (EF0-EF1) because of their weaker wind speeds, shorter duration, and inability to loft debris (Ryzhkov et al. 2005). As a result of the lower likelihood of producing a TDS and the large number of weak tornado cases, very little research has been done with TDS behavior for weak tornadoes. Schultz (2012) outlines a few case studies of weaker TDS cases and many other case studies have been completed, but large scale analyses such as the one we have undertaken omits most of the weak tornadoes. After it became clear that tornado debris signatures could in fact be a good indicator of the presence of a tornado, research began in an effort to relate tornado debris signature characteristics to tornado characteristics. Schultz noted in his paper in 2012 that TDS diameter increased slightly with intensity but there was no significant relationship. Bodine et al. 2012 noted the lack of research related to using TDS characteristics to estimate real-time tornado damage severity.

An indication of tornado information in real-time can help prevent missed tornado warnings, provide precise locations of a tornado and indicate the presence of debris. The bulk of the research this study was modeled after was taken on by Entremont et al. and presented in 2013. The research conducted by Entremont et al. sought to relate tornado intensity based on the Enhanced Fujita scale to tornado debris signature characteristics. Using a data set of 140 tornadoes from May 2010 – July 2013, TDS heights were determined for each tornado based on the highest vertical extent of lowered correlation coefficient values. Stratification by EF-scale found good correlation (R=0.79) between TDS height and tornado intensity. Entremont noted the critical height of 10,000 feet (10 kft) which indicates a strong tornado (EF2+) and the most useful case being a strong/violent tornado at a greater distance from the radar to allow TDS height differentiation. This correlation was implemented in warning operations at the Jackson, MS NWS office and thresholds were created to dictate issuance of tornado warnings or tornado emergencies. In addition, the information regarding tornado intensity was used to give emergency managers an estimate of potential impacts so they could prepare accordingly. Ultimately, Entremont's research provided the foundation of our data and allowed further analyses of tornadoes in an effort to improve real-time tornado warning communications.

The research outlined below seeks to fill some of the gaps and investigates some of the questions raised by previous studies. Of particular interest is the possibility of correlating wind speed with TDS variables as opposed to just sorting by EF-rating. By using actual wind estimates as determined by NWS damage surveys, a linear relationship can be determined instead of sorting tornado intensity into bins. Investigation of debris height (TDS height), degree of uniformity of debris within the tornado (CC minimum) and horizontal expanse of debris (TDS width) were completed for all EF2+ tornadoes. In addition, Bodine et al. (2012) mentions the spatio-temporal relationship between a TDS and the tornado lifecycle. While not intentionally researched, Bodine's observation that debris fallout leads to complication during tornado dissipation leading to a spatial spread of debris is investigated.

---

## III. METHODS

A previous dataset of tornado information from Entremont and Schultz covered the span from 10 May 2010 to 31 July 2013. Additional research conducted at the NWS in Peachtree City, GA began with tornadoes from 1 August 2013 and continued through 31 May 2014. Identification of tornadic storms relied on access of StormData from the Storm Prediction Center. Tornadoes of scale EF2 or higher were selected in order to scale down the expansive data set into higher probability TDS occurrence given time constraints. Starting latitude and longitude coordinates for each tornado were plotted as a .kmz shapefile in Google Earth along with a shapefile of NEXRAD WSR-88D dual-pol enabled radomes to identify the closest radar location. In each event, the two closest radomes and associated distances were recorded. Any tornado which was more than 100 miles away from the closest radome was thrown out. At a distance of 100nm, the lowest radar scan in most cases is over 10 kft. Weaker tornadoes are missed and stronger tornadoes are misrepresented as the radar can't see the majority of the tornado which is below 10 kft.

After identification of closest radar location, NEXRAD Level II radar data was downloaded from the National Climatic Data Center's Hierarchical Data Storage System (HDSS) Access System (HAS). Based on the start time that was listed in the downloaded StormData and the path length of the storm, the proper number of 1 hour time intervals of data was ordered for each event. This data was then viewed using Gibson Ridge (GR) Level 2 Analyst software on a scan by scan (time interval) and tilt by tilt (varying degrees within each scan) basis.

A shape file was created for each storm in the database which included the approximate path of the tornado based on starting and ending latitude and longitude coordinates. This shapefile was imported into GR2Analyst and used to assist in identifying tornadoes. For each tornado, the presence of a TDS had to be accounted for. The four panel mode for GR2Analyst was used with base reflectivity (BR), base velocity (BV), correlation coefficient (CC), and differential reflectivity (ZDR) in each of the four respective panels. In each tornado, the criteria for presence of a TDS used were as follows and are shown in Figure 1:

1. Identify a valid velocity circulation (V).
2. Low correlation coefficient (CC <.85) **collocated** with circulation.
3. Sufficient reflectivity (Z>35 dbZ) **collocated** with # 1 & 2.
4. Lowering of differential reflectivity (ZDR) near 0.

![Four-panel radar display (Z, V, ZDR, CC) of the EF-3 tornado that hit El Reno, OK on 31 May 2013. Reflectivity (Z, top-left) shows a hook echo; base velocity (V, top-right) shows a valid couplet; differential reflectivity (ZDR, bottom-left) shows values near zero collocated with the couplet; correlation coefficient (CC, bottom-right) shows a lowered, ball-like signature collocated with the velocity couplet — the tornado debris signature.](media/Honors_Thesis/fig1_el_reno.png)

**Figure 1**: EF-3 tornado that hit El Reno, OK on 31 May 2013

Once a TDS was identified, TDS-specific data as well as tornado-specific data were recorded. Each scan included record of TDS diameter, TDS width, minimum Correlation Coefficient (CC) and the height at which it occurred, latitude and longitude of the center-most point of the TDS, rotational velocity (the difference between outbound and inbound velocities at the tornado vortex), etc. In addition, damage assessment surveys for each tornado were accessed either through the Damage Assessment Toolkit (DAT) or online using public information statements (PNSs) from National Weather Service offices and maximum rated wind speeds were recorded.

The three main comparisons involved in this study were TDS diameter vs. tornado diameter as defined by the damage survey, minimum correlation coefficient vs. max rated wind speed, and maximum TDS diameter vs. max rated wind speed. To determine the TDS diameter, the "measure" tool was used in GR2Analyst to measure along radial (perpendicular to the radar) from one side of the TDS to the other using CC values less than 0.85. Minimum correlation coefficient values were determined by dragging the mouse over each pixel to determine which had the lowest value within the TDS. Finally, TDS height was determined by starting with the lowest tilt and advancing to higher tilts in succession until it was determined that a TDS was no longer present. As shown in Figure 2, the lowest tilt (0.5°) was investigated to determine the presence of a TDS based on minimum CC value. Each vertical tilt for a given scan was observed until the minimum CC no longer fell below the threshold. At the centroid of the TDS at the highest tilt which met the minimum CC value, the height was recorded and designated at the TDS height. The maximum TDS height for each tornado was determined to be the highest TDS height across all scans.

An example analysis is shown in Figure 2 for the Hattiesburg, MS EF-4 which occurred on 10 February 2013. The lowest tilt is 0.5° and a TDS is evidenced by high reflectivity, a valid velocity couplet, lowering of ZDR, and, most importantly, a CC min of 0.64. The CC min is below the threshold of 0.85 as laid out in the methodology section. The next tilt is 0.9° and a TDS is again present with a minimum CC value of 0.75. Moving to the next highest tilt, 1.4°, the minimum CC is 0.79. Next, the 1.9° tilt is observed to have a CC min of 0.94 which is greater than the threshold. Therefore, the TDS height is 3,090 ft for this particular scan. This process is repeated for each scan throughout the tornado's life cycle and the maximum TDS height is recorded as the highest TDS height across all scans.

![Four tilts (0.5°, 0.9°, 1.4°, 1.9°) of the Hattiesburg, MS TDS from 10 February 2013, each shown as a four-panel (Z, V, ZDR, CC) radar display with the TDS circled. Minimum CC and corresponding height are annotated per tilt: 0.64 @ 1200 ft, 0.75 @ 2020 ft, 0.79 @ 3090 ft, 0.94 @ 4060 ft (above threshold, so TDS height for this scan is 3,090 ft).](media/Honors_Thesis/fig2_hattiesburg.png)

**Figure 2**: Determination of maximum TDS height using GR2Analyst data. (Images courtesy Chad Entremont, NWS JAN)

---

## IV. RESULTS

A breakdown by intensity on the Enhanced Fujita scale of the 105 tornadoes which exhibited tornado debris signatures and had a complete data set are listed in Figure 3. A database of 142 tornadoes was inherited from Chris Schultz (University of Alabama-Huntsville) and Chad Entremont (NWS Jackson) spanning the period between 10 May 2010 and 29 July 2013. The research added since then spans from 4 August 2013 to 31 May 2014. 175 tornadoes from this combined list had a complete data set. For the sake of simplification and time constraints, EF0 and EF1 cases were omitted from this research bringing the total to 105 tornadoes. This number will increase as more tornadoes with a TDS occur and the data can be added to the database.

Table 1 shows a breakdown of tornado cases by EF-scale which were analyzed in this study and those which had a complete data set and were used for data analysis.

**Table 1: Breakdown of tornadoes investigated by EF-Scale**

| | EF-2 | EF-3 | EF-4 | EF-5 | TOTAL |
|---|---|---|---|---|---|
| OBSERVED | 95 | 42 | 15 | 2 | 154 |
| FINAL | 59 | 32 | 12 | 2 | 105 |

The first hypothesis investigated the relationship between max rated wind speed and minimum correlation coefficient value. Because lower correlation coefficient values correspond to less consistency throughout the scanned layer, debris is represented by a lower correlation coefficient. It was hypothesized that minimum CC value would not correspond well with tornado intensity. The results are show below in Figure 3.

![Scatter plot titled "Significant Tornado Intensity Estimated by Minimum CC Value, May 2010 – May 2014, N=105." X-axis: Correlation Coefficient (0 to 1). Y-axis: Max Rated Wind Speed (mph, 0 to 250). Points cluster loosely with a weak downward trend line: y = -46.854x + 155.52, R² = 0.1591, p = 1.40E-5. Data shows a vertical asymptote near CC = 0.21.](media/Honors_Thesis/fig3page-13.png)

**Figure 3**: Correlation of minimum correlation coefficient (CC) with max rated wind speed.

With an R² value of 0.1591 and a p value of 1.40E-5 (p<0.05), this relationship is not strong and we can reject the null hypothesis in favor of the alternative hypothesis. In addition, there are a few important takeaways. First, there is a vertical asymptote corresponding with a correlation coefficient of 0.21. This exists because the GR2Analyst software does not have the capability of representing a lower value. This certainly may lead to skewed data when seeking to represent the correlation as purely linear. In addition, the negative correlation between wind speed and correlation is notable. It makes sense that lower correlation coefficient values would represent less organization and more inconsistency throughout the layer. This, one would suppose, is debris lofted by an increasingly large tornado as the inconsistency in the layer increases. While this relationship did not prove useful, some of the results were notable.

The second hypothesis investigated the relationship between maximum TDS height and max rated wind speed. The definition of maximum TDS height is outlined in the methodology section. It was hypothesized that maximum TDS height would correspond with maximum rated wind speed because stronger winds lead to more debris and stronger updrafts in the storm. The results from the second hypothesis are below in figure 4.

![Scatter plot titled "Significant Tornado Intensity Estimated by TDS Height, May 2010 - May 2014, N=105." X-axis: Max TDS Height (kft, 0 to 50). Y-axis: Max Rated Wind Speed (mph, 0 to 250). Points show a clear positive linear trend: y = 2.10x + 108.42, R² = 0.63, p = 5.17E-25. One notable outlier appears in the upper-left quadrant (low TDS height, high wind speed) corresponding to the Granbury, TX EF4 tornado.](media/Honors_Thesis/fig4page-15.png)

**Figure 4**: Correlation of maximum TDS height with max rated wind speed.

The results of this particular relationship are compelling. An R² value of 0.63 is a good correlation and this is significant with a p value of 5.17E-25 (p<0.05). Based on the data we can again make a few interesting observations. First, there are a notable lack of data points with maximum TDS heights above 30 kft. As research continues, supplementing the data set with higher end events will be helpful. Of particular note is the outlier in the upper left quadrant of the graph corresponding to an EF4 tornado which touched down in Granbury, TX. Upon further investigation, it was determined that the tornado touched down near, and likely crossed over, Lake Granbury. It was hypothesized that the lake had less debris for the tornado to lift, thus stunting TDS height growth. This case encourages future investigation of land use characteristics and its effect on TDS height. Lastly, this line of best fit suggests a tornado which exhibits a TDS height of 20 kft has wind speeds of approximately 150mph (a high end EF-3). Information estimated tornado intensity based on radar information has not been reliable in the past due to height differences in base velocity scans and inconsistent data. While this line of best fit is by no means infallible, it provides valuable "additional information" sought by forecasters and the general public in the presence of adverse weather conditions.

The final hypothesis from this research sought to compare the diameter of a TDS to the actual width of the tornado on the ground. It was hypothesized that this relationship would not be strong because of the vertical separation between the tornado on the ground and the lowest scan of the tornado debris signature. In addition, the cone-like structure of a strong tornado leads to wider measurements at higher elevations. Nonetheless, results were compelling. This relationship was determined first by pure data and then by a particular case which sheds light on the correspondence between the tornado life cycle and TDS.

A pure data analysis was employed comparing the maximum diameter of a TDS found throughout each scan and tilt of a tornado's life cycle to the tornado width as determined by the damage survey. Results from the 56 cases for which both measurements were available is below in Figure 5. Note: many of the damage surveys did not indicate estimated tornado widths.

![Scatter plot titled "Significant Tornado Width Estimated by TDS Diameter (nm), May 2010 - May 2014, N=56." X-axis: TDS Diameter (nm, 0 to 5). Y-axis: Tornado Width (nm, 0 to 2.5). Points widely scattered with weak positive trend: y = 0.193x - 0.02, R² = 0.28, p = 2.76E-5. Data highly variable especially at wider TDS diameters — one point near TDS diameter 4nm has tornado width 2.25 (El Reno), another near TDS diameter 5nm has tornado width only 0.10.](media/Honors_Thesis/fig5page-17.png)

**Figure 5**: Correlation of TDS diameter with tornado width. Note: Cases include Moore, OK EF-5 20 May 2013, El Reno, OK EF-3 on 31 May 2013, and 08/2013-05/2014 from most recent TDS analyses.

The results from this analysis show a low correlation (R²=0.28) which is statistically significant (p=2.76E-5 for p<0.05). Of particular note is that the relationship between TDS diameter and tornado width is widely variable, especially as a TDS gets wider. For example, a TDS diameter of just over 4 nm had a tornado width of 2.25 miles (El Reno, OK 30 May 2013) while a TDS diameter of almost 5nm was only 0.10 miles wide.

To supplement the quantitative data set demonstrated above, a qualitative case study was performed on the EF-5 Moore, OK tornado from 20 May 2013. The qualitative analysis is shown in Figure 6. This particular storm was chosen because of it's a) vicinity to the radar, b) size and c) visibility based on TV station helicopter footage throughout its lifecycle. This tornado has three different stages of its lifecycle which are referred to as tornado-genesis (formation), tornado maturity, and tornado dissipation. In the first stage of the tornado's life cycle, the tornado has just touched down and is very narrow. The debris signature is slightly visible within the white circle on the correlation coefficient image. At this point, the tornado width and TDS width seem to correspond well. At tornado maturity, the tornado on the ground is massive and the debris signature seems equally as expansive. Further inquiry finds that the tornado is 1.3 miles wide while the TDS is 2.8 miles wide. Therefore, correlation clearly no longer holds true. This part of the study did not account for height variation of the lowest radar scan. Though this may exhibit slightly more correlation, the final stage of the process makes this process impractical. Tornado dissipation at 20:57 UTC shows a tornado no longer in existence and roping out. The TDS however, is now expansive and dissipated as well. At this phase, the TDS diameter is not at all indicative of the tornado on the ground. In summary, the TDS diameter is not a good indicator of tornado width, specifically in the mature and dissipation stages of the tornado's life. For most tornadoes, distinguishing between stages in a live event situation is not possible or practical.

![Three-row comparison for the Moore, OK tornado of 20 May 2013. Left column (a-c): live TV helicopter shots from News9 in Oklahoma City. Right column (d-f): correlation-coefficient radar imagery with the TDS circled. Row 1, 19:57 UTC, "tornado-genesis": narrow funnel visible on camera (a); small TDS circled on radar (d). Row 2, 20:13 UTC, "tornado maturity": massive tornado on camera with a 1.3-mile-wide measurement overlay (b); correspondingly large TDS on radar measuring 2.8 miles across (e). Row 3, 20:57 UTC, "tornado dissipation": tornado no longer visible, roping out/gone (c); TDS on radar still expansive and diffuse (f), no longer indicative of the (now-dissipated) tornado.](media/Honors_Thesis/fig6page-19.png)

**Figure 6**: a)-(c) live shots from News9 in Oklahoma City (d)-(f) CC indicating TDS diameter.

*TDS width doesn't represent tornado width in maturity or dissipation stages.*

One additional aside to the hypotheses listed above involved a time tendency of TDS height and rotational velocity which illustrates the above findings in a different manner. As can be seen in Figure 7 relating volume scan after the start of the tornado and TDS height, there was a slight lag in TDS height increase as a tornado developed. This seems to make sense given that as a tornado increased in strength it had more debris to pick up and the strength of the updraft became larger. The graph on the right shows a box and whisker plot of rotational velocity which gives an indication of how fast the column of air was rotating. The important thing to note with this graph is the significant decrease in rotational velocity after the 3rd and 4th scans of the tornado's life. While the overall velocity of the tornado was decreasing, the TDS height as demonstrated on the right continued to rise.

![Two charts side by side. Left: line chart "Average TDS Height per scan" — X-axis: Volume Scan (0 to 12), Y-axis: TDS Height (kft, 0 to 18). Line rises steadily from about 7.5 kft at scan 0 to roughly 16 kft by scan 9, then levels off/slightly declines through scan 12. Right: box-and-whisker chart "Rotational Velocity per scan" — X-axis: Volume Scan (0 to 3), Y-axis: Rotational Velocity (kts, 0 to 120). Boxes show a clear decrease in median and spread of rotational velocity from scan 0 through scan 3, even as TDS height (left chart) continues climbing over the same scans.](media/Honors_Thesis/fig7page-20.png)

**Figure 7**: Time tendency of TDS height and rotational velocity used to demonstrate the spatio-temporal relationship between a tornado and the associated TDS.

This same phenomenon was illustrated in the tornado dissipation phase of the Moore tornado case study as the tornado was gone but debris was still being lofted. This lag time in which debris is still present long after the tornado has lifted is quite common. The debris continues to be lofted by the thunderstorm updraft until the updraft weakens downstream of the tornado. This also explains why it is important to look at more than just correlation coefficient when determining whether or not a TDS is present. In the later scans following a tornado, there is still an area of low CC values, but a forecaster will know that a tornado is no longer occurring if they look at rotational velocity values.

---

## V. DISCUSSION

Since the advent of dual-polarization radar in 2010, research has been evolving to make the wealth of data more accessible to forecasters. This research seeks to take the benefits of dual-polarization radar one more step and move the information from the forecasters to the general public in an operationally focused outcome. This research has shown that tornado intensity can be reasonably estimated by tornado debris signature height, but not by minimum correlation coefficient value. In addition, tornado debris signature width is not a good indicator of tornado width.

One of the setbacks of tornado debris signatures is their inability to provide any lead time for warning issuance. In other words, debris can be lifted by a tornado and thus create a tornado debris signature only after a tornado has touched down. As result, it does not help forecasters give any advanced notice to the affected area of the possibility of a tornado. Nonetheless, the presence of a TDS can persuade forecasters to issue a tornado warning if they are unsure, serve as verification for a warning which has been issued, or provide guidance for issuing an additional warning in the case of a long track tornado. Each of these helps forecasters provide more accurate and timely information than that which was available before dual-polarization radar.

As a result of the research findings for this project, forecasters at the National Weather Service in Peachtree City, GA use TDS height alongside rotational velocity and normalized rotation (NROT) when considering whether or not to issue tornado warnings and emergencies. The thresholds outlined below indicate a tornado with wind speeds of up to 150mph requiring a tornado warning and anything above that (high end EF3 or greater) to consider or issue a tornado emergency.

Table 2 demonstrates the TDS height and rotational velocity criteria used for decision making at the National Weather Service in Peachtree City, GA.

**Table 2: TDS and rotational velocity criteria for tornado warning issuance**

| | Tornado Warning — Enhanced Wording | Tornado Emergency — Consider | Tornado Emergency — Strongly Consider |
|---|---|---|---|
| Rotational Velocity | 50-80 kts | 70-80 kts | ≥ 80 kts |
| TDS | 8-20 kft | 15-20 kft | ≥ 20 kft |

While current warning procedure at National Weather Service offices in the Southeast (Peachtree City, GA, Jackson, MS, and Huntsville, AL) relies on these TDS height thresholds to determine whether to issue a tornado warning or a tornado emergency, the hope is that warnings can eventually include estimates of wind speed for a particular tornado. The warning text shown in Figure 8 was issued for the Haralson-Paulding County EF3 tornado which occurred on March 2, 2012, the text includes the phrase "Doppler radar confirmed a tornado." The suggested first step for warning development is the inclusion of wind speeds in warning text. For example, the warning would say, "...CONFIRMED A TORNADO WITH ESTIMATED WIND SPEEDS OF ###-###MPH." This modification will provide additional context for threat severity.

After wind speeds are successfully added to warning text, the next step would be to make potential impacts clear to individuals. The information deficit model says that individuals don't act the way they are supposed to because of a lack of knowledge and an increase in knowledge will result in the desired behavioral change (Marteau 1998). This model has been adopted by meteorologists and scientists in many different fields but lacks empirical or theoretical support. Whether warnings include "Doppler radar indicated", "confirmed", or "wind speeds of", it doesn't seem that more information is always the answer. The best option over the long term will be using damage survey procedures in reverse to take EF-scale as determined by the TDS analysis and predict likely damage. While the general public will not necessarily understand wind speeds, it seems individuals will respond if structural damage is explicitly stated. For example, the warning would say "…A TORNADO CAPABLE OF CAUSING SIGNIFICANT STRUCTURAL DAMAGE TO WELL-BUILT HOMES" or "…A TORNADO CAPABLE OF TWISTING TREES AND CAUSING MINOR STRCTURAL DAMAGE TO MOBILE HOMES AND POLE BARNS."

```
BULLETIN - EAS ACTIVATION REQUESTED
TORNADO WARNING
NATIONAL WEATHER SERVICE PEACHTREE CITY GA
814 PM EST FRI MAR 2 2012

THE NATIONAL WEATHER SERVICE IN PEACHTREE CITY HAS ISSUED A

* TORNADO WARNING FOR...
  NORTHERN HARALSON COUNTY IN NORTHWEST GEORGIA
  NORTHWESTERN PAULDING COUNTY IN NORTHWEST GEORGIA
  SOUTHERN POLK COUNTY IN NORTHWEST GEORGIA

* UNTIL 900 PM EST

* AT 809 PM EST...DOPPLER RADAR CONFIRMED A TORNADO 17 MILES
SOUTHWEST OF ROCKMART...MOVING EAST AT 40 MPH.
```

**Figure 8**: Example warning issued for a tornado which produced a TDS on 2 March 2012.

One of the main setbacks for this research relates to the subjectivity of damage surveys. In each tornado case observed, the main relationship involves the maximum wind speed as determined by the meteorologist who observed the damage caused by a tornado. The process of conducting damage surveys is standardized in order to facilitate consistency among offices, but determining the estimated wind speeds from the tornado is an inexact science. For example, a tornado that touches down in the middle of an open field may have winds in excess of 150 mph but because there is no debris in its path, it will be difficult to determine an accurate rating for the tornado. On the contrary, a tornado may have winds of 100 mph but may strike structures that are poorly constructed making the damage seem more severe and thus leading to a higher estimated wind speed. The subjective nature of the data used in this study may lead to error which is hard to quantify but is important to acknowledge.

The future of TDS research is bright and largely unexplored. There is a wealth of information both that has been collected in this study and that is available for analysis with the implementation of dual-polarization radar. As outbreaks continue to occur such as that of April 29th, 2015, more tornado debris signatures will be observed and can be analyzed to either bolster or modify the above results. Methodology for the bulk of the remaining research will be the same.

An aside for future research will involve using ArcGIS to overlay tornado information with land use and vegetation characteristics. Scaling the results of TDS height based on the amount and type of debris available to be lofted will improve results. In addition, weak tornadoes rated EF0 or EF1 will be observed to determine if there is a more universal relationship for all tornadoes. For many NWS offices, EF2+ cases are extremely rare and QLCS type tornadoes are more common. In order to improve the operational significance of the research, TDS characteristics need to be observed for all wind speeds and tornado intensities.

---

## VI. ACKNOWLEDGEMENTS

This work was supported by the Ernest F. Hollings Scholarship Program which is offered by the National Oceanic and Atmospheric Administration (NOAA). This work was made possible through a summer internship at the National Weather Service Weather Forecast Office in Peachtree City, GA. Additional funding was offered by the Pressey Endowment Fund from the College of Arts and Sciences at The Ohio State University. Portions of this thesis were presented at the 2014 NWA Annual Conference in Salt Lake City, UT and at the 2015 AMS Annual Meeting in Phoenix, AZ. The author would like to thank Science Operations Officer Steve Nelson from NWS Peachtree City for his countless hours of support, guidance, and expertise on this topic. In addition, special thanks to Dr. Jay S. Hobgood for his guidance throughout the authorship and defense of this thesis.

---

## VII. REFERENCES

Balakrishnan N., and D. S. Zrnic, 1990: Use of polarization to characterize precipitation and discriminate large hail. *J. Atmos. Sci.*, 47, 1525-1540.

Bluestein, H. B., M. M. French, R. L. Tanamachi, S. Frasier, K. Hardwick, F. Junyent, and A. L. Pazmany, 2007: Close-range observations of tornadoes in supercells made with a dual-polarization, X-band, mobile Doppler radar. *Mon. Wea. Rev.*, 135, 1522–1543. doi: 10.1175/MWR3349.1

Bunkers, M. J., and M. A. Baxter, 2011: Radar tornadic debris signatures on 27 April 2011. *Electron. J. Oper. Meteor.*, 12 (7), 1–6.

Crum, T. D., & Alberty, R. L. (1993). The WSR-88D and the WSR-88D operational support facility. *Bulletin of the American Meteorological Society*, 74(9), 1669-1687.

Entremont, C., 2013: Relationship Between Tornado Debris Signature (TDS) Height and Tornado Intensity. Southeast Severe Storms Symposium, Mississippi State University.

Forbes, G., 2011: Terrible 2011 tornadoes. 36th Annual NWA Conference, Birmingham, AL, Natl. Wea. Assoc.

Kumjian, M. R., and A. V. Ryzhkov, 2008: Polarimetric signatures in supercell thunderstorms. *J. Appl. Meteor. Climatol.*, 47, 1940–1961. doi: 10.1175/2007JAMC1874.1

Marteau, T. M., Sowden, A. M. A. N. D. A. J., & Armstrong, D. (1998). Implementing research findings into practice: beyond the information deficit model. *Getting research findings into practice*, 36-42.

Ryzhkov, A., T. J. Schuur, D. W. Burgess, and D. S. Zrnić, 2005: Polarimetric tornado detection. *J. Appl. Meteor.*, 44, 557–570. doi: 10.1175/JAM2235.1

Schultz et al., 2012: Dual-Polarization Tornadic Debris Signatures Part I: Examples and Utility in an Operational Setting. *Electronic J. Operational Meteor.*, 13 (9), 120−137.

Schultz et al., 2012: Dual-Polarization Tornadic Debris Signatures Part II: Comparisons and caveats. *Electronic J. Operational Meteor.*, 13 (10), 138−150.

Trope, Y., & Liberman, N. (2010). Construal-level theory of psychological distance. *Psychological Review*, 117(2), 440-463. doi:10.1037/a0018963

Van Den Broeke, Matthew S., and Jauernic, Sabrina T., 2014: Spatial and Temporal Characteristics of Polarimetric Tornadic Debris Signatures. *J. Appl. Meteor. Climatol.*, 53, 2217–2231.

Vivekanandan, J., D. S. Zrnic, S. M. Ellis, D. Oye, A. V. Ryzhkov, and J. Straka, 1999: Cloud microphysics retrieval using S-band dual-polarization radar measurements. *Bull. Amer. Meteor. Soc.*, 80, 381–388.

Warning Decision Training Branch, 2011: Dual-pol Training. [http://www.wdtb.noaa.gov/courses/dualpol/index.html]

Zrnic, D. S., and A. V. Ryzhkov, 1998: Observations of insects and birds with a polarimetric radar. *IEEE Trans. Geosci. Remote Sens.*, 36, 661–668.

---

## Reference / Citation

Banghoff, John (2015). *Correlation of Tornado Intensity with Dual-Polarization Radar Information* (Undergraduate honors research thesis). The Ohio State University, Department of Geography, Atmospheric Sciences Program. Advisor: Jay S. Hobgood. Supported by the NOAA Ernest F. Hollings Scholarship Program; research conducted at the NWS Weather Forecast Office, Peachtree City, GA.

ResearchGate record (source of this PDF): https://www.researchgate.net/publication/301749776_Correlation_of_Tornado_Intensity_with_Dual-Polarization_Radar_Information
