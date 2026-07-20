# Federal Meteorological Handbook No. 11 — Part A: System Concepts, Responsibilities, and Procedures

**Document:** FCM-H11A-2021
**Date:** July 2021
**Agency:** Office of the Federal Coordinator for Meteorological Services and Supporting Research (OFCM/ICAMS)
**Publisher:** U.S. Department of Commerce / National Oceanic and Atmospheric Administration

**Source / Citation:**
- ICAMS Portal: https://www.icams-portal.gov/resources/ofcm/fmh/FMH11/2021_fmh11_parta.pdf

---

## Preface

The Federal Coordinator for Interagency Council for Advancing Meteorological Services (ICAMS) has the responsibility to maintain and publish Federal Meteorological Handbooks. This series of documents provides standards and procedures to facilitate the efficient collection, sharing, and use of meteorological information by agencies of the federal government and private industry.

The original Federal Meteorological Handbook, Number 11 (FMH-11), WSR-88D METEOROLOGICAL OBSERVATIONS, was prepared and published under the auspices of the Interagency Council for Advancing Meteorological Services (ICAMS). The purpose of FMH-11 is to standardize, insofar as practical, the operation of the Weather Surveillance Radar-1988, Doppler (WSR-88D) systems and the procedures used by personnel of the Departments of Commerce, Defense, and Transportation. By approving publication of this handbook, those agencies have agreed to operate their WSR-88D systems accordingly. Some flexibility under certain meteorological, siting, or mission circumstances is permitted to enhance the quality and utility of some WSR-88D products.

All revisions are coordinated among the NEXRAD tri-agencies (Department of Commerce (DOC), Department of Defense (DoD), and Department of Transportation (DOT)) and approved by ICMSSR; thus, they possess the same authority as the initial edition of FMH-11. The revision process is dependent upon the evolution of WSR-88D subsystems hardware, software, and products, as well as resources to make the updates.

The agencies should review the documents annually. The goal is to review and update the handbooks (as necessary) as part of every major WSR-88D software build release. As required, the handbooks will be updated and published in electronic format, and made available on the ICAMS home page at https://www.icams-portal.gov/resources/ofcm/ofcm.htm. Readers can make copies of the handbooks without a request for approval from the ICAMS. A summary of changes made during updates will be in the preface of each part.

In all, FMH-11 has four parts:

- **Part A** — System Concepts, Responsibilities, and Procedures
- **Part B** — Doppler Radar Theory and Meteorology
- **Part C** — WSR-88D Products and Algorithms
- **Part D** — WSR-88D Systems Description and Operational Applications

Part A provides an introduction for the entire handbook including the restatement of some policy, definitions of terms, and agencies' functions and responsibilities under the WSR-88D System Concept. It provides references for the location of systems and WSR-88D products. Familiarity with Part A should aid the user of other parts of FMH-11 and provide a better understanding of the overall WSR-88D system and operations.

### Summary of Major Changes

This issue of FMH-11 Part A replaces the January 2016 version. The document has been updated to streamline information by removing obsolete details and providing links to additional information.

Michael Bonadonna
Director, Office of the Federal Coordinator for Meteorology

---

## Definition of Terms

**WSR-88D System.** A WSR-88D system is composed of a Weather Surveillance Radar - 1988, Doppler and communications links to distribute products to various agency display systems. The functional designations are Radar Data Acquisition (RDA), Radar Product Generator (RPG), and agency user display systems.

**External User.** An external user is anyone other than a Principal User who uses WSR-88D data and products (e.g., information service companies, broadcast meteorologists, aviation interests, marine interests, industrial meteorologists, other government agencies, and universities).

**National Weather Radar Network.** The national weather radar network consists of WSR-88D sites dispersed throughout the United States and its territories.

**Network Site.** A network site is a DOC, DoD, or DOT WSR-88D site that continuously collects, collates, and makes available radar data and products in support of the national weather radar network. The data from these sites are also centrally collected and archived in real time. A WSR-88D system that continuously collects, collates, and makes available via telecommunications, radar data and products supporting the national weather radar network, the FAA's en route weather radar coverage requirements, and DoD operations. This support is performed in accordance with Federal Meteorological Handbook Number 11 (FMH No. 11) Part A, System Concepts, Responsibilities, and Procedures and such agreements as may be made among the Principal Users.

**NEXRAD Program Council (NPC).** Body that provided overall policy, management guidance, and resource commitments for the NEXRAD Program until it was retired in 1997. The NPC could be reassembled under extenuating circumstances, but presently all NPC responsibilities have been delegated to the NEXRAD Program Management Committee.

**NEXRAD Program Management Committee (NPMC).** Tri-agency committee that makes decisions regarding changes, modifications, and new work that require authority to expend significant resources.

**Non-Network Site.** A non-operational WSR-88D system used for training, development, or operational support of the NEXRAD program or an operational WSR-88D system that supports agency specific requirements and is not utilized by the tri-agencies. This includes the non-operational WSR-88D systems at the National Reconditioning Center, National Weather Service (NWS) Training Center, National Severe Storms Laboratory (NSSL), and Radar Operations Center (ROC). This also includes WSR-88D systems that support agency specific requirements (i.e., Camp Humphreys AB ROK (RKSG), Kunsan AB ROK (RKJK), and Kadena AB JA (RODN)) which are not utilized by the tri-agencies.

**Principal User.** Principal users are the NWS, Air Force, and FAA.

**Level II Data.** Digital radial base data (i.e., Reflectivity, Mean Radial Velocity, and Spectrum Width) and Dual Polarization variables (i.e., Differential Reflectivity, Correlation Coefficient, and Differential Phase) output from the signal processor in the RDA. The output also includes status information required to properly interpret the data (e.g., information on synchronization, calibration, date, time, antenna position, Nyquist velocity, and operational mode).

**Level III Products.** Output product data of the RPG that are used for weather analysis, forecasts, warnings, and weather tracking. The pre-defined products are provided in near real-time to the NWS Radar Product Central Collection Dissemination Service (RPCCDS) and archived by the National Centers for Environmental Information (NCEI).

---

## List of Resources

- **WSR-88D Radar Operations Center (ROC) web page:** https://www.roc.noaa.gov/WSR88D/
- **WSR-88D Interface Control Documents for product information:** https://www.roc.noaa.gov/WSR88D/BuildInfo/Files.aspx
- **Dual Polarization training:** training for meteorologists and non-meteorologists is available at the National Weather Service (NWS) Warning Decision Training Division (WDTD) web site: https://training.weather.gov/wdtd/
- **Software build-specific training materials:** https://training.weather.gov/wdtd/buildTraining/RPGRDA.php
- **NWS real-time Level II data monitoring:** https://radar2pub.ncep.noaa.gov/
- **NWS real-time Level III products status:** https://radar3pub.ncep.noaa.gov/
- **NWS RPCCDS information for product users:** https://www.weather.gov/tg/rpccds
- **NCEI radar resources:** order archived Level II data and Level III products via ftp, use NCEI Java Viewer to view the data, etc.: https://www.ncdc.noaa.gov/data-access/radar-data
- **Obtain RPG source code and executables for a LINUX Platform using the Common Operations and Development Environment (CODE):** https://www.weather.gov/code88d/

---

## Chapter 1: Background

### 1.1 Introduction

The impetus for originally preparing this handbook was the development, acquisition, and deployment of the Weather Surveillance Radar - 1988, Doppler (WSR-88D). The motivation for this update is to reflect the changing operation of the WSR-88D system as it evolves to meet new operational requirements.

Federal Meteorological Handbook, Number 11 (FMH-11) replaced Federal Meteorological Handbook, Number 7 (FMH-7), Weather Radar Observations when the WSR-88D was deployed. The original FMH-11 was developed by an Office of the Federal Coordinator for Meteorology (OFCM) working group, comprising personnel from the National Weather Service (NWS) within the Department of Commerce (DOC), the Air Force within the Department of Defense (DoD), and the Federal Aviation Administration (FAA) within the Department of Transportation (DOT). Subsequent updates have been written by the tri-agency WSR-88D Radar Operations Center (ROC) with editorial support from former OFCM and newly formed ICAMS for coordination and approval by the Interdepartmental Committee for Meteorological Services and Supporting Research.

### 1.2 Purpose and Scope

This handbook provides information, guidance, and instruction regarding the tri-agency management and operation of the WSR-88D systems. The following definitions apply throughout this handbook:

- **Shall:** means that a procedure or practice is mandatory.
- **Should:** means that a procedure or practice is recommended.
- **May:** means that a procedure or practice is optional.
- **Will:** means futurity, not a requirement to be applied to a procedure or practice.

### 1.3 Policy

The WSR-88D systems shall be operated to satisfy the integrated set of federal requirements that emanated primarily from the public charters of the departments represented in the Next Generation Weather Radar (NEXRAD) Program Council (NPC) and the NEXRAD Program Management Committee (NPMC). This handbook has been developed to establish standards for the tri-agency operation of the WSR-88D systems and the collection, processing, and dissemination of information to meet those requirements. As a federal system, it shall be operated to meet as many requirements as possible. However, this handbook guides operations to fulfill first those fundamental requirements of the agencies that founded the program.

### 1.4 Changes

Suggestions for modifications and additions should be sent to the WSR-88D Radar Operations Center webmaster at http://www.roc.noaa.gov/WSR88D/Comments.aspx for consideration for inclusion in a future update. No modification shall be adopted that impacts fulfillment of the integrated requirements that drove the development of the program without the consent of the NPMC. Additionally, this handbook will be reviewed and modified if the Memorandum of Agreement for the Interagency Operation of the WSR-88D is modified, expires or is terminated.

---

## Chapter 2: WSR-88D System Concept

### 2.1 Overview of Agency Functions

The following sections provide discussions of each principal user agency's functions with regard to WSR-88D system operations.

#### 2.1.1 Department of Commerce

Within the DOC, the National Oceanic and Atmospheric Administration's (NOAA) NWS is the civilian weather agency of the Federal Government. As such, it must fill a broad spectrum of climatological, meteorological, and hydrological requirements in its efforts to protect life and property and support the economy of the United States. Specifically, the NWS is responsible for the detection and public warning of hazardous weather such as tornadoes, severe thunderstorms, hurricanes, floods, flash floods, winter storms, damaging tides, and any other meteorological or hydrological event with possible harmful effects. The NWS provides weather information for marine operations covering offshore, coastal, Great Lakes, and river and harbor areas. The NWS also routinely provides forecasts for civilian aviation, reservoir regulation, and wildland firefighting agencies. To assist in fulfilling these responsibilities, the NWS operates the national weather radar network and uses information from DoD and DOT WSR-88D systems.

#### 2.1.2 Department of Defense

Within the DoD, the U.S. Air Force operates WSR-88D systems in the continental United States (CONUS), Guam, Japan, and Korea. The Air Force uses information from the DOC and DOT WSR-88D systems in support of meteorological and aerospace environmental services to the Air Force, Army, and other DoD elements. The Air Force is responsible for providing and relaying severe weather warnings for the protection of DoD resources and personnel, managing flood control reservoirs (U.S. Army Corps of Engineers), and providing environmental information to aid the military decision-making process.

#### 2.1.3 Department of Transportation

The DOT, through the FAA, is responsible for the safe and efficient utilization of the National Airspace System. In meeting this responsibility, DOT disseminates information on the location and intensity of potentially hazardous weather conditions to pilots, air traffic controllers, air traffic flow management, and others concerned with aviation. The DOT obtains and processes data from DOC, DoD, and DOT WSR-88D systems for use by DOT and DOC personnel located in DOT facilities.

### 2.2 System Support Management

Operational support for all deployed WSR-88D units is the responsibility of the tri-agency ROC located in Norman, Oklahoma. The ROC provides centralized radar operations support, field assistance, software maintenance, engineering support, and depot-level support (e.g., bull gear replacement, certain Dual Polarization repairs) for the WSR-88D units deployed by the three Principal Users. In addition, the ROC provides tri-agency support for analyzing the potential impacts of wind turbines, communications towers, and other obstacles and potential sources of interference.

### 2.3 Memorandum of Agreement for the Interagency Operation of the WSR-88D

Policies, procedures, and operational concepts, as defined in this handbook, have been agreed to by each principal user agency. Each agency shall endeavor to support, to the highest degree possible and in accordance with the terms of the Memorandum of Agreement (MOA) for Interagency Operation of the WSR-88D, the data, product, and operational requirements of the other agencies. This supportive service shall be consistent with the capabilities and mission priorities of the agency that has received the request for support. The MOA also forms the basis for the membership, leadership, and activities of the Unit Radar Committee (URC) that shall be at each WSR-88D site.

The current MOA is available at: http://www.roc.noaa.gov/WSR88D/PublicDocs/MOA.pdf

### 2.4 External User Access to WSR-88D Level III Products and Level II Data

Real-time access to Level III products and Level II data can be achieved through two networks the NWS manages: (1) Level III products can be obtained through the NWS' Radar Product Central Collection Dissemination Service (RPCCDS); and (2) Level II data can be obtained through the NWS' Level II Data Collection and Distribution Network.

#### 2.4.1 Level III Products

The RPCCDS makes a predefined subset of all WSR-88D products available to external users, in near real time, from every WSR-88D. Users can obtain Level III products from the RPCCDS via a dedicated connection or file transfer protocol. Additional RPCCDS information is available at: https://www.weather.gov/tg/rpccds. There is a predefined subset of products provided to SBN/NOAAPORT users and a few products from each WSR-88D available at: http://radar.weather.gov. Non-NEXRAD agency connections to individual radar sites for products are not permitted.

#### 2.4.2 Level II Data

The principal user agencies offer direct access to Level II data from operational sites on a limited basis to support government operations. Only certified systems will be permitted to connect to radars and only with advance tri-agency approval. For other users, the NWS operates a WSR-88D Level II Data Collection and Distribution Network. This network includes all 122 NWS radars, 25 DoD radars, and 12 OCONUS FAA radars. These data are available in near real time from many sources and can be used or redistributed without any restriction. Information on the network is available at http://www.roc.noaa.gov/WSR88D/Level_II/Level2Info.aspx.

#### 2.4.3 Retention by the National Centers for Environmental Information (NCEI)

Archive Level II and Archive Level III data are electronically sent to the National Centers for Environmental Information (NCEI) in real time permanent retention. The NCEI receives, archives, and makes these products and data available upon request. Information on the WSR-88D data in the NCEI archives can be found at https://www.ncdc.noaa.gov/data-access/radar-data.

---

## Chapter 3: Site Responsibilities

### 3.1 Introduction

The WSR-88D system is vital to supporting the operational mission of each principal user agency. Therefore, WSR-88D systems shall be operated to satisfy the integrated needs of all three agencies. The units shall be operated in accordance with the procedures described in this handbook and as agreed to by the URC within the terms of the MOA for the Interagency Operation of the WSR-88D. The weather forecast office (WFO), weather flight or operational weather squadron that is the manager of the Master System Control Function (MSCF) chairs the URC.

A list of WSR-88D sites can be found at: http://www.roc.noaa.gov/WSR88D/Program/NetworkSites.aspx

Additional site-specific information can be found in the Site ID Database: http://www.roc.noaa.gov/WSR88D/Program/SiteID.aspx

### 3.2 Network Site Responsibilities

WSR-88D Network Sites shall:

- Operate continuously, 24 hours per day
- Collect, collate, and disseminate high quality radar data and products in support of the national weather radar network, the FAA's en route weather radar coverage, and DoD operations.
- Use one of the operational modes and volume coverage patterns (VCPs) agreed to by the URC.
- Set the default precipitation VCP as agreed to by the URC.
- Set the Mode Selection Function to switch the RPG Operational Mode to Clear Air (i.e., to a Clear Air VCP) and to switch the mode from Clear Air to Precipitation as agreed to by the URC.
- Apply appropriate clutter filtering (e.g., Clutter Mitigation Decision Algorithm (CMD)) to reduce ground clutter and anomalous propagation (AP) for the WSR-88D systems the WFOs control, including DoD and DOT systems, via the MSCF.

In addition to the above responsibilities, DoD WSR-88D Network Sites shall:

- Maintain an Implementing Agreement (IA) with the NWS WFO controlling the MSCF (where applicable) regarding the WSR-88D maintenance. The DoD maintenance organization will initiate and maintain the IA. The IA template can be obtained from the DoD WSR-88D focal point listed in Appendix B of the MOA for Interagency Operation of the WSR-88D.

### 3.3 Non-network Site Responsibilities

- Accept the same responsibilities as Network Sites
- Maintain an IA with the NWS WFO controlling the MSCF (where applicable) regarding the WSR-88D maintenance. The DoD maintenance organization will initiate and maintain the IA. The IA template can be obtained from the DoD WSR-88D focal point listed in Appendix B of the MOA for Interagency Operation of the WSR-88D.

---

## Chapter 4: Data Types, Resolution, Operational Modes, Volume Coverage Patterns, and Products

### 4.1 Introduction

This chapter outlines the WSR-88D data types, data resolution, data processing modes, operational modes, volume coverage patterns, and the full suite of products that may be generated.

### 4.2 WSR-88D Data Types

The WSR-88D RDA produces three Base Data Moments (Reflectivity, Mean Radial Velocity, and Spectrum Width) and three Dual Polarization Variables (Differential Reflectivity, Correlation Coefficient, and Differential Phase) for every scanned elevation angle. These data types are described in more detail in Chapter 2 of FMH-11 Part C.

#### 4.2.1 Base Data Moments

**Reflectivity** – The measure of the efficiency of a target in intercepting and returning radio energy. With hydrometeors, it is a function of the drop size distribution, number of particles per unit volume, physical state (i.e., ice or water), shape, and aspect.

**Mean Doppler Velocity** – Reflectivity-weighted average motion of targets toward or away from the radar within a given volume sample.

**Spectrum Width** – A measure of dispersion of velocities within the radar sample volume. Standard deviation of the mean radial velocity spectrum.

#### 4.2.2 Dual Polarization Variables

**Differential Reflectivity** – The ratio of the reflected horizontal and vertical power returns. It indicates drop shape, which provides an estimate of average drop size and precipitation type (e.g., hail or rain). In addition, Differential Reflectivity can be used to identify some non-meteorological targets.

**Correlation Coefficient** – A correlation between the reflected horizontal and vertical power returns. It indicates regions where there is a mixture of precipitation types, such as rain and snow or rain and hail, as well as non-meteorological targets.

**Differential Phase** – A comparison of the returned phase difference between the horizontal and vertical pulses. This phase difference is caused by the difference in the number of wave cycles (or wavelengths) along the propagation path for horizontal and vertically polarized waves. It indicates precipitation type and intensity.

### 4.3 Spatial Resolution

Based on the elevation within the VCP, these data are collected in Super Resolution or Standard Resolution.

#### 4.3.1 Super Resolution

Super Resolution data are collected in the lowest 2 or 3 scans, depending on the VCP. During super resolution scans, the RDA processes data (i.e., Base Moments and Dual Polarization Variables) with 0.5° azimuthal resolution.

#### 4.3.2 Standard Resolution

Standard resolution data produce Base Moments and Dual Polarization Variables with 1° azimuthal by 0.25 km (0.13 nm) range resolution. Reflectivity data are provided to a maximum range of 460 km (248 nm) while the Doppler data and Dual Polarization Variables are provided to 300 km (161 nm).

### 4.4 Operational Modes

Two operational modes have been implemented: Clear Air and Precipitation. Each mode has one product generation list and at least two VCPs. Selection of the operational mode is closely related to the detected coverage of precipitation. The Mode Selection Function (MSF) is designed to automatically determine if precipitation is occurring within 230 km (124 nm) of the radar. The MSF examines the area of reflectivity returns at a specified intensity and compares it to the predefined threshold determined by the URC.

Automatic mode switching from Clear Air to Precipitation and from Precipitation to Clear Air is operator-specified based on URC agreement. Precipitation accumulation estimates are made in all operational modes.

#### 4.4.1 Mode A – Precipitation Mode (VCPs 12, 112, 212, and 215)

This mode should be used when significant weather echoes are present or severe weather is occurring or is anticipated. Usually, this mode will have been selected automatically due to the detection of reflectivity exceeding the predefined threshold. At times, however, such as during the early, mid-level formation of convective echoes, the RPG MSCF operator may choose to enter the Precipitation Mode manually.

#### 4.4.2 Mode B – Clear Air Mode (VCPs 31, 32, and 35)

This mode may be used when there is no detectable precipitation or when precipitation intensity is light and/or areal extent is small, such as light stratiform precipitation or snow. The RPG software will not allow a change to Clear Air Mode until precipitation exceeding the predefined thresholds has not been detected for the period specified in the Clear Air Mode Selection Time.

### 4.5 Volume Coverage Patterns

During operations, the antenna is controlled by automatic scanning programs. Volume coverage patterns are matched to an operational mode to optimize product generation for given meteorological situations; the various VCPs are defined in Chapter 5 of FMH-11 Part C. Table 4-1 is provided for reference. At 18 Network Sites, a supplemental Base Tilt scan below 0.5 degrees elevation may be enabled in the VCPs.

#### 4.5.1 Dynamic Scanning Techniques

Three techniques, Automated Volume Scan Evaluation and Termination (AVSET), Supplemental Adaptive Intra-Volume Low-Level Scan (SAILS), and Mid-Volume Rescan of Low-Level Elevations (MRLE) can significantly shorten the time between data collection at the lowest elevation scan angles. AVSET terminates the current volume scan after the radar has scanned all elevations with significant return. SAILS inserts up to three extra scan(s) at the lowest level of the volume scan (VCP 12, 112, 212, 215, or 35) to provide an additional, evenly-spaced low-level scan(s). MRLE inserts extra scans of up to the four lowest elevations near the middle of the volume scan (VCP 12, 212, 215) to provide an additional mini volume of mid-level elevations. Implementation of AVSET, SAILS, and MRLE are user designated.

### 4.6 WSR-88D Product Suite

WSR-88D product information can be found in the Product Specification and RPG to Class 1 User Interface Control Documents (ICDs) available at http://www.roc.noaa.gov/WSR88D/BuildInfo/Files.aspx and in the RPCCDS and NOAAPORT documents: https://www.weather.gov/tg/rpccds. More detailed descriptions of WSR-88D products and algorithms can be found in FMH-11 Part C.

### Table 4-1: VCP Comparison Table for RPG Operators, July 2020

> *Note: This table is reproduced from the PDF. In the original document it appears as a formatted table image. The VCP comparison table provides operational parameters for each VCP including: number of elevation angles, scan strategy, approximate volume completion time, and whether SAILS/MRLE/AVSET are available.*

*(See the original PDF at the citation link above for the complete formatted VCP Comparison Table.)*

---

## Appendix A: Acronyms and Abbreviations

| Acronym | Definition |
|---------|-----------|
| AGL | Above Ground Level |
| AP | Anomalous Propagation |
| AVSET | Automated Volume Scan Evaluation and Termination |
| CMD | Clutter Mitigation Decision Algorithm |
| CODE | Common Operations and Development Environment |
| CONUS | Continental United States |
| DOC | Department of Commerce |
| DOD | Department of Defense |
| DOT | Department of Transportation |
| FAA | Federal Aviation Administration |
| FMH | Federal Meteorological Handbook |
| IA | Implementing Agreement |
| ICMSSR | Interdepartmental Committee for Meteorological Services and Supporting Research |
| ICAMS | Interagency Council for Advancing Meteorological Services |
| MOA | Memorandum of Agreement |
| MRLE | Mid-Volume Rescan of Low-Level Elevations |
| MSCF | Master System Control Function |
| MSL | Mean Sea Level |
| NCEI | National Centers for Environmental Information |
| NEXRAD | Next Generation Weather Radar |
| NOAA | National Oceanic and Atmospheric Administration |
| NPC | NEXRAD Program Council |
| NPMC | NEXRAD Program Management Committee |
| NSSL | National Severe Storms Laboratory |
| NWS | National Weather Service |
| OCONUS | Outside the Contiguous United States |
| OFCM | Office of the Federal Coordinator for Meteorological Services and Supporting Research |
| PRF | Pulse Repetition Frequency |
| QLCS | Quasi-Linear Convective System |
| RDA | Radar Data Acquisition |
| ROC | WSR-88D Radar Operations Center |
| RPCCDS | Radar Product Central Collection Dissemination Service |
| RPG | Radar Product Generator |
| SAILS | Supplemental Adaptive Intra-Volume Low-Level Scan |
| SZ-2 | Sachidananda – Zrnic Algorithm |
| URC | Unit Radar Committee |
| VCP | Volume Coverage Pattern |
| WDTD | Warning Decision Training Division |
| WFO | Weather Forecast Office |
| WG/DRMO | Working Group for Doppler Radar Meteorological Observations |
| WSR-88D | Weather Surveillance Radar - 1988, Doppler |

---

## Appendix B: Glossary

**Adaptable Parameter:** Generally, data related to a specific WSR-88D system. These data may consist of meteorological or hydrological parameters, or of geographic boundaries, political boundaries, system configuration, telephone numbers, or similar data. Such data may be generated at either a centralized location or locally.

**Automated Volume Scan Evaluation and Termination:** An algorithm that meets WSR-88D agency requirements for faster VCP updates, especially in severe weather situations. The algorithm terminates a volume scan after the WSR-88D has scanned all of the elevations with significant returns.

**Clutter Mitigation Decision Algorithm:** An advanced science algorithm that identifies clutter on a scan-by-scan basis and automatically builds a Bypass Map each volume scan.

**Mid-Volume Rescan of Low-Level Elevations:** A dynamic scanning technique that inserts extra scans of up to the four lowest elevations near the middle of the volume scan into VCPs 12, 212, and 215. The effect of this scanning technique is to provide information about severe weather signatures at or near the surface as well as from the Quasi-Linear Convective System (QLCS) mesovortex genesis region.

**Mode Selection Function:** This function enables automatic switching between Precipitation Mode and Clear Air Mode as well as performing the mode switch manually.

**NEXRAD Program Council:** A NEXRAD Program tri-agency organization composed of senior representatives from DOC, DoD, and DOT, and the Federal Coordinator for Meteorological Services and Supporting Research. The NPC formally retired on November 17, 1997, after granting the PMC overall authority for the NEXRAD program.

**NEXRAD Program Management Committee:** A NEXRAD Program tri-agency organization comprised of representatives of DOC (NWS), DoD (Air Force Life Cycle Management Center), and DOT (FAA). The NPMC provides oversight of the NEXRAD program budget, policy, resource commitment, and management guidance throughout the operational/sustainment life cycle of the WSR-88D program to ensure that both common and unique agency requirements are addressed and resolved. The PMC also serves as a higher level Configuration Control Board for proposed major product improvement changes that affect the WSR-88D system configurations operationally deployed within the three agencies.

**Supplemental Adaptive Intra-Volume Low-Level Scan:** A dynamic scanning technique that inserts new low-level split cut scan into VCPs 12, 112, 212, 215, and 35. The effect of this scanning technique is to decrease the time interval between lowest angle split cut scans.

**SZ-2 (Sachidananda–Zrnic Algorithm):** Provides a range unfolding technique to alleviate the effects of the fundamental range-velocity ambiguity that exists with Doppler weather radars.

**Unit Radar Committee:** A coordinating committee, established by the MOA for Interagency Operation of the WSR-88D, composed of representatives of each principal user agency associated with a particular WSR-88D system.

---

## Relevance to rustywx

Defines the operational context for NEXRAD data: scan strategies, resolution modes, data types, and the split-cut scanning technique that produces the velocity "purple haze" problem addressed by other research in this collection.