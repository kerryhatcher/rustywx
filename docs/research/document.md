# Overview of a Decade of Field Experiments with the Atmospheric Imaging Radar

**David J. Bodine\*, James M. Kurdzo†, Casey B. Griffin‡, Robert D. Palmer\*§, Bradley Isom¶, Feng Nai‖, Andrew Mahre†\*\*, Mark Yeary\*‡‡, Tian-You Yu\*‡‡**

\*Advanced Radar Research Center, University of Oklahoma, Norman, OK
†MIT Lincoln Laboratory, Lexington, MA
‡College at Brockport, State University of New York, Brockport, NY
§School of Meteorology, University of Oklahoma, Norman, OK
¶StormQuant, Inc., Titusville, FL
‖Cooperative Institute for Severe and High-Impact Weather Research and Operations, University of Oklahoma, Norman, OK
\*\*Applied Physics Laboratory, Johns Hopkins University, Laurel, MD
‡‡School of Electrical and Computer Engineering, University of Oklahoma, Norman, OK

2022 IEEE Radar Conference (RadarConf22). 978-1-7281-5368-1/22/$31.00 ©2022 IEEE. DOI: 10.1109/RadarConf2248738.2022.9764270

---

## Abstract

Phased array radar (PAR) systems are widely considered the future of atmospheric science radar technology because they offer much faster and more versatile scanning than traditional, parabolic dish radars. The Atmospheric Imaging Radar (AIR) is a mobile PAR developed at the University of Oklahoma (OU) that uses digital beamforming to collect volumetric observations rapidly (5 to 10 s). Between 2011 – 2019, annual field campaigns were conducted to study thunderstorms, resulting in one of the largest collections of PAR data for weather research and extensive involvement of undergraduate and graduate students. In this study, the AIR system, past research findings, and new data examples are highlighted. In addition, the open-access AIR data archive is discussed, which allows interested users to download these unique PAR observations.

**Index Terms** — Phased Array Radar. Imaging Radar. Weather Radar. Tornado. Mesoscale Convective System.

---

## I. Introduction

The atmosphere evolves rapidly over short time scales across many scales of motion. In particular, severe thunderstorms can produce hazards in seconds to minutes (e.g., tornadogenesis) [1]–[7]. Rapid-scanning radars are thus critical to detecting severe weather as well as capturing quickly evolving and dynamic processes needed to understand their formation. Rapid-scanning mobile radars provide close-range observations of severe storms.

To address the need for faster volume scans, several phased array radars (PARs) have been developed for atmospheric science research and advancement of future operational radar network prototypes. Most of these PARs transmit and receive an electronically steered pencil beam, including the National Weather Radar Testbed PAR [8], the MWR-05XP [2], and Rapid Doppler on Wheels [9]. In 2011, the ARRC commissioned the Atmospheric Imaging Radar (AIR) [10] (Figs. 1, 2). The AIR transmits a vertically spoiled beam and receives data using 36 subarray antennas. Using imaging or digital beamforming, the AIR applies different sets of weights to each subarray channel to form multiple simultaneous pencil beams within a 20° field of view. Combined with mechanical rotation in azimuth, this enables faster volumetric scans than other PAR systems and an instantaneous and contiguous view in the vertical plane.

The goal of this study is to summarize the AIR field campaigns and data sets collected between 2011 – 2019, which led to one of the most extensive mobile PAR data sets for weather research. A concise review of research findings from AIR data sets will be presented, and AIR data examples from unpublished studies will be presented to highlight advantages of imaging technology. A short description of the Advanced Radar Research Center's (ARRC's) data repository is also provided, focusing on downloadable AIR cases.

![Photo of the Atmospheric Imaging Radar mounted on a mobile truck platform in a grassy field under a stormy sky, with the radar antenna array visible on the raised mast.](media/document/fig1-air-photo.png)

**Fig. 1.** Photo of the Atmospheric Imaging Radar. The AIR is mounted on a flatbed truck, with the phased array antenna and support electronics enclosure raised on a mast above the truck bed, positioned in an open field with a storm-influenced sky in the background.

---

## II. AIR System Overview and Field Experiment Design

The AIR was a 36-element, vertically oriented, X-band phased array mobile radar capable of digital beamforming [10], [11]. This capability allows for radar imaging vertically, resulting in instantaneous vertical cross sections. With mechanical rotation in azimuth over 90°, the AIR can obtain 90°-by-20° volumetric sectors every 6–7 s.

A summary of the AIR's specifications is provided in Table I. The transmit horn produces a 20°-by-1° (elevation and azimuth, respectively), horizontally polarized beam that is received by the 36 subarrays. Each subarray channel collects IQ data at every pulse. Data are beamformed offline using a variety of traditional and adaptive non-parametric beamforming techniques, ranging from windowed Fourier to Robust Capon [12]. In general, traditional Fourier beamforming was used to process the data sets, and the array aperture combined with Fourier beamforming results in approximately 20, 1°-by-1° beams. Data are then oversampled by a factor of 2 in both azimuth and elevation, and the 5-MHz, pulse-compressed waveform is sampled at 30 m in range [13].

The AIR was extremely well-suited to observe rapidly evolving weather phenomena such as tornadoes. Tornadoes change (and can form/decay) on the order of seconds [2], [3], making it extremely important to scan at high temporal and spatial resolution in order to resolve critical details related to tornadogenesis, tornado maintenance, and tornado dissipation. By mounting the antenna on a mobile platform, researchers were able to navigate to within a few km of forming and ongoing tornadoes. The combined spatial and temporal resolution of the AIR made for the highest-combined (temporal and spatial) resolution of tornadoes in history [11].

An additional important feature of the AIR is the ability to form *consecutive* beams in the vertical dimension. For example, traditional rotating dish antennas and even previously used mobile phased arrays often leave large gaps in elevation in order to increase volumetric update rates. This is due to a limited number of beams (e.g., a limited bandwidth for a phased-array system), or the use of a pencil beam that would take too many rotations of a dish antenna to achieve acceptable volumetric update rates. Given that the AIR creates instantaneous vertical cross sections, the *full* volume is collected with high spatial and temporal resolution, allowing for the analysis of new features in a holistic sense. One drawback of the imaging technique, however, is higher vertical sidelobes and reduced sensitivity compared to parabolic reflector systems. Adaptive beamforming techniques, however, can help address this issue by suppressing vertical sidelobes [12], [14], [15].

The AIR was developed and built in an interdisciplinary manner at the ARRC at the University of Oklahoma, involving students, staff, and faculty from the Electrical and Computer Engineering, Mechanical Engineering, and Meteorology departments. Students led key components of the design of radar software and hardware [16], [17]. The AIR pedestal was designed and built as an undergraduate Capstone project by students in Mechanical Engineering, while much of the radar software/hardware development was part of a PhD dissertation project [17]. Field experiments were largely led by senior Ph.D. students from the Meteorology and Electrical and Computer Engineering departments. Students led forecasting efforts, served as the in-field instrument principal investigator (PI), and operated the AIR. This approach resulted in unprecedented opportunities for graduate and undergraduate students to participate in field experiments, with over a dozen students involved for at least one year of deployments. Additionally, National Science Foundation Research Experiences for Undergraduates students at the National Weather Center in Norman, OK were involved in both field experiments and data analysis. A plethora of undergraduate reports, peer-reviewed conference and journal manuscripts, Master's theses, and Ph.D. dissertations resulted, making it one of the most-prolific radars for student research for nearly a decade.

![Timeline graphic showing AIR milestones from 2007 to 2019, with a small inset photo of the AIR truck deployed in the field.](media/document/fig2-timeline-photo-inset.png)

**Fig. 2.** Timeline of major AIR milestones. The timeline spans 2007–2019 with labeled milestones: "System development begins" (~2007), "System completion & first data" (~2009), "First spring field campaign and tornado case" (~2011), and "AIR retired & museum exhibit created" (~2019). Annual field campaigns are bracketed over the 2011–2019 span, with a small inset photograph of the AIR truck in the field near the "2013" label.

### TABLE I — AIR System Specifications

| Parameter | Value |
|---|---|
| Frequency | 9.55 GHz |
| Transmitter | 3.5-kW TWT |
| Transmit Beamwidth | 20° × 1° |
| Beamwidth (after beamforming) | 1° × 1° |
| Polarization | Single (H) |
| Range resolution | 30 m |
| Number of elements | 36 |

---

## III. Summary of AIR Datasets and Research Findings

AIR field experiments documented a large number of supercell and tornado cases, in addition to other mesoscale phenomena. From 2012 – 2019, 46 separate deployments occurred on supercell thunderstorms including observations of at least 10 tornadoes. In addition, cold fronts and outflow boundaries were targeted to study their turbulent structure. Deployments on mesoscale convective systems (MCSs) and isolated deep convection were conducted to facilitate three-dimensional wind retrievals. In this section, we briefly review the research findings stemming from these data sets.

The AIR has observed all phases of the tornado lifecycle as well as numerous instances of tornadogenesis failure. Tornadogenesis was observed during the 24 May 2016 Woodward, Oklahoma EF-0 tornado and the 19 May 2013 Shawnee, Oklahoma EF-3 tornado [7]. During the 24 May 2016 tornadogenesis case, rotation initially failed to develop upward above 1 km AGL resulting in tornadogenesis failure. However, 1 – 2 min after tornadogenesis failure, rotation rapidly intensifies through at least 5 km AGL to tornado intensity, simultaneously in the lowest 3 km and upward above 3 km over the next 30 s [18]. In an instance of cyclic tornadogenesis, the AIR documented a looped vortex path during the occlusion and rapid formation of a new EF-3 tornado in only 1 min [7]. AIR analysis of the 21 May 2014 Denver, CO supercell documented the rapid evolution of a vortex that failed to reach tornado strength (subtornadic vortex), illustrating its formation during a rear-flank gust front surge [11].

During the mature stages, the AIR documented rapidly evolving debris processes, velocity signature changes, and storm-scale processes within tornadic storms. During the 19 May 2013 Shawnee, Oklahoma tornado, small pockets of debris appeared and ascended preferentially on the south side of the tornado over a period of 30 s [11]. During the Tipton, Oklahoma tornado, rapidly ascending debris were noted during periods of greater vortex tilt [19]. The tornado also exhibited periodic low-level (<500 m AGL) intensity changes occurring both simultaneously and upward in another instance [19]. During a Fujiwara-like interaction of two mesocyclones, the primary mesocyclone weakened over 20 s when the two mesocyclones were in close proximity [11].

The AIR captured the late mature and dissipation stages of the 27 May 2015 Canadian, TX tornado [20]. Time-height analysis of tornado vortex signatures revealed a multi-mode decay process, with the tornado weakening first in a near-simultaneous manner above 1.25 km and then weakening downward to 500 m. About 2 min following the initial weakening, the final dissipation occurs with a rapid decrease in rotation velocity at all altitudes in 30 s.

Although not as extensively studied as tornadoes, scientists used AIR data to analyze other mesoscale phenomena. An important advantage of the AIR compared to traditional radar scans are the instantaneous vertical cross sections obtained. On 19 September 2015, the AIR performed stationary RHI scans every 1-s through a cold front and documented the full lifecycle of several Kelvin-Helmholtz Instabilities [21]. The efficacy of 3D advection correction techniques was also examined for a single-cell thunderstorm [22].

---

## IV. Case Examples and the Open-Access AIR Data Archive

In this section, a few example cases and scientific applications are highlighted from the AIR data archive. The benefits of both high-temporal resolution and instantaneous vertical cross sections are discussed. Finally, the open-access AIR data archive is described.

### A. 16 May 2017 Wheeler, TX, tornado

The AIR crew documented the mature stage of an EF-2 tornado near Wheeler, TX, on 16 May 2017. The AIR operated 110-degree wide sector scans every 7 s with a maximum unambiguous velocity of 25 m s⁻¹. Among the interesting features documented were the lifecycles of several weak-echo reflectivity bands (WRBs) [5], one example of which is shown in Fig. 3. The vertical cross section shows the WRB, which is a narrow column of reduced radar reflectivity factor. In a horizontal view, the WRB is located about 1 km to the north (down range) of the tornado and stretches cyclonically around the north and east side of the tornado (Fig. 3c). The WRB is distinct from the tornado's weak-echo hole (WEH), a circularly shaped area of low reflectivity caused by centrifuging. Vertical cross sections show a strong horizontal vortex located down range from the tornado, and the WRB is located between the tornado and the center of the horizontal vortex (Fig. 3b).

While the authors of [5] hypothesized that these bands were associated with horizontal vortices, the AIR's vertically continuous data clearly show that a strong horizontal vortex is associated with the WRB. In contrast to initial hypotheses, the WRB is not the result of hydrometeor centrifuging within the horizontal vortex, but instead is a vertically elongated feature displaced toward the updraft side of the horizontal vortex.

![Three-panel figure: a) vertical cross section of radar reflectivity factor showing a diagonal band structure with the WRB annotated by an arrow, height 0-10+ km vs range -12 to 12+ km, color scale 10 to 45 dBZ; b) vertical cross section of Doppler velocity on the same axes, showing a tornado and vortex signature annotated with arrows, color scale -60 to +60 m/s; c) PPI (plan position indicator) scan of radar reflectivity factor at 4.0° elevation in X-Y range coordinates, with WEH and WRB both annotated by arrows near the storm's hook-echo region.](media/document/fig3-wheeler-tornado.png)

**Fig. 3.** Instantaneous vertical cross section through the Wheeler, TX 16 May 2017 tornado at 22:29:58 UTC, showing a) radar reflectivity factor, b) Doppler velocity and c) a PPI scan of radar reflectivity factor. The weak-reflectivity band (WRB) is annotated on a) and shows its position north of the tornado. The tornado is evident from the strong aliased velocities in b) and the weak-echo hole (WEH) in c).

### B. 17 May 2016 Elk City, OK, Quasi-Linear Convective System

Detecting tornadoes in quasi-linear convective systems (QLCSs) is exceptionally difficult with the current operational radar network, leading to reduced warning lead-time for QLCS tornadoes compared to supercell tornadoes. Rapid volume scans may illuminate processes related to QLCS mesovortex and tornado formation, thus improving their detection.

On 17 May 2016, AIR captured the transition stage of a high-precipitation supercell becoming a QLCS near Elk City, OK. During this brief AIR deployment (0328 – 0333 UTC), several mesovortices developed along the leading edge of an advancing QLCS outflow surge (Fig. 4). The outflow advanced at approximately 25 m s⁻¹ to the southeast, and 6 – 8 inch diameter trees were reported down around 0330 UTC. At 0330:48 UTC, AIR observations show two small-scale hook echoes associated with three distinct mesovortices (Fig. 4a,b). The northern hook echo had two embedded mesovortices along it (labeled A and B), with an inflection in the hook echo near B. Over the next 20 s, all three mesovortices intensified considerably and inbound velocities of mesovortex A and C became aliased (Fig. 4c,d). While no confirmed tornado was associated with these mesovortices, this analysis illustrates the need to obtain volume scans of mesovortices every 10 – 20 s.

![Four-panel figure arranged 2x2: a) and c) plan-position radar reflectivity factor (Z, dBZ) at 3° elevation for 17-May-2016 03:30:48 UTC and 03:31:10 UTC respectively, both in Zonal/Meridional distance (km) coordinates with a fan-shaped scan sector, color scale 0-60+ dBZ, with arrows pointing to hook-echo tips; b) and d) corresponding Doppler velocity (V, m/s) panels for the same two times, color scale roughly -20 to +20 m/s, with three circled mesovortices labeled A, B, C, and a dashed line indicating an inflection point near mesovortex B.](media/document/fig4-qlcs-mesovortices.png)

**Fig. 4.** Evolution of the QLCS mesovortices between 03:30:48 UTC and 03:31:10 UTC, showing a) and c) radar reflectivity factor (Z) and b) and d) Doppler velocity. The circles show the locations of mesovortices A, B, and C. The solid lines point to the tips of the hook echo, and the dashed line shows the inflection point associated with mesovortex B. All three mesovortices intensify over the 22-s period shown.

### C. 18 May 2018 Wichita, KS, MCS

The structure of convective updrafts is difficult to observe since the uncertainties in vertical velocity retrievals (e.g., from dual-Doppler) are high [23], [24]. One approach to spatially sampling (near-)vertical velocities is to point a phased array radar vertically and image its vertical structure within a conical region. On 18 May 2018, the crew operated the AIR in a near-vertically pointing mode (83° elevation) during an MCS overpass around 0430 UTC, and two different vertical cross sections are shown in Fig. 5. Despite a disorganized appearance on radar, near-vertical velocities sampled by the AIR (Fig. 5a) exceeded 25 m s⁻¹ around 6 km AGL.¹ Unfortunately, severe attenuation did not permit observations past 8 km AGL. Another vertical cross section reveals strong horizontal vorticity between 4–5 km AGL (Fig. 5b), occurring on relatively small scales (about 1 km across). While strong horizontal vorticity is often seen at low altitudes (e.g., along gust fronts), strong horizontal vorticity at middle levels may be attributed to highly turbulent eddies within deep convection.

> ¹Updraft speeds are likely higher since hydrometeor fall speed effects are not included.

![Two-panel figure showing conical vertical scans (near-zenith, 83° elevation) of Doppler velocity (V, m/s) through a mesoscale convective system near Wichita, KS on 18 May 2018. Both panels are inverted-triangle/cone shaped plots with vertical distance (km, 0-25) on the y-axis and range (km) on the x-axis, color scale roughly -25 to +25 m/s. Panel a) shows a small yellow/red patch annotated "Vertical velocities exceeding 25 m/s" near the apex (low altitude). Panel b) shows a region annotated "Strong horizontal vorticity" around 4-5 km altitude, appearing as an adjacent red/green (inbound/outbound) couplet.](media/document/fig5-wichita-mcs.png)

**Fig. 5.** AIR radial velocity from near-zenith (83° elevation angle), conical scans through a mesoscale convective system near Wichita, KS on 18 May 2018. In panel a), strong vertical velocities are observed within the updraft including a small-scale region of aliased velocities implying updraft exceeding 25 m s⁻¹. In panel b), a horizontal vortex is seen at about 3 km AGL.

### D. Open-Access AIR Data Archive

Freely available PAR data are relatively difficult to access. In the spirit of open data access, promoting scientific discovery, and creating educational opportunities, the ARRC has made several of its past PAR (and other) data sets available online (https://arrc.ou.edu/data.html). Data sets are provided in standard community formats, such as CF/radial or DORADE sweep files, to facilitate interfacing with commonly used, community radar software packages (e.g., Python ARM Radar Toolkit or Py-ART [25] or the Lidar Radar Open Software Environment). On this data site, any user can access several AIR data sets, including moment data from the 31 May 2013 El Reno, OK EF-3 tornado (widest tornado on record) [6], [26], the 19 May 2013 Shawnee, OK, EF-3 tornado, and a supercell on 18 May 2017 near McLean, TX (its non-tornadic phase). The ARRC is expanding this website to include more of the AIR's archived data. In the meantime, archived data sets can be requested on the ARRC's data page.

---

## V. Conclusions

Phased array radar technology affords critical new capabilities for atmospheric science researchers seeking to understand fine-scale processes and predict severe hazard formation. This study summarizes a decade of field campaigns with the Atmospheric Imaging Radar, a phased array system that uses imaging (or digital beamforming). Using imaging, the AIR could scan a volumetric sector in only 5 to 10 s and simultaneously collect a 20°-by-1° vertical cross section.

The AIR documented 46 supercells and 10 tornadoes, facilitating several published studies of tornadogenesis, mature tornado evolution, and dissipation. High-temporal resolution observations from the AIR have revealed simultaneous formation of tornadoes within a column, and a two-stage dissipation process. A key advantage of imaging is the ability to obtain a continuous and instantaneous vertical view whereas typical weather radar scanning patterns leave gaps that make vertical structure more difficult to deduce. Here, the vertical structure of a tornado and a weak-echo reflectivity band are illustrated as well as small-scale updraft maxima and turbulent structure within deep convection. In addition, the rapid intensification of three QLCS mesovortices is documented over 20 s, conveying the need for observations as fast as every 10 s to detect their intensification. Several AIR data sets are now available for download through the ARRC's data webpage. Since the AIR's retirement, student research with AIR data has continued, and a small museum exhibit on the AIR was created for the National Weather Museum and Science Center.

In 2022, the ARRC will field two new advanced phased array radars. The ARRC's Polarimetric Atmospheric Imaging Radar (PAIR; [27]) will be a dual-polarization, C-band PAR capable of imaging and traditional pencil beam scanning. Polarimetric capabilities will enable better studies of both microphysical and dynamic processes in storms, and PAIR's longer wavelength provides greater immunity to attenuation. The ARRC is also developing a mobile, digital-at-every-element (all-digital) PAR called Horus [28], which will operate at S band to evaluate all-digital PAR technology as a candidate technology to replace NEXRAD. In addition to these systems, the ARRC has developed novel concepts for future systems, including an S-band, 1°-beamwidth all-digital array called the Transportable Phased Array Radar (TPAR) to deploy worldwide for science field campaigns and a pair of Ka-band imaging systems for studies of microphysics and dynamics of clouds and precipitation.

Using imaging can improve temporal resolution for a next-generation operational phased array weather radar network [29]. While the AIR used a 20°-wide transmit beam, the reduced sensitivity and higher sidelobe levels may not be desirable for operational applications. However, advanced PAR technology (particularly all-digital systems) can change the transmit beam width rapidly to optimize the improvement in temporal resolution and minimize data degradation.

---

## Acknowledgment

The authors are indebted to the ARRC engineering staff (John Meier and Redmond Kelley, in particular) for maintaining the AIR and helping ensure successful field deployments, even on short notice. The authors also thank many students who have helped with AIR field campaigns (Andrew Byrd, Javier Lujan, Martin Satrio, Kyle Pittman, among others). We also thank three anonymous reviewers for their comments.

---

## Footnote (funding, page 1)

AIR data analysis and field deployments have been supported by several NSF grants, including NSF AGS-1303685, AGS-1823478, and AGS-2114817. Under "Distribution Statement A," this work is approved for public release. Distribution is unlimited. James Kurdzo's involvement in this material is supported by the U.S. Air Force under Air Force Contract FA8702-15-D-0001. Any opinions, findings, conclusions, or recommendations expressed in this material are those of the author(s) and do not necessarily reflect the views of the U.S. Air Force.

---

## References

[1] H. B. Bluestein, C. C. Weiss, and A. L. Pazmany, "Mobile Doppler radar observations of a tornado in a supercell near Bassett, Nebraska, on 5 June 1999. Part I: Tornadogenesis," *Mon. Wea. Rev.*, vol. 131, pp. 2954–2967, 2003.

[2] H. B. Bluestein, M. M. French, I. PopStefanija, R. T. Bluth, and J. B. Knorr, "A mobile, phased-array Doppler radar for the study of severe convective storms: The MWR-05XP," *Bull. Amer. Meteor. Soc.*, vol. 91, pp. 579–600, 2010.

[3] M. M. French, H. B. Bluestein, I. PopStefanija, C. A. Baldi, and R. T. Bluth, "Reexamining the vertical development of tornadic vortex signatures in supercells," *Mon. Wea. Rev.*, vol. 141, no. 12, pp. 4576–4601, 2013.

[4] ——, "Mobile, phased-array, Doppler radar observations of tornadoes at X-band," *Mon. Wea. Rev.*, vol. 142, no. 3, pp. 1010–1036, 2014.

[5] J. L. Houser, H. B. Bluestein, and J. C. Snyder, "A fine-scale radar examination of the tornadic debris signature and weak-echo reflectivity band associated with a large, violent tornado," *Mon. Wea. Rev.*, vol. 144, pp. 4101–4130, 2016.

[6] J. M. Kurdzo, D. J. Bodine, B. L. Cheong, and R. D. Palmer, "High-temporal resolution polarimetric X-band Doppler radar observations of the 20 May 2013 Moore, Oklahoma tornado," *Mon. Wea. Rev.*, vol. 143, pp. 2711–2735, 2015.

[7] R. M. Wakimoto, Z. B. Wienhoff, H. B. Bluestein, D. J. Bodine, and J. M. Kurdzo, "Mobile radar observations of the evolving debris field compared with a damage survey of the Shawnee, Oklahoma tornado of 19 May 2013," *Mon. Wea. Rev.*, vol. 148, pp. 1779–1803, 2020.

[8] D. S. Zrnić, J. F. Kimpel, D. E. Forsyth, A. Shapiro, G. Crain, R. Ferek, J. Heimmer, W. Benner, T. J. McNellis, and R. J. Vogt, "Agile-beam phased array radar for weather observations," *Bull. Amer. Meteor. Soc.*, vol. 88, pp. 1753–1766, 2007.

[9] J. Wurman and M. Randall, "An inexpensive, mobile, rapid-scan radar," in *30th Conf. on Radar Meteor.*, Munich, Germany, 2001.

[10] B. Isom, R. Palmer, R. Kelley, J. Meier, D. Bodine, M. Yeary, B. L. Cheong, Y. Zhang, T.-Y. Yu, and M. I. Biggerstaff, "The Atmospheric Imaging Radar: Simultaneous volumetric observations using a Phased Array Weather Radar," *J. Atmos. Oceanic Technol.*, vol. 30, pp. 655–675, 2013.

[11] J. M. Kurdzo, F. Nai, D. J. Bodine, T. A. Bonin, B. L. Cheong, J. Lujan, A. Mahre, and A. D. Byrd, "Observations of severe local storms and tornadoes with the Atmospheric Imaging Radar," *Bull. Amer. Meteor. Soc.*, vol. 98, pp. 915–935, 2017.

[12] J. Li, P. Stoica, and Z. Wang, "On robust capon beamforming and diagonal loading," *IEEE Trans. Signal Process.*, vol. 51, pp. 1702–1715, 2003.

[13] J. M. Kurdzo, B. L. Cheong, R. D. Palmer, G. Zhang, and J. B. Meier, "A pulse compression waveform for improved-sensitivity weather radar observations," *J. Atmos. Oceanic Technol.*, vol. 31, pp. 2713–2731, 2014.

[14] F. Nai, S. Torres, and R. Palmer, "Adaptive beamforming for weather observations using the Atmospheric Imaging Radar," in *2013 IEEE Int. Symp. on Phased Array Sys. Tech.*, IEEE, Ed., Waltham, MA, 2013.

[15] F. Nai, S. M. Torres, and R. D. Palmer, "Adaptive beamspace processing for phased-array weather radars," *IEEE Tran. Geosci. Remote Sens.*, vol. 54, pp. 5688–5698, 2016.

[16] J. Meier, R. Kelley, B. Isom, M. Yeary, and R. Palmer, "Leveraging software defined radio techniques for digital weather radar receiver design," *IEEE Trans. Instr. Meas.*, vol. 61, pp. 1571–1582, 2012.

[17] B. Isom, "The Atmospheric Imaging Radar for high resolution observations of severe weather," Ph.D. dissertation, University of Oklahoma, 2012.

[18] C. B. Griffin, D. Bodine, A. Mahre, J. Kurdzo, and R. Palmer, "High-temporal resolution observations of tornadogenesis using the Atmospheric Imaging Radar," in *39th Int. Conf. on Radar Meteor.*, Amer. Meteor. Soc., Ed., Nara, JAPAN, 2019.

[19] A. Mahre, J. M. Kurdzo, D. J. Bodine, C. B. Griffin, R. D. Palmer, and T. Yu, "Analysis of the 16 May 2015 Tipton, Oklahoma, EF-3 tornado at high spatiotemporal resolution using the Atmospheric Imaging Radar," *Mon. Wea. Rev.*, vol. 146, pp. 2103–2124, 2018.

[20] C. B. Griffin, D. J. Bodine, J. M. Kurdzo, A. Mahre, and R. D. Palmer, "High-temporal resolution observations of the 27 May 2015 Canadian, Texas tornado using the Atmospheric Imaging Radar," *Mon. Wea. Rev.*, vol. 147, pp. 873–891, 2019.

[21] A. Mahre, T.-Y. Yu, R. D. Palmer, and J. M. Kurdzo, "Observations of a cold front at high spatiotemporal resolution using an X-band phased array imaging radar," vol. 8, no. 30, 2017.

[22] A. Shapiro, J. G. Gebauer, N. A. Dahl, D. J. Bodine, A. Mahre, and C. K. Potvin, "Spatially variable advection correction of Doppler radial velocity data," *J. Atmos. Sci.*, vol. 78, pp. 167–188, 2021.

[23] C. K. Potvin, L. J. Wicker, and A. Shapiro, "Assessing errors in variational dual-Doppler wind syntheses of supercell thunderstorms observed by storm-scale mobile radars," *J. Atmos. Oceanic Technol.*, vol. 29, no. 8, pp. 1009–1025, 2012.

[24] M. Oue, P. Kollias, A. Ryzhkov, and E. P. Luke, "Toward exploring the synergy between cloud radar polarimetry and Doppler spectral analysis in deep cold precipitating systems in the arctic," *J. Geophys. Res. Atmos.*, vol. 123, pp. 2797–2815, 2018.

[25] J. J. Helmus and S. M. Collis, "The Python ARM Radar Toolkit (Py-ART), a library for working with weather radar data in the Python programming language," *J. Open Research Software*, vol. 4, e.25, 2016.

[26] H. B. Bluestein, J. C. Snyder, and J. B. Houser, "A multiscale overview of the El Reno, Oklahoma, tornadic supercell of 31 May 2013," *Wea. Forecasting*, vol. 30, pp. 525–552, 2015.

[27] J. L. Salazar, T.-Y. Yu, M. McCord, J. Diaz, J. A. Ortiz, C. Fulton, M. Yeary, R. Palmer, B. L. Cheong, J. M. Kurdzo, and B. Isom, "An ultra-fast scan C-band Polarimetric Atmospheric Imaging Radar," in *IEEE Conf. Phased Array Sys. Tech.*, IEEE, Ed., Waltham, MA, 2019.

[28] M. Yeary, R. Palmer, C. Fulton, J. Salazar, and H. Sigmarsson, "Update on an S-band all-digital mobile phased array radar," in *IEEE Radar Conf.*, IEEE, Ed., Atlanta, GA, 2021, pp. 1–5.

[29] M. Weber, K. Hondl, N. Youssouf, Y. Jung, D. Stratman, B. Putnam, X. Wang, T. Schuur, C. Kuster, Y. Wen, J. Sun, J. Keeler, Z. Ying, J. Cho, J. Kurdzo, S. Torres, C. Curtis, D. Schvartzman, J. Boettcher, F. Nai, H. Thomas, D. Zrnic, I. Ivic, D. Mirkovic, C. Fulton, J. Salazar, G. Zhang, R. Palmer, M. Yeary, K. Cooley, M. Istok, and M. Vincent, "Towards the next generation operational meteorological radar," *Bull. Amer. Meteor. Soc.*, vol. 102, no. 7, pp. E1357–E1383, 2021.

---

## Reference / Citation

D. J. Bodine, J. M. Kurdzo, C. B. Griffin, R. D. Palmer, B. Isom, F. Nai, A. Mahre, M. Yeary, and T.-Y. Yu, "Overview of a Decade of Field Experiments with the Atmospheric Imaging Radar," *2022 IEEE Radar Conference (RadarConf22)*, New York City, NY, USA, 2022, pp. 1–6.

**DOI:** [10.1109/RadarConf2248738.2022.9764270](https://doi.org/10.1109/RadarConf2248738.2022.9764270)

**IEEE Xplore:** https://ieeexplore.ieee.org/document/9764270
