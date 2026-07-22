# Federal Meteorological Handbook No. 11 — Part C: WSR-88D Products and Algorithms

**Document:** FCM-H11C-2017  
**Date:** October 2017  
**Agency:** Office of the Federal Coordinator for Meteorological Services and Supporting Research (OFCM)

**Source / Citation:**
- ICAMS Portal: https://www.icams-portal.gov/resources/ofcm/fmh/FMH11/fmh11partC.pdf

---

U.S. DEPARTMENT OF COMMERCE/ National Oceanic and Atmospheric Administration

OFFICE OF THE FEDERAL COORDINATOR FOR
METEOROLOGICAL SERVICES AND SUPPORTING RESEARCH

FEDERAL METEOROLOGICAL HANDBOOK NO. 11

WSR-88D
METEOROLOGICAL
OBSERVATIONS
PART C
WSR-88D PRODUCTS
AND ALGORITHMS

FCM-H11C-2017

Silver Spring, MD
October 2017

OFFICE OF THE
FEDERAL COORDINATOR FOR
METEOROLOGICAL SERVICES AND SUPPORTING RESEARCH

WSR-88D
METEOROLOGICAL OBSERVATIONS

FEDERAL
METEOROLOGICAL
HANDBOOK NO. 11

PART C
WSR-88D PRODUCTS
AND ALGORITHMS

FCM-H11C-2017
Silver Spring, MD

iii

Preface

The Federal Coordinator for Meteorological Services and Supporting Research has the responsibility to maintain
and publish Federal Meteorological Handbooks. This series of documents provides standards and procedures to
facilitate the efficient collection, sharing, and use of meteorological information by agencies of the federal government and private industry.
For background on this particular handbook, see the preface to Part A, Federal Meteorological Handbook, Number
11 (FMH-11), WSR-88D METEOROLOGICAL OBSERVATIONS.
This part of FMH-11 is largely informational. Part C provides includes a description of each product with operational characteristics, potential uses, and operational considerations. It then describes the meteorological and hydrometeorological algorithms used to process the Doppler radar signal and the resulting base data to produce the WSR88D products. Finally, descriptions of the WSR-88D operational modes and volume coverage patterns are provided.
Summary of Changes:
This version of Part C replaces the version published in April 2006. This version incorporates the many system
changes made since that date, updates the document as of Radar Product Generator Build 15 (released in Fall 2014),
and adds information based on operational experience gained during the first decade of WSR-88D operational use
and case studies.
William Schulz, PhD
Federal Coordinator for Meteorological Services and
Supporting Research

iv

Table of Contents
Preface......................................................................................................................................................iv
Table of Contents...................................................................................................................................v
List of Figures.......................................................................................................................................vii
Chapter 1: Introduction..................................................................................................................... 1-1
1.1 Background..............................................................................................................................................................1-1
1.2 Purpose and Scope.................................................................................................................................................1-1
1.3 Organization............................................................................................................................................................1-1

Chapter 2: WSR-88D Meteorological Products............................................................................. 2-1
2.1 Introduction.............................................................................................................................................................2-1
2.2 Clutter Products......................................................................................................................................................2-6
2.3 Base Products........................................................................................................................................................2-12
2.4 Cross-Section Products........................................................................................................................................2-40
2.5 Dual Polarization Base Variables........................................................................................................................2-47
2.6 Reflectivity-Derived Products.............................................................................................................................2-56
2.6.6 High Resolution Enhanced Echo Tops..........................................................................................................2-82
2.7 Velocity-Derived Products.................................................................................................................................2-106
2.8 Dual Polarization-Derived Products................................................................................................................2-125
2.9 LegacyPrecipitation Estimation Products.......................................................................................................2-135
2.10 Legacy Snowfall Estimation Products...........................................................................................................2-151
2.11 Dual Polarization-Derived Precipitation Estimation Products.................................................................2-167
2.12 Aviation Hazard Products...............................................................................................................................2-185
2.13 Other Products..................................................................................................................................................2-194
2.14 Removed Products............................................................................................................................................2-206

Chapter 3: Meteorological and Hydrometeorological Algorithms.............................................. 3-1
3.1 Introduction.............................................................................................................................................................3-1
3.2 Legacy-Derived Algorithms......................................................................................................................................3-1
3.3 Dual Polarization-Derived Algorithms..............................................................................................................3-71
3.4 Aviation Hazards Algorithms.............................................................................................................................3-76
3.5 Removed Algorithms...........................................................................................................................................3-80

Chapter 4: Overview: Data Processing Algorithms....................................................................... 4-1
4.1 Introduction.............................................................................................................................................................4-1
4.2 RDA-Based Data Processing Algorithms...........................................................................................................4-1
v

4.2.3 Point Clutter Rejection........................................................................................................................................4-2
4.3 RPG-Based Data Processing Algorithms.........................................................................................................4-11
References....................................................................................................................................................................4-25

Chapter 5: Operational Modes And Volume Coverage Patterns................................................. 5-1
5.1 Introduction.............................................................................................................................................................5-1
5.2 Operational Modes.................................................................................................................................................5-1
5.3 Volume Coverage Patterns....................................................................................................................................5-1
5.4 Mode Selection Function. ..................................................................................................................................5-22
5.5 Volume Coverage Pattern Adaptable Parameters. ..........................................................................................5-23

Appendix A: Acronyms And Abbreviations.................................................................................. A-1
Appendix B: Glossary.........................................................................................................................B-1

vi

List of Figures


> **Figure 2-1: Clutter Filter Control Product (CFC #24). ............................................................... 2-7**

> Figure 2-2: Clutter Likelihood Reflectivity Product (CLR #132). .............................................. 2-9


> **Figure 2-2: Clutter Likelihood Reflectivity Product (CLR #132). .............................................. 2-9**

> Figure 2-3: Clutter Likelihood Doppler Product (CLD #133).................................................. 2-11


> **Figure 2-3: Clutter Likelihood Doppler Product (CLD #133).................................................. 2-11**

> Figure 2-4a Reflectivity Product (R #19). .................................................................................... 2-14


> **Figure 2-4a Reflectivity Product (R #19). .................................................................................... 2-14**

> Figure 2-4b Reflectivity Product (R #20). .................................................................................... 2-15


> **Figure 2-4b Reflectivity Product (R #20). .................................................................................... 2-15**

> Figure 2-4c Reflectivity Product (R #21). .................................................................................... 2-16


> **Figure 2-4c Reflectivity Product (R #21). .................................................................................... 2-16**

> Figure 2-4d Reflectivity Product (R #16). .................................................................................... 2-17


> **Figure 2-4d Reflectivity Product (R #16). .................................................................................... 2-17**

> Figure 2-4e Reflectivity Product (R #17). .................................................................................... 2-18


> **Figure 2-4e Reflectivity Product (R #17). .................................................................................... 2-18**

> Figure 2-4f Reflectivity Product (R #18). .................................................................................... 2-19


> **Figure 2-4f Reflectivity Product (R #18). .................................................................................... 2-19**

> Figure 2-4g Reflectivity Data Array Product (DR #94). ............................................................ 2-20


> **Figure 2-4g Reflectivity Data Array Product (DR #94). ............................................................ 2-20**

> Figure 2-4h: Reflectivity Super Resolution Data Array Product (DR #153). ......................... 2-21


> **Figure 2-4h: Reflectivity Super Resolution Data Array Product (DR #153). ......................... 2-21**

> Figure 2-5a: Mean Radial Velocity Product (V #25). ................................................................. 2-24


> **Figure 2-5a: Mean Radial Velocity Product (V #25). ................................................................. 2-24**

> Figure 2-5b: Mean Radial Velocity Product (V #26). ................................................................. 2-25


> **Figure 2-5b: Mean Radial Velocity Product (V #26). ................................................................. 2-25**

> Figure 2-5c: Mean Radial Velocity Product (V #27). ................................................................. 2-26


> **Figure 2-5c: Mean Radial Velocity Product (V #27). ................................................................. 2-26**

> Figure 2-5d: Mean Radial Velocity Product (V #22). ................................................................. 2-27


> **Figure 2-5d: Mean Radial Velocity Product (V #22). ................................................................. 2-27**

> Figure 2-5e: Mean Radial Velocity Product (V #23). ................................................................. 2-28


> **Figure 2-5e: Mean Radial Velocity Product (V #23). ................................................................. 2-28**

> Figure 2-5f: Mean Radial Velocity Product (V #24). .................................................................. 2-29


> **Figure 2-5f: Mean Radial Velocity Product (V #24). .................................................................. 2-29**

> Figure 2-5g: Mean Radial Velocity Data Array Product (DV #99). ......................................... 2-30


> **Figure 2-5g: Mean Radial Velocity Data Array Product (DV #99). ......................................... 2-30**

> Figure 2-5h: Mean Radial Velocity Super Resolution Data Array Product (SDV #154). ..... 2-31


> **Figure 2-5h: Mean Radial Velocity Super Resolution Data Array Product (SDV #154). ..... 2-31**

> Figure 2-6: Integrated Terminal Weather System Digital Base Velocity Product (DDBV #93). .2-33


> **Figure 2-6: Integrated Terminal Weather System Digital Base Velocity Product (DDBV #93). .2-33**

> Figure 2-7a Spectrum Width Product (SW #28). ....................................................................... 2-36


> **Figure 2-7a Spectrum Width Product (SW #28). ....................................................................... 2-36**

> Figure 2-7b Spectrum Width Product (SW #29). ....................................................................... 2-37


> **Figure 2-7b Spectrum Width Product (SW #29). ....................................................................... 2-37**

> Figure 2-7c Spectrum Width Product (SW #30). ....................................................................... 2-38


> **Figure 2-7c Spectrum Width Product (SW #30). ....................................................................... 2-38**

> Figure 2-7d: Spectrum Width Super Resolution Data Array Product (SDW #155). ............ 2-39


> **Figure 2-7d: Spectrum Width Super Resolution Data Array Product (SDW #155). ............ 2-39**

> Figure 2-8a: Cross-Section Reflectivity Product (RCS #50). .................................................... 2-43


> **Figure 2-8a: Cross-Section Reflectivity Product (RCS #50). .................................................... 2-43**

> Figure 2-8b: Cross-Section Reflectivity Product (RCS #85). .................................................... 2-44


> **Figure 2-8b: Cross-Section Reflectivity Product (RCS #85). .................................................... 2-44**

> Figure 2-8c: Cross-Section Velocity Product (VCS #51). .......................................................... 2-45


> **Figure 2-8c: Cross-Section Velocity Product (VCS #51). .......................................................... 2-45**

> Figure 2-8d: Cross-Section Velocity Product (VCS #86). ......................................................... 2-46


> **Figure 2-8d: Cross-Section Velocity Product (VCS #86). ......................................................... 2-46**

> Figure 2-9a: Correlation Coefficient Product (CC #160). ......................................................... 2-48


> **Figure 2-9a: Correlation Coefficient Product (CC #160). ......................................................... 2-48**

> vii

vii


> **Figure 2-9b: Digital Correlation Coefficient Product (DCC #161). ........................................ 2-49**

> Figure 2-10a: Differential Reflectivity Product (ZDR #158)..................................................... 2-51


> **Figure 2-10a: Differential Reflectivity Product (ZDR #158)..................................................... 2-51**

> Figure 2-10b: Digital Differential Reflectivity Product (DZD #159)....................................... 2-52


> **Figure 2-10b: Digital Differential Reflectivity Product (DZD #159)....................................... 2-52**

> Figure 2-11a: Specific Differential Phase Product (KDP #162). ............................................. 2-54


> **Figure 2-11a: Specific Differential Phase Product (KDP #162). ............................................. 2-54**

> Figure 2-11b: Digital Specific Differential Phase Product (DKD #163). ............................... 2-55


> **Figure 2-11b: Digital Specific Differential Phase Product (DKD #163). ............................... 2-55**

> Figure 2-12: Base Reflectivity Data Array Edited with DQA Product (DRQ #195). ........... 2-58


> **Figure 2-12: Base Reflectivity Data Array Edited with DQA Product (DRQ #195). ........... 2-58**

> Figure 2-13a: Composite Reflectivity Product (CR #37)............................................................ 2-61


> **Figure 2-13a: Composite Reflectivity Product (CR #37)............................................................ 2-61**

> Figure 2-13b :Composite Reflectivity Product (CR #38). ......................................................... 2-62


> **Figure 2-13b :Composite Reflectivity Product (CR #38). ......................................................... 2-62**

> Figure 2-13c Composite Reflectivity Product (CR #35). ........................................................... 2-63


> **Figure 2-13c Composite Reflectivity Product (CR #35). ........................................................... 2-63**

> Figure 2-13d Composite Reflectivity Product (CR #36). .......................................................... 2-64


> **Figure 2-13d Composite Reflectivity Product (CR #36). .......................................................... 2-64**

> Figure 2-13e: Composite Reflectivity Edited for Anomalous Propagation Product (CRE #97).


> **Figure 2-13e: Composite Reflectivity Edited for Anomalous Propagation Product (CRE #97).**

> 2-65

2-65

> **Figure 2-13f: Composite Reflectivity Edited for Anomalous Propagation Product (CRE #98).**

> 2-66

2-66

> **Figure 2-13g: Composite Reflectivity Edited for Anomalous Propagation Product (CRE #95).**

> 2-67

2-67

> **Figure 2-13h: Composite Reflectivity Edited for Anomalous Propagation Product (CRE #96).**

> 2-68

2-68

> **Figure 2-14a: Layer Composite Reflectivity Average Low-Level Product (LRA #63)........... 2-71**

> Figure 2-14b: Layer Composite Reflectivity Average Mid-Level Product (LRA #64). ......... 2-72


> **Figure 2-14b: Layer Composite Reflectivity Average Mid-Level Product (LRA #64). ......... 2-72**

> Figure 2-14c: Layer Composite Reflectivity Average High-Level Product (LRA #89). ........ 2-73


> **Figure 2-14c: Layer Composite Reflectivity Average High-Level Product (LRA #89). ........ 2-73**

> Figure 2-14d: Layer Composite Reflectivity Maximum Low-Level Product (LRM #65). .... 2-74


> **Figure 2-14d: Layer Composite Reflectivity Maximum Low-Level Product (LRM #65). .... 2-74**

> Figure 2-14e: Layer Composite Reflectivity Maximum Mid-Level Product (LRW #66). ..... 2-75


> **Figure 2-14e: Layer Composite Reflectivity Maximum Mid-Level Product (LRW #66). ..... 2-75**

> Figure 2-14f: Layer Composite Reflectivity Maximum High-Level Product (LRM #90)...... 2-76


> **Figure 2-14f: Layer Composite Reflectivity Maximum High-Level Product (LRM #90)...... 2-76**

> Figure 2-14g: Layer Composite Reflectivity Anomalous Propagation Removed Product (LRM


> **Figure 2-14g: Layer Composite Reflectivity Anomalous Propagation Removed Product (LRM**

> #67). ................................................................................................................................................... 2-77

#67). ................................................................................................................................................... 2-77

> **Figure 2-15: User-Selectable Layer Reflectivity Maximum Product (ULR #137). ................. 2-79**

> Figure 2-16: Echo Tops Product (ET #141). .............................................................................. 2-81


> **Figure 2-16: Echo Tops Product (ET #141). .............................................................................. 2-81**

> Figure 2-17: High Resolution Enhanced Echo Tops Product (EET #135). .......................... 2-84


> **Figure 2-17: High Resolution Enhanced Echo Tops Product (EET #135). .......................... 2-84**

> Figure 2-18: Hail Index Product (HI #59). .................................................................................. 2-87


> **Figure 2-18: Hail Index Product (HI #59). .................................................................................. 2-87**

> Figure 2-19a: Hybrid Scan Reflectivity Product (HSR #33). .................................................... 2-89


> **Figure 2-19a: Hybrid Scan Reflectivity Product (HSR #33). .................................................... 2-89**

> Figure 2-19b: Digital Hybrid Scan Reflectivity Product (DHR #32). ..................................... 2-90


> **Figure 2-19b: Digital Hybrid Scan Reflectivity Product (DHR #32). ..................................... 2-90**

> viii

viii


> **Figure 2-20: Vertically Integrated Liquid Water Product (VIL #57). ...................................... 2-93**

> Figure 2-21: High Resolution Digital Vertically Integrated Liquid Product (DVL #134). ... 2-96


> **Figure 2-21: High Resolution Digital Vertically Integrated Liquid Product (DVL #134). ... 2-96**

> Figure 2-22a: Storm Structure Alphanumeric Tabular Listing (SS #62). ................................ 2-99


> **Figure 2-22a: Storm Structure Alphanumeric Tabular Listing (SS #62). ................................ 2-99**

> Figure 2-22b: Storm Structure Trend Display (SS #62). ......................................................... 2-100


> **Figure 2-22b: Storm Structure Trend Display (SS #62). ......................................................... 2-100**

> Figure 2-23a: Storm Tracking Information Product (STI #58). ............................................ 2-103


> **Figure 2-23a: Storm Tracking Information Product (STI #58). ............................................ 2-103**

> Figure 2-23b: Storm Tracking Information Product (STI #58). ............................................ 2-104


> **Figure 2-23b: Storm Tracking Information Product (STI #58). ............................................ 2-104**

> Figure 2-23c: Storm Tracking Information Combined Attribute Table (STI #58). ............ 2-105


> **Figure 2-23c: Storm Tracking Information Combined Attribute Table (STI #58). ............ 2-105**

> Figure 2-24a: Storm-Relative Mean Radial Velocity Map Product (SRM #56). ................... 2-108


> **Figure 2-24a: Storm-Relative Mean Radial Velocity Map Product (SRM #56). ................... 2-108**

> Figure 2-24b: Storm-Relative Mean Radial Velocity Region Product (SRR #55). ............... 2-109


> **Figure 2-24b: Storm-Relative Mean Radial Velocity Region Product (SRR #55). ............... 2-109**

> Figure 2-25a: Velocity Azimuth Display Wind Profile Product (VWP #48). ....................... 2-114


> **Figure 2-25a: Velocity Azimuth Display Wind Profile Product (VWP #48). ....................... 2-114**

> Figure 2-25b: Velocity Azimuth Display Product (VAD #84). ............................................... 2-115


> **Figure 2-25b: Velocity Azimuth Display Product (VAD #84). ............................................... 2-115**

> Figure 2-26a Mesocyclone Detection Product (MD #141). ................................................... 2-118


> **Figure 2-26a Mesocyclone Detection Product (MD #141). ................................................... 2-118**

> Figure 2-26b: Digital Mesocyclone Detection Product (DMD #149). ................................. 2-119


> **Figure 2-26b: Digital Mesocyclone Detection Product (DMD #149). ................................. 2-119**

> Figure 2-27: Tornado Vortex Signature Product (TVS #61)................................................... 2-121


> **Figure 2-27: Tornado Vortex Signature Product (TVS #61)................................................... 2-121**

> Figure 2-28: Tornado Vortex Signature Rapid Update Product (TRU #143). ..................... 2-124


> **Figure 2-28: Tornado Vortex Signature Rapid Update Product (TRU #143). ..................... 2-124**

> Figure 2-29a Hydrometeor Classification Product (HC #164). ............................................. 2-127


> **Figure 2-29a Hydrometeor Classification Product (HC #164). ............................................. 2-127**

> Figure 2-29b: Digital Hydrometeor Classification Product (DHC #165). ........................... 2-128


> **Figure 2-29b: Digital Hydrometeor Classification Product (DHC #165). ........................... 2-128**

> Figure 2-30 : Hybrid Scan Hydrometeor Classification Product (HHC #177). ................... 2-130


> **Figure 2-30 : Hybrid Scan Hydrometeor Classification Product (HHC #177). ................... 2-130**

> Figure 2-31: Digital Instantaneous Precipitation Rate Product (DPR #176). ...................... 2-132


> **Figure 2-31: Digital Instantaneous Precipitation Rate Product (DPR #176). ...................... 2-132**

> Figure 2-32 Melting Layer Product (ML #166)..........................................................................2-134


> **Figure 2-32 Melting Layer Product (ML #166)..........................................................................2-134**

> Figure 2-33: One-Hour Rainfall Accumulation Product (OHP #78). .................................. 2-136


> **Figure 2-33: One-Hour Rainfall Accumulation Product (OHP #78). .................................. 2-136**

> Figure 2-34a: Hourly Digital Precipitation Array (Part I) (DPA #81). .................................. 2-139


> **Figure 2-34a: Hourly Digital Precipitation Array (Part I) (DPA #81). .................................. 2-139**

> Figure 2-34b: Hourly Digital Precipitation Array (Part II) (DPA #81).................................. 2-139


> **Figure 2-34b: Hourly Digital Precipitation Array (Part II) (DPA #81).................................. 2-139**

> Figure 2-35: Three-Hour Rainfall Accumulation Product (THP #79). ................................ 2-141


> **Figure 2-35: Three-Hour Rainfall Accumulation Product (THP #79). ................................ 2-141**

> Figure 2-36a: Storm Total Rainfall Accumulation Product (STP #80). ................................ 2-144


> **Figure 2-36a: Storm Total Rainfall Accumulation Product (STP #80). ................................ 2-144**

> Figure 2-36b: Digital Storm Total Rainfall Accumulation Product (DSP #138). ................ 2-145


> **Figure 2-36b: Digital Storm Total Rainfall Accumulation Product (DSP #138). ................ 2-145**

> Figure 2-37: User-Selectable Rainfall Accumulation Product (USP #31). ............................ 2-147


> **Figure 2-37: User-Selectable Rainfall Accumulation Product (USP #31). ............................ 2-147**

> Figure 2-38 Supplemental Precipitation Data (SPD #82). ...................................................... 2-150


> **Figure 2-38 Supplemental Precipitation Data (SPD #82). ...................................................... 2-150**

> Figure 2-39: One-Hour Snow Depth Accumulation Product (OSD #145). ........................ 2-153


> **Figure 2-39: One-Hour Snow Depth Accumulation Product (OSD #145). ........................ 2-153**

> Figure 2-40: One-Hour Snow Water Equivalent Accumulation Product (OSW #144). .... 2-155


> **Figure 2-40: One-Hour Snow Water Equivalent Accumulation Product (OSW #144). .... 2-155**

> ix

ix


> **Figure 2-41: Storm Total Snow Depth Accumulation Product (SSD #147). ....................... 2-158**

> Figure 2-42: Storm Total Snow Water Equivalent Product (SSW #146). ............................. 2-160


> **Figure 2-42: Storm Total Snow Water Equivalent Product (SSW #146). ............................. 2-160**

> Figure 2-43: User-Selectable Snow Depth Accumulation Product (USD #151). ................ 2-163


> **Figure 2-43: User-Selectable Snow Depth Accumulation Product (USD #151). ................ 2-163**

> Figure 2-44: User-Selectable Snow Water Equivalent Accumulation Product (USW #150). .2-166


> **Figure 2-44: User-Selectable Snow Water Equivalent Accumulation Product (USW #150). .2-166**

> Figure 2-45: One-Hour Accumulation Product (OHA #169). .............................................. 2-169


> **Figure 2-45: One-Hour Accumulation Product (OHA #169). .............................................. 2-169**

> Figure 2-46: Digital Accumulation Array Product (DAA #170). ........................................... 2-171


> **Figure 2-46: Digital Accumulation Array Product (DAA #170). ........................................... 2-171**

> Figure 2-47: Digital One-Hour Accumulation Difference Product (DOD #174). ............. 2-173


> **Figure 2-47: Digital One-Hour Accumulation Difference Product (DOD #174). ............. 2-173**

> Figure 2-48a: Storm Total Accumulation Product (STA #171). ............................................ 2-178


> **Figure 2-48a: Storm Total Accumulation Product (STA #171). ............................................ 2-178**

> Figure 2-48b: Digital Storm Total Accumulation Product (DSA #172). .............................. 2-179


> **Figure 2-48b: Digital Storm Total Accumulation Product (DSA #172). .............................. 2-179**

> Figure 2-49: Digital Storm Total Difference Product (DSD #175)........................................ 2-182


> **Figure 2-49: Digital Storm Total Difference Product (DSD #175)........................................ 2-182**

> Figure 2-50: Digital User-Selectable Accumulation Product (DUA #173). .......................... 2-184


> **Figure 2-50: Digital User-Selectable Accumulation Product (DUA #173). .......................... 2-184**

> Figure 2-51a: Eddy Dissipation Rate Product (EDR #156). .................................................. 2-186


> **Figure 2-51a: Eddy Dissipation Rate Product (EDR #156). .................................................. 2-186**

> Figure 2-51b: Eddy Dissipation Confidence Product (EDC #157). ..................................... 2-187


> **Figure 2-51b: Eddy Dissipation Confidence Product (EDC #157). ..................................... 2-187**

> Figure 2-52: Gust Front MIGFA Product (GFM #140). ........................................................ 2-189


> **Figure 2-52: Gust Front MIGFA Product (GFM #140). ........................................................ 2-189**

> Figure 2-53: Hail Hazard Layers Product (HHL #179). .......................................................... 2-191


> **Figure 2-53: Hail Hazard Layers Product (HHL #179). .......................................................... 2-191**

> Figure 2-54: Icing Hazard Levels Product (IHL #178). .......................................................... 2-193


> **Figure 2-54: Icing Hazard Levels Product (IHL #178). .......................................................... 2-193**

> Figure 2-55: Archive III Status Product (ASP #152). .............................................................. 2-196


> **Figure 2-55: Archive III Status Product (ASP #152). .............................................................. 2-196**

> Figure 2-56: Example Free Text Message (FTM #75).............................................................. 2-197


> **Figure 2-56: Example Free Text Message (FTM #75).............................................................. 2-197**

> Figure 2-57: General Status Message Product (GSM #2). ...................................................... 2-198


> **Figure 2-57: General Status Message Product (GSM #2). ...................................................... 2-198**

> Figure 2-58: Radar Coded Message National Graphic (RCM #74). ...................................... 2-201


> **Figure 2-58: Radar Coded Message National Graphic (RCM #74). ...................................... 2-201**

> Figure 2-59 User Alert Message (UAM #73). ...........................................................................2-205


> **Figure 2-59 User Alert Message (UAM #73). ...........................................................................2-205**

> Figure 2-60: Reflectivity Enhanced Resolution Product (DR7 #194).................................... 2-208


> **Figure 2-60: Reflectivity Enhanced Resolution Product (DR7 #194).................................... 2-208**

> Figure 2-61:Mean Radial Velocity Enhanced Resolution Product (DV7 #199). ................. 2-210


> **Figure 2-61:Mean Radial Velocity Enhanced Resolution Product (DV7 #199). ................. 2-210**

> Figure 2-62 Combined Shear Product (CS #87). ......................................................................2-212


> **Figure 2-62 Combined Shear Product (CS #87). ......................................................................2-212**

> Figure 2-63 Mesocyclone Product (M #62). ..............................................................................2-215


> **Figure 2-63 Mesocyclone Product (M #62). ..............................................................................2-215**

> Figure 2-64 Mesocyclone Rapid Update Product (MRO #139). ............................................ 2-218


> **Figure 2-64 Mesocyclone Rapid Update Product (MRO #139). ............................................ 2-218**

> Figure 2-65: Severe Weather Analysis Product (SWA: SWR 43, SWV 44, SWW 45, SWS 46). .2-221


> **Figure 2-65: Severe Weather Analysis Product (SWA: SWR 43, SWV 44, SWW 45, SWS 46). .2-221**

> Figure 2-66:Severe Weather Probability Product (SWP #47).................................................. 2-223


> **Figure 2-66:Severe Weather Probability Product (SWP #47).................................................. 2-223**

> Figure 3-1: Velocity Azimuth Display Product. ............................................................................. 3-2


> **Figure 3-1: Velocity Azimuth Display Product. ............................................................................. 3-2**

> Figure 3-2: Velocity Azimuth Display Wind Profile Product. ...................................................... 3-3


> **Figure 3-2: Velocity Azimuth Display Wind Profile Product. ...................................................... 3-3**

> x

x


> **Figure 3-3: Enhanced Echo Tops Algorithm. ............................................................................... 3-6**

> Figure 3-4: Hail Detection Algorithm Process............................................................................... 3-9


> **Figure 3-4: Hail Detection Algorithm Process............................................................................... 3-9**

> Figure 3-5: Hail Index Product. ..................................................................................................... 3-10


> **Figure 3-5: Hail Index Product. ..................................................................................................... 3-10**

> Figure 3-6: Storm Cell Identification and Tracking Algorithm Overview............................... 3-16


> **Figure 3-6: Storm Cell Identification and Tracking Algorithm Overview............................... 3-16**

> Figure 3-7: Storm Cell Segments Algorithm................................................................................. 3-16


> **Figure 3-7: Storm Cell Segments Algorithm................................................................................. 3-16**

> Figure 3-8: Component Development within Storm Cell Centroids Processing.................... 3-18


> **Figure 3-8: Component Development within Storm Cell Centroids Processing.................... 3-18**

> Figure 3-9: Storm Cell Centroid Locations................................................................................... 3-19


> **Figure 3-9: Storm Cell Centroid Locations................................................................................... 3-19**

> Figure 3-10: Cell-based vs. Grid-based VIL.................................................................................. 3-19


> **Figure 3-10: Cell-based vs. Grid-based VIL.................................................................................. 3-19**

> Figure 3-11 Storm Cell Tracking Process. .................................................................................... 3-22


> **Figure 3-11 Storm Cell Tracking Process. .................................................................................... 3-22**

> Figure 3-12: Storm Track Information Product Symbols........................................................... 3-24


> **Figure 3-12: Storm Track Information Product Symbols........................................................... 3-24**

> Figure 3-13: Storm Track Information Product........................................................................... 3-25


> **Figure 3-13: Storm Track Information Product........................................................................... 3-25**

> Figure 3-14: Mesocyclone Product Comparison.......................................................................... 3-32


> **Figure 3-14: Mesocyclone Product Comparison.......................................................................... 3-32**

> Figure 3-15: Mesocyclone Detection Product.............................................................................. 3-33


> **Figure 3-15: Mesocyclone Detection Product.............................................................................. 3-33**

> Figure 3-16: Two-Dimensional Features. ..................................................................................... 3-41


> **Figure 3-16: Two-Dimensional Features. ..................................................................................... 3-41**

> Figure 3-17 TVS Definition............................................................................................................. 3-42


> **Figure 3-17 TVS Definition............................................................................................................. 3-42**

> Figure 3-18 ETVS Definition.......................................................................................................... 3-43


> **Figure 3-18 ETVS Definition.......................................................................................................... 3-43**

> Figure 3-19: TVS Graphic Product................................................................................................ 3-44


> **Figure 3-19: TVS Graphic Product................................................................................................ 3-44**

> Figure 3-20: Reflectivity Enhanced Resolution Product............................................................. 3-52


> **Figure 3-20: Reflectivity Enhanced Resolution Product............................................................. 3-52**

> Figure 3-21: Digital Hybrid Scan Reflectivity Product ............................................................... 3-53


> **Figure 3-21: Digital Hybrid Scan Reflectivity Product ............................................................... 3-53**

> Figure 3-22: One-Hour Rainfall Accumulation Product. ........................................................... 3-63


> **Figure 3-22: One-Hour Rainfall Accumulation Product. ........................................................... 3-63**

> Figure 3-23: Snow Accumulation Algorithm Overview.............................................................. 3-67


> **Figure 3-23: Snow Accumulation Algorithm Overview.............................................................. 3-67**

> Figure 3-24: Storm-Total Snow Depth Accumulation Product. ............................................... 3-68


> **Figure 3-24: Storm-Total Snow Depth Accumulation Product. ............................................... 3-68**

> Figure 3-25 Melting Layer Product. .............................................................................................. 3-72


> **Figure 3-25 Melting Layer Product. .............................................................................................. 3-72**

> Figure 3-26 Height vs. Radial Array. ............................................................................................. 3-73


> **Figure 3-26 Height vs. Radial Array. ............................................................................................. 3-73**

> Figure 3-27: Inputs to the HCA. ................................................................................................... 3-74


> **Figure 3-27: Inputs to the HCA. ................................................................................................... 3-74**

> Figure 3-28 QPE Equations............................................................................................................ 3-75


> **Figure 3-28 QPE Equations............................................................................................................ 3-75**

> Figure 3-29 Flow Chart to IHL...................................................................................................... 3-77


> **Figure 3-29 Flow Chart to IHL...................................................................................................... 3-77**

> Figure 3-30: Vertical Cross-Section of IHL.................................................................................. 3-78


> **Figure 3-30: Vertical Cross-Section of IHL.................................................................................. 3-78**

> Figure 3-31: VIL Values for a Single Storm as a Function of Range and Volume Coverage Pattern


> **Figure 3-31: VIL Values for a Single Storm as a Function of Range and Volume Coverage Pattern**

> Elevation Samples. (Mahoney, 1987).............................................................................................. 3-84

Elevation Samples. (Mahoney, 1987).............................................................................................. 3-84

> **Figure 4-1: Illustration of Staggered PRT Time Series............................................................... 4-10**

> xi

xi


> **Figure 4-2: Reflectivity Factor Signatures. .................................................................................... 4-12**

> Figure 4-3: Radar Product Discriminations. ................................................................................ 4-13


> **Figure 4-3: Radar Product Discriminations. ................................................................................ 4-13**

> Figure 4-4: MPDA Velocity Product Comparison. ..................................................................... 4-22


> **Figure 4-4: MPDA Velocity Product Comparison. ..................................................................... 4-22**

> Figure 5-1: VCP 11. ........................................................................................................................... 5-5


> **Figure 5-1: VCP 11. ........................................................................................................................... 5-5**

> Figure 5-2: VCP 12............................................................................................................................. 5-7


> **Figure 5-2: VCP 12............................................................................................................................. 5-7**

> Figure 5-3:VCP 21............................................................................................................................... 5-9


> **Figure 5-3:VCP 21............................................................................................................................... 5-9**

> Figure 5-4: VCP 121. ....................................................................................................................... 5-12


> **Figure 5-4: VCP 121. ....................................................................................................................... 5-12**

> Figure 5-5: VCP 211. ....................................................................................................................... 5-14


> **Figure 5-5: VCP 211. ....................................................................................................................... 5-14**

> Figure 5-6: VCP 212. ....................................................................................................................... 5-15


> **Figure 5-6: VCP 212. ....................................................................................................................... 5-15**

> Figure 5-7: VCP 221. ....................................................................................................................... 5-18


> **Figure 5-7: VCP 221. ....................................................................................................................... 5-18**

> Figure 5-8: VCP 31. ......................................................................................................................... 5-20


> **Figure 5-8: VCP 31. ......................................................................................................................... 5-20**

> Figure 5-9: VCP 32. ......................................................................................................................... 5-22


> **Figure 5-9: VCP 32. ......................................................................................................................... 5-22**


xii

List of Tables


> **Table 2-1: WSR-88D Meteorological Products.............................................................................. 2-3**

> Table 4-1: SAILS Insert Elevation vs Termination Angle.......................................................... 4-25


> **Table 4-1: SAILS Insert Elevation vs Termination Angle.......................................................... 4-25**

> Table 5-1: VCP 11 Characteristics.................................................................................................... 5-4


> **Table 5-1: VCP 11 Characteristics.................................................................................................... 5-4**

> Table 5-2: VCP 12 Characteristics.................................................................................................... 5-6


> **Table 5-2: VCP 12 Characteristics.................................................................................................... 5-6**

> Table 5-3: VCP 21 Characteristics.................................................................................................... 5-8


> **Table 5-3: VCP 21 Characteristics.................................................................................................... 5-8**

> Table 5-4: VCP 121 Characteristics................................................................................................ 5-11


> **Table 5-4: VCP 121 Characteristics................................................................................................ 5-11**

> Table 5-5: VCP 211 Characteristics................................................................................................ 5-13


> **Table 5-5: VCP 211 Characteristics................................................................................................ 5-13**

> Table 5-6: VCP 212 Characteristics................................................................................................ 5-15


> **Table 5-6: VCP 212 Characteristics................................................................................................ 5-15**

> Table 5-7: VCP 221 Characteristics................................................................................................ 5-17


> **Table 5-7: VCP 221 Characteristics................................................................................................ 5-17**

> Table 5-8: VCP 31 Characteristics, Long Pulse............................................................................ 5-19


> **Table 5-8: VCP 31 Characteristics, Long Pulse............................................................................ 5-19**

> Table 5-9: VCP 32 Characteristics, Short Pulse............................................................................ 5-21


> **Table 5-9: VCP 32 Characteristics, Short Pulse............................................................................ 5-21**

> Table 5-10: An Example of Typical WSR-88D PRF Characteristics Used Operationally (Delta PRI


> **Table 5-10: An Example of Typical WSR-88D PRF Characteristics Used Operationally (Delta PRI**

> Set C)................................................................................................................................................... 5-24

Set C)................................................................................................................................................... 5-24

xiii

xiv

Chapter 1: Introduction

### 1.1 Background.

The material presented in Part C, WSR-88D Products and Algorithms, of Federal Meteorological Handbook No.
11 (FMH-11) describes the meteorological and hydrological products and algorithms implemented in the Weather
Surveillance Radar-1988, Doppler (WSR- 88D). Additional supplementary material is presented to give the user
insight into the subtleties and variations in product interpretations that may arise out of variations in algorithm performance due to differing location, scanning strategy employed, or operational configuration. This Part also serves
to document the meteorological processing that is intrinsic to the WSR-88D system.
The material in Part C is as of software Build 15.0 for the Radar Data Acquisition (RDA) and Radar Product Generator (RPG), Build 13.0 for the Open System Principal User Processor (OPUP) and Build 6.1 for
the Supplemental Product Generator (SPG). The sections covering the usage, strengths/applications and limitations are based primarily on the inputs provided by agency personnel familiar with the operation and meteorological
use of the WSR-88D, the National Weather Service’s (NWS’s) Warning Decision Training Branch (WDTB), Radar
Operations Center (ROC) subject matter experts, and a support services contractor. This version of Part C supersedes the April 2006 version.

### 1.2 Purpose and Scope.

Part C brings together the existing knowledge of WSR-88D products and algorithms.
Additional and more
detailed information regarding operating instructions are contained in baseline WSR-88D technical manuals, other
parts of this Handbook, and the Memorandum of Agreement among the DOC, DoD, and DOT for Interagency
Operation of the WSR-88D dated 24 March 2008 (available at: http://www.roc.noaa.gov/WSR88D/PublicDocs/
MOA.pdf).

### 1.3 Organization.

Part C is organized into five chapters.
• Chapter 1 is this introduction.
• Chapter 2, WSR-88D Meteorological Products, contains a description of each WSR- 88D hydrological and
meteorological product, organized by product type. Product types are organized from base to derived products.
• Chapter 3, Meteorological and Hydrometeorological Algorithms, summarizes the various types of meteorological processing and is intended to serve as a reference for the operational user. It contains a functional
description, listing of the operational parameters, and a discussion of the operational considerations of each
algorithm. A bibliography is included for those desiring or requiring additional in-depth detail.
• Chapter 4, Signal Processing Algorithms, provides brief descriptions of signal processing algorithms.
• Chapter 5, Operational Modes and Volume Coverage Patterns, describes the WSR-88D operational modes and
volume coverage patterns.
• Appendix A lists acronyms and abbreviations used in Part C. Appendix B is a glossary of terms used in this
Handbook. In the 1991 version of Part C, Appendix A provided information on the default product data
levels. This product level information can now be found in the Interface Control Document (ICD) for the
Product Specification, Document Number 2620003T. The document number suffix, “T” in this case for Build
14.0, corresponds to the software build for which the document is baselined. There were no changes to
this document for Build 15.0. The Product Specification document is at: http://www.roc.noaa.gov/WSR88D/
BuildInfo/Files.aspx
• The WSR-88D images used in Part C are largely from an OPUP display mainly for product- comparison
purposes. Other displays used in this section include AWIPS, CVG, and NOAA’s Weather & Climate Toolkit
(WCT). A list of the primary user display systems is in Part D, Section 2.4, of this Handbook.
1-1

April 2017

1-2

Chapter 2: WSR-88D Meteorological Products

### 2.1 Introduction.

This chapter provides an operational description of the WSR-88D meteorological products. Table 2-1 provides a
listing of those products and their system identifiers. Products are organized in the following groups: Clutter; Base;
Cross-Section; Dual Polarization Base Variables; Reflectivity-Derived; Velocity-Derived; Dual Polarization-Derived;
Legacy Precipitation Estimation; Legacy Snowfall Estimation; Dual Polarization-Derived Precipitation Estimation;
Aviation Hazard; Other; and Removed. Several products have been removed since the 2006 version of this chapter.
These products remain in this chapter since they may still be obtained via radar data archives such as the National
Centers for Environmental Information (NCEI).
The section covering each product contains its description and its operational characteristics, a brief statement describing its intended operational usage, and a listing of the known operational strengths and limitations.
The description of the product and how it is generated is followed by a brief narrative and by an itemization of the
operational characteristics that include:
• System Identifier (ID) - The unique identification for that product within the WSR- 88D system. This information includes the product mnemonic and identification number.
• Data - The data or information presented in the products.
• Processing - The processing that has been used to prepare the data or information for presentation.
• Availability - The frequency at which, or conditions under which, the product is available.
• Presentation - A description of how the product is presented.
• Resolution - The geographical area represented by a single product data value.
• Coverage - The geographic area covered by the entire product.
• Data Levels - The parameter resolution or quantization of the data or information in the product. This product
level information is available in the Interface Control Document (ICD) for the Product Specification at: http://
www.roc.noaa.gov/WSR88D/BuildInfo/Files.aspx
• Annotations - Additional alphanumeric information included with the product.
• Special Symbols - Where used, special symbols are described.
• Adaptation Data - Those parameters that can be varied to adapt the product to its application.
Overlays - The following algorithms produce products that can be overlaid on all geographic products to improve operational utility in a given application:
• Gust Front MIGFA Hail Index
• Melting Layer Mesocyclone*
• Mesocyclone Rapid Update* Mesocyclone Detection Severe Weather Probability* Storm Tracking Information
Tornado Vortex Signature Tornado Vortex Rapid Update
• Combined Attribute Table - The following elements make up the table:
• Cell-Based Vertically Integrated Liquid Water
• Hail
Probability of Hail
Probability of Severe Hail
Maximum Expected Hail Size
• Mesocyclone Detection
• Storm Characteristics
Storm Identification
Storm Location
2-1

October 2017

•

Height and Value of Maximum Reflectivity
Height of 30 dBZ Echo Top
Forecast Storm Movement
Tornado Vortex Signature

The usage section describes the potential meteorological applications of the product and lists some of its intended
operational uses.
The strengths/applications section lists the strong points of the products which relate to its applications. The limitations section describes the limiting factors of the product (and its related algorithm). For products based upon
the meteorological algorithm processing, the reader should refer to Chapter 3 for additional discussion.
Where available and appropriate, a pictorial or tabular representation of each product is provided.
*These algorithms have been removed as of summer 2010.

2-2

FMH-11-Part C


> **Table 2-1: WSR-88D Meteorological Products**


List of Products

Mnemonic

System ID

Paragraph

Clutter Products

-

Clutter Filter Control

CFC


2.2.1

Clutter Likelihood Reflectivity

CLR


2.2.2

Clutter Likelihood Doppler

CLD


2.2.3

Base Products

-

Reflectivity

R

16, 17, 18, 19, 20, 21

2.3.1

Reflectivity Data Array

DR


2.3.1

Reflectivity Super Resolution Data Array
Mean Radial Velocity
Mean Radial Velocity Data Array
Mean Radial Velocity Super Resolution Data Array
Integrated Terminal Weather System Digital Base Velocity
Spectrum Width
Spectrum Width Super Resolution Data Array

SDR
V
DV
SDV
DBV
SW
SDW

22, 23, 24, 25, 26, 27
28, 29, 30

2.3.1
2.3.2
2.3.2
2.3.2
2.3.3
2.3.4
2.3.4

Cross-Section Products

-

Reflectivity
Mean Radial Velocity

RCS
VCS

Dual Polarization Base Variables

-

Correlation Coefficient
Digital Correlation Coefficient
Differential Reflectivity
Digital Differential Reflectivity
Specific Differential Phase
Digital Specific Differential Phase

CC
DCC
ZDR
DZD
KDP
DKD

Reflectivity-Derived Products

-

Base Reflectivity Data Array Edited with DQA
Composite Reflectivity
Composite Reflectivity Edited for Anomalous Propagation
Layer Composite Reflectivity Average
Layer Composite Reflectivity Maximum

DRQ
CR
CRE
LRA
LRM

35, 36, 37, 38
95, 96, 97, 98
63, 64, 89
65, 66, 90

2.6.1
2.6.2
2.6.2
2.6.3
2.6.3

Layer Composite Reflectivity Anomalous Propagation Removed
User-Selectable Layer Composite Reflectivity Maximum
Echo Tops
High Resolution Enhanced Echo Tops
Hail Index

APR
ULR
ET
EET
HI


2.6.3
2.6.4
2.6.5
2.6.6
2.6.7

Hybrid Scan Reflectivity

HSR


2.6.8

Digital Hybrid Scan Reflectivity

DHR


2.6.8

Vertically Integrated Liquid

VIL


2.6.9

High Resolution Digital Vertically Integrated Liquid

DVL


2.6.10

Storm Structure

SS


2.6.11

2-3

2.2

2.3

2.4
50, 85
51, 86

2.4.1
2.4.1

2.5

2.5.1
2.5.1
2.5.2
2.5.2
2.5.3
2.5.3

2.6

October 2017


> **Table 2-1: WSR-88D Meteorological Products (Continued)**


List of Products

Reflectivity-Derived Products (Continued)
Storm Tracking Information

Mnemonic
STI

System ID

Velocity-Derived Products
Storm-Relative Radial Velocity—Region
Storm-Relative Radial Velocity—Map
Super Ob
Velocity Azimuth Display
Velocity Azimuth Display Wind Profile
Mesocyclone Detection
Digital Mesocyclone Detection Data Array
Tornado Vortex Signature
Tornado Vortex Signature Rapid Update

Paragraph

2.6

2.6.12

2.7
SRR
SRM
SO
VAD
VWP
MD
DMD
TVS
TRU


Dual Polarization-Derived Products

2.7.1
2.7.1
2.7.2
2.7.3
2.7.3
2.7.4
2.7.4
2.7.5
2.7.6

2.8

Hydrometeor Classification
Digital Hydrometeor Classification

HC
DHC


2.8.1
2.8.1

Hybrid Hydrometeor Classification
Digital Instantaneous Precipitation Rate
Melting Layer

HHC
DPR
ML


2.8.2
2.8.3
2.8.4

OHP
DPA
THP
STP
DSP
USP
SPD


Legacy Precipitation Estimation Products
One-Hour Rainfall Accumulation
Hourly Digital Precipitation Array
Three-Hour Rainfall Accumulation
Storm Total Rainfall Accumulation
Digital Storm Total Rainfall Accumulation
User-Selectable Rainfall Accumulation
Supplemental Precipitation Data

2.9

Legacy Snowfall Estimation Products
One-Hour Snow Depth Accumulation
One-Hour Snow Water Equivalent Accumulation
Storm Total Snow Depth Accumulation
Storm Total Snow Water Equivalent Accumulation
User-Selectable Snow Depth Accumulation
User-Selectable Snow Water Equivalent Accumulation

2.9.1
2.9.2
2.9.3
2.9.4
2.9.4
2.9.5
2.9.6

2.10
OSD
OSW
SSD
SSW
USD
USW


Dual Polarization-Derived Precipitation Estimation Products

2.10.1
2.10.2
2.10.3
2.10.4
2.10.5
2.10.6

2.11

One-Hour Accumulation

OHA


2.11.1

Digital Accumulation Array
Digital One-Hour Accumulation Difference
Storm Total Accumulation
Digital Storm Total Accumulation
Digital Storm Total Difference Accumulation

DAA
DOD
STA
DSA
DSD


2.11.2
2.11.3
2.11.4
2.11.4
2.11.5


> **Table 2-1: WSR-88D Meteorological Products (Concluded)**

> 2-4

2-4

FMH-11-Part C

List of Products

Dual Polarization-Derived Precipitation Estimation Products, Continued
Digital User-Selectable Accumulation

Mnemonic

System ID

Paragraph

2.11

DUA


EDR
EDC
GFM
HHL
IHL


ASP
FTM
GSM
RCM
UAM


Reflectivity Enhanced Resolution

DR7


2.14.1

Mean Radial Velocity Enhanced Resolution
Combined Shear
Mesocyclone
Mesocyclone Rapid Update

DV7
CS
M
MRU


2.14.2
2.14.3
2.14.4
2.14.5

Severe Weather Analysis
Reflectivity
Mean Radial Velocity
Spectrum Width
Radial Shear

SWA
SWR
SWV
SWW
SWS


2.14.6
2.14.6
2.14.6
2.14.6
2.14.6

Severe Weather Probability

SWP


2.14.7

Aviation Hazards Products
Eddy Dissipation Rate
Eddy Dissipation Confidence
Gust Front MIGFA
Hail Hazard Layers
Icing Hazard Layers

2.12

Other Products
Archive III Status Product
Free Text Message
General Status Message
Radar Coded Message
User Alert Message

2.11.6
2.12.1
2.12.1
2.12.2
2.12.3
2.12.4

2.13

Removed Products

2.13.1
2.13.2
2.13.3
2.13.4
2.13.5

2.14

2-5

October 2017


### 2.2 Clutter Products.


#### 2.2.1 Clutter Filter Control.

The Clutter Filter Control product (CFC) provides a display of the type of clutter suppression (Bypass Map and/
or Forced Filtering) in effect for the user-selected elevation segment. The Bypass Map can be from either the offline
generation of a static Bypass Map or the Clutter Mitigation Decision (CMD, see Section 4.2.4.3) generation of a
dynamic Bypass Map, which occurs every volume scan. The CFC is available for the lowest elevation angle in each
segment. The limits for each segment are 1.05°, 1.65°, 4.05°, 6.45°, and 19.5°.
2.2.1.1 Operational Characteristics.
System ID: CFC, Product #34.
Data: Active clutter filtering scheme, forced filtering, or no filtering.
Processing: The RDA Status and Control (RDASC) generates the Clutter Filter Bypass Map.
Availability: Once per volume scan for the lowest elevation angle in each of five segments on user request except
for SAILS slices.
Presentation: Polar coordinate image for user-selected elevation segment and each redundant channel, where applicable (Figure 2-1).
Resolution: 1 km (0.54 nm) x 1°.
Coverage: Radar centered, 230 km (124 nm) radius.
Data Levels: 4.
Annotations:
• Standard set*
• Elevation Segment Number
• Bypass Map generation date/time
Adaptation: None.
2.2.1.2 Usage. Graphically display clutter filter settings.
2.2.1.3 Strengths/Applications:
•
•

Enables Clutter Filter settings to be graphically displayed
Assists in verifying correct clutter settings have been set

2.2.1.4: Limitations. Even though clutter filtering can be applied to a range of 460 km (248 nm), this product only displays clutter filter information to 230 km (124 nm)
_____
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-6

FMH-11-Part C


> **Figure 2-1: Clutter Filter Control Product (CFC #24).**

> An example Clutter Filter Control product (OPUP display) from the Saint Louis, MO (KLSX) WSR88D at 12:38 UTC on 18 April 2013. The area in blue indicates where no filtering is taking place. The

An example Clutter Filter Control product (OPUP display) from the Saint Louis, MO (KLSX) WSR88D at 12:38 UTC on 18 April 2013. The area in blue indicates where no filtering is taking place. The
yellow pixels are the RDA-generated Clutter Bypass Map. In this case, the map was generated with
CMD.

2-7

October 2017


#### 2.2.2 Clutter Likelihood Reflectivity.

The Clutter Likelihood Reflectivity product (CLR) provides a displayable image of the percentage probability that
the radar is detecting ground clutter in the reflectivity radial samples. The product content is based on the output
of the Radar Echo Classifier (REC) algorithm using the Anomalous Propagation (AP)/Clutter target logic.
2.2.2.1 Operational Characteristics.
System ID: CLR, Product #132.
Data: Clutter likelihood (in percent).
Processing: The REC algorithm searches the three base data moments (R, V and SW), within a discrete sampling area, for patterns characteristic of ground clutter and anomalous propagation returns. It combines its results
using “fuzzy logic” to produce a likelihood as a percentage, that a given range bin is a return from ground clutter.
Availability: Once per elevation scan.
Presentation: Polar coordinate image of probability coded range bins (Figure 2-2).
Resolution: 1 km (0.54 nm) x 1°.
Coverage: Radar centered, 230 km (124 nm) radius.
Data Levels: 11.
Annotations:
• Standard set*
• Data level code
• Site adaptable parameters
• Maximum data value detected
Adaptation: None.
2.2.2.2 Usage. The CLR product indicates the likelihood that each range gate is contaminated by clutter return
and provides a measure of reflectivity data quality. This assists in reflectivity data interpretation and application
of the data. Additionally, output of the REC algorithm is used by the precipitation accumulation function to reject
reflectivity range gates that are clutter contaminated.
2.2.2.3 Strengths/Applications. With availability of all three moments, CLR will indicate probability that each
range gate is clutter.
2.2.2.4 Limitations. The CLR results must be used with caution in areas where there are range overlaid velocity
data.
____
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-8

FMH-11-Part C


> **Figure 2-2: Clutter Likelihood Reflectivity Product (CLR #132).**

> An example Clutter Likelihood Reflectivity product (OPUP display) from the Saint Louis, MO (KLSX)

An example Clutter Likelihood Reflectivity product (OPUP display) from the Saint Louis, MO (KLSX)
WSR-88D at 12:38 UTC on 18 April 2013. In this case, the product was generated while some type of
interference was being detected toward the southwest. This product was created at the same time as
the product shown in Figure 2-3.

2-9

October 2017


#### 2.2.3 Clutter Likelihood Doppler.

The Clutter Likelihood Doppler product (CLD) provides a displayable image of the percentage probability that
the radar is detecting ground clutter in the Doppler radial samples. The product is based on the output of the
REC algorithm using the AP/Clutter target logic.
2.2.3.1 Operational Characteristics.
System ID: CLD, Product #133.
Data: Clutter likelihood (in percent).
Processing: The REC algorithm searches the three base data moments (R, V and SW), within a discrete sampling area, for patterns characteristic of ground clutter and anomalous propagation returns. It combines its results
using “fuzzy logic” to produce a likelihood as a percentage, that a given range bin is contaminated by return from
ground clutter.
Availability: Once per elevation scan.
Presentation: Polar coordinate image of probability coded range bins (Figure 2-3).
Resolution: 1 km (0.54 nm) x 1°.
Coverage: Radar centered, 230 km (124 nm) radius.
Data Levels: 12.
Annotations:
• Standard set*
• Data level code
• Site adaptable parameters
• Maximum data values detected
Special Symbols: None.
Adaptation: None.
2.2.3.2 Usage. The data are input by the REC and used to reject Doppler gates that are clutter contaminated. The
CLD product indicates the likelihood that each range gate is contaminated by return from clutter targets and provides a measure of velocity data quality. This can assist in velocity field interpretation and application of the data.
The value displayed for each 1 km (0.54 nm) bin is the maximum of the four corresponding Doppler bins.
2.2.3.3 Strengths/Applications. With availability of all three moments, the CLD will indicate probability that each
range gate is clutter.
2.2.3.4 Limitations. Probabilities of clutter are not available in range folded areas.
____
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-10

FMH-11-Part C


> **Figure 2-3: Clutter Likelihood Doppler Product (CLD #133).**

> An example Clutter Likelihood Doppler product (OPUP display) from the Saint Louis, MO (KLSX)

An example Clutter Likelihood Doppler product (OPUP display) from the Saint Louis, MO (KLSX)
WSR- 88D at 12:38 UTC on 18 April 2013. In this case, the product was generated while some type
of interference was being detected toward the southwest. This product was created at the same time
as the product shown in Figure 2-2.

2-11

October 2017


### 2.3 Base Products.


#### 2.3.1 Ref﻿lectivity.

Reflectivity products provide displays of reflectivity data for each elevation angle scan at spatial resolutions up to
the maximum capability of the RDA. The products are available for several combinations of display resolution and
coverage; products are available for 8, 16, or 256 data levels. A separate version of each product is available for each
antenna scan, depending upon the VCP employed.
2.3.1.1 Operational Characteristics.
System ID: R, Product #16, #17, #18, #19, #20, and #21; DR, Product #94; SDR, Product #153.
Data: Reflectivity (dBZ).
Availability: Once per volume scan for each elevation slice.
Presentation: Polar coordinate of reflectivity data (Figures 2-4, a-h).
Resolution:
• 0.25 km (0.13 nm) x 0.5° (#153 Split Cuts)
• 0.25 km (0.13 nm) x 1° (#153 non-Split Cuts)
• 1 km (0.54 nm) x 1° (#16, #19, #94)
• 2 km (1.1 nm) x 1° (#17, #20)
• 4 km (2.2 nm) x 1° (#18, #21)
Coverage: All reflectivity products are radar centered.
• 460 km (248 nm) radius (#17, #18, #20, #21, #94, #153)
• 230 km (124 nm) radius (#16, #19)
Data Levels:
• 256 (#94, #153)
• 16 (#19, #20, #21)
• 8 (#16, #17, #18)
Annotations:
• Standard set*
• Calibration constant (scaling constant used by the Programmable Signal Processor [PSP] to calculate reflectivity)
• Data level code
• Elevation angle
• Maximum reflectivity value detected
Adaptation: None.
2.3.1.2 Usage.
• As an aid in the analysis of meteorological events by locating and tracking storms.
• Primary use is for surveillance as well as a detailed interpretation on a storm-by- storm basis.
• Identify severe weather signatures, the bright band, and boundaries.
• Monitor the evolution of the planetary boundary layer.
• Estimate rainfall intensity.
2.3.1.3 Strengths/Applications.
• Observe precipitation intensity, movement, and trends.
2-12

FMH-11-Part C

•
•
•
•
•
•
•

Evaluate environmental conditions and meteorological characteristics such as inversions or moisture layers,
especially in the Clear Air Mode.
Identify ice cloud layers and even very light precipitation characteristics.
Identify and locate the freezing/melting level.
Observe and at times even track non-precipitation phenomena such as birds, bats, insects, smoke, volcanic ash,
chaff, etc.
Weak returns from refractive index gradients and small particulates such as insects reveal many characteristics of
the boundary layer.
Determine the location and motion of wind shear lines and boundaries such as gust fronts, synoptic fronts, sea
breezes, and wind-shifts of all kinds.
Determine significant convective storm structural features such as Weak Echo Region (WER), Bounded Weak
Echo Region (BWER), hook echoes, and even evidence for Rear Flank Downdraft (RFD) existence. Line Echo
Wave Patterns (LEWPs) and squall lines can be identified.

2.3.1.4 Limitations.
• Data levels cannot be changed.
• Residual ground and point clutter and AP can contaminate data.
• Chaff and biological echoes are often difficult to distinguish from precipitation echoes.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-13

October 2017


> **Figure 2-4a Reflectivity Product (R #19).**

> An example Reflectivity product (OPUP display) at the 0.5° elevation angle with a resolution of 1° x

An example Reflectivity product (OPUP display) at the 0.5° elevation angle with a resolution of 1° x
1 km (0.54 nm) and 16 data levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18
April 2013. Figures 2-4a through 2-4h are at the same time and elevation angle for comparison purposes.

2-14

FMH-11-Part C


> **Figure 2-4b Reflectivity Product (R #20).**

> An example Reflectivity product (OPUP display) at the 0.5° elevation angle with a resolution of 1° x 2

An example Reflectivity product (OPUP display) at the 0.5° elevation angle with a resolution of 1° x 2
km (1.1 nm) and 16 data levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April
2013.

2-15

October 2017


> **Figure 2-4c Reflectivity Product (R #21).**

> An example Reflectivity product (OPUP display) at the 0.5° elevation angle with a resolution of 1° x 4

An example Reflectivity product (OPUP display) at the 0.5° elevation angle with a resolution of 1° x 4
km (2.2 nm) and 16 data levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April
2013.

2-16

FMH-11-Part C


> **Figure 2-4d Reflectivity Product (R #16).**

> An example Reflectivity product (OPUP display) at the 0.5° elevation angle with a resolution of 1° x 1

An example Reflectivity product (OPUP display) at the 0.5° elevation angle with a resolution of 1° x 1
km (0.54 nm) and 8 data levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April
2013.

2-17

October 2017


> **Figure 2-4e Reflectivity Product (R #17).**

> An example Reflectivity product (OPUP display) at the 0.5° elevation angle with a resolution of 1° x 2

An example Reflectivity product (OPUP display) at the 0.5° elevation angle with a resolution of 1° x 2
km (1.1 nm) and 8 data levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April
2013.

2-18

FMH-11-Part C


> **Figure 2-4f Reflectivity Product (R #18).**

> An example Reflectivity product (OPUP display) at the 0.5° elevation angle with a resolution of 1° x 4

An example Reflectivity product (OPUP display) at the 0.5° elevation angle with a resolution of 1° x 4
km (2.2 nm) and 8 data levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April
2013.

2-19

October 2017


> **Figure 2-4g Reflectivity Data Array Product (DR #94).**

> An example Reflectivity Data Array product (OPUP display) at the 0.5° elevation angle from the Saint

An example Reflectivity Data Array product (OPUP display) at the 0.5° elevation angle from the Saint
Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-20

FMH-11-Part C


> **Figure 2-4h: Reflectivity Super Resolution Data Array Product (DR #153).**

> An example Reflectivity Super Resolution Data Array split cut product (OPUP display) at the 0.5° elevation angle from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

An example Reflectivity Super Resolution Data Array split cut product (OPUP display) at the 0.5° elevation angle from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-21

October 2017


#### 2.3.2 Mean Radial Velocity.

The Mean Radial Velocity products provide displays of radar estimated mean radial velocity based on the value
contained in the first gate of each set of gates (i.e., gate 1 of 1, gate 1 of 2, or gate 1 of 4). They are available for
several combinations of coverage and display resolution; products are available for 8, 16, or 256 data levels. In addition, Products #99 and #154 have a quantization of 0.5 ms-1 or 1 ms-1, depending on what the RDA is providing. A
separate version of each product is available for each antenna scan taken at a constant elevation for each elevation
angle in the current VCP.
2.3.2.1 Operational Characteristics.
System ID: V, Product #22, #23, #24, #25, #26, and #27; DV, Product #99; SDV, Product #154.
Data: Mean radial velocity (m/s).
Availability: Once per volume scan for each elevation slice.
Presentation: Polar coordinate of velocity data (Figures 2-5, a-h).
Resolution:
• 0.25 km (0.13 nm) x 0.5° (#22, #25, #154)
• 0.25 km (0.13 nm) x 1° (#99)
• 0.50 km (0.27 nm) x 1° (#23, #26)
• 1 km (0.54 nm) x 1° (#24, #27)
Coverage: All velocity products are radar centered.
• 300 km (162 nm) radius (#154)
• 230 km (124 nm) radius (#24, #27, #99)
• 115 km (62 nm) radius (#23, #26)
• 60 km (32 nm) radius (#22, #25)
Data Levels:
• 256 (#99, #154)
• 16 (#25, #26, #27)
• 8 (#22, #23, #24)
Annotations:
• Standard set*
• Data level code
• Elevation angle
• Maximum data value detected (both positive and negative)
Adaptation: Data levels.
2.3.2.2 Usage.
• Aids wind flow structure and shear recognition of the atmosphere on various scales.
• The detection and location of tornadic circulations, mesocyclones, and other atmospheric vortices.
• Determination of local wind field characteristics.
• Can be used to identify boundary layer characteristics or outflow regions.
2.3.2.3 Strengths/Applications.
• Estimate magnitude of radial velocities. Ground relative wind speeds (and directions, in so far as they can be
derived and used for use as input into warnings, statements, and forecasts).
• Aid in determining kinematic atmospheric structure via radial velocities. Atmospheric jets and temperature advection can be determined within the local radar coverage area and with adequate reflectors.
2-22

FMH-11-Part C

•
•
•

Aid in determining internal convective storm kinematic structure via radial velocity patterns. Vortices such as
Tornado Vortex Signatures or mesocyclones and divergence intensity signatures at storm top or in association
with microbursts can be identified.
High temporal and spatial resolution.
Aid in creating, adjusting, or updating hodographs.

2.3.2.3 Limitations.
• Range folding may obscure data.
• Velocity aliasing can mask real velocities or shears.
• Velocities may exceed product data levels or even the signal processing specified velocity data levels. The scale is
locally adaptable at the MSCF.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-23

October 2017


> **Figure 2-5a: Mean Radial Velocity Product (V #25).**

> An example Velocity product (OPUP display) with a resolution of 0.5° x 0.25 km (0.13 nm) and 16

An example Velocity product (OPUP display) with a resolution of 0.5° x 0.25 km (0.13 nm) and 16
data levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013. Figures 2-5a
through 2-5h are at the same time and elevation angle for comparison purposes.

2-24

FMH-11-Part C


> **Figure 2-5b: Mean Radial Velocity Product (V #26).**

> An example Velocity product (OPUP display) with a resolution of 1° x 0.5 km (0.27 nm) and 16 data

An example Velocity product (OPUP display) with a resolution of 1° x 0.5 km (0.27 nm) and 16 data
levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-25

October 2017


> **Figure 2-5c: Mean Radial Velocity Product (V #27).**

> An example Velocity product (OPUP display) with a resolution of 1° x 1 km (0.54 nm) and 16 data

An example Velocity product (OPUP display) with a resolution of 1° x 1 km (0.54 nm) and 16 data
levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-26

FMH-11-Part C


> **Figure 2-5d: Mean Radial Velocity Product (V #22).**

> An example Velocity product (OPUP display) with a data resolution of 0.5° x 0.25 km (0.13 nm) and 8

An example Velocity product (OPUP display) with a data resolution of 0.5° x 0.25 km (0.13 nm) and 8
data levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-27

October 2017


> **Figure 2-5e: Mean Radial Velocity Product (V #23).**

> An example Velocity product (OPUP display) with a resolution of 1° x 0.5 km (0.27 nm) and 8 data

An example Velocity product (OPUP display) with a resolution of 1° x 0.5 km (0.27 nm) and 8 data
levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-28

FMH-11-Part C


> **Figure 2-5f: Mean Radial Velocity Product (V #24).**

> An example Velocity product (OPUP display) with a resolution of 1° x 1 km (0.54 nm) and 8 data levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

An example Velocity product (OPUP display) with a resolution of 1° x 1 km (0.54 nm) and 8 data levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-29

October 2017


> **Figure 2-5g: Mean Radial Velocity Data Array Product (DV #99).**

> An example Velocity Data Array product (OPUP display) at the 0.5° elevation angle from the Saint

An example Velocity Data Array product (OPUP display) at the 0.5° elevation angle from the Saint
Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-30

FMH-11-Part C


> **Figure 2-5h: Mean Radial Velocity Super Resolution Data Array Product (SDV #154).**

> An example Velocity Super Resolution Data Array product (OPUP display) at the 0.5° elevation angle

An example Velocity Super Resolution Data Array product (OPUP display) at the 0.5° elevation angle
from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-31

October 2017


#### 2.3.3 Integrated Terminal Weather System Digital Base Velocity.

The Integrated Terminal Weather System (ITWS) Digital Base Velocity product (DBV) provides mean radial velocity in a digital array format to support processing external to the WSR-88D (i.e., primarily FAA ITWS). This product
is generated for each azimuth scan (elevation based) as requested. Indications for data “below the Signal-to-Noise
threshold” and “Range Folding” for each array element is provided. The mean radial velocity range is -63.5 to +63
m/s in 0.5 m/s increments. This data coding is independent of the Doppler velocity resolution of the data provided by the RDA.
2.3.3.1 Operational Characteristics.
System ID: DBV, Product #93.
Data: Mean radial velocity (m/s).
Processing: Mean radial velocity generation.
Availability: Updated once per volume scan for each elevation slice.
Presentation: Data array in polar coordinates (Figure 2-6).
Resolution: 1 km (0.54 nm) x 1°.
Coverage: Lesser of 115 km (62 nm) radius or the range at which the beam intersects 5.5 km (18 kft) Above
Ground Level (AGL).
Data Levels: 256.
Annotations:
• Standard set*
• Minimum data value (in meters per second)
• Increment (in meters per second)
• Number of data levels
• Elevation (in degrees)
• Maximum negative velocity detected (kts)
• Maximum positive velocity detected (kts)
Adaptation: None.
2.3.3.2 Usage. The ITWS will display this data array product according to ITWS requirements.
2.3.3.3 Strengths/Applications. Provides ITWS system users with high-resolution (256 data levels) mean radial
velocity data.
2.3.3.4 Limitations.
• Range folding may obscure data.
• Improper dealiasing may result in erroneous velocity values.
• Product data file size is very large.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-32

FMH-11-Part C


> **Figure 2-6: Integrated Terminal Weather System Digital Base Velocity Product (DDBV #93).**

> An example Integrated Terminal Weather System Digital Base Velocity product (OPUP display) at the

An example Integrated Terminal Weather System Digital Base Velocity product (OPUP display) at the
0.5° elevation angle from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18.

2-33

October 2017


#### 2.3.4 Spectrum Width.

The Spectrum Width products display mean radial velocity spectrum width estimates as a radial image for each
elevation angle scanned, depending on the VCP employed. The estimates are based on the value contained in the
first gate of each set of gates (i.e., gate 1 of 1, gate 1 of 2, or gate 1 of 4). The products are available for several
combinations of display resolution and coverage; products are available for 8 or 256 data levels. A separate version
of each product is available for each antenna scan taken at an available constant elevation angle, depending on the
volume coverage pattern employed.
2.3.4.1 Operational Characteristics.
System ID: SW, Product #28, #29, and #30; SDW, Product #155.
Data: Spectrum width (m/s).
Availability: Updated once per volume scan at each elevation angle.
Presentation: Polar coordinate image of spectrum width data (Figures 2-7, a-d).
Resolution:
• 0.25 km (0.13 nm) x 0.5° (#155 Split Cuts)
• 0.25 km (0.13 nm) x l° (#28, #155 non-Split Cuts)
• 0.5 km (0.27 nm) x l° (#29)
• 1 km (0.54 nm) x 1° (#30)
Coverage: All spectrum width products are radar centered.
• 300 km (162 nm) radius (#155)
• 230 km (124 nm) radius (#30)
• 115 km (62 nm) radius (#29)
• 62 km (32 nm) radius (#28)
Data Levels:
• 256 (#155)
• 8 (#28, #29, #30)
Annotations:
• Standard set*
• Data level code
• Elevation angle
• Maximum spectrum width value detected (kts)
Adaptation: None.
2.3.4.2 Usage.
• The SW product should be used in conjunction with other products for maximum utility.
• Spectrum width data are related to the turbulence intensity as well as the wind shear across the beam. Beam
width and antenna rotation speed also contribute to the width. Primary use of spectrum width is to estimate
turbulence associated with:
▪ Thunderstorms
▪ Fronts and gust fronts
▪ Clear air
• Useful for evaluating validity of the mean radial velocity estimates.
• Evidence of convective development will often appear in the spectrum width field before any significant return
is detected in the reflectivity field.
2-34

FMH-11-Part C

•

Can be used to determine the structure of the bright band because of increased variance in velocities through
the bright band.

2.3.4.3 Strengths/Applications.
• Can be used to infer weak signal returns near the noise threshold.
• A number of other applications have been suggested for low values. For example they indicate flow acceleration
such as with a deepening tropical storm or hurricane, or flow into a strong updraft. Low values within updrafts
also indicate unmixed updrafts characterized by high helicity.
• It has been suggested that broad values can be used to indicate a weakening hurricane, a turbulent updraft becoming mixed with ambient air, a downdraft, or data contamination with three-body scattering.
2.3.4.4 Limitations.
• As with the Mean Radial Velocity Product, range folding may obscure needed spectrum width data.
• Movement of ground clutter may result in high spectrum width values. For example, cars on the road and
blowing leaves on trees in the summer can contribute to high velocity variances in the range bin. Turbulent flow
around ground targets such as buildings and water towers may also result in high spectrum widths.
• Weak power returns will cause erratic values near the noise level.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-35

October 2017


> **Figure 2-7a Spectrum Width Product (SW #28).**

> An example Spectrum Width product (OPUP display) at the 0.5° elevation angle with a resolution of

An example Spectrum Width product (OPUP display) at the 0.5° elevation angle with a resolution of
1° x 0.25 km (0.13 nm) and 8 data levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC
on 18 April 2013. Figures 2-7a through 2-7d are at the same time and elevation angle for comparison
purposes.

2-36

FMH-11-Part C


> **Figure 2-7b Spectrum Width Product (SW #29).**

> An example Spectrum Width product (OPUP display) with a resolution of 1° x 0.5 km (0.27 nm) and 8

An example Spectrum Width product (OPUP display) with a resolution of 1° x 0.5 km (0.27 nm) and 8
data levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-37

October 2017


> **Figure 2-7c Spectrum Width Product (SW #30).**

> An example Spectrum Width product (OPUP display) with a resolution of 1° x 1 km (0.54 nm) and 8

An example Spectrum Width product (OPUP display) with a resolution of 1° x 1 km (0.54 nm) and 8
data levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-38

FMH-11-Part C


> **Figure 2-7d: Spectrum Width Super Resolution Data Array Product (SDW #155).**

> An example Spectrum Width Super Resolution Data Array split cut product (OPUP display) with a resolution of 0.5° x 0.25 km (0.13 nm) and 256 data levels at an elevation of 0.5° from the Saint Louis,

An example Spectrum Width Super Resolution Data Array split cut product (OPUP display) with a resolution of 0.5° x 0.25 km (0.13 nm) and 256 data levels at an elevation of 0.5° from the Saint Louis,
MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-39

October 2017


### 2.4 Cross-Section Products.


#### 2.4.1 Cross-Section Reflectivity and Mean Radial Velocity.

The Cross-Section products provide a vertical cross-section of either reflectivity or mean radial velocity as an image
for a user-selected vector. The operator defines the vector by using two end points, up to 230 km (124 nm) apart,
and at any orientation and location within 230 km (124 nm) of the radar. The products are produced by mapping
the nearest value in range (within certain limits), to a point in the plane of the vertical cross-section. For data gaps,
the WSR-88D linearly interpolates between the mapped values, both horizontally and vertically as necessary, for the
Cross-section products.
2.4.1.1 Operational Characteristics.
System ID: RCS, Product #50 and #85; VCS, Product #51 and #86.
Data:
• Reflectivity or mean radial velocity (dBZ or m/s)
• End points of cross-section
Processing:
• Map nearest data value from volume scan to the point in the plane of the vertical cross- section defined by the
intersection of the plane and radial.
• Interpolate linearly (both horizontally and vertically) between mapped values, as necessary to fill blank areas.
• When several data values from a volume scan map into a grid box of the plane of the vertical cross-section, the
maximum data value is selected.
• Vertical interpolation between grid boxes, in which the centers of the beams are located at the radial distances
of the cross-section.
• The presentation of the display does not account for beam broadening as a function of range.
Availability: Upon user request for one of the data types listed above.
Presentation:
• Cartesian image of requested data; Cartesian data plane of height versus distance (Figures 2-8, a-d).
• Radar located at origin of product display axes for range height indicator (RHI) version of product (one end
point at 0, 0). Otherwise end points may be placed anywhere within 124 nm of the radar as long as the baseline of the product is less than 124 nm.
Resolution: 1 km (0.54 nm) horizontal x 0.5 km (0.27 nm) vertical.
Coverage: 230 km (124 nm) radius x 21.3 km (70,000 ft) vertical.
Data Levels:
• 16 (#50, #51)
• 8 (#85, #86)
Annotations:
• Standard set*
• Data level code
• Data type
• Location of vector center and the end points (azimuth and range)
• Maximum data value detected and location
Adaptation: Velocity data levels.
2.4.1.2 Usage.
• Vertical viewing of storms or other radar features along a selected path of interest.
2-40

FMH-11-Part C

•

Reflectivity cross-section useful for estimating echo tops and, to a limited degree, vertical structure of reflectivity.
▪ The sensitivity of the WSR-88D allows the detection of mid-level and high-level clouds. The range of detection is dependent on the ice crystal and snowflake composition.
▪ Stratocumulus clouds will often contain precipitation aloft as raindrops or ice crystals and snowflakes, or both,
large enough to be detected by the WSR-88D.
▪ Precipitation events can be monitored as they evolve and dissipate by observations of descending reflectivity
patterns.
▪ Moisture or turbulent layers, or both, can be observed from backscatter from refractivity fluctuations.
▪ The evolution of the planetary boundary layer can be monitored from the backscatter from refractivity structure, particulates, and biological targets.
▪ Horizontal rolls and the evolution of shear oriented convergent bands/cloud streets at the top of the planetary boundary layer can be monitored. The depth of the planetary boundary layer can be monitored during a
diurnal cycle.
▪ Features such as the bright band may be observed.
• Mean Radial Velocity cross-sections can be used to identify boundary layer characteristics, outflow regions,
storm summit divergence, jets aloft, descending rear inflow jets, mesocyclone vertical extent, radial convergence
aloft, etc.
• Setting of the end points at the radar location will produce a simulation of the more traditional RHI display.
2.4.1.3 Strengths/Applications.
• Detects vertical extent and placement of precipitation, clouds (containing precipitation sized particulates), insects, smoke plumes, and volcanic ash eruptions.
• Verify existence and location of a bright band.
• Estimate heights of given reflectivity values including echo tops (18.5 dBZ).
• Evaluate storm structure features such as the WER and BWER along the cross-section axis.
• Monitor vertical development and dissipation of precipitation.
• Velocity cross-sections aid in determining storm top divergence, strength and vertical extent of the convective storm Deep Convergence Zone or the Mid-Altitude Radial Convergence region, the mesocyclone, and the
TVS.
• Aids in estimating vertical depth and placement of various layers such as cold air, gust fronts, and other types of
fronts and regions of convergence and divergence.
• Has proven value in gaining insight into atmospheric kinematic structure in a research setting.
2.4.1.4 Limitations.
• Mean Radial Velocity cross-sections along an arbitrary axis will have mean radial velocity values at unknown
orientations to cross-section plane.
• Cross-section placement may hamper evaluation of storm structure.
• Height estimates are inaccurate due to beam width increases with range, gaps in the VCP, refractive index variations, etc.
• Non-contiguous volume scanning and observations at long range will produce coarse resolution.
• Echo tops and bases are truncated.
• Fast-moving storms may appear to be strongly tilted.
• The horizontal and vertical extent of echo layers will depend on the beam width, pulse width, and range to the
target. Generally, the result is to enlarge the features due to the depth/width of the radar beam.
• Reflectivity values of layers may be reduced due to incomplete beam filling.
2-41

October 2017

•
•
•
•

Interpolation between gaps may enlarge or miss features.
Maxima, minima, and strong gradients of reflectivity will be reduced because of effective beam width and gatelength considerations.
Reflectivity may be caused by backscatter from birds, insects, particulate matter or precipitation elements, or the
refractive index structure.
Displays have an aspect ratio that exaggerates the vertical scale (21.3 km (70,000 ft) as it relates to the horizontal
scale (230 km (124 nm)) causing vertical stretching and relative apparent horizontal shrinking of features.

_____
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-42

FMH-11-Part C


> **Figure 2-8a: Cross-Section Reflectivity Product (RCS #50).**

> An example Cross-Section Reflectivity product (OPUP display) with 16 data levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

An example Cross-Section Reflectivity product (OPUP display) with 16 data levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-43

October 2017


> **Figure 2-8b: Cross-Section Reflectivity Product (RCS #85).**

> An example Cross-Section Reflectivity product (OPUP display) with 8 data levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

An example Cross-Section Reflectivity product (OPUP display) with 8 data levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-44

FMH-11-Part C


> **Figure 2-8c: Cross-Section Velocity Product (VCS #51).**

> An example Cross-Section Velocity product (OPUP display) with 16 data levels from the Saint Louis,

An example Cross-Section Velocity product (OPUP display) with 16 data levels from the Saint Louis,
MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-45

October 2017


> **Figure 2-8d: Cross-Section Velocity Product (VCS #86).**

> An example Cross-Section Velocity product (OPUP display) with 8 data levels from the Saint Louis,

An example Cross-Section Velocity product (OPUP display) with 8 data levels from the Saint Louis,
MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-46

FMH-11-Part C


### 2.5 Dual Polarization Base Variables.


#### 2.5.1 Correlation Coefficient.

A measure of the correlation between the reflected horizontal and vertical power returns of a dual polarized signal.
It is a good indicator of precipitation types, such as rain, snow, and mixed, and is therefore useful in identification
of the melting layer. Low correlation coefficient values are associated with non-meteorological scatterers. Low
values are also found in regions of Mie scattering, indicative of very large hail when associated with meteorological
echoes.
2.5.1.1 Operational Characteristics.
System ID: CC, Product #160; DCC, Product #161.
Data: A unitless ratio of horizontal power returns divided by vertical power returns.
Processing: Correlation coefficient data are corrected for noise at the RDA.
Availability: Once per elevation for each elevation slice.
Presentation: Polar image and data array (Figures 2-9, a-b).
Resolution:
• 0.25 km (0.13 nm) x 1° (#161)
• 1 km (0.54 nm) x 1° (#160)
Coverage: All correlation coefficient products are radar centered.
• 300 km (162 nm) radius (#161)
• 230 km (124 nm) radius (#160)
Data Levels:
• 256 (#161)
• 16 (#160)
Annotations: Standard set*.
Adaptation: None.
2.5.1.2 Usage. In combination with other base data, correlation coefficient is used to form a conceptual four-dimensional model of precipitation type.
2.5.1.3 Strengths/Applications.
• Identification of regions of mixed precipitation type.
• Identification of meteorological versus non-meteorological echoes.
• Identification of large hail.
• Identification of melting layer. Melting layer will generally appear as a circular band of lower correlation coefficient values, particularly at mid-level and high-level elevation angles.
• Can be used to identify lofted tornado debris.
• Can be used to evaluate quality of radar calibration. Calibration is suspect if significant regions of correlation
coefficient values above 1.0 are observed.
2.5.1.4 Limitations.
• Product can be noisy, particularly at lower elevation angles, due to non-meteorological echo and near the edges
of weather echo due to low signal-to-noise ratio.
• GMAP clutter filtering can cause low bias in weather regions.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.
2-47

October 2017


> **Figure 2-9a: Correlation Coefficient Product (CC #160).**

> An example Correlation Coefficient product (OPUP display) at the 0.5° elevation angle with a resolution of 1° x 1 km (0.54 nm) and 16 data levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38

An example Correlation Coefficient product (OPUP display) at the 0.5° elevation angle with a resolution of 1° x 1 km (0.54 nm) and 16 data levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38
UTC on 18 April 2013. Figures 2-9a and 2-9b are at the same time and elevation angle for comparison purposes.

2-48

FMH-11-Part C


> **Figure 2-9b: Digital Correlation Coefficient Product (DCC #161).**

> An example Digital Correlation Coefficient product (OPUP display) at the 0.5° elevation angle with a

An example Digital Correlation Coefficient product (OPUP display) at the 0.5° elevation angle with a
resolution of 1° x 0.25 km (0.13 nm) and 256 data levels from the Saint Louis, MO (KLSX) WSR-88D
at 12:38 UTC on 18 April 2013.

2-49

October 2017


#### 2.5.2 Differential Reflectivity.

The logarithm of the ratio of the reflected horizontal and vertical power returns. It is a good indicator of the mean
particle shape in the sample volume.
2.5.2.1 Operational Characteristics.
System ID: ZDR, Product #158; DZD, Product #159.
Data: Ten times the logarithm of the ratio of horizontal power return divided by vertical power return, dB.
Processing: Differential reflectivity data are corrected for noise at the RDA. The RPG smooths the data, then
corrects for attenuation and system calibration through the dual pol preprocessor.
Availability: Once per elevation for each elevation slice.
Presentation: Polar image and data array (Figures 2-10, a-b).
Resolution:
• 0.25 km (0.13) x 1° (#159)
• 1 km (0.54 nm) x 1° (#158)
Coverage: All differential reflectivity products are radar centered.
• 300 km (162 nm) radius (#159)
• 230 km (124 nm) radius (#158)
Data Levels:
• 256 (#159)
• 16 (#158)
Annotations: Standard set*.
Adaptation: None.
2.5.2.2 Usage. In combination with other base data, used to form a four-dimensional model of precipitation type.
2.5.2.3 Strengths/Applications.
• In thunderstorms, good for the detection of hail, of graupel growth associated with updrafts and of liquid
water above the melting layer marking updraft location.
• Higher differential reflectivity is often observed in the melting layer and in association with some non-meteorological echoes.
2.5.2.4 Limitations.
• Can be noisy in areas of low signal-to-noise ratio and non-meteorological scatterers.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-50

FMH-11-Part C


> **Figure 2-10a: Differential Reflectivity Product (ZDR #158).**

> An example Differential Reflectivity product (OPUP display) at the 0.5° elevation angle with a resolution of 1° x 1 km (0.54 nm) and 16 data levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38

An example Differential Reflectivity product (OPUP display) at the 0.5° elevation angle with a resolution of 1° x 1 km (0.54 nm) and 16 data levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38
UTC on 18 April 2013. Figures 2-10a and 2-10b are at the same time and elevation angle for comparison purposes.

2-51

October 2017


> **Figure 2-10b: Digital Differential Reflectivity Product (DZD #159).**

> An example Digital Differential Reflectivity product (OPUP display) at the 0.5° elevation angle with a

An example Digital Differential Reflectivity product (OPUP display) at the 0.5° elevation angle with a
resolution of 1° x 0.25 km (0.13 nm) and 256 data levels from the Saint Louis, MO (KLSX) WSR-88D
at 12:38 UTC on 18 April 2013.

2-52

FMH-11-Part C


#### 2.5.3 Specific Differential Phase.

The along-the-radial derivative of the differential phase (PHI) shift, or the rate of change in the phase difference
between the horizontal and vertical pulses.
2.5.3.1 Operational Characteristics.
System ID: KDP, Product #162; DKD, Product #163.
Data: Degrees per kilometer.
Processing: Total differential phase is unwrapped, median filtered, average filtered, interpolated between meteorological echoes along a radial, then specific differential phase is computed over a limited increment. See the Dual
Polarization Preprocessor Algorithm Enunciation Language (AEL) document for details.
Availability: Once per elevation scan for each elevation slice.
Presentation: Polar image and data array (Figures 2-11, a-b).
Resolution:
• 0.25 km (0.13 nm) x 1° (#163)
• 1 km (0.54 nm) x 1° (#162)
Coverage: All specific differential phase products are radar centered.
• 300 km (162 nm) radius (#163)
• 230 km (124 nm) radius (#162)
Data Levels:
• 256 (#163)
• 16 (#162)
Annotations: Standard set*.
Adaptation: None.
2.5.3.2 Usage. In combination with other dual polarization data fields to identify areas of heavy rain and as input
to the dual polarization Quantitative Precipitation Estimation (QPE) for determining rain rates in areas of partial
beam blockage.
2.5.3.3 Strengths/Applications.
• Strongly related to rain rate.
• Sensitive to radar calibration, partial beam blockage, propagation effects and system noise.
2.5.3.4 Limitations. Negative values are possible and are not used by the QPE.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-53

October 2017


> **Figure 2-11a: Specific Differential Phase Product (KDP #162).**

> An example Specific Differential Phase product (OPUP display) at the 0.5° elevation angle with a resolution of 1° x 1 km (0.54 nm) and 16 data levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38

An example Specific Differential Phase product (OPUP display) at the 0.5° elevation angle with a resolution of 1° x 1 km (0.54 nm) and 16 data levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38
UTC on 18 April 2013. Figures 2-11a and 2-11b are at the same time and elevation angle for comparison purposes.

2-54

FMH-11-Part C


> **Figure 2-11b: Digital Specific Differential Phase Product (DKD #163).**

> An example Digital Specific Differential Phase product (OPUP display) at the 0.5° elevation angle with

An example Digital Specific Differential Phase product (OPUP display) at the 0.5° elevation angle with
a resolution of 1° x 0.25 km (0.13 nm) and 256 data levels from the Saint Louis, MO (KLSX) WSR88D at 12:38 UTC on 18 April 2013.

2-55

October 2017


### 2.6 Reflectivity-Derived Products.


#### 2.6.1 Base Reflectivity Data Array Edited with DQA.

The reflectivity data have been edited by the Data Quality Assurance (DQA) algorithm (see Section 4.3.1). Constant
power signature artifacts, anomalous propagation, ground clutter, solar strobes, and spikes/speckles are removed.
2.6.1.1 Operational Characteristics.
System ID: DRQ, Product #195.
Data. DQA-edited Reflectivity (encoded – convert to dBZ)
Availability: Once per volume scan for each elevation slice except for SAILS slices.
Presentation: Polar coordinate of reflectivity data (Figure 2-12).
Resolution: 1 km (0.54 nm) x 1°.
Coverage: Radar centered, 460 km (248 nm); lesser range for higher elevation slices.
Data Levels: 256.
Annotations:
• Standard set*
• Data level code
• Elevation angle
• Maximum reflectivity value detected
• Number of artifact edited radials in the elevation
• AVSET status
Adaptation: None.
2.6.1.2 Usage.
• Is the only reflectivity product edited for non-weather contaminants (except chaff) especially useful for automated algorithms?
• As an aid in the analysis of meteorological events by locating and tracking storms.
• Primary use is for surveillance as well as a detailed interpretation on a storm-by-storm basis.
• Identify severe weather signatures, the bright band, and boundaries.
• Monitor the evolution of the planetary boundary layer.
• Estimate rainfall intensity.
2.6.1.3 Strengths/Applications.
• Observe precipitation intensity, movement, and trends.
• Evaluate environmental conditions and meteorological characteristics such as inversions or moisture layers,
especially in the Clear Air Mode.
• Identify ice cloud layers and even very light precipitations characteristics.
• Identify and locate the freezing/melting level.
• Observe and at times even track non-precipitation phenomena such as birds, bats, insects, smoke, volcanic ash,
chaff, etc.
• Weak returns from refractive index gradients and small particulates such as insects reveal many characteristics of
the boundary layer.
• Determine the location and motion of wind shear lines and boundaries such as gust fronts, synoptic fronts, sea
breezes, and wind-shifts of all kinds.
• Determine significant convective storm structural features such as WER, BWER, hook echoes, and even evidence for Rear Flank Downdraft (RFD) existence.
2-56

FMH-11-Part C

•

Line Echo Wave Patterns (LEWP) and squall lines can be identified

2.6.1.4 Limitations.
• Data levels cannot be changed.
• Residual data contaminants might remain in data.
• Chaff echoes are often difficult to distinguish from precipitation echoes.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-57

October 2017


> **Figure 2-12: Base Reflectivity Data Array Edited with DQA Product (DRQ #195).**

> An example Base Reflectivity Data Array Edited with DQA product (OPUP display) from the Saint

An example Base Reflectivity Data Array Edited with DQA product (OPUP display) from the Saint
Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-58

FMH-11-Part C


#### 2.6.2 Composite Reflectivity.

The Composite Reflectivity products (CR and CRE, edited for anomalous propagation) provide composite reflectivity data displayable as an image. For each geographical resolution element, this product provides the highest
reflectivity value above the resolution element available from any elevation angle scan of a volume scan. Thus, the
value displayed for a given location within the product could come from any of the elevation scans contained in the
current VCP. Storm information generated by the various meteorological algorithms (e.g., Storm Cell Identification
and Tracking (SCIT) algorithm, Hail Detection Algorithm (HDA), Mesocyclone Detection Algorithm (MDA), and
Tornado Detection Algorithm (TDA)) are included as annotations or graphic overlays, or both, as the user selects.
When selected, the algorithm-generated information is provided for all identified storms via the Combined Attribute Table (CAT).
2.6.2.1 Operational Characteristics.
System ID: CR, Product #35, #36, #37, and #38; CRE, Product #95, #96, #97, and #98.
Data:
• Composite reflectivity (dBZ).
• Meteorological algorithm output (various units).
Processing:
• Assign highest value of reflectivity from any elevation angle in volume scan
• Produce combined attribute table
Availability: Updated once per volume scan.
Presentation: Cartesian image of composite reflectivity values (Figures 2-13, a-h).
Resolution:
• 1 x 1 km (0.54 x 0.54 nm) (#35, #37, #95, #97)
• 4 x 4 km (2.2 x 2.2 nm) (#36, #38, #96, #98)
Coverage: All composite reflectivity products are radar centered.
• 460 km (248 nm) radius (#36, #38, #96, #98)
• 230 km (124 nm) radius (#35, #37, #95, #97)
Data Levels:
• 16 (#37, #38, #97, #98)
• 8 (#35, #36, #95, #96)
Annotations:
• Standard set*
• Combined Attribute Table
• Data level code
• Maximum CR or CRE data value detected
Adaptation: None.
2.6.2.2 Usage. CR
• Permits a view of maximum reflectivity levels for the total vertical volume within the range of the radar. As
such, it provides a synopsis of the most important reflectivity features in the entire vertical coverage area.
• Allows the user to quickly establish the maximum aerial extent of reflectivity patterns and locate reflectivity
maxima with respect to the surface of the Earth. However, information regarding the height above ground or
the 3-dimensional structure of the reflectivity pattern must be determined by the use of other products.
• Under favorable reflectivity conditions, a bright band feature may enable the user to identify the presence of the
melting level.
2-59

October 2017

CRE
• Similar to CR except that much of the AP is removed.
2.6.2.3 Strengths/Applications. CR
• Provides help selecting vertical cross-sections through maximum reflectivity core.
• The CAT is available and provides algorithm established critical storm characteristics at a glance.
• Reveals highest reflectivity in all echoes.
• Determine some storm structural features and intensity trends in storms when compared to base products.
• Reveals the existence of a bright band or other layers by one or more concentric radar- centered circles.
CRE
• Similar to CR except that much of the AP is removed.
2.6.2.4 Limitations. CR
• Low-level reflectivity signatures are obscured.
• Height of reflectivity is unknown.
• Echo aloft cannot be distinguished from precipitation reaching the surface.
• Ice crystal clouds aloft, such as altostratus and cirrus, may appear as areas of surface precipitation.
• Non-precipitation echoes (e.g., point clutter, anomalous propagation, biological targets, and particulate matter)
may contaminate the product.
CRE
• Similar to CR except that while much of the AP is removed, residual AP will persist under some circumstances.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-60

FMH-11-Part C


> **Figure 2-13a: Composite Reflectivity Product (CR #37).**

> An example Composite Reflectivity product (OPUP display) with a resolution of 1 km x 1 km (0.54 nm

An example Composite Reflectivity product (OPUP display) with a resolution of 1 km x 1 km (0.54 nm
x 0.54 nm) and 16 data levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April
2013. Figures 2-13a through 2-13h are at the same time for comparison purposes.

2-61

October 2017


> **Figure 2-13b :Composite Reflectivity Product (CR #38).**

> An example Composite Reflectivity product (OPUP display) with a resolution of 4 km x 4 km (2.2 nm

An example Composite Reflectivity product (OPUP display) with a resolution of 4 km x 4 km (2.2 nm
x 2.2 nm) and 16 data levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April
2013.

2-62

FMH-11-Part C


> **Figure 2-13c Composite Reflectivity Product (CR #35).**

> An example Composite Reflectivity product (OPUP display) with a resolution of 1 km x 1 km (0.54 nm

An example Composite Reflectivity product (OPUP display) with a resolution of 1 km x 1 km (0.54 nm
x 0.54 nm) and 8 data levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April
2013.

2-63

October 2017


> **Figure 2-13d Composite Reflectivity Product (CR #36).**

> An example Composite Reflectivity product (OPUP display) with a resolution of 4 km x 4 km (2.2 nm x

An example Composite Reflectivity product (OPUP display) with a resolution of 4 km x 4 km (2.2 nm x

### 2.2 nm) and 8 data levels from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.


2-64

FMH-11-Part C


> **Figure 2-13e: Composite Reflectivity Edited for Anomalous Propagation Product (CRE #97).**

> An example Composite Reflectivity Edited for Anomalous Propagation product (OPUP display) with

An example Composite Reflectivity Edited for Anomalous Propagation product (OPUP display) with
a resolution of 1 km x 1 km (0.54 nm x 0.54 nm) and 16 data levels from the Saint Louis, MO (KLSX)
WSR- 88D at 12:38 UTC on 18 April 2013.

2-65

October 2017


> **Figure 2-13f: Composite Reflectivity Edited for Anomalous Propagation Product (CRE #98).**

> An example Composite Reflectivity Edited for Anomalous Propagation product (OPUP display) with

An example Composite Reflectivity Edited for Anomalous Propagation product (OPUP display) with
a resolution of 4 km x 4 km (2.2 nm x 2.2 nm) and 16 data levels from the Saint Louis, MO (KLSX)
WSR-88D at 12:38 UTC on 18 April 2013.

2-66

FMH-11-Part C


> **Figure 2-13g: Composite Reflectivity Edited for Anomalous Propagation Product (CRE #95).**

> An example Composite Reflectivity Edited for Anomalous Propagation product (OPUP display) with

An example Composite Reflectivity Edited for Anomalous Propagation product (OPUP display) with
a resolution of 1 km x 1 km (0.54 nm x 0.54 nm) and 8 data levels from the Saint Louis, MO (KLSX)
WSR- 88D at 12:38 UTC on 18 April 2013.

2-67

October 2017


> **Figure 2-13h: Composite Reflectivity Edited for Anomalous Propagation Product (CRE #96).**

> An example Composite Reflectivity Edited for Anomalous Propagation product (OPUP display) with a

An example Composite Reflectivity Edited for Anomalous Propagation product (OPUP display) with a
resolution of 4 km x 4 km (2.2 nm x 2.2 nm) and 8 data levels from the Saint Louis, MO (KLSX) WSR88D at 12:38 UTC on 18 April 2013.

2-68

FMH-11-Part C


#### 2.6.3 Layer Composite Reflectivity.

The Layer Composite Reflectivity products (LRA and LRM are available for three layers; the depth of each layer is
controlled by adaptation data, but must be greater than 6,000 ft. The APR version of the product is available only
for the low-layer. Default values for the layers are surface to 24,000 ft MSL (low), 24,000 ft to 33,000 ft MSL
(mid), and 33,000 to 60,000 ft MSL (high). For a layer, the layering algorithm establishes a composite value by
either taking the maximum or computing an average of the individual values within the layer grid box.
2.6.3.1 Operational Characteristics.
System ID: LRA (average), Product #63, #64 and #89 (low-, mid-, and high-layers respectively). LRM (maximum),
Product #65, #66, and #90 (low-, mid-, and high-layers respectively). APR (anomalous propagation removed), Product
#67 (low-layer).
Data:
• LRA and LRM: Composite reflectivity (dBZ)
• APR: Composite reflectivity, results of AP editor
Processing:
• Averaging algorithm
• Layer Composite algorithm
Availability: Six products per volume scan (average and maximum reflectivity for low, middle, and high layers) plus
APR product for low layer.
Presentation: Cartesian image of layer composite reflectivity values (Figures 2-14, a-g).
Resolution: 4 x 4 km (2.2 x 2.2 nm).
Coverage: Radar centered, 230 x 230 km (124 x 124 nm).
Data Levels: 8.
Annotations:
• Standard set*
• Calibration constant (scaling constant used by the PSP to calculate reflectivity)
• Data level code
• Layer boundary (vertical depth)
• Maximum reflectivity value detected.
Adaptation:
• Composite method (average or maximum)
• Depth of layers
2.6.3.2 Usage. LRA:
• Intended primarily for distribution to the Air Route Traffic Control Centers (ARTCCs) for use of the Center
Weather Service Unit (CWSU) meteorologist and for the development of regional layer composite reflectivity
maps.
• Comparison of LRA maps with CR products can provide insight regarding vertical development of both stratiform and cumuliform regimes.
• The high-layer product is useful for identifying those weather systems with extensive vertical development.
LRM:
• Intended primarily for distribution to the ARTCCs for use of the CWSU meteorologist and for the development of regional layer composite reflectivity maps.
• Comparison of layer Composite maps with composite reflectivity can provide insight regarding vertical development of both stratiform and cumuliform regimes.
2-69

October 2017

•
•

The high-layer product is useful for identifying those weather systems with extensive vertical development.
Useful for monitoring heights of reflectivity maxima.

APR:
• Used as the LRM to prevent misinterpretation and applications when AP is detected prior to developing the
product.
2.6.3.3 Strengths/Applications. LRA:
• Use the mid-level product to help differentiate meteorological echoes from ground clutter.
LRM:
• Mid-level and high-level products are used to estimate the height of the higher reflectivity values.
• Comparison of Reflectivity and mid-layer or high-layer LRM products may aid in determining height interval of
maximum reflectivity and a storm’s intensity trend.
• Use the mid-level product to help differentiate meteorological echoes from ground clutter.
APR:
• The APR algorithm attempts to distinguish weather targets from clutter targets.
2.6.3.4 Limitations LRA and LRM:
• Altitude limitations on scanning may result in missing significant low-level reflectivity at longer ranges in the
low-layer product and high-level reflectivity at closer ranges in the high-layer product.
• 8-level products have course resolution.
APR:
• The performance of the APR product is reliant on the application of appropriate clutter filtering.
• Algorithm assumes all low-level data within 45 km (22 nm) of the RDA and below 1 km above radar level is
clutter. This will result in meteorological data being removed at times.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-70

FMH-11-Part C


> **Figure 2-14a: Layer Composite Reflectivity Average Low-Level Product (LRA #63).**

> An example Layer Composite Reflectivity Average Low-Level product (OPUP display) from the Saint

An example Layer Composite Reflectivity Average Low-Level product (OPUP display) from the Saint
Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013. Figures 2-14a through 2-14g are at the
same time for comparison purposes.

2-71

October 2017


> **Figure 2-14b: Layer Composite Reflectivity Average Mid-Level Product (LRA #64).**

> An example Layer Composite Reflectivity Average Mid-Level product (OPUP display) from the Saint

An example Layer Composite Reflectivity Average Mid-Level product (OPUP display) from the Saint
Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-72

FMH-11-Part C


> **Figure 2-14c: Layer Composite Reflectivity Average High-Level Product (LRA #89).**

> An example Layer Composite Reflectivity Average High-Level product (OPUP display) from the Saint

An example Layer Composite Reflectivity Average High-Level product (OPUP display) from the Saint
Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-73

October 2017


> **Figure 2-14d: Layer Composite Reflectivity Maximum Low-Level Product (LRM #65).**

> An example Layer Composite Reflectivity Maximum Low-Level product (OPUP display) from the Saint

An example Layer Composite Reflectivity Maximum Low-Level product (OPUP display) from the Saint
Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-74

FMH-11-Part C


> **Figure 2-14e: Layer Composite Reflectivity Maximum Mid-Level Product (LRW #66).**

> An example Layer Composite Reflectivity Maximum Mid-Level product (OPUP display) from the Saint

An example Layer Composite Reflectivity Maximum Mid-Level product (OPUP display) from the Saint
Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-75

October 2017


> **Figure 2-14f: Layer Composite Reflectivity Maximum High-Level Product (LRM #90).**

> An example Layer Composite Reflectivity Maximum High-Level product (OPUP display) from the

An example Layer Composite Reflectivity Maximum High-Level product (OPUP display) from the
Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-76

FMH-11-Part C


> **Figure 2-14g: Layer Composite Reflectivity Anomalous Propagation Removed Product (LRM**

> #67).

#67).
An example Layer Composite Reflectivity Anomalous Propagation Removed product (OPUP display)
from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-77

October 2017


#### 2.6.4 User-Selectable Layer Composite Reflectivity Maximum.

The User-Selectable Layer Composite Reflectivity Maximum product (ULR) is polar gridded and allows the operator to choose any layer (1,000 ft or more in depth) from the surface to 21.3 km (70,000 ft). The layer is a composite reflectivity maximum.
2.6.4.1 Operational Characteristics.
System ID: ULR, Product #137.
Data: Composite reflectivity (dBZ).
Processing:
• Maximum reflectivity selection
• Layer Composite algorithm
Availability: Updated once per volume scan.
Presentation: Polar coordinate image of layer composite reflectivity values (Figure 2-15).
Resolution: 1 km (0.54 nm) x 1°.
Coverage: Radar centered, 230 km (124 nm) radius.
Data Levels: 16.
Annotations:
• Standard set*
• Calibration constant (scaling constant used by the PSP to calculate reflectivity)
• Composite method (maximum)
• Data level code
• Layer boundary (vertical depth)
• Maximum reflectivity value detected
Adaptation: Layer Boundaries.
2.6.4.2 Usage. Similar to the LRM usage, but with the flexibility of the operator specifying the layer heights and
thickness. This flexibility enhances monitoring storm growth, severe storm warning operations, and forecasts
of aircraft icing and convective storms.
2.6.4.3 Strengths/Applications.
• The layer can be selected to meet user needs; can be as thin as 1,000 ft.
• Has higher resolution by using polar coordinates and more data levels than standard LRM products.
• Can be used to locate and monitor the bright band.
• Can be used to monitor aircraft icing and storm growth.
• Product is a constant altitude layer similar to a Constant Altitude Plan Position Indicator (CAPPI) and can be
used to monitor features at a fixed altitude instead of a sloping surface (i.e., base products).
• Can be used to display reflectivity above certain thresholds used by, for example, the
• Hail Index algorithm.
2.6.4.4 Limitations. Height of data within selected layer is unavailable.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-78

FMH-11-Part C


> **Figure 2-15: User-Selectable Layer Reflectivity Maximum Product (ULR #137).**

> An example User-Selectable Layer Reflectivity Maximum product (OPUP display) from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

An example User-Selectable Layer Reflectivity Maximum product (OPUP display) from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-79

October 2017


#### 2.6.5 Echo Tops.

The Echo Tops product provides the estimated height of the 18.5 dBZ (default) return, as estimated by the Echo
Tops algorithm, rounded to the nearest 5,000 ft MSL for display. The absence of qualifying echoes is reflected in
a color graphic product stating “NO ECHOES DETECTED.”
2.6.5.1 Operational Characteristics.
System ID: ET, Product #41.
Data: Height value of highest (in altitude) data-point in thousands of feet MSL meeting the minimum reflectivity
value of 18.5 dBZ (adaptable parameter).
Processing: Vertically Integrated Liquid/Echo Tops algorithm.
Availability: Once per volume scan.
Presentation: Cartesian image of echo top height information (Figure 2-16).
Resolution: 4 x 4 km (2.2 x 2.2 nm) at 5,000-ft interval
Coverage: Radar centered, 230 km (124 nm) radius 5,000 to 70,000 ft maximum in 5,000- ft increments in vertical.
Data Levels: 16.
Annotations:
• Standard set*
• Data level code
• Maximum data value detected (height in feet, MSL)
Adaptation: None.
2.6.5.2 Usage.
• Primary use of the product is to identify those storms with greater vertical development.
• Echo Tops heights are useful as part of briefings prepared for aviation interests and the general public.
• Can aid the user in defining the storm updraft flank, a strong updraft region or the presence of vertical updraft
tilt within a storm.
• Observation of collapsing echo tops can aid in timing the onset of a severe weather event.
2.6.5.3 Strengths/Applications.
• Assist in discriminating AP and other forms of non-precipitation echoes from precipitation.
• Aids in identification of storm structural features.
• May indicate elevated echo before any low-level echo is detected.
2.6.5.4 Limitations.
• There is no correction for data contamination from sidelobes which may result in overestimated tops. There is
also no correction for the effects of beam broadening with range; nor for the “stair-step” product appearance
due to the echo truncation at radar beam center-line and the use of fixed elevation angle sampling.
• There is no upward extrapolation from the last elevation where echo was detected.
• An echo top height is frequently incorrectly estimated because the VCP sampling is such that the true echo top
lies in the vertical gap between successive elevation scans or it is above the highest elevation scan. The latter
condition will frequently be true for storms within the cone of silence near the radar.
• Owing to beam broadening with range, discreet elevation angle sampling, and variation in the actual refractive
index of the atmosphere, the echo top estimates are inaccurate, often by one or more product data level increments (5,000 ft).
• Echo Top heights can differ significantly from visual cloud top heights.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.
2-80

FMH-11-Part C


> **Figure 2-16: Echo Tops Product (ET #141).**

> An example Echo Tops product (OPUP display) from the Oklahoma City, OK (KTLX) WSR-88D at

An example Echo Tops product (OPUP display) from the Oklahoma City, OK (KTLX) WSR-88D at
23:09 UTC on 31 May 2013.

2-81

October 2017


#### 2.6.6 High Resolution Enhanced Echo Tops.

The High Resolution Enhanced Echo Tops product (EET) provides the echo top information, but with linear interpolation providing finer vertical resolution as compared to ET. The result is a product with 1,000 ft increments in
the vertical. The horizontal resolution is also finer with the use of a polar coordinate instead of a Cartesian coordinate. Data values are referenced to MSL.
2.6.6.1 Operational Characteristics.
System ID: EET, Product #135.
Data: Height value, reached through linear interpolation, of highest (in altitude) sample volume point in thousands of feet MSL meeting the minimum reflectivity value of 18.5 dBZ (default).
Processing: Horizontal and vertical interpolation of reflectivity, Digital High Resolution Enhanced Echo Tops
algorithm and DQA.
Availability: Updated once per volume scan.
Presentation: Polar coordinate image of echo top heights (Figure 2-17).
Resolution: 1 km (0.54 nm) x 1° at 1,000-ft interval
Coverage: Radar centered, 345 km (186 nm) radius 1,000 to 70,000 ft in 1,000-ft increments in vertical.
Data Levels: 199.
Annotations:
• Standard set*
• Maximum data value detected (height in feet, MSL)
• Echo top threshold value (in dBZ)
• Number of artifact edited radials in volume
• Digital Data Level to EET conversion information
• Number of spurious points removed
• Maximum elevation angle of VCP (accounts for AVSET [see Section 4.2.11])
Adaptation: Color levels.
2.6.6.2 Usage. Provides the user with an indication of the upper boundary of significant reflectivity. The product is
designed to provide improved vertical estimates of precipitation tops and is superior to the ET product for aviation
and other interests.
2.6.6.3 Strengths/Applications.
• 1,000 ft vertical resolution vs. 5,000 ft vertical resolution for ET.
• Finer horizontal resolution (1° x 1 km (0.54 nm)) in the radar framework of polar coordinates vs. the ET product which is on a Cartesian grid and has a resolution of 4 x 4 km (2.2 x 2.2 nm).
• Assist in discriminating AP and other forms of non-precipitation echoes from precipitation.
• Aids in identification of storm structural features.
• May indicate elevated echo before any low-level echo is detected.
2.6.6.4 Limitations.
• There is no correction for data contamination from side lobes; which may result in overestimated tops. There is
also no correction for the effects of beam broadening with range.
• An echo top height is frequently incorrectly estimated because the VCP sampling is such that the true echo
top lies in the vertical gap between successive elevation scans or it is above the highest elevation scan. The
latter condition will frequently be true for storms within the cone of silence and the product will indicate it is
“topped”. Vertical interpolation attempts to correct for this, but these errors are still present to some extent.
2-82

FMH-11-Part C

•
•
•

Owing to beam broadening with range and variation in the actual refractive index of the atmosphere, echo top
estimates are inaccurate, often by 5,000 ft or more.
Echo Top heights can differ significantly from visual cloud top heights.
All volume products should be used with caution at ranges within the cone-of-silence and at far ranges where
only upper portions of weather are scanned and low-level echoes would be missed.

*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-83

October 2017


> **Figure 2-17: High Resolution Enhanced Echo Tops Product (EET #135).**

> An example Enhanced Echo Tops product (OPUP display) from the Oklahoma City, OK (KTLX) WSR88D at 23:09 UTC on 31 May 2013.

An example Enhanced Echo Tops product (OPUP display) from the Oklahoma City, OK (KTLX) WSR88D at 23:09 UTC on 31 May 2013.

2-84

FMH-11-Part C


#### 2.6.7 Hail Index.

For each storm cell identified by the SCIT algorithm, the Hail Index product (HI) provides: a) the probability the
storm will produce severe-sized (1.9 cm (¾ in) or larger) hail (POSH), b) the probability that the storm will produce
hail of any size (POH), and b) the maximum expected size of that hail (MEHS). The HI product is produced by the
Hail Detection Algorithm (HDA). The HDA uses environmental temperature information and the distribution and
magnitude of reflectivity to determine the hail probabilities and size.
2.6.7.1 Operational Characteristics.
System ID: HI, Product #59.
Data:
• Output of Hail Detection Algorithm (percent, inches).
• Meteorological algorithm output
Processing: Hail Detection Algorithm.
Availability: Hail probabilities and size are computed for each SCIT-identified storm and updated once per volume
scan.
Presentation:
• Alphanumeric annotation to other products.
• Formatted table of alphanumeric values (Figure 2-18)
• Special hail symbols overlaid on the identified storm on any geographically-based image product
• HDA adaptable parameters in a tabular alphanumeric display
Coverage: Hail probabilities and size will be computed for storms within 230 km (124 nm) of the radar.
Data Levels: 5. Probabilities are provided in increments of 10% with values between 0% and 100%. Hail sizes are
provided in increments of 0.6 cm (0.25 in) up to a maximum of 10 cm (4 in); estimated hail sizes over 10 cm (4
in) are displayed as >4.00. Any storm cells beyond the coverage range are given probabilities and a size of “UNKNOWN”.
Annotations:
• Standard set*
• Site adaptable parameters
• Storm ID
Special Symbol: The hail symbol is a large or small green isosceles triangle, filled or unfilled depending on
adaptable probabilities of severe hail and probabilities of hail. In addition, the maximum expected hail size
rounded to the nearest inch is displayed in the middle of or next to the triangle. If the rounded MEHS is 0, an “*”
is displayed.
Adaptation: See Section 3.2.4.
2.6.7.2 Usage.
• Provides the user with estimated probabilities of hail and severe hail and estimated maximum hail size.
• Selected hail probabilities and sizes can be used to generate alerts.
2.6.7.3 Strengths/Applications.
• The HI product has shown a very high probability of detection in cells that contain severe hail, especially greater than one-inch diameter hail. A POSH of 50% has the best skill as a warning threshold.
• Storms actually producing very large hail are often correctly indicated as producing such hail by the algorithm.

2-85

October 2017

2.6.7.4 Limitations.
• For the HDA to perform well, it needs as algorithm input, accurate and timely measurements of the MSL altitudes for the 0° C and -20° C levels.
• Values of POH, POSH, and MEHS will fluctuate at close ranges, especially in Volume Coverage Pattern (VCP)
21, due to gaps in coverage at higher elevations.
• Values of POH, POSH, and MEHS lack accuracy at long ranges due to the broad radar beam and limited elevation angles sampling the storm.
• POSH and MEHS may be overestimated in weak wind and tropical environments.
• For storms beyond 230 km (124 nm), HI will indicate hail as unknown.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-86

FMH-11-Part C


> **Figure 2-18: Hail Index Product (HI #59).**

> An example Hail Index product (OPUP display) from the Oklahoma City, OK (KTLX) WSR-88D at

An example Hail Index product (OPUP display) from the Oklahoma City, OK (KTLX) WSR-88D at
23:09 UTC on 31 May 2013.

2-87

October 2017


#### 2.6.8 Hybrid Scan Reflectivity.

The Hybrid Scan Reflectivity products are a display of the reflectivity values used in the conversion from reflectivity to rainfall rate (Hybrid Scan, see Section 3.2.11.1). Reflectivity for each azimuth and range is obtained from
one of the four lowest tilts. In addition, the data has undergone a series of quality control steps, including corrections for beam blockage, spurious noise, outliers, ground returns, and for the change in beam altitude with range.
2.6.8.1 Operational Characteristics.
System ID: HSR, Product #33; DHR, Product #32.
Data: Reflectivity (dBZ).
Processing: Precipitation Processing System.
Availability: Once per volume scan.
Presentation: Polar coordinate image of reflectivity values (Figure 2-19, a-b).
Resolution: 1 km (0.54 nm) x 1°.
Coverage: Radar centered, 230 km (124 nm) radius of radar.
Data Levels:
• 256 (#32)
• 16 (#33)
Annotations:
• Standard set*
• Maximum Data Value (dBZ)
Adaptation: Algorithm variables.
2.6.8.2 Usage. Permits the user the ability to see the graphic product from which Precipitation Processing System
(PPS) precipitation accumulations are calculated.
2.6.8.3 Strengths/Applications.
• Permits view of reflectivity used in PPS products.
• Help assess the accuracy of PPS products.
• Quick view of inconsistencies within the PPS products.
• Assists in discriminating between precipitation returns due to AP and clutter residue.
2.6.8.4 Limitations.
• The bin selected for inclusion in the hybrid scan product is highly dependent upon the output of the REC algorithm and its ability to correctly identify returns from ground clutter. Therefore, limitations listed for the CLR
product are applicable to the hybrid scan reflectivity product. The results must be used with caution in areas
where there is range folded/overlaid data.
• Ground clutter and AP are sometimes in the product display.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-88

FMH-11-Part C


> **Figure 2-19a: Hybrid Scan Reflectivity Product (HSR #33).**

> An example Hybrid Scan Reflectivity product (OPUP display) from the Saint Louis, MO (KLSX) WSR88D at 12:38 UTC on 18 April 2013. Figures 2-19a and 2-19b are at the same time for comparison

An example Hybrid Scan Reflectivity product (OPUP display) from the Saint Louis, MO (KLSX) WSR88D at 12:38 UTC on 18 April 2013. Figures 2-19a and 2-19b are at the same time for comparison
purposes.

2-89

October 2017


> **Figure 2-19b: Digital Hybrid Scan Reflectivity Product (DHR #32).**

> An example Digital Hybrid Scan Reflectivity product (OPUP display) from the Saint Louis, MO (KLSX)

An example Digital Hybrid Scan Reflectivity product (OPUP display) from the Saint Louis, MO (KLSX)
WSR-88D at 12:38 UTC on 18 April 2013.

2-90

FMH-11-Part C


#### 2.6.9 Vertically Integrated Liquid.

The Vertically Integrated Liquid Water product (VIL) displays VIL values as an image. The output of the VIL algorithm (Section 3.10) is used to produce the product that is updated once per volume scan.
2.6.9.1 Operational Characteristics.
System ID: VIL, Product #57.
Data: VIL values in kg m-2
Processing: VIL algorithm.
Availability: Once per volume scan.
Presentation: Cartesian image of VIL values (Figure 2-20). The product is displayable in full or quarter screen
format.
Resolution: 4 x 4 km (2.2 x 2.2 nm).
Coverage: Radar centered, 230 km (124 nm) radius.
Data Levels: 16. Values greater than 70 kg m-2 are truncated to 70 kg m-2
Annotations:
• Standard set*
• Data level code
• Maximum VIL value
Adaptation: Maximum data level code.
2.6.9.2 Usage.
• VIL is very useful in monitoring the general radar echo pattern for the beginning stages of significant convective development and for helping to distinguish thunderstorms from rain showers. As convective development
progresses, relative values of VIL are useful in distinguishing strong, possibly severe, thunderstorms from those
not likely to be severe.
• Significantly high (“significantly” will vary with different meteorological conditions) VIL values should be reviewed in conjunction with other severe weather guidance products (e.g., HI, MD, and TVS to assess the severe
weather likelihood of a given storm).
2.6.9.3 Strengths/Applications.
• Locate the most significant storms.
• Aids judging storm strength.
• Useful for distinguishing storms producing large hail once the hail thresholds have been established.
• Persistent high VIL values associated with supercells.
• Rapid decrease in VIL values may signify the onset of wind damage.
2.6.9.4 Limitations.
• VIL values are biased by drop size.
• Values for warnings may change daily and across the warning area.
• Values are air mass dependent.
• Values within 37 km (20 nm) of radar are underestimated due to the cone of silence.
• Grid-Based VIL values will differ from Cell-Based VIL values.
• VIL values for a strongly tilted or a fast moving storm will be lower than if the storm was vertical.
• The presence of hail, beam overshooting of low-level precipitation, beam broadening with range, and non-vertically contiguous VCPs may all affect the accuracy of the VIL estimation for a given storm.
2-91

October 2017

•
•
•
•

The relationship of VIL to severe weather needs to be quantified for different meteorological conditions and
climatic regimes.
May be contaminated by non-meteorological echoes.
VIL fluctuates widely as a storm’s range changes, especially with VCP 21.
Values at distant ranges (> 185 km (100 nm)) may be unreliable.

*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-92

FMH-11-Part C


> **Figure 2-20: Vertically Integrated Liquid Water Product (VIL #57).**

> An example Vertically Integrated Liquid Water product (OPUP display) from the Oklahoma City, OK

An example Vertically Integrated Liquid Water product (OPUP display) from the Oklahoma City, OK
(KTLX) WSR-88D at 23:09 UTC on 31 May 2013. Figures 2-20 and 2-21 are at the same time for
comparison purposes.

2-93

October 2017


#### 2.6.10 High Resolution Digital Vertically Integrated Liquid.

The High Resolution Digital Vertically Integrated Liquid (DVL) product was developed for the FAA in order to
show a finer resolution in the lower VIL values rather than higher values.
2.6.10.1 Operational Characteristics.
System ID: DVL, Product #134.
Data: VIL values kg m-2
Processing: Digital High Resolution VIL Algorithm and DQA
Availability: Once per volume scan.
Presentation: Polar coordinate image of VIL values (Figure 2-21).
Resolution: 1 km (0.54 nm) x 1°.
Coverage: Radar centered, 460 km (248 nm) radius.
Data Levels: 256.
Annotations:
• Standard set*
• Maximum VIL value
• Number of artifact edited radials in volume
• Digital Data Level to VIL conversion information
• Maximum elevation angle of VCP (accounts for AVSET)
Adaptation: Data level code.
2.6.10.2 Usage.
• VIL is very useful in monitoring the general radar echo pattern for the beginning stages of significant convective development and for helping to distinguish thunderstorms from rain showers. As convective development
progresses, relative values of VIL are useful in distinguishing strong, possibly severe, thunderstorms from those
not likely to be severe.
• Significantly high (“significantly” will vary with different meteorological conditions)
• VIL values should be reviewed in conjunction with other severe weather guidance products (such as, HI, MD,
and TVS to assess the severe weather likelihood of a given storm).
• High data resolution on the lower end of the VIL range will be helpful to the FAA and aviation community.
2.6.10.3 Strengths/Applications.
• Available out to 460 km (248 nm) from the radar.
• Locate the most significant storms.
• Higher data resolution than that provided by VIL.
• Useful for distinguishing storms producing large hail once the hail thresholds have been established.
• Persistent high VIL values associated with supercells.
• Rapid decrease in VIL values may signify the onset of wind damage.
• Could indicate VIL of elevated echo before any low-level echo is detected.
2.6.10.4 Limitations.
• VIL values are biased by drop size.
• Values for warnings may change daily and across the warning area.
• Values are air mass dependent.
2-94

FMH-11-Part C

•
•
•
•
•
•
•
•
•

Values within 37 km (20 nm) of the radar are underestimated due to the cone of silence.
High resolution VIL values will differ from the Grid-based VIL values and from Cell- Based VIL values (Figure
2-53b).
VIL values for a strongly tilted or a fast moving storm will be lower than if the storm was vertical.
The presence of hail, beam overshooting of low-level precipitation, beam broadening with range, and non-vertically contiguous VCPs may all affect the accuracy of the VIL estimation for a given storm.
The relationship of VIL to severe weather needs to be quantified for different meteorological conditions and
climatic regimes.
May be contaminated by non-meteorological echoes.
VIL fluctuates widely with range especially with VCP 21.
Values at distant ranges (> 185 km (100 nm)) may be unreliable.
All volume products should be used with caution at ranges within the cone-of-silence and at far ranges where
only upper portions of weather are scanned and low-level echoes would be missed.

*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-95

October 2017


> **Figure 2-21: High Resolution Digital Vertically Integrated Liquid Product (DVL #134).**

> An example High Resolution Digital Vertically Integrated Liquid Water product (OPUP display) from

An example High Resolution Digital Vertically Integrated Liquid Water product (OPUP display) from
the Oklahoma City, OK (KTLX) WSR-88D at 23:09 UTC on 31 May 2013. Figures 2-20 and 2-21 are
at the same time for comparison purposes.

2-96

FMH-11-Part C


#### 2.6.11 Storm Structure.

The Storm Structure product (SS) provides, for each identified storm cell, information regarding the structure of
the storm cell, including trend information. This contains information from the SCIT algorithm and HDA. Part of
this product is in a tabular alphanumeric format; the other part forms the basis of the Trends display. The Trends
display consists of time plots of algorithm derived storm cell and hail attributes. The Trends display can be selected
for any storm cell identified by SCIT and displays up to the previous 10 volume scans of attributes. For each cell,
the attributes in the Trends display are the top height, base height, height of the maximum reflectivity, probability
of hail, probability of severe hail, cell-based VIL, maximum reflectivity, and centroid (mass-weighted center) height.
All adaptable parameters used as inputs to the algorithm to generate data for this product are available in an alphanumeric display
2.6.11.1 Operational Characteristics.
System ID: SS, Product #62.
Data: Output from the SCIT algorithm and HDA. Processing: SCIT algorithm and HDA. Availability: Once per
volume scan.
Presentation: The graphics part of the product is the Trends Display. The Trends Display contains four graphs in
time of storm cell and hail attributes. The Trends display can be selected for any storm cell identified by the SCIT
algorithm and displays up to the previous 10 volume scans of attributes. For each cell, the attributes in the Trends
display are the top height, base height, height of the maximum reflectivity, probability of hail, probability of severe
hail, cell-based VIL, maximum reflectivity, and centroid height (Figure 2-22b).
• Alphanumeric Display (Figure 2-22a).
This product is displayable only as a table of storm cell attributes containing the following information for up to an
adaptable number of storm cells identified by the SCIT algorithm.
• Storm Cell ID
• Current Storm Position in (AZRAN) degrees and nm to the nearest integer from the RDA
• Storm base in kft
• Storm top in kft
• Cell-Based VIL in kg/m2
• Maximum Reflectivity in dBZ
• Height of Maximum Reflectivity in kft
• Number of Storm Cells
By default, all storm cells (entries), up to a maximum of 100, can be displayed in this format. However, the
MSCF operator has the capability to limit the number of storm cells included in this format from 10 to 100.
Coverage: Radar centered, 460 km (248 nm) radius of the radar.
Data Levels: Alphanumeric values (units vary).
Annotations:
• Standard set*
• Number of storms detected.
Adaptation: Algorithm parameters, see Section 3.2.7.
2.6.11.2 Usage.
• Provides user with additional information regarding the character of individually identified thunderstorms.
• Monitoring the time history of storm characteristics may aid in determining when a change in intensity occurs
and aid in forecasting the onset of a severe weather event.
2-97

October 2017

•
•

Monitoring the value and altitude of maximum reflectivity may give insight as to where a thunderstorm is in its
life cycle. Caution may be exercised in height fluctuations. These can occur due to gaps in the VCP used.
Maximum reflectivity and storm top may be used to generate alerts.

2.6.11.3 Strengths/Applications.
• Trends product is generated from SS.
• Related data included in CAT.
2.6.11.4 Limitations.
• Storm tops higher than at the highest elevation angle are identified as “unknown.”
• Echo feature heights (e.g., storm top and maximum reflectivity) are subject to errors owing to beam width,
placement, refraction, and gaps in the VCP used.
• When storm cells are in close proximity, portions of cells can become interchanged.
• See Section 3.2.7.4.3 for more limitation information.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-98

FMH-11-Part C


> **Figure 2-22a: Storm Structure Alphanumeric Tabular Listing (SS #62).**

> An example Storm Structure Alphanumeric Tabular Listing (OPUP display) from the Oklahoma City,

An example Storm Structure Alphanumeric Tabular Listing (OPUP display) from the Oklahoma City,
OK (KTLX) WSR-88D on 31 May 2013 at 23:09 UTC.

2-99

October 2017


> **Figure 2-22b: Storm Structure Trend Display (SS #62).**

> An example cell trends display for the Storm Structure product (OPUP display) from the Oklahoma

An example cell trends display for the Storm Structure product (OPUP display) from the Oklahoma
City, OK (KTLX) WSR-88D on 31 May 2013 at 23:55 UTC. The STI product identified and tracked a
very strong storm.

2-100

FMH-11-Part C


#### 2.6.12 Storm Tracking Information.

The Storm Tracking Information product (STI) is the output of the Storm Cell Identification and Tracking (SCIT)
algorithm. It provides the user with information about the past, current, and forecast positions of thunderstorms. It
can be produced in a tabular format of alphanumeric values, as a standalone graphic product, or as a graphic overlay
to other products. The algorithm is run each volume scan and correlates the current position of each storm cell
with the past history of all storms, thereby updating the track of each storm. A storm for which no past position is
found is identified as new and assigned a new unique storm ID. All site adaptable parameters identified as inputs to
the algorithms used to generate the data for this product are also available.
2.6.12.1 Operational Characteristics.
System ID: STI, Product #58.
Data: Storm tracking and trends.
Processing: SCIT algorithm output.
Availability: Once per volume scan.
Presentation:
• Table of storm cell attributes including for each cell the storm ID, storm position (AZ/RAN, in degrees and nm),
direction and speed (FCST MVT in degrees and kts), tracking errors (ERR/MEAN in nm and nm), maximum reflectivity (DBZM, in dBZ), and height of the maximum reflectivity (HGT, in kft). This table can be displayed with other
products (Figure 2-23c).
• Graphic display of past, current, and forecast storm cell tracks. This graphics display can be overlaid on other
products (Figure 2-23, a-b).
• Alphanumeric list of SCIT adaptable parameters.
Coverage: Radar centered, 460 km (248 nm) radius.
Data Levels: N/A.
• Each algorithm identified storm cell is given a unique alphanumeric storm ID. Storm IDs contain two characters, a letter (A to Z) followed by a one digit number (0 to 9). Storm IDs are recycled every 260 storm
cells.
• The past and future position interval and number of positions are adaptable parameters and can vary over a
range of 5 to 60 minutes (in 5-minute increments). The default interval is 15 minutes.
Annotations:
• Standard set*
• Total number of identified storms
Special Symbols: For each storm, past positions are shown as small (5-pixel diameter) white filled circles and
forecast positions are white plus (+) marks of similar size. The current position is a circle (5-pixel diameter)
within which is an “X.” If the storm speed is less than an adaptable parameter (default is 2.5 m/s), no past or
forecast positions are displayed, and the current location will have a circle around the “X.” A white line
connects past, current, and forecast positions.
Adaptation: See Section 3.2.7.
2.6.12.2 Usage.
• Monitor the position and movement of identified storms.
• Provide an estimate of the future movement of identified storms.
• Forecast movement can be used to generate alerts for storms not currently within an alert box but projected to
be (see User Alert Message, Section 2.58).
2-101

October 2017

2.6.12.3 Strengths/Applications.
• Product works best for well-defined and widely separated storm cells.
• Improved performance with VCP 12 due to more sampling at lower elevations.
• A large number of past tracks or future forecast positions for a particular cell indicate a more reliable track.
• The STI product is useful as an overlay on base and volume products.
• Cell motion is used in storm-relative velocity products (SRR, SRM).
• Cell attributes are critical inputs to HI and SS products.
2.6.12.4 Limitations.
• Errors may occur in the identification of cells and in the calculations of attributes especially when cells are
located near one another.
• Unrepresentative movements are possible due to irregular development/dissipation of cells.
• The product does not provide movement of areas of showers.
• Actual storm motion exceeding certain thresholds (Section 3.4) will lead to misidentification of storms and very
large errors in motion.
• Forecast tracks are always straight lines because they are based on a linear extrapolation of past tracks, therefore
storms traveling a curvilinear path or changing direction will not be accurately forecast.
• Intended for application to well defined, isolated thunderstorms; results in other applications will be uncertain.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-102

FMH-11-Part C


> **Figure 2-23a: Storm Tracking Information Product (STI #58).**

> An example Storm Tracking Information product (OPUP display) from the Oklahoma City, OK (KTLX)

An example Storm Tracking Information product (OPUP display) from the Oklahoma City, OK (KTLX)
WSR-88D on 31 May 2013 at 23:09 UTC.

2-103

October 2017


> **Figure 2-23b: Storm Tracking Information Product (STI #58).**

> An example Storm Tracking Information product (OPUP display) overlaid with a Reflectivity Data Array product from the Oklahoma City, OK (KTLX) WSR-88D on 31 May 2013 at 23:09 UTC.

An example Storm Tracking Information product (OPUP display) overlaid with a Reflectivity Data Array product from the Oklahoma City, OK (KTLX) WSR-88D on 31 May 2013 at 23:09 UTC.

2-104

FMH-11-Part C


> **Figure 2-23c: Storm Tracking Information Combined Attribute Table (STI #58).**

> An example Storm Tracking Information Combined Attributed Table product (OPUP Display) from the

An example Storm Tracking Information Combined Attributed Table product (OPUP Display) from the
Oklahoma City, OK (KTLX) WSR-88D on 31 May 2013 at 23:09 UTC.

2-105

October 2017


### 2.7 Velocity-Derived Products.


#### 2.7.1 Storm-Relative Mean Radial Velocity.

The Storm-Relative Mean Radial Velocity products (SRR and SRM) provide an estimate of the mean radial velocity
for: (a) a small geographic area centered upon or near an identified storm with the storm motion removed (Region),
or (b) the entire area of radar coverage (to 124 nm (230 km)) with the mean storm motion removed (Map). The
velocity displayed is based on the maximum value contained in each one of the 0.25 km (0.13 nm) gates of the two
or four gates, respectively, contained in the product data resolution. The product is produced upon request for any
elevation angle available. The radial component of storm motion used to derive the product is the storm motion
value computed for the identified cell by the SCIT algorithm or a value input by the user. The value of storm motion used to adjust the mean radial velocity values is user-selectable at the time of product request or it defaults to
the vector average of all identified storms if not selected. Each product contains 16 data levels for storm-adjusted
mean radial velocity.
2.7.1.1 Operational Characteristics.
System ID: SRR (Region), Product #55; SRM (Map), Product #56.
Data:
• Computed motion of a single storm from Storm Cell Identification and Tracking (SCIT) algorithm (m/s)
• Storm-relative mean radial velocity (m/s).
Processing:
• SCIT algorithm
• Remove radial component of computed storm motion from the mean radial velocity values.
• Use location for generation when specified (for SRR)
Availability: Upon request for any elevation angle.
Presentation: Polar coordinate image of derived mean radial velocity values (Figures 2- 24, a-b).
Resolution:
• SRR: 0.5 km (0.27 nm) x 1°
• SRM: 1 km (0.54 nm) x 1°
Coverage:
• SRR: Product is centered at the location of the storm center, 50 x 50 km (27 x 27 nm) window within 230 km
(124 nm) of the radar.
• SRM: Product is radar centered, 230 km (124 nm) radius.
Data Levels: 16.
Annotations:
• Standard set*
• Coordinates of product center (SRR)
• Data level code
• Elevation angle
• Maximum data value detected (after storm motion removed)
• Vector average of storm motion (all storms for SRM, single storm for SRR, or operator defined)
Adaptation: Storm motion (optional); Center Point (SRR).

2-106

FMH-11-Part C

2.7.1.2 Usage.
• As an aid in the visual identification of mesocyclones, TVSs, shear regions, or divergence in the mean radial
velocity field when signatures are being confused by addition of storm motion component. Note that a constant
is removed from the radial velocity field. Thus, radial velocity shear is not changed.
• For a line of thunderstorms or a convective system where storm motions are similar, an average storm motion
(computed or user input) can be removed (SRM).
2.7.1.3 Strengths/Applications.
• Provides 3D velocity structure of a storm when used as in a four-panel (or all-tilts) display.
• An aid to human recognition of mesocyclones, TVSs, shear regions, and divergence.
• Most useful with faster moving storms and in radar coverage areas where the storm motion is largely along the
radials.
• Operator may input storm motion for both SRM and SRR.
• When using SRR the displayed max and minimum velocities can more easily be located in the product window.
• SRR is a useful alert paired product for the Mesocyclone Detection and TVS.
• SRR has better resolution than does the SRM.
2.7.1.4 Limitations.
• If the computed storm motion is significantly in error, or a vector average of storms moving in a widely divergent pattern is used, storm rotational signatures may still be confused.
• Is inappropriate when ground-relative winds are desired.
• Average storm-relative motion will vary from volume scan to volume scan, especially for the SRM product.
• SRR is confined to a limited viewing area.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-107

October 2017


> **Figure 2-24a: Storm-Relative Mean Radial Velocity Map Product (SRM #56).**

> An example Storm-Relative Mean Radial Velocity Map product (OPUP display) at the 0.5° elevation

An example Storm-Relative Mean Radial Velocity Map product (OPUP display) at the 0.5° elevation
angle from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18.

2-108

FMH-11-Part C


> **Figure 2-24b: Storm-Relative Mean Radial Velocity Region Product (SRR #55).**

> An example Storm-Relative Mean Radial Velocity Region product (OPUP display) at the 0.5° elevation angle from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18.

An example Storm-Relative Mean Radial Velocity Region product (OPUP display) at the 0.5° elevation angle from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18.

2-109

October 2017


#### 2.7.2 SuperOb.

The SuperOb product (SO) contains 8-bit precision radial velocity data as an average of all base velocity bins within
discrete sampling areas for a predetermined time span. Each elevation scan is divided into multiple sampling areas,
or SuperOb cells. Velocity data are averaged within these SuperOb cells for each elevation of the VCP. If the VCP
changes during the time span, thereby causing a change in elevation angles, a tolerance is established so that elevation differences beyond the tolerance will cause velocity data for the new elevation(s) to be averaged separately.
Elevation differences within the tolerance value cause velocity data from the new elevation(s) to be included in the
averages for the nearest elevation. If a SuperOb cell does not meet this minimum number of samples within the
time span, an average is not computed for that cell.
2.7.2.1 Operational Characteristics.
System ID: SO, Product #136.
Data:
• Average radial velocity
• Root mean square of the average radial velocities
• Azimuth, latitude, longitude, height, and time deviation.
Processing: Time spans are established as are sample sizes and velocity averages taken.
Availability: The time span over which the velocity data are averaged is adaptable between 10 minutes and 3 hours
with the default value being 1 hour. The product is generated at the end of the time span.
Presentation: The product is not displayed in WFOs
Resolution and Coverage: The resolution and range extent of the SuperOb cells are determined from RPG
adaptation data. Adaptation data defines a range size (default 2.7 km (5 km)) and an azimuth size (default 6 degrees). Adaptation data also defines a maximum range for processing (default 100 km (54 nm)).
• Product is contained within 230 km (124 nm) of the radar.
Data Levels: 256.
Annotations:
• Standard set*
• Data level code
• Elevation angle
• Maximum data values detected
• Product center point coordinates.
Adaptation:
• SuperOb cells range size and azimuth size.
• Minimum number of samples
• Time span over which the velocity data are averaged.
2.7.2.2 Usage. The product is designed for NCEP and use in numerical models.
2.7.2.3 Strengths/Applications. Permits WSR-88D wind data to be ingested and used in numerical weather prediction models.
2.7.2.4 Limitations. Velocities are computed for large volumes.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.
2-110

FMH-11-Part C


#### 2.7.3 Velocity Azimuth Display and Velocity Azimuth Display Wind Profile.

The output from the Velocity Azimuth Display (VAD) algorithm is used to create two products; the Velocity Azimuth Display (VAD) and the Velocity Azimuth Display Wind Profile (VWP).
The VAD product is a graphical presentation of the VAD algorithm processing logic for the particular altitude.
(Figure 2-25b) This graphical presentation is a data plot of mean radial velocity values versus azimuth angle for one
specific reporting altitude along with the best-fit sine wave curve that is used to compute the horizontal wind speed
and direction. This horizontal wind speed and direction data are used to build the VWP product.
The VWP product is a graphic display of wind barbs plotted on a height scale in 1,000-foot increments. (Figure
2-25a) The current plot (far right) and up to 10 previous plots may be displayed simultaneously on a time versus
height scale. Wind speed and direction for up to 30 altitudes are displayed as wind barbs on a height scale. All altitudes are referenced to mean sea level. Wind speed and direction are reported to the highest altitude with sufficient
signal available for processing by the VAD algorithm. If the VAD derived wind estimate at a given height is invalid
(i.e., failed threshold for RMS, symmetry, or number of points), winds for that height is reported as “ND”.
The VWP product also includes an alphanumeric (tabular format) portion. This table provides specific information
pertaining to each wind entry for up to 52 VAD derived winds. These 52 entries include the selected VWP altitudes
(up to 30), an additional one wind per elevation at a constant slant range (up to 20), and the low altitude supplemental winds (2). Wind entries are ordered by increasing altitude. Note: The vertical velocity (W) and divergence (DIV)
columns contains valid data only for constant slant range wind estimates. For all other estimates, these fields contain
“NA”, indicating not applicable.”
2.7.3.1 Operational Characteristics.
System ID: VAD, Product # 84; and VWP, Product #48.
Data: VAD and VWP: wind speed (KT) and direction (degrees).
Processing: VAD algorithm.
Availability:
• The VAD product is available for any user-specified altitude that is generated by the most current VWP.
• The VWP is generated routinely and updated once per volume scan.
Presentation:
• The VAD is a graphic plot of mean radial velocity versus azimuth angle. The best-fit sine wave is overlaid on the
plot of the velocity points (Figure 2-25b).
• The VWP is a graphic display of wind barbs plotted on a height staff. If the wind is calm, i.e., sufficient echoes
are present but the speed is less than 5 knots, a circle of 5 pixels in diameter is placed at the appropriate location. The current and up to 10 previous plots may be displayed simultaneously along a time axis (Figure
2-25a).
Resolution: Nearest 5 kt.
Data Levels:
• 30 VWP altitudes, and 8 VAD
• 8 VAD color levels
• 5 RMS levels for VWP
Annotations:
• For the VWP product:
▪ Standard set*
▪ Data level code
2-111

October 2017

▪ Maximum wind speed and direction for the current plot
▪ Nyquist Co-interval
▪ Site adaptable parameters
• For the VAD product:
▪ Standard Annotations
▪ Slant Range
▪ Elevation Angle
▪ Wind Speed and Direction
▪ Root Mean Square (RMS) Error
▪ Computed Altitude of Wind (MSL)
Special Symbols:
• For the VAD product, the velocity data are plotted as single points on a grid of velocity versus azimuth. The
best-fit function is plotted over the field of velocity points as a linked vector in a contrasting color.
• For the VWP product, the wind speed and direction are plotted using the standardmeteorological wind barb
presentation.
Adaptation:
• Beginning and ending azimuth for analysis
• Reporting altitudes
• Data symmetry threshold
• Root mean square error
2.7.3.2 Usage.
• VAD-derived winds are included in the RCM for national distribution. The profiles from multiple radars can
then be composited and streamline analysis for large areas can be accomplished.
• The algorithm can diagnose the mean wind using less than 360° azimuth. This allows a determination of winds
even when there is no measurable reflectivity through large segments of azimuth. The VAD for a specific computed wind is available if the user suspects the computed value or wishes to analyze the data to a greater level of
detail. The VAD can be used to understand qualities of the VWP product.
• The VAD can be used to determine convergence or divergence.
• The VWP product provides a timely determination of the boundary layer wind profile. Time versus height
profile plots can be generated allowing the user to keep track of significant changes due to advection or other
significant meteorological mechanisms.
• For a VWP reporting altitude for which wind data are missing, VAD can be used to establish why wind data are
not being reported. It can also provide the necessary information for adjusting adaptation data, or VAD analysis
range and elevation angle, in order to increase the likelihood of obtaining wind data in low reflectivity environments.
2.7.3.3 Strengths/Applications. VAD:
• VAD winds are available in Clear Air or Precipitation Mode.
• The VAD algorithm does not require 360 degrees of data.
• Check missing or suspicious wind data on the VWP.
• Update Environmental Winds Table. The VAD winds are fed into the Environmental Winds Table for use in

2-112

FMH-11-Part C

the velocity dealiasing algorithm. This helps minimize dealiasing errors.
VWP:
• The VWP may be of assistance in many operations such as in severe weather, aviation, hydrology, and general
forecasting.
• The VWP can be used to create/adjust hodographs.
2.7.3.4 Limitations.
VAD:
• Needs sufficient data points (adaptable)
• May be unreliable in disturbed environments.
• Large flocks of migrating birds may produce anomalous wind data.
• Available for pre-established altitudes only as designated at the RPG HCI for the VWP.
VWP:
• Measurable returns needed - at least 25 data points are required on the individual VAD for data to be encoded at
that altitude.
• Winds are not encoded if RMS error or symmetry thresholds are exceeded. ND will be plotted if RMS exceeds

### 9.7 kts or symmetry exceeds 13.6 kts regardless of data levels.

• Generally only representative of winds within 37 km (20 nm) of the RDA.
• Birds can bias VWP values significantly, especially migratory birds.
• Deformations in the wind field (e.g., fronts, troughs) over the area of analysis can bias the analysis.
• When the area of analysis is reduced by performing the analysis at higher elevation angles and shorter ranges,
more care must be taken to account for the vertical component of the Doppler-derived mean radial velocity, especially in areas of precipitation.
• It is necessary to provide the algorithm with symmetry, RMS, and data point thresholds for bounding the data
before analysis in order to force the algorithm to disregard spurious data.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-113

October 2017


> **Figure 2-25a: Velocity Azimuth Display Wind Profile Product (VWP #48).**

> An example Velocity Azimuth Display Wind Profile product (OPUP display) from the Saint Louis, MO

An example Velocity Azimuth Display Wind Profile product (OPUP display) from the Saint Louis, MO
(KLSX) WSD-88D on 18 April 2013 at 14:59 UTC shows the wind estimates produced in the vertical
for the last 11 volume scans. “ND” indicates levels where the algorithm was not able to produce a
reliable wind estimate.

2-114

FMH-11-Part C


> **Figure 2-25b: Velocity Azimuth Display Product (VAD #84).**

> An example Velocity Azimuth Display product (OPUP display) from the Saint Louis, MO (KLSX) WSD88D on 18 April 2013 at 14:59 UTC.

An example Velocity Azimuth Display product (OPUP display) from the Saint Louis, MO (KLSX) WSD88D on 18 April 2013 at 14:59 UTC.

2-115

October 2017


#### 2.7.4 Mesocyclone Detection.

The Mesocyclone Detection products provide information regarding the existence and nature of vortices associated
with thunderstorms. This product is derived from the Mesocyclone Detection Algorithm (MDA) which is separate
and independent from the legacy Mesocyclone algorithm; although, the MD product is analogous to the legacy Mesocyclone (M) product. These products provide information regarding identified cyclonic shear circulation features.
Circulations are assigned a strength rank number based on the rotational velocity of each of its component 2D
features. The strength rank values range from 1 (the weakest) to 25 (the strongest).
The product is generated in a format that provides an alphanumeric tabular display for all identified circulations
simultaneously, a graphic display, or a graphic overlay to other products. If there is no output from the MDA for a
particular volume scan, a version of the product that exhibiting the null condition is produced.
DMD “Detection Status”:
• TOP (Topped): When the radar has scanned an elevation over the feature without adding any 2D components
to the feature.
• EXT (Extrapolated): A 3D feature found in the previous volume but not yet time associated with any feature
from the current volume. The extrapolated feature position provided in the product is an estimated position
based on the time between volume scans and the feature’s tracked motion. If no tracked motion for the feature
is available, the SCIT storm motion from the previous volume scan is used.
• UPD (Updated): A 3D feature detected in both the current and previous volume scans but not yet topped.
2.7.4.1 Operational Characteristics.
System ID: MD, Product #141; DMD, Product #149.
Data: Mesocyclone detections of varying strength, sorted by strength ranks, then by Mesocyclone Strength
Index (MSI); Meteorological algorithm output.
Processing: Mesocyclone Detection Algorithm and MD Tracking Information.
Availability: Updated once per volume scan.
Presentation:
• Formatted table of alphanumeric values (MD only) (Figure 26a)
• Graphic display of circulation symbols at computed positions
• Graphic overlay of circulation symbol at its computed position (Figures 26, a-b)
Coverage: Radar centered, 230 km (124 nm) radius.
Data Levels: 3
• Circulations with strength rank less than 5
• Circulations with strength rank greater than or equal to 5
• Circulations with strength rank greater than or equal to 5 and either detected on the lowest elevation, or with a
base height at or below 1 km
Annotations:
• Standard set*
• Combined Attribute Table
• Circulation ID, Closest Storm ID, Strength Rank, Speed/Direction of movement, Base, and Depth
Special Symbols: MD:
• The symbol for a circulation having a strength rank less than 5 is a thin, yellow, open circle centered at the position of the circulation’s base 2D component. The diameter of the circle is based on the diameter of the circulation but with a minimum diameter of 14 pixels. Note that this particular symbol will only be displayed if the
default adaptation data setting for the Minimum Display Filter Rank is lowered from its default value of 5.
2-116

FMH-11-Part C

•
•
•
•

The symbol for a circulation having strength rank greater than or equal to 5 is a thick, yellow, open circle centered at the position of the circulation’s base 2D component. The diameter of the circle is based on the diameter of the circulation but with a minimum diameter of 14 pixels.
The symbol for a circulation with a strength rank value greater than or equal to 5 that extends to the lowest elevation angle or has a base less than 1 km is a thick, yellow circle with spokes. The diameter of the circle is based
on the diameter of the circulation, but with a minimum diameter of 14 pixels.
Past position symbols are indicated by solid diamonds. Up to 10 past positions can be included for a given circulation.
Forecast position symbols are indicated by Xs. Up to 6 forecast positions can be included for a given circulation
but the number of forecast positions will never exceed the number of past positions. The forecast position
interval is 5 minutes.

DMD: None.
Adaptation: MD:
• Minimum Reflectivity, default 0 dBZ.
• Minimum Display Filter Rank, default 5.
• Overlap Display Filter, default Yes (meaning, apply filter that suppresses elevated circulations that overlap lower
circulations).
DMD: None.
2.7.4.2 Usage. As an aid in the identification, alerting, and warning for mesocyclones which very often (>90%)
are correlated with severe weather (hail, damaging winds, and tornadoes). It appears that ~20% are associated with
tornadoes. Research has shown that a strength rank of 5 most closely matches the MESO threshold in the legacy
Mesocyclone algorithm.
As discussed in more detail in Section 3.7.4 and seen in Figure 2-26a, a Mesocyclone Strength Index (MSI) is calculated for each 3D feature. In calculating the MSI, the strength ranks of all 2D feature components are multiplied
by 1,000, weighted by the average air density in a standard atmosphere, and integrated vertically across the halfpower beam width depth at the height of the 2D feature. Integration is done from the feature’s base to its top or
THRESHOLD (maximum 3D Couplet Core Top), whichever is lower in altitude. The integrated value is divided by
the total depth (with half-power beam width added).
2.7.4.3 Strengths/Applications.
• Identify mesocyclones. Like all algorithms, the MDA is imperfect, so the operator must examine reflectivity,
velocity, and/or SRM to verify the existence of mesocyclones.
• Identify mid-level mesocyclones that develop downward toward the surface sometimes becoming associated
with tornadoes.
2.7.4.4 Limitations.
• Time continuity is not required for feature identification. However, the display of past positions easily identifies
those circulations with time continuity between two or more volume scans.
• At longer ranges, mesocyclones may not be detected due to poor aspect ratios, random beam placement during
radar sampling, and the majority of the 3-D circulations being below the height of the lowest elevation angle.
• The operator does not know which base-data elevation angle to examine for detection verification. Algorithm
output is in terms of height and not scan angle. However, circulations detected on the lowest angle are indicated
by circles with spokes.
• Range folding and velocity aliasing may interfere with successful mesocyclone detection. Velocity aliasing can
itself induce false algorithm detections.
• See Section 3.2.9 for a discussion of the operational considerations of the MDA.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.
2-117

October 2017


> **Figure 2-26a Mesocyclone Detection Product (MD #141).**

> An example Mesocyclone Detection product (OPUP display) overlaid on a Velocity Data Array product

An example Mesocyclone Detection product (OPUP display) overlaid on a Velocity Data Array product
from the Oklahoma City, OK (KTLX) WSR-88D on 31 May 2013 at 23:09 UTC. Identification numbers
for identified circulations are included.

2-118

FMH-11-Part C


> **Figure 2-26b: Digital Mesocyclone Detection Product (DMD #149).**

> An example Digital Mesocyclone Detection product (OPUP display) overlaid on a Velocity Data Array

An example Digital Mesocyclone Detection product (OPUP display) overlaid on a Velocity Data Array
product from the Oklahoma City, OK (KTLX) WSR-88D on 31 May 2013 at 23:09 UTC.

2-119

October 2017


#### 2.7.5 Tornado Vortex Signature.

The Tornado Vortex Signature product (TVS) provides the TVS ID; TVS location; low-level and maximum shear;
average shear; lowest altitude and depth associated with each detected signature. The product is provided as an overlay to imagery products or as an alphanumeric list of data.
2.7.5.1 Operational Characteristics.
System ID: TVS, Product #61.
Data:
• Output of Tornado Detection Algorithm
• TVS attribute table
Processing: Tornado Detection Algorithm.
Availability: Updated each volume scan.
Presentation:
• Formatted table of alphanumeric values (Figure 2-27)
• Graphic symbol overlay at TVS position (Figure 2-27)
Coverage: Radar centered, 460 km (248 nm) radius.
Data Levels: TVS and Elevated TVSs (ETVSs).
Annotations:
• Standard set*
• Site adaptable parameters
Special Symbol: The TVS symbol is an inverted, red-filled isosceles triangle with a 3/16 inch base and a 3/8
inch altitude. The elevated TVS symbol is an open red triangle with similar dimensions.
Adaptation: See Section 3.2.9.
2.7.5.2 Usage. As an aid in the identification, forecasting, and warning of severe weather associated with tornadoes.
2.7.5.3 Strengths/Applications.
• Multiple velocity thresholds make it possible to detect peak shear within broader regions of strong shear.
• The TVS product aids in identification of potentially tornadic circulations within or outside of mesocyclones.
• The algorithm, through adaptable parameters, can make it possible to fine-tune the product for differing environments and storm types.
2.7.5.4 Limitations.
• Adaptable parameters need more research to better fine-tune the algorithm and product performance.
• The algorithm and therefore the product have high false-alarm rates.
• Insufficient research has been done on ETVSs. The ROC recommends leaving the adaptable parameter value
for the number of ETVSs displayed at zero.
• Algorithm and product introduce additional uncertainty.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-120

FMH-11-Part C


> **Figure 2-27: Tornado Vortex Signature Product (TVS #61).**

> An example Tornado Vortex Signature product (OPUP Display) overlaid on a Reflectivity Data Array

An example Tornado Vortex Signature product (OPUP Display) overlaid on a Reflectivity Data Array
product from the Oklahoma City, OK (KTLX) WSR-88D on 31 May 2013 at 23:09 UTC. The corresponding alphanumeric product is also displayed.

2-121

October 2017


#### 2.7.6 Tornado Vortex Signature Rapid Update.

The Tornado Vortex Signature Rapid Update product (TRU) is generated to provide updated TDA information.
The TDA data for each elevation scan is checked for vertical continuity with data from the elevations that have been
completed thus far in the current volume scan. This information is combined with TDA and Storm Tracking Information from the previous volume scan to form the TRU product.
2.7.6.1 Operational Characteristics.
System ID: TRU, Product #143.
Data:
• TRU output
• TRU attribute table.
Processing: TDA, MDA, and Storm Track Algorithms.
Availability: Updated upon completion of each elevation scan.
Presentation:
• Formatted table of alphanumeric values (Figure 2-28).
• Graphic display of TVS symbol and feature symbols at computed positions (Figure 2-28).
• Graphic overlay of TVS symbol at its computed position.
• With the following exceptions, the format of the TRU graphic attribute and alphanumeric tabular portions of
the product will follow the non-rapid update TVS product: feature status will be reported as EXT, PER, INC,
and, NEW to denote extrapolated, persistent, increasing, and new features, respectively; and the character ^
(hexadecimal value 5E) will be placed next to attributes that were computed from current volume scan detections.
Coverage: Radar centered, 460 km (248 nm) radius.
Data Levels:
• TVS
• ETVS
• Extrapolated features (EXT)
• Persistent (PER)
• Increasing (INC)
• New Feature (NEW)
Annotations: Standard set*.
Adaptation: See Section 3.2.9.4.
2.7.6.2 Usage.
Provides TVS updates with the completion of each elevation angle scanned thereby enabling the operational meteorologist to make warning and other related decisions before the end of the radar volume scan.
2.7.6.3 Strengths/Applications.
• Intermediate algorithm output is available before end of volume scan.
• TRU tracks features in order to check time continuity.
2.7.6.4 Limitations.
• Classification as INC or PER may be the result of sampling issues versus an actual feature attribute change.
• The TRU graphical attribute table and alphanumeric attribute table contain attributes from both previous and
current volume scan information.
2-122

FMH-11-Part C

•
•

Insufficient research has been done on ETVSs. The ROC recommends leaving the adaptable parameter value
for the number of ETVSs displayed at zero.
Feature matching ability dependent on motion supplied by the SCIT algorithm which may not represent the
actual movement of the velocity signature.

*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-123

October 2017


> **Figure 2-28: Tornado Vortex Signature Rapid Update Product (TRU #143).**

> An example Tornado Vortex Signature Rapid Update product (OPUP Display) overlaid on a Reflectivity Data Array product from the Oklahoma City, OK (KTLX) WSR-88D on 31 May 2013 at 23:09 UTC.

An example Tornado Vortex Signature Rapid Update product (OPUP Display) overlaid on a Reflectivity Data Array product from the Oklahoma City, OK (KTLX) WSR-88D on 31 May 2013 at 23:09 UTC.
The corresponding alphanumeric product is also displayed.

2-124

FMH-11-Part C


### 2.8 Dual Polarization-Derived Products.


#### 2.8.1 Hydrometeor Classification.

The Hydrometeor Classification products (HC and DHC) are estimates of the most probable dominant scatterer
type in each sample volume. Currently, the possible hydrometeor classifications are:
Biological (BI)
Ground Clutter (GC)
Ice Crystals (IC)
Dry Snow (DS)
Wet Snow (WS)
Rain (RA)
Heavy Rain (HR)
Big Drops (BD)
Graupel (GR)
Rain mixed with Hail (HA)
Unknown (UK)
2.8.1.1 Operational Characteristics.
System ID: HC, Product #164; DHC, Product #165.
Data: Enumerated hydrometeor classification values.
Processing: A fuzzy logic and dual polarization parameter input weighting scheme determines the hydrometeor
classification for each sample volume from a subset of allowable classifications. Allowable classifications are determined by several hard rules and by the proximity of the sample volume to the melting layer. Melting layer information is obtained from the Melting Layer Detection Algorithm (MLDA, see Section 3.3.1) (Also, see Melting Layer
[ML], Section 2.8.4). If no allowable classification receives a fuzzy logic score that meets a minimum threshold, a
classification of unknown is assigned to the sample volume.
Availability: Once per elevation.
Presentation: A polar grid of hydrometeor classifications (Figure 2-29, a-b).
Resolution:
• 0.25 km (0.13) x 1° (#165)
• 1 km (0.54 nm) x 1° (#164)
Coverage: All hydrometeor classification products are radar centered.
• 300 km (162 nm) radius (#165)
• 230 km (124 nm) radius (#164)
Data Levels: 12
Annotations: Standard set*.
Adaptation: None.
2.8.1.2 Usage.
In combination with other products, useful in determining the dominant hydrometeor type for each sample volume
and to interrogate dual polarization-based precipitation accumulation products.

2-125

October 2017

2.8.1.3 Strengths/Applications.
• Delineation between precipitation and non-precipitation echoes.
• Identification of regions of possible hail.
2.8.1.4 Limitations.
• Highly dependent upon an accurate melting layer height estimation from the Melting Layer Detection Algorithm.
• Noise in the base dual polarization parameter data fields and/or radar calibration problems can negatively impact hydrometeor classification results.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-126

FMH-11-Part C


> **Figure 2-29a Hydrometeor Classification Product (HC #164).**

> An example Hydrometeor Classification product (OPUP display) from the Saint Louis, MO (KLSX)

An example Hydrometeor Classification product (OPUP display) from the Saint Louis, MO (KLSX)
WSR- 88D at 12:38 UTC on 18 April 2013. Figures 2-29a and 2-29b are at the same time and elevation angle for comparison purposes.

2-127

October 2017


> **Figure 2-29b: Digital Hydrometeor Classification Product (DHC #165).**

> An example Digital Hydrometeor Classification product (OPUP display) from the Saint Louis, MO

An example Digital Hydrometeor Classification product (OPUP display) from the Saint Louis, MO
(KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-128

FMH-11-Part C


#### 2.8.2 Hybrid Hydrometeor Classification.

The Hybrid Scan Hydrometeor Classification product (HHC) provides hydrometeor classifications used by the QPE
(see Section 3.3.3) algorithm to compute precipitation rate. This product is formatted as a digital data array and is
assembled from the lowest elevation range bins which are not flagged as being ground clutter or anomalous propagation or blocked by more than x percent, i.e. those bins used to compute dual polarization precipitation rates.
2.8.2.1 Operational Characteristics.
System ID: HHC, Product #177.
Data: A polar grid of hydrometeor classification enumerated values used in the computation of QPE precipitation
rates.
Processing: Once QPE determines that a precipitation rate can be computed for a given bin, the hydrometeor
classification associated with that bin is stored in the HHC product. The elevation-based hydrometeor classification data used to create this hybrid product (see DHC, Section 2.30) is first smoothed by a 9-bin mode filter.
Availability: Once per volume scan.
Presentation: A polar coordinate image of hydrometeor classification types (Figure 2-30).
Resolution: 0.25 km (0.13 nm) x 1°.
Coverage: Radar centered, 230 km (124 nm) radius.
Data Levels: 256.
Annotations: Standard set*
Adaptation: None.
2.8.2.2 Usage. Provides the user with a graphical means to determine the hydrometeor classification for each sample volume used by the QPE algorithm to estimate precipitation accumulation.
2.8.2.3 Strengths/Applications. Used in the generation of external products for monitoring flash flood potential,
basin estimated precipitation, etc.
2.8.2.4 Limitations. The bin selected for inclusion in the hybrid scan product is highly dependent upon the output
of the dual polarization Hydrometeor Classification Algorithm (HCA) and its ability to correctly categorize returns.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-129

October 2017


> **Figure 2-30 : Hybrid Scan Hydrometeor Classification Product (HHC #177).**

> An example Hybrid Scan Hydrometeor Classification product (OPUP display) from the Saint Louis,

An example Hybrid Scan Hydrometeor Classification product (OPUP display) from the Saint Louis,
MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-130

FMH-11-Part C


#### 2.8.3 Digital Instantaneous Precipitation Rate.

The Digital Instantaneous Precipitation Rate product (DPR) provides a hybrid scan polar grid of instantaneous
QPE precipitation rate values formatted as a digital data array. The product is assembled from the lowest elevation
range bins that can be used to compute a precipitation rate.
2.8.3.1 Operational Characteristics.
System ID: DPR, Product #176.
Data: The DPR is a hybrid scan polar grid of instantaneous QPE precipitation rates to the nearest 0.001 in/hr.
Processing: The QPE algorithm receives dual polarization moments and hydrometeor classes from the HCA
and uses this data to compute instantaneous QPE precipitation rates, if possible. QPE uses the lowest elevation
possible to compute these rates. Once a precipitation rate has been computed for a given bin, it is put into the
DPR product and its corresponding hydrometeor class is put into the HHC product.
Availability: Once per volume scan.
Presentation: A polar coordinate image of rates (Figure 2-31).
Resolution: 0.25 km (0.13 nm) x 1°.
Coverage: Radar centered, 230 km (124 nm) radius.
Data Levels: 65,536.
Annotations: Standard set*
Adaptation: Algorithm variables.
2.8.2.3 Usage. Provides the hybrid scan rate array for use by forecasters for monitoring flash-flooding and flooding,
for now-casting, and for investigating the accuracy of dual polarization precipitation estimates and for use by applications external to the WSR-88D.
2.8.2.3 Strengths/Applications. Used in the generation of external products for monitoring flash flood potential,
basin estimated precipitation, etc.
2.8.2.4 Limitations.
• The bin selected for inclusion in the hybrid scan product is highly dependent upon the accuracy of the dual
polarization HCA and the MLDA.
• The product can contain typically very small areas of erroneously high rates most likely caused by misclassification by the HCA.
• Echoes which are not precipitation reaching the ground, such as virga or ground clutter, can contaminate the
product, although less than in PPS products.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-131

October 2017


> **Figure 2-31: Digital Instantaneous Precipitation Rate Product (DPR #176).**

> An example Digital Instantaneous Precipitation Rate product (OPUP display) from the Saint Louis,

An example Digital Instantaneous Precipitation Rate product (OPUP display) from the Saint Louis,
MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-132

FMH-11-Part C


#### 2.8.4 Melting Layer.

The Melting Layer product (ML) is produced from the output of the Melting Layer Detection Algorithm (MLDA).
It is intended for use as an overlay on other radar products. It consists of four approximately concentric contours
about the radar that indicate the position of the radar beam relative to the melting layer top and bottom heights.
The concentricity can be deformed as (especially) fronts move across the radar domain. Keeping in mind the threedimensional reality represented by the two-dimensional radar display, the inner-most contour represents the location
at which the upper edge of the radar beam first intersects the bottom of the melting layer. The next two contours in
range represent the location where the center of the radar beam intersects the melting layer bottom and top, respectively. Finally, the outer-most contour shows the location where the lower edge of the radar beam intersects the top
of the melting layer.
2.8.4.1 Operational Characteristics.
System ID: ML, #166
Data: Contours representing the relative position (in range) of the radar beam with the melting layer top and
bottom heights.
Processing: The MLDA identifies regions of wet snow aloft to define the melting layer then averages over approximately 20 degrees in azimuth and over at least three volume scans of wet snow detections.
Availability: Once per elevation scan.
Presentation: Overlay product with contours (Figure 2-32).
Resolution: 0.13 nm (1/4 km) x 1°
Coverage: range to 230 km (124 nm).
Data Levels: 4 contours.
Annotations: Standard set*.
Adaptation: Flag to use radar data only, model data grid in conjunction with the MLDA data, or the single model
sounding nearest to radar location in conjunction with the MLDA data.
2.8.4.2 Usage. An overlay on other elevation-based radar products, particularly the HC and the QPE products.
2.8.4.3 Strengths/Applications.
• Input to the HCA for use in determining allowable hydrometeor types at each sample volume.
• Evaluation of HC and QPE products.
2.8.4.4 Limitations.
• Although the ML product itself has resolution to 0.13 nm (1/4 km), the MLDA output has a resolution of 0.54
nm (1 km).
• The MLDA requires that the number of wet snow detections reaches a threshold before making an estimate of
melting layer height. If insufficient wet snow detections are made, the algorithm outputs a default melting layer
height obtained from the 0° C height from the RAP model data.
• ML may be inaccurate during some weather conditions such as a melting layer above below freezing air or multiple melting layers.
• Only the first melting layer encountered coming down through the atmosphere is represented.
• The melting layer determination from the previous volume is overlaid on the current volume’s data.

2-133

October 2017


> **Figure 2-32 Melting Layer Product (ML #166).**

> An example Melting Layer product overlaid on a Correlation Coefficient product (OPUP display) from

An example Melting Layer product overlaid on a Correlation Coefficient product (OPUP display) from
the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-134

FMH-11-Part C


### 2.9 LegacyPrecipitation Estimation Products.


#### 2.9.1 One-Hour Rainfall Accumulation.

The One-Hour Rainfall Accumulation product (OHP) provides the hourly total rainfall accumulation on a 2 x 2 km
(1.1 x 1.1 nm) grid within the 230 km (124 nm) radius of the radar. The product is updated once per volume scan
by the legacy PPS algorithm to provide the precipitation accumulation for the last hour. The total is a running total.
If there is too much missing data this product is not generated. The DPA version of this product is produced for
use in computers external to the WSR-88D.
2.9.1.1 Operational Characteristics.
System ID: OHP, Product #78.
Data: Adjusted hourly radar rainfall accumulation estimates (in/hr).
Processing: Precipitation Processing System.
Availability: Once per volume scan.
Presentation: Polar coordinate image of rainfall accumulation values (Figure 2-33).
Resolution: 2 km (1.1 nm) x 1°.
Coverage: Radar centered, 230 km (124 nm) radius.
Data Levels: 16.
Annotations:
• Standard set*
• Date and End Time of Rainfall Integration
• Maximum Data Value
• Radar Bias Estimate
• Effective G-R Pair Sample Size
• Missing Periods
• Gage Adjustment Bias Flag
Adaptation: See Section 3.2.11.
2.9.1.2 Usage. Assess changes in precipitation intensity in time and space, and for flash flood warnings.
2.9.1.3 Strengths/Applications.
• Assess rainfall accumulations for flash flood watches, warnings, and statements.
• Can be used for Short Term Forecasts and Hazardous Weather Outlooks.
• Time lapse can provide movement and rainfall relative to river basins and other features.
• Other water management applications.
2.9.1.4 Limitations.
• After extended outages, first product will not be generated for 54 (adaptable) minutes.
• Although every effort has been made to remove non-precipitation reflectivity returns from the calculation of
precipitation, residual ground clutter or anomalous propagation may contaminate data.
• The rainfall estimation bias calculation is for the 230 km (124 nm) radius of coverage and may not be representative of one particular area of interest.
• See Section 3.2.11 for discussions regarding operational considerations of the algorithms.
• The appropriate Z-R relationship may be hard to estimate and vary within the storm.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.
2-135

October 2017


> **Figure 2-33: One-Hour Rainfall Accumulation Product (OHP #78).**

> An example One-Hour Rainfall Accumulation product (OPUP display) from the Saint Louis, MO

An example One-Hour Rainfall Accumulation product (OPUP display) from the Saint Louis, MO
(KLSX) WSR-88D at 11:56 UTC on 18 April 2013.

2-136

FMH-11-Part C


#### 2.9.2 Hourly Digital Precipitation Array.

The Hourly Digital Precipitation Array product (DPA) provides an hourly running total of radar-rainfall estimates
in an array format (not display oriented) to support processing performed external to the WSR-88D. The product
is updated once per volume scan by the PPS algorithm to provide the precipitation accumulation for the last hour.
The product is available in 256 data levels for each array element.
2.9.2.1 Operational Characteristics.
System ID: DPA, Product #81.
Data: Radar precipitation accumulation estimates (in/hr).
Processing: Precipitation Processing System.
Availability: Once per volume scan.
Presentation: Example in Figures 2-34, a-b.
Resolution:
• 1/40 x 1/40 Limited Fine Mesh (LFM) grid for accumulation data; 1/4 LFM grid for rate data
• 131 boxes x 131 boxes
Coverage: Radar centered, 230 km (124 nm) radius
Data Levels: 256, accumulation; 8, rate; no color level codes.
Annotations:
• Standard set*
• Supplemental Data, including
• End Date and End Time Hourly Accumulation
• Radar Bias Estimate
• Effective G-R Pair Sample Size
• Memory Span (Hours) used in Bias Estimate
• Total Number of Isolated Sample Volumes
• Total Number of Replaced and Interpolated Outliers
• Mean Percent Area Reduction of all Scans in Accumulation
• Mean Bi-scan Ratio of all Scans during Accumulation Period
• Number of Bad Scans in Current Hour
• Number of Outliers in Current Hour
• Current VCP Number
• Current Operational (Weather) Mode
• Missing Periods (Begin Date; Begin Time; End Date; End Time)
• Preprocessing Algorithm
Adaptation data
• Minimum Reflectivity Threshold
• Maximum Reflectivity Threshold
• Maximum Beam Blockage
• Maximum Likelihood of Clutter
• Maximum Range Bi-Scan
• Minimum Echo Area
2-137

October 2017

•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•

Minimum Area-Averaged Reflectivity
Maximum Area Percent Reduction
Z-R Multiplicative Coefficient
Z-R Power Coefficient
Minimum Reflectivity to convert to Rate
Maximum Reflectivity to convert to Rate
Exclusion Zones
Minimum Range Bi-Scan
Rate Algorithm
Maximum Storm Speed
Threshold Max Time Difference
Minimum Area Time Continuity
Time Continuity Parameter #1
Time Continuity Parameter #2
Maximum Rate Echo Area Change
Range Cut-Off
Range Effect Coefficient #1
Range Effect Coefficient #2
Range Effect Coefficient #3
Minimum Precip. Rate
Maximum Precip. Rate
Accumulation Algorithm
Threshold Elapsed Time to Restart
Maximum Time for Interpolation
Minimum Time in Hourly Period
Threshold Hourly Outlier
Ending Time Gage Accumulation
Maximum Period Accumulation Value
Maximum Hourly Accumulation Value
Adjustment Algorithm
Time Bias Estimation
Threshold Number of Gage-Radar Pairs
Reset Bias Value
Longest Allowable Lag (Hours)
Bias Applied Flag

Adaptation: See Section 3.2.11.
2.9.2.2 Usage.
• The DPA supports subjective and objective hydrometeorological forecast procedures for flash flood watches and warnings, river stage forecasting, water management applications, and other hydrometeorological
requirements for numerical use of precipitation data in computers external to the WSR-88D.
• Provides hourly running total digital radar-rainfall estimates in an array format to support processing performed external to the WSR-88D.
2-138

FMH-11-Part C

2.9.2.3 Strengths/Applications. Provides high resolution precipitation estimates for generation of external hydrological products such as flash flood assessment and warnings.
2.9.2.4 Limitations. Product size is very large.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.
*** ORPG DATABASE PRODUCT LOAD UTILITY ***
-> Number of Products Available=5881
-> Message ID=5750
-> Product Info: LBuffer# 108 MSGLEN 012692 VOLNUM 137 ELEV 04
WARNING: Layer selected exceeds 15 (the current limit), CVT accepts up to 30 WARNING: Number of Layers (16) is out of range (1-15).
CVT will accept up to 30 layers until a new limit is defined.
-> Set Processing ONLY for Layer Number 16 packet code 1 found
Packet 1: Write Text (No Value) Summary Information Length of Data Block (in bytes) = 3692
I Starting Point: 0 J Starting Point: 0 Message to follow:
ADAP(38) 0.90 50.00 50.00 50.00 99.70 -32.00 20.00 80.00 60.00

### 300.00 1.40 0.00 70.00 3.00 25.00 15.00 200.00 24.00 13.20


### 200.00 230.00 0.00 1.00 0.00 0.00 103.80 60.00 30.00 54.00


### 400.00 0.00 400.00 800.00 50.00 10.00 1.00 168.00 FBIAS(13)



> **Figure 2-34a: Hourly Digital Precipitation Array (Part I) (DPA #81).**


GAGE-RADAR MEAN FIELD BIAS TABLE
LAST BIAS UPDATE TIME: 08/22/98 16:12 BIAS APPLIED ? NO
MSPAN (HRS) NO. G_R PAIRS AVG. GAGE(MM) AVG. RADAR(MM) MEAN FLD BIAS 0.001 0.000 1.016 1.090 0.932

### 1.000 0.000 1.863 1.538 1.211


### 2.000 0.000 2.648 1.933 1.370


### 3.001 0.000 2.825 2.032 1.391


### 4.998 0.000 2.908 2.089 1.392


### 10.004 0.000 2.935 2.118 1.386


### 168.006 13.494 2.627 2.094 1.255


### 719.819 126.212 2.417 2.104 1.149


### 2160.295 212.154 2.381 2.109 1.129


### 9999044.000 277.982 2.365 2.112 1.120

SUPL(29)RATE SCAN 1 DATE: 12690 TIME:10624
RATE SCAN 2 DATE: 12690 TIME:10880
RATE SCAN 3 DATE: 12690 TIME:11264
RATE SCAN 4 DATE: 12690 TIME:11520
RATE SCAN 5 DATE: 12690 TIME:11904
RATE SCAN 6 DATE: 12690 TIME:12160
RATE SCAN 7 DATE: 12690 TIME:12544
RATE SCAN 8 DATE: 12690 TIME:12800
RATE SCAN 9 DATE: 12690 TIME:13184
RATE SCAN 10 DATE: 12690 TIME:13440
RATE SCAN 11 DATE: 12690 TIME:13824
RATE SCAN 12 DATE: 12690 TIME:14080
RATE SCAN 13 DATE: 12690 TIME:14336
RATE SCAN 14 DATE: 12690 TIME:14720 HOURLY ACCUMULATION END DATE.......: 12690 HOURLY ACCUMULATION
END TIME.......: 14400 TOTAL NO. OF BLOCKAGE BINS REJECTED: 0 TOTAL NO. OF CLUTTER BINS REJECTED.: 4707


> **Figure 2-34b: Hourly Digital Precipitation Array (Part II) (DPA #81).**

> 2.9.3 Three-Hour Rainfall Accumulation.


#### 2.9.3 Three-Hour Rainfall Accumulation.

The Three-Hour Rainfall Accumulation product (THP) provides the sum of the present plus two previous clock

2-139

October 2017

hours of rainfall accumulations from the PPS algorithm. It can be produced as often as once per hour for the previous three-hour period. This product is not generated if data for more than one of the clock hours used in computing the totals are missing.
2.9.3.1 Operational Characteristics.
System ID: THP, Product #79.
Data: Radar-estimated, three-hour total rainfall accumulations (inches).
Processing: Precipitation Processing System.
Availability: Once per volume scan but the accumulation data only changes once per hour.
Presentation: Polar coordinate image of rainfall accumulations (Figure 2-35).
Resolution: 2 km (1.1 nm) x 1°.
Coverage: Radar centered, 230 km (124 nm) radius.
Data Levels: 16.
Annotations:
• Standard set*
• Current rate bias estimate
• Ending date and time of precipitation rate integration
• Maximum three-hour precipitation accumulation detected
• Mean error variance of bias estimate for three-hour period
Adaptation: See Section 3.2.11.
2.9.3.2 Usage.
• Excessive three-hour rainfall accumulations can indicate a potential for flooding.
• Used in conjunction with a later OHP, the THP can delineate areas where basin saturation may have
already occurred and thereby provide guidance for estimating changes in the flooding threshold when using
the OHP.
2.9.3.3 Strengths/Applications.
• Provides a longer rainfall accumulation interval than OHP.
• For very long precipitation events, can be used with STP for storm totals analysis.
• Can be used for providing flash flood guidance.
2.9.3.4 Limitations.
• Data used to generate the product updated only once per hour.
• As a guide to current accumulation amounts, the hourly update rate may not be adequate. Products more
than 30 minutes old should be used with caution to estimate current three-hour rainfall amounts.
• Although every effort has been made to remove non-precipitation reflectivity from the calculation of precipitation, residual ground clutter or anomalous propagation may contaminate data.
• See Section 3.2.11 for discussions of the algorithm’s operational considerations.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-140

FMH-11-Part C


> **Figure 2-35: Three-Hour Rainfall Accumulation Product (THP #79).**

> An example Three-Hour Rainfall Accumulation product (OPUP display) from the Saint Louis, MO

An example Three-Hour Rainfall Accumulation product (OPUP display) from the Saint Louis, MO
(KLSX) WSR-88D at 13:59 UTC on 18 April 2013.

2-141

October 2017


#### 2.9.4 Storm Total Rainfall Accumulation.

The Storm Total Rainfall Accumulation products are displays of continuously updated radar-estimated rainfall accumulations within 230 km (124 nm) of the radar. The accumulations are from the PPS algorithm. The storm total
rainfall is defined as the total rainfall accumulation since the last one-hour break in significant rainfall over the total
area of coverage. These products are generated even when periods of missing data occur.
2.9.4.1 Operational Characteristics.
System ID: STP, Product #80; DSP, Product #138.
Data: Radar-estimated storm-total rainfall accumulations (inches).
Processing: Precipitation Processing System.
Availability: Updated once per volume scan.
Presentation: Polar coordinate image of rainfall accumulations (Figure 2-36, a-b).
Resolution: 2 km (1.1 nm) x 1°
Coverage: Radar centered, 230 km (124 nm) radius.
Data Levels:
• 256 (#138)
• 16 (#80)
Annotations:
• Standard set*
• Beginning and ending dates and times of precipitation rate integration
• Current rate bias estimate
• Current error variance of bias estimate
• Maximum data value.
Adaptation: See Section 3.2.11
2.9.4.2 Usage.
• Monitoring of total precipitation accumulation regardless of duration.
• Estimation of total basin run-off due to a single storm.
• Estimation of basin saturation due to previous rainfall events.
• Evaluation of flood reports.
2.9.4.3 Strengths/Applications.
• Monitor total precipitation accumulation.
• Estimate degree of soil saturation within river basin or area.
• Estimate river basin runoff.
• Post-storm analysis.
• Time lapse used to follow motion of storms relative to river basin location and orientation.
2.9.4.4 Limitations.
• Periods of missing data can be included without the knowledge of the operator, thus compromising data quality.
• Although every effort has been made to remove non-precipitation reflectivity from the calculation of precipitation, residual ground clutter or anomalous propagation may contaminate data.
2-142

FMH-11-Part C

See Section 3.2.11 for discussions regarding the algorithm’s operational considerations.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-143

October 2017


> **Figure 2-36a: Storm Total Rainfall Accumulation Product (STP #80).**

> An example Storm Total Rainfall Accumulation product (OPUP display) from the Saint Louis, MO

An example Storm Total Rainfall Accumulation product (OPUP display) from the Saint Louis, MO
(KLSX) WSR-88D at 21:00 UTC on 18 April 2013. Figures 2-36a and 2-36b are at the same time for
comparison purposes.

2-144

FMH-11-Part C


> **Figure 2-36b: Digital Storm Total Rainfall Accumulation Product (DSP #138).**

> An example Digital Storm Total Rainfall Accumulation product (OPUP display) from the Saint Louis, MO

An example Digital Storm Total Rainfall Accumulation product (OPUP display) from the Saint Louis, MO
(KLSX) WSR-88D at 21:00 UTC on 18 April 2013.

2-145

October 2017


#### 2.9.5 User-Selectable Rainfall Accumulation.

The User-Selectable Rainfall Accumulation product (USP) provides a radar-estimated rainfall accumulation map
displayed as an image, for a user-selected accumulation period, from the PPS algorithm. The product format and
content are the same as other PPS products, except the accumulation period is of variable duration (in whole
clock hours), ranging from a beginning to an ending time specified by the user.
2.9.5.1 Operational Characteristics.
System ID: USP, Product #31.
Data: Radar rainfall accumulation estimates (inches).
Processing: Precipitation Processing System.
Availability: Up to 10 different accumulation periods once per volume scan.
Presentation: Polar coordinate image of accumulated precipitation (Figure 2-37).
Resolution: 2 km (1.1 nm) x 1°.
Coverage: Radar centered, 230 km (124 nm) radius.
Data Levels: 16.
Annotations:
• Standard set*
• Data level code
• Maximum data values detected
• Times and dates of the beginning and end (clock hour) of the rainfall rate integration
• Mean-field bias in the radar estimate of the precipitation rate
• Effective sample size associated with the bias estimate
Adaptation: Algorithm parameters.
2.9.5.2 Usage. Used to estimate precipitation accumulation over a user-specified period.
2.9.5.3 Strengths/Applications.
• Flexible time interval to meet varying weather situations.
• In addition to the one-hour and the 24-hour default USP products, any others generated for dedicated users are
available by OTR to dial-up users.
2.9.5.4Limitations.
• USP accumulations are updated only at the top of the hour.
• Product may contain missing time periods. At least two-thirds of the specified hourly accumulations must be
available for product generation.
• Only 10 products can be generated per volume scan.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-146

FMH-11-Part C


> **Figure 2-37: User-Selectable Rainfall Accumulation Product (USP #31).**

> An example User-Selectable Rainfall Accumulation product (OPUP display) from the Saint Louis, MO

An example User-Selectable Rainfall Accumulation product (OPUP display) from the Saint Louis, MO
(KLSX) WSR-88D at 13:01 UTC on 18 April 2013. The duration for this example is two hours.

2-147

October 2017


#### 2.9.6 Supplemental Precipitation Data.

The Supplemental Precipitation Data product (SPD) provides selected supplemental data, generated or collected
during the operation of the PPS algorithm, in a compact array or an alphanumeric format (not a display-compatible format), or both. Up to ten different preselected sets of supplemental data can be requested by computers external to the WSR-88D unit using a single request code. The supplemental data are also appended
to the Hourly Digital Precipitation Array (Section 2.9.2).
2.9.6.1 Operational Characteristics.
System ID: SPD, Product #82.
Data:
• Number of isolated sample volumes
• Number of replaced and interpolated outliers (reflectivity factor)
• Number of interpolated outliers (hourly accumulation)
• Percent area reduction
• Bi-scan ratio
• Flag (bad scan)
• Missing period indicator (with beginning and ending times if missing period present)
• Computed bias estimate and its error variance, and flag (apply bias)
• GAGE-RADAR SET information
• The number of gage-radar pairs
• For each pair, the gage ID, its azimuth and range (from the radar), the hourly gage accumulation estimate and
matching radar accumulation estimate (in inches, to nearest
• .01 in), and an indicator as to whether the pair used in the BIAS determination
• Data from the RAIN GAGE DATABASE, including the present number of reporting gages, the date and time of
last data base update, the ID, latitude, longitude, azimuth, range, gage type (accumulator or incremental) and number
of reports for each gage
• For each report, the timestamp, value (in inches, to nearest .01in), and, for incremental- type gages, the duration (in
minutes)
Processing: Precipitation Processing System.
Availability: Updated each volume scan (Figure 2-38).
Data Levels: Most supplemental data are a single numeric value with the exception of the 1/4 LFM precipitation
rate grid, which is an 8-level data array.
Annotations:
• Standard set*
• Maximum data value
• Bias applied flag
• Bias estimate
• Effective # gage-radar pairs
• Memory Span (Hours) used in bias estimate
• Average scan date (last bias update)
• Average scan time (last bias update)
• Total number of blockage bins rejected
• Total number of clutter bins rejected
2-148

FMH-11-Part C

•
•
•
•
•
•
•

Total number of final bins smoothed
Hybrid scan percent bins filled
Hybrid scan total rain area
Begin missing period date
Begin missing period time
End missing period date
End missing period time

Adaptation: None.
2.9.6.2 Usage.
• Provides supplementary information regarding the performance of the PPS algorithm to computers external to the
WSR-88D.
• Primary use is for research and post analysis for system performance checks.
2.9.6.3 Strengths/Applications.
• Provides information on precipitation correction bias and other information on rain gage-radar pairs.
• Provides information on the number of isolated bins and outliers.
2.9.6.4 Limitations. None.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-149

October 2017


> **Figure 2-38 Supplemental Precipitation Data (SPD #82).**

> An example Supplemental Precipitation Data product from the Oklahoma City, OK (KTLX) WSR-88D

An example Supplemental Precipitation Data product from the Oklahoma City, OK (KTLX) WSR-88D
on 10 May 2013 at 05:23 UTC (AWIPS display).

2-150

FMH-11-Part C


### 2.10 Legacy Snowfall Estimation Products.


#### 2.10.1 One-Hour Snow Depth Accumulation.

The One-Hour Snow Depth Accumulation product (OSD) provides estimates of the snow depth accumulations
during the past running clock hour.
2.10.1.1 Operational Characteristics.
System ID: OSD, Product #145.
Data: Radar snow depth accumulation estimates (inches).
Processing: Snow Accumulation Algorithm.
Availability: Once per volume scan.
Presentation: Polar coordinate image of snow depth accumulation estimates (Figure 2-39).
Resolution: 1 km (0.54 nm) x 1°.
Coverage: Radar centered, 230 km (124 nm) radius
Data Levels: 16.
Annotations:
• Standard set*
• Beginning and ending dates and times of accumulations
• Data level codes
• SAA adaptable parameters and their values
• Maximum data value detected
• Azimuth and range of the maximum data value
• Range/height correction applied
• Missing periods of accumulations in minutes
Adaptation: See Section 3.2.12.
2.10.1.2 Usage. Provides estimated snow depth accumulation over last hour.
2.10.1.3 Strengths/Applications.
• Can be used to distinguish areas of high and low snowfall rates and trends and movement of these areas.
• Has adaptable parameters which have been optimized for part of the US which can be changed by the operator
• Snow depth estimates were intended for dry snow (i.e., snow which is not melting).
2.10.1.4 Limitations.
• Does not account for blowing or drifting snow.
• The same adaptable parameters are applied to the entire radar coverage, yet actual measured values (such as Z-S
relationship and snow ratio) will vary within a storm in time and space.
• The snow depth estimates are not accurate for snow that is melting (in the air or on the ground), but the product/algorithm does not distinguish between dry snow and other precipitation types.
• Accuracy of snow estimates is reduced for small periods of time such as 1 hour.
• Range of accurate estimates is at most 150 km, but can be less for very low-topped snow producing clouds such
as lake effect snow.
• Does not account for advection of snow while falling.
• Data levels are fixed and only provide a range of estimates (versus a digital estimate).
2-151

October 2017

*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-152

FMH-11-Part C


> **Figure 2-39: One-Hour Snow Depth Accumulation Product (OSD #145).**

> An example One-Hour Snow Depth Accumulation product (OPUP display) from the Duluth, MN

An example One-Hour Snow Depth Accumulation product (OPUP display) from the Duluth, MN
(KDLH) WSR-88D at 23:29 UTC on 15 March 2013.

2-153

October 2017


#### 2.10.2 One-Hour Snow Water Equivalent Accumulation.

The One-Hour Snow Water Equivalent Accumulation product (OSW) provides estimates of the water equivalent of
the snow that has fallen during the past running clock hour.
2.10.2.1 Operational Characteristics.
System ID: OSW, Product #144.
Data: Radar snow water equivalent accumulation estimates (in/hr).
Processing: Snow Accumulation Algorithm.
Availability: Updated once per volume scan.
Presentation: Polar coordinate image of snow water equivalent estimates (Figure 2-40).
Resolution: 1 km (0.54 nm) x 1.
Coverage: Radar centered, 230 km (124 nm radius).
Data Levels: 16.
Annotations:
• Standard set*
• Beginning and ending dates and times of accumulations
• Data level codes
• SAA adaptable parameters and their values
• Maximum data value detected
• Azimuth and range of the maximum data value
• Range/height correction applied
• Missing periods of accumulations in minutes
Adaptation: See Section 3.2.12.
2.10.2.2 Usage. Provides estimates of water equivalent of the snow accumulation during the past hour.
2.10.2.3 Strengths/Applications.
• Can be used to distinguish areas of light and heavy snow rates and trends, and movement of areas.
• Has regionally optimized adaptable parameters which can be changed by the operator.
• Can be used with lower reflectivity values than the PPS.
• Has a range/height correction.
2.10.2.4 Limitations.
• The same adaptable parameters are applied to the entire radar coverage, yet actual measured values (such as Z –
S relationship_ will vary in time and space.
• Accuracy of snow estimates is reduced for small periods of time, such as one hour.
• Range of accurate estimates is at most 150 km, but can be less for very low-topped snow producing clouds such
as lake effect snow.
• Does not account for advection of snow while falling.
• Data levels are fixes and only provide a range of estimates (versus a digital estimate)
• Does not account for blowing or drifting snow.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.
2-154

FMH-11-Part C


> **Figure 2-40: One-Hour Snow Water Equivalent Accumulation Product (OSW #144).**

> An example One-Hour Snow Water Equivalent Accumulation product (OPUP display) from the Duluth,

An example One-Hour Snow Water Equivalent Accumulation product (OPUP display) from the Duluth,
MN (KDLH) WSR-88D at 23:29 UTC on 15 March 2013.

2-155

October 2017


#### 2.10.3 Storm Total Snow Depth Accumulation.

The Storm Total Snow Depth Accumulation product (SSD) provides a running total of radar-estimated snowfall
depth. The product provides estimates since the snow accumulations were last manually reset to zero.
2.10.3.1 Operational Characteristics.
System ID: SSD, Product #147.
Data: Radar-estimated snow depth accumulation (inches).
Processing: Snow Accumulation Algorithm.
Availability: Once per volume scan.
Presentation: Polar coordinate image of snow depth estimates (Figure 2-41).
Resolution: 1 km (0.54 nm) x 1°.
Coverage: Radar centered, 230 km (124 nm) radius.
Data Levels: 16.
Annotations:
• Standard set*
• Beginning and ending dates and times of accumulations
• Data level code
• SAA adaptable parameters and their values
• Maximum data value detected
• Azimuth and range of the maximum data value
• Range/height corrections applied
• Missing periods of accumulations in minutes
Special Symbols: None.
Adaptation: See Section 3.2.12.
2.10.3.2 Usage. Provides estimated snow accumulations over storm duration.
2.10.3.3 Strengths/Applications.
• Can be used to determine the amount of snowfall even when and where no reports are available.
• Has regionally optimized adaptable parameters which can be changed by the operator.
• Can accumulate with lower reflectivity values than the PPS.
• Has a range/height correction.
2.10.3.4 Limitations.
• The same adaptable parameters are applied to the entire radar coverage area, yet actual measured values (such as
Z-S relationship and snow ratio) will vary in time and space.
• Snow depth estimates are not accurate for snow that is melting (in the air or on the ground), but the product does not distinguish between dry snow and other precipitation types.
• Range of accurate estimates is at most 150 km, but can be less for very low-topped snow producing clouds
such as lake effect snow.
• Does not account for advection of snow after being observed by the radar.
• Does not account for blowing or drifting snow.
• Data levels are fixed and only provide a range of estimates (versus a digital estimate).
2-156

FMH-11-Part C

*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-157

October 2017


> **Figure 2-41: Storm Total Snow Depth Accumulation Product (SSD #147).**

> An example Storm Total Snow Depth Accumulation product (OPUP display) from the Duluth, MN

An example Storm Total Snow Depth Accumulation product (OPUP display) from the Duluth, MN
(KDLH) WSR-88D at 02:13 UTC on 16 March 2013.

2-158

FMH-11-Part C


#### 2.10.4 Storm Total Snow Water Equivalent Accumulation.

The Storm Total Snow Water Equivalent Accumulation product (SSW) provides a running total of radar-estimated
snow water equivalent. The product provides estimates since the snow accumulations were last manually reset to
zero.
2.10.4.1 Operational Characteristics.
System ID: SSW, Product #146.
Data: Radar snow water equivalent accumulation estimates (inches).
Processing: Snow Accumulation Algorithm.
Availability: Updated once per volume scan.
Presentation: Polar coordinate image of snow water equivalent estimates (Figure 2-42).
Resolution: 1 km (0.54 nm) x 1°.
Coverage: Radar centered, 230 km (124 nm) radius.
Data Levels: 16.
Annotations:
• Standard set*
• Beginning and ending dates and times of accumulations
• Data level code
• SAA adaptable parameters and their values
• Maximum data value detected
• Azimuth and range of the maximum data value
• Range/height correction applied
• Missing periods of accumulations in minutes
Adaptation: See Section 3.2.12.
2.10.4.2 Usage. Provides estimates of snow water equivalent over the storm duration.
2.10.4.3 Strengths/Applications.
• Can be used to determine the amount of snowfall even when and where no reports are available.
• Has regionally optimized adaptable parameters which can be changed by the operator.
• Can accumulate with lower reflectivity values than the PPS.
• Has a range/height correction.
• Can also be used to estimate cool stratiform rainfall.
2.10.4.4 Limitations.
• The same adaptable parameters are applied to the entire radar coverage area, yet actual measured values (such as
Z-S relationship and snow ratio) will vary in time and space.
• Range of accurate estimates is at most 150 km, but can be less for very low-topped snow producing clouds such
as lake effect snow.
• Does not account for advection of snow after being observed by the radar.
• Does not account for blowing or drifting snow.
• Data levels are fixed and only provide a range of estimates (versus a digital estimate).
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.
2-159

October 2017


> **Figure 2-42: Storm Total Snow Water Equivalent Product (SSW #146).**

> An example Storm Total Snow Water Equivalent Accumulation product (OPUP display) from the Duluth, MN (KDLH) WSR-88D at 02:13 UTC on 16 March 2013.

An example Storm Total Snow Water Equivalent Accumulation product (OPUP display) from the Duluth, MN (KDLH) WSR-88D at 02:13 UTC on 16 March 2013.

2-160

FMH-11-Part C


#### 2.10.5 User-Selectable Snow Depth Accumulation.

The User-Selectable Snow Depth Accumulation product (USD) provides radar-estimates of snow depth accumulations over a user- defined number of whole hours (Time Duration) ending on a user-defined hour (End Hour) of
the day (UTC). The accumulations are made up of hourly accumulations updated once per hour near the top of the
hour. Products will be blank unless at least two-thirds of the top-of-the-clock hourly accumulations are available.
2.10.5.1 Operational Characteristics.
System ID: USD, Product #151.
Data: Radar Snow Depth accumulation estimates (inches).
Processing: Snow Accumulation Algorithm.
Availability: Up to 10 different accumulation periods once per volume scan.
Presentation: Polar coordinate image (Figure 2-43).
Resolution: 1 km (0.54 nm) x 1°.
Coverage: Radar centered, 230 km (124 nm) radar radius.
Data Levels: 16.
Annotations:
• Standard set*
• Beginning and ending dates and times of accumulations
• Data level code
• Maximum data value detected
• Azimuth and range of the maximum data value
• Range/height correction applied
• End Hour
• Time Duration
• Available Hours
Special Symbols: None.
Adaptation: See Section 3.2.12.
2.10.5.2 Usage. Provides estimated snow accumulations over a user-selectable period.
2.10.5.3 Strengths/Applications.
• Can be used to determine the amount of snowfall even when and where no reports are available.
• Has regionally optimized adaptable parameters which can be changed by the operator.
• Can accumulate with lower reflectivity values than the PPS.
• Has a range/height correction.
• Users can select the time period and end hour of accumulations. For example, a user could have a daily (24
hour) accumulation end at 12 UTC.
2.10.5.4 Limitations.
• The same adaptable parameters are applied to the entire radar coverage area, yet actual measured values (such as
Z-S relationship and snow ratio) will vary in time and space.
• Snow depth estimates are not accurate for snow that is melting (in the air or on the ground), but the product
does not distinguish between dry snow and other precipitation types.

2-161

October 2017

•
•
•
•

Range of accurate estimates is at most 150 km, but can be less for very low-topped snow producing clouds such
as lake effect snow.
Does not account for advection of snow after being observed by the radar.
Does not account for blowing or drifting snow.
Data levels are fixed and only provide a range of estimates (versus a digital estimate).

*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-162

FMH-11-Part C


> **Figure 2-43: User-Selectable Snow Depth Accumulation Product (USD #151).**

> An example User-Selectable Storm Depth Accumulation product (OPUP display) from the Duluth, MN

An example User-Selectable Storm Depth Accumulation product (OPUP display) from the Duluth, MN
(KDLH) WSR-88D at 01:01 UTC on 16 March 2013.

2-163

October 2017


#### 2.10.6 User-Selectable Snow Water Equivalent Accumulation.

The User-Selectable Snow Water Equivalent Accumulation product (USW) provides a radar estimate of the water
equivalent that has resulted from snowfall over a user-specified period of time.
2.10.6.1 Operational Characteristics.
System ID: USW, Product #150.
Data: Radar estimates of snow water equivalent accumulations (inches).
Processing: Snow Accumulation Algorithm.
Availability: Up to 10 different accumulation periods once per volume scan.
Presentation: Polar coordinate image of snow water equivalent accumulation estimates (Figure 2-44)
Resolution: 1 km (0.54 nm) x 1°
Coverage: Radar centered, 230 km (124 nm) radar radius.
Data Levels: 16.
Annotations:
• Standard set*
• Beginning and ending dates and times of accumulations
• Data level code
• Maximum data values detected
• Azimuth and range of the maximum data value
• Range/height correction applied
• End Hour
• Time Duration
• Available Hours
Adaptation: See Section 3.2.12.
2.10.6.2 Usage. Provides estimates of snow water equivalent over a user-selected number of hours ending at a selected
hour.
2.10.6.3 Strengths/Applications.
• Can be used to determine the amount of snowfall even when and where no reports are available.
• Has regionally optimized adaptable parameters which can be changed by the operator.
• Can accumulate with lower reflectivity values than the PPS.
• Has a range/height correction.
• Users can select the time period and end hour of accumulations. For example, a user could have a daily (24
hour) accumulation end at 12 UTC.
2.10.6.4 Limitations.
• The same adaptable parameters are applied to the entire radar coverage area, yet actual measured values (such as
Z-S relationship and snow ratio) will vary in time and space.
• Snow depth estimates are not accurate for snow that is melting (in the air or on the ground), but the product
does not distinguish between dry snow and other precipitation types.
• Range of accurate estimates is at most 150 km, but can be less for very low-topped snow producing clouds such
as lake effect snow.
• Does not account for advection of snow after being observed by the radar.
2-164

FMH-11-Part C

•
•

Does not account for blowing or drifting snow.
Data levels are fixed and only provide a range of estimates (versus a digital estimate).

*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-165

October 2017


> **Figure 2-44: User-Selectable Snow Water Equivalent Accumulation Product (USW #150).**

> An example User-Selectable Storm Water Equivalent Accumulation product (OPUP display) from the

An example User-Selectable Storm Water Equivalent Accumulation product (OPUP display) from the
Duluth, MN (KDLH) WSR-88D at 01:01 UTC on 16 March 2013.

2-166

FMH-11-Part C


### 2.11 Dual Polarization-Derived Precipitation Estimation Products.


#### 2.11.1 One-Hour Accumulation.

The One-Hour Accumulation product (OHA) provides the hourly precipitation accumulation on a 2 km (1.1 nm) x
1° grid within 230 km (124 nm) of the radar. The product is updated once per volume scan by the QPE algorithm
to provide the precipitation accumulation for the past 60 minutes. If there is too much missing data this product is
not generated.
2.11.1.1 Operational Characteristics.
System ID: OHA, Product #169.
Data: Adjusted hourly radar rainfall accumulation estimates (in/hr).
Processing: Precipitation estimates are computed by the QPE algorithm. The product is updated every volume
scan.
Availability: Once per volume scan.
Presentation: Polar coordinate image of hourly rainfall accumulation values (Figure 2-45).
Resolution: 2 km (1.1 nm) x 1°.
Coverage: Radar centered, 230 km (124 nm) radius.
Data Levels: 16.
Annotations:
• Standard set*
• Date and End Time of Rainfall Integration
• Maximum Data Value
• Radar Bias Estimate
• Effective G-R Pair Sample Size
• Missing Periods
• Gage Adjustment Bias Flag
Adaptation: See Section 3.3.3 and the Guidance on Adaptable Parameters Operator Handbook (WSR-88D Handbook) for discussions regarding operational considerations of the algorithms and their parameters.
2.11.1.2 Usage.
• Assess changes in precipitation intensity in time and space, and for flood and flash flood watches, warnings, and
statements.
• Can be used for Short Term Forecasts and Hazardous Weather Outlooks.
• Time lapse can provide movement and precipitation relative to river basins and other features.
• Other water management applications.
2.11.1.3 Strengths/Applications.
• Mitigates contamination by non-precipitating echoes vs. non-dual polarization precipitation estimates.
• Reduces biases and improves the accuracy of precipitation accumulations (vs. non-dual polarization estimates)
for different hydro-classes, including hail, wet snow (bright- banding), dry snow, big-drops, and heavy rain.
2.11.1.4 Limitations.
• The creation of the dual polarization hybrid scan and the rates calculated are highly dependent upon the accuracy of the HCA and the MLDA.
• The product can contain typically very small areas of erroneously high precipitation accumulations most likely
2-167

October 2017

•

caused by misclassification by the HCA.
Echoes which are not precipitation reaching the ground, such as virga, biota, or ground clutter, can contaminate
the product, although less than in PPS precipitation products.

*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-168

FMH-11-Part C


> **Figure 2-45: One-Hour Accumulation Product (OHA #169).**

> An example One-Hour Accumulation product (OPUP display) from the Saint Louis, MO (KLSX) WSR88D at 11:56 UTC on 18 April 2013. Figures 2-44 and 2-45 are at the same time for comparison

An example One-Hour Accumulation product (OPUP display) from the Saint Louis, MO (KLSX) WSR88D at 11:56 UTC on 18 April 2013. Figures 2-44 and 2-45 are at the same time for comparison
purposes.

2-169

October 2017


#### 2.11.2 Digital Accumulation Array.

The Digital Accumulation Array product (DAA) provides the hourly precipitation accumulation on 250 m (0.13 nm)
x 1° grid within the 230 km (124 nm) radius of the radar. The product is updated once per volume scan by the QPE
algorithm to provide the precipitation accumulation for the running past 60 minutes (i.e. one hour). If there is too
much missing data this product is not generated.
2.11.2.1 Operational Characteristics.
System ID: DAA, Product #170.
Data: One-hour radar precipitation accumulation estimates without gage bias (in/hr).
Processing: QPE algorithm provides the precipitation estimates for this product. The product is updated once
per volume scan.
Availability: Once per volume scan.
Presentation: Polar coordinate image of precipitation accumulation values (Figure 2-46).
Resolution: 0.25 km (0.13 nm) x 1°.
Coverage: Radar centered, 230 km (124 nm) radius.
Data Levels: 256.
Annotations:
• Standard set*
• Date and End Time of Rainfall Integration
• Maximum Data Value
• Radar Bias Estimate
• Effective G-R Pair Sample Size
• Missing Periods
• Gage Adjustment Bias Flag
Adaptation: See Section 3.3.3 and the Guidance on Adaptable Parameters Operator Handbook (WSR-88D
Handbook).
2.11.2.2 Usage.
• The DAA supports subjective and objective hydrometeorological forecast procedures for flash flood watches
and warnings, river stage forecasting, water management applications, and other hydrometeorological requirements for numerical use of precipitation data in computers external to the WSR-88D.
• Provides hourly running total digital radar-rainfall estimates in an array format to support processing performed
external to the WSR-88D.
• Never has a bias applied. As a result can be used by bias computing applications.
2.11.2.3 Strengths/Applications. Provides high-resolution precipitation estimates for generation of external hydrological products such as flood assessment and warnings.
2.11.2.4 Limitations. Same as the OHA product.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-170

FMH-11-Part C


> **Figure 2-46: Digital Accumulation Array Product (DAA #170).**

> An example Digital Accumulation Array product (OPUP display) from the Saint Louis, MO (KLSX)

An example Digital Accumulation Array product (OPUP display) from the Saint Louis, MO (KLSX)
WSR- 88D at 11:56 UTC on 18 April 2013. Figures 2-44 and 2-45 are at the same time for comparison purposes.

2-171

October 2017


#### 2.11.3 Digital One-Hour Accumulation Difference.

The Digital One-Hour Accumulation Difference product (DOD) provides the hourly precipitation accumulation
difference between the QPE and PPS algorithms on a 250 m (0.13 nm) x 1° grid within the 230 km (124 nm) radius
of the radar. The difference is computed by subtracting PPS accumulations from the QPE algorithm accumulations
over the same running one hour time span. If there is too much missing data this product is not generated.
2.11.3.1 Operational Characteristics.
System ID: DOD, Product #174.
Data: Hourly radar precipitation accumulation differences between the QPE and PPS algorithms (inches).
Processing: The QPE algorithm subtracts hourly estimates from the PPS hourly estimates.
Availability: Once per volume scan.
Presentation: Polar coordinate image of precipitation accumulation values (Figure 2-47).
Resolution: 0.25 km (0.13 nm) x 1°.
Coverage: Radar centered, 230 km (124 nm) radius.
Data Levels: 256.
Annotations:
• Standard set*
• Date and End Time of Rainfall Integration
• Maximum Data Value
• Radar Bias Estimate
• Effective G-R Pair Sample Size
• Missing Periods
• Gage Adjustment Bias Flag
Adaptation: See the Guidance on Adaptable Parameters Operator Handbook (WSR-88D Handbook).
2.11.3.2 Usage. Provides a graphical display of precipitation accumulation differences between the PPS and QPE
algorithms in a running one-hour window of time.
2.11.3.3 Strengths/Applications. Highlights differences between PPS and QPE precipitation estimates such as in
areas where QPE has eliminated erroneous accumulations.
2.11.3.4 Limitations. This product has the same limitations as the OHA and OHP products.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-172

FMH-11-Part C


> **Figure 2-47: Digital One-Hour Accumulation Difference Product (DOD #174).**

> An example Digital One-Hour Accumulation Difference product (OPUP display) from the Saint Louis, MO

An example Digital One-Hour Accumulation Difference product (OPUP display) from the Saint Louis, MO
(KLSX) WSR-88D at 11:56 UTC on 18 April 2013.

2-173

October 2017


#### 2.11.4 Storm Total Accumulation.

The Storm Total Accumulation products are displays of continuously updated radar-estimated storm total precipitation accumulations within 230 km (124 nm) of the radar. The storm total precipitation is defined as the accumulation since the beginning of the storm event. The STA has data available in 16 data levels and the DSA has data
available in 256 data levels for more precision.
2.11.4.1 Operational Characteristics.
System ID: STA, Product #171; DSA, Product #172
Data: Radar-derived total precipitation accumulation estimates from the QPE algorithm (inches).
Processing: This product represents the total precipitation accumulation measured since the beginning of a
storm event. A storm event begins when the precipitation of at least a precipitation rate threshold is met or exceeded for a given area threshold. The storm total accumulation will continue until the precipitation is below the
rate and area thresholds for at least a threshold time period (an hour by default). These thresholds are adaptable parameters.
Availability: Updated once per volume scan.
Presentation: Polar coordinate image of storm total accumulations (Figures 2-48a and 2- 48b). In addition, STA
also has a text list (paired-alphanumeric) of important relevant algorithm data and adaptable parameters and
their values. DSA also has a list of important data and adaptable parameters (values only – no text). Those
data are listed in the Annotations section below.
Resolution:
• 0.25 km (0.13 nm) x 1° (#172)
• 2 km (1.1 nm) x 1° (#171)
Coverage: Radar centered, 230 km (124 nm) radius.
Data Levels:
• 256 (#172)
• 16 (#171)
Annotations: STA:
• Standard set*
• Start date of accumulation
• Start time of accumulation
• Maximum accumulation (inches)
• Ending date of accumulation
• Ending time of accumulation
• Mean-field-bias (bias information for future implementation)
• Sample Size (Effective No. Gage/Radar Pairs) (for future implementation)
• Pair-Alphanumeric data, including:
▪ Melting Layer Detection Algorithm
○ Default Melting Layer Depth
○ Manual Override Flag
▪ Rate Algorithm
○ Kdp Multiplier Coefficient
○ Kdp Power Coefficient
○ Z-R Multiplier Coefficient
2-174

FMH-11-Part C

○ Z-R Power Coefficient
○ Zdr/Z Multiplier Coefficient
○ Zdr/Z Power Coefficient for Z
○ Zdr/Z Power Coefficient for Zdr
○ Maximum Correlation Coef./Kdp
○ Maximum Kdp Beam Blockage
○ Maximum Usability Blockage
○ Maximum Reflectivity
○ Minimum Kdp Usage Rate
○ Wet Snow Multiplicative Coefficient
○ Graupel Multiplicative Coefficient
○ Rain/Hail Multiplicative Coefficient
○ Dry Snow Multiplicative Coefficient
○ Crystals Multiplicative Coefficient
○ Percent of the precipitation rate grid filled
○ PAIF Precipitation Rate Threshold
○ PAIF Precipitation Area Threshold
○ Precipitation Detection Time Threshold
○ Number of Exclusion Zones
▪ Accumulation Algorithm
○ Threshold Elapsed Time to Restart
○ Maximum Time for Interpolation
○ Maximum Hourly Accumulation Value
▪ Adjustment Algorithm
○ Time bias Estimation (bias information for future implementation)
○ Threshold Number of Gage-Radar Pairs (bias information for future implementation)
○ Reset Bias Value (bias information for future implementation)
○ Longest Allowable Lag (Hours) (bias information for future implementation)
○ Bias Applied Flag (bias information for future implementation)
DSA:
▪ Standard set*
▪ Start date of accumulation
▪ Start time of accumulation
▪ Maximum accumulation (inches)
▪ Ending date of accumulation
▪ Ending time of accumulation
▪ Mean-field-bias (bias information for future implementation)
Adaptation Data, including:
▪ Melting Layer Detection Algorithm
○ Default Melting Layer Depth
2-175

October 2017

○ Manual Override Flag
▪ Rate Algorithm
○ Kdp Multiplier Coefficient
○ Kdp Power Coefficient
○ Z-R Multiplier Coefficient
○ Z-R Power Coefficient
○ Zdr/Z Multiplier Coefficient
○ Zdr/Z Power Coefficient for Z
○ Zdr/Z Power Coefficient for Zdr
○ Maximum Correlation Coef./Kdp
○ Maximum Kdp Beam Blockage
○ Maximum Usability Blockage
○ Maximum Reflectivity
○ Minimum Kdp Usage Rate
○ Wet Snow Multiplicative Coefficient
○ Graupel Multiplicative Coefficient
○ Rain/Hail Multiplicative Coefficient
○ Dry Snow Multiplicative Coefficient
○ Crystals Multiplicative Coefficient
○ Percent of the precipitation rate grid filled
○ PAIF Precipitation Rate Threshold
○ PAIF Precipitation Area Threshold
○ Precipitation Detection Time Threshold
○ Number of Exclusion Zones
▪ Accumulation Algorithm
○ Threshold Elapsed Time to Restart
○ Maximum Time for Interpolation
○ Maximum Hourly Accumulation Value
▪ Adjustment Algorithm
○ Time Bias Estimation (bias information for future implementation)
○ Threshold Number of Gage-Radar Pairs (bias information for future implementation)
○ Reset Bias Value (bias information for future implementation)
○ Longest Allowable Lag (Hours) (bias information for future implementation)
○ Bias Applied Flag (bias information for future implementation)
▪ Supplemental Data
○ Scan Date
○ Scan Time
○ Flag Precip Detected
○ Flag Storm Total Active
○ Flag Precip Begin
○ Last Date Precip
2-176

FMH-11-Part C

○ Last Time Precip
○ Percent of Hybrid Rate Filled
○ Highest Elevation Angle Used
○ Total Precipitation Area
○ Spot Blanking Volume Status
▪ Bias-Related Fields, for future implementation
○ Time Bias Value Last Updated Locally
○ Date Bias Value Last Updated Locally
○ Time of Last Update of Local Bias Table
○ Date of Last Update of Local Bias Table
○ Observation Time of Latest Bias Table
○ Observation Date of Latest Bias Table
○ Generation Time of Latest Bias Table
○ Generation Date of Last Bias Table
○ Mean-Field Bias Estimate
○ Effective G-R Pair Sample Size
○ Memory Span used in Bias Estimate
○ AWIPS Site ID of Most Recent Bias Source
Adaptation: See Section 3.3.3 and Adaptable Parameters Handbook (Guidance on Adaptable Parameters).
2.11.4.2 Usage.
• Assess changes in precipitation intensity in time and space, and for flood and flash flood watches, warnings, and
statements.
• Can be used for Short Term Forecasts and Hazardous Weather Outlooks.
• Time lapse can provide movement and precipitation relative to river basins and other features.
• Other water management applications such as estimation of basin run-off and soil saturation.
• Post-storm analysis such as evaluation of flood reports.
2.11.4.3 Strengths/Applications.
• Same as OHA
• Monitoring accumulations throughout a weather event, regardless of the duration.
2.11.4.4 Limitations.
• Same as OHA
• The radar can erroneously stay in the same storm total for extended periods of time owing to high frequency
of showers in some areas or due to unaccounted for clutter residue. If that is the case, the one-hour and storm
total accumulations need to be manually reset to zero, when appropriate
• When there are rare small periods of missing data, the algorithm will attempt to interpolate over the missing
period. Obviously, this interpolated data are not as accurate and can be included without the knowledge of the
operator.
See the Guidance on Adaptable Parameters Operator Handbook (WSR-88D Handbook) for discussions regarding
the algorithm’s operational considerations.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.
2-177

October 2017


> **Figure 2-48a: Storm Total Accumulation Product (STA #171).**

> An example Storm Total Accumulation product (OPUP display) from the Saint Louis, MO (KLSX)

An example Storm Total Accumulation product (OPUP display) from the Saint Louis, MO (KLSX)
WSR- 88D at 21:00 UTC on 18 April 2013.

2-178

FMH-11-Part C


> **Figure 2-48b: Digital Storm Total Accumulation Product (DSA #172).**

> An example Digital Storm Total Accumulation product (OPUP display) from the Saint Louis, MO

An example Digital Storm Total Accumulation product (OPUP display) from the Saint Louis, MO
(KLSX) WSR-88D at 21:00 UTC on 18 April 2013.

2-179

October 2017


#### 2.11.5 Digital Storm Total Difference.

The Digital Storm Total Difference product (DSD) provides the storm total precipitation accumulation difference
between the QPE and PPS algorithms on a 250 m (0.13 nm) x 1° grid within the 230 km (124 nm) radius of the
radar. The difference is computed by subtracting accumulations of the PPS from the accumulations of the QPE
algorithm. The DSD product consists of differences computed throughout the course of a storm event. This
product will be generated as long as either the QPE algorithm or the PPS is within a storm total accumulation. The
product is updated once per volume scan by the QPE algorithm. If there is too much missing data this product is
not generated.
2.11.5.1 Operational Characteristics.
System ID: DSD, Product #175.
Data: A storm total precipitation accumulation difference between the QPE and PPS algorithms (inches).
Processing: QPE and PPS algorithms.
Availability: Once per volume scan.
Presentation: Polar coordinate image of rainfall accumulation values (Figure 2-49).
Resolution: 0.25 km (0.13 nm) x 1°.
Coverage: Radar centered, 230 km (124 nm) radius.
Data Levels: 256.
Annotations:
• Standard set*
• Date and End Time of Rainfall Integration
• Maximum Data Value
• Radar Bias Estimate
• Effective G-R Pair Sample Size
• Missing Periods
• Gage Adjustment Bias Flag
Adaptation: See the Guidance on Adaptable Parameters Operator Handbook (WSR-88D Handbook).
2.11.5.2 Usage. Provides a graphical display of precipitation accumulation differences between the PPS and QPE
algorithms throughout a storm event.
2.11.5.3 Strengths/Applications.
• Highlights differences between PPS and QPE algorithms.
• Gives users a way to visualize differences in precipitation computations (e.g. elimination of spurious accumulations from clutter and biota).
2.11.5.4 Limitations.
• Although every effort has been made to remove non-precipitation reflectivity returns from the calculation of
precipitation, residual ground clutter or anomalous propagation may contaminate data. This contamination is
expected to be worse for the PPS.
• There could be a significant difference at the beginning and/or ending of a storm event because of differences
in sensitivity between the two algorithm inputs.
• This product has the same limitations as the Digital Storm Total Accumulation (DSA) and Digital Storm Total
Precipitation (DSP) products.

2-180

FMH-11-Part C

*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-181

October 2017


> **Figure 2-49: Digital Storm Total Difference Product (DSD #175).**

> An example Digital Storm Total Difference product (OPUP display) from the Saint Louis, MO (KLSX)

An example Digital Storm Total Difference product (OPUP display) from the Saint Louis, MO (KLSX)
WSR-88D at 21:00 UTC on 18 April 2013.

2-182

FMH-11-Part C


#### 2.11.6 Digital User-Selectable Accumulation.

The Digital User-Selectable Accumulation (DUA) product provides a map of precipitation accumulations over a
user-selected time period from the QPE algorithm displayed as an image. The product format and content are the
same as the Digital Accumulation Array (DAA) and Digital Storm Total Accumulation (DSA) products, except the
accumulation period is of variable duration, ranging from a beginning to an ending time specified by the user. The
DUA product can cover a span of time, in minutes, as small as 15 minutes and as long as 24 hours with a temporal
precision of one minute.
2.11.6.1 Operational Characteristics.
System ID: DUA, Product #173.
Data: Accumulation estimates from the QPE algorithm (inches).
Processing: 1-hour and 3-hour accumulations are automatically generated every volume scan. A 24-hour accumulation is generated once each day at 1200Z. OTRs are generated as they are received.
Availability: Up to 10 different accumulation periods once per volume scan. Additional products will be generated in response to OTRs.
Presentation: Polar coordinate image of accumulated precipitation (Figure 2-50).
Resolution: 0.25 km (0.13 nm) x 1°.
Coverage: Radar centered, 230 km (124 nm) radius.
Data Levels: 256.
Annotations:
• Standard set*
• Data level code
• Maximum data values detected
• Times and dates of the beginning and end (clock hour) of the rainfall rate integration
• Mean-field bias in the radar estimate of the precipitation rate
• Effective sample size associated with the bias estimate
Adaptation: See the Guidance on Adaptable Parameters Operator Handbook (WSR-88D Handbook).
2.11.6.2 Usage. Used to estimate precipitation accumulation over a user-specified period.
2.11.6.3 Strengths/Applications.
• Same as OHA
• Flexible time interval to meet varying weather situations.
• In addition to the one-hour and the 24-hour default DUA products, other time periods generated by OTR are
available to any type of user.
2.11.6.4 Limitations.
• Same as OHA
• A maximum of 10 unique products (i.e. covering different time periods) can be routinely generated per volume
scan. Additional products will be generated in response to OTRs.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-183

October 2017


> **Figure 2-50: Digital User-Selectable Accumulation Product (DUA #173).**

> An example Digital User-Selectable Accumulation product (OPUP display) from the Saint Louis, MO

An example Digital User-Selectable Accumulation product (OPUP display) from the Saint Louis, MO
(KLSX) WSR-88D at 13:01 UTC on 18 April 2013.

2-184

FMH-11-Part C


### 2.12 Aviation Hazard Products.


#### 2.12.1 Eddy Dissipation Rate and Eddy Dissipation Confidence.

The eddy dissipation products provide values of in-cloud atmospheric turbulence (EDR) and of an associated data
quality control index or “confidence” (EDC). These products are generated from the NEXRAD Turbulence Detection Algorithm (NTDA) to support the detection of conditions hazardous to aviation.
2.12.1.1 Operational Characteristics.
System ID: EDR, Product #156; EDC, Product #157.
Data: EDR in (m2/s3)1/2; confidence in percent.
Availability: Once per volume scan for each elevation slice.
Presentation: Polar coordinate of velocity data (Figures 2-51, a-b).
Resolution: 2 km (1.1 nm) x 1°.
Coverage: Radar centered, 230 km (124 nm) radius.
Data Levels:
• 64 (#156)
• 8 (#157)
Annotations:
• Standard set*
• Data level code
Adaptation: None.
2.12.1.2 Usage. When used in conjunction, these products aid in detecting conditions hazardous to aviation.
2.12.1.3 Strengths/Applications. Identify and locate areas of turbulence.
2.12.1.4 Limitations. Data levels cannot be changed.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-185

October 2017


> **Figure 2-51a: Eddy Dissipation Rate Product (EDR #156).**

> An example Eddy Dissipation Rate product (OPUP display) from the Saint Louis, MO (KLSX) WSR88D at 12:38 UTC on 18 April 2013. Figures 2-50a and 2-50b are at the same the same time for

An example Eddy Dissipation Rate product (OPUP display) from the Saint Louis, MO (KLSX) WSR88D at 12:38 UTC on 18 April 2013. Figures 2-50a and 2-50b are at the same the same time for
comparison purposes.

2-186

FMH-11-Part C


> **Figure 2-51b: Eddy Dissipation Confidence Product (EDC #157).**

> An example Eddy Dissipation Confidence product (OPUP display) from the Saint Louis, MO (KLSX)

An example Eddy Dissipation Confidence product (OPUP display) from the Saint Louis, MO (KLSX)
WSR- 88D at 12:38 UTC on 18 April 2013.

2-187

October 2017


#### 2.12.2 Gust Front MIGFA.

The MIGFA (Machine Intelligent Gust Front Algorithm) product provides convergence boundary detections (such
as gust fronts) with 10 and 20 minute forecast positions. MIGFA is available as a stand-alone product or as an overlay.
2.12.2.1 Operational Characteristics.
System ID: GFM, Product #140.
Data: Radial Velocity (m/s).
Availability: Once per volume scan.
Presentation: Overlay of current and forecast positions (Figures 2-52).
Resolution: None.
Coverage: Radar centered, 70 km (38 nm) radius.
Data Levels: None.
Annotations:
• Standard set*
• Count of detections
Adaptation: None.
2.12.2.2 Usage. Aid in detecting conditions hazardous to aviation.
2.12.2.3 Strengths/Applications. Identify and locate convergence boundaries (i.e. gust fronts).
2.12.2.4 Limitations.
• Might not resolve convergence boundary feature with a relatively wide width radar signature.
• Tendency to drop convergence boundary features as they become orthogonally aligned crossing over the radar.
• Some detections are bird or bat roost activity, especially at sunrise and sunset.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-188

FMH-11-Part C


> **Figure 2-52: Gust Front MIGFA Product (GFM #140).**

> An example Gust Front MIGFA product (OPUP display) overlaid on a Mean Radial Velocity Data Array

An example Gust Front MIGFA product (OPUP display) overlaid on a Mean Radial Velocity Data Array
product from the Oklahoma City, OK (KTLX) WSR-88D at 01:46 UTC on 01 June 2013.

2-189

October 2017


#### 2.12.3 Hail Hazard Layers.

The Hail Hazard Layers product provides vertical extent of hail throughout each radar volume as reported by
the dual polarization hydrometeor classification algorithm. The top and bottom altitudes of each hail location are
provided. Hail detection altitudes are reported at a vertical resolution of 1,000 feet referenced to MSL. Hail severity
(related to size) and confidence are also reported for the hail altitude top and bottom. Initial version of the product
has severity and confidence set to a not-used default.
2.12.3.1 Operational Characteristics.
System ID: HHL, Product #179.
Data: Hydrometeor Classification Algorithm and model data.
Availability: Once per volume scan.
Presentation: Polar coordinate of hail coverage and altitude (Figures 2-53).
Resolution: 1 km (0.54 nm) x 1°.
Coverage: Radar centered, 300 km (162 nm) radius.
Data Levels: 71.
Annotations:
• Standard set*
• Data level code
Adaptation: None.
2.12.3.2 Usage. Aid in detecting conditions hazardous to aviation.
2.12.3.3 Strengths/Applications.
• Identify and locate areas of hail.
• In absence of surface hail reports, indicates areas of hail formation or growth aloft.
• Presence of hail aloft might distinguish a storm cell with a greater severe potential than a storm cell without hail
aloft.
2.12.3.4 Limitations.
• Data levels cannot be changed.
• Hail detection limited to performance of HCA rain/hail class.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-190

FMH-11-Part C


> **Figure 2-53: Hail Hazard Layers Product (HHL #179).**

> An example Hail Hazard Layers product (OPUP display) from the Saint Louis, MO (KLSX) WSR-88D

An example Hail Hazard Layers product (OPUP display) from the Saint Louis, MO (KLSX) WSR-88D
at 12:38 UTC on 18 April 2013.

2-191

October 2017


#### 2.12.4 Icing Hazard Levels.

The Icing Hazard Levels product provides vertical extent of icing throughout each radar volume reported as graupel by the dual polarization hydrometeor classification algorithm and combined with NWS model data. The top and
bottom altitudes of each icing location are provided. Icing detection altitudes are reported at a vertical resolution of
1,000 feet referenced to MSL. Icing severity and confidence are also reported for the icing altitude top and bottom.
Initial version of the product has severity and confidence set to a non-used default.
2.12.4.1 Operational Characteristics.
System ID: IHL, Product #178.
Data: Hydrometeor Classification Algorithm, Melting Layer Detection Algorithm, and model data.
Availability: Once per volume scan.
Presentation: Polar coordinate of icing coverage and altitude (Figures 2-54).
Resolution: 1 km (0.54 nm) x 1°.
Coverage: Radar centered, 300 km (162 nm) radius.
Data Levels: 71
Annotations:
• Standard set*
• Data level code
Adaptation: None.
2.12.4.2 Usage. Aid in detecting conditions hazardous to aviation.
2.12.4.3 Strengths/Applications.
• Identify and locate areas of icing.
• Initial version (graupel class with model) corresponds well to locations of icing pilot reports (PIREPS) and is
useful as such for regions without PIREPS (due to planes not flying in areas during storms).
2.12.4.4 Limitations.
• Data levels cannot be changed.
• Initial version (graupel class with model) will not report icing hazard conditions present that do not generate
graupel (such as pure freezing rain or freezing drizzle).
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-192

FMH-11-Part C


> **Figure 2-54: Icing Hazard Levels Product (IHL #178).**

> An example Icing Hazard Levels product (OPUP display) from the Saint Louis, MO (KLSX) WSR-88D

An example Icing Hazard Levels product (OPUP display) from the Saint Louis, MO (KLSX) WSR-88D
at 12:38 UTC on 18 April 2013.

2-193

October 2017


### 2.13 Other Products.


#### 2.13.1 Archive III Status Product.

The Archive III Status product (ASP) contains eight of hours of RPG status log messages. The product contains all
RPG status log and error messages since the last product was generated. The messages are color-coded according
to message type. RPG information and general status messages are green. RPG warnings are amber. RDA and RPG
alarms are red. Narrowband communications messages are blue.
2.13.1.1 Operational Characteristics.
System ID: ASP, Product #152.
Data: RPG status logs.
Availability: Once every eight hours at 00:00, 08:00, and 16:00 UTC.
Presentation: Color-coded text file (Figure 2-55).
Resolution: None.
Coverage: None.
Data Levels: None.
Annotations:
• Product name
• Radar ID
• Date and time of log
• Message number, type, and text
• RPG information and general status messages
• RPG warnings and errors
• RDA and RPG alarms
• Narrowband communications status
Adaptation: None.
Usage. Diagnose and troubleshoot radar system status.
Strengths/Applications. Identify all status and error messages.
Limitations. None.

2-194

FMH-11-Part C

2-195

October 2017


> **Figure 2-55: Archive III Status Product (ASP #152).**

> An example Archive III Status product on 05 June 2013 at 15:57:42 UTC (NOAA’s Weather & Climate

An example Archive III Status product on 05 June 2013 at 15:57:42 UTC (NOAA’s Weather & Climate
Toolkit display). The radar ICAO is not shown here to preserve anonymity. Several status messages
have been removed from this example in order to show a variety of messages.

2-196

FMH-11-Part C


#### 2.13.2 Free Text Message.

The Free Text Message product (FTM) is an alphanumeric message generated by a radar operator/maintainer at the
MSCF HCI for transmission to RPG interfaces. A message may be designated to be transmitted to a specific unit
interface or several interfaces. The message format may be a pre-worded message for further user editing.
2.13.2.1 Operational Characteristics.
System ID: FTM, Product #75.
Availability: When produced by the operator. Presentation: An alphanumeric message (examples below).
Annotations: RPG ID.
Adaptation: Alphanumeric character strings defining the message content, source, and intended destination.
KXXX HAS CHANGED ITS Z/R RELATIONSHIP TO DEFAULT.
THE KXXX RADAR IS DOWN FOR MAINTENANCE AND SHOULD BE OPERATIONAL AGAIN
AROUND 1800 UTC.
KXXX IS BACK IN OPERATION. DUAL POL PRODUCTS ARE NOT OPERATIONAL. WE APOLOGIZE FOR ANY INCONVENIENCE.
KXXX RADAR IS INOPERABLE. REPLACEMENT PARTS ARE ON ORDER...BUT NO ESTIMATED RETURN TO SERVICE IS KNOWN AT THIS TIME. AN UPDATE WILL BE ISSUED ONCE NEW INFORMATION BECOMES AVAILABLE. WE APOLOGIZE FOR THE RADAR OUTAGE.
THE ECHOES THAT ARE SHOWING UP ON THE KXXX WSR-88D DATA OVER THE GULF OF
MEXICO APPEAR TO BE CHAFF.

> **Figure 2-56: Example Free Text Message (FTM #75)**

> 2.13.3 General Status Message.


#### 2.13.3 General Status Message.

The General Status Message product (GSM) is the first Product Data Level message transmitted by the RPG to a
Class 1 user upon connection. The GSM describes the state of the RDA and RPG. This data informs the user about
operational modes, the scan strategy, and equipment status of the RDA and RPG.
2.13.3.1 Operational Characteristics.
System ID: GSM, Message #2.
Availability: At the top of each hour and when the state of the NEXRAD system changes. A GSM will also be
sent at the start of the elevation of an AVSET-terminated VCP.
Presentation: A graphic depicting the state of the NEXRAD system (Figure 2-57).
Annotations:
• Radar ID
• Date and time
• Operational mode
• VCP elevations
• Status of AVSET, SAILS, and site-specific scan strategies
• RDA, RPG, and Class 1 user states
• Wideband and narrowband states
Adaptation: None.

2-197

October 2017


> **Figure 2-57: General Status Message Product (GSM #2).**

> An example General Status Message product (OPUP display).

An example General Status Message product (OPUP display).

2-198

FMH-11-Part C


#### 2.13.4 Radar Coded Message.

The Radar Coded Message product (RCM) is based on grid squares approximately 10 km (5.4 nm) on a side. It is
composed of three parts preceded by a communications header. Part A: Reflectivity is produced automatically and
is presented to the user as a tabular listing of alphanumerics. Part B: Velocity Azimuth Display is also produced
automatically and is presented to the user as an alphanumeric message. Part C: Remarks, consists of automatically
provided information that can be augmented at the option of the user. The automatically produced portion of Part
C is presented to the user as an alphanumeric message. The RCM is not manually edited. For more detailed information regarding the RCM please refer to the ICD given in Chapter 1 of this handbook.
2.13.4.1 Operational Characteristics.
System ID: RCM, Product #74.
Data:
• Composite reflectivity
• Hail index
• Maximum echo top
• Mesocyclone detection
• Storm centroid
• Storm structure
• Storm tracking information
• Tornadic vortex signature
• Velocity azimuth display
Processing: Convert composite reflectivity to 1/16 x 1/16 LFM “gridded” data.
Availability: Up to twice per hour at scheduled time(s).
Presentation:
• Part A: Internal - Pixel Image (Figure 2-58) External - Alphanumeric
• Part B: Internal – Alphanumeric External - Alphanumeric
• Part C: Internal – Alphanumeric External - Alphanumeric
Resolution:
• Parts A and C: 1/16 x 1/16 LFM grid
• Part B: Up to 19 heights
Coverage:
• Parts A and C: Radar centered, 460 km (248 nm) radius
• Part B: Surface to 50,000 ft MSL
Data Levels: 8.
Annotations:
• Standard set*
• Data level
• Maximum data value for reflectivity
• Maximum echo top
Adaptation.
• Scheduled production time(s)
• VAD levels
2-199

October 2017

2.13.4.2 Usage. Provides summary level radar information. Its primary use is in the preparation National Radar
Summary Chart for aviation and other interests. Other uses are derivative.
2.13.4.3 Strengths/Applications. Provides a national summary of radar echoes.
2.13.4.4 Limitations.
• Product is not edited.
• LFM model and grid no longer used, so grid does not match any numerical model.
• Other Federal government, private and universities provide more frequent national and regional mosaics; and
provide more sophisticated ways to remove clutter and AP.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-200

FMH-11-Part C


> **Figure 2-58: Radar Coded Message National Graphic (RCM #74).**

> An example Radar Coded Message National Graphic product from 05 June 2013 at 13:21 UTC (AWIPS display).

An example Radar Coded Message National Graphic product from 05 June 2013 at 13:21 UTC (AWIPS display).

2-201

October 2017


#### 2.13.5 User Alert Message.

The User Alert Message product (UAM) provides a brief message to the user to report an indication of severe
weather. The message contains the location, type and severity of the weather event, and, if appropriate, storm speed
and direction of movement. Alerts are updated once per volume scan. The alert message is issued first when the
selected alert condition is initially detected; the alert message is issued only for initial detections of the selected alert
condition following periods of non-detection lasting at least one volume scan. Whenever the alert message is issued,
it is accompanied by an audio alert if the device has been activated.
The message is based on the detection of any alerting phenomenon in either of two alert areas. The message indicates the radar ID, the date and time, the alert area, the azimuth and range of the 16 x 16 km (8.6 x 8.6 nm) grid box
in which the detected phenomenon was found, the user- established phenomenon and threshold, the value meeting
or exceeding the threshold, and the storm ID (if the alert is associated with a storm). The message indicates one of
the four following conditions:
• Alerts active - at least one alert category has been detected following a volume scan devoid of any alerts.
• Alerts inactive - this is the first volume scan for which there are no alerts for any categories following a volume
scan that had at least one alert category for which an alert was detected.
• Alert active for a category - this is the first volume scan for which alerts were detected for a category and area
following a volume scan devoid of alerts for that category.
• Alert message update - alerts are detected, however, none of the above conditions hold (i.e., no new alerts were
detected).
2.13.5.1 Operational Characteristics.
System ID: UAM, Product #73.
Data: Alert information containing the location, type, and severity of weather event as derived by the meteorological algorithms (Figure 2-59).
Processing:
• Alert generation
• Kinematic Series algorithms
• PPS algorithm
• Storm Series algorithms
Availability: Automatic, upon detection of each user-selected alert condition at the end of each volume scan.
Presentation: Plain language message accompanied by an audio alert:
UAM NNN (RDA) HH:MM (hour.minute), MM:DD:YY (month:day:year) Alert Area: (1 or 2)
Alert Box Center Azimuth: XXX.X (tenths of degrees) Alert Box Center Range: XXX.X (tenths of nm)
Alert Category: (Grid, Volume, or Forecast) Threshold Value: (user-selected value and units) Exceeding Value: (alert
triggering value and units) Storm ID.
Coverage: Area(s) of coverage are user defined.
Data Levels: Alert thresholds are user defined.
Annotations:
• Standard set*
• Location of alerting phenomenon
• Severe weather information, as appropriate

2-202

FMH-11-Part C

Adaptation:
• Alertable phenomena (Table 2-2)
• Alert boxes
• Alert values for each phenomenon are set at the MSCF (Table 2-2)
• Alert value (one per phenomenon) is selected from the values set at the MSCF
2.13.5.2 Usage. Notifies user upon detection of alerting phenomenon within a specified alert box.
2.13.5.3 Strengths/Applications.
• Alerts user of potential severe weather threat when selected.
• Alerts user only once until the alert condition no longer exists (for at least one volume scan) and then reappears.
This attempts to prevent annoying repeated notification.
2.13.5.4 Limitations. Anomalous values can trigger an alert.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-203

October 2017


> **Table 2-2 Meteorological Phenomena for Alerts**

> Alert Categories

Alert Categories

Coverage Radius (km/
nm)

I. Grid Group
Composite Reflectivity
Echo Tops
Severe Weather Probability
Velocity (lowest elevation)
Vertically Integrated Liquid
II. Volume Group (non-grid, algorithm outputs)
Maximum Hail Size
Maximum One-Hour Precipitation Accumulation
Maximum Storm Reflectivity
Probability of Hail
Probability of Severe Hail
Mesocyclone Detection
Storm Top
Tornado Vortex Signature
Velocity Azimuth Display Winds
(lowest elevation)
III. Forecast Group (storm oriented)
Maximum Hail Size
Maximum Storm Reflectivity
Probability of Hail
Probability of Severe Hail
Storm Top
Tornado Vortex Signature

-

Threshold Criteria
-

345/186
230/124
230/124
115/62
230/124
-

dBZe: 6 values
kft: 6 values
Probability (%): 6 values
knots: 6 values
kgm-2: 6 values
-

230/124
230/124

1 inch up to 4.0 inches in increments of 0.25
inches
inches: 6 values

230/124
230/124
230/124
230/124
230/124
115/62
NA

dBZe: 6 values
%
%
Strength Rank: 6 values
kft: 6 values
Detected
kts: 6 values

-

-

230/124
230/124
230/124
230/124
230/124
115/62

1 inch up to 4.0 inches in increments of .25
inches.
dBZe: 6 values
%
%
kft: 6 values
Detected

Expansion provisions accommodate additional alert categories as follows:
Group I
Group II
Group III


additional
additional
additional

2-204

FMH-11-Part C


> **Figure 2-59 User Alert Message (UAM #73).**

> An example User Alert Message product from the Radar Operations Center Testbed (KCRI) WSR88D on 19 June 2013 at 17:13 UTC (AWIPS display).

An example User Alert Message product from the Radar Operations Center Testbed (KCRI) WSR88D on 19 June 2013 at 17:13 UTC (AWIPS display).

2-205

October 2017


### 2.14 Removed Products.

2.14.1.1 Reflectivity Enhanced Resolution.
This product will be removed in RPG Build 16.0. The Reflectivity Enhanced Resolution product (DR7) provides
displays of reflectivity data for each elevation angle scan at a spatial resolution of 1 km (0.54 nm) x 1°, available
with 128 data levels.
2.14.1.1 Operational Characteristics.
System ID: DR7, Product #194.
Data: Reflectivity (dBZ).
Availability: Once per volume scan for each elevation slice.
Presentation: Polar coordinate of reflectivity data (Figure 2-60).
Resolution: 1 km (0.54 nm) x 1°
Coverage: 230 km (124 nm) radius, radar centered.
Data Levels: 128.
Annotations:
• Standard set*
• Calibration constant (scaling constant used by the Programmable Signal Processor [PSP] to calculate reflectivity)
• Data level code
• Elevation angle
• Maximum reflectivity value detected
Adaptation: None.
2.14.1.2 Usage.
• As an aid in the analysis of meteorological events by locating and tracking storms.
• Primary use is for surveillance as well as a detailed interpretation on a storm-by-storm basis.
• Identify severe weather signatures, the bright band, and boundaries.
• Monitor the evolution of the planetary boundary layer.
• Estimate rainfall intensity.
2.14.1.3 Strengths/Applications.
• Observe precipitation intensity, movement, and trends.
• Evaluate environmental conditions and meteorological characteristics such as inversions or moisture layers,
especially in Clear Air Mode.
• Identify ice cloud layers and even very light precipitation characteristics.
• Identify and locate the freezing/melting level.
• Observe and at times even track non-precipitation phenomena such as birds, bats, insects, smoke, volcanic ash,
chaff, etc.
• Weak returns from refractive index gradients and small particulates such as insects reveal many characteristics of
the boundary layer.
• Determine the location and motion of wind shear lines and boundaries such as gust fronts, synoptic fronts, sea
breezes, and wind-shifts of all kinds.
• Determine significant convective storm structural features such as Weak Echo Region (WER), Bounded Weak
Echo Region (BWER), hook echoes, and even evidences for Rear Flank Downdraft (RFD) existence. Line Echo
Wave Patterns (LEWP) and squall lines can be identified.
2-206

FMH-11-Part C

2.14.1.4 Limitations.
• Data levels cannot be changed.
• Residual ground and point clutter and AP can contaminate data.
• Chaff and biological echoes are often difficult to distinguish from precipitation echoes.
_____
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); and Operational Mode.

2-207

October 2017


> **Figure 2-60: Reflectivity Enhanced Resolution Product (DR7 #194).**

> An example Reflectivity Enhanced Resolution product (OPUP display) at the 0.5° elevation angle

An example Reflectivity Enhanced Resolution product (OPUP display) at the 0.5° elevation angle
from the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-208

FMH-11-Part C


#### 2.14.2 Mean Radial Velocity Enhanced Resolution.

This product will be removed in RPG Build 16.0. The Mean Radial Velocity Enhanced Resolution product (DV7)
provides display or radar estimated mean radial velocity. The product is available at a resolution of 0.50 km (0.27
nm) x 1° with 128 data levels.
2.14.2.1 Operational Characteristics.
System ID: DV7, Product #199.
Data: Mean radial velocity (m/s).
Availability: Once per volume scan for each elevation slice.
Presentation: Polar coordinate of velocity data (Figure 2-61).
Resolution: 0.50 km (0.27 nm) x 1°.
Coverage: 230 km (124 nm) radius, radar centered.
Data Levels: 128.
Annotations:
• Standard set*
• Data level code
• Elevation angle
• Maximum data value detected (both positive and negative)
Adaptation: Data levels.
2.14.2.2 Usage.
• Aids wind flow structure and shear recognition of the atmosphere on various scales.
• The detection and location of tornadic circulations, mesocyclones, and other atmospheric vortices.
• Determination of local wind field characteristics.
• Can be used to identify boundary layer characteristics or outflow regions.
2.14.2.3 Strengths/Applications.
• Estimate magnitude of radial velocities. Ground relative wind speeds (and directions, in so far as they can be
derived and used for use as input into warnings, statements, and forecasts).
• Aid in determining kinematic atmospheric structure via radial velocities. Atmospheric jets and temperature advection can be determined within the local radar coverage area and with adequate reflectors.
• Aid in determining internal convective storm kinematic structure via radial velocity patterns. Vorticies such as
tornado vortex signatures or mesocyclones and divergence intensity signatures at storm top or in association
with microbursts can be identified.
• High temporal and spatial resolution.
• Aid in creating, adjusting, or updating hodographs.
2.14.2.4 Limitations.
• Range folding may obscure data.
• Velocity aliasing can mask real velocities or shears.
• Velocities may exceed product data levels or even the signal processing specified velocity data levels. The scale is
locally adaptable at the MSCF.
_____
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); and Operational Mode.
2-209

October 2017


> **Figure 2-61:Mean Radial Velocity Enhanced Resolution Product (DV7 #199).**

> An example Velocity Enhanced Resolution product (OPUP) display at the 0.5° elevation angle from

An example Velocity Enhanced Resolution product (OPUP) display at the 0.5° elevation angle from
the Saint Louis, MO (KLSX) WSR-88D at 12:38 UTC on 18 April 2013.

2-210

FMH-11-Part C


#### 2.14.3 Combined Shear.

This product was removed in RPG Build 11. The Combined Shear product (CS) provides the combined radial and
azimuthal shear of the mean radial velocity, as calculated by the CS algorithm. It is presented as an image of filtered
shear values containing 16 data levels. Upon user request, all site adaptable parameters identified as inputs to the
algorithms used to generate data for this product are available in alphanumeric tabular form.
2.14.3.1 Operational Characteristics.
System ID: CS, Product #87.
Data: Combined azimuthal and radial shear.
Processing: CS algorithm.
Availability: Once per volume scan at a single elevation angle, as selected at the Master System Control Function
(MSCF); lowest elevation is default.
Presentation: Cartesian image of combined shear values (Figure 2-62).
Resolution: 0.5 x 0.5 km (0.27 x 0.27 nm).
Coverage: Radar centered, 230 x 230 km (124 x 124 nm) to 4 x 4 km (2.2 x 2.2 nm).
Data Levels: 16.
Annotations:
• Standard set*
• Data level code
• Elevation angle
• Position and magnitude of maximum shear value
• Site adaptable parameters
• Spatial resolution of product
Adaptation: None.
2.14.3.2 Usage. Visual identification of wind shear phenomena associated with gust fronts, downbursts, microbursts, synoptic fronts, and mesoscale/storm scale rotational phenomena.
2.14.3.3 Strengths/Applications. This product has been used to aid in manual gust front and mesocyclone and
misocyclone identification.
2.14.3.4 Limitations.
• For various reasons this product has not been thoroughly tested in an operational environment
• Extensive filtering of both velocity data and shear data may remove significant misoscale and mesoscale phenomena (i.e., 40 m (130 ft) to 4 km (2.2 nm))
• Critical values have not been established
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-211

October 2017


> **Figure 2-62 Combined Shear Product (CS #87).**

> An example Combined Shear product (OPUP display) from the Oklahoma City, OK (KTLX) WSR-88D

An example Combined Shear product (OPUP display) from the Oklahoma City, OK (KTLX) WSR-88D
on 04 May 1999 at 01:02 UTC .

2-212

FMH-11-Part C


#### 2.14.4 Mesocyclone.

This product was removed in RPG Build 12. The Mesocyclone product (M) provides information regarding the
existence and nature of vortices associated with thunderstorms. This product is generated from the output of the
legacy Mesocyclone Algorithm. The product provides information regarding identified shear features within the
storm. Features are classified as uncorrelated shear (sufficiently large, symmetrical, but not vertically correlated);
3-dimensional shear regions (vertically correlated, but not symmetrical); and mesocyclones (sufficiently large, vertically correlated, and symmetrical).
Using the Storm Tracking Information, Mesocyclone feature types are related to past feature types and tracked.
2.14.4.1 Operational Characteristics.
System ID: M, Product #62.
Data:
• Identified shear and mesocyclone features
• Mesocyclone Algorithm output
Processing: Legacy Mesocyclone Algorithm.
Availability: Updated once per volume scan.
Presentation: The product is generated in a format that provides an alphanumeric tabular display for all identified
features simultaneously, a graphic display, or a graphic overlay to other products. If there is no output from the
Mesocyclone Algorithm for a particular volume scan, a version of the product that exhibits the null condition is
produced (Figure 2-63).
• Formatted table of alphanumeric values.
• Graphic display of mesocyclone symbol and feature symbols at computed positions.
• Graphic overlay of mesocyclone symbol at its computed position.
Coverage: Radar centered, 230 km (124 nm) radius.
Data Levels: 3
• Mesocyclone
• 3-dimensional correlated shear
• Uncorrelated shear
Annotations:
• Standard set*
• Combined Attribute Table
• Site adaptable parameters
Special Symbols: The mesocyclone or 3D correlated shear symbol will be placed directly over the position of the
mesocyclone or shear at the lowest elevation scan in which it was detected.
• The mesocyclone symbol on the 230 km (124 nm) radius product is a yellow open circle with a 3/16-inch diameter, whose perimeter is 4 pixels wide. For a window product or a magnified base product, the size is proportional to the average of the radial and azimuthal diameters (if less than 3/16-inch, it defaults to 3/16-inch).
• The 3-dimensional correlated shear symbol on the 230 km (124 nm) radius product is a yellow open circle with
a 3/16-inch diameter, whose perimeter is 1 pixel wide. For a window product or a magnified base product, the
size is proportional to the average diameter (if less than 3/16-inch, it defaults to 3/16-inch).
Adaptation: See Section 3.5.2.1

2-213

October 2017

2.14.4.2 Usage. As an aid in the identification, alerting, and warning for mesocyclones which very often (>
90%) are correlated with severe weather (hail, damaging winds, and tornadoes). It appears that 10% - 15% are
associated with tornadoes.
2.14.4.3 Strengths/Applications.
• Identify mesocyclones. However, because this, like all algorithms, is imperfect the operator must examine reflectivity, velocity, and/or SRM to verify the existence of mesocyclones.
• Identify mid-level mesocyclones that develop downward toward the surface, some that become associated with
tornadoes.
• Because of the very frequent severe weather association with mesocyclones, detection of these features often
signals at least a severe thunderstorm.
• See Section 3.5.2.2 for a discussion of the operational strengths/applications of the algorithm.
2.14.4.4 Limitations.
• Time continuity is not employed. Under most circumstances feature verification requires that it be present for at
least two volume scans.
• While the legacy Mesocyclone Algorithm requires the vertical correlation of only two symmetric and sufficiently
strong features, normally a depth of at least 10,000 feet is needed for an operator-determined mesocyclone.
• The algorithm only detects cyclonic rotations even though anticyclonic circulations are sometimes also associated with severe thunderstorms.
• At longer ranges, mesocyclones may not be detected due to poor aspect ratios and the random beam placement
during radar sampling.
• Operator does not know which base-data elevation angle to examine for detection verification. Algorithm output is in terms of height and not scan angle.
• Successful detection of mesocyclones depends upon the selection of optimum high and low momentum and
other thresholds. Thresholds vary by storm type which itself can vary due to changes in airmass. Those airmasses can themselves change across the radar scanned coverage area.
• Range folding and velocity aliasing may interfere with successful mesocyclone detection. Velocity aliasing can
itself induce false algorithm detections.
• See Section 3.5.2.3 for a discussion of the operational limitations of the algorithm.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-214

FMH-11-Part C


> **Figure 2-63 Mesocyclone Product (M #62).**

> An example Mesocyclone product overlaid on a Storm-Relative Mean Radial Velocity Map product

An example Mesocyclone product overlaid on a Storm-Relative Mean Radial Velocity Map product
from the Paducah, KY (KPAH) WSR-88D on 07 May 2003 at 01:31 UTC (AWIPS display). Mesocyclone features are labeled with the associated storm label (P4, E2, Y3, and Y4). The Combined Attribute Table appears in the upper portion of the screen.

2-215

October 2017


#### 2.14.5 Mesocyclone Rapid Update.

This product was removed in RPG Build 12. The Mesocyclone Rapid Update product (MRU) is generated to provide legacy Mesocyclone Algorithm information after each elevation scan as opposed to only at the end of a volume
scan. The processing is based on data from the elevations that have been completed thus far in the current volume
scan. This information is combined with Mesocyclone and Storm Tracking Information from the previous volume
scan to form the MRU product.
The average motion of all SCIT-identified storm cells from the previous volume scan are used to derive a forecast position at the current volume scan time. In feature type order, the forecast position of each feature from
the previous volume is matched to the closest feature from the current volume scan, within a search radius defined
by the SCIT algorithm adaptation data. Current 3D features which are not matched to a feature from the previous volume scan are assigned the status of “New”. If previous volume scan data are unavailable, all features are
reported as New. Current features inherit the attributes of the matched previous feature (associated storm ID,
feature type, maximum tangential shear, height of maximum tangential shear, top height, base azimuth, base
range, base height, azimuth diameter, range diameter).
The position attributes (base azimuth, range, and height) of a previous feature matched to a current feature are
updated to the current detection. If the top height of the matched feature is higher, the feature top height is
updated. The position attributes of a previous feature not matched to a current feature is set to the extrapolated
forecast position. The statuses of unmatched previous features are assigned to “Extrapolated.” Strength attributes
are updated if they increase in magnitude.
The strength attributes are feature type and maximum tangential shear. If the maximum tangential shear is updated, the radial and azimuthal diameters and the height of the maximum tangential are also updated. Features with
increasing strength attributes are then assigned the status of Increasing. All other matched features are assigned the
status of Persistent. Attribute data updated with current volume data are so identified. At the end of the volume
scan extrapolated features are removed.
This product is generated in a format that can be used to generate an alphanumeric tabular display, a graphic display, or a graphic overlay to other products. On alphanumeric displays, the status (Persistent, Increasing, New,
or Extrapolated) of each feature is reported. In the graphic symbol display, feature status is reported as either
Extrapolated or Current. Current features include all features with a status of Increasing, Persistent, or New. If
on a particular elevation scan there is no output (i.e., no features of any type are identified), a version of the
product is produced that exhibits the null condition.
MRU Feature Status is defined as:
• NEW: A 3D feature detected in the current volume and not time associated with any feature from the previous
volume.
• PER: (Persistent) A feature found in both the current and previous volume scans and neither of the two
strength attributes have increased in the current volume. The two strength parameters are feature type and maximum tangential shear.
• INC: (Increasing): As PER but with one or both of the strength attributes increasing in intensity in the current
volume scan.
• EXT: (Extrapolated): A 3D feature found in the previous volume, but not yet time associated with any feature
from the current volume. The extrapolated feature position provided in the product is an estimated position
based on the time between the volume scans and the SCIT storm motion from the previous volume scan.
2.14.5.1 Operational Characteristics.
System ID: MRU, Product #139.
Data:
• Feature status
2-216

FMH-11-Part C

•
•

Identified shear and mesocyclone features
Meteorological detection algorithm output

Processing: Legacy Mesocyclone Algorithm and Storm Tracking Information.
Availability: Each elevation angle.
Presentation. This product is generated in a format that can be used to generate an alphanumeric tabular
display, a graphic display, or a graphic overlay to other products. On alphanumeric displays, the status (Persistent,
Increasing, New, or Extrapolated) of each feature status is reported. In the graphic symbol display, features
status is reported as either extrapolated or current. Current features include all features with a status of Increasing,
Persistent, or New. If on a particular elevation scan there is no output (i.e., no features of any type are identified),
a version of the product is produced that exhibits the negative condition. (A caret (^) symbol is used in the
MRU’s graphic attribute table and tabular alphanumeric product next to feature attributes updated with information from the current volume scan) (Figure 2-64).
• Formatted table of alphanumeric values
• Graphic display of mesocyclone symbol and feature symbols at computed positions
• Graphic overlay of mesocyclone symbol at its computed position
Coverage: Radar centered, 230 km (124 nm) radius.
Data Levels: 3
• Mesocyclone
• 3-dimensional correlated shear
• Feature status including New, Extrapolated, Persistence, or Increasing
Annotations:
• Standard set*
• Elevation angle
• Site adaptable parameters
Adaptation: None.
2.14.5.2 Usage. Provides mesocyclone updates with the completion of each elevation angle scanned thereby
enabling the operational meteorologist to make warning and other related decisions before the end of the radar
volume scan.
2.14.5.3 Strengths/Applications.
• Intermediate algorithm output is available before end of volume scan.
• MRU tracks features in order to check time continuity.
2.14.5.4 Limitations.
• Classification as Increasing or Persistent may be the result of sampling issues versus an actual feature attribute
change.
• The MRU graphical attribute table and alphanumeric attribute table contain attributes from both previous and
current volume scan information.
• Feature matching ability is dependent on motion supplied by SCIT algorithm which may be in error.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-217

October 2017


> **Figure 2-64 Mesocyclone Rapid Update Product (MRO #139).**

> An example Mesocyclone Rapid Update product overlaid on a Storm-Relative Mean Radial Velocity

An example Mesocyclone Rapid Update product overlaid on a Storm-Relative Mean Radial Velocity
Map product from the Paducah, KY (KPAH) WSR-88D on 07 May 2003 at 01:01 UTC (AWIPS display).

2-218

FMH-11-Part C


#### 2.14.6 Severe Weather Analysis.

These products were removed in RPG Build 11. The Severe Weather Analysis products (SWA) provide, at the highest product resolution available, separate maps of reflectivity, mean radial velocity, spectrum width, radial shear, or
storm-relative mean radial velocity for 50 x 50 km (27 x 27 nm) areas. When produced because it has been selected
for generation due to the detection of an alert condition, the product is automatically generated for the elevation
angle nearest the “critical altitude” for the meteorological phenomenon causing the alert and is centered at the coordinates of that phenomenon. The product can also be generated upon user request for a specified elevation angle
and geographic center point.
To aid in the visual interpretation of the mean radial velocity, the storm motion can be removed from the velocity
data by manually replacing the mean radial velocity panel with the SRR product.
2.14.6.1 Operational Characteristics.
System ID: SWA: SWR (reflectivity) #43, SWV (mean radial velocity) #44, SWW (spectrum width) #45, and SWS
(radial shear) #46.
Data:
• Mean radial velocity (m/s).
• Reflectivity (dBZ).
• Spectrum width (m/s).
Processing:
• Radial Shear algorithm
• Range-Altitude-Elevation Angle algorithm
Availability:
• Upon user request
• Automatically upon detection of an alerting phenomenon
Presentation: Four-quadrant, polar coordinate, image of data values; one data type per quadrant (window)
(Figure 2-65). The SWV product requires referencing to the RDA position for correct interpretation. Therefore, azimuths from the RDA with 10° intervals are placed on the image. Range marks at 21 km (11 nm) intervals
beginning at 21 km (11 nm) from the RDA are also included.
Resolution:
• SWR: 1 km (0.54 nm)
• SWV: 0.25 km (0.13 nm)
• SWW: 0.25 km (0.13 nm)
• SWS: 0.5 km (0.27 nm)
Coverage: 50 x 50 km (27 x 27 nm) window within 230 km (124 nm) of the radar. Product is centered
either at location of alerting phenomenon or user-requested coordinates.
Data Levels: 16.
Annotations:
• Standard set*
• Alert category
• Coordinates of product center
• Data level code for each product
• Elevation angle
2-219

October 2017

•
•

Height above ground level of center of window containing meteorological phenomenon
Maximum data values for each data type

Adaptation:
• Data levels (velocity only)
• Product center coordinates
• Velocity product type
2.14.6.2 Usage.
• Provides the user with the highest resolution radar information available for a severe weather event and allows
the user to see all radar moments simultaneously without paging through multiple products. Data are available as
a four quadrant product or as individual products.
• Intended primarily for the detailed analysis of severe convective weather.
2.14.6.3 Strengths/Applications.
• All three base product resolutions are the best available.
• Operator can examine three base products simultaneously allowing direct comparison.
• SRR can be used instead of Mean Radial Velocity
• Effective in investigating convective storms
• Displays product center height
• Provides ability to analyze and evaluate reflectivity, velocity, and spectrum width of a storm at various
heights.
2.14.6.4 Limitations.
• Product data resolution not identical
• Limited viewing area.
• Takes time to request, receive, and view the product.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.

2-220

FMH-11-Part C


> **Figure 2-65: Severe Weather Analysis Product (SWA: SWR 43, SWV 44, SWW 45, SWS 46).**

> An example Severe Weather Analysis product from the Paducah, KY (KPAH) WSR-88D on 07 May

An example Severe Weather Analysis product from the Paducah, KY (KPAH) WSR-88D on 07 May
2003 at 02:51 UTC (AWIPS display). This is an example of the 4-quadrant display composed of the
Severe Weather Reflectivity product in the upper left, Severe Weather Velocity in the upper right, Severe Weather Spectrum Width in lower left, and Severe Weather Radial Shear in the lower right. All of
the products shown are in the maximum data resolution.

2-221

October 2017


#### 2.14.7 Severe Weather Probability.

This product was removed in RPG Build 11. The Severe Weather Probability product (SWP) provides a
mapped display of the set of probability values calculated by the SWP algorithm for cells that the algorithm defined from the Vertically Integrated Liquid (VIL) array. The SWP value is not a true probability, but rather a measure of relative storm severity. The SWP values are directed related to the horizontal extent of VIL values greater
than a specified threshold. The displayed SWP values are assigned to the centers of the defined VIL cells.
2.14.7.1 Operational Characteristics.
System ID: SWP, Product #47.
Data: SWP values calculated by the SWP algorithm.
Processing:
• Output from VIL algorithm.
• SWP algorithm.
• Algorithm for placing SWP values on corresponding VIL cells.
Availability: Once per volume scan.
Presentation:
• Map of SWP values displayed at the centers of the corresponding VIL cells.
• Overlay of mapped SWP values (Figure 2-66).
Resolution: 4 x 4 km (2.2 x 2.2 nm)
Coverage: The SWP box size is adaptable and expandable in multiples of a 4 x 4 km (2.2 x 2.2 nm) box. The
default presentation ‘box’ is 28 x 28 km (15.1 x 15.1 nm). At full range, the minimum box size has to be 16 x 16
km (8.8 x 8.8 nm) to accommodate two 7 x 9 pixel integers. Radar centered, 230 km (124 nm) radius.
Data Levels: > 0 to < 100%.
Annotations:
• Standard set*
• Data level code
• Maximum data value detected (percent)
• SWP box size
Adaptation: See Section 3.5.3.1
2.14.7.2 Usage. SWP’s main strengths are its ability to automatically extract strong cells from the current radar
pattern, to draw a user’s attention to those cells, and, if the necessary VIL climatology is available, to provide objective guidance on the likelihood of severe weather associated with those cells.
2.17.7.3 Strengths/Applications. Quick identification of strongest storms.
2.14.7.4 Limitations.
• Only incorporates VIL; no environmental conditions considered.
• Since the SWP equation uses VIL calculations as input, inaccuracies in VIL values will affect the SWP output.
• Threshold values for severe storms will vary with differing climatology. Each site will have to establish the longterm climatology of VIL and severe weather.
• May be contaminated with non-precipitation echoes.
• Other considerations arising from the performance of the SWP algorithm are discussed in Section 3.5.3.3.
*Standard set includes Product Name; Radar ID; Date and Time of elevation, volume, or azimuthal scan; Radar
Position; Radar Elevation above MSL (feet); Operational Mode; and AVSET status flag, when appropriate.
2-222

FMH-11-Part C


> **Figure 2-66:Severe Weather Probability Product (SWP #47).**

> An example Severe Weather Probability product (OPUP display) from the Oklahoma City, OK (KTLX)

An example Severe Weather Probability product (OPUP display) from the Oklahoma City, OK (KTLX)
WSR-88D. The numbers west and northwest of the radar represent the percentage probability of severe weather based on the SWP Algorithm.

2-223

October 2017

2-224

Chapter 3: Meteorological and Hydrometeorological Algorithms

### 3.1 Introduction.

The WSR-88D provides the operational meteorological community with state of-the-art automation for processing
of dual pol Doppler weather radar data. The algorithmic processing derives explicit meteorological and hydrometeorological information and guidance from signal-processed data. This chapter provides information on many of
these algorithms.
Hydrometeorology is a combination of meteorology and hydrology. Briefly, meteorology deals with precipitation
before it reaches the ground, while hydrology deals with the precipitation once it reaches the ground (both surface
and ground water). The WSR-88D has a Snow Accumulation Algorithm (SAA) to estimate snow depth and (liquid)
snow water equivalent from radar data (described in Section 3.2.12). It also has two methods of estimating rainfall:
the Precipitation Processing Subsystem (PPS), described in Section 3.2.11, and Quantitative Precipitation Estimation (QPE), described in Section 3.3.3. The PPS relies on only base reflectivity, although it depends upon the Radar
Echo Classifier (REC, described in Section 3.2.10), which is based on reflectivity, velocity, and spectrum width for
determining the probability of ground clutter detection. The QPE is based on echo classification, which uses several polarimetric variables, and rainfall rate relationships derived from reflectivity, differential reflectivity, and specific
differential phase.
For further reference, Fulton et al. (1998) describes the PPS along with a few changes that were taking place in the
late 1990s. The PPS was further modified with the introduction of Enhanced Precipitation Preprocessing (EPRE),
which began deployment in 2004. Giangrande and Ryzhkov (2008) describe how QPE is determined from hydrometeor echo classifications.

### 3.2 Legacy-Derived Algorithms.


#### 3.2.1 Velocity Azimuth Display.

The Velocity Azimuth Display (VAD) Algorithm is used to obtain the vertical profile of horizontal wind speed,
direction, divergence and vertical velocity for the region of the atmosphere surrounding a Doppler radar. Velocity
data at different azimuths collected from a Doppler radar scanning the atmosphere at a constant elevation angle
about a vertical axis is used. This algorithm performs a harmonic analysis along with a best-fit test on the Doppler
velocities around the circumference of a circle at a specified slant range to obtain these parameters (Figure 3-1). The
vertical wind velocity is obtained through a series of steps involving the relationship between horizontal wind speed
and conservation of mass through a constant elevation surface above the radar.
As mentioned above, for each horizontal wind estimate, the VAD Algorithm performs a harmonic analysis, i.e.,
calculates the best fit sine wave regression equation, along with a best fit test on the Doppler velocities around the
circumference of a circle between the beginning and ending azimuths at a specified slant range and elevation angle.
This process is done for a specific slant range, range and altitude when a specific VAD product is requested (Figure
3-1). Areas of significant blockage are not used. The best-fit test uses the calculated RMS velocity to identify outliers, and eliminates them from the regression analysis. As a final check on the quality of the wind estimates produced, tests are made on the minimum number of data points used, fit symmetry, and Root Means Square (RMS)
velocity. If any one of these tests fails, the computed wind is displayed as “ND” (meaning “no data”) on the VAD
Wind Profile product (Figure 3-2).

3-1

October 2017


> **Figure 3-1: Velocity Azimuth Display Product.**

> This example of a Velocity Azimuth Display product (OPUP display) shows the fit of radial winds measured at a 5000 feet with respect to the theoretical sine wave curve.

This example of a Velocity Azimuth Display product (OPUP display) shows the fit of radial winds measured at a 5000 feet with respect to the theoretical sine wave curve.

3-2

FMH-11-Part C


> **Figure 3-2: Velocity Azimuth Display Wind Profile Product.**

> This example of a Velocity Azimuth Display Wind Profile product (OPUP display) shows the wind estimates produced in the vertical for the last 11 volume scans. “ND” indicates levels where the algorithm

This example of a Velocity Azimuth Display Wind Profile product (OPUP display) shows the wind estimates produced in the vertical for the last 11 volume scans. “ND” indicates levels where the algorithm
was not able to produce a reliable wind estimate.

3-3

October 2017

3.2.1.1 Operational Parameters.
• AZIMUTH: Azimuthal position, in radians.
• HEIGHT (Radar): The radar height above sea level, in km.
• DENSITY (Atmospheric): A set of density values for each altitude, in kg/km3.
• DENSITY (Atmospheric Gradient): A set of density gradient values at each altitude, in kg/km4.
• ELEVATION: Elevation angle, in radians.
• FIT TESTS: The number of times the fit test procedure is to be run (2).
• REFLECTIVITY FACTOR (ZE): The effective radar reflectivity factor of a SAMPLE VOLUME, in mm6/
m3.
• THRESHOLD (Begin in Azimuth): Starting azimuth for VAD analysis, radians.
• THRESHOLD (Data Points): The minimum number of data points allowed for the Fourier least squares fitting,
dimensionless (25).
• THRESHOLD (End Azimuth): Ending azimuth for VAD analysis, in radians.
• THRESHOLD (Symmetry): A value for determining symmetry, in km/hr (25.2).
• THRESHOLD (Velocity): A RMS velocity threshold (18), in km/hr.
• TIME (Scan): The beginning time of a scan, in hours. Precise to 1/3600 hr.
• VAD (Analysis Ranges): The set of specific slant range(s) for each elevation angle at which horizontal wind estimates are computed. The VAD Range is included in the VAD Analysis Ranges, in 1/4 km.
• VAD RANGE: The subset of VAD (Analysis Ranges) used to compute estimates of vertical velocity and divergence, in 1/4 km.
• VELOCITY (Doppler): Doppler velocities in a SAMPLE VOLUME, in km/hr.
3.2.1.2 Strengths/Applications.
• Using the velocity and reflectivity data points, the velocity Azimuth Display Algorithm attempts to calculate the
wind direction and speed at an operator selected height at a selectable but nearby radius from the radar.
• VAD Winds are available in clear air or precipitation mode. Generally speaking, the wind estimates will be slightly better in clear air mode since the radar antenna rotation is slower.
• The VAD Algorithm does not require 360 degrees of data. The algorithm only requires 25 (adaptable) data
points (a sample from 25 degrees of azimuth), and they don’t have to be contiguous. It is possible to only
sample a certain sector to produce the VAD winds e.g., 135° to 225° to get an estimate of the winds ahead of
a front. The “Beginning” and “Ending” azimuth is set at the RPG HCI (under Unit Radar Committee (URC)
control).
• Update Environmental Winds Table. The VAD winds are fed into the Environmental Winds Table for use in
the velocity dealiasing algorithm. This helps minimize dealiasing errors.
• VAD winds are included on the Radar Coded Message (RCM).
3.2.1.3 Limitations.
• Needs sufficient data points - Clear, cold, dry air often lacks scatterers. No sine wave will be plotted unless there
are at least 25 data points (default parameter).
• May be unreliable in disturbed environments. The algorithm does not account for deformation but assumes
horizontal uniformity of the wind field. If there is a front or boundary near the RDA, the data will often fail
either RMS or symmetry thresholds.
• Available for pre-established altitudes only. As designated at the RPG HCI for the VAD Wind Profile.
• Large flocks of migrating birds may produce anomalous wind data. The averaging of the motion of birds in
conjunction with the motion of the wind, can lead to erroneous wind data. Birds can cause the speed to be
off by several knots and the direction to be off by several degrees. Typical symptoms include an “explosion of
reflectivity returns in an expanding “donut” pattern centered on the RDA just after sunset.
3-4

FMH-11-Part C


#### 3.2.2 Echo Tops.

The Echo Tops (ET) Algorithm requires only reflectivity data. The reflectivity data values for each sample volume
are assigned a number representing fifteen categories. If the return power is less than a threshold (defaulted to 18.5
dBZ) it is assigned a value of zero. The data are also filtered to remove any spurious data. A data point must have
at least two adjacent points with a category value of at least one, otherwise the isolated point is removed. That is, at
least two of the four possible sample volumes at a particular elevation with adjoining sides to the sample volume in
question must be in category one or above. This final data set is the one used as input to the Echo Tops Algorithm.
The Echo Tops Algorithm itself estimates the maximum echo top height for each 4 x 4 km (2.2 x nm) grid box in
an array covering a radius of 230 km (124 nm) from the radar. For each elevation scan, the algorithm checks each
reflectivity factor value against a “Minimum Significant Reflectivity” threshold. For those sample volumes meeting
this threshold, an echo top height is calculated from sea level to the center of the beam and mapped to the appropriate grid box. The final value of each grid box is the maximum echo top height mapped onto it.
3.2.2.1 Operational Parameters.
• AZIMUTH: Azimuthal position, in radians. Precise to 10-3 radians.
• BEAM WIDTH: The angular width of the radar beam between the half-power points, in radians (0.017).
• SAMPLE VOLUME: A data sample volume whose dimensions are 1 degree in azimuth, 1.0 km in range, and 1
degree in depth (perpendicular to the radar beam).
• CATEGORIES: Categorized effective reflectivity factor data for each SAMPLE VOLUME.
• ELEVATION: Elevation angle, in radians.
• RANGE (Slant): The slant range to the center of a SAMPLE VOLUME, in km.
• BOX (4 km x 4 km (2.2 nm x 2.2 nm)): Square grid boxes which are 4 km on a side and cover ranges from 0 to
230 km (124 nm). The intersection of 4 boxes is centered on the radar location.
• MINIMUM SIGNIFICANT REFLECTIVITY: The minimum reflectivity considered to be non-zero for the
purpose of determining the echo top; ranges from -33.0 to 94.0 dBZ; default, 18.3 dBZ.
3.2.2.2 Strengths/Applications.
• The Echo Tops Algorithm estimates of the Echo Top heights for each 4 x 4 km (2.2 x 2.2 nm) grid box.
• Quick estimation of the most intense convection; higher echo tops.
• Assist in differentiating non-precipitation echoes from real storms.
• Aids in identification of storm structure features such as tilt, updraft flank, max top over strong low-level reflectivity gradient, etc.
• May detect mid-level echoes before low-level echoes are detected.
3.2.2.3 Limitations.
• The precision for measuring the height decreases with range because of beam broadening. At a range of 230
km (124 nm), the one-half power beam width is 1980 m (6,500 ft).
• The algorithm does not correct for data contamination from side lobes. The height of echo tops could be overestimated from this effect.
• Tops will be underestimated close to the radar due to the cone of silence.
• Due to the lack of vertical extrapolation and gaps between beam placements, all tops are along the beam resulting in a “stair-step” appearance, at times concentric rings, and often considerable underestimates of the actual
echo tops.
• No upward extrapolation from the last elevation angle where precipitation was detected.
• Difficult to locate the highest echo top in a storm due to lack of upward vertical extrapolation and lack of precsision (heights are displayed in 5000 ft increments).
• Echo top heights from this algorithm do not have enough precision to be used reliably for severe weather warnings.
3-5

October 2017


#### 3.2.3 High Resolution Enhanced Echo Tops.

The High Resolution Enhanced Echo Tops (EET) Algorithm ingests processed reflectivity factor data from the
Data Quality Assurance (DQA) Algorithm (see Section 4.3.1). The DQA Algorithm identifies and removes radials
contaminated with constant power function signatures as well as regions of anomalous propagation clutter (and
other contaminants) from the reflectivity factor data. Upon completion of each elevation cut of the radar volume,
the DQA passes the processed data to receiving algorithms as an elevation cut. The original radar volume’s elevation
cut spatial integrity is maintained. The reflectivity factor data provided after DQA processing are not quantized. The
original resolution of the reflectivity factor data is maintained.
The EET produces an output product of estimated echo top heights on a 1° x 0.54 nm (1 km) polar grid resolution
to a range of 186 nm (345 km). Each point of the EET product represents processing through a vertical column of
the radar volume. Each column is populated by range gate sample volumes from the intersected elevation tilt planes
of the radar volume. The EET determines the altitudes along the elevation tilt plane for range gate sample volumes
whose reflectivity factor equals or exceeds the reflectivity factor threshold. The EET analyzes vertically upwards
through successive elevation tilt planes in this manner. For any vertical column, altitude is determined through
vertical linear interpolation between successive elevation tilt planes when the reflectivity factor threshold is exceeded
for the lower plane and not exceeded for the upper plane. This processing functionality mitigates the computationally-introduced “stair-step” artifact of the ET product.


> **Figure 3-3: Enhanced Echo Tops Algorithm.**

> The EET determines the proper echo top height (blue circles) within the radar volume by selecting a

The EET determines the proper echo top height (blue circles) within the radar volume by selecting a
range gate along a radial or interpolating in the vertical between two on adjacent elevation tilt planes
that bisect the threshold level.
3-6

FMH-11-Part C


> **Figure 3-3 illustrates the method used by EET to determine the echo top height. An echo top deck (dark line) of**

> constant altitude defined by the 18.5 dBZ threshold is modeled with higher reflectivity factor below and lesser

constant altitude defined by the 18.5 dBZ threshold is modeled with higher reflectivity factor below and lesser
reflectivity factor above. The orange lines represent individual elevation tilt planes within the radar volume. Circles
along the tilt planes represent (not to scale) sections of individual range gate sample volumes comprising the radial.
The blue circles represent the echo top height determined by the EET Algorithm for this case. In the highlight box,
the EET echo top height is indicated with the blue circles based on vertical linear interpolation between the higher reflectivity factor sample volumes on the lower elevation angle tilt plane and the lower reflectivity factor sample
volumes on the higher elevation angle tilt plane. The diagonally downward pointing arrow from the left-most blue
circle to the left-most yellow circle on the lower angle elevation tilt plane is provided to contrast EET with the
“stair-step” altitude selection methodology of the ET product.
The EET provides the echo top height in 1 kft resolution along with a flag value to indicate if the echo top height is
“topped”. A “topped” echo top occurs when the last elevation tilt plane has range gate sample volumes with reflectivity factor that meets or exceeds the threshold. In the case of the Figure 3-3, the “topped” EET echo top heights
are all along the left-most elevation tilt plane below the actual modeled echo top height. This region is the “coneof-silence” for the radar and typically will have “topped” echo top heights during significant convection in close
proximity to the radar. The EET product is provided as a bzip2 compressed product for external user systems.
3.2.3.1 Operational Parameters. None.
3.2.3.2 Strengths/Applications.
• A vertical linear interpolation technique is utilized to estimate echo top heights between elevation tilt planes.
• An indicator if an echo top height is “topped”.
• High spatial resolution of 1° x 0.54 nm (1 km).
• Data resolution of 1 kft.
3.2.3.3 Limitations.
• The precision for measuring the height decreases with range because of beam broadening. At a range of 230
km (124 nm) the half-power beam width is 4040 meters.
• The algorithm does not correct for data contamination from side lobes. The height of echo tops could be overestimated from this effect.
• The echo tops may be undetectable by the radar at close range with a limited range of elevations.
• The vertical change of reflectivity factor may not be linear.
• No adaptable parameters are available.

#### 3.2.4 Hail.

The purpose of the Hail Detection Algorithm (HDA) is to provide for each storm cell the following three estimates:
• The Probability of Hail (POH) of any size,
• The Probability of Severe Hail (POSH) (or hail ¾” in diameter), and
• The Maximum Expected Hail Size (MEHS).
Based on drop-size/hailstone distribution and empirical studies, the algorithm assumes that large reflectivity values
observed aloft (above the freezing level [0°C]) are most likely hail in the midst of large concentrations of supercooled liquid water.
This algorithm analyzes storm cell and environmental data available in a specific format. The STORM CELL CENTROIDS Algorithm (see Section 3.2.7.2) provides storm cell data as input to this algorithm. The SCIT Algorithm
(see Section 3.2.7) identifies individual cells within a convective storm instead of the entire storm. A storm cell
is defined as a core of a three dimensional region of significant reflectivity values. Each storm cell is made up of
two-dimensional components in horizontal proximity at adjacent elevation angles of radar observation. A compo3-7

October 2017

nent is a minimum aerial extent of reflectivity values greater than or equal to a specific reflectivity threshold at one
elevation. The algorithm’s inputs are environmental data and storm cell components’ maximum reflectivity and
height ARL (of the mass weighted center (or centroid)). The environmental data are the height above mean sea level
(MSL) of the 0°C and -20°C environmental temperatures (which is usually derived from a nearby sounding).
To determine the POH of any size for each storm cell, the height of the highest component with a large maximum
reflectivity value (of at least a threshold value) which is above the freezing level is used in an empirical relationship.
The higher the component is above the freezing level, the greater the POH.
To determine the POSH and MEHS for each storm cell, the algorithm uses a relationship between reflectivity
and the Hailfall Kinetic Energy (HKE) in order to calculate E, which is the flux of kinetic energy of hailstones.
E is calculated from components with large maximum reflectivity values (of at least a threshold value) above the
freezing level. The larger the components’ maximum reflectivity values, the larger their E. A height and reflectivity
weighted vertical integration of the E is done for all components within a storm cell (which meet the relative height
and reflectivity criteria). The vertical integration of E is weighted toward components with very large (of at least a
threshold value) maximum reflectivity values above the height of the -20°C environmental temperature (Figure 3-4).
The vertical integration results in a parameter called the Severe Hail Index (SHI). The greater the collective depth of
components in a storm cell with large E values and the higher those components are (above the freezing level), the
larger a storm cell’s SHI value. The POSH is calculated from SHI and a threshold which is a function of the height
of the freezing level. The MEHS for each storm cell is computed using SHI in an empirical formula. The algorithm
is designed to work independent of cell type, tilt, and overhang. The primary product produced by the algorithm is
Hail Index (HI) which can be useful in identifying cells that have the potential to produce hail. The Hail Index Attribute Table will be available at the top of the product which lists the Cell ID, Azimuth and Range, POSH or POH,
the MEHS, and the last line in the table identifies the altitudes of the temperatures and the date/time at which the
information was last updated (see Figure 3-5). If the cell is beyond the hail processing range of 230 km (124 nm),
then the hail estimates are labeled as UNKNOWN in the Attribute Table.

3-8

FMH-11-Part C


> **Figure 3-4: Hail Detection Algorithm Process**


3-9

October 2017


> **Figure 3-5: Hail Index Product.**

> This Hail Index product (OPUP display) example from the Oklahoma City, OK (KTLX) WSR-88D on

This Hail Index product (OPUP display) example from the Oklahoma City, OK (KTLX) WSR-88D on
31 May 2013 at 23:09 UTC includes identification numbers for identified storms. The hail symbol is
a green isosceles triangle, filled or unfilled depending on adaptable probabilities of severe hail and
probabilities of hail. In addition, the maximum expected hail size rounded to the nearest inch is displayed in the middle of the triangle. The Combined Attribute Table provides alphanumeric storm information.
3.2.4.1 Operational Parameters.
• HEIGHT (0° C): The height of the 0° C environmental temperature (or freezing level), in km AGL.
• HEIGHT (-20° C): The height of the -20° C environmental temperature, in km AGL.
• HEIGHT (Component): The height of the center of mass of a component, in km AGL.
3-10

FMH-11-Part C

•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•
•

HKE COEFFICIENT #1: A multiplicative factor used in computing the Hailfall Kinetic Energy (5x10-4).
HKE COEFFICIENT #2: A multiplicative exponential factor used in computing the Hailfall Kinetic Energy
(8.4x10-2).
HKE COEFFICIENT #3: An operand factor used in computing the Hailfall Kinetic Energy (10).
Maximum REFLECTIVITY (Component): Maximum (averaged) reflectivity value detected in an individual
COMPONENT, in dBZe.
POSH COEFFICIENT: A multiplicative factor used in computing the POSH from the SHI (29).
POSH OFFSET: An offset used in computing the POSH from the SHI (50), in percent.
SHI HAIL SIZE COEFFICIENT: A multiplicative factor used in calculating the MEHS from the SHI (0.1).
SHI HAIL SIZE EXPONENT: The power to which the SHI is raised in calculating the Maximum Expected
Hail Size from the SHI (0.5).
STORM CELL: A STORM CELL is a three dimensional region composed of COMPONENTs characterized
by reflectivity values above a given threshold.
COMPONENT (Storm Cell): A COMPONENT is a two dimensional region of a STORM CELL which meets
reflectivity and area thresholds, ordered from lowest to highest for each STORM CELL.
THRESHOLD (HKE Reflectivity Weighting Lower Limit): The lower limit of reflectivity values used in the
reflectivity weighting function for the POSH calculation (40), in dBZe.
THRESHOLD (HKE Reflectivity Weighting Upper Limit): The upper limit of reflectivity values used in the
reflectivity weighting function for the POSH calculation (50), in dBZe.
THRESHOLD (minimum Reflectivity POH): Minimum maximum REFLECTIVITY (Component) used in the
calculation of the POH (45), in dBZe.
THRESHOLD (POH Height Difference #1): Maximum height difference which correlates to 0% POH (1.625),
in km.
THRESHOLD (POH Height Difference #2): Maximum height difference which correlates to 10% POH
(1.875), in km.
THRESHOLD (POH Height Difference #3): Maximum height difference which correlates to 20% POH
(2.125), in km.
THRESHOLD (POH Height Difference #4): Maximum height difference which correlates to 30% POH
(2.375), in km.
THRESHOLD (POH Height Difference #5): Maximum height difference which correlates to 40% POH
(2.625), in km.
THRESHOLD (POH Height Difference #6): Maximum height difference which correlates to 50% POH
(2.925), in km.
THRESHOLD (POH Height Difference #7): Maximum height difference which correlates to 60% POH (3.3),
in km.
THRESHOLD (POH Height Difference #8): Maximum height difference which correlates to 70% POH (3.75),
in km.
THRESHOLD (POH Height Difference #9): Maximum height difference which correlates to 80% POH (4.5),
in km.
THRESHOLD (POH Height Difference #10): Maximum height difference which correlates to 90% POH (5.5),
in km.
WARNING THRESHOLD SELECTION MODEL COEFFICIENT: A factor multiplied by the HEIGHT
(0°C) in the Warning Threshold Selection Model (57.5x102), in Jm-2s-1.
WARNING THRESHOLD SELECTION MODEL OFFSET: An offset used in the Warning Threshold Selection Model (-121x105), in Jm-1s-1.
3-11

October 2017

3.2.4.2 Strengths/Applications.
• In operational use, the MEHS parameter has provided a useful rough estimate of the maximum hail size.
• The HDA has shown a very high probability of detection in cells that contain severe hail, especially greater than
one-inch diameter hail. A POSH of 50% or greater has shown skill as a warning threshold.
3.2.4.3 Limitations.
• The HDA needs as input, accurate and timely measurements of the MSL altitudes for the 0°C and -20°C levels.
Failure to update this information will degrade the algorithm’s performance.
• Values of POH, POSH, and MEHS will fluctuate at close ranges, especially in VCP 21, due to gaps in coverage
at higher elevation slices.
• The values for POH, POSH, and MEHS may fluctuate at longer ranges from the radar due to the limited number of slices through the cell.
• The maximum hail processing range is 230 km (124 nm). For cells beyond 230 km (124 nm), hail will be identified as UNKNOWN.
• POSH and MEHS tend to overestimate the chances and size of hail in weak wind and tropical environments
where freezing levels are higher and more melting during descent occurs. The accuracy of the hail estimates
partially depends upon the accuracy of cell (component) information. MEHS is an estimation of the largest hail
in the cell, and often times, most of the hail from a cell is smaller.

#### 3.2.5 Vertically Integrated Liquid Water.

The Vertically Integrated Liquid Water (VIL) Algorithm requires only reflectivity data. The reflectivity data values
for each sample volume are assigned a number representing 15 display categories. If the return power is less than
a threshold (defaulted to 18.5 dBZ) it is assigned a value of zero. If the return power is greater than or equal to 56
dBZ, the corresponding liquid water value is assigned a value of 5.4 gm-3. The categorization of the data are performed using a look-up table to arrive at liquid water values. The filtering routine removes any spurious data. A data
point must have at least two adjacent points with a category value of at least one, otherwise the isolated point is
removed. That is, at least two of the four possible sample volumes at a particular elevation with adjoining sides to
the sample volume in question must be in category one or above. This final data set is the one used as input to the
VIL Algorithm.
The VIL Algorithm converts weather radar reflectivity factor data into liquid-water content values based on theoretical studies of drop-size distributions and empirical studies of the relationship between reflectivity factor and
liquid-water content. The algorithm described uses an equation that relates reflectivity factor to liquid water content
for one such relationship:
M = 3.44 x 10-3 Z4/7
where M = liquid water content (gm-2) Z = radar reflectivity (mm6m-3).
The values are derived for each 4 x 4 km (2.2 x 2.2 nm) grid box; then vertically integrated. VIL values are output in
units of mass per area (kg m-2). The algorithm assumes reflectivity returns are from liquid water.
For each elevation angle, each 4 x 4 km (2.2 x 2.2 nm) grid box is assigned the largest liquid water value of all the
sample volumes located within the grid box. However, a liquid water value may not exceed 5.4 gm-3 (or equivalently,
reflectivity values are capped at 56 dBZ.) All other liquid- water values in each grid box are ignored. This attempts
to compensate for the fact that the storm may be tilted and moving during the time required for a complete volume
scan. These partial liquid-water content values are then integrated vertically to arrive at VIL values for each grid box.
If the VIL value for a grid box exceeds 80 kgm-2 it is adjusted to 80 kg m-2 to mitigate the large reflectivity values
associated with hail.
3.2.5.1 Operational Parameters.
• AZIMUTH: Azimuthal position, in radians. Precise to 10-3 radians.
• BEAM WIDTH: The angular width of the radar beam between the half-power points (0.017), in radians.
3-12

FMH-11-Part C

•
•
•
•
•

SAMPLE VOLUME: A data sample volume whose dimensions are 1 degree in azimuth, 1.0 km in range, and 1
degree in depth (perpendicular to the radar beam).
RANGE (Slant): The slant range to the center of a SAMPLE VOLUME, in km.
BOX (4 km x 4 km [2.2 nm x 2.2 nm]): Square grid boxes which are 4 km on a side and cover ranges from 0 to
230 km (124 nm).
REFLECTIVITY FACTOR (ZE): The effective radar reflectivity factor of a SAMPLE VOLUME, in mm6/m3.
(Maximum value of 56 dBZ.)
Maximum Liquid Water Threshold: 1 to 200 kgm-2; default, 80 kgm-2: Threshold against which integrated liquid
water is tested. Maximum allowable computed VIL.

3.2.5.2 Strengths/Applications.
• A set of VIL values corresponding to the boxes (4 km x 4 km [2.2 nm x 2.2 nm] Grid) is output by this algorithm.
• Locate the most significant storms. (Best used when comparing storms located at about the same range). High
VIL values correspond to deep areas of high reflectivity indicative of strong updrafts. VIL Density (VIL divided
by Echo Tops) has also shown some skill indicating significant storms.
• Useful for distinguishing storms with large hail once threshold values have been established. Establishing a VIL
of the Day using climatological data and/or sounding data can be of some limited use for initial development,
but better skill can be achieved by real-time comparison between VIL values and spotter reports.
• Persistent high VIL values are associated with supercells. Rapid decrease in VIL values may signify the onset of
wind damage. (Use caution with this technique because gaps in the current VCP 21 can create abrupt decreases
in VIL values.)
3.2.5.3 Limitations.
• Hail can produce fictitious values of liquid water due to enhanced reflectivity values. Therefore, a maximum
value of 80 kgm-2 is set as a ceiling of the display to mitigate this effect.
• Except for the lowest tilt, the current implementation has no earth curvature correction, i.e., the earth is considered flat when mapping data in polar coordinates to the rectilinear coordinates. The values obtained at distant
ranges may be misleading because liquid water below the radar beam is not sensed.
• Large radar sample volumes may cause errors at long ranges.
• Values for warnings may change daily and across the warning area. Values are air mass dependent. An important
consideration is the need to quantify the regional, seasonal, diurnal, and air mass variations in VIL magnitude
versus severe weather. Under differing meteorological conditions, the minimum VIL associated with severe
weather may vary considerably. Warm, moist environments tend to require higher VIL values for severe weather
occurrences than do cool, dry conditions.
• Values are range dependent. Values within 40 km (20 nm) of the radar are underestimated. This is due to the
cone of silence.
• Values at longer ranges are occasionally unreliable. The reflectivity value at 0.5° is integrated down to the
ground. At distant ranges the beam may be cutting through the highly reflective hail cores in the mid-levels of a
storm producing an overestimation of VIL. With very low- topped convection VIL values may be underestimated at long ranges.
• Cell-based VIL values are computed from the vertical component of an identified cell. Consequently, the
Grid-based VIL values will differ from Cell-based VIL values, in shear situations where the cells are tilted in the
vertical.
• VIL values for a strongly tilted or a fast moving storm will be lower than if the storm was vertical or moving
slower. The upper portion of the storm may extend into another grid box.
• May be contaminated by non-precipitation echoes.
• More VIL fluctuation with VCP 21 than VCP 11 and VCP 12. There are fewer gaps in VCP 11 and VCP 12.
This is mainly within 60 nm of the radar. An example of this phenomenon is shown in Figure 3-31.
3-13

October 2017


#### 3.2.6 Digital High Resolution Vertically Integrated Liquid Water.

The Digital High Resolution Vertically Integrated Liquid Water (DVL) Algorithm ingests processed reflectivity factor data from the DQA Algorithm. The DQA Algorithm identifies and removes radials contaminated with constant
power function signatures as well as regions of anomalous propagation clutter (and other contaminants) from the
reflectivity factor data. Upon completion of each elevation cut of the radar volume, the DQA passes the processed
data to receiving algorithms as an elevation cut. The original radar volume’s elevation cut spatial integrity is maintained. The reflectivity factor data provided after DQA processing are not quantized. The original resolution of the
reflectivity factor data are maintained.
The DVL produces an output product of a digitally encoded estimate of VIL on a 1° x 0.54 nm (1 km) polar grid
resolution to a range of 460 km (248 nm). Each point of the DVL product represents processing through a vertical
column of the radar volume. Each column is populated by a set of range gate sample volumes from each intersected elevation tilt plane of the radar volume. The DVL determines the partial VIL contribution from each intersected
elevation tilt plane of the column by selecting the range gate sample volume with the largest reflectivity factor, converting it to equivalent liquid water, and vertically integrating through the depth of the range gate sample volume.
The calculation of the partial VIL is the same as that done for the VIL Algorithm except that DVL uses non-quantized reflectivity factor data and includes the conversion to VIL of reflectivity factor below 18 dBZ threshold and
above the greater dBZ (i.e., all reflectivity used).
At completion of the last elevation tilt plane, DVL tallies the partial VIL contributions in each column to arrive at
the column total VIL in kgm-2. The total VIL value is capped at 80 kgm-2 if exceeded. The resolution of DVL varies
due to the digital encoding. The DVL digitally encodes the data using a linear scale for VIL less than 0.19 kgm-2
and a log scale thereafter up to the cap threshold. The DVL product is provided as a bzip2 compressed product for
external user systems.
3.2.6.1 Operational Parameters. None
3.2.6.2 Strengths.
• High spatial resolution of 1o x 0.54 nm (1 km).
• Inclusion of reflectivity factor below 18 dBZ heightens depiction of incipient development of convective
weather and enhances use of VIL in winter weather situations.
• Provides fine scale depiction of VIL at levels below that depicted by the minimum threshold of the original
VIL product.
• Use of DQA mitigates impact of data contaminants on VIL calculation.
3.2.6.3 Limitations.
• This algorithm has a bias towards larger drop sizes. Clouds containing a large number of small precipitation
drops produce very small values.
• The values obtained at distant ranges might be misleading (biased low) because a portion of the total storm
liquid water is not measured below the lowest radar beam.
• The values obtained at close ranges might be misleading (biased low) because a portion of the total storm liquid
water is not measured above the highest radar beam.
• Large radar sample volumes may cause errors at long ranges.
• No dual pol-based hydrometeor classification is used to distinguish liquid from solid scatterer returns. All reflectivity is treated as liquid scatterers.
• There are no adaptable parameters.

#### 3.2.7 Storm Cell Identification and Tracking.

The objective of the Storm Cell Identification and Tracking (SCIT) Algorithm is to identify, track, and forecast the
movement of storm cells. The primary graphic product produced by this algorithm is Storm Track Information
(STI). The STI product can display up to 100 cells identified by the SCIT Algorithm on a single product. It is also
3-14

FMH-11-Part C

possible to display the actual past positions of the centroid on up to 13 (default 10) previous volume scans. Data
developed by this algorithm are used extensively as input to several other products (i.e., HI, SS, SRM, SRR, M, TVS,
RCM, CR Combined Attribute Table).
A storm cell is defined as a 3-dimensional region of significant reflectivity values above a specified threshold. It is
assumed to be made up of reflectivity radial runs called segments and in turn 2- dimensional storm components
composed of segment groups and occurring at different radar elevation angles. These components with calculated
mass weighted centroids are then vertically correlated into a cell with an established centroid.
The SCIT Algorithm consists of four sub-functions: Storm Cell Segments, Storm Cell Centroids, Storm Cell Tracking, and Storm Position Forecast (Figure 3-6). The Storm Cell Segments sub-function identifies the radial sequences
of reflectivity (segments), and outputs information on these segments to the Storm Cell Centroids sub-function.
The Storm Cell Centroids sub-function groups the segments into two-dimensional components, vertically correlates
these components into three-dimensional cells, and calculates these cells’ attributes. The cells and their attributes are
output to Storm Cell Tracking and Storm Position Forecast. Storm Cell Tracking monitors the movement of the
cells by matching cells found in the current volume scan to the cells from the previous volume scan. Storm Position
Forecast predicts future centroid locations based on a history of the cell’s movement.
The algorithm proceeds stepwise:
1. Storm Cell Segments - Identify segments along a radial of continuous reflectivity above the minimum reflectivity
threshold and then identify reflectivity regions composed of adjacent segments by correlating the segments azimuthally.
2. Storm Centroids - Classify and rank vertically correlated reflectivity regions as storms.
3. Storm Tracking - Correlate storms from scan-to-scan to establish tracks.
4. Storm Position Forecast - Project the track forward in time. Below, each of the sub-algorithms are discussed in their
order of processing.

3.2.7.1 Storm Cell Segments.
This algorithm defines a radial processing technique which identifies radial sequences of reflectivity, or segments,
as part of the processing to identify storm cells. These segments are runs of contiguous sample volumes with
reflectivity values greater than or equal to a specified threshold and have a combined length greater than a specified
segment length threshold. Also, a segment may contain a specified number of contiguous sample volumes which are
within a specified dropout reflectivity value below the reflectivity threshold.
The algorithm has multiple reflectivity thresholds (and a minimum segment length threshold for each reflectivity
threshold). For each elevation scan the algorithm searches for segments using each of the reflectivity thresholds as a
minimum value (Figure 3-7).

3-15

October 2017


> **Figure 3-6: Storm Cell Identification and Tracking Algorithm Overview**



> **Figure 3-7: Storm Cell Segments Algorithm**

> 3-16

3-16

FMH-11-Part C

Reflectivity information is quantified. The basic measurements of reflectivity are made in 1° x 0.54 nm (1 km) range
bins. The function of this algorithm is to combine the individual range bins into storm segments along the radial.
Note the segments along individual radials.
For each segment, the following attributes are calculated and saved: maximum reflectivity (using a three (adaptable)
gate average), mass weighted length, and mass weighted length squared. In addition to those calculated attributes,
the following attributes are saved for each segment: azimuth, reflectivity threshold, beginning range, and ending
range. These attributes are used as inputs by the STORM CELL CENTROIDS Algorithm where radially adjacent
segments are combined into storm components.
3.2.7.1.1 Operational Parameters.
• AZIMUTH: The azimuthal position of a radial or CELL SEGMENT, in degrees.
• DROPOUT NUMBER THRESHOLD: 1 to 4; default, 2. The maximum number of sample volumes below
the minimum reflectivity threshold and above the dropout reflectivity threshold that may be included in a storm
segment.
• ELEVATION: The angle of the elevation scan, in degrees.
• MASS MULTIPLICATIVE FACTOR: multiplicative factor used in computing the PRECIPITATION INTENSITY for the MASS calculation (486), in (mm6/m3)(hr/mm).
• MASS WEIGHTED FACTOR: A factor used in computing the MASS of a SAMPLE VOLUME (53 x 103), in
(hr)(kg)/(km2m2).
• PRECIPITATION INTENSITY EXPONENT: The power to which the PRECIPITATION INTENSITY is
raised in calculating the effective REFLECTIVITY FACTOR (1.37), in dBZe.
• RANGE (Slant) Slant range to the center of a SAMPLE VOLUME, in km.
• REFLECTIVITY AVERAGING FACTOR: The number of SAMPLE VOLUMES used for computing a segment’s maximum (average) REFLECTIVITY FACTOR (3).
• REFLECTIVITY FACTOR (Sample Volume): The effective radar reflectivity factor of a SAMPLE VOLUME,
in dBZe.
• SAMPLE VOLUME: A data sample volume whose (half power) dimensions are 1 km (0.54 nm) in range (or
length) and approximately 1° in azimuth and depth (perpendicular to the radar beam).
• THRESHOLD (Dropout Count): The maximum number of contiguous SAMPLE VOLUMEs with a REFLECTIVITY FACTOR less than the THRESHOLD (Reflectivity) by less than or equal to the THRESHOLD
(Dropout Reflectivity Difference) that may be included in a CELL SEGMENT (2).
• THRESHOLD (Dropout Reflectivity Difference): The difference below THRESHOLD (Reflectivity) in effective reflectivity that a SAMPLE VOLUME may still be included in a CELL SEGMENT (5), in dBZe.
• THRESHOLD (maximum Reflectivity Mass): The maximum REFLECTIVITY FACTOR used in the MASS
WEIGHTED LENGTH and MASS WEIGHTED LENGTH SQUARED calculations (80), in dBZe.
• THRESHOLDS (Reflectivity): a set of minimum effective reflectivities which the REFLECTIVITY FACTOR
of a SAMPLE VOLUME must meet or exceed to be included in a CELL SEGMENT. The REFLECTIVITY
FACTOR of the SAMPLE VOLUMEs in a CELL SEGMENT of a THRESHOLD (Reflectivity) must meet or
exceed the same THRESHOLD (Reflectivity) (60, 55, 50, 45, 40, 35, 30), in dBZe.
• THRESHOLDS (Segment Length): A set of minimum lengths of a CELL SEGMENT for each reflectivity
threshold (1.9), in km.

3.2.7.1.2 Strengths/Applications.
See Section 3.2.11.5.2.
3.2.7.1.3 Limitations.
See Section 3.2.11.5.3.
3.2.7.2 Storm Centroids.
3-17

October 2017

This algorithm identifies convective storm cells by grouping cell segments into components; computing the components’ attributes; vertically correlating the components into cells; and computing the cells’ attributes. A segment
is a radial sequence of significant reflectivities; component is a two dimensional area of significant reflectivity; and a
centroid is the mass weighted center of a three dimensional region of significant reflectivity. This algorithm identifies the individual high reflectivity cores or cells within convective storms.
First, to identify cells, the algorithm combines radially overlapping and azimuthally adjacent radial segments (from
the STORM CELL SEGMENTS Algorithm) into two-dimensional potential components. Since there are multiple
reflectivity thresholds used to find segments, only segments found on the same elevation scan with the same specified reflectivity threshold are combined. A potential component which has at least a specified number of segments
and aerial extent becomes a component.
Next, a search is done for overlapping components of different reflectivity thresholds on the same elevation scan.
If the center of a component found with a higher reflectivity threshold falls within the boundaries of another component, the component found with the higher reflectivity threshold is saved, and the other is discarded. In addition,
the components on each elevation scan are sorted by decreasing mass (Figure 3-8).
Then the components are vertically correlated; when components are correlated, they are assigned to the same cell.
The centers of mass of the components at adjacent elevation scans (starting at the lowest) are compared for proximity with respect to the x and y plane. For each component, the distance from the center of every component in
the next highest elevation scan is compared until a component is found within a specified search radius. Since the
components at each elevation scan are sorted by decreasing mass, the components with the largest masses will be
compared first. If no match is found for a component, then the search radius is increased, and the comparison is
done again. The comparison may be done up to three times with increasing search radii. If at least two components
(on adjacent elevation scans) are vertically correlated, a cell is created and its centroid and attributes are calculated
(Figure 3-9).
If two cells’ centroids are within spatial proximity, the cells are merged. To merge two cells, their centroids must be
within a specified horizontal distance, and their bases and tops must be within a specified vertical and angular separation. When merging two cells, one cell’s components are added to the other cell, and a new centroid is calculated.


> **Figure 3-8: Component Development within Storm Cell Centroids Processing.**

> 3-18

3-18

FMH-11-Part C


> **Figure 3-9: Storm Cell Centroid Locations.**



> **Figure 3-10: Cell-based vs. Grid-based VIL.**

> 3-19

3-19

October 2017

The components (which compose the cells) are saved along with the following attributes: the elevation angle, mass
weighted center (in Cartesian (x and y) and polar coordinates (azimuth and range)), height (AGL), mass, maximum
reflectivity, and reflectivity threshold. In addition, the following cell attributes are calculated: centroid (in Cartesian
(x and y) and polar coordinates (azimuth and range)), height, maximum reflectivity, height of the maximum reflectivity, top, base, Cell-based vertically integrated liquid (VIL), and number of components.
A calculation of VIL is made for each cell identified by Storm Cell Centroids by vertically integrating maximum
reflectivity values of a cell’s correlated components. This is a different calculation than the gridded VIL product
(VIL). A fast-moving or highly tilted storm will usually have a higher Cell-based VIL than Grid-based VIL (Figure
3-10).
Next, to reduce the crowding, cells which are still within spatial proximity are deleted. If two cells are still within a
specified horizontal distance and the difference in their cell depths is greater than a specified threshold, then the cell
with the lesser Cell-based VIL is deleted.
Finally, the remaining cells are sorted by Cell-based VIL and secondly maximum reflectivity. The cells and components and their attributes are used as inputs to the HAIL CORE ALOFT, STORM CELL TRACKING, and
STORM POSITION FORECAST Algorithms.

3.2.7.2.1 Operational parameters.
• Average DELTA AZIMUTH (Elevation): The average angular width of the radials in the ELEVATION scan,
in degrees.
• AZIMUTH (Segment): The azimuthal position of a CELL SEGMENT, in degrees.
• BEAM WIDTH: The angular distance between half-power points on either side of the center of the radar beam,
in degrees.
• Beginning RANGE (Segment): The slant range to the beginning (the front of the first sample volume) of a
CELL SEGMENT, in km.
• CELL SEGMENT: A contiguous run of SAMPLE VOLUMES along a radial with reflectivity values above one
of multiple reflectivity thresholds with the following attributes: AZIMUTH, beginning RANGE, ELEVATION
angle, ending RANGE, MASS WEIGHTED LENGTH, MASS WEIGHTED LENGTH SQUARED, maximum REFLECTIVITY FACTOR, and THRESHOLD (Reflectivity).
• ELEVATION: Angle of the elevation scan, in degrees.
• Ending RANGE (Segment): The slant range to the end (the back of the last sample volume) of a CELL SEGMENT, in km.
• MASS WEIGHTED LENGTH (Segment): The mass density weighted length of a CELL SEGMENT, in kg/
km.
• MASS WEIGHTED LENGTH SQUARED (Segment): The mass density weighted length squared of a CELL
SEGMENT, in kg/km.
• Maximum REFLECTIVITY FACTOR (Segment): The maximum (average)reflectivity factor of a CELL SEGMENT, in dBZe.
• NUMBER OF SEGMENTS: The number of CELL SEGMENTS identified on each ELEVATION and
THRESHOLD (Reflectivity).
• RANGE SAMPLE SPACING: The difference in slant range between two adjacent SAMPLE VOLUMEs along
a radial, i.e., the length of a SAMPLE VOLUME (1), in km.
• THRESHOLD (Azimuthal Separation): The maximum azimuthal separation required for assigning CELL SEGMENTS into the same COMPONENT (1.5), in degrees.
• THRESHOLDS (Component Area): A set of required minimum areas for a COMPONENT. There is an area
threshold for each reflectivity threshold used to find CELL SEGMENTs (10), in km2.
• THRESHOLD (Depth Delete): The maximum difference in the depths of two STORM CELLs required to
delete one of the STORM CELLs (4), in km.
3-20

FMH-11-Part C

•
•
•
•
•
•

•
•

THRESHOLD (Elevation Merge): The maximum difference in the elevation angles between the top of one STORM
CELL and the bottom of another STORM CELL required to merge the STORM CELLs (3.0), in degrees.
THRESHOLD (Height Merge): The maximum difference in the height between the top of one STORM CELL
and the bottom of another STORM CELL required to merge the STORM CELLs (4), in km.
THRESHOLD (Horizontal Delete): The maximum horizontal distance between two centroids required to delete one of the STORM CELLs (5), in km.
THRESHOLD (Horizontal Merge): The maximum horizontal distance between two centroids required to
merge the STORM CELLs (10), in km.
THRESHOLD (NUMBER OF SEGMENTS): The minimum number of CELL SEGMENTs required in a COMPONENT (2).
THRESHOLDS (Reflectivity): A set of minimum effective reflectivities used to find CELL SEGMENTs and
COMPONENTs and ordered from largest to smallest (60, 55, 50, 45, 40, 35, 30), in dBZe. The reflectivity factors of the sample volumes in a CELL SEGMENT must meet or exceed the same THRESHOLD (Reflectivity). And only CELL SEGMENTs which have been found using the same THRESHOLD (Reflectivity) can be
assigned to the same COMPONENT.
THRESHOLDS (Search Radii): A set of distances away from a COMPONENT’s mass weighted center which
a search is made for another COMPONENT’s mass weighted center on the next elevation scan with which to
correlate (5, 7.5, 10), in km.
THRESHOLD (Segment Overlap): The minimum slant range overlap required for assigning CELL SEGMENTs to the same component (1.95), in km.

3.2.7.2.2 Strengths/Applications.
See Section 3.2.11.5.2.
3.2.7.2.3 Limitations.
See Section 3.2.11.5.3.
3.2.7.3 Storm Cell Tracking.
The STORM CELL TRACKING Algorithm monitors the movement of storm cells by matching storms found in
the current volume scan to the storm cells from the previous volume scan in time and space, through the use of a
correlation table. The storm cells are matched as follows. Starting with the most intense cell (i.e., largest cell-based
VIL value) in the current volume scan, the centroid position is compared to the projected centroid positions of cells
from the previous volume scan. A cell’s projected centroid position is its forecasted position for the current volume scan. The cell from the previous volume scan which is correlated is the cell with a projected centroid located
within an adaptable range which is closest to the current cell. When a cell is correlated, it is considered the same cell
and assigned the same storm cell ID. Then the next most intense cell in the current volume scan is compared to all
uncorrelated cells in the previous volume scan, and so on until all cells in the current volume scan are processed.
Once a cell from the previous volume scan is correlated, it is not compared to any more cells in the current volume
scan. If no projected centroid positions are within the adaptable range of a cell’s centroid position, the cell remains
uncorrelated and is assigned a new storm cell ID. If more than a specified amount of time has passed between subsequent volume scans, then no matching is done, and all storm cells in the current volume scan are considered new.
The centroid positions used are in a Cartesian coordinate system with the radar at the origin, and where the X-axis
denotes east-west directions and the Y-axis denotes north-south directions. To complete the prediction process, the
STORM POSITION FORECAST Algorithm must be used (Figure 3-11).
The ID assigned to a Cell consists of a letter-number combination (A0, B0, C0...Z0, A1, B1...Z1, A2, B2...Z9). This
adds some value to the ID, such that storms with long lifetimes can be easily identified. The number has precedence
over the letter in this scheme. The list of IDs will reset to begin with A0 when the RPG is rebooted, or when a
threshold time interval has lapsed without cells.
The STI Attribute Table appears at the top of the STI product, and contains information on all identified cells. An
STI Alphanumeric Product is received and stored in a text file along with every STI Graphic Product.
3-21

October 2017


> **Figure 3-11 Storm Cell Tracking Process.**

> Centroid location is compared with forecast location of centroids from the previous volume scan.

Centroid location is compared with forecast location of centroids from the previous volume scan.

3.2.7.3.1 Operational Parameters.
• CORRELATION (Speed): Speed used to compute the CORRELATION (Distance), in km/hr.
• CORRELATION (Table): A data set used to keep track of the positions of correlated STORM CELLS.
• DIRECTION (Storm Cell): The direction from which a STORM CELL is moving, in degrees. Precise to 10-4
deg.
• ID: A unique label from a circular list assigned to a STORM CELL throughout its existence.
• SPEED (Storm Cell): Speed of a STORM CELL, in km/hr. Precise to 10-4 km/hr.
• STORM CELL: A three-dimensional region composed of COMPONENTs characterized by reflectivity values
above a given threshold, ordered by cell-based VIL.
• TIME (Maximum): The maximum allowed TIME BETWEEN VOLUME scans (20), in minutes. Storm Correlation between the current and previous volume scans is not performed if the scan separation exceeds this
value.
• TIME (Scan): The beginning time of a volume scan, in hours. Precise to 1/3600 hr.
• X-POSITION (Storm Cell): X-coordinate of the centroid (or center of mass weighted volume) of a STORM
CELL, in km. Precise to 10-4 km.
• Y-POSITION (Storm Cell): Y-coordinate of the centroid (or center of mass weighted volume) of a STORM
CELL, in km. Precise to 10-4 km.
3.2.7.3.2 Strengths/Application.
See Section 3.2.11.5.2.
3.2.7.3.3 Limitations.
See Section 3.2.11.5.3.
3.2.7.4 Storm Position Forecast.
3-22

FMH-11-Part C

The purpose of the STORM POSITION FORECAST Algorithm is to predict the future centroid locations of
storms (or storm cells) based on a history of their movement. The first volume scan a storm is detected, it is considered ‘new’, and the forecast movement used by the algorithm for processing purposes is either: a) the average
movement of all identified cells, or b) if no other cells are identified, the default speed and direction as set at the
Master System Control Function (MSCF). After the first volume scan a storm is detected, it is considered a ‘continuing storm’, and a forecast movement is computed based on a linear least squares extrapolation of the storm’s
previous positions. The linear least squares fits are for both X- position versus time and Y-position versus time. This
process is continued for each consecutive volume scan that a storm is tracked.
Forecast positions are computed in equal time steps (0, 15, 30, 45, or 60 minutes) for each continuing storm. The
number of forecast positions computed for a storm depends on the scaled forecast error and a permissible error.
The scaled forecast error is the accuracy of the forecast from the previous volume scan for the storm, or forecast
error, scaled by the ratio of a user specified error interval over the time between volume scans. The permissible
error is a user specified allowable error scaled by the error interval over the length (in time) of the forecast. Basically, the poorer a forecast was for a cell for the past volume scan, the fewer the number of forecast positions. Each
volume scan a vector-average storm motion is computed from all the continuing storms, and this average storm
motion is assigned to any new storms.
The STORM POSITION FORECAST Algorithm is the final step in the storm identification and movement prediction process. It utilizes information output by the STORM CELL TRACKING and the STORM CELL CENTROIDS Algorithms. Therefore, it cannot be applied until the completion of that analysis, which requires a complete volume scan of data. Resulting products are then generated (Figure 3-13; see Figure 3-12 for product symbol
definitions).

3.2.7.4.1 Operational Parameters.
• ALLOWABLE ERROR: The maximum acceptable error in the track of a STORM CELL allowed for the minimum forecast interval, in km (20).
• CORRELATION (Table): A data set used to keep track of the positions of correlated STORM CELLs.
• DEFAULT SPEED: A user-supplied speed at which storm cells are expected to move, in km/hr.
• DEFAULT DIRECTION: A user-supplied direction from which storm cells are expected to move, in degrees.
• ERROR INTERVAL: The amount of time upon which the ALLOWABLE ERROR was based, in hours
(0.25).
• FORECAST INTERVAL: A set of time intervals for which STORM CELL positions may be projected into the
future, in hours (0.25).
3.2.7.4.2 Strengths/Applications.
• The product works best with well-defined widely separated cells.
• A large number of past tracks, and/or four forecast positions signify a more reliable cell movement. Uneven
spacing between past tracks, fewer than four forecast positions, and/or re-identification of cells indicate less
reliable forecast positions.
• The STI product is useful as an overlay on volume products, but not limited to volume products.
• Cell motion is used in Storm Relative Velocity products (SRM, SRR).
• Cell attributes are critical inputs to the Hail Index product.

3-23

October 2017


> **Figure 3-12: Storm Track Information Product Symbols**


3-24

FMH-11-Part C


> **Figure 3-13: Storm Track Information Product.**

> This Storm Track Information product (OPUP display) example from the Oklahoma City, OK (KTLX)

This Storm Track Information product (OPUP display) example from the Oklahoma City, OK (KTLX)
WSR-88D on 31 May 2013 at 23:09 UTC includes past positions of storms (white circles), 15-minute
projected positions (white crosses), and the Combined Attribute Table of storm information.

3.2.7.4.3 Limitations.
• The algorithm does attempt to prevent non-meteorological targets (e.g., anomalous propagation or clutter) in
the reflectivity data from being considered segments. Clutter filtering is being applied in the WSR-88D, but it is
not always adequate or correctly applied. When non-meteorological targets are identified as segments, this may
lead to falsely identified storm cells or parts of storm cells in the STORM CELL CENTROIDS Algorithm.
• At long ranges, only the lowest elevation scans of a volume scan will contain components. For example, at 120
nm, the bottom of radar beam at 0.5° is nearly 18 kft ARL. Components must be found on at least two con3-25

October 2017

•

•
•
•
•
•
•
•
•
•

secutive elevation scans to be considered a storm cell. Storm cells at long ranges may not have enough vertical
extent to be detected at even two elevation scans, and, therefore, will not be identified.
Rarely, problems may arise in the vertical correlation process which will lead to improper identification of cells
and/or computation of their attributes. When several cells are clustered closely together, the algorithm may
combine separate components on an elevation scan into one component. Also, the algorithm may either falsely
split a cell into two or more cells or combine a group of cells into one cell.
The cell merging and deletion processes attempts to decrease the cluttered nature of cells. But deletion and
merging of cells may decrease the performance of downstream algorithms using cell and component data.
Alternatively (as studied and developed by the NSSL), this algorithm uses the Severe Hail Index (SHI) from
the HAIL CORE ALOFT Algorithm (instead of Cell-based VIL) to sort cells and in the cell deletion process
(which reduces crowding).
This algorithm averages actual changes in cell movement and erratic movement due to centroid shifting which
occurs in some storm cells.
The forecast track is always a straight line.
Because several volume scans are used for the forecast, a sudden shift in a centroid location will be damped out
until the new track becomes established.
The accuracy of the forecasted movement provided by this algorithm is limited by the accuracy of the tracking
algorithm. For example, if the STORM CELL TRACKING Algorithm inaccurately matches storm cells between volume scans, then the forecasted movement of those cells will also be inaccurate.
Errors may occur in the identification of cells and the calculation of cell attributes when cells are in close proximity.
Large errors may occur in the attributes of cells close to the RDA, especially in VCP 21.
Unrepresentative movements are possible due to propagational effects. Due to development or dissipation, the
high reflectivity cores change location within an identified cell from one volume scan to the next, resulting in
false representation of the movement of the cell.


#### 3.2.8 Mesocyclone Detection Algorithm.

This section describes the primary modules 1D, 2D, and 3D processing steps of the Mesocyclone Detection Algorithm (MDA). For completeness, this section also includes a brief overview of other MDA modules; tracking, and
trending. The major steps in MDA are as follows: 1) threshold velocity data by reflectivity value; 2) identify MDA
1D Features; 3) identify MDA 2D Features; 4) identify MDA 3D Features; 5) classify MDA 3D Features; 6) track
MDA 3D Features; 7) trend MDA 3D Features.
3.2.8.1 MDA Overview.
The MDA uses pattern recognition techniques to detect mesocyclones. These techniques define a process used for
searching through Doppler velocity data for symmetric regions of large azimuthal shear. The MDA is based on the
extraction of significant attributes which characterize mesocyclones.
The MDA locates mesocyclones where a mesocyclone is defined as a three-dimensional region in a storm which
rotates (usually cyclonically), and is closely correlated with severe weather. The MDA uses the systematic procedure
described below.
The MDA uses radial velocity and reflectivity data to detect storm-scale (1 - 10 km (0.54 – 5.4 nm)) cyclonic vortex
signatures and diagnose the attributes of the detected signatures to determine if they are associated with tornadoes
and/or damaging wind. The algorithm starts by identifying one-dimensional (1D) shear segments (pattern vectors)
from mean radial velocity data. To help limit the search for circulations to those associated with storm cells, the
algorithm only searches velocity data from sample volumes that have reflectivities above THRESHOLD (minimum
Reflectivity) and are below THRESHOLD (maximum Shear Segment Height). Shear segments (pattern vectors) are
an azimuthal run of velocities whose gate-to-gate shear is continuously cyclonic. Gate-to-gate means the sample
volumes are from adjacent radials and at the same range. A look-ahead function, that is range dependent, mitigates
3-26

FMH-11-Part C

problems with small perturbations in shear during shear segment construction. All shear segments must also pass
strength and length criteria.
Shear segments on each elevation scan in azimuthal and radial proximity are combined into potential two-dimensional (2D) features. Using multiple strength rank thresholds, 2D vortex cores of different strength rank are isolated
from broader regions of 2D azimuthal shear. Strength rank is a function of rotational velocity, shear, and range. If a
potential 2D feature still has enough shear segments and meets aspect ratio criteria, it is checked for overlap with all
previously saved 2D features on the elevation scan. If weaker features overlap stronger features, the weaker features
are discarded.
2D features from adjacent elevation scans are vertically correlated into potential three-dimensional (3D) features.
The mesocyclone 3D features are associated with storm cells and their attributes are computed and saved.
After all 3D mesocyclone features have been identified, features are time associated. A first guess location is made,
using a motion vector from the previous volume scan. 3D features within a certain radius of the first guess point
become association candidates. Additional 3D features are also added as potential candidates for association as radii
are increased around the first guess point. The best candidate for time association is found by sorting the candidates
within each distance threshold first by strength rank and then by circulation type. The 4D detections are classified
by vortex type (e.g., Mesocyclones, Low-core mesocyclones) and the classifications are saved for display purposes.
Attributes of 4D detections are used to calculate time trends. Trend and time- height information of tracked 4D
detection attributes are saved for display purposes.
At the display device, 4D detection attributes, their classifications, and characteristics are presented in an attribute
table. Graphical overlays communicate vortex type (e.g., Mesocyclones, Low-core mesocyclones), location, and
strength to forecasters. Feature strength can be used by forecasters to remove weaker detections from overlay displays that become too cluttered.
3.2.8.2 One-Dimensional (1D) Features
Shear segments, 1D features, are identified on each elevation scan from velocity data in azimuthally adjacent radials. It is assumed the radar rotates clockwise, and the radials are approximately 1 degree in azimuthal width with no
gaps between radials. The counter-clockwise velocity difference is computed for each pair of sample volumes (from
adjacent radials) that are constant in range, closer than THRESHOLD (maximum Shear Segment Distance), below
THRESHOLD (maximum Shear Segment Height) ARL, and coincide with reflectivity values above THRESHOLD
(minimum Reflectivity). If a pair of sample volumes has a positive velocity difference (cyclonic shear) or the first
(most counter clockwise) velocity value is valid and the second is missing, then a shear segment is started. If the
first or both velocity values are range folded or missing, a shear segment is not started. If subsequent velocity data
at the same range exhibits anticyclonic shear with respect to the first velocity value in the shear segment, the shear
segment is ended at the last velocity value exhibiting cyclonic shear, including look-ahead radials. When subsequent
negative or neutral azimuthal shear is computed or missing or range folded data are found, the algorithm looks
ahead a number of radials which varies with range. If the next non-missing, non-range folded velocity value exhibits
cyclonic shear with respect to the last non-missing, non-range folded velocity value, look-ahead mode is canceled
and cyclonic shear is again searched for and the shear vector becomes larger. If the number of look-ahead radials is
exceeded, the vector is ended with the last velocity value exhibiting cyclonic shear.
All 360 radials in a sweep are processed as shear vectors are identified. At the end of the 360- degree sweep, processing continues until all 1D features that are open are closed. This overlap processing allows 1D features to
overlap the beginning / ending radial so that all shear regions around the beginning radial can be identified. If the
number of 1D features in the volume scan meets or exceeds THRESHOLD (maximum # 1D Features), then processing immediately skips over the remainder of the 1D identification step and proceeds to the 1D attribute identification.
For each shear segment the following information is computed: beginning Az (azimuth), ending Az, beginning
velocity, ending velocity, shear segment delta V (Ending velocity - beginning velocity), length of the shear segment
3-27

October 2017

(distance between beg Az and end Az), shear (delta V / length of the shear segment), max gate-to-gate delta V (GTGDV) of any two adjacent radials, azimuth of the max GTGDV, range, and strength rank.
One-Dimension feature strength ranks are selected from a look-up table of range dependent values of velocity
difference and shear. Each entry is scanned from lowest to highest strength rank. The shear segment strength rank
is the largest rank in which 1) the shear segment GTGDV is above THRESHOLD (minimum Velocity Difference),
a range dependent threshold or 2) the shear segment velocity difference is greater than THRESHOLD (minimum
Velocity Difference) and shear segment shear is greater than THRESHOLD (minimum Shear), a range- dependent
threshold. If the computed strength rank is less than 1, then the shear segment is discarded.
If the length of a shear segment exceeds THRESHOLD (maximum Shear Segment Length) then the shear segment
is considered to be too long. The algorithm searches within the shear segment, beginning with the first velocity,
to see if an embedded vector is present that passes the strength thresholds for the next larger strength rank than
the original “long” shear segment. If a next larger strength rank vector is found, the Cartesian length of the core
shear segment that was found is computed. If the length of the core shear segment is more than THRESHOLD
(maximum Core Shear Segment Length) then the strength rank is increased by one and a new core shear segment
is sought. This process is repeated until the Cartesian length of the core shear segment is less than or equal to
THRESHOLD (maximum Core Shear Segment Length). At this point the shear-segment attributes of the core
shear segment are re-computed, and the shear segment is saved. If a core shear segment cannot be found whose
length is less than or equal to THRESHOLD (maximum Core Shear Segment Length), the entire original “long”
shear segment is discarded.
The final output of the 1D analysis is a list of shear segments and 11 of their attributes.
3.2.8.3 Two-Dimensional (2D) Features.
Once all shear segments have been identified on an elevation scan, they are combined into potential two-dimensional (2D) features. Processing begins with the first available shear segment that has a maximum strength rank. If any
other shear segment with the same strength rank as the first segment overlaps in azimuth and is separated in range
by no more than THRESHOLD (maximum 2D Construction Radial Distance) from the original segment, then the
shear segment is added to the 2D feature. Once all shear segments that have maximum strength rank are checked
for proximity, another 2D feature is started by selecting an unused shear segment that has the maximum strength
rank. A search is conducted for other shear segments that meet azimuth and distance criteria. This process continues until all possible 2D features of maximum strength rank are found. If the total number of shear segments in a
2D feature is less than THRESHOLD (minimum # Shear Segments) or if any feature extends less than THRESHOLD (minimum 2D Radial Diameter) in the radial direction, the 2D feature is discarded.
Each potential 2D feature at the initial, maximum, strength rank is examined for overlapping shear segments. If
any shear segments within a feature have the same range, then their combined length is calculated. If the combined
shear segment has a length greater than THRESHOLD (maximum Shear Segment Length), then the weaker (based
on the shear-segment rotational velocity) is discarded. If both shear segments used to produce the combined shear
segment are the same strength and the maximum shear-segment length has been exceeded, then the entire shear
segment is discarded. Otherwise, the two shear segments are combined into one and the shear-segment attributes
are re-computed.
The algorithm then computes the potential 2D feature aspect ratio. If the 2D feature aspect ratio (radial distance
/ azimuthal distance) is greater than or equal to THRESHOLD (maximum Aspect Ratio) or less than or equal to
THRESHOLD (minimum Aspect Ratio), the 2D feature is discarded. 2D features having a total number of segments less THRESHOLD (minimum # Shear Segments) or a diameter greater than or equal to THRESHOLD
(maximum 2D Diameter) are discarded. Two- dimensional features that have an azimuth of the maximum inbound
velocity which is greater than the azimuth of the maximum outbound velocity are also discarded.
The next step is to build a list of potential 2D features by grouping shear segments that have the next smaller rank
with those that have the maximum strength rank. Processing is the same, as described above. All 2D features are
3-28

FMH-11-Part C

checked for overlapping shear segments, appropriate aspect ratios, total number of shear segments, threshold radius,
and azimuth of maximum inbound and outbound velocities. Two-dimensional features are built for each of the
strength rank categories from maximum to minimum rank. Each iteration includes shear segments from the current
and all higher ranks. Shear segments are reused in different features. All unused shear segments are discarded.
If and when the number of 2D features in the volume scan meets or exceeds THRESHOLD (maximum # 2D
Features), then processing immediately skips over the remainder of the 2D identification step and proceeds to the
2D classification step.
Attributes for each potential 2D feature are computed, including 2D feature strength rank. If certain thresholds and
criteria are not met, the potential 2D feature is discarded.
The Az/Ran of max v and Az/Ran of min v for each 2D feature are computed. A running average of max v
and min v across 3 shear segments within each 2D feature are also computed. The result is a smooth max v and
smooth min v for the 2D feature. The following attributes are computed for all 2D features: rotational velocity =
abs (smooth max v - smooth min v)/ 2, shear = rotational velocity divided by the distance between max v and min
v, centroid location = Az/Ran of the midpoint of the line between max v and min v, centroid height, diameter =
the distance between max v and min v, maximum GTGDV for all shear segments in the 2D feature, max and min
azimuth and range bounded by the max v and min v points, and elevation angle. The 2D feature attributes and
strength rank are saved.
The next step is to extract azimuthal shear cores on an elevation slice. This analysis begins by comparing locations
of 2D features of highest strength rank with those having the next lower strength rank. If a feature with lower rank
does not contain, within its azimuth and range boundaries, a 2D feature of next higher rank, then the 2D feature
and its attributes including its strength rank is retained. If any shear segments are shared between a 2D feature of
smaller strength rank and a feature of higher strength rank or if a centroid of a feature of larger strength rank is
within the maximum range and azimuthal extent of another feature of lesser strength rank, then the lower strength
rank feature is retained. If a lower strength rank feature overlaps more than one feature of higher strength rank,
then the lower strength rank feature is discarded and the higher strength rank features are retained as individual 2D
features. Feature core extraction is repeated comparing features with highest strength rank values with features of all
smaller strength rank values. Comparisons continue in a double loop structure where all features of higher strength
rank value are compared to all features with smaller strength rank values. The last iteration compares all rank 2 features with all rank 1 features.
By extracting the feature cores and reconstructing 2D features from features of different strength rank, as described
above, some 2D features may contain new values for max v, min v, max GTGDV, and shear. This processing leads
to a change in the 2D feature strength rank; therefore, strength rank needs to be re-computed for each 2D feature.
At the end of 2D processing, 23 attributes are saved for each 2D feature.
3.2.8.4 Vertically-Associated 3-Dimensional (3D) Features.
Once all of the 2D features have been constructed and saved for the second two tilts, the algorithm vertically
correlates the 2D features from different elevation scans into vertically-associated 3D features. Processing begins
with any 2D feature on the first tilt. A list is assembled of all 2D features on the second tilt whose center is within
THRESHOLD (minimum 2D Association Distance) of the 2D feature on the first tilt. If there are more than one,
the list of second tilt candidates is sorted first by strength rank and then by distance. Next, all of the 2D features on
the second tilt are found that are within a specified annulus size of the center of the 2D feature on the first tilt. If
more than one candidate exists, second tilt candidates are sorted first by strength rank and then by distance. Then,
the second tilt candidates from the specified annulus are added to the bottom of the list of candidate that were in
the second search area. The algorithm repeats this process until the search radius annulus equals THRESHOLD
(maximum 2D Association Distance). All the previous 3D processing steps are repeated for all 2D features on the
first tilt. In the end, each 2D feature on the first tilt has a list of sorted candidates associated with it.
Next, the algorithm guarantees that each 2D feature on the higher elevation is used only once or not at all. A check
3-29

October 2017

is made to see if any 2D features on the higher elevation appear first in the candidate list of more than one lower
elevation 2D feature. If so, the first higher elevation 2D feature on the list of a lower elevation 2D feature whose
center is closer is retained as the first element in its list. The higher elevation 2D feature at the top of the list of the
lower elevation 2Dfeature whose center is farther away is removed from the candidate list and the second candidate
2D feature is promoted to the top in the list. This process continues until each 2D feature on the higher elevation is
uniquely associated (first on the list of candidates) with one 2D feature on the lower elevation. If any feature from
the higher elevation scan cannot be vertically associated with another lower elevation 2D feature because it is first
on the candidate list, or if the higher elevation feature becomes unassociated from all lower 2D features, the higher
elevation 2D feature remains un-associated, possibly becoming a candidate for association with a 2D feature on the
next higher elevation. In the end, each 2D feature on the lower elevation is associated with the first candidate from
the list of 2D features on the higher elevation. These associations result in 3D features. Each 2D feature on the first
tilt is used only once or not at all.
The algorithm then creates candidate lists for all the 2D features on the third tilt from those on the second tilt and
makes sure that 2D features on the second tilt are used only once or not at all. This process progresses up through
the volume scan stopping when the top elevation scan is reached and all 3D features have been identified.
If and when the number of 3D features in the volume scan meets or exceeds THRESHOLD (maximum # 3D
Features), then processing immediately skips over the remainder of the 3D identification process and proceeds with
the 3D classification process.
All unused 2D features are discarded. Several calculated attributes are now described.
A mesocyclone strength index (MSI) is calculated for each 3D feature. Strength ranks of all 2D feature components
are multiplied by 1000, weighted by the average air density in a standard atmosphere, and integrated vertically across
the half-power beam width depth at the height of the 2D feature. Integration is done from the feature’s base (plus
the half-beam width depth) to its top (plus half-beam width depth) or THRESHOLD (maximum 3D Couplet Core
Top), whichever is lower in altitude. The integrated value is divided by the total depth (with half-power beam width
added).
An MSI rank is calculated for each 3D feature. The algorithm vertically integrates rotational velocity, shear, and GTGDV in the same manner as strength ranks were integrated in the calculation of MSI. An MSI rank is assigned to
each 3D feature by looking up rank values associated with integrated values of rotational velocity, shear, and GTGDV in a table.
A 3D strength rank, core base, core top, and core depth are calculated for all 3D features. The 3D strength rank is
defined as the strongest continuous core of 2D features of a given strength rank that is at least 3 km (1.6 nm) of
half-beam width in depth. The base of the core must be below 5 km and the top of the core must be at or below 8
km (4.3 nm). The 3D strength rank, core base, core top, and core depth are associated with each 3D feature.
Fifty-seven attributes, some derived from 3D features themselves and some calculated, are saved for later use. Ten
attributes and their height values from 2D features that are components of 3D features are saved for time-height
cross-sections.
3.2.8.5 Rapid Update.
Rapid update allows MDA to output 3D feature information at the end of every tilt, beginning with the first tilt,
rather than at the end of the volume scan. As 3D features are built, 3D feature information is updated so that forecasters can see if 3D features are increasing in intensity. Rapid update increases algorithm output lead-time, allowing
forecasters more time to view algorithm output and issue severe weather warnings.
After 2D features have been identified on the first tilt of a new volume scan, their locations are compared to forecast locations of 3D features from the previous volume scan. This comparison is described in the time association
step of the MDA tracking module. The time association information defines 3D feature tracks. After time association, before MDA processes data from the second tilt, the direction and speed of each detection is computed
3-30

FMH-11-Part C

and the position of severe weather feature icons and table attribute information on the display are updated for
the current volume scan. Low-level attributes of the 2D feature on the current volume scan; low-level diameter,
low-level rotational velocity, low-level shear, and low-level gate-to-gate velocity difference; are assigned and the rest
of the attributes (e.g., circulation type, mesocyclone strength index, maximum rotational velocity, maximum shear)
are inherited from time associated 3D features detected on the previous volume scan.
The 3D features that are not time associated with 2D features found on the new volume scan are identified as “extrapolated” features. If 3D features from the previous volume scan are associated with features on the current volume scan, their classification is changed from extrapolated to matched. Extrapolated features that are not matched
by the time the radar beam reaches three km over the previously detected feature base height are removed from the
product.
When the RDA provides radar data from higher tilts, 1D and 2D features are identified by MDA. New 2D features are vertically associated with adjacent-tilt 2D features, 3D features, or retained as potential bases of new 3D
features. Feature attributes are updated when: 1) a new 3D feature is identified during the current volume scan; 2)
feature attributes indicate a mesocyclone is increasing in severity; or 3) all information has been obtained for a mesocyclone (i.e., the feature is topped).
At the end of the volume scan MDA computes an average motion and speed of all 3D features and computes forecast positions for all 3D features and the resulting product is generated (Figures 3-14 and 3-15).
3.2.8.6 External Interfaces.
MDA interfaces with the SCIT [Centroids] Algorithm to obtain an average storm depth of the 10 strongest storm
cells. Storm cell strength is based on SHI and maximum reflectivity. The average storm depth is used to determine
if a 3D velocity feature meets criteria to classify the MDA detection as a low-core circulation.
MDA also uses SCIT information to associate each 3D velocity feature with a storm cell. This association information can be used in the storm attribute table to let the user know that a storm cell has a mesocyclone associated with
it.
3.2.8.7 Adaptable Parameters.
The MDA has many adaptable parameters to allow maximum flexibility in fine-tuning algorithm performance. The
vast majority of the parameters are intended for ROC use only, not for users to change during operations.
Users are allowed to activate or deactivate a display filter and to specify a minimum strength rank value, below which
3D features are marked for non-display. Users are also allowed to activate or deactivate a switch that allows forecasters to define their own mesocyclone criteria. By default, MDA requires mesocyclones to have predefined values for
minimum strength rank, maximum base height, and minimum depth.

3-31

October 2017


> **Figure 3-14: Mesocyclone Product Comparison.**

> Mesocyclone (M) (upper left) versus Mesocyclone Detection (MD) (lower right) product difference in

Mesocyclone (M) (upper left) versus Mesocyclone Detection (MD) (lower right) product difference in
circulation depiction. MD detections with spikes indicate that the circulation was detected on the lowest radar elevation tilt. The MD detections without spikes, regular circles, indicate that the circulation
was detected aloft.

3-32

FMH-11-Part C


> **Figure 3-15: Mesocyclone Detection Product.**

> This Mesocyclone Detection product example is overlaid on a Reflectivity Data Array product. The

This Mesocyclone Detection product example is overlaid on a Reflectivity Data Array product. The
MD includes identification numbers for identified storms. The yellow symbols are related to the circulation strength rank for the cell (less than 5 is a thin, yellow, open circle; greater than or equal to 5 is a
thick open circle), the diameter of the circle is based on the diameter of the circulation. Past position
symbols are indicated by solid diamonds. The Combined Attribute Table provides alphanumeric storm
information.
3.2.8.8 Operational Parameters.
• AZIMUTH: Azimuthal position, in radians.
• ELEVATION: Elevation angle, in radians.

3-33

October 2017

•
•
•
•
•
•
•

•
•
•
•
•
•
•
•
•

•
•
•

RADAR (Beam Width): Width of the radar beam. Values may range from 0.0 to 10.0 degrees and the precision
is at least 0.1 degree.
RADAR (Height): Height of the RDA in km.
RADIAL: The set of sample volumes, only one at each RANGE (Slant), along a constant AZIMUTH and ELEVATION.
RADIUS (Earth): The radius of the Earth (6371), in km.
RANGE (Slant): The slant range to the center of a SAMPLE VOLUME, in km.
REFLECTIVITY FACTOR: The effective radar reflectivity factor (SAMPLE VOLUME) assigned to a (velocity) SAMPLE VOLUME, in dBZe.
SAMPLE VOLUME: A data sample volume along a radial whose (half power) dimensions are described by the
azimuthal and vertical beam widths and the RANGE (Slant) sampling interval. These dimensions are approximately 1 degree in azimuthal and vertical width (perpendicular to the beam) and 0.25 km (0.13 nm) in RANGE
(Slant) (or length) for Velocity sample volumes and 1.0 km (0.54 nm) in RANGE (Slant) for Reflectivity (Factor)
sample volumes.
VELOCITY (Sample Volume): The mean radial velocity of a SAMPLE VOLUME, in m/s.
STANDARD ATMOSPHERE AIR DENSITY LOOK-UP TABLE VALUES: The Standard Atmosphere Air
Density is used to calculate the 3D Feature (MSI). The Standard Atmosphere Air Density is interpolated from a
table of look-up values.
THRESHOLD (minimum Reflectivity): Minimum reflectivity (dBZ) needed to process radial velocity data; default 0 dBZ, range = -25 dBZe (process all data) to 35 dBZe (process precipitation data only), precision: 1 dBZe.
This adaptable parameter should have URC level of change authority.
THRESHOLD (maximum Shear Segment Height): The height above which shear segments are not identified;
default 15 km (8.1 nm), range 1 to 15 km, precision: 1 km (0.54 nm). This configuration parameter can be used
to change MDA’s functionality.
THRESHOLD (maximum Shear Segment Distance): The Range (Slant) beyond which shear segments are not
identified; default 230 km (124 nm), range 0 to 230 km (124 nm), precision: 1 km (0.54 nm). This configuration
parameter can be used to change MDA’s functionality.
THRESHOLD (maximum # 1D Features): Maximum number of 1D Features the algorithm can process per elevation scan; default 5000, range 2000 to 5000, precision: 1. This configuration parameter can be used to change
MDA’s functionality.
THRESHOLD (maximum # 2D Features): Maximum number of 2D Features the algorithm can process per
elevation scan; default 500, range 400 to 1000, precision: 1. This configuration parameter can be used to change
MDA’s functionality.
THRESHOLD (maximum # 3D Features): Maximum total number of 3D Features the algorithm can process
per volume scan; default 500, range 100 to 500, precision: 1. This configuration parameter can be used to change
MDA’s functionality.
THRESHOLD (minimum # Shear Segments): Minimum number of 1D shear segments needed to declare a
group of segments a 2D shear feature. For example, if you want a minimum diameter of 1 km (0.54 nm) for the
2D features, then divide diameter (1 km (0.54 nm)) by the gate spacing (250 m (0.13 nm)) to get the minimum
number of shear segments per 2D feature equal to 4; default 4, range 3 to 8, precision: 1. This configuration
parameter can be used to change MDA’s functionality.
THRESHOLD (minimum Strength Rank): Minimum strength rank to locate shear segments, rank 2D features, and rank 3D features; default 1, range 1 to 5, precision: 1. Value must be less than or equal to maximum
Strength Rank. This configuration parameter can be used to change MDA’s functionality.
THRESHOLD (maximum Strength Rank): Maximum strength rank to locate shear segments, rank 2D features,
and rank 3D features; default 25, range 5 to 25, precision: 1. Value must be greater than or equal to minimum
Strength Rank. This configuration parameter can be used to change MDA’s functionality.
THRESHOLD (minimum Velocity Difference): Minimum velocity difference (m/s) needed to save 1D azi3-34

FMH-11-Part C

•

•

•

•

•

•
•

•

muthal shear segments. This value is also used as the minimum gate-to-gate velocity difference (m/s) needed
to save 1D azimuthal shear segments. The parameter is also part of an array, from 1 to 25, corresponding to
the thresholds for strength ranks of 1 to 25; default is range dependent, range 7.5 to 130.0 m/s, precision: 0.5
m/s. This parameter is RANGE (Slant)-dependent. If the RANGE (Slant) to the shear segment is between
THRESHOLD (beginning Range Linear Reduction) and THRESHOLD (ending Range Linear Reduction), the
velocity difference values are modified so that their values at THRESHOLD (beginning Range Linear Reduction) and at THRESHOLD (ending Range Linear Reduction) are reduced by 75%, and at intermediate RANGES (Slant), the velocity difference values are interpolated linearly between the THRESHOLD (beginning Range
Linear Reduction) and THRESHOLD (ending Range Linear Reduction) values. Beyond THRESHOLD (ending
Range Linear Reduction), the minimum velocity difference is kept at 75% of the original values. This configuration parameter can be used to change MDA’s functionality.
THRESHOLD (minimum Shear): Minimum shear (ms-1 km-1) needed to save 1D azimuthal shear segments.
The parameter is also part of an array, from 1 to 25, corresponding to the thresholds for strength ranks of 1 to
25; default, range 1.75 to 21.00 ms-1 km-1, precision: 0.25 ms-1 km-1. This parameter is RANGE (Slant)-dependent. Values given in a table are applied to RANGES (Slant) from 0 to THRESHOLD (beginning Range
Linear Reduction). If the RANGE (Slant) to the shear segment is between THRESHOLD (beginning Range
Linear Reduction) and THRESHOLD (ending Range Linear Reduction), the shear are modified so that their
values at THRESHOLD (beginning Range Linear Reduction) and at THRESHOLD (ending Range Linear
Reduction) are reduced by 50%, and at intermediate RANGES (Slant), the shear values are interpolated linearly
between the THRESHOLD (beginning Range Linear Reduction) and THRESHOLD (ending Range Linear
Reduction) values. Beyond THRESHOLD (ending Range Linear Reduction), the minimum shear is kept at 50%
of the original values. This configuration parameter can be used to change MDA’s functionality.
THRESHOLD (beginning Range Linear Reduction): The beginning RANGE (Slant) at which the velocity
difference and shear thresholds start to drop off linearly; default 100 km (54 nm), range 0 to 230 km (124 nm),
precision: 1 km (0.54 nm). Value must be less than THRESHOLD (ending Range Linear Reduction). This configuration parameter can be used to change MDA’s functionality.
THRESHOLD (ending Range Linear Reduction): The ending RANGE (Slant) at which the velocity difference
and shear thresholds start to drop off linearly; default 200 km (108 nm), range 0 to 230 km (124 nm), precision:
1 km (0.54 nm). Value must be greater than THRESHOLD (beginning Range Linear Reduction.) This configuration parameter can be used to change MDA’s functionality.
THRESHOLD (maximum Shear Segment Length): Maximum length (km) allowed for a shear segment. This
assumes that a stronger “core” shear segment does not exist within the shear segment under consideration;
default 10 km (5.4 nm), range 5 to 15 km (2.7 to 8.1 nm), precision: 1 km (0.54 nm). The THRESHOLD (maximum Shear Segment Length) must be greater than or equal to THRESHOLD (maximum Core Shear Segment
Length). This configuration parameter can be used to change MDA’s functionality.
THRESHOLD (maximum Core Shear Segment Length): Maximum length (km) allowed for a shear segment
whose “core” region is used in lieu of the full length of the actual shear segment because the original shear
segment is more than THRESHOLD (maximum Shear Segment Length). Shear segments whose core region is
larger than THRESHOLD (maximum Core Shear Segment Length) are discarded; default 10 km (5.4 nm), range
5 to 15 km (2.7 to 8.1 nm), precision: 1 km (0.54 nm). THRESHOLD (maximum Core Shear Segment Length)
must be less than or equal to THRESHOLD (maximum Shear Segment Length.) This configuration parameter
can be used to change MDA’s functionality.
THRESHOLD (maximum # Shear Segments): The maximum number of shear segments allowed per 2D feature,
default 200 shear segments, range 50 to 200, precision: 1. This configuration parameter can be used to change
MDA’s functionality.
THRESHOLD (maximum Aspect Ratio): The maximum aspect ratio (RANGE (Slant) divided by AZIMUTH)
allowed for a 2D feature; default 2.0, range 1.0 to 4.0, precision: 0.5. The THRESHOLD (maximum aspect
ratio) must be greater than THRESHOLD (minimum Aspect Ratio). This configuration parameter can be used
to change MDA’s functionality.
THRESHOLD (minimum Aspect Ratio): The minimum aspect ratio (RANGE (Slant) to AZIMUTH) allowed
3-35

October 2017

•
•

•
•

•
•

•
•
•
•

•

•

•
•

for a 2D feature; default 0.0, range 0.0 to 2.0, precision: 0.5. The THRESHOLD (minimum aspect ratio) must
be less than THRESHOLD (maximum Aspect Ratio). This configuration parameter can be used to change
MDA’s functionality.
THRESHOLD (maximum 2D Feature Height): Maximum allowable height (km) to build 2D features. No 2D
features will be saved above this height; default 12 km (6.5 nm); range 6 to 12 km (3.2 to 6.5 nm), precision: 1
km (0.54 nm). This configuration parameter can be used to change MDA’s functionality.
THRESHOLD (minimum 2D Radial Diameter): Minimum radial diameter (km) of a 2D feature (or the
RANGE (Slant). Difference between shear segments having the minimum and maximum center RANGES
(Slant)); default 1.00 km (0.54 nm), range 0.75 to 2.50 km (0.4 to 1.3 nm), precision: 0.05 km. This configuration
parameter can be used to change MDA’s functionality.
THRESHOLD (maximum 2D Diameter): Maximum diameter (km) of a 2D Feature. Distance between min
and max velocities in a 2D Feature; default 15 km (8.1 nm), range 10 to 20 km (5.4 to 10.8 nm), precision: 1 km
(0.54 nm). This configuration parameter can be used to change MDA’s functionality.
THRESHOLD (maximum 2D Distance): Maximum allowable radial distance (km). Construction Radial between shear segments for segments to become associated into 2D features; default 1.00 km (0.54 nm), range
0.25 (sample volume size) to 2.00 km (0.4 to 1.3 nm), precision: 0.25 km (0.13 nm). This configuration parameter can be used to change MDA’s functionality.
THRESHOLD (2D Feature minimum Range): Minimum range from the radar allowed for a 2D Feature; default
5 km (2.7 nm), range 0 to 15 km (0 to 8.1 nm), precision: 1 km (0.54 nm). This configuration parameter can be
used to change MDA’s functionality.
THRESHOLD (minimum 2D Association Distance): The minimum search radius distance used to associate 2D
features into 3D features; default 2 km (1.1 nm), range 2 to 10 km (1.1 to 5.4 nm), precision: 1 km (0.54 nm).
(This parameter must be less than THRESHOLD (maximum 2D Association Distance.) This configuration
parameter can be used to change MDA’s functionality.
THRESHOLD (maximum 2D Association Distance): The maximum search radius
distance used to associate 2D features into 3D features; default 8 km; range 5 to 10 km, precision: 1 km. (This
parameter must be greater than THRESHOLD (minimum 2D Association Distance.) This configuration parameter can be used to change MDA’s functionality.
THRESHOLD (maximum 3D Couplet Core Base): Maximum allowable base altitude (km) of a 3D couplet of a
particular 3D Strength Rank “core”. All 3D couplets whose base is above this level are discarded; default 5 km;
range 2 to 5 km, precision: 1 km. This configuration parameter can be used to change MDA’s functionality.
THRESHOLD (minimum 3D Couplet Core Depth): Minimum allowable half- beamwidth depth (km) of a 3D
circulation couplet of a particular 3D Strength Rank core. All 3D couplets less than this depth are discarded;
default 3 km, range 2 to 5 km, precision: 1 km. This configuration parameter can be used to change MDA’s
functionality.
THRESHOLD (maximum 3D Couplet Core Top): Maximum allowable top (km) of a 3D couplet core”. All 3D
couplets whose “core top” is above this level are adjusted so that the core top is set to THRESHOLD (maximum 3D Couplet Core Top). Also, MSI integration does not exceed this level; default 8 km, range 8 to 12 km,
precision: 1 km. This configuration parameter can be used to change MDA’s functionality.
THRESHOLD (maximum Parm Height): Maximum height (km) at which the values for maximum rotational
velocity, shear, and gate-to-gate velocity difference are taken from 2D features for each 3D feature; default 12
km, range 6 to 12 km, precision: 1 km. This configuration parameter can be used to change MDA’s functionality.
THRESHOLD (minimum Low-Core Mesocyclone Depth Fraction): The minimum storm-relative depth fraction allowed to classify 3D features as low-core mesocyclones; default 0.25 (which is 25%, range 0.10 to 0.50,
precision: 0.05. This configuration parameter can be used to change MDA’s functionality.
THRESHOLD (maximum Low-Core Mesocyclone Depth): The maximum absolute depth (km) allowed to classify 3D features as low-core mesocyclones; default 3 km, range 0 to 5 km, precision: 1 km. This value must be
3-36

FMH-11-Part C

•

•
•
•
•
•
•

•

•

•

•

•
•

less than or equal to the depth that results from the multiplication of the mesocyclone depth by the THRESHOLD (minimum Low-Core mesocyclone Depth Fraction). This configuration parameter can be used to change
MDA’s functionality.
THRESHOLD (maximum Altitude Low-Core Mesocyclone Base): The maximum circulation base altitude (km)
allowed to classify 3D features as low-core mesocyclones; default 3 km, range 0 to 5 km, precision: 1 km. This
value must be less than or equal to THRESHOLD (maximum 3D Couplet Core Base). This configuration parameter can be used to change MDA’s functionality.
THRESHOLD (Convergence Look Ahead): Maximum number of sample volumes allowed for the “lookahead” feature when building radial convergence pattern vectors; default 4 sample volumes, range 3 to 6, precision: 1. This configuration parameter can be used to change MDA’s functionality.
THRESHOLD (Convergence minimum Length): Minimum distance required between min and max velocity
values within a Convergence Vector; default 1.0, range 1.0 to 2.0 km, precision: 0.5 km. This configuration parameter can be used to change MDA’s functionality.
THRESHOLD (Convergence Delta Velocity): Minimum velocity difference allowed for saving radial convergence pattern vectors; default 5 m/s; range 4 to 8 m/s, precision: 1 m/s. This configuration parameter can be
used to change MDA’s functionality.
THRESHOLD (Convergence maximum Height): Maximum height allowed for saving radial convergence pattern vectors; default 8 km, range 4 to 8 km, precision: 1 km. This configuration parameter can be used to change
MDA’s functionality.
THRESHOLD (Convergence Shear): Minimum Shear required before convergence vectors need to be searched
for a shear core; default 1.0 ms-1 km-1 range 0.5 to 2.0 ms-1 km-1, precision: 0.5 ms-1 km-1. This configuration
parameter can be used to change MDA’s functionality.
THRESHOLD (Convergence Buffer Zone): Buffer zone outside the diameter of a 2D feature within which
radial convergence pattern vectors are to be associated with the 2D feature for convergence calculations; default
2 km, range 0 to 4 km, precision: 1 km. This configuration parameter can be used to change MDA’s functionality.
THRESHOLD (3D Feature Low-Level Convergence Height): The maximum altitude for calculating low-level convergence for 3D features; default 2 km, range 1 to 3 km, precision: 1 km. This value must be less than
THRESHOLD (3D Feature Mid-Level Convergence Height1). This configuration parameter can be used to
change MDA’s functionality.
THRESHOLD (3D Feature Mid-Level Convergence Height1): The minimum altitude for calculating mid-level
convergence for 3D features; default 2 km, range 2 to 4 km, precision: 1 km. This value must be greater than
THRESHOLD (3D Feature Low-Level Convergence Height). This configuration parameter can be used to
change MDA’s functionality.
THRESHOLD (3D Feature Mid-Level Convergence Height2): The maximum altitude for calculating mid-level
convergence for 3D features; default 4 km, range 3 to 6 km, precision: 1 km. This value must be greater than
THRESHOLD (3D Feature Mid- Level Convergence Height1). This configuration parameter can be used to
change MDA’s functionality.
THRESHOLD (max number of elevations in 3D Feature): The maximum number of elevations allowed in a
3D Feature; default 22, range 5 to 25 elevation slices, precision: 1. This parameter should equal the maximum
number of elevations of the WSR-88D VCP’s. This configuration parameter can be used to change MDA’s
functionality.
THRESHOLD (min Rank LT): Minimum strength rank for a low core mini-supercell and NSSL defined Mesocyclone; default 5, range 3 to 7, precision: 1 This configuration parameter can be used to change MDA’s functionality.
THRESHOLD (Shallow Allowed): Boolean flag indicating that MDA is allowed to identify and classify shallow
circulations; default TRUE; range TRUE - FALSE. This configuration parameter can be used to change MDA’s
functionality.
3-37

October 2017

•
•
•
•

•
•

THRESHOLD (Meso min Rank Shallow): Minimum strength rank for a shallow circulation; default 5, range 2
to 5, precision: 1. This configuration parameter can be used to change MDA’s functionality.
THRESHOLD (Meso min Depth Shallow): Minimum depth for a shallow circulation; default 1.0, range 0.5 to

### 1.5 km, precision: 0.5 km. This configuration parameter can be used to change MDA’s functionality.

THRESHOLD (Meso max Top Shallow): Maximum top for a shallow circulation; default 3 km, range 1 to 3
km, precision: 1 km. This configuration parameter can be used to change MDA’s functionality.
THRESHOLD (Overlap Display Filter): Boolean value that allows a user to turn on the algorithm based overlap display filter. The overlap display filter eliminates the display of 3D features that overlap lower elevation
features; default TRUE, range TRUE to FALSE. This adaptable parameter should have URC level of change
authority.
THRESHOLD (minimum Display Filter Rank): The minimum strength rank below which 3D Features are
marked not to be displayed; default 5, range 1 to 5, precision: 1. This adaptable parameter should have URC
level of change authority.
THRESHOLD (Use Near Storm Environment (NSE)): Boolean value that allows a user to turn on the algorithm’s use of NSE data.

3.2.8.9 Strengths/Applications.
• The algorithm automatically processes 3-dimensional velocity data to identify regions that resemble mesocyclones.
• A mid-level mesocyclone that lowers toward the surface may indicate a tornado is developing.
• The strength of MDA is that the algorithm allows for the detection of an entire spectrum of storm-scale circulations with different strength and spatial characteristics. These circulations are then classified using a variety of
definitions.
• The detection techniques have been made much more robust than in earlier versions of mesocyclone detection.
Individual forecasters/users can take advantage of the algorithm’s robustness by adjusting a display filter to
meet their needs.
• The MDA includes time association routines to provide tracking and trend data for the rapid synthesis of many
data points from a number of elevations and volume scans into single-parameter trends.
• Time-Height trends provide useful guidance information that cannot be easily calculated in the short amount of
time to make warning decisions.
3.2.8.10 Limitations.
• The algorithm only detects cyclonic rotations, not anticyclonic rotations.
• Identification is influenced by aspect ratio.
• Range thresholds may discard or improperly classify mesocyclones. No data within 5 km (2.7 nm) is processed
by the mesocyclone algorithm. The operator must search base products for evidence mesocyclones.
• The operator must examine reflectivity, velocity/SRM to verify existence of operationally significant mesocyclones.
• The radar horizon and cone-of-silence can prevent the radar from detecting circulations at times.
• Velocity signatures may be obscured or degraded where there are improperly dealiased velocity data or where
data are obscured by range-folded echoes.
• While MDA uses a minimum reflectivity threshold to confine the search for shear segments typically associated
with storms, MDA uses no reflectivity structures (BWER, Hook Echo, etc.) to identify tornadic circulations.
• The algorithm may falsely identify shear segments and 3D features in areas of higher reflectivity and high
spectrum width such as ground clutter, sea breezes, gust fronts, close to the RDA, and at the edge of the first
trip.
• The shear segment image is sized to hold 5000 shear segments. Additionally the total number of 2D and 3D
features are set to 800 and 200 respectively. If any of these array limits are exceeded, processing stops and potentially tornadic storms may go undetected.
3-38

FMH-11-Part C

•
•

Squall line often produce numerous, transient, operationally insignificant circulations along their leading
edge.
MDA will perform best on isolated supercell storms.


#### 3.2.9 Tornado Detection Algorithm.

Much of the information contained in this section can be found in Mitchell et al 1998. For more information about
the performance of the Tornado Detection Algorithm (TDA), see Mitchell 1995.
The TDA uses radar data to identify intense, small circulations that are producing or are likely to produce tornadoes.
The algorithm starts by identifying one-dimensional (1D) pattern vectors from (mean radial) velocity data. To help
limit the search of circulations to those associated with the low-levels of storm cells, the algorithm only searches
velocity data from sample volumes that 1) have reflectivities above a specified threshold, 2) are within a threshold
range, and 3) are below a threshold height. Pattern vectors are gate-to-gate velocity differences that exceed a specified velocity difference threshold. Gate-to-gate means the sample volumes are from adjacent radials and at the same
range. Next, for six (by default) differential velocity thresholds, the pattern vectors on each elevation scan that are in
azimuthal and horizontal proximity and exceed the same differential velocity threshold are combined into potential
two-dimensional (2D) features. Then, the potential 2D features are trimmed such that only one pattern vector remains at any range within the feature. Afterward, if a potential 2D still has enough pattern vectors and has a below
threshold aspect ratio, it is checked for overlap with all previously saved 2D features on the elevation scan. If the
potential 2D feature overlaps no other 2D features, it is saved as a new 2D feature. Next, the 2D features from adjacent elevation scans are vertically correlated into potential three-dimensional (3D) features. Potential 3D features
with enough 2D features are saved as 3D features. Lastly in the identification process, each 3D feature is compared
against thresholds to determine if it is an Elevated Tornado Vortex Signature (ETVS) or TVS or not. Finally, the
TVSs and ETVSs are associated with storm cells.
3.2.9.1 Pattern Vectors.
Pattern vectors are identified on each elevation scan from velocity data from azimuthally adjacent radials. (It is
assumed that radials are approximately 1 degree in azimuthal width and have no gaps between them.) The counter-clockwise velocity difference is computed for each pair of sample volumes (from the adjacent radials) that are
constant in range that are below a threshold height ARL and within a maximum processing range. If the sample
volumes have a velocity difference above a minimum threshold and their corresponding reflectivities are at least a
minimum threshold, then the pair is saved as a pattern vector. For each pattern vector the following information is
then computed: the range, azimuthal difference (between the radials), and the beginning and ending radials’ azimuths. If the number of pattern vectors found on an elevation scan ever exceeds a specified maximum number
allowed, the search for pattern vectors stops for the elevation scan.
3.2.9.2 2D Features.
Once all the pattern vectors have been identified on an elevation scan, they are combined into potential 2D features.
In order for two pattern vectors to be correlated to the same potential 2D feature, they must be within an azimuthal
and range separation proximity thresholds and exceed the same differential velocity threshold. Multiple differential
velocity thresholds are used to isolate core (i.e., stronger) circulations imbedded within long azimuthal shear regions
(e.g., radially oriented gust fronts). By default, six differential velocity thresholds are used to construct potential 2D
features. All the pattern vectors on an elevation scan are processed once for each differential velocity threshold,
starting with the greatest.
As the potential 2D features are built using each of the differential velocity thresholds, each of the potential 2D
features have their associated pattern vectors sorted by increasing range and then trimmed. The trimming results in
only one pattern vector for any range within the potential 2D feature and is accomplished in the following manner.
Each potential 2D feature is processed from beginning to ending range. At each range within the feature, the pattern vector retained at the next range is the one that is closest in azimuth to a reference (pattern) vector, and all others are trimmed (or not saved). If multiple pattern vectors are equally close in azimuth to the reference vector, then
3-39

October 2017

the one with the greatest velocity difference is selected. The reference vector changes for each new range within the
feature. For the first range, the reference vector is at the second range; otherwise, the reference vector is the pattern
vector retained at the previous (lesser) range. For the first range, the reference vector is the pattern vector on the
second range that is closest to the first pattern vector (at the first range).
Next, the 2D features are determined from the potential 2D features. First, the following potential 2D feature attributes are calculated: azimuth, range, height, X-coordinate, Y-coordinate, beginning azimuth, ending azimuth, beginning range, ending range, # of pattern vectors, maximum velocity difference, average elevation, maximum shear,
azimuthal diameter, radial diameter, and the aspect ratio. If a potential 2D feature’s aspect ratio is less than a specified threshold value, then the feature is compared for overlap with all previously saved 2D features on that elevation
scan. When a feature overlaps another, its boundaries (i.e., beginning and ending azimuths and ranges) exceed those
of the other feature. If the potential 2D feature overlaps no other 2D features, it is saved as a new 2D feature. If
the potential 2D feature overlaps one 2D feature, the 2D feature acquires many of the potential 2D features attributes, e.g., range, azimuth, height, etc. If the potential 2D feature overlaps more than one 2D feature, it is deleted
(i.e., not saved as a 2D feature). Finally, after all the 2D features are found on the elevation scan, they are sorted by
decreasing maximum delta velocity (see Figure 3-16). If and when the number of 2D features in the volume scan
meets or exceeds the threshold maximum number allowed in the volume scan, then processing immediately skip
over the remainder of the 2D functionality to the 3D functionality.
3.2.9.3 3D Features, TVSs, and ETVSs.
Once all of the 2D feature have been constructed and saved for the volume scan, the algorithm vertically correlates
the 2D features from different elevation scans into 3D features. Starting with the lowest elevation scan, for each
remaining 2D feature, a new potential 3D feature is started with the 2D feature. Then, all other 2D features on the
elevation scan within the circulation radius are discarded from future 3D processing. Remember, the 2D features on
each elevation are sorted by decreasing maximum delta velocity; so the first 2D feature found always has the strongest maximum delta velocity. Also, once a 2D feature is assigned to a potential 3D feature it is removed from future
consideration in other potential 3D features. Then, the 2D features on the next elevation scan are searched until
one is found within the circulation radius of the last 2D feature assigned to the potential 3D feature. The first 2D
feature found is vertically correlated into the same potential 3D feature. And, all other 2D features on that elevation
scan within the circulation radius are discarded. If no 2D features are found at the next elevation scan within the
circulation radius, then that elevation scan is skipped, and the following elevation scan is similarly searched for a 2D
feature to vertically correlate. Only one elevation scan can be skipped in the vertical stack of 2D features. This one
elevation scan gap provides some flexibility to allow for 2D features that should be part of the potential 3D feature
but were missed possibly because of range aliasing or improper velocity dealiasing. Once all elevation scans are processed for that 3D feature, the entire vertical correlation process is repeated starting with the next undiscarded and
uncorrelated 2D feature on the lowest elevation scan.

3-40

FMH-11-Part C


> **Figure 3-16: Two-Dimensional Features.**

> Multiple velocity thresholds used to identify stronger shear embedded within weaker shear.

Multiple velocity thresholds used to identify stronger shear embedded within weaker shear.
Once all potential 3D features have been found, they are thresholded to determine if they are 3D features, TVSs, or
ETVSs. First, the 2D features within each potential 3D feature are sorted by increasing height. Then, if a potential
3D feature contains at least a minimum threshold number of 2D features and the number of 3D features is less
than a threshold number, it is saved as a 3D feature, and its attributes (e.g., base height, shear, and maximum delta
velocity) are computed. Next, each 3D feature is checked to determine whether it is a TVS Feature; a TVS Feature
is a TVS or ETVS (Figures 3-17 and 3-18). If a 3D feature has at least a minimum threshold depth and a minimum
threshold velocity difference and if its base is above a minimum elevation angle and height thresholds (i.e., its base
is not on the lowest elevation angle or below a certain height), the 3D feature is saved as an ETVS. Otherwise, if
the 3D feature has at least a minimum threshold depth and if it’s base maximum delta velocity or maximum delta
velocity is above threshold, the 3D feature is saved as a TVS. When saving a TVS Feature, if the number of TVSs
or ETVSs meets or exceeds the threshold maximum number allowed, the features (TVSs or ETVSs) are sorted and
the one with the smallest TVS (Base Delta Velocity) and, secondly, the smallest maximum TVS (Delta Velocity) is
discarded. Lastly, each TVS Feature is associated with the nearest storm cell that is within a threshold maximum association distance. When a TVS Feature is associated with a storm cell it is assigned the same ID. If a TVS Feature
is not within the threshold distance from any storm cell, it has an ID of “??”.
3.2.9.4 Rapid Update.
In a fashion similar to the Mesocyclone Algorithm Rapid Update, in the TDA Rapid Update features are updated with each elevation angle. Rapid update allows TDA to output 3D feature information at the end of every tilt,
beginning with the first tilt, rather than only at the end of the volume scan. As 3D features are built, 3D feature
information is updated so that forecasters can see if 3D features are increasing in intensity. Rapid update increases
3-41

October 2017

algorithm output lead-time, allowing forecasters more time to view algorithm output and issue severe weather warnings.
After 2D features have been identified on the first tilt of a new volume scan, their locations are compared to forecast locations of 3D features from the previous volume scan. This comparison is described in the time association
step of the TDA tracking module. The time association information defines 3D feature tracks. However, in contrast
to the Mesocyclone Algorithm the TDA requires 3 contiguous elevations of 2D couplets before combining them
into a 3D TVS.
After time association, before TDA processes data from the second tilt, the direction and speed of each detection is
computed and the position of severe weather feature icons and table attribute information on the display are updated for the current volume scan. Low-level attributes of the 2D feature on the current volume scan; low-level characteristics such as shear and gate-to-gate velocity difference; are assigned and the rest of the attributes are inherited
from time associated 3D features detected on the previous volume scan.
The 3D features that are not time associated with 2D features found on the new volume scan are identified as “extrapolated” features. If 3D features from the previous volume scan are associated with features on the current volume scan, their classification is changed from extrapolated to matched. Extrapolated features that are not matched
on the current elevation scan or the next are removed from the product.


> **Figure 3-17 TVS Definition**


3-42

FMH-11-Part C


> **Figure 3-18 ETVS Definition**


3-43

October 2017


> **Figure 3-19: TVS Graphic Product.**

> An example Tornado Vortex Signature product (OPUP Display) overlaid on a Reflectivity Data Array

An example Tornado Vortex Signature product (OPUP Display) overlaid on a Reflectivity Data Array
product from the Oklahoma City, OK (KTLX) WSR-88D on 31 May 2013 at 23:09 UTC. The corresponding alphanumeric product is also displayed.
When the RDA provides radar data from higher tilts, 1D and 2D features are identified by TDA. New 2D features are vertically associated with adjacent-tilt 2D features, 3D features, or retained as potential bases of new 3D
features. Feature attributes are updated when: 1) a new 3D feature is identified during the current volume scan; 2)
feature attributes indicate a TVS is increasing in severity; or 3) all information has been obtained for a TVS (i.e., the
feature is topped).
At the end of the volume scan TDA computes an average motion and speed of all 3D features and computes forecast positions for all 3D features and the resulting product generated (see Figure 3- 19).
3-44

FMH-11-Part C

3.2.9.5 Operational Parameters.
• AZIMUTH: The azimuthal position of a radial in degrees.
• ELEVATION: The elevation angle of the radial or scan, in degrees.
• RADIAL: The set of sample volumes, only one at each RANGE (Slant), along a constant AZIMUTH and ELEVATION.
• RADIUS (Earth): The radius of the Earth (6371), in km.
• RANGE (Slant): The slant range to the center of a SAMPLE VOLUME, in km.
• REFLECTIVITY FACTOR (Sample Volume): The effective radar reflectivity factor assigned to a (velocity)
SAMPLE VOLUME, in dBZe.
• SAMPLE VOLUME: A data sample volume along a radial whose (half power) dimensions are described by
the azimuthal and vertical beam widths and the range sampling interval. These dimensions are approximately 1
degree in azimuthal and vertical width (perpendicular to the beam) and 0.25 km in range (or length) for Velocity
sample volumes and 1.0 km in range for Reflectivity (Factor) sample volumes.
• STORM CELL (ID): IDs are a set of unique labels for algorithm identified storm cells.
• STORM CELL (X-coord): The set of x-coordinates for algorithm identified storm cells, in deg.
• STORM CELL (Y-coord): The set of y-coordinates for algorithm identified storm cells, in deg.
• VELOCITY (Sample Volume): The mean radial velocity of a SAMPLE VOLUME, in m/s.
• THRESHOLD (2D Vector Azimuthal Distance): The maximum AZIMUTH distance allowed for two Pattern
Vectors to be associated into the same 2D Feature, in degrees; default 1.5°, range 0.0° to 4.0°.
• THRESHOLD (2D Vector Radial Distance): The maximum radial distance allowed between two Pattern Vectors to be associated into the same 2D Feature, in km; default 0.5 km, range 0.0 km to 3.0 km.
• THRESHOLD (Average Delta Velocity Height): The minimum height below which all 2D Features comprising
a 3D Feature are assigned an equal weighting of 1, in km; default 3.0 km, range 0.0 km to 10.0 km.
• THRESHOLD (Circulation Radius1): The maximum horizontal radius used for searching for 2D Features
on adjacent or the same ELEVATION scans in building a 3D Feature. This radius is used when the RANGE
(Slant) of an assigned 2D Feature is less than or equal to THRESHOLD (Circulation Radius (Range)), in km;
default 2.5 km, range 0.0 to 10.0 km.
• THRESHOLD (Circulation Radius2): The maximum horizontal radius used for searching for 2D Features
on adjacent or the same ELEVATION scans in building a 3D Feature. This radius is used when the RANGE
(Slant) of an assigned 2D Feature is greater than THRESHOLD (Circulation Radius [Range]), in km; default 4.0
km, range THRESHOLD (Circulation Radius1) to 10.0 km.
• THRESHOLD (Circulation Radius (Range): The RANGE (Slant) beyond which THRESHOLD (Circulation
Radius2) is invoked, otherwise THRESHOLD (Circulation Radius1) is used, in km; default 80 km, range 0 to
230 km (124 nm).
• THRESHOLD (Differential Velocity): Six velocity difference thresholds used as criteria for building 2D Features, in m/s; defaults 11, 15, 20, 25, 30, 35 m/s; ranges 10 to 75, 15 to 80, 20 to 85, 25 to 90, 30 to 95 and 35 to
100 m/s. Note: 1) The first threshold should be equal to Vector Velocity Difference; 2) Threshold values should
increase from smallest to largest; 3) It is recommended that the difference between successive threshold values
not exceed 5 m/s (e.g., 20,25,30,35,40,45 m/s).
• THRESHOLD (maximum # 2D Features): Maximum number of 2D Features the algorithm can process per
volume scan; default 600, range 600 to 800.
• THRESHOLD (maximum # 3D Features): Max total number of 3D Features the algorithm can process per
volume scan; default 35, range 30 to 50.
• THRESHOLD (maximum # ETVS): Maximum number of Elevated TVS’s the algorithm can process per volume scan; default 20, range 15 to 25.

3-45

October 2017

•
•
•
•
•
•
•
•
•
•
•
•
•
•
•

THRESHOLD (maximum # Pattern Vectors): Maximum number of Pattern Vectors the algorithm can process
per elevation scan; default 2500, range 1500 to 3000.
THRESHOLD (maximum # TVS): Maximum number of TVS’s the algorithm can process per volume scan;
default 15, range 15 to 25.
THRESHOLD (maximum 2D Feature Aspect Ratio): The maximum allowable aspect ratio (ΔRANGE (Slant)/
ΔAZIMUTH) for a 2D Feature, in km/km; default 4.0, range 1.0 to 10.0 km/km.
THRESHOLD (maximum Pattern Vector Height): The maximum height at which Pattern Vectors are identified, in km; default 10.0 km, range: 1.0 to 15.0 km.
THRESHOLD (maximum Pattern Vector Range): The maximum RANGE (Slant) at which pattern vectors are
identified, in km; default 100 km, range 0 to 230 km (124 nm).
THRESHOLD (maximum Storm Association Distance): The maximum distance from a storm within which to
associate TVS and ETVS detections with storm cell detections. Association is not required to declare a TVS or
ETVS detection, in km; default 20.0 km, range 0.0 to 20.0 km.
THRESHOLD (minimum # 2D Features Per 3D Feature): The minimum number of 2D Features needed to
make a 3D Feature (TVS or ETVS); default 3, range 1 to 10.
THRESHOLD (minimum # of Pattern Vectors Per 2D Feature): The minimum number required to declare a
2D Feature; default 3, range 1 to 10.
THRESHOLD (minimum 3D Feature Depth): The minimum depth required to declare a TVS or an ETVS, in
km; default 1.5 km, range 0.0 to 5.0 km.
THRESHOLD (minimum 3D Feature Low-Level Delta Velocity): The minimum radial velocity difference at the
base ELEVATION scan required to declare a TVS or ETVS, in m/s; default 25 m/s, range 0 to 100 m/s.
THRESHOLD (minimum Reflectivity): The minimum reflectivity value required in a SAMPLE VOLUME for it
to be used in a Pattern Vector, in dBZ; default 0 dBZ, range -20 to 20 dBZ.
THRESHOLD (minimum TVS Base Elevation): The lowest ELEVATION angle to which the base of a 3D
Feature must extend to declare a TVS, in degrees; default 1.0°, range: 0.0° to 10.0°. Either height or ELEVATION criteria must be met to declare a TVS.
THRESHOLD (minimum TVS Base Height): The minimum height ARL to which the base of a 3D circulation
must extend to be declared a TVS, in km; default 0.6 km, range 0. to 10.0 km. Either height or ELEVATION
criteria must be met to declare a TVS.
THRESHOLD (minimum TVS Delta Velocity): The minimum radial velocity difference of the maximum 3D
Feature delta velocity required to declare a TVS detection, in m/s; default 36 m/s, range 0 to 100 m/s.
THRESHOLD (Vector Velocity Difference): The minimum required gate-to gate velocity difference required
for Pattern Vectors, in m/s; default 11 m/s, range 10 to 75 m/s. This threshold should be equal to the first
THRESHOLD (Differential Velocity).

3.2.9.6 Strengths/Applications.
• The algorithm searches for gate-to-gate shear and detects TVSs, which are related to tornadic circulations.
• Multiple velocity-difference thresholds make it possible to isolate small regions of shear within broader regions
and allow performance tuning through adaptable parameter changes.
• A distinction is made between different types of shears (TVS vs. ETVS, delta velocity calculations), and more
information is provided about the base and depth of circulations.
• The TDA vector velocity threshold is not range dependent. Thus, pattern vectors, and thus circulations in TDA
2D and 3D processing, are given equal weight regardless of range.
• Many of the adaptable parameters allow the TDA to become more sensitive, i.e., identify more circulations.
• The TDA is based on the paradigm that TVSs associated with tornadoes may be sampled by adjacent radar
beams (Doviak and Zrnic 1975, Brown et al 1978). Ideally, though, it is desired that each beam be centered on
the velocity peaks of the circulation. As a result, less stringent Differential Velocities are used in the TDA which
3-46

FMH-11-Part C

•
•
•

affords a greater number of detections.
A more robust vertical association scheme is employed to help avoid false detections. In light of modern computing, the TDA has been designed to allow a greater amount of information to be processed.
The TDA operates independently of any mesocyclone detection algorithm. Therefore, the TDA is allowed the
freedom to identify tornadic circulations within storms that do not contain mesocyclones (e.g., non-supercell
tornadoes) and storms that contain undetected mesocyclones.
Currently, the TDA has a higher Probability of Detection (POD) than the previous TVS Algorithm and performs best during events characterized by isolated supercells.

3.2.9.7 Limitations.
• Adaptable parameters need more research. Parameters which work well in one type of meteorological setting
may not be as effective in other situations.
• High false alarm rates especially in squall lines and tropical cyclones. A high False Alarm Ratio (FAR) with TDA
may result in over-warning, or desensitizing forecasters. False alarms have also been caused by vehicle traffic
during clear, cold, inversion conditions.
• Little research has been done to date relating the occurrence of tornadoes to ETVSs. Forecasters should use
ETVS output with caution until they develop a better understanding of its utility.
• Beyond 80 km (50 nm) TDA most likely detects strong mesocyclones. No circulations are identified beyond
approximately 100 km (62 nm).
• Doppler radar’s ability to measure thunderstorm mesocyclones and tornadoes is primarily dependent on the
relationship between the vortex size and the size of the radar sample volume.
• Discrete azimuthal sampling may or may not coincide with the peak rotational velocities in the TVS.
• Signature position in relation to storm structure should be used to filter false detections.
• Time and range continuity must be considered.
• Classification as INC (increasing) or persistent (PER) may be the result of sampling issues versus an actual
change of the feature.
• The TRU graphical attribute table and alphanumeric product contain attributes from both the previous and
current volume scan.
• Feature matching ability is dependent on the motion supplied by the SCIT Algorithm.
• There is no functionality within the 3D processing that filters multiple circulations in close horizontal proximity.
Therefore, multiple TVSs and ETVSs can be detected very close to each other.
• Improperly dealiased velocity data will degrade the algorithm performance.
• Discarded velocity bins (possible with some dealiasing techniques) can degrade algorithm performance.
• More testing and data analysis is required to determine more accurately how the TDA performs in various weather
scenarios.
• Range folded echoes often obscure the velocity data making the detection of pattern vectors (and, hence, circulations) impossible.
• While the TDA uses a Minimum Reflectivity Threshold to confine the search for pattern vectors within the
higher reflectivities typically associated with storms, the TDA uses no reflectivity structures (BWER, Hook
Echo, etc.) to identify tornadic circulations.
• The algorithm may falsely identify pattern vectors in areas of higher reflectivity such as ground clutter, sea
breezes, and gust fronts.
• The algorithm only detects pattern vectors, and thus TVSs, with cyclonic shear.
• Because of sampling limitations at very close ranges, large tornadic circulations may span several radials. In
the middle of these circulations there may be very little shear observable in the radial velocity data. Therefore,
in these cases, the TDA may miss this type of circulation (i.e., not identify it as a TVS) or identify two circulations.
3-47

October 2017

•
•

Squall line events present a challenge to the TDA because numerous, transient, TVSs are often detected along
the leading edge. This is especially true when the squall line is aligned along a radial creating near zero radial
velocities.
The TDA does not use range dependent velocity difference thresholds. Thus, TVSs are given equal weight regardless of range.


#### 3.2.10 Radar Echo Classifier.

The Radar Echo Classifier (REC) Algorithm is a real-time software package that processes base radar data through a
“fuzzy” logic process to determine the likelihood that the radar is detecting a specific category of target. The likelihood is defined for each base reflectivity and Doppler (velocity and spectrum width) bin for each elevation. Initially,
the REC was designed to determine the likelihood that the radar is detecting AP/ground clutter. Future plans are to
refine the determining process to identify other specific target categories (precipitation, large hail, biological, clutter
residue, etc.) and to help discriminate between different phenomena (rain/snow, convective/stratiform, etc.).
The REC output includes digital arrays that contain the bin-by-bin likelihood (in percent) that the specific target
category has been identified. The digital files are intended for use by other real-time algorithms (for instance, PRECIPITATION PREPROCESSING, COMPOSITE REFLECTIVITY, etc.) that need to make decisions based on
the target identification. The REC output also includes graphic products that depict the likelihood of target identification. The graphic products are intended to assist users in making operational and maintenance decisions and to
increase operator confidence in REC identifications.
The REC identification is based on a comparison of the pattern of the base radar data over a small bounded area
with the expected patterns of specific targets. These bounded PATTERN CHARACTERISTICS have been selected
based on their ability to discriminate between different target categories.
The first major component of the REC Algorithm computes PATTERN CHARACTERISTIC values for each base
data radar bin. The PATTERN CHARACTERISTIC value is computed using the highest resolution, accuracy, and
precision of the base radar data.
The second major component of the REC applies the TARGET CATEGORY SCALING FUNCTIONS to each
PATTERN CHARACTERISTIC value to generate a set of SCALED CHARACTERISTIC values. For each radar
bin and each PATTERN CHARACTERISTIC, the SCALED CHARACTERISTIC value represents the likelihood
that the PATTERN CHARACTERISTIC value indicates the selected target.
The third major component of the REC defines the TARGET LIKELIHOOD by weighting the SCALED CHARACTERISTIC values using the TARGET PROBABILITY WEIGHTS and then summing the weighted SCALED
CHARACTERISTIC values. For each radar bin, the TARGET LIKELIHOOD expresses the likelihood that the
radar information from that bin is a member of the target class.
The RADAR ECHO CLASSIFIER processing will be performed in real time. The resulting digital output will be
provided for use by the Precipitation Processing Subsystem (PPS) Preprocessing Algorithm and other algorithms as
needed. In addition, graphical depictions of the REC output will be made available for visual analysis.
3.2.10.1 Operational Parameters.
• BASE REFLECTIVITY: Base reflectivity data. Accuracy is defined by the accuracy of the base radar data, currently 0.5 dBZ for the WSR-88D.
• BASE VELOCITY: Base Doppler velocity data. Accuracy is defined by the accuracy of the base radar data,
currently 0.5 m/s for the WSR-88D.
• BASE SPECTRUM WIDTH: Base spectrum width data. Accuracy is defined by the accuracy of the base radar
data, currently 0.5 m/s for the WSR-88D.
• REFLECTIVITY AZIMUTH ANGLE: Azimuthal position information for the BASE REFLECTIVITY radial. Accuracy of at least 0.1 degree.
3-48

FMH-11-Part C

•
•
•
•
•
•
•
•

DOPPLER AZIMUTH ANGLE: Azimuthal position information for the BASE VELOCITY radial. Accuracy
of at least 0.1 degree.
TARGET CATEGORY SCALING FUNCTIONS: Predetermined functions for each TARGET CATEGORY and each PATTERN CHARACTERISTIC used to scale the PATTERN CHARACTERISTIC value to the
SCALED CHARACTERISTIC value, generates a likelihood ranging from 0 to 1.
TARGET CATEGORY PROBABILITY WEIGHTS: Probability weighting values for each TARGET CATEGORY which are multiplied to the SCALED CHARACTERISTIC value to derive the TARGET LIKELIHOOD.
SPIN CHANGE THRESHOLD: Difference in reflectivity between successive range gates to be considered
significant “spin”. Accuracy of at least 0.5 dBZ.
SPIN REFLECTIVITY THRESHOLD: Reflectivity value below which no “spin” is computed. Accuracy of at
least 0.5 dBZ.
RADIAL_EXTENT: The number of radials (+/-) used to define the region for computing ZTXTR, ZSIGN,
and VSTDV. Accurate to one radial.
Z_RANGE_EXTENT: The number of reflectivity range bins (+/-) used to define the region for computing
ZTXTR and ZSIGN. Accurate to one reflectivity range bin.
D_RANGE_EXTENT: The number of Doppler range bins (+/-) used to define the region for computing
VSTDV. Accurate to one Doppler range bin.

3.2.10.2 Strengths/Applications.
• The algorithm produces a digital output, the Target Likelihood Array that is provided for use by the PPS. The
Target Likelihood Array expressed the likelihood that a target is AP or clutter.
• This algorithm assists in the identification of clutter, AP echo, and thus, precipitation.
3.2.10.3 Limitations.
• Because the PATTERN CHARACTERISTICS of different targets may be similar at times, the REC will not
likely generate a unique and specific identification at each location.
• This Algorithm description assumes the azimuthal resolution/spacing of the reflectivity information is the same
as the Doppler azimuthal resolution. In addition, this description assumes a Doppler range resolution of 250 m
(0.13 nm) and a reflectivity range resolution of 1000 m (0.54 nm). This may be changed in time.
• The REC presents the identification as a probability (likelihood) and requires that the user of the information
(whether human or algorithm) determine a level of confidence and tolerance in the decision process.
• The algorithm cannot now assign a likelihood that a target is biological in nature.
• The REC clutter likelihood percentage values tend to be biased toward higher values in areas of range overlaid
data.

#### 3.2.11 Precipitation Processing Subsystem.

The Precipitation Processing Subsystem (PPS) is a set of hydrometeorological algorithms used to compute maps of
1-hour, 3-hour, storm total and user selectable precipitation accumulations. Functionally, this is done in five sequential steps:
Step 1. Enhanced PPS Preprocessing (EPRE)
Step 2. Precipitation Rate
Step 3. Precipitation Accumulation
Step 4. Precipitation Adjustment
Step 5. Precipitation Products.
These algorithms were designed to provide accurate, high-resolution precipitation measurements to be used as input
3-49

October 2017

to hydrologic river forecast models and for a variety of other applications. The algorithms contain significant quality
control procedures designed to minimize ground clutter and anomalous propagation, improve range performance,
eliminate outliers, and account for mean-field bias. The PPS is tuned to provide accumulations that are generally unbiased in terms of averages over the area of coverage. Smaller scales of precipitation accumulation are less accurate
and single grid accumulations often vary significantly from coincidental rain gage values.
Although the WSR-88D provides high quality reflectivity data, other sources of error in the conversion of reflectivity to precipitation rate make it important to use data in real time from several automated rain gages to adjust the
radar-derived precipitation fields to “ground truth.” A source of potential error is in rain gage sampling. Non-representative sampling of the precipitation caused by an inadequate number of gages, poor placement of gages, an unfortunate distribution of precipitation patterns, or error in the gage estimates can affect the computation of the bias
adjustment. To the extent possible, the Precipitation Adjustment algorithm described in Section 3.2.11.4 is designed
to take these factors into account.
3.2.11.1 Enhanced Precipitation Preprocessing.
The advantages of the EPRE algorithm are to allow for precipitation processing with new VCPs and to update
some processing logic. This logic considers terrain blockage and clutter contamination on a point-by-point basis in
building the Hybrid Scan. The EPRE algorithm affects how the reflectivity data are processed prior to converting to
rainfall rate, such as which elevation is used for a particular range and azimuth. There are several operational impacts that affect product generation as well as the user interface, such as changes to adaptable parameters.
• New VCPs
• PPS and EPRE
• Exclusion Zones
• AP/Clutter Removal
• EPRE Effects on Precipitation Product Appearance
The EPRE algorithm provides the ability to accept new VCPs for precipitation processing. The EPRE algorithm is
the first component of the PPS. The EPRE assembles a two-dimensional (230 km (124 nm) by 360 degrees) HYBRID SCAN (Reflectivity) Array from the volumetric reflectivity base data for use by the PPS and the Radar Coded
Message (RCM). In the future, other algorithms may use the HYBRID SCAN (Reflectivity) Array.
The selection of reflectivity data are based on the functional philosophy of the original Preprocessing Algorithm,
i.e., that the volumetric reflectivity data that is most representative of ground precipitation is obtained from the
lowest radar beam that is neither blocked nor contaminated by ground clutter. While preserving the philosophy of
the Preprocessing Algorithm, the EPRE applies refined logic and new information to generate HYBRID SCAN
(Reflectivity) arrays that are more representative of precipitation fields. In addition, the EPRE was designed to work
with any WSR-88D VCP.
Specifically, the EPRE uses anomalous propagation (AP)/Clutter likelihood reflectivity (CLR) information from
the Radar Echo Classifier (REC) Algorithm to more effectively and precisely remove ground clutter contamination
from the HYBRID SCAN (Reflectivity) and uses high precision, high resolution radar beam blockage information
from the Blockage Algorithm to smooth beam correction boundaries (see Figures 3-20 and 3-21 for an example of
the difference in reflectivities the REC produces in creating a Digital Hybrid Scan Reflectivity (DHR) product).
Exclusion Zones may be created to prevent known areas of persistent clutter residue (e.g., wind generator farms,
highways) from contaminating the HYBRID SCAN (Reflectivity). These zones are used to remove residual clutter
returns when the clutter filters are not able to remove the high power returns.
The EPRE maps input base reflectivity data to the whole degree HYBRID SCAN (Reflectivity) array if the input
data meets the following criteria:
• The beam blockage must not exceed the BLOCKAGE THRESHOLD,
• The AP/Clutter likelihood must not exceed the CLUTTER THRESHOLD, and
3-50

FMH-11-Part C

•

The data are not in an activated exclusion zone.

An input BASE REFLECTIVITY DATA bin that passes the above tests is corrected for any partial beam blockage
and proportionally mapped to the nearest whole degree HYBRID SCAN (Reflectivity) bins. The proportionality is
based on the overlap between the input radial azimuth angle and the whole degree azimuth angle of the HYBRID
SCAN (Reflectivity). The overlap is used to weight the contribution of the input reflectivity data (in power) and the
weights and weighted power are summed for each whole degree bin.
The average power for a whole degree bin is computed if the total input weighting for that bin exceeds the BIN
WEIGHT THRESHOLD. The average reflectivity is computed and inserted into the HYBRID SCAN (Reflectivity)
if that bin has not already been filled during a lower elevation tilt. For each HYBRID SCAN bin, the elevation angle
of the BASE REFLECTIVITY DATA being used in the HYBRID SCAN (Reflectivity) is stored in the HYBRID
SCAN (Elevation Angle) array.

3-51

October 2017


> **Figure 3-20: Reflectivity Enhanced Resolution Product.**

> This Reflectivity Enhanced Resolution product (OPUP display) at the 0.5° elevation is shown for comparison with the corresponding Digital Hybrid Scan Reflectivity product in Figure 3-21.

This Reflectivity Enhanced Resolution product (OPUP display) at the 0.5° elevation is shown for comparison with the corresponding Digital Hybrid Scan Reflectivity product in Figure 3-21.

3-52

FMH-11-Part C


> **Figure 3-21: Digital Hybrid Scan Reflectivity Product**

> This Digital Hybrid Scan Reflectivity product (OPUP Display) is shown for comparison with the 0.5°

This Digital Hybrid Scan Reflectivity product (OPUP Display) is shown for comparison with the 0.5°
Reflectivity Enhanced Resolution product in Figure 3-20 to show how the Radar Echo Classifier
removes clutter before processing to produce precipitation products. Note that most of the high reflectivity values to the northeast clockwise through the southwest have been removed. The precipitation
echoes have been preserved.
The HYBRID SCAN (Reflectivity) array is considered to be filled when the portion of the array filled exceeds the
FULL HYBRID SCAN THRESHOLD or when the next to highest elevation in the volume has been completed
(whichever comes first). The “highest elevation” test is only applied to prevent the PPS volumetric processing from
occurring at the end of the volume, thus preserving that period for processing other processor intensive algorithms.
It is expected that very few HYBRID SCAN (Reflectivity) bins will not be filled because of this restriction.
3-53

October 2017

When the HYBRID SCAN (Reflectivity) array is full, the EPRE computes the area of echo covered by reflectivity
values exceeding RAIN DETECTION DBZ THRESHOLD. If the area is less than the RAIN DETECTION
AREA THRESHOLD, the NO RAIN FLAG is set to notify further PPS algorithms to ignore the HYBRID SCAN
(Reflectivity) data values. If the NO RAIN FLAG is continuously set for a time period equal to or exceeding RAIN
DETECTION TIME THRESHOLD, the RESET PPS STORM TOTAL FLAG is set. Note, this logic replicates the
PDF logic, but uses the HYBRID SCAN (Reflectivity).

3.2.11.1.1 Operational parameters.
• AP/CLUTTER ID: For each tilt, an array of bins identifying the likelihood of AP/Clutter for each BASE REFLECTIVITY DATA bin. Values range from 0 to 100 percent and the precision is at least 1 percent.
• BASE REFLECTIVITY DATA: Radar base reflectivity data. Accuracy is defined by the accuracy of the base
data. Currently reflectivity values range from ~ -32 dBZ to +90 dBZ and the precision is 0.5 dBZ.
• BEAM BLOCKAGE: For each VCP for each tilt, an array of (3600 X 230) bins defining the portion of the
radar beam blocked for each tenth of a degree azimuth and each whole kilometer in range. Values range from 0
to 100 percent and the precision is at least 1 percent.
• BEAM WIDTH: Width of the radar beam. Values may range from 0.88° to 0.96°, default is 0.90°, and the precision is at least 0.01°.
• BIN WEIGHT THRESHOLD: Bin weight required to compute average power value for HYBRID SCAN
(Reflectivity). Expected values range from 0.0 to 100.0 percent, default is 50.0 percent, and the precision is 0.1
percent.
• BLOCKAGE THRESHOLD: Maximum portion of beam blocked to allow use of BASE REFLECTIVITY
DATA in HYBRID SCAN (Reflectivity). Expected values range from 0 to 100 percent, default is 50 percent,
and the precision is 1 percent.
• CLUTTER THRESHOLD: Maximum AP/CLUTTER ID value to allow use of BASE
• REFLECTIVITY DATA in HYBRID SCAN (Reflectivity): Expected values range from 0 to 100 percent, default is 50 percent, and the precision is 1 percent.
• DATE (BEGIN VOLUME): Julian date at the beginning of volume. Precision 1 day.
• DATE (END OF REFLECTIVITY TILT): Julian date at the end of the reflectivity tilt. Precision 1 day.
3.2.11.1.2 Strengths/Applications.
See Section 3.2.11.5.2.
3.2.11.1.3 Limitations.
See Section 3.2.11.5.3.

3.2.11.2 Precipitation Rate.
The PRECIPITATION RATE Algorithm executes each time the PRECIPITATION PREPROCESSING Algorithm is completed. If the FLAG (Zero Hybrid) is on, the algorithm sets the FLAG (Zero Rate) and updates the
reference values for the time continuity test. Otherwise, the PRECIPITATION RATE Algorithm uses preprocessed
reflectivity factor (HYBRID SCAN) data from the PRECIPITATION PREPROCESSING Algorithm to estimate
precipitation rates for 1° by 2 km (1.1 nm) sample volumes within a radius of 230 km (124 nm). The RATE SCAN
is produced for input to the PRECIPITATION ACCUMULATION Algorithm. Also, three quality control related
procedures are performed within the PRECIPITATION RATE Algorithm.
Precipitation rates are empirically determined from a relationship with reflectivity factor data. The precipitation rates
from two adjacent 1° by 1 km (0.54 nm) volumes along the same radial are averaged to obtain values for the 1° by
2 km (1.1 nm) RATE SCAN. The RATE SCAN is comprised of 41,400 1° by 2 km (1.1 nm) sample volumes. The
rate values are in dBR’s.
Based on the time continuity of the total field volumetric precipitation rate on a scan-to-scan basis, a decision is
made whether the current RATE SCAN should be used by the PRECIPITATION ACCUMULATION Algorithm
3-54

FMH-11-Part C

or be discarded. This test is intended to identify those cases where the between scan increase/decrease of the total
volumetric precipitation rate is greater than the increase/decrease expected from precipitation development/decay.
These changes could occur as a result of spurious RF interference, transient system noise, or anomalous propagation. Echo areas from storms entering/leaving the scanning region between scans could also cause this parameter to
suddenly increase/decrease.
To minimize the chance of rejecting scans because of echo movements into and out of the field of view, the total
volumetric precipitation rate is examined for both the entire field of view and for an area with a radius somewhat
less than 230 km (124 nm) (inner radius). The inner radius is computed from a climatological maximum speed for
echo movement and the time between scans. This test is only considered valid if the time between the current and
last good scan is less than a maximum difference at which time continuity is expected.
A range effect correction is then applied to all RATE SCAN values beyond a specified cut-off range. The correction function contains three coefficients which may vary from site to site and with the season. Inputs to the correction function are the range and precipitation rate. This procedure corrects for the effects of signal degradation
due to beam losses and partial beam filling which, on the average, reduce the precipitation rate estimates at further
ranges.
The area-averaged precipitation rates over each 1/4 LIMITED FINE MESH (LFM) rectangular grid box (approximately 40 km x 40 km (22 nm x 22 nm)) are computed for those boxes whose centers are located within 230 km
(124 nm) of the radar. These are obtained by averaging the rates from all RATE SCAN sample volumes whose
centers fall within each 1/4 LFM grid box. These data will be used further downstream at the regional/national
processing level for important quality control applications and possibly for the construction of a National Radar
Summary Chart.
Reflectivity factor data being used by this algorithm are assumed to have been pre-processed as described in the
ENHANCED PRECIPITATION PREPROCESSING Algorithm. The HYBRID SCAN (1° by 1 km (0.54 nm))
data have not been spatially averaged to obtain the 1° by 2 km (1.1 nm) resolution required for precipitation processing. A precipitation rate estimate based on the averaged reflectivity is not identical to the average of the precipitation rates based on the full resolution reflectivity data. Therefore, each pair of 1° x 1 km (0.54 nm) reflectivity
values being used to estimate a 1° x 2 km (1.1 nm) precipitation rate are first converted from reflectivity to precipitation rate and then averaged to obtain a RATE SCAN (1° x 2 km (1.1 nm)) value.
The time continuity test checks whether scans are bad. Bad scans are rejected or discarded from further processing
by the PRECIPITATION ACCUMULATION and subsequent algorithms. This information is saved so that the
number of bad scans can be appended to some of the final precipitation products. The intent of this test is not to
identify all cases where bad data may be present. It provides a simple means to remove scans which indicate sudden
and unreasonable echo development/ decay.
The LFM grid is a rectangular grid based on a polar stereographic projection. An LFM grid box represents an area
whose size and shape varies with latitude. Therefore the size and shape of the grid boxes will vary slightly over
the area covered by the radar and even more from radar to radar (35 to 45 km2 over the conterminous U.S. for the
1/4th LFM grid). The 1/4th LFM grid boxes used here are defined to have 1/4th LFM grid points as their centers
and a mesh length of 47.625 km (25.72 nm) at the standard latitude (60° N). The information required to generate
the grid are the latitude and longitude of the radar, the mesh length at 60° N latitude, and the standard longitude
(105° W).
In order to cover the radar umbrella out to 230 km (124 nm), even at the lower latitudes of the conterminous United States, a 13 x 13 array of 1/4th LFM grid boxes will be required. This array will always be 13 x 13 regardless of
the latitude of the site. This grid should be positioned in such a way that the radar site falls within grid box (7, 7).
This array must be compacted (e.g., elimination of all 0 rows, run-length encoding of rows) to reduce storage and
especially communications loadings. Compaction must be done in such a way that the source 13 x 13 array can be
reconstructed with the use of nominal computer resources.
3-55

October 2017

3.2.11.2.1 Operational Parameters.
• FLAG (Zero Hybrid) - A set or cleared flag indicating, if set, that no precipitation exists in the current scan.
• HYBRID SCAN - Reflectivity factor data on a 1° x 1 km (0.54 nm) polar grid from 1 to 230 km (124 nm), in
dBZe. These data were composited from four elevation scans by the PRECIPITATION PREPROCESSING
Algorithm. A precision of at least 1 dBZe is required.
• Average TIME (Scan) - The average scan time of the four elevation scans used to construct the HYBRID
SCAN. This is a time of occurrence, not duration.
• Maximum SPEED (Storm) - The climatologically derived maximum expected storm SPEED (90.0), in km/hr.
• THRESHOLD (Max Time Difference) - Maximum time between scans allowed by the time continuity test. A
precision of at least 0.01 hour is required.
• RANGE (Cut-Off) - The range beyond which a range effect correction must be applied in km.
• COEFFICIENTS (Range Effect) - Three coefficients used to specify the range effect correction function.
• BOX (1/4 LFM Grid) - Rectangular grid box which is 1/4th of the LFM grid. Consists of a file specifying the
RATE SCAN data sample volumes whose centers fall within each of the 1/4th LFM grid boxes.
• COEFFICIENT (Multiplicative Z-R) - Multiplicative coefficient in the Z-R conversion equation.
• COEFFICIENT (Z-R Power) - Power coefficient in the Z-R conversion equation.
• RATE (Zero Precipitation) - Precipitation rate assumed to be zero precipitation, in mm hr-1.
• PARAMETER (Time Continuity #1) - The allowable rate of change of the ratio of volumetric precipitation
rates when the echo area is equal to minimum AREA (Time Continuity), in hr-1.
• PARAMETER (Time Continuity #2) - The allowable rate of change of the ratio of volumetric precipitation
rates when the echo area is equal to the full radar umbrella out to the 230 km (124 nm) range, in hr-1.
• Minimum AREA (Time Continuity) - Minimum precipitation area to allow time continuity tests on volumetric
precipitation rates, in km2.
• Maximum RATE (Echo Area Change) - Maximum rate of change of echo area allowed to pass the time continuity test when the volumetric precipitation rate cannot be tested due to the minimum AREA (Time Continuity), in km2/hr.
• PRECIPITATION STATUS MESSAGE - An alphanumeric message which includes the radar ID, TIME
(Stamp), current radar status, current operational mode, current scan strategy, TIME (Last Precipitation Detected), CATEGORY (Precipitation), number of gages in data base, and time since last update to the gage data
base.
• CATEGORY (Precipitation) - The precipitation category currently in effect. The three possible categories (displayed as “Precip Status”) are Accumulating (ACCUM), Not Accumulating (NO ACCUM), and None. “Not accumulating” refers to detected precipitation that is below thresholds set for significant precipitation (i.e., RAIN
DETECTION DBZ THRESHOLD and RAIN DETECTION AREA THRESHOLD).
3.2.11.2.2 Strengths/Applications.
See Section 3.2.11.5.2.
3.2.11.2.3 Limitations.
See Section 3.2.11.5.3.
3.2.11.3 Precipitation Accumulation.
The PRECIPITATION ACCUMULATION Algorithm uses the previous and current precipitation rate (RATE
SCAN) data sets output by the PRECIPITATION RATE Algorithm to estimate the accumulation (mm) during
all or parts of the scan-to-scan period. The period accumulation scan(s) generated during the current pass of this
algorithm plus those produced within the hourly period under consideration are then used to estimate an hourly
running total or a clock hour total accumulation scan. This hourly accumulation scan is input to the PRECIPITA3-56

FMH-11-Part C

TION ADJUSTMENT Algorithm. The PRECIPITATION ACCUMULATION Algorithm also checks the hourly
accumulations for suspect values and modifies these under certain conditions.
The technique used to estimate the accumulation during the scan-to-scan period depends upon the time between
scans. If the time between scans is not too large, a simple average precipitation rate is computed for the scan-toscan period. This average is computed for each of the 1° by 2 km (1.1 nm) sample volumes which make up the
scan. These averages are then multiplied by the time between scans to construct a period accumulation scan with
a 1° x 2 km (1.1 nm) resolution, which gives the estimated scan-to-scan accumulation. This scan is comprised of
41,400 1° x 2 km (1.1 nm) sample volumes.
If the time between scans is too large to use simple averaging (equivalent to linear interpolation), the precipitation
rates for each scan are used separately to compute accumulations for the beginning and ending parts of the scanto-scan period. The remainder of the scan-to-scan period, centered midway between the two scans, is flagged as a
missing period. In this case, two 1° x 2 km (1.1 nm) resolution period accumulation scans are constructed.
Next, the beginning and ending times for the hourly accumulation period are established. Normally, this period
extends backward from the current scan time to a time one hour earlier. However, if a clock hour was passed during
the scan-to-scan period, these times coincide with the beginning and end of the most recently completed clock
hour. The clock-hourly period coincides with that over which rain gage accumulations are tabulated and compared
to radar-rainfall estimates in the Advanced Weather Interactive Processing System (AWIPS) application that determines gage-radar Bias corrections for ingest back into the PRECIPITATION ADJUSTMENT Algorithm. Indeed,
the radar-rainfall estimates for this application are provided by the clock- hourly version of the DPA product.
Weighting each period accumulation scan by the fraction of it which falls in the hourly accumulation period, a 1°
x 2 km (1.1 nm) resolution hourly accumulation scan for the specified hourly period is constructed. However, if
too much of the specified hourly period is not covered by period accumulation scans (e.g., is missing), no hourly
accumulations are constructed and the processing stream continues with the PRECIPITATION ADJUSTMENT
Algorithm.
Finally, each hourly accumulation scan sample volume value is checked against a threshold to see if it is reasonable.
If it is greater than the threshold, (i.e., an outlier) and the values of all neighboring sample volumes are below the
threshold, an interpolated accumulation is computed. These changes are made in such a way that subsequent modifications to outliers are not affected by the changes to those previously identified.
This algorithm requires precipitation rate scans from both the previous and current outputs of the PRECIPITATION RATE Algorithm. Only scans flagged as good in the PRECIPITATION RATE Algorithm are used (i.e.,
previous scan means previous good scan). Zero rate scans (scans not actually generated, but assumed to be zero
everywhere) can be good scans. The algorithm is sufficiently flexible so that it can provide accumulation information for as much of the scan-to-scan period as possible even when the time between scans is larger than 5 minutes.
However, the error associated with the accumulation will grow rapidly as the time between scans increases. Therefore, in order to provide, to the maximum extent possible, an uninterrupted precipitation record, the method used
to save the previous precipitation rate scan must be safe, even from temporary system shutdowns and restarts.
Whenever the current precipitation rate scan is the first scan in a new clock hour, clock hour accumulations are
computed instead of running hourly accumulations (note: the only difference between clock hour and running
hourly accumulations are the starting and ending times for the one hour accumulation period). Since the time between scans may be large if a system problem occurs, the clock hour accumulation period could be set to begin up
to 2 hours before the current scan time. In addition, missing periods must be taken into account so that hourly integration criteria can be checked. Therefore, all scan-to-scan period accumulation scans for periods ending any time
after 1 hour prior to the previous scan time must be saved. The previous scan time is the last good scan collected
prior to the current scan. The method used to save these period accumulation scans must be safe, even from temporary system shutdowns and restarts.

3-57

October 2017

3.2.11.3.1 Operational Parameters.
• RATE SCAN - Precipitation rate data on a 1° x 2 km (1.1 nm) polar grid from 1 to 230 km (124 nm), in mm/
hr. A precision of at least 0.1 mm/hr and a dynamic range of at least 0 to 400 mm/hr are required (not generated for times when FLAG (Zero Rate) or when FLAG (Bad) are set).
• Average TIME (Scan) - The average scan time of the lower elevation scans used to construct the HYBRID
SCAN. This is a time of occurrence, not duration.
• FLAG (Zero Rate) - A set or cleared flag for each average TIME (Scan) indicating, if set, that all precipitation
rate values can be assumed to be zero.
• Maximum TIME (Interpolation) - The maximum period over which a period accumulation scan can be computed using two precipitation rate scans, in hours (approximately 0.5 hours).
• Minimum TIME (Period) - The minimum period of time during an hourly accumulation period for which
ACCUMULATION SCAN (Period) data are required in order to estimate the hourly accumulation in hours
(approximately 0.90 hours).
• ACCUMULATION SCAN (Period) - Interpolated or extrapolated period precipitation accumulation data on a
1° by 2 km (1.1 nm) polar grid from 1 to 230 km (124 nm), in mm.
• TIME (Last precipitation detected) - The time at which the Precipitation Detection Function last detected precipitation.
• PRECIPITATION STATUS MESSAGE - An alphanumeric message which includes the radar ID, TIME
(Stamp), current radar status, current operational mode, current scan strategy, TIME (Last Precipitation Detected), and current and previous CATEGORY (Precipitation).
• THRESHOLD (Hourly Outlier) - The maximum hourly rainfall amount allowed in an hourly accumulation scan
sample volume (400), in mm. A precision of at least 0.1 mm is required.
• Ending TIME (Gage accumulation) - The time, each hour, when hourly radar and gage accumulations are required by the PRECIPITATION ADJUSTMENT Algorithm. Note: this time will always coincide with a clockhour.
3.2.11.3.2 Strengths/Applications.
See Section 3.2.11.5.2.
3.2.11.3.3 Limitations.
See Section 3.2.11.5.3.
3.2.11.4 Precipitation Adjustment.
The PRECIPITATION ADJUSTMENT Algorithm provides the capability to apply a mean-field, multiplicative
correction to selected PPS accumulation products, based upon rain gage vs. radar comparison information contained in a “Bias Table” ingested periodically into the RPG from an external source (e.g., AWIPS).
Bias Tables are generated by the Weather Forecast Office (WFO) version of the AWIPS Multisensor Precipitation
Estimator (MPE) function, based upon comparison of hourly rain gage reports against unbiased, radar-generated precipitation estimates. The gage reports are ingested into AWIPS from various collection networks while the
precipitation estimates are provided by the DPA product received from the RPG. At co-located positions where
both the radar and gage estimates are non-zero, the two types of reports are assembled into a vector of “gage-radar
pairs”. From these pairs, analyses are performed over various time periods ranging from short (“instantaneous”) to
mid (multi-hourly; daily; weekly) to long (“climatological”) term in order to determine mean-field biases and auxiliary data. These data are assembled into rows in the Bias Table, arranged in ascending, temporal order (presently 10).
Each row contains: the Bias (correction); the memory time span (in hours) over which the analysis was performed;
the effective sample size (a weighted estimate of the number of gage-radar pairs used in the analysis); the weighted-average gage-rainfall estimate; and the weighted-average radar-rainfall estimate, for that time span.
The tables are shipped from the AWIPS MPE function to all the radars associated with that WFO as part of a Bias
Table Message, on an hourly basis. This is done at a specific time each hour, set via a cron that allows the gage-ra3-58

FMH-11-Part C

dar pairs for the past clock-hour to be processed - typically about 25 minutes past the top of the hour. Tables may
be sent more frequently, at operator discretion, if, for example, additional rain gage reports are received or suspect
reports have been purged by QC procedures.
When the Bias Table arrives at the RPG, it is stored in a linear buffer, from which it is ingested by the Precipitation
Adjustment Algorithm-task each time that task executes (i.e., every volume scan). If the table is recognized as new,
based on date/time-of-generation comparison, it replaces the previous version internally (which, upon RPG startup, is a default table of nominal values).
Then, the “Most Representative” Bias from among those determined over the various time spans of the table is
extracted. This is done in a straightforward procedure in which, sequencing in ascending, temporal order, the bias in
the first row whose Sample Size exceeds an adaptable parameter (Minimum No. Gage-Radar Pairs) is selected. The
adaptable parameter (NGRPS) is found in the Hydromet Adjustment Algorithm adaptation data menu of the RPG
Human Computer Interface (HCI); its default value is 10 and its range is 6 – 30. If the user wishes the selected bias
to trend more toward the short term - i.e., based on the present precipitation, NGRPS may be lowered; whereas if
the user wishes it to trend more toward the long term - i.e., seasonal or climatological, NGRPS may be raised. Note
that this parameter will also be dependent on the density of reporting rain gages under the radar umbrella, and the
user may wish to raise/lower the default in the presence of a dense/sparse network.
Once established, the Bias correction is either applied - or not applied - to selected PPS rainfall accumulation products, depending on whether or not the (adaptable) Bias Applied Flag is turned On or Off (default setting: False).
(Note, while the Bias Flag is set in the Precipitation Adjustment Adaptable Parameters Menu at the HCI, biases are
actually applied in the Precipitation Products task.) If the flag is Off, no biases are applied momentarily. If it is On,
the present Bias is applied as one mean-field, multiplicative correction to all (non-zero; non-missing) grid points of
the most recent period or hour of the OHP, THP, STP, USP, and DSP products.
Note that these products may not be completely unbiased if the Bias Flag is presently Off; nor is the present Bias
correction necessarily applied through the duration of a product if the flag is On. Rather, only the most recent
component portion of each product is affected by the present Bias value and flag setting. For THP and USP (whose
constituent hours span clock-hourly periods), the Bias that was in effect at the end of each clock hour is either
applied or not applied to that hour=s accumulation, depending on the momentary setting of the Bias Applied Flag.
For STP and DSP, the Bias in effect at each time is, likewise, either applied or not applied to each of the constituent
(volume scan-to-volume scan) periods that comprise the storm-total duration. Only the OHP product is “pure” in
the sense that the entire product will either be adjusted, or not adjusted, by the present Bias correction value.
It is thus recommended that, prior to the onset of precipitation, a determination (True/False) be made for the Bias
Flag and then maintained through the duration of the event, unless it becomes obvious that the original choice
was not the most prudent. Note that the Bias is most likely to fluctuate toward the beginning of an event (as it gets
“established”), particularly if a long time has passed since the previous event or if meteorological conditions have
changed significantly in the interim (e.g., from cool, stratiform rain to convective).
The only PPS accumulation product to which the mean-field Bias adjustment is never applied (regardless of the
Flag setting) is the DPA. This is because that product is used, in its unadjusted form, in the AWIPS MPE application to determine the next Bias Table, by comparison to hourly rain gage reports. However, the selected Bias correction is contained in an appended alphanumeric layer of that product, along with the associated fields Sample Size,
Memory Span, and Bias Applied Flag. A complete copy of the present Bias Table, itself, is also contained in this
alphanumeric layer, along with the Date and Time the Bias was last updated locally.
The PPS reflectivity products (i.e., Hybrid Scan Reflectivity (HSR) and (DHR)) also never have a bias correction applied. However, similar to DPA, DHR contains the present Bias and its associated fields (as above) in an appended
alphanumeric layer (though not the complete Bias Table).
Alphanumeric information about the Bias is also available in the paired alphanumeric products to OHP, THP and
STP, which show the selected Bias and the associated fields (Effective) Sample Size and Memory Span (hours), as
3-59

October 2017

well as the setting of the Bias Applied Flag. The THP paired alpha product depicts these fields in tabular form, for
each of the three hours comprising the product; while the OHP and STP paired alpha products show the most
recent values of each USP does not have a paired alphanumeric product but, rather, a table across the top of the
graphic product that depicts the Bias for each hour comprising the (variable-length) USP. That table contains up to
three pages (eight hours per page) to depict the biases for up to the 24-hour maximum duration of the product.
Finally, the all-alphanumeric Supplemental Precipitation Data (SPD) product contains the Bias, its related fields (as
above), and the Date and Time of its last update on its first page; and a listing of the complete Bias Table on its
second page.

3.2.11.4.1 Operational parameters.
The following is received from an external source (AWIPS) once per hour, automatically (or more frequently, upon
manual intervention):
• Bias Table Message: a message containing a Header and a Bias Table, as follows:
• Header, consisting of the following:
▪ AWIPS Site (origination) ID: 3-char (e.g., OUN)
▪ NEXRAD Radar (destination) ID: 3-char (e.g., TLX)
▪ Observation Date/Time: {yr;mo;da;hr;mn;sc} of gage & radar observations (end of clock hour)
▪ Generation Date/Time: {yr;mo;da;hr;mn;sc} of message generation
▪ No. Rows in ensuing table (2-12; default 10)
▪ Bias Table: A table of gage and radar information analyzed over a number of aggregate time spans (corresponding to No. Rows) ranging from short (“instantaneous”) to mid to long (“climatological”) term, consisting of the following fields: (*note: all fields scaled by 1000):
▪ Memory Span of analysis (range: 0.001 – 1x107) (*actually presented as log before being scaled by 1000)
▪ (Effective) No. of Gage-Radar Pairs (range: 0 – 1x105) (*technically, Sigma Inverse of exponential analysis)
▪ Average Gage Estimate (range: 0.00 – 254.00 mm): (Exponentially normalized)
▪ Average Radar Estimate (range: 0.00 – 254.00 mm): (Exponentially normalized)
▪ Mean-field Bias Correction (range: 0.01 – 100.00): (Gage/Radar Ratio).
The following fields are received from predecessor task PRECIPITATION ACCUMULATION:
• AVERAGE DATE/TIME (Scan): The average date/time of the elevation scans used to construct the present
Hybrid Scan. Serves as the current Time Stamp and the ending time of the most recent accumulation period
{modified Julian day; secs within day}.
• ACCUMULATION SCAN (Period): Precipitation accumulation data for most recent scan-to-scan period, on 1°
x 2 km (1.1 nm) polar grid from 1 to 230 km (124 nm); units 0.1 mm.
• ACCUMULATION SCAN (Hourly): Precipitation accumulation data for most recent hour, on 1° x 2 km (1.1
nm) polar grid from 1 to 230 km (124 nm); units 0.1 mm.
• MAX VAL (Hourly): Maximum Hourly Accumulation value; units 0.1 mm.
• SCAN TYPE (Hourly): Indicator of whether Hourly Scan ends at present time (i.e., Average Date/Time (Scan),
if zero) or at top of most recent clock-hour (if non-zero).
• BEGINNING DATE/TIME (Hourly Accumulation): Beginning date/time of hourly accumulation period
{modified Julian day; secs within day}.
• ENDING DATE/TIME (Hourly Accumulation): Ending date/time of hourly accumulation period {modified
Julian day; secs within day}.
• FLAG (Zero Scan-to-Scan): Flag indicating, if set, that all locations in the current scan-to-scan period can be
assumed of zero accumulation (in which case, Accumulation Scan (Period) not ingested nor generated).
• FLAG (Zero Hourly): Flag indicating, if set, that all locations in the current hour can be assumed of zero accu3-60

FMH-11-Part C

•
•
•
•
•
•

mulation (in which case, Accumulation Scan (Hour) not ingested nor generated).
FLAG (No Hourly): Flag indicating, if set, that no hourly accumulations could be generated, due to missing
data (in which case, Accumulation Scan (Hour) not ingested nor generated).
The following fields are received from the RPG HCI adaptation data menu:
Reset Bias: Value to which Bias is set upon initialization or if excessive time passes since new Bias Table received (range: 0.5 to 2.0; default: 1.0).
Longest Lag: Longest time lag since last Bias Table received that still allows “Best Bias” to be extracted from
that table (range: 100 to 1000 hrs; default: 168 hrs [i.e., one week]).
Threshold # G-R Pairs: Threshold # of gage-radar pairs in a Bias Table row that must be exceeded for the
“Best Bias” to be selected from that row (range: 6 to 30; default: 10).
Bias Flag: True/False flag indicating whether the selected Bias will be applied to the PPS accumulation products (default: False).

The following fields are output by this task:
• BIAS (Current): “Best” G-R Bias value most recently selected from the Bias Table (range: 0.01 to 100.00).
• MEMSPAN (Current): Memory Span associated with the Bias most recently selected from Bias Table (i.e., from
the same row) (range: 0.001 to 1x107).
• G-R PAIR SIZE (Current): Effective Sample Size associated with the Bias most recently selected from Bias
Table (i.e., from the same row) (range: 0 to 1x105).
• DATE/TIME (Bias Calculation): Date & Time BIAS and associated fields most recently selected from Bias
Table {modified Julian day; secs within day}.

3.2.11.4.2 Operational Considerations.
If the Bias Table (expected at least once per hour) is late arriving, an adjustment is performed upon the most
recently-received table (stored internally) whereby the “effective no. of gage-radar pairs” fields in all table rows are
degraded in accordance with an exponential decay factor. That factor is determined as the exponential inverse of
the ratio of the lag (in hours) since the last Bias Table was received to the Memory Span (also in hours) over which
the analysis was performed for each table row. The effect of this procedure is to make it more likely that the “Most
Representative” Bias will be extracted from a table row based upon a longer (more climatological) Memory Span.
If the Bias Table is so late arriving that the time lag exceeds another adaptable parameter - Longest Lag, the Bias
reverts to a default value called the Reset Bias (also adaptable). The routine value of LGLAG is 168 hours (i.e., one
week), with a range of 100 to 1000 hours, while RESBI has a default of 1.0 with a range of 0.5 to 2.0.
3.2.11.5 Precipitation Products.
The PRECIPITATION PRODUCTS creates Hydro- meteorological products from hourly and scan-to-scan accumulations generated by the PRECIPITATION ACCUMULATION Algorithm and adjusted by the current BIAS
computed by the PRECIPITATION ADJUSTMENT Algorithm if FLAG (apply BIAS) is set. Digital, graphical,
and alphanumeric products are generated. The digital product is an hourly running total or clock hour accumulation
mapped to a 1/40th LFM rectangular (approximately 4 km x 4 km (2.2nm x 2.2 nm)) grid. The graphical products
are: (1) an hourly running total or clock hour accumulation, (2) a three hour total accumulation generated on the
clock hour, and (3) a storm total accumulation. The graphical products are all displayed at a resolution of 2 km (1.1
nm) x 1°. The alphanumeric SPD product is displayed in ASCII format. The graphical and alphanumeric products
are designed primarily for display systems while the digital product is designed for use on external computer systems. Other products pertaining to PPS, including DHR (Figures 2-19b and 3-21), DSP (Figure 2-36b), and USP
(Figure 2-37), are described in additional documents, including the Interface Control Document (ICD) for RPG/To
Class 1 User and the ICD for Product Specification.
The hourly running totals or clock hour totals on the 1/40th LFM grid are obtained by determining the mean of
all adjusted ACCUMULATION SCAN (Hourly) sample volumes whose polar coordinate centers fall within each
3-61

October 2017

1/40th LFM grid box. At the far ranges where no sample volume centers fall inside a box, the sample volume value
at the sample volume whose center is closest to the center of the grid box becomes the value at the grid box. Annotations are automatically added to identify the product and to provide information related to how the data used to
generate this product were processed.
The hourly running totals or clock hour totals on the 2 km (1.1 nm) x 1o grid are scaled to 16 levels for use as a
display and annotations are added automatically to produce the PRODUCT (OHP, Figure 3-22).

3-62

FMH-11-Part C


> **Figure 3-22: One-Hour Rainfall Accumulation Product.**

> An example One-Hour Rainfall Accumulation product (OPUP display) from the Saint Louis, MO

An example One-Hour Rainfall Accumulation product (OPUP display) from the Saint Louis, MO
(KLSX) WSR-88D at 11:56 UTC on 18 April 2013.
The three clock hour totals are computed hourly by summing the available individual clock hour totals for the past
three hours. At least two of the three hours of data must be available and missing periods should be noted. The
data are then scaled to 16 accumulation levels for use as a display, and annotations are added automatically to produce the PRODUCT (THP).
The storm total (total precipitation since the last one hour break in significant precipitation) is generated whenever
certain scan-to-scan accumulation parameters are exceeded. It is then updated using each ACCUMULATION SCAN
(Scan-to-Scan) received until being reset after a one-hour break in significant precipitation. If FLAG (apply BIAS) is
3-63

October 2017

set, the ACCUMULATION SCAN (Scan-to-Scan) is adjusted using the computed BIAS. The data are then scaled to
16 levels for use as a display and annotations are added automatically to produce the PRODUCT (STP).
The ARRAY PRODUCT (Digital Precipitation) on the 1/40th LFM grid provides hourly running total or clock
hour total precipitation accumulation estimates in a digital array format to support hydrometeorological requirements for numerical use of precipitation data in computers external to the RPG. In addition to the precipitation
array data, an extensive set of annotations (IDENTIFIER INFORMATION and SUPPLEMENTAL DATA) will
be included automatically as part of this product. This information is intended for use in higher level (regional/national) processing to identify certain characteristics about the data up to that point in the processing stream. It will
be used as part of the information for accomplishing more discriminating quality control functions at the higher
level of processing.
In order to cover the radar umbrella out to 230 km (124 nm) even at the lower latitudes of the conterminous United
States, a 131 by 131 array of 1/40th LFM grid boxes will be required. This array will always be 131 by 131 regardless of the latitude of the site. This grid should be positioned in such a way that the radar site falls within the grid
box (66, 66). The ARRAY PRODUCT (Digital Precipitation) must be compacted (e.g., elimination of all 0 rows, run
length encoding of rows) to reduce storage and especially communications loadings. Compaction must be done in
such a way that the source 131 by 131 array can be reconstructed with the use of nominal computer resources. The
1/4th LFM area averaged precipitation rate data (8 coded precipitation rate levels) for each scan used to generate
the ARRAY PRODUCT (Digital Precipitation) will be automatically included as part of the annotations (SUPPLEMENTAL DATA) to the ARRAY PRODUCT (Digital Precipitation). The values for the 13 by 13 1/4th LFM grid
were computed by the PRECIPITATION RATE Algorithm. These must be compacted subject to the constraints
specified above.
The PRODUCT (THP) uses the PRECIPITATION TOTALs (Hourly) for the last three clock hours. In order to
provide these products on a consistent basis, the method used to save the PRECIPITATION TOTALs (Hourly)
must be safe, even from temporary system shutdowns and restarts.
The PRODUCT (STP) uses the previous set of PRECIPITATION TOTAL (Storm). Again, the method used to
save these data must be safe, even from temporary shutdowns and restarts.

3.2.11.5.1 Operational Parameters.
• FLAG (apply BIAS): A set or cleared flag indicating whether the bias should be applied.
• Current BIAS: The current BIAS generated by the PRECIPITATION ADJUSTMENT Algorithm.
• ACCUMULATION Scan (Hourly): The hourly radar precipitation accumulation SCAN (Hourly) data for an
hourly running period or clock hour on a 1° x 2 km (1.1 nm) polar grid from 1 to 230 km (124 nm). A precision
of at least 1 mm and a dynamic range of at least 0 to 1600 mm are required. Includes the beginning TIME (Accumulation) and ending TIME (Accumulation).
• ACCUMULATION SCAN (Scan-to-Scan): The total scan-to-scan accumulation data on a 1° x 2 km (1.1 nm)
polar grid from 1 to 230 km (124 nm) for the period from the previous time to the current time. A precision of
at least 0.1 mm and a dynamic range of at least 0 to 400 mm are required. Includes the previous average TIME
(Scan) and current average TIME (Scan).
• CATEGORY (Precipitation): The precipitation category currently in effect, i.e., Accumulating, Not Accumulating, or None (described in Section 3.2.12.2.1).
• TIME (Stamp): The time at which the Precipitation Detection support function was last executed. A precision
of at least 1/1200 hour is required.
• PRECIPITATION STATUS MESSAGE: An alphanumeric message which includes the radar.
• ID, TIME (Stamp), current radar status, current operational mode, current scan strategy, TIME (Last Precipitation Detected), CATEGORY (Precipitation), number of gages in data base, and time since last update to the
gage data base.
3-64

FMH-11-Part C

•
•
•
•

•
•
•
•
•
•
•

TIME (Last Precipitation Detected): The time at which the Precipitation Detection Function last detected precipitation. A precision of at least 1/1200 hour is required.
FLAG (Zero Scan-to-Scan): A set or cleared flag indicating, if set, that all current ACCUMULATION SCAN
(scan-to-Scan) values can be assumed to be equal to ACCUMULATION (Zero Interpolated).
FLAG (Zero Hourly Accumulation): A set or cleared flag indicating, if set, that all current ACCUMULATION
SCAN (Hourly) values can be assumed to be zero.
BOX (1/40th LFM Grid): Rectangular grid box centered on 1/40th LFM grid points. At 60° N the mesh length
is 4.7625 km (2.57 nm). Specifies the scan’s sample volumes whose centers fall within each grid box. If none,
the sample volume whose center is closest to the center of the grid box is specified. Grid boxes whose centers
are more than 230 km (124 nm) from the radar are not assigned any sample volumes.
RATE (1/4th LFM Grid Box): Area-average rate (8 level coded value) in each ¼ LFM grid square. A 13 by 13
grid of values for each RATE SCAN used in constructing the hourly accumulations.
FLAG (No Hourly Accumulation): A set or cleared flag indicating, if set, that no hourly accumulations were
computed for the hour ending at the current ending TIME (Accumulation).
SUPPLEMENTAL DATA: A set of varied data, determined during the execution of the precipitation processing series algorithms, which is included as part of an alphanumeric product. Elements of the data will also be
included as annotations to the other precipitation products.
IDENTIFIER INFORMATION: Consists of annotations such as the radar I.D., product name, time (beginning and ending), date, and missing period times. The times must be in hours and minutes UTC.
GAGE REPORTs (Accumulator): Reported values of accumulation in mm at each gage and time of occurrence
(to the nearest 1/60 hour).
GAGE REPORTs (Incremental): Reported values of incremental accumulation in mm, increment duration
(hours to the nearest 1/60 hour) and time of occurrence (to the nearest 1/60 hour).
GAGE-RADAR SET: Set of associated pairs of hourly radar and hourly rain gage accumulations.

3.2.11.5.2 Strengths/Applications.
• The PPS Algorithm creates hydrometeorological products from hourly and scan-to- scan accumulations generated by the algorithm and adjusted by the current BIAS. Digital, graphical, and alphanumeric products are
generated.
• Significant quality controls are designed to produce better products by:
▪ Minimizing overestimation due to ground return caused by anomalous propagation,
▪ Eliminating reflectivity outliers and spurious noise, and
▪ Reducing the effects of beam blockage.
3.2.11.5.3 Limitations.
The PPS Algorithm does not provide sufficient information to distill and integrate heavy precipitation information
into a flash flood alert map.
• Does not account for:
▪ Below beam effects (wind, evaporation, coalescence)
▪ Non-uniform Z-R relationships within the radar coverage area.
• Does not always account for:
▪ Bright band contamination
▪ Hail contamination
▪ Inaccuracies due to radar outages
▪ Inaccuracies due to Z-R limitations.

3-65

October 2017


#### 3.2.12 Snow Accumulation Algorithm.

The purpose of the Snow Accumulation Algorithm (SAA) is to provide estimates of accumulated snow water
equivalent (S) and snow fall (SD). The algorithm was developed for use during dry snow, or snow that is not melting
while falling or on the ground. Therefore, the adaptable parameters have been optimized for dry snow. Like the PPS
for rain, SAA uses a Z-S relationship to convert reflectivity to a rate of S. Rates of S are also converted to rates of
SD using a user adaptable snow water ratio. SAA converts the rates to accumulations of S and SD for a storm total
accumulation and over different accumulation periods (such as one-hour and three-hour).
For some SAA adaptable parameters, optimum settings were derived for different regions of the CONUS using
data from one representative site within that region.
Ideally, input reflectivity data for the SAA should come from as close to the ground as possible without ground
(clutter) contamination. In this version of SAA, the algorithm uses data from the EPRE. The EPRE uses beam
blockage information from the Blockage Algorithm and AP/Clutter likelihood from the REC Algorithm to construct a HYBRID SCAN array of uncontaminated, unblocked reflectivity data from the volume of base radar reflectivity data. If the beam blockage or likelihood of AP/Clutter exceeds adaptable thresholds, reflectivity information from a higher elevation is used. The EPRE also adjusts (or adds) power for partial beam blockage. In addition,
EPRE stores the elevation angle from which the reflectivity data are obtained for each HYBRID SCAN bin and
outputs the elevation angle information in an HYBRID SCAN Elevation Angle array for use by the SAA and other
algorithms.
Whenever a HYBRID SCAN of reflectivity data are available from the EPRE, SAA processes the data (see Figure
3-23 for an overview of the SAA). During periods of no observed snow at the surface, the algorithm accumulates
bogus snow totals returns that are only aloft. SAA relies on the user to reset accumulations when snow has started
to reach the ground.
Before converting reflectivities to rates, SAA filters incoming reflectivity data to further mitigate contamination
from isolated sample volumes that are likely not dry snow (such as residual clutter). Reflectivities below an adaptable
minimum reflectivity threshold are not considered snow and are ignored. Algorithm developers recognized that
sometimes light snow can result in reflectivities below the minimum reflectivity threshold, but accumulation rates
observed with below threshold reflectivities are extremely light. A sample volume with reflectivities above an adaptable maximum reflectivity threshold are either set 1) to no data if the sample volume is isolated or 2) to the reflectivity corresponding to the average rate of its neighbors if the sample volume is not isolated.
Next, reflectivities from the hybrid scan are converted to rates of S using a Z-S relationship in the form of Z = αSβ,
where α and β are the adaptable Z-S multiplicative and power coefficients, respectively.
Once rates of S are computed, SAA applies a simple range / height correction. Above a minimum threshold height,
S is multiplied by a range / height correction factor to mitigate underestimation caused when the radar beam begins
to overshoot the precipitation. The range / height correction increases with height and is computed from a second
order polynomial. The development version of SAA used range (vs. height) as the dependent variable.

3-66

FMH-11-Part C


> **Figure 3-23: Snow Accumulation Algorithm Overview.**


3-67

October 2017


> **Figure 3-24: Storm-Total Snow Depth Accumulation Product.**

> An example Snow Total Snow Depth Accumulation product (OPUP display).

An example Snow Total Snow Depth Accumulation product (OPUP display).
The SAA computes SD values by multiplying S by an adaptable snow ratio. To compute accumulations, each volume
scan S and SD rates are multiplied by the time span between volume scans producing a scan to scan accumulation
for both S and SD. For the first volume scan after an accumulation reset, the time span is from the beginning to the
end of the hybrid scan. Otherwise, the time span is from the end of the previous hybrid scan to the end of the current hybrid scan. As time progresses, S and SD scan to scan accumulations are accrued to the S and SD storm total
accumulations, respectively. For different accumulation periods (such as storm total), the scan-to- scan accumulations over the accumulation period are summed (see Figure 3-24 for a product example). For an hour-long accumulation period, scan-to-scan accumulations must exist for an adaptable minimum time threshold. Therefore, after an
accumulation reset, accumulations will not be available for a specific accumulation period until the minimum time
threshold has passed for that accumulation period. For all accumulations no extrapolation is done over missing time
periods.
3-68

FMH-11-Part C

The SAA will be performed in real-time after the EPRE has run. The EPRE is typically run near the middle of the
volume scan, but could be run as late as after the second-to-last elevation scan in the volume scan.
3.2.12.1 Operational Parameters.
• ACCUMULATION PERIODS: Time periods over which accumulations are to be computed, such as 1-hr and
3-hr, in hrs.
• ACCUMULATION RESET FLAG: Flag indicating that all accumulations should be reset to zero; units: none,
range of values: True or False, precision: N/A.
• BASE ELEVATION: Elevation angle of the lowest elevation scan in the volume scan; units: degrees, range of
values: 0.1 to 2.0, precision: at least 0.1 degrees.
• DATE (END HYBRID SCAN): Date of the last radial used in the hybrid scan, in Julian Days since 01 January
1970.
• DATE (START HYBRID SCAN): Date of the first radial used in the hybrid scan, in Julian Days since 01 January 1970.
• HYBRID SCAN: Preprocessed reflectivity data mapped to a polar grid from 0 (due north) to 359 in whole degrees in azimuth and 0 to 229 km in range. Resolution of the grid is 1 degree in azimuth and 1 km in range. In
addition, for each grid point, the hybrid scan must retain its elevation angle; units: dBZe, range of values: -32 to
95, precision: 0.5.
• HYBRID SCAN Elevation Angles: For each bin in the HYBRID SCAN array, the elevation angle of the base
reflectivity data used, units: degrees, range of values: 0.0 to 20.0, precision: at least 0.1.
• MAXIMUM REFLECTIVITY THRESHOLD: Maximum reflectivity to convert to S, default value: 40.0, units:
dBZe, range of values: 30.0 to 55.0, precision: 0.5.
• MINIMUM HEIGHT CORRECTION THRESHOLD: Minimum height (ARL) to apply the range/height
correction, default value, units: km, range of values: 0.01 to 20.00, precision: 0.01.
• MINIMUM REFLECTIVITY THRESHOLD: Minimum reflectivity to convert to S,default value: 5.0, units:
dBZe, range of values: -10.0 to 25.0, precision: 0.5.
• MINIMUM TIME THRESHOLD (Period): The minimum length of time for which scan to scan accumulations are required to estimate accumulations over a one-hour period of time, default value: 54, units: minutes,
range of values: 0 to 60, precision: 1.
• RANGE / HEIGHT CORRECTION COEFFICIENT #1: Coefficient #1 in the equation used to range/
height correct estimates, default value, units: dimensionless, range of values: -5.0 to 5.0, precision: 0.0001.
• MINIMUM HEIGHT CORRECTION THRESHOLD: Minimum height (ARL) to apply the range/height
correction, default value, units: km, range of values: 0.01 to 20.00, precision: 0.01.
• RANGE / HEIGHT CORRECTION COEFFICIENT #2: Coefficient #2 in the equation used to range/
height correct estimates, default value, units: km-1, range of values: -0.5 to 0.5, precision: 0.0001.
• RANGE / HEIGHT CORRECTION COEFFICIENT #3: Coefficient #3 in the equation used to range/
height correct estimates, default value, units: km-2, range of values: - 0.001 to 0.001, precision: 0.0001.
• SNOW WATER RATIO: The ratio of SD to S, default value, units: in (snow depth) /in (snow water equivalent), range of values: 4.0 to 100.0, precision: 0.1 in/in.
• TIME (END HYBRID SCAN): Time of the last radial used in the hybrid scan, in fractional hours since midnight.
• TIME (START HYBRID SCAN): Time of the first radial used in the hybrid scan, in fractional hours since
midnight.
• TIME SPAN THRESHOLD: Maximum time allowed between the end of the previous hybrid scan and the
beginning of the current hybrid scan, default value: 11, units: minutes, range of values: 3 to 30, precision: 1.
• Z-S MULTIPLICATIVE COEFFICIENT: Multiplicative coefficient in the Z-S relationship; also referred to as
alpha, default value, units: N/A, range of values: 10 to 1000, precision: 1.
3-69

October 2017

•

Z-S POWER COEFFICIENT: Exponential (power) coefficient in the Z-S relationship; also referred to as beta,
default value, units: N/A, range of values: 1.00 to 3.00, precision: 0.01.

3.2.12.2 Strengths/Applications.
• This algorithm is designed to estimate both the snowfall accumulation and the equivalent liquid water accumulation.
• The SAA’s adaptable parameters were optimized for different WSR-88Ds around the CONUS. During development, the WSR-88Ds for SAA optimization were chosen so there would be at least one site within each geographic area which normally receives widespread snow storms.
• SAA’s adaptable parameters were optimized for best overall performance throughout a winter season.
• SAA’s Z-S relationships were primarily optimized from high quality hourly snow measurements. Measurement
locations were also verified using the Global Positioning System.
• The maximum reflectivity threshold is used to filter reflectivities which usually are not producing dry snow.
3.2.12.3 Limitations.
• The SAA tends to overestimate SD when compared to snowfall that was subject to melting below the radar
beam, including on the ground.
• With “wet snow” occurring with low-level temperatures of > ~ 30° F, the SAA will tend to overestimate snowfall.
• When comparing SAA estimates with snow measurements, low quality measurements should be used with
caution. At WSR-88Ds where the adaptable parameters were not optimized, it is likely SAA performance will be
diminished by an unknown amount, especially for situations where there are large geographical differences between the study WSR-88Ds and other WSR-88Ds. For example, large difference in radar heights will likely result
in different range / height corrections.
• During SAA development and in other work, a high degree of variability has been observed with snow - radar
measurements in shorter time intervals (such as Z-S relationships within an hour). Accordingly, SAA performance at small time intervals (such as 1-hour) is not as good as longer time intervals (such as storm totals).
• Since the snow ratio can vary greatly in time and space, SAA users should use caution in changing the snow
ratio during a snowstorm based on in situ measurements. Algorithm developers found hourly snow ratios varied
greatly and were a poor forecast of the next hour’s snow ratio.
• Due to the low-topped nature of snowstorms, the SAA is subject to underestimation that becomes substantial
beyond 70 km (at an elevation angle of 0.5° and increases with range). The simple range / height correction
scheme extends the useful range to at most 150 km when the hybrid scan is using the lowest elevation scan. For
areas where the lowest elevation scan is blocked at close range and for mountain top radars, the SAA’s effective
range will be less.
• SAA cannot discriminate between reflectivities representing dry snow reaching the ground and representing other targets. Therefore, the other targets will be included in SAA estimates resulting in overestimates or bogus estimates. Some common examples are virga, rain, birds, and chaff. In order to eliminate these bogus accumulations
and have realistic storm total accumulations, the user must reset SAA accumulations when snow starts reaching
the ground. However, a hybrid scan can contain reflectivities representing both snow reaching the ground and
representing other targets at the same time. For example, snow does not start reaching the ground at the same
time throughout the radar umbrella. After an accumulation reset, areas where snow has not yet reached the
ground may be having virga which SAA will add to accumulation estimates.
• SAA cannot discriminate between snow and other types of precipitation. A common example is when snow and
rain are falling under a radar umbrella at the same time but in different areas. If the areas of snow and rain did
not change location, it would be relatively easy to keep track of which area contained snow and have useful SAA
estimates. However, if the areas of rain and snow are changing, it would be unrealistic to keep track of the areas
containing snow, and SAA estimates should only be used with caution.

3-70

FMH-11-Part C

•
•
•

In rare cases, reflectivities greater than the maximum reflectivity threshold have been observed with heavy snow.
In these cases, SAA tended to underestimate snow.
SAA was developed to be used for dry snow, or snow that is not encountering temperatures of ~ 30°F or higher.
When and where a bright band occurs, SAA tends to overestimate.


### 3.3 Dual Polarization-Derived Algorithms.

3.3.1Melting Layer Detection Algorithm.
The Melting Layer Detection Algorithm (MLDA) uses unique signatures of correlation coefficient and differential
reflectivity within the melting layer to identify the heights of the top and bottom of the melting layer for each radial.
These heights are updated every volume scan and are used to produce a Melting Layer (ML) graphic product. The
ML is an overlay and is available every volume scan for every elevation angle based on MLDA from the prior volume scan (see Figure 3-25 for a sample image of the MLDA).
The algorithm ingests radial-based data from the elevations of 4° through 10° (in precipitation mode scans). Below the 4° elevation angle, the signature of the melting layer is smeared due to beam broadening with range; thus,
accurate detections are not possible. Above the 10° elevation angle, there is little data unless storms are very nearby
the radar. In addition to the radial-based data, the MLDA uses MLDA from previous volume scans. The number
of volume scans used depends on the Volume Coverage Pattern (VCP) mode, which may be either precipitation or
clear-air. Finally, a default 0 Celsius height is ingested which either comes from a user-defined height or the height
from the latest model run. For model data, either a radar coverage grid or a radar-location single point vertical profile is used.
The MLDA eliminates any bins in a radial that will not be used for identifying the melting layer top and bottom
heights for that radial. Bins identified as ground clutter, biologicals, no data, or unknown by the Hydrometeor Classification Algorithm (HCA) are excluded. Bins where the signal to noise ratio is less than 5 and any bins where the
slant range is above the maximum climatological height for a melting layer, which is set to 6 km (19kft) are excluded.
The MLDA then computes a weighted value based on the likeliness of that bin being within the melting layer. The
algorithm uses reflectivity, differential reflectivity, correlation coefficient, and the elevation angle to compute this
weighted value. The higher the weighted value, the more likely it is that the bin exhibits a melting layer signature.
The algorithm will look for reflectivity values between 30 and 47 dBZ, differential reflectivity values between 0.8
and 2.2 dB, and correlation coefficient values between 0.9 and 0.97. If these conditions pass, a non-zero value is
assigned to a given bin that is weighted according to its elevation angle. The higher the elevation angle, the higher
the weight because higher elevation angles are less susceptible to beam broadening effects.

3-71

October 2017


> **Figure 3-25 Melting Layer Product.**

> An example Melting Layer product overlaid on a Reflectivity Data Array product. The thin outer contour is where the bottom edge of the beam exits the top of the melting layer. The thick outer contour is

An example Melting Layer product overlaid on a Reflectivity Data Array product. The thin outer contour is where the bottom edge of the beam exits the top of the melting layer. The thick outer contour is
where the beam center point exits the melting layer. The thick inner contour is where the beam center
point enters the melting layer. The thin inner contour is where the top edge of the beam enters the
melting layer.
Once all the valid bins have been assigned a weighted value, these values are sorted according to their height and
radial into a height vs. radial array with a vertical resolution of 0.1 km and radial resolution of 1 degree. A portion
of an example array is shown in Figure 3-26.

3-72

FMH-11-Part C


> **Figure 3-26 Height vs. Radial Array.**

> An example of a height vs. radial array for the MLDA.

An example of a height vs. radial array for the MLDA.
The array is a combination of weighted values from the current volume scan and the previous 2 volume scans
(when using a precipitation VCP) and the previous 5 volume scans (when using a clear-air VCP). Next, the MLDA
computes the melting layer top and bottom heights for each radial from the weighted values if a threshold for the
weighted values is met. If the threshold is not met, the melting layer top and bottom are not computed from the
melting layer detections. The melting layer top and bottom will be generated by one of two other methods. First,
the algorithm will see if other radials in the current scan had enough melting layer detections to determine a melting
layer top and bottom, and use the average top and bottom heights of those radials. If no radials have a melting layer
top and bottom height determined from melting layer detections, then it will use the default 0 Celsius height defined
at the radar. The top will be defined as the 0 Celsius height and the bottom will be 500 meters below that.
3.3.1.1 Operational Parameters.
• Use MLDA Heights: Allows the MLDA to compute heights from the dual polarization data or to use only the
input from the radar environmental data (0° Celsius height).
3.3.1.2 Strengths.
• Overlays of MLDA output on elevation based radar products can help in the interpretation of base data.
• The solid outer contour (which is the radar beam center exiting the melting layer) has been found to be a useful
proxy for the wet bulb 0° Celsius height.
3.3.1.3 Limitations.
• Azimuthal computations can lead to discontinuities in the contours.
• The MLDA cannot account for multiple melting layers. The first, top-down encountered melting layer is depicted.
• Projection from the middle elevation angles to other tilts may not be representative.
3-73

October 2017


#### 3.3.2 Hydrometeor Classification Algorithm.

The Hydrometeor Classification Algorithm (HCA) uses dual polarization data to determine the dominant scatterer
type within that resolution volume. The output by the HCA is the Hydrometeor Classification product (HC) and is
produced for every elevation angle.
There are 12 classes defined in the HCA: biological scatterers (BI), ground clutter/anomalous propagation (GC),
ice crystals (IC), dry snow (DS), wet snow (WS), light/moderate rain (RA), heavy rain (HR), big drop (BD), graupel
(GR), hail possibly mixed with rain (HA), unknown (UK), and no data (ND).
The HCA assigns a hydrometeor type for each radar bin based on the dual polarization inputs. The inputs are based
on pre-defined thresholds for the various hydrometeor types, and the hydrometeor type with the highest likelihood
value will be assigned to that bin. The inputs are found in Figure 3-27.


> **Figure 3-27: Inputs to the HCA.**

> This is a listing of the inputs used to calculate HCA.

This is a listing of the inputs used to calculate HCA.
3.3.2.1 Operational Parameters. None.
3.3.2.2 Strengths.
• HCA enables a more accurate assessment of hail detection.
• HCA assists with improved discrimination between non-meteorological and meteorological targets as well as
discrimination between rain and snow.
3.3.2.3 Limitations.
• HCA has limited value where more than one classification may be valid, such as areas of mixed precipitation.
• The performance of HCA relies on well-tuned ZDR data and a properly identified melting layer.

#### 3.3.3 Quantitative Precipitation Estimation Algorithm.

The Quantitative Precipitation Estimation (QPE) algorithm applies the benefits of dual pol to radar precipitation
estimations. This algorithm selects which sample volumes to use at each horizontal location based on terrain-based
blockage data, correlation coefficient (corrected for noise), and hydrometeor classification. The goal is to use, for
each location, the lowest unblocked sample volume not contaminated by clutter. For each selected sample volume,
the algorithm computes precipitation rates based on the base data values and sample volume’s height (above radar
3-74

FMH-11-Part C

level) relative to the top of the melting layer. The top of the melting layer is determined by the MLDA. The final
precipitation rate for each sample volume is called the combined rainfall rate, since the rate is based on a combination of decisions. Finally, the algorithm determines whether it is precipitating at least a threshold rate over at least a
threshold area to initiate or terminate a storm total accumulation.
QPE uses the output from the HCA and the MLDA to select rain rates for each range bin. HCA and MLDA help
to prevent non-meteorological returns from being converted to rain rate and to determine the best rain rate computation for the particular bin.
The three different QPE rain rate calculation methods are each based on different inputs. The equations are given
in Figure 3-28. The choice of equation is dependent on the classification value and the height with respect to the
melting layer.


> **Figure 3-28 QPE Equations**

> Both QPE and PPS have one-hour rainfall products. The One Hour Accumulation (OHA) is the QPE version of

Both QPE and PPS have one-hour rainfall products. The One Hour Accumulation (OHA) is the QPE version of
the PPS One Hour Precipitation product (OHP). Similarly, both QPE and PPS have storm total rainfall products.
The Storm Total Accumulation product (STA) is the QPE version of the PPS Storm Total Precipitation product
(STP). Examples of QPE products can be found in Chapter 2.
3.3.3.1 Operational Parameters.
Several adaptable parameters are available, which impact the QPE estimates:
• Dry Snow multiplier
• Ice Crystals multiplier
• Wet Snow multiplier
• Graupel multiplier
• Rain/Hail multiplier
3.3.3.2 Strengths.
• Preventing returns from non-meteorological targets from conversion to rainfall is well supported by the dual pol
identification of ground clutter and biological returns.
• Three different rain rate equations that are applied bin-by-bin based on the classification value.
• Mitigation of bright band contamination.
• Mitigation of hail contamination.
3.3.3.3 Limitations.
• QPE performance is highly dependent on ZDR calibration.
• If an assigned classification value is invalid, that will increase the error of the rainfall estimate.
3-75

October 2017


### 3.4 Aviation Hazards Algorithms.


#### 3.4.1 Hail Hazard Layers.

The Hail Hazard Layers (HHL) Algorithm detects hail hazard areas within 162 n mi (300 km) of the radar. HHL
accomplishes this through analysis of the Hydrometeor Classification Algorithm’s (HCA) data for each elevation
scan of a NEXRAD radar volume. HCA data are in ¼ km range bins. HHL down-samples the occurrence of the
HCA rain/hail class to 1 km range bins. A 1 km range bin is populated with the rain/hail class if at least one of
the associated ¼ km HCA range bins also has the rain/hail class. For each elevation scan, each range bin implies an
associated altitude. An altitude top and an altitude bottom are computed for each range bin with a rain/hail classification. The top altitude is found from the top of the NEXRAD beam and the bottom is found from the bottom of
the NEXRAD beam. The top and bottom altitudes are updated with each elevation scan through completion of the
radar volume. This effectively provides the vertical extent of rain/hail detection by HCA for each column of associated 1 km range bins. Within 50 km range of the radar (generally the ground clutter zone), a speckle filter is used to
edit some of the spurious, isolated rain/hail class range bins.
The HHL output product grids are 1° x 1 km. The grid components are hail top altitude, severity, and confidence
and hail bottom altitude, severity, and confidence. The initial HHL does not compute severity or confidence. Severity will be based on dual pol hail sizing.
3.4.1.1 Operational Parameters.
None.
3.4.1.2 Strengths.
• Available out to 162 nm (300 km) from the radar.
• Aid in detecting conditions hazardous to aviation with vertical depiction of identified areas of hail.
• In absence of surface hail reports, indicates areas of hail formation or growth aloft.
• Presence of hail aloft might distinguish a storm cell as one with a greater severe potential than a storm cell without
hail aloft.
3.4.1.3 Limitations.
• Hail detection limited to performance of HCA rain/hail class.
• Hail that is present at altitudes beneath the lowest elevation scan angle will not be detected.

#### 3.4.2 Icing Hazard Levels.

The Icing Hazard Levels (IHL) Algorithm detects icing hazard areas within 162 n mi (300 km) of the radar. Referring to the flow diagram (Figure 3-29), IHL requires dual pol radar data and meteorological model data. The model
data are typically updated hourly on a 13 km grid. Vertical profiles from the model temperature and relative humidity fields are created from the collection of model grid points within the radar domain. The grid is ingested directly
by IHL and also the Melting Layer Detection Algorithm (MLDA). The dual pol radar data are updated typically
every 4 to 10 minutes. MLDA provides an altitude range for a single melting layer based on analysis of the dual pol
radar parameters with support from the model vertical profiles when necessary. The MLDA melting layer is used
within the HCA to determine hydrometeor classifications. IHL ingests the HCA classifications.

3-76

FMH-11-Part C


> **Figure 3-29 Flow Chart to IHL**

> IHL uses the graupel hydrometeor classification from HCA as an indicator of the presence of supercooled water

IHL uses the graupel hydrometeor classification from HCA as an indicator of the presence of supercooled water
and an icing hazard. Graupel develops as snow crystals and aggregates fall from above through a layer of supercooled water. The layer must be of sufficient depth to transform snow into graupel. Thus, graupel typically is an
indicator of the lower (or bottom) altitudes of the icing hazard. IHL uses vertical profiles of the thermodynamic
fields to provide an improved estimate of the upper (or top) altitudes of the icing hazard only above where graupel
was indicated. Figure 3-30 below shows an IHL vertical cross-section example of the graupel detections (dark blue)
augmented by the high model interest (cyan) to form the complete graupel-based IHL version.

3-77

October 2017


> **Figure 3-30: Vertical Cross-Section of IHL**

> An altitude top and an altitude bottom are computed. The top altitude is found from the top of the thresholded

An altitude top and an altitude bottom are computed. The top altitude is found from the top of the thresholded
high model interest. The bottom altitude is found from the bottom of the NEXRAD beam associated with the lowest elevation scan to contain graupel. This effectively provides the vertical extent of the icing hazard estimated from
the graupel HCA and augmented by model interest for each radar volume.
The IHL output product grids are 1° x 1 km. The grid components are icing hazard top altitude, severity, and
confidence and icing hazard bottom altitude, severity, and confidence. The initial IHL does not compute severity or
confidence. Additional, derived information from analysis of the gridded model data such as indications of multiple melting layers will be combined with dual pol parameters directly in future versions of IHL to expand the icing
hazard coverage beyond the graupel detection.
3.4.2.1 Operational Parameters. None.
3.4.2.2 Strengths.
• Available out to 162 nm (300 km) from the radar.
• Aid in detecting conditions hazardous to aviation with vertical depiction of identified areas of icing hazard
potential.
• Initial version (graupel class with model) corresponds well to locations of icing Pilot Reports (PIREPS) and is
useful as such for regions without PIREPS (due to planes not flying in areas during storms).
3-78

FMH-11-Part C

3.4.2.3 Limitations.
• Initial version (graupel class with model) will not report icing hazard conditions present that do not generate
graupel (such a pure freezing rain or freezing drizzle).
• Icing hazard detection limited to performance of HCA graupel class.
• Any icing hazard that is present at altitudes beneath the lowest elevation scan angle will not be detected.
Hallowell, R. G., M. F. Donovan, D. J. Smalley, B. J. Bennett, 2013: Icing Hazard Detection with NEXRAD IHL.
Proc. American Meteorological Society’s 36th International Radar Conference, Breckenridge.

#### 3.4.3 Machine Intelligent Gust Front Algorithm.

The Machine Intelligent Gust Front Algorithm (MIGFA) detects convergence boundaries and provides forecasted locations of the detected boundaries within 37.8 nm (70 km) of the radar. MIGFA accomplishes this through
analysis of the reflectivity and radial velocity data from the lower tilts of a NEXRAD radar volume. MIGFA processes these data from all operational radar volumes. In most cases, the 0.5° and 1.5° elevation angle tilts are used.
For VCPs 12 and 212, the 1.3° elevation angle tilt is used instead of the 1.5° elevation angle tilt with the 0.9° elevation angle tilt skipped. The 0.5° elevation angle tilt from SAILS scanning is not used in MIGFA. These elevation
angle tilts used are split cuts. So the data from each split cut elevation tilt is provided as input to MIGFA mapped
to the radial velocity azimuth angles. More so, for reflectivity MIGFA merges the two elevation angle tilts into one
data field. A linear weighting blends the reflectivity data to mitigate potential anomalous propagation contamination
within 30 km of the radar.
MIGFA converts the reflectivity and radial velocity fields into images for processing. From them, additional “interest images” are created. The interest images are created by analyzing the input data for many different characteristics
that are indicative of convergence boundaries. The information across any interest image essentially depicts the
certainty that a particular characteristic is present (ranging from highly likely to highly unlikely). For each volume,
all interest images are combined through a weighting process. The combined interest image is further analyzed in
the context of past combined interest images (up to 30 minutes prior) along with prior- volume anticipation images
to yield a pseudo-final feature image. From that, MIGFA reports all detected convergence boundaries meeting a
threshold requirement.
Each MIGFA detection is comprised of a chain of points. Each point’s distance from the radar in Cartesian coordinates (kilometers) is provided. Detections also have associated 10 minute and 20 minute forecasts. The forecasts are
similarly comprised of chains of points. Additional detection information provided is the u- and v-components of
the boundary propagation, the u- and v- components of the wind behind the boundary, and the wind speed differential across the boundary.
3.4.3.1 Operational Parameters.
None.
3.4.3.2 Strengths.
• Available out to 37.8 nm (70 km) from the radar.
• Boundaries at or approaching airport terminal locations may indicate a wind shear hazard.
• Outflow boundaries often are associated with strong thunderstorms.
• Differential boundary propagation is incorporated into forecasted positions.
• Boundary location may indicate an area of heightened convective initiation potential.
• Detected outflow boundaries might signify the potential for wind damage.
3.4.3.3 Limitations.
• The actual extent of a detected convergence boundary might extend beyond the maximum radius of the product.
• Actual convergence boundaries will not be detected if below the radar’s sensitivity.
3-79

October 2017

•
•
•
•

Not all MIGFA identified features are reported in the product. Only those that pass a convergence threshold are
reported.
Might not resolve convergence boundary feature with a relatively wide width radar signature.
Tendency to drop convergence boundary features as they become orthogonally aligned crossing over the radar.
Some detections are bird or bat roost activity especially at sunrise and sunset.

Smalley, D., J., B. J. Bennett, and R. Frankel, 2005: MIGFA: The Machine Intelligent Gust Front Algorithm for
NEXRAD. Preprints, 32nd Conf. on Radar Meteorology, Albuquerque, N.M., Amer. Meteor. Soc., paper 8.4.

#### 3.4.4 NEXRAD Turbulence Detection Algorithm.

No algorithm description available.

### 3.5 Removed Algorithms.


#### 3.5.1 Combined Shear.

This algorithm computes combined shear of the radial velocities at a single elevation within a volume scan. This
combined shear is related, but not equivalent to the total shear of the horizontal wind field.
On an azimuth-by-azimuth basis, the algorithm performs the following calculations:
• Computes the running averages of Doppler velocity over a specified number of sample volumes. Only those
data values with corresponding received power levels greater than the velocity power threshold are included in
the averages.
• Computes the running differences of the radially averaged Doppler velocities. These differences are taken over
a radial distance equal to the averaging distance. For final processing, the differences are assigned to Cartesian
grid points.
• Computes the azimuthal differences of the radially averaged Doppler velocities. These differences are taken
between adjacent azimuths at a constant range. These differences are divided by the distances between the effective sample volume centers, with the quotients then assigned to Cartesian grid points.
Once all azimuths have been processed, the following are performed sequentially within the rectangular Cartesian
grid:
• Computes the average radial and azimuthal differences at each grid point.
• Filters the difference fields using a centered, two-dimensional filter.
• Combines the shear of the radial velocities at each grid point by squaring the radial and azimuthal shears, adding, and taking the square root.
• Thresholds the combined shears, keeping only those values above the combined shear threshold.
This process yields a field of shear values that are displayed in the Combined Shear (CS) product.
3.5.1.1 Operational Parameters.
• Combined Shear Threshold: 0.0 to 5.0 x 10-3 s-l; default, 2.0 x 10-3 s-l: The minimum combined shear value allowed for acceptance in the final shear field.
• Domain Resolution: 0.5 to 4.0 km (0.27 to 2.2 nm); default, l km (0.54 nm): The spatial resolution of shear data
after mapping onto the Cartesian grid; the effective resolution of the products.
• Domain X Minimum: minus 116 to 0 km (-63 to 0 nm); default, minus 116 km (-63 nm): The lower left X (W-E)
coordinate relative to the radar for the rectangular Cartesian grid for the interpolated shears.
• Domain X Size: 0 to 232 km (0 to 125 nm); default, 232 km (125 nm): The length of west-east side of Cartesian
grid box (the radar is on the intersection of the grid boxes).
• Domain Y Minimum: minus 116 to 0 km (-63 to 0 nm); default, minus 116 km (-63 nm): The lower left Y (N-S)
coordinate relative to the radar for the rectangular Cartesian grid for the interpolated shears.
3-80

FMH-11-Part C

•
•
•
•
•
•
•

Domain Y Size: 0 to 232 km (0 to 125 nm); default, 232 km (125 nm): The length of north-south side of the
Cartesian grid box (the radar is at the intersection of the grid boxes).
Maximum Number of Radial Samples: 650 to 660; default, 660: The maximum number of samples in one radial.
Number of Filter Points: 1 to 25; default, 9: The number of data points used in the uniform filter applied to the
mean shear (azimuthal) and mean shear (radial) fields.
Number Threshold: 0.25 to 0.75; default, 0.75: The minimum fraction of radial and azimuthal differences.
Radial Shear Flag Value: minus 999.9 to -1.0; default, -999.9: The default value for filtered radial and azimuthal
and combined shear.
Sample Volume Number: 1 to 5; default, 3: The number of contiguous sample volumes to be averaged to produce each estimate of average radial velocity.
Velocity Power Threshold: 0 to 10 dB; default, 5 dB: The received power above which velocities will be processed.

3.5.1.2 Strengths/Applications.
Radial shear and azimuthal shear are combined into a single field of values. In this way separate products are not
required.
3.5.1.3 Limitations.
• Extensive filtering is done in order to reduce the noisiness of the shear data. Radial and azimuthal shears are
combined in order to get a shear value of some magnitude regardless of the viewing aspect of the radar. The
extensive filtering will reduce small- scale shears.
• Comprehensive research has not been done to investigate the capabilities and weaknesses with the algorithm
and product.

#### 3.5.2 Mesocyclone Algorithm.

The Mesocyclone Algorithm (M) uses a pattern recognition technique to detect mesocyclones. This technique
defines a process used for searching through Doppler velocity data for symmetric regions of large azimuthal shear.
The Mesocyclone Detection Algorithm is based on the extraction of significant attributes that characterize mesocyclones. The first step is to search for a consistent increase of Doppler velocity in the azimuthal direction at a
constant range with clockwise antenna rotation. (A consistent decrease of Doppler velocity is required for counterclockwise antenna rotation.) A “pattern vector” is formed when a series of azimuthally adjacent sample volumes
of increasing or decreasing Doppler velocity ends. A pattern vector contains seven components: the slant range,
the azimuth angles at both ends of the series, the Doppler velocities that correspond to those azimuth angles at the
slant range, and the tangential shear and angular momentum. A pattern vector that does not have the magnitudes
of angular momentum and azimuthal shear typical of mesocyclones is discarded. The remaining pattern vectors are
consolidated to form “features.” A “feature” is a set of pattern vectors in close proximity. If a feature is too small
it is discarded. If a feature is sufficiently large, but not symmetrical, it is classified as a shear region. Sufficiently
large, symmetrical shear regions are characteristic of mesocyclones. If these regions are in close vertical proximity, a
mesocyclone is identified. Lesser shear regions in close vertical proximity identify 3-dimensional shear regions. The
remaining features characterize uncorrelated shear.
3.5.2.1 Operational Parameters.
• AZIMUTH: Azimuthal position, in radians.
• ELEVATION: Elevation angle, in radians.
• RADIUS (Earth): The radius of the Earth (6371), in km.
• RANGE (Slant): The slant range to the center of a SAMPLE VOLUME, in km.
• SAMPLE VOLUME: A data sample volume whose dimensions are 1 degree in azimuth, 0.25 km in range, and 1
degree in depth (perpendicular to the radar beam).
• THRESHOLD (Feature Height): A value that represents the maximum height of possible mesocyclone FEATUREs (8), in km.
3-81

October 2017

•
•
•
•
•
•
•
•
•
•
•
•
•
•

THRESHOLD (High Momentum): A value which represents the minimum magnitude of angular momentum
expected in a mesocyclone in the presence of low shear (540.0), in km2/hr.
THRESHOLD (Radial Distance): A value which represents the maximum distance in the radial direction between PATTERN VECTORs within the same FEATURE (0.75), in km.
THRESHOLD (Meso cyclone-High Shear): A value which represents the minimum magnitude of shear expected in a mesocyclone in the presence of low angular momentum (14.4), in hr-1.
THRESHOLD (Low Momentum): A value which represents the minimum magnitude of angular momentum in
a mesocyclone (180.0), in km2/hr.
THRESHOLD (Meso Shear): A value which represents the minimum cyclone Low magnitude of shear expected in a mesocyclone (7.2), in hr-1.
THRESHOLD (Meso Azimuth): A value that represents the maximum cyclone tangential separation of PATTERN VECTORs to be considered part of the same FEATURE (0.034), in radians.
THRESHOLD (Pattern Vector): A value which represents the minimum number of
PATTERN VECTORs required to build a FEATURE (10.0).
THRESHOLD (Far Ratio): A maximum value which represents the Maximum upper bound of a range of values related to the ratio of radial and azimuthal diameters of a FEATURE at ranges further than THRESHOLD
(Range) (4.0).
THRESHOLD (Far Minimum Ratio): A minimum value which represents the lower bound of a range of values
related to the ratio of radial and azimuthal diameters of a FEATURE at ranges further than THRESHOLD
(Range) (1.6).
THRESHOLD (Maximum Ratio): A maximum value which represents the upper bound of a range of values
related to the ratio of radial and azimuthal diameters of a FEATURE at ranges closer than THRESHOLD
(Range) (2.0).
THRESHOLD (Minimum Ratio): A minimum value which represents the lower bound of a range of values related
to the ratio of radial and azimuthal diameters of a FEATURE at ranges closer than THRESHOLD (Range) (0.5).
THRESHOLD (Range): A variable that represents the range at which long-range symmetry criteria take effect,
in km (140.0 km).
VELOCITY (Doppler): Doppler velocities in a SAMPLE VOLUME, in km/hr.

3.5.2.2 Strengths/Applications.
• The algorithm automatically processes 3-dimensional velocity data to identify regions that may contain operationally significant mesocyclones.
• A mid-level mesocyclone that lowers toward the surface may indicate a tornado is developing.
3.5.2.3 Limitations.
• The operator should use the algorithm as a safety net and manually examine reflectivity, velocity/SRM to verify
the existence of operationally significant mesocyclones.
• Algorithm does not consider time continuity; consequently, transient, operationally insignificant circulations are
identified.
• The radar horizon and cone-of-silence can prevent the radar from detecting circulations at times.
• Velocity signatures may be obscured or degraded where there are improperly dealiased velocity data or where
data are obscured by range-folded echoes.
• The algorithm only requires two vertically linked elevation angles. No storm depth criteria are applied. Operationally insignificant circulations are identified.
• The algorithm only detects cyclonic rotations, not anticyclonic rotations.
• Identification is influenced by aspect ratio.
• Don’t know which elevation angle to examine shear. - Attribute Table and mesocyclone Alphanumeric Product
only give height.
3-82

FMH-11-Part C

•
•
•

•
•

Range thresholds may discard or improperly classify mesocyclones. No data within 10 km (5.4 nm) is processed
by the Mesocyclone Algorithm.
The Mesocyclone Algorithm uses no reflectivity structures (BWER, Hook Echo, etc.) to identify tornadic circulations.
Shear thresholds are not continuously variable with range, rather they are step-wise which introduces a nonrealistic effect. Because of averaging across the beam that spreads with increasing range, real shears between
inbound and outbound velocity maxima do continuously decrease with range. More elaborate shear thresholds
would perform better. However, there is an inherent distance limitation for signature recognition, due to spreading of the beam versus signature size.
Algorithm default values adapted for classic supercells.
Various operational parameters need to be optimized for best performance.


#### 3.5.3 Severe Weather Probability.

Severe Weather Probability (SWP) Algorithm estimates the probability that a given cell will produce severe weather.
The algorithm defines cells from the VIL grid by associating with each VIL maximum (>10 x 106 kg km-2) a specified number of VIL grid values surrounding that maximum. The center of one cell may not be part of any other
cell, but the edges of two cells may overlap. For a given cell, the VIL values are used to calculate parameters related
to the size of the cell and to the area of the cell with VIL values exceeding certain thresholds. These parameters
are then used to solve an equation (developed by statistically relating archived VIL data to concurrent reports of
severe weather) to estimate the SWP for that cell. Both the number of VIL grid boxes used to constitute a cell and
the equation coefficients are site adaptable parameters. The default cell size is 28 x 28 km (15.1 x 15.1 nm) and the
default coefficients are those derived from Oklahoma data. This algorithm has not been updated since its introduction into the WSR-88D.
3.5.3.1 Operational Parameters.
• LIQUID WATER (Integrated): The integrated liquid water values (per grid box), for a column within a
STORM, in kg/km2.
• SEVERE WEATHER COEFFICIENTS*: The set of coefficients (SW1...SW6) of a regression equation that
determines the severe weather probability. The default values are SW1 = 6.90, SW2 = 7.39, SW3 = 0.22, SW4 =
3.67, SW5 = 0.01 and SW6 = 2.55.
• Box (4 km x 4 km (2.2 nm x 2.2 nm) Grid): Square grid boxes which are 4 km on a side and cover ranges from 0
to 230 km (124 nm).
• Box (SWP): Composed of a square array of boxes (4 km x 4 km (2.2 nm x 2.2 nm)).
• SWP BOX SIZE: The size of the SWP analysis area, in odd numbered multiples of 4 x 4 kilometer boxes (e.g.,
28 x 28, 36 x 36, 44 x 44...etc.) The default size is 44 x 44 km (23.8 nm x 23.8 nm).
* Note that these coefficients will vary substantially with the quality of the severe weather ground-truth information, size of the data set, the radar, and location; therefore, these are essentially examples.
3.5.3.2 Strengths/Applications. Locate the most significant storms.
3.5.3.3 Limitations.
• SWP is biased on VIL and ground truth records of severe weather occurrence only.
• An accurate estimate of severe weather probability at any location requires a large climatological radar data set
and accurate severe weather ground-truth data set for that radar site. The required climatology of VIL and severe weather occurrence data does not exist for any site at this time.
• As with any empirical, statistical technique, there is the possibility that the statistical sample does not reflect the
true population. How much of the variance in the true population is explained by this equation is not known; in
the dependent sample, 24% of the variance was explained.
• The lack of earth curvature correction described in the limitations section of the VIL Algorithm applies.
3-83

October 2017


> **Figure 3-31: VIL Values for a Single Storm as a Function of Range and Volume Coverage Pattern Elevation Samples. (Mahoney, 1987)**


3-84

FMH-11-Part C

References
Ahnert, P. R., M. D. Hudlow, E. R. Johnson, D. R. Greene, and M. R. Dias, 1983: Proposed “on- site” precipitation processing system for NEXRAD. Preprints, 21st Conf. on Radar Meteorology, Edmonton, Amer. Meteor.
Soc., 378-385.
Ahnert, P. R., M. D. Hudlow, and E. R. Johnson, 1984: Validation of the “on-site” precipitation processing system for NEXRAD. Preprints, 22nd Conf. on Radar Meteorology, Zurich, Amer. Meteor. Soc.
Ahnert, P. R., W. F. Krajewski, and E. R. Johnson, 1986: Kalman filter estimation of radar rainfall field bias. Preprints, 23rd Conf. on Radar Meteorology, Snowmass, CO, Amer. Meteor. Soc., 33-37.
Albers, S. C., 1989: Two-dimensional velocity dealiasing in highly sheared environments.
Preprints, 24th Conf. on Radar Meteorology, Tallahassee, FL, Amer. Meteor. Soc., 411-414.

Bergen, W. R. and S. C. Albers, 1988: Two- and three-dimensional dealiasing of Doppler radar velocities. J.
Atmos. Ocean. Technol., 5, 305-319.
Brown, R. A., L. R. Lemon, and D. W. Burgess, 1978: Tornado detection by pulsed Doppler radar.
Mon. Wea. Rev., 106, 29-38.

Browning, K. A., and R. Wexler, 1968: The determination of kinematic properties of a wind field using Doppler
radar. J. Appl. Meteor., 7, 105-113.
Conway, J. W., K. D. Hondl, and M. D. Eilts, 1997: Minimizing the Doppler Dilemma using a unique redundant scanning strategy and multiple pulse repetition frequency dealiasing algorithm. Preprints, 28th Conf. on
Radar Meteorology, Austin, TX, Amer. Meteor. Soc., 315- 316.
Conway, J. W., and W. D. Zittel, 2000: An examination of tornadic signatures associated with the May 3, 1999 outbreak using a new WSR-88D scanning strategy. Preprints, 20th Conf. on Severe Local Storms, Orlando, FL, Amer.
Meteor. Soc., 37-39.
Crum, T. D. and R. L. Alberty, 1993: The WSR-88D and the WSR-88D Operational Support Facility. Bull. Amer. Meteor. Soc., 74,
1669–1687.

Dual Polarimetric Preprocessor Algorithm Description, 2011. Available by request from the Radar Operations Center
Eilts, M. D., and S. D. Smith, 1990: Efficient Dealiasing of Doppler velocities using local
environmental constraints., J. Atmos. Ocean Technol., 7, 118-128.

Elvander, R. C., 1977: Relationships between radar parameters observed with objectively defined echoes and
reported severe weather occurrences. Preprints, 10th Conf. on Severe Local Storms, Portland, OR, Amer. Meteor.
Soc., 73-76.
Forsyth, D. E., C. L. Bjerkaas, and P. J. Petrocchi, 1981: Modular Radar Analysis Software System (MRASS). Preprints, 20th Conf. on Radar Meteorology, Paris, Amer. Meteor. Soc., 696-699.
Fulton, R. A., J. P. Breidenbach, D-J. Seo, D. A. Miller, and T. O’Bannon, 1998: The WSR-88D rainfall algorithm,
Weather and Forecasting, 13, 377-395.
Giangrande, S., and A. Ryzhkov, 2008: Estimation of Rainfall Based on the Results of Polarimetric Echo Classification, J. Appl. Meteor. Climatol., 47, 2445-2462.
Greene, D. R., and R. A. Clark, 1971: An indicator of explosive development in severe storms.
Preprints, 7th Conf. on Severe Local Storms, Kansas City, MO, Amer. Meteor. Soc., 97-104.

Harris, F. I., K. M. Glover, and G. R. Smythe, 1985: Gust Front Detection and Prediction.
Preprints, 14th Conf. on Severe Local Storms, Indianapolis, IN, Amer. Meteor. Soc., 342-345.
3-85

October 2017
Hennington, L. D., and D. W. Burgess, 1981: Automatic recognition of mesocyclones from single Doppler radar data. Preprints,
20th Conf. on Radar Meteorology, Amer. Meteor. Soc., Indianapolis, IN, 704-706.
Hudlow, M. D., J. A Smith, M. L. Walton, and R. C. Shedd, 1989: NEXRAD - New Era in Hydrometeorology in the
United States. Preprints, International Symposium on Hydrological Applications of Weather Radar, University of Salford, Salford, England, August 14-17, 1989, paper B-1.

Hudlow, M. D., D. R. Greene, P. R. Ahnert, W. G. Krajewski, T. R. Sivaramakrishnan, M. R. Dias, and E. R. Johnson, 1983: Proposed off-site precipitation processing system for NEXRAD. Preprints, 21st Conf. on Radar
Meteorology, Munich, Amer. Meteor. Soc.
Jendrowski, P. J., 1988: Regionalization of the NEXRAD severe weather probability algorithm.
Preprints, 15th Conf. on Severe Local Storms, Baltimore, MD, Amer. Meteor. Soc., 205-208.

Johnson, J. T., P. L. MacKeen, A. Witt, E. D. Mitchell, G. J. Stumpf, M. D. Eilts, and K. W. Thomas, 1998:
The storm cell identification and tracking algorithm: an enhanced WSR-88D algorithm, Wea. Forecasting, 13,
263-276.
Kessinger, C., S. Ellis, and J. VanAndel, 1999: A fuzzy logic, radar echo classification scheme for the WSR-88D.
Preprints, 29th Conf. on Radar Meteorology, Amer. Meteor. Soc,, Montreal, 576-579.
Klazura, G. E. and D. A. Imy, 1993: A Description of the Initial Set of Analysis Products Available from the NEXRAD WSR-88D System. Bull. Amer. Meteor. Soc., 74, 1293–1311.
Lhermitte, R. M., and D. Atlas, 1961: Precipitation motion by pulse Doppler. Proc., 9th Weather Radar Conference, Amer. Meteor.
Soc., 218-223.

McGovern, W. E., R. E. Saffle, and K. C. Crawford, 1984: Verification results from 1982-84 operational radar
reflectivity experiment. Preprints, 22nd Conf. on Radar Meteorology, Zurich, Amer. Meteor. Soc., 188-191.
Mahapatra, P.R., D.S. Zrnic, and M.D. Eilts, 1994: Strategies for mitigating range and Doppler ambiguities in
the WSR-88D., Tech Memo, NSSL.
Mahoney, E. A., 1987: Limitations of the Vertically Integrated Liquid Water Algorithm during the NEXRAD Era. Master’s Thesis, Univ.
of Oklahoma, Norman, OK.

Melting Layer Detection Algorithm Algorithm Description, 2012. Available by request from the Radar Operations Center
Mitchell, E. D., 1995: An enhanced NSSL tornado detection algorithm. Preprints, 27th Conf. on Radar Meteorology, Vail, CO,
Amer. Meteor. Soc., 406-408.

Mitchell, E. D., S. V. Vasiloff, G. J. Stumpf, A. Witt, M. D. Eilts, J. T. Johnson, and K. W. Thomas, 1998:
The National Severe Storms Laboratory tornado detection algorithm, Wea. Forecasting, 13, 352-366.
O’Bannon, T., 1997: Using a “terrain-based” hybrid scan to improve WSR-88D precipitation estimates. Preprints, 28th Conf. on Radar Meteorology, Austin TX, Amer. Meteor. Soc., 506- 507.
Pratte, F., D. Ecoff, J. VanAndel, and R. J. Keeler, 1997: AP Clutter mitigation in the WSR-88D. Preprints, 28th
Conf. on Radar Meteorology, Austin, TX, Amer. Meteor. Soc., 504-505.
Rabin, R. M. and D. Zrnic’, 1980: Subsynoptic-scale vertical wind revealed by dual Doppler radar and VAD analysis. J. Atmos. Sci., 37, 644-654.
Seo, D. J., J. P. Breidenbach, and E. R. Johnson, Real-time estimation of mean field bias in radar rainfall data, J.
Hydrol., 233, 1999.
Seo, D. J., J. P. Breidenbach, R. A. Fulton, D. A. Miller, and T. D. O’Bannon, 2000: Real-time adjustment of
range-dependent biases in the WSR-88D rainfall estimates due to non-uniform vertical profile of reflectivity. J.
Hydrometeor., 1, 222-240.
Shedd, R. C., J. A. Smith, and M. L. Walton, 1989: Sectorized Hybrid Scan Processing of the NEXRAD Precipitation
3-86

FMH-11-Part C
Processing System. Preprints, International Symposium on Hydrological Applications of Weather Radar, University of Salford, Salford, England, August 14-17, 1989.

Smalley, D., J. and B. J. Bennett, 2001: Recommended improvements to the Open RPG AP-Edit algorithm.
Lincoln Laboratory Weather Project Memorandum No. 43PM Wx-0081, November 29, 2001, 37pp.
Smalley, D., J. and B. J. Bennett, 2002: Using ORPG to enhance NEXRAD products to support FAA critical systems. Preprints, 10th Conf. on Aviation, Range and Aerospace Meteorology, Porland, OR, Amer. Meteor. Soc.,77-80.
Smalley, D., J., B. J. Bennett, and M. L. Pawlak, 2003: New products for the NEXRAD ORPG to support FAA critical systems.
Preprints, 19th Conf. on Interactive Information Processing Systems, Long Beach,CA, Amer. Meteor. Soc., paper 14.12.

Steiner, M., J. Smith, C. Kessinger, and B.S. Ferrier, 1999: Evaluation of algorithm parameters for radar data quality control. Preprints, 29th Conf. on Radar Meteorology, Montreal, Amer. Meteor. Soc., 267-269.
Steiner, M. and J. A. Smith, 2001: Use of Three-Dimensional Reflectivity Structure for Automated Detection and
Removal of Non-precipitating Echoes in Radar Data. J. Atmos. Ocean Technol., 5, 673-686.
Stumpf, G. J., A. Witt, E. D. Mitchell, P. L. Spencer, J. T. Johnson, M. D. Eilts, K. W. Thomas, and D. W. Burgess, 1998: The National Severe Storms Laboratory mesocyclone detection algorithm for the WSR-88D, Wea.
Forecasting, 13, 304-326.
Super, A. B., and E. W. Holroyd; 1998: Snow accumulation algorithm for the WSR-88D: Final Report. Bureau
of Reclamation Report R-98-05, Denver, CO, July, 75pp.
Warning Decision Training Branch, 2011: Dual Polarization Radar Principles and System Operations. Available at:
http://wdtb.noaa.gov/courses/dualpol/documents/DualPolRadarPrinciples.pdf
WSR-88D Radar Operations Center, 2014: ICD for product specification–build 14.0 and ICD for
RPG to CLASS 1 user–build 14.0, available at: http://www.roc.noaa.gov/WSR88D/BuildInfo/Files.aspx
Winston, H. A., and L. J. Ruthi, 1986: Evaluation of RADAP II severe storm detection algorithms.
Bull. Amer. Meteor. Soc., 61, 145-150

Winston, H. A., 1988: A Comparison of three radar-based severe-storm-detection algorithms on Colorado high
plains thunderstorms. Wea. Forecasting, 3, 131-140.
Witt, A. M. D. Eilts, G. J. Stumpf, J. T. Johnson, E. D. Mitchell, and K. W. Thomas, 1998: An enhanced hail
detection algorithm for the WSR-88D, Wea. Forecasting, 13, 286-303.
Zittel, W. D., 1993: On the performance of three velocity dealiasing techniques over a range of Nyquist velocities: Preliminary findings. Preprints, 26th Conf. on Radar Meteorology., Norman, OK, Amer. Meteor. Soc., 53-55.

3-87

October 2017

3-88

Chapter 4: Overview: Data Processing Algorithms

### 4.1 Introduction.

In addition to the computation of the three radar moments and Dual Pol variables, the signal processing includes
several additional processing steps designed to enhance the meteorological information content of the data. The
sections that follow provide an overview of the functional attributes of these processing algorithms and identify the
known operational considerations.

### 4.2 RDA-Based Data Processing Algorithms.

The algorithms described in this section are executed by the RDA computer processors. These algorithms improve
the availability, accuracy, and usability of the three radar base data moments (Z, V, and W) as well as the Dual Pol
variables (ZDR, CC, and PHI) prior to distributing these data in Level II format.

#### 4.2.1 Radial-by-Radial Noise Estimation.

The WSR-88D uses the Radial-by-Radial Noise Estimation technique to estimate the system noise power dynamically from the in-phase and quadrature data components at every antenna position (radial). The in-phase and
quadrature data components belong to a single radar channel (e.g., either horizontal or vertical). The technique is designed to detect radar volumes that do not contain significant weather signals and estimate the system noise power
from samples at these locations. Because it does not use the Fourier coefficients, this method overcomes the limitations of spectral-processing-based techniques in cases when the numbers of samples at each range volume is small.
This is usually the case in the so called surveillance mode where the unambiguous range is long and the number of
range volumes devoid of signal is more than sufficient for accurate noise estimation. When the PRT yields shorter
unambiguous ranges, it is possible that the majority of samples contain signal as storms span or exceed the entire
unambiguous range. In such cases, the algorithm is unable to produce reliable noise power estimates. For this reason, the algorithm will estimate the noise using data from the longer PRT when two PRTs are collected to calculate
moments for the radial.
Given a set of data, obtained with uniform PRT, with M samples in sample-time and N range bins to operate on,
the steps of the basic algorithm are:
1. Strong point clutter detection and rejection.
2. Detect flat sections in the range power profile. If there are no flat sections detected, return failure code and exit.
Otherwise, for each flat section, estimate the noise level. Take the smallest value to be the intermediate noise
level.
3. Discard all samples that belong to range gates for which the estimated power is greater than the intermediate
noise power multiplied by the SNR threshold. The SNR threshold is dependent on the number of samples M
and is chosen from a table. If, after discarding the samples, the product of M and the number of samples remaining in the active set (i.e., for further processing) is less than MIN_SAMPLE_SIZE, return failure code and
exit.
Note: The MIN_SAMPLE_SIZE is set to 800.
4. Apply a “range persistence” filter. The filter finds 10 or more consecutive range gates with power values that are
larger than the median power in the set, and discards them. If, after discarding the samples, M times the number
of samples remaining in the active set is less than MIN_SAMPLE_SIZE, return failure code and exit.
5. Calculate the mean power from samples at range gates that have not been discarded (i.e., active range bins).
6. Discard samples at range bins where the SNR exceeds threshold.
7. Apply running sum filter as follows:
4-1

October 2017

a. Calculate mean power.
b. Perform running sum of RUN_SUM_LENGTH range bins. RUN_SUM_LENGTH is calculated by
rounding the fraction 500/M.
c. Mark all running sum points that are larger than the product of the mean power from (a) and the RUN_
SUM_THR (1.12 multiplied by RUN_SUM_LENGTH). Let the number of these samples be N_run_sum_
gt_thr. Also, mark all running sum data points to the left and to the right (of the consecutive running sum
points larger than the threshold) that are larger than the mean power from (a) multiplied by RUN_SUM_
LENGTH.
d. Exit algorithm if
i.

N_run_sum_gt_thr is smaller than RUN_SUM_PERC (obtained from Appendix A) times the total number
of averaged points. Return the mean power from (a).

ii.

the number of repetitions of this step is 10. Return the mean power from (a).

iii.

the total number of remaining samples is less than 200. Return failure code only if the first iteration; otherwise, return the mean power from (a).

e. Discard all samples used to obtain averaged points marked in (c) and create new data set from the remaining
samples. Repeat the process starting at (a).
The noise estimate produced by the algorithm is treated differently in each type of cut. Because the algorithm
requires a sufficient number of range gates devoid of signal to produce a noise estimate, the following is recommended in cuts where both long and short PRTs are collected:
• In split and SZ-2 cuts, the noise value obtained from the long PRT scan, at each antenna position, should be
passed along for use in the short PRT sweep.
• If radial data are divided into two groups where one belongs to the long and other to the short PRT (e.g., batch
and staggered cut). Only long PRT data should be used for noise estimation.

#### 4.2.2 Interference Suppression.

The receiver can have an optional interference suppression unit installed. This hardware, when active, monitors received signals in the frequency band just above and just below the bandwidth of the receiver. If significant amounts
of RF energy are detected just outside the main receiver channel, the presence of interference is assumed and a
flag is sent to the signal processor while the interference is perceived to be active. The Hardwired Signal Processor
(HSP) contains hardware controlled by this flag, which can affect the processed data on a bin by bin basis. For the
reflectivity estimates, if the interference flag is set during a current bin, valid data from a previous bin is repeated.
This continues as long as the interference suppression flag is active. This minimizes reflectivity bias errors due to
interference detection. For the Doppler channels, the data value is set to zero for bins while the flag is active. This
minimizes velocity estimate errors due to contamination from interference.

#### 4.2.3 Point Clutter Rejection.

Strong point clutter in the power-sum array is censored by the signal processor, using the following algorithm.
Point Clutter Detection -- The rules for detecting strong point clutter are as follows: The nth bin is declared to be a
point clutter cell if its power value exceeds those of both its second nearest neighbors by a threshold factor (TCN).
In other words,
if
P(n) exceeds
TCN*P(n-2)
and
P(n) exceeds TCN*P(n+2)
where
TCN is the point clutter threshold factor (always greater than 1.0),
4-2

FMH-11-Part C

P(n) is the power-sum value for the nth range cell, and
n
is the 250 meter (500 meter in long pulse mode) range cell index.
Point Clutter Censoring -- The formulae for censoring strong point clutter in the power-sum array P(n) via data substitution are as follows. If the nth range cell is an isolated point clutter cell (i.e., it is a point clutter cell, but neither
of its immediate neighboring cells is a clutter cell), the replacement schemes:
Replace
P(n-l)
with
P(n-2)
Replace
P(n)
with
0.5*P(n-2) + 0.5*P(n+2)
Replace
P(n+l)
with
P(n+2)
If the nth and (n+l)th range bins constitute an isolated clutter pair, the bin replacement scheme is:
Replace
P(n-l)
P(n-2)
with
Replace
P(n)
P(n-2)
with
Replace
P(n+l)
with
P(n+3)
Replace
P(n+2)
with
P(n+3)
Note: Strings of three or more successive clutter cells are prevented by the nature of the algorithm.

#### 4.2.4 Clutter Filtering.

Recovery of usable meteorological data from range bins contaminated by ground clutter return is the primary purpose of the clutter suppression technique employed by the WSR-88D. The amount of data that can be recovered
and the reliability of the reflectivity, velocity and spectrum width estimates from clutter contaminated range bins are
functions of the clutter suppression technique and the characteristics of the clutter itself.
4.2.4.1 Gaussian Model Adaptive Processing.
Clutter filtering accomplished using a WSR-88D-tuned version of the SIGMET Gaussian Model Adaptive Processing (GMAP) clutter filtering technique. Unlike the Legacy IIR clutter filter that indiscriminately reduced power from
all return whose radial velocity falls within the effective notch width, GMAP is applied in the frequency domain and
uses iterative algorithm processing to reduce the power centered around zero within a specified clutter spectrum
width.
Additionally, GMAP interpolates over any removed power components, thus reconstructing the spectrum of any removed meteorological-based power return. This step significantly reduces clutter filter-induced bias in the base data
estimates, thereby improving the accuracy of these estimates.
The fundamental tenet of GMAP is that clutter targets produce a Gaussian frequency distribution around zero velocity with a known, narrow spectrum width. The starting spectrum width (seed) value used in the WSR-88D is 0.4
m/s. For each range bin, GMAP processes the frequency spectrum to identify the power centered on zero velocity.
Using this power value, GMAP calculates a clutter Gaussian having a 0.4 m/s (seed value) spectrum width. GMAP
then applies this initial clutter Gaussian to the frequency spectrum. If needed, GMAP iteratively recalculates the
clutter Gaussian using successively narrower spectrum width values until only clutter power is defined within the
clutter Gaussian. All power points within the final resultant clutter Gaussian are assumed to be from clutter and are
removed from the spectrum.
Both the commonly used IIR clutter filter and GMAP work extremely well in situations when the meteorological
returns are offset from zero velocity. However, when the meteorological return includes components that are at or
near zero, GMAP performs much better. GMAP adapts its filter, on a bin-by-bin basis, based on the clutter’s power
and spectral shape.
Additionally, GMAP provides the capability to “rebuild” the power spectrum of any removed meteorological
return, thereby significantly reducing clutter filter-induced bias in the base data estimates. To achieve this, GMAP
4-3

October 2017

analyzes the resultant frequency spectrum (after signal removal from within the clutter Gaussian) to determine if
there is any meteorological-like power return above the calculated noise level. If there is power above the noise level, GMAP calculates a Gaussian weather model using these points. This weather model is applied to the frequency
spectrum to interpolate over the removed “clutter” power components.
GMAP repeats this process (applying a weather Gaussian and recomputing the spectrum) until the computed power
changes by less than 0.2dB and the velocity changes by less than 0.5% of the Nyquist velocity (Siggia and Passarelli, 2004). The result is that this process effectively rebuilds the meteorological return whose spectrum overlaps the
clutter Gaussian. This significantly reduces the clutter-filter induced bias that weakens reflectivity estimates and
causes velocity data to be biased away from zero.
Although not widely known, the WSR-88D total suppression solution relies on a two-step process to achieve the
amount of suppression required to remove high power terrain targets. The first step, clutter filtering as described
above, is augmented by a second process called “Clutter Censoring”.
4.2.4.2 Clutter Censoring.
Given that GMAP has inherent limits which cap the amount of power that can be removed (the upper limit on
WSR-88D suppression is between 50 and 60 dBZ depending on specific VCP and PRF in use), one would reasonably expect significant residual return displayed in complex or mountainous clutter regimes. However, review of
mountainous site data does not show these expected high residual returns.
The higher than expected suppression capability is due to a technique called “Clutter Censoring”. This censoring
technique is the second step of the WSR-88D clutter suppression process. In an area of identified clutter, power
is first removed using the GMAP filter technique, then the remaining power is measured and the censoring code
calculates a new power removal factor for each range cell equal to the residual power. This dynamic formula efficiently censors the residual clutter power. The RDA performs censoring of each ¼ km range cell which results in
less impact on the surrounding meteorological return.
4.2.4.3 Clutter Mitigation Decision.
The goal of the Clutter Mitigation Detection (CMD) Algorithm is to provide guidance on where to apply the
GMAP clutter filter. CMD uses fuzzy logic to determine whether a gate contains return from clutter. Using this
information CMD builds a clutter Bypass Map by setting a clutter probability flag for each gate in which clutter contamination is considered likely. The GMAP clutter filter is then applied to these gates to reduce the clutter power
contamination prior to the calculation of the based data estimates. The goal of CMD is to only apply the GMAP
clutter filter to gates that most likely contain clutter.
CMD uses a fuzzy logic approach to combine the information from a number of so-called “Feature Fields” into a
single decision-making field. Some of the fields (TDBZ, SPIN) are computed using information from a small region
in range (the “kernel”) around the gate. Other fields, such as CPA, are computed from information from only the
individual gate.
In the current version, the kernel is 1-dimensional, along the beam in range. The length of the kernel is specified
individually for each relevant feature field.
1. TDBZ - DBZ texture: Regions of clutter exhibit rapidly changing reflectivity from gate-to-gate. DBZ texture is
a measure of the gate-to-gate change in value of dBZ. It is computed over a kernel. First, the gate-to-gate dBZ
difference is computed for each gate, and then squared. TDBZ is the mean of this squared difference over the
kernel.
2. SPIN - DBZ spin change: Like TDBZ, SPIN is designed to find regions of rapidly changing reflectivity. SPIN is
based on the number of significant changes in sign in the gradient of the reflectivity field over a kernel. A gate is
considered to be a spin change point if (a) the mean absolute differences in dBZ, from the previous gate to this
gate, and this gate to the next gate, exceeds a threshold (SPIN_THRESHOLD) and (b) the sign of the slope has
4-4

FMH-11-Part C

changed since the last spin point. The number of spin points is computed and then expressed as a percentage
of the number of possible spin changes.
3. CPA - clutter phase alignment: In clutter the phase of each pulse in the time series for a particular gate is almost
constant since the clutter does not move significantly and is at a constant distance from the radar. In noise,
the phase from pulse to pulse is random. In weather, the phase from pulse to pulse will vary depending on the
velocity of the targets within the illumination volume. CPA is computed as the length of the cumulative phasor
vector, normalized by the sum of the magnitudes for the pulses. CPA is computed at a single gate. It ranges from 0 to 1. In clutter, CPA is typically above 0.95. In weather, CPA is often close to 0, but increases in
weather having a velocity close to 0 and a narrow spectrum width. In noise, CPA is typically less than 0.05.
4. ZDR_SDEV - standard deviation of ZDR in range: In clutter, the ZDR field is very noisy and varies widely
from gate to gate. Therefore, the standard deviation of ZDR in range is a good indicator of the likelihood of
clutter. The standard deviation is computed over a small number of gates, 7 in this version. One advantage of
using the standard deviation of ZDR, rather than ZDR itself, is that the absolute calibration of ZDR is difficult
to obtain, whereas the standard deviation is not sensitive to the absolute calibration.
5. PHIDP_SDEV - standard deviation of PHIDP in range: In clutter, the PHIDP field is very noisy and varies
widely from gate to gate. Therefore, the standard deviation of PHIDP in range is a good indicator of the likelihood of clutter. The standard deviation is computed over a small number of gates, 7 in this version. One advantage of using the standard deviation of PHIDP, rather than PHIDP itself, is that the absolute value of PHIDP
depends on the system PHIDP value, whereas the standard deviation is independent of the system value.
6. Handling censored data in the feature fields: In computing the feature fields, data moments which have been
censored are ignored. In addition to the censoring which occurs upstream of the algorithm, CMD sets the clutter flag at a gate to FALSE if the SNR value at the date is below a set threshold. This prevents the application of
a clutter filter at that gate, which improves efficiency.
7. Converting feature fields to interest fields: The values in the feature fields are dependent on what the field physically represents. In order to combine fields in a meaningful manner, feature fields are first converted into socalled “Interest Fields” by applying a so-called membership transfer function to the feature field. The function
converts feature values into interest values between 0 and 1. These interest fields may then be manipulated by
the fuzzy logic system to produce the final decision-making field - in this case, the clutter flag.
8. The CMD clutter flag array is used to build standard bypass maps to determine where to apply clutter filtering.

#### 4.2.5 Base Moment Estimation.

The base radar data (reflectivity, velocity, and spectrum width) are commonly called the three radar moments. This
is because reflectivity, velocity, and spectrum width are moments of the Doppler Spectrum, which is a distribution
of returned signal power as a function of the Doppler Velocity. Reflectivity is the 0th moment, velocity is the 1st
moment, and spectrum width is the 2nd central moment. These moment data are derived using return from only
the horizontal channel.
1. Reflectivity (dBZ): The RDA signal processor calculates corrected reflectivity, dBZ. It uses a log scale equation
based on a dB version of the radar equation for distributed targets. It is based upon the product of the resultant weather signal (from the Doppler spectrum) and the transmitter/receiver gains, clutter power, noise power
(N) cancellation, dBZ0, and range normalization. The dBZ values are calculated from horizontal backscatter
(echoes).
2. Velocity (V): The RDA signal processor calculates velocity after point clutter filtering of the horizontal returns
(echoes). Velocity is a product of radar wavelength (l), sampling time (1 /PRF), and R1 lag.
3. Spectrum Width (W): Spectrum width is a measure of wind shear and turbulence. The RDA signal processor
calculates spectrum width after point clutter filtering of the horizontal returns (echoes). Spectrum width is the
computed variance for R0 and R1 lags.
4-5

October 2017


#### 4.2.6 Super Resolution Processing.

For the WSR-88D, Super Resolution (SuperRes) base data are defined as velocity spectral moment estimates (base
data) with a grid resolution of 250 meters (range) by 0.5 deg (azimuth). Note: “Legacy” base data resolution is 1 km
by 1 deg for reflectivity and 250 m by 1 deg for velocity and spectrum width). SuperRes data are processed on only
the split cut data collection elevations. Legacy resolution processing is done on all other elevations.
1. The range (depth) of the resolution volume is dictated by the transmitter pulse shape and the receiver filter response time. In short pulse (1.57 μs) operation, the sample volume depth is always 250 meters. To maintain this
volume depth, SuperRes data are not range averaged into 1km data range gate.
2. Finer azimuthal sampling is achieved by centering time-series data radials on 0.5 deg azimuthal increments
by modifying the angle table used by the signal processor to define coherent processing intervals within the
time-series data stream. To obtain the desired azimuthal resolution and maintain compatibility with existing signal processing functions, the system collects overlapping 1-deg radials every 0.5 deg. Further, for each range gate
the von Hann window is applied if clutter filtering is not needed (this provides the desired azimuthal resolution).
Conversely, the Blackman window is used if clutter filtering is needed (this provides the required clutter suppression and exceeds the desired azimuthal resolution, but leads to large errors of estimates).

#### 4.2.7 Range Averaging (VCP 31).

The receiver contains a hardware matched filter component which is designed to maximize signal to noise ratios
for the return signals. It is out of necessity designed for optimal performance for the short pulse mode only. When
operating in long pulse (VCP 31 for example) an additional signal processing function is needed. The range gate size
time series is generated from the matched filter is 500m for the transmitter’s wide pulse (wide pulse is also commonly referred to as long pulse, the indicator on the transmitter itself uses Narrow/Wide for pulse width selection,
but it is more commonly referred to as short/long by engineers and meteorologists using the WSR-88D). The RPG
requires its input data in 250m range bins, so each 500m bin is duplicated into two 250m range bins before data are
produced in the RDA for the RPG.

#### 4.2.8 Range Unfolding.

The range unfolding algorithm resolves range ambiguity of radial velocity and spectrum width by assigning each
250 m (500 m in long pulse) range cell to one of the following three categories:
• Valid Doppler return (which may be derived from either first, second, third, or fourth trip echoes)
• Insufficient signal-to-noise ratio (SNR) Doppler return,
• Overlaid Doppler return.
The determination of overlaid Doppler returns is made by comparing the echo power for each range cell, which
is obtained from a low pulse repetition frequency waveform, to the echo power from each other cell that will be
overlaid, i.e., folded, on this cell under a high pulse repetition waveform. If, for the high pulse repetition frequency,
the unambiguous range is ra, the returns at a distance, d, from the radar will be overlaid with returns from d+n*ra,
where n is an integer. For each range cell, the echo power is compared to the echo power of all the other range cells
separated by an integer multiple of ra. If the echo power for the range cell is not greater than the echo power of
every other potentially overlaid cell by a threshold parameter (TOVER, default 5 dB), then the Doppler return for
this cell is not valid, i.e., it is either overlaid or insufficient. If the echo power for this cell does exceed any significant
echo power from overlay candidates by this threshold parameter, then the cell will be labeled valid, subject to the
SNR test, described below.
Echo power is compared to the noise level in order to determine those range cells with an insufficient SNR. If the
echo power is greater than the noise level by a threshold parameter, then the cell is labeled as being either valid or
overlaid, depending on the result of the above test. If the echo power is not greater than the noise level by this
threshold parameter, then it is labeled as insufficient, independent of the above test.

#### 4.2.9 Batch Processing.

4-6

FMH-11-Part C

The Batch waveforms, used at mid-elevations, combine alternating groups of surveillance (Low PRF) and Doppler
(High PRF) pulses to produce radial velocity, spectrum width, and reflectivity. Batch cut processing contains both
surveillance and Doppler pulse trains within each 1° radial in order to make combined reflectivity and Doppler processing possible. Batch cut data are suitable for middle elevations where ground clutter contamination is not prevalent, but Doppler cut data are range ambiguous.
The batch cut processing uses return from the high PRF pulses to calculate Reflectivity, Velocity, and Spectrum
Width. Batch cut processing saves the power of the surveillance pulses and uses this data to range unfold the Doppler pulses to the lesser of the unambiguous range of the surveillance pulses or 300 km. In areas where the high
PRF data are range folded, the reflectivity moment is calculated from the low PRF pulses. (Note: Reflectivity derived
from the low PRF pulses has a significantly higher error of estimates than the estimate derived from the high PRF
data.)
Due to the low number of surveillance pulses, GMAP cannot be used to perform clutter filtering on the surveillance data. Therefore, batch surveillance data are filtered by DC removal.

#### 4.2.10 Sachidananda Zrnic – 2 Processing.

The Sachidananda Zrnic (SZ)-2 algorithm provides a sophisticated method to recover overlaid data by changing the
phase of (or phase coding) each transmitted pulse of the Doppler scan with a systematic sequence known as the
switching code. This phase coding scheme provides a method to separate overlaid signals in the spectral domain.
For example, if there is only first trip signal in the return, the switching code is subtracted to give the cohered first
trip signal from which moments are recovered normally. However, if there is second trip signal added to the first
trip signal due to range folding, cohering for the first trip by subtracting the switching code aligned with the transmission pulse recovers first trip, but not all the phase shift is removed from the second trip signal. The remaining
phase shift in second trip is called the modulation code. The modulation code evenly distributes the second trip
signal across the Doppler frequency interval in eight replicas of the fully cohered second trip signal. Therefore, the
second trip signal does not significantly bias the first trip velocity calculation. Third and fourth trip overlaid signals
may be recovered as well. However, the algorithm is limited to recovering a maximum of two trips out of a total of
four possible overlaid trips.
The SZ-2 algorithm follows the current WSR-88D scan strategy by using two sweeps at the same elevation (thus the
“2” in SZ-2); (1) a Surveillance scan to use as ‘truth’ data to aid in the proper placement in range of the higher velocities from (2) a high PRF, phase coded Doppler scan. When attempting to recover overlaid signals, it is necessary
to process and then remove the stronger of the two overlaid signals before attempting to process the weaker signal.
A strong return, or strong trip, is not always in the first trip. Similarly, a weaker return, or weak trip, is not always in
the second trip. Often, the stronger of the two overlaid signals is beyond the unambiguous range. Therefore, strong
trip does not imply first trip and weak trip does not imply second trip.
1. Cohere to strong trip by subtracting switching code.
2. Recover strong trip moments: reflectivity, velocity, and spectrum width.
3. Go to frequency domain by applying a Fourier Transform.
4. Notch out strong trip centered on the velocity of the strong trip. This removes the strong trip competing power
leaving two replicas of the modulated weak trip signal.
5. Return to time domain by applying an Inverse Fourier Transform.
6. Cohere to weak trip by subtracting the modulation code that coheres from strong trip to weak trip.
7. Recover weak trip moments: power (used in censoring only) and velocity. Spectrum width for the weak trip
comes from the Surveillance scan.
8. Properly place moments using the Surveillance scan data to place recovered strong and weak trip reflectivity and
velocity into proper first and second trip. Use calculated spectrum width from strong trip. For weak trip, use the
4-7

October 2017

spectrum width from the Surveillance Scan.

#### 4.2.11 Automated Volume Scan Evaluation and Termination.

The Automated Volume Scan Evaluation and Termination (AVSET) function is designed to terminate the current
volume scan after the radar has scanned all elevations with significant return. In other words, once the elevation angle overshoots available radar return, the volume scan is terminated because there is no operational benefit realized
by continuing the current volume scan, and a new volume scan is begun. The net effect of AVSET is to shorten the
elapsed time between data collection on low elevation angles of consecutive volume scans when no significant data
are available on higher elevation tilts (i.e., lower elevations are scanned more frequently).
The AVSET algorithm calculates the areal coverage of significant return present on each elevation angle above
5.0° and, if the amount of significant return fails to exceed the predefined thresholds, command an “end of volume scan”. This commanded “end of volume scan” causes the system to enter its normal transition (RDA antenna
retrace, RPG conclude algorithm processing and product generation) to prepare for the start of a new volume scan.
AVSET begins process on the first elevation angle > 5.0°. This is done to ensure adequate vertical coverage to support precipitation accumulation processing.
The AVSET function executes at the WSR-88D Radar Data Acquisition (RDA) unit. For each elevation at and
above 5.0°, AVSET calculates the areal coverage of return 18dBZ and greater, 30dBZ and greater and 38dBZ and
greater. If the areal coverage is less than or equal to the Initial Pass Significant Low Reflectivity Areal Coverage1
Threshold AND the areal coverage is less than or equal to the Initial Pass Significant High Reflectivity Areal Coverage Threshold AND the areal coverage is less than or equal to the Initial Pass Small Core Reflectivity Areal Coverage Threshold AND the Change in Areal Coverage Threshold2 is not met THEN AVSET terminates the volume
scan AFTER completion of the next higher elevation. For example, in VCP 12 if the thresholds are not met on
the 6.4° elevation, then AVSET will terminate the volume scan after completion of the 8.0° elevation slice. In this
example, the data from the 6.4° elevation and the 8.0° elevation slice will be collected by the RDA and processed by
the RPG.

#### 4.2.12 Dual Polarimetric Preprocessor.

Raw (time series) radar data undergo processing at the Radar Data Acquisition (RDA) unit before being passed to
the Radar Product Generator (RPG) in the form of base moments. In addition to the horizontally-polarized WSR88D base moments (reflectivity, radial velocity, and spectrum width), a dual polarization RDA provides differential
reflectivity corrected for noise (between horizontally and vertically polarized reflectivity) and adjusted for calibration
bias, differential propagation phase shift, and cross-polar (horizontal and vertical) correlation coefficient corrected
for noise. Upon receipt at the RPG, base moments undergo processing by several RPG tasks, such as Process Base
Data and velocity dealiasing. The output data from these tasks then go to dual polarization preprocessing.
The preprocessing outputs include smoothed and corrected dual polarimetric base moment data, signal-to-noise
ratio, texture data, and (range-weighted) specific differential phase. Smoothing is a running average technique employing a specified number of sample bins for each feature. Preprocessing precedes data quality assessment, melting
layer detection, radar echo identification, and precipitation estimation.
4.2.12.1 Operational Parameters. None.
4.2.12.2 Strengths. Preprocessing enables better data to be used in algorithms that create dual pol products.
The Initial Pass Significant Low Reflectivity Areal Coverage Threshold is used to “forecast” the likelihood of meaningful meteorological return two elevation angles above the processed elevation (e.g., if the areal coverage is below the threshold on the 6.4° elevation
slice then it is expected that there will not be notable return present on the 10° elevation slice and above). In the context of height ARL,
there are significant height differences between the elevation slices. For example, at 40 nm the center of the beam for the 6.4° elevation
is approximately 27,000ft ARL while the center of the beam for the 10° elevation is approximately 42,000ft ARL; a difference of
15,000ft.
If the Change in Areal Coverage Threshold is met then AVSET reprocesses the elevation with lower thresholds and uses these
lower thresholds to determine whether or not to terminate the volume scan.
4-8

FMH-11-Part C

4.2.12.3 Limitations.
• As range increases, the beam width increases, which results in degraded polarimetric measurements.
• Non-uniform beam filling, particularly at great ranges, can limit the quality of polarimetric measurements.
• Regions having a strong vertical gradient of precipitation within a storm may cause negative KDP values, which
can limit KDP usefulness.
The Dual Pol variables are mathematical results calculated using input from the return from both the horizontal and
vertical channels.
1. Differential Reflectivity (ZDR). This parameter is the ratio of horizontal reflectivity to vertical reflectivity. It
provides a measure of hydrometeors’ relative dimensions and is calculated by the RDA signal processor after
point clutter filtering.
2. Correlation Coefficient (CC). This parameter is a measure of the correlation between horizontal and vertical
signals and hence provides a measure of scatterers’ orientation and shape. It is calculated by the RDA signal
processor after point clutter filtering. It is the ratio of the cross-correlation at lag0 between the horizontal and
vertical paths and the squared root of the product of lag0 in the two paths.
3. Differential Phase (PHI). This parameter is the phase difference between horizontal and vertical signals and is
a measure of total path propagation time difference in the medium traversed. It is computed by the RDA signal
processor after point clutter filtering. For rain, this is a measure of total liquid water along the path. It is the
inverse tangent of the cross- correlation at lag0 between the horizontal and vertical paths.

#### 4.2.13 Future Enhancements.

4.2.13.1 Coherency-Based Threshold (CBT).
Currently all data are censored based solely on the SNR threshold applied to the return power received from the
horizontal polarized channel. With the recent deployment of Dual Polarization fleet wide, CBT will take advantage
of this new capability by using returns from both of the two orthogonally polarized channels. With CBT, valid radar
return will be based on the sum of the cross-correlation estimates as well as the power and autocorrelation from
each of the dual polarization channels. CBT restores some data that would have otherwise been censored based on
return power below the single polarized channel SNR threshold, thereby improving the radar’s data coverage without negatively affecting overall data quality.
Because the Dual Pol implementation of the WSR-88D effectively splits the transmitted power in half, data lost
owing to the effective reduction of 3 dB or more in radar sensitivity is at least partially recovered with CBT. Weak
reflectivity features such as wind shift lines or thunderstorm outflow boundaries which were no longer detectable
after the recent upgrade to Dual Pol are expected to be, at least partially, recovered with CBT. Identification of these
wind shift boundaries is particularly important to the aviation community and aircraft safety.
4.2.13.2 Staggered PRT.
Staggered Pulse Repetition Time (SPRT) is a method for mitigating range-velocity ambiguities. The primary feature of SPRT is the use of two distinct pulse repetition intervals. The two PRT’s can be selected to provide good
unambiguous range (from the longest of the two) while supporting a velocity dealiasing method for extending the
unambiguous velocity. In SPRT (see Fig. 4-1), overlaid echoes can occur during T1 receive time due to its short unambiguous range even when weather returns are contained by the unambiguous range of T2. The SPRT algorithm
features recovery of estimates in such overlaid echo conditions.

4-9

October 2017


> **Figure 4-1: Illustration of Staggered PRT Time Series**

> 4.2.13.3 CLEAN-AP.

4.2.13.3 CLEAN-AP.
CLEAN-AP is Clutter Environment Analysis using Adaptive Processing. It is a ground clutter identification and
filtering algorithm. The primary feature of CLEAN-AP is the use of phase information from the Autocorrelation
Spectral Density (ASD) to estimate the extent of clutter contamination. CLEAN-AP uses the results of the ASD to
adaptively select data windowing and for determining the number of spectral coefficients to remove in the clutter
filtering process.
CLEAN-AP makes possible for SPRT to employ effective clutter filters since it does not require uniform sampling
as in the case of GMAP. The SPRT portion of the algorithm also features processing of estimates in overlaid echo
conditions.
The major functions of the algorithm are: (1) Clutter Identification, (2) Clutter Filtering, (3) PRT Segment (Overlaid Echo) Management, (4) Moment and Polarimetric Variable Estimation, (5) Velocity Dealiasing, and (6)
Point Target Censoring and Determination of Significant Returns. Following are general descriptions of the functions employed in the algorithm.
1. Clutter Identification. The CLEAN-AP algorithm takes advantage of a feature of the ASD that produces
a phase distortion when signals containing clutter components are subjected to a data window. The spectral
leakage resulting from this processing causes the phase of the complex ASD to depart from a normally
linear behavior in regions near the center of the Nyquist interval. The amount of departure from a linear
function is an indication of the strength of the clutter. The algorithm uses the strength of the clutter to
adaptively select not only the number of clutter associated spectral coefficients to remove, but also selects
an appropriate data window function that optimizes the clutter filtering process. The ASD is an autocorrelation
of the lag-0 and lag-1 Discrete Fourier Transform of the data samples. The ASD is thus the cross-correlation
spectrum of the signal with itself (convolution of the signal samples with a one sample delayed version).
The phase information in this complex result is retained. For signals containing narrow spectrum width components, spectral leakage causes distortions in the phase. Since radar signal returns from clutter exhibit narrow
spectrum widths with respect to weather, this phase distortion can be used to estimate clutter strength. For
polarimetric radars like the WSR-88D, clutter identification and filtering needs to be done in both the horizontal (H) and the vertical (V) channels. The ASD from the H channel can be used to determine clutter filtering
parameters and used as a master for controlling the filtering process in both H and V channels. Both channels
can also use the ASD function to independently identify clutter and determine parameters.
2. Clutter Filtering (Clutter Associated Coefficient Removal and Interpolation). The clutter identification
process selects one of the following data window functions to be used in generating the filtered power spectrum: (1) Rectangular, (2) von Hann, (3) Blackman, (4) Blackman-Nuttal. The spectral coefficients for both
the H and V channels are modified during the clutter filtering process according to the filter control parameters
generated during the identification function. The appropriate number of coefficients about the zero point is
removed from the windowed spectrum. The removed coefficients are replaced with new ones based on interpolation.
3. PRT Segment (Overlaid Echo) Management. Samples associated with the long PRT yield longer unam4-10

FMH-11-Part C

biguous ranges. Samples from the short PRT create overlaid echoes, but provide a higher maximum unambiguous velocity. The algorithm selects the best sample set for variable estimation and censors overlaid echoes
where appropriate.
4. Moment and Polarimetric Variable Estimation. The moments and variables are generated by the standard
estimators for the baseline polarimetric WSR-88D. For velocity, estimates come from both the short and
long PRT samples. The velocity dealiasing function uses both sets of velocity estimates. Spectrum width
values are derived from the standard R0/R1 estimator.
5. Velocity Dealiasing. The velocities from the short (V1) and the long (V2) PRT’s alias in different ways. The
difference in the estimates of V1 and V2 are used to determine a set of dealiasing rules. For an example
where the PRT ratio between short and long is 2/3, this Velocity Difference Transfer Function (VDTF)
yields three regions that coincide with the velocities that are (1) lower than the minimum Nyquist velocity
of V1, (2) within the Nyquist range for V1, and (3) greater than the Nyquist velocity for V1. In regions 1 and
3, the velocity estimates for the short pulse (V1) are aliased, but in different ways. The VDTF is a piece-wise
linear function with distinct values associated with the three regions. The estimate of V1 is compared to the
value of the VDTF and this identifies which of the three regions in which the estimate lies. For region 1, two
times the Nyquist velocity of the short PRT is subtracted from the estimate of V1. For region 3, two times
the Nyquist velocity of the short PRT is added to the estimate of V1. For region 2, the estimate of V1 is not
aliased and it is the value used. Because the values for V1 that are compared to the VDTF are estimates, some
errors can occur where the wrong de-aliasing rule is applied. This results in what is termed a “catastrophic
error”. These errors are well understood and are corrected in the RPG with the velocity dealiasing algorithm.
6. Point Target Censoring and Determination of Significant Returns. Strong point clutter censoring is
done on all of the power and cross correlation arrays in the same manner as the other WSR-88D methods.
This is done prior to moment and variable estimation. Censoring of weak returns is done on the moments and
variables as per the usual method using a signal to noise ratio threshold.

### 4.3 RPG-Based Data Processing Algorithms.

The algorithms described in this section execute on the RPG processors and do not impact the Level II data. Output from these algorithms is used by downstream meteorological algorithms and Level III product generation tasks.

#### 4.3.1 Data Quality Assurance.

The Data Quality Assurance (DQA) Algorithm provides an alternate source of reflectivity factor data to WSR-88D
algorithms internal to the RPG. This algorithm does not yield a product that is available to external systems but
results from DQA processing are available as a product in the form of the Base Reflectivity Data Array Product
Edited with DQA (see Section 2.6.1). The DQA identifies and removes radials contaminated with constant power
function signatures as well as regions of anomalous propagation and ground clutter from the reflectivity factor data.
Additionally, DQA identifies and removes solar strobes, radial spikes, and speckle. Radially-isolated spikes and random speckle are also filtered from the reflectivity data along with solar strobes. This is the only such edited reflectivity factor data available. Upon completion of each elevation cut of the radar volume, the DQA passes the processed
reflectivity factor data to receiving algorithms as an elevation cut. The original radar volume’s elevation cut spatial
integrity is maintained. The reflectivity factor data provided after DQA processing are not quantized. The original
resolution of the reflectivity factor data are maintained. While DQA returns edited reflectivity factor data, it requires
for analysis the original three moments of radar data for the elevation tilt plane. The EET algorithm, the DVL
algorithm, and the DRQ algorithm require the DQA data as input. DQA does not process super resolution data or
SAILS elevation cuts (see 4.3.6). Recombined, legacy resolution (1° x 0.54 nm [1km]) data provide input to DQA.
The DQA produces an intermediate, elevation cut product internal to the RPG of 1° x 0.54 nm (1 km) original
polar grid resolution to a maximum range of 460 km (248 nm). This is the identical elevation cut of reflectivity
factor data except for the removal of identified data contaminants. The DRQ product is simply each DQA elevation
cut packaged as an external ORPG product. The DQA first independently analyzes every radial for the presence
4-11

October 2017

of a constant power function signature. This radial signature is represented by a steady increase in reflectivity factor
with distance from the radar following the inverse range-squared relationship. That is, reflectivity factor increases
with distance as a function of the inverse of range squared for equal returned power along the length of the radial.

> **Figure 4-2 illustrates the contrast between individual radials – one with weather, the other with a constant power**

> function signature. Identified constant power function radials have their original reflectivity factor replaced with the

function signature. Identified constant power function radials have their original reflectivity factor replaced with the
“no data” value.


> **Figure 4-2: Reflectivity Factor Signatures.**

> Sample radials depicting typical reflectivity factor signatures for a radial involving weather echo (blue)

Sample radials depicting typical reflectivity factor signatures for a radial involving weather echo (blue)
and a radial with a constant power function (orange).
DQA follows the constant power function radial analysis with an analysis to identify and remove AP and ground
clutter in the elevation tilt plane. The identification of AP uses three concepts applied in one and two dimensions
as shown in Figure 4-3. The spectral signature of non-moving AP and ground clutter differs from that of weather. The AP is typified by high reflectivity factor data coincident with near zero Doppler moment values. The AP
typically is observed at low elevations nearer the radar; thus, DQA uses elevation discrimination to identify areas of
AP. Thirdly, DQA exploits the observation that AP often has a visual texture of greater variability in the reflectivity
factor data as compared to areas of weather when viewed in two dimensions. The identified AP have their original
reflectivity factor replaced with the NEXRAD “no data” value. The AP editor is deactivated in DQA when CMD is
operational in the RDA.

4-12

FMH-11-Part C


> **Figure 4-3: Radar Product Discriminations.**

> The three general concepts for the AP editor portion of the DQA Algorithm are shown.

The three general concepts for the AP editor portion of the DQA Algorithm are shown.
DQA also removes solar strobes from the elevation data. A solar ephemeris computation determines radials impacted by the sun’s position. The weak power solar return is removed. Weather along those radials is retained. Spikes are
isolated contiguous gates of reflectivity along a radial. They are identified and removed using a technique adapted
from that developed at the National Severe Storms Laboratory. A speckle filter with DQA performs a multi-pass of
the data with a 2D kernel to remove isolated speckle debris in an elevation
4.3.1.1 Operational Parameters. None
4.3.1.2 Strengths/Applications
• Combined removal of AP, clutter, constant power function radials, solar strobes, spikes, and speckle.
4-13

October 2017

•
•
•

Provides an alternate, edited source of reflectivity factor data to WSR-88D algorithms.
Identified problematic data removed from reflectivity factor; little, if any, additional editing required.
Removes “bulls-eye” and “starburst” full volume constant power function phenomenon often associated with a
WSR-88D radar in maintenance mode or suffering a hardware failure.

4.3.1.3 Limitations.
• Identified problematic data removed from reflectivity factor; no ability to recover those original values in DQA
product.
• Constant power functions are not removed in the presence of weather signature.
• Actual weather signatures can be erroneously removed.
• The AP editor will not analyze areas lacking Doppler moment data.
• Dense noise speckle will remain.
• Radially contiguous noise (such as from cellular towers) might remain.

#### 4.3.2 Data Recombination.

WSR-88D meteorological algorithms were designed to ingest “legacy” resolution (reflectivity = 1 km by 1 deg; velocity and spectrum width = 250 m by 1 deg) base data. When Super Resolution data (250 m by 0.5 deg for reflectivity, velocity and spectrum width) is received from the RDA, the Recombination algorithm is used to reformat the
data into a resolution that the algorithms can use.
Azimuthal radial recombination takes two Super Resolution (0.5 deg) radials and combines them into one 1.0 deg
radial. The recombination algorithm assumes a bimodal Doppler spectrum model - that is, the spectral moments
(signal power, Doppler velocity, and spectrum width) of each Super Resolution radial completely characterize the
underlying Doppler spectrum density (Gaussian assumption), and the spectral moments of the recombined 1.0 deg
radial correspond to a composite Doppler spectrum that is the average of the two super-resolution radial Doppler
spectra.
1. Reflectivity recombination:

With the assumption of the model described above, recombined reflectivity Zr (in mm6/m3) on a 1 km-by-1
deg grid is obtained by averaging the corresponding eight Super Resolution reflectivities (also in mm6/m3)
from a 250 m-by-0.5 deg grid:

2. Velocity recombination:
Recombined velocity (Vr) on a 250 m-by-1 deg grid is obtained from the Super Resolution reflectivities and
velocities on a 250 m-by-0.5 deg grid by combining azimuthally adjacent range gates (v1 and v2).
4-14

FMH-11-Part C

Before computing Vr from the two azimuthally adjacent range gates, v1 and v2 are dealiased to avoid improper
averaging in the case that one velocity is aliased and the other one is not. After combining v1 and v2 into Vr, Vr
is re-aliased to the corresponding Nyquist interval.
3. Spectrum Width recombination:

•

•
•

The recombined spectrum width (Wr) on a 250 m-by-1 deg grid is obtained from the Super-Resolution reflectivities, velocities, and spectrum widths on a 250 m-by-0.5 deg grid by combining azimuthally adjacent range gates
(w1 and w2).
A challenging aspect of this algorithm is that radial recombination occurs in the RPG, which is after the RDA
has thresholded the base data based on the signal-to-noise ratio, censored unrecoverable range-folded data, and
quantized valid data for efficient transmission and storage. Most of the complexity of the recombination algorithm arises from this fact and from special conditions that require extra processing.
Missing radial data: This occurs if a radial pair is incomplete for a given antenna position. That is, if the closest
available radial to complete a pair is separated by an azimuth angle larger than expected (fat radial). If only one
of the super-resolution radials is available, the recombined radial is generated from it. That is, it is assumed that
the missing radial is the same as the one available. If none of the radials in the pair are available, a recombined
radial is not produced.
Beginning and end of elevation scan: This occurs if a matching radial to form a pair simply does not exist.
This would be the situation, for example, if the last radial in the elevation scan was the first radial in a pair. The
recombination algorithm treats these cases as a missing radial data case.
Pulse Repetition Frequency (PRF) sector boundary: This occurs if radials in a pair are collected with different
Pulse Repetition Times (PRT). This case is also treated as the missing radial data case with the stipulation that
the radial with the smaller Nyquist velocity (longer PRT) is considered missing (the radial with the larger Nyquist velocity is retained).


#### 4.3.3 Automatic Pulse Repetition Frequency Selection.

The Automatic Pulse Repetition Frequency (Auto PRF) technique analyzes echo returns in the lowest elevation
scan, determines that PRF yielding the smallest obscuration, and assigns that to the elevation scans contained in the
lowest 7.0°. This new VCP definition is downloaded to the RDA and invoked at the beginning of the next volume
scan.
Using data from the surveillance waveform in the lowest elevation scan, echo power is computed. Then a measure
of the overlaid echoes is computed for four PRFs from the allowable Doppler ranges for that site. The PRF yielding
the least amount of overlaid echo is selected for subsequent processing up to 7.0° in elevation. Above that elevation, there are no problems with overlaid echoes.

#### 4.3.4 Storm-Based Auto PRF.

The Auto PRF algorithm has been in the WSR-88D since the beginning of the program. This algorithm works very
well but it simply selects the Doppler PRF that results in the least amount of range folded (purple) data for the
entire area within a 230 km radius of the radar without consideration for the importance of any individual storm.
To address this limitation, the Storm-Based Auto PRF function was implemented to augment the legacy Auto PRF
algorithm. The Storm-Based Auto PRF function has two options which can be selected by the operator: MultiStorm and Single (Storm).
4.3.4.1 Multi-Storm.
Within the Storm-Based Auto PRF Mode, the Multi Storm Auto PRF option automatically tracks the 33 most sig3
The Multi Storm Auto PRF option will track and process up to 3 storms (NOTE: Only cells that have a VIL > 20kg/m3
4-15

October 2017

nificant storms (based on the Cell-Based VIL) using the Storm Cell Identification and Tracking (SCIT) algorithm
output and assigns the PRF that results smallest area of range-obscured data for those storms.
Upon selection of the Storm-Based Auto PRF Mode, the default option (Multi Storm-Based Auto PRF) will identify the 3 most significant storms based on the Cell-Based VIL ranking. Using SCIT output, the location of the 3
identified storms are projected for the next volume scan. For each projected storm location, calculate a “storm
circle”. Calculate the area of range obscured data within each storm circle for each Doppler PRF. Select the PRF
that provides the smallest area of range-obscured data. For each subsequent volume scan, repeat this sequence to
dynamically forecast the location of the 3 most significant storms and select the best PRF for those storms.
The result of this application is a dynamic PRF that tracks the most significant storms and continuously assigns the
“Best” Doppler PRF for those storms.
1. When this option is enabled, automatically select the 3 most significant storms based on the Cell-Based VIL
2. Use the “forecast position” from SCIT to project where these storms will be during the next volume scan
3. Using the forecast locations, calculate a “storm circle” for each storm. The “storm circle” is defined as the
boundary of a 20 km4 radius circle around the projected storm location
4. Modify the Auto PRF Algorithm to:
▪ Calculate the area of range-obscured data within each “storm circle” for each Doppler PRF
▪ Select the PRF that results in the smallest area of range-obscured data within the “storm circles”
▪ Assign this PRF to the VCP
▪ Download the modified VCP to the RDA
5. Each subsequent volume scan, select the top 3 storms from the ranking of storm based on the Cell-Based VIL
and repeat steps 2-5 until there are no storms identified by the SCIT algorithm or the operator selects another
Auto PRF option
6. When there are no cells identified, suspend Storm-Based Auto PRF function and use the Auto (Elevation) PRF
Algorithm.
7. When a new convective cell is identified by the SCIT algorithm, reactivate the Storm- Based Auto PRF function,
Multi Storm-Based Auto PRF option.
The result of the Storm-Based Auto PRF Mode, Multi Storm Auto PRF option is a dynamic function that tracks
the three most intense storms, as determined based on the Cell-Based VIL ranking, and continuously assigns the
“Best” Doppler PRF for those storms.
4.3.4.2 Single-Storm.
The operator selected Single-Storm Auto PRF option will automatically track a selected storm of interest and assign
the best PRF for that storm.
Upon selection of this option, the Storm-Based Auto PRF Mode, selected Single-Storm Auto PRF option, the algorithm will accept the Cell ID as designated by the operator. This cell ID will be tracked as the cell of interest. The
Single-Storm Auto PRF option will rely on the SCIT algorithm output to project the location of the selected cell
for the next volume scan. Then, for this forecast location, the Storm-Based Auto PRF algorithm will select the PRF
that provides the smallest area of range-obscured data over the storm of interest. Each subsequent volume scan,
the function would automatically use the SCIT Storm ID forecast location for the storm of interest and select the
“Best PRF” based on this forecast location.
are considered). If there are fewer than 3 storms, it tracks the number of storms available.

The 20 km radius was based on the same logic used in determining the correlation distance for Mesocyclone Detection Algorithm to associate a “Low Core” circulation with a SCIT identified cell location.
4-16

FMH-11-Part C

1. Modify the PRF Selection function to accept an operator selected Cell ID
2. Modify the PRF Selection function to use this Cell ID as the “storm of interest”
3. Use the “forecast position” from SCIT to project where the storm of interest will be next volume scan
4. Using this forecast location, calculate a “storm circle”. The “storm circle” is defined as the boundary of a 20 km
radius circle around the projected storm location
5. Modify the Auto PRF Algorithm to:
▪ Calculate the area of range-obscured data within the “storm circle” for each Doppler PRF
▪ Select the PRF that results in the smallest area of range-obscured data within the “storm circle”
▪ Assign this PRF to the VCP
▪ Download the modified VCP to the RDA
6. Each subsequent volume scan, repeat steps 3 through 5 until one of the conditions listed in Step 7 are satisfied
7. Continue to use this storm ID as the basis for the Auto PRF Algorithm until either:
▪ The Storm ID moves beyond 230 km from the radar,
▪ The particular Storm ID is no longer identified by SCIT,
▪ The operator selects a different PRF control option, or
▪ The operator selects a different “storm of interest”
8. If the Storm ID moves beyond 230 km from the radar, or the particular Storm ID is no longer identified by
SCIT, disable the Selected (Single) Storm Auto PRF option and reactivate the Multi Storm Auto PRF option. (If
there are no identified storms within 230 km, then suspend the Multi Storm-Based Auto PRF option and use
the legacy Auto PRF Algorithm.)
9. When a new convective cell is identified by the SCIT algorithm, reactivate the Storm-Based Auto PRF Mode,
Single Storm-Based Auto PRF option.
The result of the Storm-Based Auto PRF Mode, Single-Storm Auto PRF option is a dynamic function that tracks a
storm of interest and continuously assigns the “Best” Doppler PRF for that storm.

#### 4.3.5 Velocity Dealiasing.

Velocity dealiasing is accomplished by one of two available techniques. By default, the Two-Dimensional Velocity
Dealiasing Algorithm (2D-VDA) will be used unless either the Velocity (Measurement) Increment is set to 1 m/s
(1.94 kts), the operator chooses to disable the 2D-VDA and use the Legacy Velocity Dealiasing Algorithm (VDA),
or the operator assigns more than one PRF for data collection (different PRF for one or more defined PRF sectors).
4.3.5.1 Two-Dimensional Velocity Dealiasing Algorithm (2D-VDA).
The 2D-VDA, originally developed in the Research Applications Program at the National Center for Atmospheric
Research by Jing and Wiener (1993), attempts to dealias connected two-dimensional regions within an elevation scan
by minimizing all detected velocity discontinuities. It calculates the difference between a gate and the neighboring
gates, puts paired gates into a smoothness function, and applies a least squares method to find suitable velocity values which minimize the smoothness function.
To realize the full potential of the two-dimensional approach, the 2D-VDA must be applied to a full elevation scan.
This is done in two steps. In the first step the full field is used to generate an environmental wind table. In order to
conserve computer CPU and memory resources, the 2D- VDA may sub-sample complex, large fields. This is done
by gridding a velocity field azimuthally and radially and computing a median velocity value for the center of each
grid. The second step partitions the elevation scan, and the 2D-VDA then dealiases smaller features such as mesocyclones and tornado vortex signatures. Finally, the internally generated environmental wind table is used to place
small isolated regions in the correct Nyquist co-interval, ±2kVN where k is an integer.
4-17

October 2017

A newer version of the basic 2D-VDA, developed by ROC personnel and described by Witt et al. (2009) applies a
weighting factor to the velocity differences. The closer the difference is to 0 or 2VN, the greater the weight that is
given to the velocity difference. As the velocity difference approaches VN, the weight goes to 0. Use of the weighting factor reduces or eliminates the contribution of noisy or unreliable data to the optimization setting. Another
change that improved the dealiasing further was to remove velocity data that is associated with high spectrum width
before applying the 2D-VDA. In the version of the 2D-VDA being fielded for the WSR-88Ds, spectrum width data
are used to weight the corresponding velocity differences. Differences associated with high spectrum width receive
a smaller weighting during the global optimization of the algorithm. The spectrum width and velocity difference
weights are multiplied together to produce a final weight.
Another enhancement is to treat as separate regions two areas connected by a narrow bridge of noisy velocity data.
Finally, the fielded version of the 2D-VDA temporarily removes velocity data considered to be contaminated by
side lobes. These velocities typically are near zero. The velocity data are restored after the rest of the field has been
dealiased by comparing them to nearby data already dealiased and adjusting them by ±2kVN.
The 2D-VDA is not expected to cause any perceptible delay in product availability. The 2D-VDA can be used with
all VCPs except VCP 121 which uses the MPDA (see Section 4.3.5.3). For all other VCPs except VCP 31, a site may
choose to use different Doppler PRFs for different azimuthally-defined sectors within an elevation cut to reduce
range folding in that sector. When “sectorized” PRFs are used, the 2D-VDA cannot be used. Instead, the RPG
automatically reverts to the legacy VDA. Once a site switches back to using the same PRF for a scan, the 2D-VDA
will again be invoked. A site will have the option of reverting to the legacy VDA and disabling the 2D-VDA.
4.3.5.2 Legacy Velocity Dealiasing Algorithm.
The legacy velocity dealiasing algorithm is automatically used when the velocity measurement increment is 1 m/s
(1.94 kts) or the operator has assigned different a PRF for one or more PRF sector definitions.
The base data Doppler velocities from the radar are ambiguous. Velocities outside the region +VNyQ (Nyquist velocity) are shifted by the radar by +2n VNyQ into +VNyQ where n is an integer. These errors, or aliasing of the velocity, are corrected by using continuity of velocity along radials and between adjoining radials at the same range. The
velocity dealiasing algorithm proceeds as follows:
• For each radial, the ambiguous velocity at each sample bin is compared with the unambiguous velocity at a
previous (closer in range) sample bin. Only velocities that come from range bins meeting the SNR threshold and
that are not range folded are considered good. The five bins closer to the radar are searched for a valid sample.
When the previous five sample bin values indicate below SNR threshold or range folded, a nine-point average
of surrounding data are used for comparison. The nine points will include the four preceding unambiguous
values along the same radial and five values from the previous radial. The five values are taken from the bin
adjacent to the sample volume and the next four bins in increasing range. When a nine-point average cannot
be computed, the algorithm looks back (toward the radar) up to 30 range cells (adaptable value) on the current
working radial, and ahead (away from the radar) up to 15 range cells (adaptable value) on the previous saved
radial. If a velocity for comparison cannot be found, the value is rejected as bad. If a valid velocity for comparison is found, the algorithm compares the difference in these velocities to a threshold (TH_l). This threshold is
typically 5 m/s (10 kts) for clear air mode, and 10 to 15 m/s (19 to 29 kts) for precipitation mode. If the velocity difference is greater than TH_1, the algorithm attempts to place the ambiguous velocity to within TH_1 of
its nearest radial neighbor (or nine- point average, etc.) by adding or subtracting integer multiples of 2 VNYQ.
• If the nearest radial neighbor is within five range cells, but the ambiguous velocity cannot be dealiased to within
a velocity difference TH_1 of this neighbor, the ambiguous velocity is compared to a nine-point average of its
neighbors. This nine-point average is as defined above. If the ambiguous velocity can be dealiased to within a
derived threshold TH_2 of the nine-point average, it is dealiased. If the ambiguous velocity is not within TH_2,
nor can it be dealiased to within TH_2, the ambiguous velocity is rejected as bad. TH_2 is the larger of TH_1,
40% of the nine-point average, or twice the standard deviation of the nine points.
• When the ambiguous velocity is locally isolated, the current working radial (up to 30 range cells toward the
radar) or previous saved radial (up to 15 range cells away from the radar) is searched for a velocity with which
4-18

FMH-11-Part C

•

•

•

to be compared. If such a velocity is found, and the ambiguous velocity is within a velocity difference threshold
TH_3 or can be dealiased to within TH_3, it is dealiased. The threshold TH_3 is typically 1.5 TH_1. If a velocity to compare with is found, but the ambiguous velocity cannot be dealiased to within TH_3, the ambiguous
velocity is rejected as bad.
The algorithm also checks for dealiasing errors. This is referred to as re-unfolding. It will correct such errors on
the current working radial when either: 1) a site-adaptable number of range cells, NUM_1, exhibit large velocity
differences, TH_4, in azimuth, or 2) two large velocity jumps of opposite sense, exceeding TH_5 in magnitude,
exist along the radial and the jumps are separated by no more than a site-adaptable number of range cells, e.g.,
75. The parameter NUM_1 is typically 5 for clear air mode and 10 for precipitation mode. TH_4 is approximately 60% of the Nyquist interval. The parameter TH_5 is the smaller of 80% of the Nyquist interval and 45
m/s (87 kts). For the case of large azimuthal shear, velocities are re-unfolded using a least-squares technique.
The technique compares the velocity in question with its nearest neighbor in the preceding azimuth and nearest neighbor in the same radial (farther from the radar). The velocity in question is re- unfolded if, by adding
or subtracting a Nyquist interval to or from it, the sum of squared velocity differences between the velocity in
question and its two neighbors is minimized.
If the re-unfolded radial contains no large velocity jumps, it may be copied for subsequent use. If only one large
velocity jump exceeding TH_5 in magnitude is identified along the current working radial, this radial will not be
copied for subsequent use. The copy of the previous radial is allowed to be up to five radials old. If five successive radials have large velocity jumps, azimuthal information becomes unavailable for dealiasing.
If five consecutive velocities have been removed from the radial, an attempt is made to replace them. For each
rejected velocity, a reference velocity is calculated. This reference velocity is either a radial neighbor or the
average of the 15 unambiguous velocities along the previous radial (the four closer, the current, and the ten
further in range). When the first value removed in the string of five is within a relaxed threshold TH_6 of the
reference velocity or can be dealiased to within this threshold, the initially removed velocity is replaced. If the
rejected velocity can be dealiased to within TH_6 of the reference velocity, it is dealiased. The threshold TH_6
is the lesser of 40% of the Nyquist interval and 22.5 m/s (44 knots). Once the first rejected velocity is replaced,
subsequent replacement uses the threshold TH_3 to compare the next rejected velocity and a running average
of the replaced velocities.

4.3.5.3 Multiple Pulse Repetition Frequency (PRF) Dealiasing Algorithm.
The Multiple Pulse Repetition Frequency Dealiasing Algorithm (MPDA) mitigates range folding by combining
Doppler data from up to three scans at the same elevation angle but where each scan uses a different PRF. Changing
the PRF changes the unambiguous range and thus the areas that may be range folded. The primary PRFs used are
~1282, ~1095, and ~857 Hz with unambiguous ranges of 117 km, 137 km, and 175 km, respectively. In the WSR88D System they are referred to simply as PRFs 8, 6, and 4, respectively. Experience has shown the MPDA works
well out to about 175 km but is inconsistent in reducing range folding beyond this range. These are then range
dealiased, aligned, and processed to produce a final dealiased velocity field based on the combined scans. The actual
unambiguous ranges and corresponding Nyquist velocities depend on the operating frequency of the radar and may
be expressed as follows:
RaVa = c2/(8f)
where c is the speed of light, f is frequency, Ra is the Unambiguous range and Va is the Nyquist velocity. Typical
single PRFs, Nyquist velocities, and unambiguous ranges are shown in Table 5-10 of this Handbook.
Another feature of the MPDA is its approach to velocity dealiasing. MPDA takes advantage of having up to three
independent measurements of velocity for the same location in space. It first tries to dealias all locations for which
there are three velocities. If no satisfactory solution is found with three velocities, it will next try to dealias using
pairs of velocities. Some locations will only have two velocities because the third velocity is range folded. Finally,
MPDA completes its processing by dealiasing velocities for locations for which only one velocity is available. While
the use of multiple velocity estimates generally improves the reliability of dealiasing, a region with only velocity data
4-19

October 2017

from the scan using PRF 4 may have dealiasing errors because its Nyquist velocity is only about 21 m/s.
The MPDA requires VCP 121 (Table 5-4 of this Handbook). It requires three Doppler scans, besides the surveillance scan, at both 0.5 and 1.5 degrees elevation. It requires two additional Doppler Scans in addition to the Batch
mode scans at 2.4 and 3.3 degrees elevation. Lastly, it adds one additional Doppler scan at 4.3° elevation. Above 4.3°
elevation, the VCP is identical to VCP21.
The MPDA incorporates the WSR-88D’s RDA’s phase coding algorithm for the first Doppler scan at 0.5 and 1.5
degrees to help reduce residual range folding. The second and third Doppler scans are range unfolded at the RDA
(See Figure 4-4). At higher elevations, the MPDA incorporates a range-unfolding algorithm similar to the one
implemented at the RDA for the extra Doppler scans. At the 0.5 and 1.5 degree elevations VCP 121 is collects data
from the highest to the lowest PRF. For the batch scans above 1.5 degree elevation, the lowest PRF is collected first
followed by the highest and then the second highest PRF. At these elevations, the dual polarization fields are derived
from the first Doppler scan and the lowest PRF Doppler scan generally has more coverage than the higher PRF
scans.
The MPDA dealiasing scheme is a multi-step process that arrives at a final dealiased velocity solution at each radar
gate. Throughout the processing steps described below, “seed” velocities are used to check the gate-by-gate MPDA
results for consistency. These seeds can be single previously dealiased velocity solutions along the same radial, averages of previously dealiased gates, or estimates from the Environmental Wind Table (EWT).
4.3.5.3.1 Dealiasing Steps.
The main dealiasing steps are:
1. Solutions from Velocity Triplets (tight constraint). This first step considers only gates at which three velocity
estimates are present. The three estimates must be dealiased within a small velocity difference of each other and
within a threshold velocity difference of a seed velocity as previously defined for a final solution to be accepted.
In general, this processing accounts for about 57% of the final dealiased field.
2. Solutions from Velocity Triplets (relaxed constraint). The second step considers only gates at which three
velocity estimates are present. The three estimates and a seed velocity must be dealiased within a larger velocity
difference than step (1).
3. Solutions from Pairs within Triplets. The algorithm next attempts to dealias velocity pairs that did not pass
the dealiasing tests in steps a and b. The use of velocity pairs may be due to there being only two estimates at
a particular location in space or due to the failure to find a solution using triplets. In the case where triplets are
present three solutions are possible. However, the first pair that provides an acceptable solution is retained. Note
that at 4.3° elevation, where only two velocity cuts are obtained, MPDA processing begins with this step.
4. Solutions from Single Estimates. At this point the remaining solutions are derived from the single velocity estimates that exist within the unsolved triplets and pairs, and those locations in space that only contained
only one estimate. The single estimates are dealiased using seed values from the previous steps and increasingly
relaxed thresholds. Once this processing is complete, more than 99.99% of the gates contain a final dealiased
velocity value.
5. Use original velocity estimates. The remaining gates are assigned one of the three original velocity estimates
that are closest to an average of the surrounding dealiased gates.
4.3.5.3.2 Error Mitigation Schemes.
After each of the steps described in Section 4.3.5.3, error mitigation is applied to check for outliers and for azimuthal and radial inconsistencies in the dealiased field.
1. Despeckling. A despeckling function is applied after each processing step. This routine check for single velocity gates whose solutions differ significantly from surrounding gates. Several averages of surrounding gates are
checked against the gate in question. If the gate can be dealiased within a strict threshold of one of the averag4-20

FMH-11-Part C

es, it is assigned a value. Otherwise, it is set to missing.
2. Azimuthal Error Correction. This routine searches for runs of gates along radials that differ significantly
when compared to adjacent azimuthal values. Adjacent azimuths on both sides of the azimuth in question are
considered in the checking. If the azimuth in question can be dealiased within a strict threshold of its adjacent
azimuths, values are assigned to it. Otherwise, the gates on the radial are set to missing.
3. Radial Error Correction. This routine searches for large gate-to-gate jumps along radials. If a jump is encountered, an attempt is made to dealias it into the correct Nyquist interval based on averages of other radially adjacent gates within a strict threshold. If the gates in question can be dealiased, new values are assigned, otherwise
the gates are set to missing.
4.3.5.3.3 Operational Parameters.
• Threshold (Range Unfold Power Difference): Minimum power difference (in dB) between the first and second
trip echoes in order to assign unambiguously a range to velocities. (Default is 5 dB.) This parameter is used to
range unfold the second and third velocity fields for the MPDA and is entirely equivalent to TOVER used by
the RDA.
• Threshold (Fix Trip Minimum Bin) and Threshold (Fix Trip Maximum Bin): These two parameters define a
narrow annulus at the end of each trip of Doppler data the MPDA uses to clear out noisy velocity data. The
default value is -7 bins for the first parameter and -3 bins for the second parameter; range is +/- bins relative to
the last bin of each trip.
• Threshold (Tight Overlap Size): This parameter is applied to the first triplet solution (tight constraint). The
velocity estimates from all three PRFs must fall within +/- this parameter’s value.
• Threshold (Loose overlap Size): This parameter is applied to the second triplet solution (relaxed constraint). The
velocity estimates from all three PRFs must fall within +/- this parameter’s value.
4.3.5.4.3 Strengths/Applications. The MPDA has been increasingly used by operational sites to reduce “purple
haze,” especially in hurricane environments (Figure 4-4).


> **Figure 4-4: MPDA Velocity Product Comparison.**

> Velocity product produced using VCP 211 (left) compared with a product produced by the MPDA

Velocity product produced using VCP 211 (left) compared with a product produced by the MPDA
(right) for the same situation. Notice that the range folding, represented by the solid purple color, is
eliminated in the MPDA-produced velocities. The data shown is from the Oklahoma City, OK (KTLX)
WSR-88D during a widespread precipitation event on 30 December 2006 at 02:02 UTC (CVG Display).
4-21

October 2017

4.3.5.3.5 Limitations.
• In situations with very strong extensive echoes, range folding may be present in all input velocity fields despite
having different PRFs. Under those conditions, the MPDA will also have extensive range folding.
• In areas of coverage where there is only velocity data from the lowest PRF, the velocity data may be dealiased
incorrectly. This may be more pronounced between 230 and 300 km at low elevation angles.
• Because the MPDA requires two extra sweeps of Doppler data, the time between elevations is somewhat greater for its VCP than for normal precipitation mode VCPs.
• Algorithms that rely on vertical continuity may not perform as well with the MPDA because of the translation
and/or morphology of features.
• Because data are combined from three different sweeps, shear features near the radar may become distorted or
have weakened gradients. This could have a negative impact on the Tornado Detection Algorithm. Users should
exercise caution if the MPDA is used when there are tornadic storms near the radar.
4.3.5.4 Enhanced MPDA.
To enhance the performance of the MPDA algorithm, one of the additional velocity scans is collected using SZ-2
processing. Because the SZ-2 algorithm runs in the RDA’s processor and the MPDA runs in the RPG, the two techniques are readily combined. The SZ-2 algorithm range unfolds velocity data for the first Doppler scan on a radial-by-radial basis at the RDA which transfers each radial to the RPG. The extra Doppler scans are range unfolded
using legacy range unfolding logic. At the RPG the MPDA replaces the residual range- folded data in the SZ-2 Doppler scan using data from the extra scans and performs velocity dealiasing as described above. To match the MPDA
requirement of using PRF 8 for the first Doppler cut, the enhanced VCP 121 uses the specifications for VCP 211
which also uses PRF 8. To accommodate the SZ-2 processing, which requires 64 pulses (43 pulses are used in the
baseline VCP 121), the enhanced VCP 121 antenna rotation rate is slowed to the VCP 211 rate for the first Doppler
scan at both 0.5 and 1.45 degrees elevation. Also, the number of surveillance pulses was increased from 15 to 17 to
match VCP 211. The slower rotation rates add about fifteen seconds to the total volume scan time for the enhanced
VCP 121 for an estimated completion time of 5 min 45 sec.

#### 4.3.6 Supplemental Adaptive Intra-Volume Low-Level Scan.

Supplemental Adaptive Intra-Volume Low-Level Scan (SAILS) inserts one supplemental split cutNote 3,4 scan
(lowest elevation defined in the VCP definition, normally 0.5°) into the existing severe weather VCPs 12 and 212.
This new split cut scan is inserted into the “middle” of the volume scan to evenly space, as close as possible, the
time intervals between low-level data updates. The “middle” of the volume scan is adaptive and determined on a
volume scan-to-volume scan basis based on the termination angle determined by the Automated Volume Scan Evaluation and Termination (AVSET) function.
Note3: Split Cut is a term used to describe the technique of scanning a particular elevation two or more times, using
a different PRF for each full scan. This technique is used to accurately place targets in range using a low PRF and to
collect accurate velocity data using a high PRF.
Note4: For sites with site-specific scanning strategies, the lowest elevation angle may be other than 0.5°.
Split Cut data collection was chosen because it provides the required number of samples to ensure effective clutter
filtering, provides range unambiguous reflectivity data which is the basis to range unfold velocity data and supports
Super Resolution data processing. Using Split Cut scanning to collect the supplemental low-level scan adds approximately 35 seconds to the volume scan duration. This results in VCP 12 completion times of approximately 285 seconds when AVSET does not terminate the VCP early and a completion time of approximately 225 seconds when
AVSET terminates the volume coverage pattern after collection of the 6.4° scan.
SAILS is only available for execution with VCPs that allow SAILS, as determined by a flag set in the VCP definition.
“Insert Elevations” are determined as follows:

4-22

FMH-11-Part C

1. Initial Insertion Elevation Point (Assumption: The termination elevation of this volume scan will be the same as
the previous volume scan.)
a. Estimate the volume scan duration of the previous volume scan. Total volume scan duration is the scanning
time of each completed elevation scan (360° divided by the scan rate (identified in the VCP definition) +
0.5, then truncated to the whole second).
b. Calculate the SAILS volume scan duration. Add the previous volume scan duration and the supplemental
split cut scan duration (for VCP 12, the 0.5 split cut duration is ~ 31 secs (17 sec for the Surveillance scan
and 14 sec for the Doppler scan). This is the SAILS scan duration.
c. Determine the midpoint target of the calculated SAILS volume scan duration. (SAILS scan duration divided
by 2).
d. Calculate running cumulative volume scan duration for each elevation of the baseline VCP (without the
SAILS split cut).
e. Compare the running cumulative volume scan duration for each elevation to the volume scan midpoint target value. Insert the supplemental scan above the elevation that has the smallest Delta between the running
cumulative scan duration and the volume scan midpoint target.
f. Download this new VCP definition to the RDA.
2. Final Insertion Elevation Point. At the beginning of the last elevation of the current volume scan, the SAILS
function compares the current termination elevation angle to the previous volume scan termination angle.
a. If the termination angle of the current volume scan is the same as the termination angle of the previous
volume scan then the Initial Insertion Elevation Point equals the Final Insertion Elevation Point. No further
action is required.
b. If the termination angle of the current volume scan is NOT the same as the termination angle of the previous volume scan then the SAILS function accomplishes Steps 5.i.1 through 5.i.6 (above).
3. Table 4-1 presents an example of the SAILS insertion angle verses termination angle for VCP 12.

4-23

October 2017


> **Table 4-1: SAILS Insert Elevation vs Termination Angle**

> SAILS 0.5° Elevation Insertion Point Based on VCP Termination Angle

SAILS 0.5° Elevation Insertion Point Based on VCP Termination Angle
Elevation
Angles (VCP
12)

VCP 12
Elevation
Duration

Standard
Termination
Angle
= 19.5

AVSET

AVSET

AVSET

AVSET

AVSET

Termination
Angle

Termination
Angle

Termination
Angle

Termination
Angle

Termination
Angle

= 12.5
31 Sec
31 Sec
31 Sec
15 Sec

= 10.0
31 Sec
31 Sec
31 Sec
15 Sec

14 Sec
31 Sec
14 Sec

14 Sec
31 Sec
14 Sec

14 Sec
14 Sec
14 Sec
13 Sec
13 Sec
13 Sec

0.5°
0.9°
1.3°
1.8°
0.5°
2.4°
0.5°
3.1°
0.5°
4.0°
5.1°
6.4°
8.0°
10.0°
12.5°
15.6°
19.5°

31 Sec
31 Sec
31 Sec
15 Sec

31 Sec
31 Sec
31 Sec
15 Sec

= 15.6
31 Sec
31 Sec
31 Sec
15 Sec

= 8.0
31 Sec
31 Sec
31 Sec
15 Sec
31 Sec
14 Sec

= 6.4
31 Sec
31 Sec
31 Sec
15 Sec
31 Sec
14 Sec

14 Sec

14 Sec

14 Sec

14 Sec
14 Sec
14 Sec
14 Sec
13 Sec
13 Sec
13 Sec
13 Sec
13 Sec

14 Sec
31 Sec
14 Sec
14 Sec
14 Sec
13 Sec
13 Sec
13 Sec
13 Sec
13 Sec

14 Sec
31 Sec
14 Sec
14 Sec
14 Sec
13 Sec
13 Sec
13 Sec
13 Sec

14 Sec

14 Sec

14 Sec
14 Sec
14 Sec
13 Sec
13 Sec

14 Sec
14 Sec
14 Sec
13 Sec

14 Sec
14 Sec
14 Sec

Duration

243 Sec

274 Sec

261 Sec

248 Sec

235 Sec

222 Sec

209 Sec

0.5° Elevation
Update Times
(sec)

243*

136 and
138*

136 and
125*

122 and
126*

122 and
113*

108 and
114*

108 and
101*

Note: These times are estimates. Times will vary slightly based on individual radar performance.
* Plus Retrace Time (approximately 10 seconds to return antenna from highest to lowest tilt end to perform
calibration functions.
The SAILS supplemental scan uses the same antenna rotation rates and data acquisition schemes as defined for the
baseline VCP in which it is used. Therefore, the supplemental scan data are processed using the same moment estimation methods and data processing techniques as the standard low-level data for that VCP.
References
Jing, Zhongqi, Wiener, Gerry (1993). “Two-Dimensional Dealiasing of Doppler Velocities”. Journal of Atmopheric
and Oceanic Technology, 10, pp 798 – 808.
Siggia, A., and R. E. Passarelli, Jr., 2004: Gaussian model adaptive processing (GMAP) for improved ground clutter
cancelation and moment estimation. Preprints, Third European Conf. on Radar in Meteorology and Hydrology, Visby, Sweden, Copernicus Gesellschaft, 67-73.
Witt, Arthur, Brown, Rodger A., Jing, Zhongqi (2009). “Performance of a New Velocity Dealiasing Algorithm for
the WSR-88D”. 34th Conference on Radar Meteorology, Williamsburg, VA, American Meteorological Society. P4.8.
http://ams.confex.com /ams/34Radar/techprogram/paper_155951.htm

4-24

FMH-11-Part C

4-25

October 2017

4-26

Chapter 5: Operational Modes And Volume Coverage Patterns

### 5.1 Introduction.

This chapter describes the WSR-88D operational modes and volume coverage patterns (VCPs). There are two operational modes: Precipitation and Clear Air. Each operational mode uses multiple VCPs.

### 5.2 Operational Modes.


The WSR-88D initiates data acquisition and processing in either Precipitation Mode or Clear Air Mode, based on operator settings. Selecting
various adaptable parameters, the system can be set up to automatically switch between modes based on areal coverage and strength of echoes. Either
mode can also be manually selected.


#### 5.2.1 Precipitation Mode.

This mode is used when enough significant echoes are present. It may be entered automatically, when the detected
echoes meet coverage and strength thresholds, or it may be entered manually. The initial operating capability included VCPs 11 and 21 (Sections 5.3.1.1 and 5.3.2) for this mode. However, in 2005 VCPs 12 and 121 (Sections 5.3.1.2
and 5.3.3.1) were added followed by VCPs 211, 212, and 221 (Sections 5.3.3.2, 5.3.3.3, and 5.3.3.4) in 2007. With
these new VCPs, there are now three sub-classifications of the Precipitation Mode. Those are the Shallow Precipitation Group (VCP 21), Deep Convection Group (VCPs 11 and 12), and the Multiple PRF Dealiasing Algorithm
(MPDA) and Sachidananda-Zrnic (SZ)-2 Group (VCPs 121, 211, 212, and 221).

#### 5.2.2 Clear Air Mode.

This mode is used when there is no detectable precipitation or when precipitation intensity and aerial extent have
not exceeded the respective thresholds. The initial operating capability included VCPs 31 and 32 (Sections 5.3.4.1
and 5.3.4.2) for this mode. The two VCPs have identical elevation angles but vary in the radar pulse width and the
pulse repetition time. Long pulse (VCP 31) can be used routinely to provide maximum signal-to-noise ratios and
resultant sensitivity. Short pulse (VCP 32) can be used when a greater Nyquist velocity is desired for this mode.

### 5.3 Volume Coverage Patterns.

During normal operations, the antenna is controlled by automatic scanning strategies. The VCPs are matched to
an operational mode to optimize product generation for given meteorological situations. In each operational mode,
scanning is continuous to support the needs of the users. Principal Users are informed of the current radar operational mode and the current VCP through the use of General Status Messages (GSMs).
In 2005 a VCP numbering convention was adopted and new VCPs were added, followed by the addition of new
VCPs in 2007. To accommodate the new and legacy VCPs, the following five groups have been identified.
• Deep Convection Group: Two digits beginning with 1, VCPs 11 and 12.
• Shallow Precipitation Group: Two digits beginning with 2, VCP 21.
• Multiple PRF Dealiasing Algorithm and Sachidananda-Zrnic Group: Three digits beginning with 1 or 2, VCPs
121, 211, 212, and 221.
• Clear Air Group: Two digits beginning with 3, VCPs 31 and 32.
• Site-Specific VCPs Group: Previously defined VCP numbers.
Within the definition of each VCP, the WSR-88D employs three unique collection techniques to maximize the
amount and quality of the data while minimizing the time it takes to collect the data. The following paragraphs describe these data collection techniques.
• Split Cut is the term used to describe the technique of scanning a particular elevation slice two or more times,
using a different PRF for each full scan. This technique is used to accurately place targets in range using a low
PRF and to collect accurate velocity data using a high PRF. For the lowest elevations scans where efficient clutter suppression is required and velocity range folding is likely, all VCPs employ the Split Cut technique using a
5-1

October 2017

Contiguous Surveillance (CS) scan followed by one or more Contiguous Doppler (CD) scans.
▪ Contiguous Surveillance (CS) is a constant low PRF (long Rmax) employed for the entire 360° scan to
determine proper target location and intensity (dBZ). Due to the long Rmax, no range unfolding technique is
applied since all target locations are considered to be unambiguous or correct. CS operations are part of the
“Split Cut” mode.
▪ Contiguous Doppler (CD) is a constant high PRF (short Rmax and high Vmax ) employed for the entire
360° scan or multiple high PRFs employed in operator- defined sectors to accurately determine “1st guess”
velocity and spectrum width information. A result of the high PRF (short Rmax) is that multiple trip echoes
can occur and, therefore, a range unfolding technique using data from the CS (or Batch) scan must be applied.
CD operations are also part of the “Split Cut” mode.
• Batch Mode (B) is used in the middle angles of most VCPs. The Batch Mode (B) technique uses alternating
low and high PRFs on each radial for one full rotation at each elevation angle. Along each radial, the radar starts
transmitting pulses using a low PRF (long Rmax) to obtain target intensity and location information. Then,
before the antenna completes a 1° sweep, the transmitter quickly switches to high PRF (high Vmax) to obtain
more accurate velocity information. This alternating back and forth between a low and a high PRF is done for
each radial until a full 360° scan has been completed. The two data sets resulting from the different PRFs are
combined to resolve range ambiguity. The Batch Mode is used at elevation slices between 1.8° and 6.5° in VCPs
11, 12, 21, 211, 212, 221 and 32 (not employed in VCP 31) where ground clutter contamination is generally not
a problem. Additionally, VCP 121 employs the Batch Mode as part of the “Split Cut” data collection for middle
elevation slices 2.4°, 3.4° and 4.3°, and it is used alone at the elevation slice of 6.0°.
• Contiguous Doppler X (CDX) (or Contiguous Doppler with no range unfolding) combines a high PRF and a
rapid antenna rotation rate to obtain all base data in the higher elevation slices (>7°). Even though a high PRF is
used, no range-unfolding algorithm is applied to the data. This is because at these higher elevation angles range
folded echoes are highly unlikely. (For example, at 7.5°, the radar beam is already at ~ 50,000 feet at 62 nm
range (the shortest CD Rmax)). CDX is employed at all elevation slices >7° in VCPs 11, 12, 21, 121, 211, 212,
and 221 and above 3° in VCP 31 (CDX is not employed in VCP 32).
WSR-88D VCPs are designed to automatically and continuously scan predefined elevation slices, from low to high
angles, regardless of the areal coverage or significance of the return present on those elevations. However, with
newer radar technology, the operator can opt to “modify” VCPs using two radar functions.
• Automated Volume Scan Evaluation and Termination (AVSET) is intended to provide faster volume scan
updates in rapidly evolving meteorological conditions. The algorithm does not operate below 5° so it is not active while in Clear Air Mode. The basic premise of the AVSET function is to terminate the current volume scan
after the radar has scanned all elevations with significant return. Once the elevation angle overshoots available
radar return, the volume scan is terminated because there is no operational benefit realized by continuing the
current scan; a new volume scan is then begun. The net effect of the AVSET function is that low elevations are
scanned more frequently, without impacting the quality or accuracy of the base data estimates. The WSR-88D
operator can command AVSET on or off. See Section 4.2.11 for more information.
• Supplemental Adaptive Intra-Volume Low-Level Scan (SAILS) inserts one supplemental Split Cut scan
(lowest elevation defined in the VCP definition, normally 0.5°) into the existing severe weather VCPs 12 and
212. This supplemental Split Cut scan is inserted into the “middle” of the volume scan to evenly space, as close
as possible, the time intervals between low-level data updates. The “middle” of the volume scan is adaptive and
determined on a volume scan-to-volume scan basis based on the termination angle determined by the AVSET
function. SAILS can provide almost twice as many low-level base products for a given period than the current
severe weather VCPs 12 and 212 without impacting the quality of base data estimates. See Section 4.3.5 for
more information.
These functions are independent and may be executed together or separately. Please note that the interaction of
AVSET and SAILS may drastically change the VCP characteristics defined in the following sections. For example,
the standard VCP 12 scans 0.5° every ~250 seconds. If AVSETis enabled, 0.5° may be scanned every ~185 to ~250
5-2

FMH-11-Part C

seconds. With AVSET and SAILS enabled, 0.5° may be scanned every ~112 to ~142 seconds.

#### 5.3.1 Deep Convection Group, VCPs 11 and 12.

The purposes of VCPs 11 and 12 are to: 1) detect and track storms; 2) detect shear, mesocyclones, and other hazardous types of weather associated with deep moist convection; 3) provide precipitation estimates; and 4) obtain
wind profiles to supplement soundings.
5.3.1.1 VCP 11.
VCP 11 is designed to sample severe and non-severe precipitation events. VCP 11 scans 14 elevation cuts in 5 minutes using Split Cut (CS and CD) for the 0.5° and 1.5° elevation scans, Batch (B) processing from 2.4° through 6.2°,
and Contiguous Doppler (CDX) for 7.5° through 19.5°. (See Table 5-1 and Figure 5-1.)
5.3.1.1.1 Parameters.
• Short pulse (1.57 µs; PRF 318 to 1304 Hz)
• Unambiguous (Nyquist) velocity range 8 to 32.8 m/s (15.5 to 63.7 knots)
• Fourteen elevation angles
• 5-minute update rate
• Velocity data within 300 km (162 nm) radius
• Hydrometeorological data within 230 km (124 nm) radius
• Reflectivity data within 460 km (248 nm) radius
• Separate Surveillance and Doppler scans are taken at the two lowest elevation angles to improve clutter filter
performance, maximize the velocity accuracy and unambiguous interval
• Lowest seven elevation angles are contiguous
5.3.1.1.2 Strengths/Applications.
• More rapid update rate than VCP 21 (6 minutes)
• More elevation scans than VCP 21, enhancing both storm evaluation and algorithm output
• Better for monitoring convection than VCP 21
• Slightly better accuracy than VCP 12
5.3.1.1.3 Limitations.
• Slower update rate than VCP 12 (4.2 minutes)
• Lacks the low-level overlapping beams of VCP 12
• More widespread overlaid echo and velocity dealiasing failures as compared to VCP 121
• Slightly decreased accuracy of the base data estimates as compared to VCP 21

5-3

October 2017


> **Table 5-1: VCP 11 Characteristics**

> Scan

Scan

Surveillance

Angle

AZ Rate

Period

WF

PRF

(°)

(°/sec)

(sec)

Type

No.

Doppler PRF No.

#
Pulses






# Pulse

#
Pulse

#
Pulse

# Pulse

#
Pulse

0.5

18.68

19.38

CS



–

–

–

–

–

0.5

19.23

18.83

CD

–

–






1.45

19.84

18.25

CS



–

–

–

–

–

1.45

19.23

18.83

CD

–

–






2.4

16.12

22.46

B








3.4

17.90

20.23

B








4.3

17.90

20.23

B








5.3

17.46

20.74

B








6.2

17.47

20.72

B








7.5

25.17

14.38

CDX

–

–






8.7

25.40

14.25

CDX

–

–






10.0

25.42

14.24

CDX

–

–






12.0

25.47

14.22

CDX

–

–






14.0

25.51

14.19

CDX

–

–






16.7

25.60

14.14

CDX

–

–






19.5

25.70

14.09

CDX

–

–






Notes:
• Default Doppler PRF numbers are underlined and bold; Doppler PRFs are editable <7.0 degrees
• Sum of periods, which is “data collection” time = 279.17 secs / 4.65 mins, transition times will vary
• Volume scan update time is about 5 minutes
• See Table 5-10 for PRF No. information

5-4

FMH-11-Part C


> **Figure 5-1: VCP 11.**

> VCP 11 samples fourteen elevation angles with sixteen antenna rotations in 5 minutes. The lowest

VCP 11 samples fourteen elevation angles with sixteen antenna rotations in 5 minutes. The lowest
two angles use Split Cut (CS/CD), middle angles use Batch (B), and higher angles use Contiguous
Doppler (CDX) mode. The lines representing the beam elevation with height as a function of range
assume standard atmospheric refraction of the beam.
5.3.1.2 VCP 12.
VCP 12 is designed for deep convection, with better vertical resolution and faster volume scan update than VCP 11.
VCP 12 scans 14 elevation cuts in 4.2 minutes using both Split Cut (CD and CS) for the 0.5°, 0.9° and 1.3° elevation
scans, Batch (B) processing from 1.8° through 6.4°, and Contiguous Doppler (CDX) for 8.0° through 19.5°. (See

> **Table 5-2 and Figure 5-2.)**

> 5.3.1.2.1 Parameters.

5.3.1.2.1 Parameters.
• Short pulse (1.57 µs; PRF 318 to 1304 Hz)
• Unambiguous (Nyquist) velocity range 8 to 32.8 m/s (15.5 to 63.7 knots)
• Fourteen elevation angles
• 4.2-minute update rate
• Velocity data within 300 km (162 nm) radius
• Hydrometeorological data within 230 km (124 nm) radius
• Reflectivity data within 460 km (248 nm) radius
• Separate Surveillance and Doppler scans are taken at the three lowest elevation angles to improve clutter filtering performance, maximize the velocity accuracy and unambiguous interval
• Lowest seven elevation angles are contiguous
5.3.1.1.2 Strengths/Advantages.
• Algorithms that depend on low-level data perform better with VCP 12
5-5

October 2017

•
•
•
•
•
•

Provides more data in the lowest levels of the atmosphere by increasing the low-level vertical resolution
Fastest update rate of current VCPs
Provides better vertical definition of storm structure
Better rainfall and snowfall estimates than for VCP 11 or VCP 21 due to increased low-level scanning
Improved storm characterization by algorithms
Six elevation scans below 4° (overlapping beam sampling)

5.3.1.2.3 Limitations.
• Faster antenna rotation rates, especially above the lowest three elevation angles, slightly degrades velocity and
reflectivity estimates as compared to VCP 11 due to fewer pulses in the sample estimate
• Product availability is limited for some display systems due to communications bandwidth limitations

> **Table 5-2: VCP 12 Characteristics**

> Scan

Scan

Surveillance

Angle

AZ
Rate

Period

WF

PRF

(°)

(°/sec)

(sec)

Type

No.

Doppler PRF No.

#
Pulses






#
Pulses

#
Pulses

#
Pulses

#
Pulses

#
Pulses

0.5

21.15

17.12

CS



–

–

–

–

–

0.5

24.99

14.48

CD

–

–






0.9

21.15

17.12

CS



–

–

–

–

–

0.9

24.99

14.48

CD

–

–






1.3

21.15

17.12

CS



–

–

–

–

–

1.3

24.99

14.48

CD

–

–






1.8

24.64

14.69

B








2.4

26.40

13.71

B








3.1

26.40

13.71

B








4.0

26.40

13.71

B








5.1

28.00

12.93

B








6.4

28.00

12.93

B








8.0

28.40

12.75

CDX

–

–






10.0

28.88

12.53

CDX

–

–






12.5

28.74

12.60

CDX

–

–






15.6

28.74

12.60

CDX

–

–






19.5

28.74

12.60

CDX

–

–






Notes:
• Default Doppler PRF numbers are underlined and bold; Doppler PRFs are editable <7.0 degrees
• Sum of periods, which is “data collection” time = 239.55secs / 3.99 mins, transition times will vary
• Volume scan update time is about 4.2 minutes
• See Table 5-10 for PRF No. information

5-6

FMH-11-Part C


> **Figure 5-2: VCP 12.**

> VCP 12 samples fourteen elevation angles with seventeen antenna rotations in 4.2 minutes. The

VCP 12 samples fourteen elevation angles with seventeen antenna rotations in 4.2 minutes. The
lowest three angles use Split Cut (CS/CD), middle angles use Batch (B), and higher angles use Contiguous Doppler (CDX) mode. The lines representing the beam elevation with height as a function of
range assume standard atmospheric refraction of the beam.

#### 5.3.2 Shallow Precipitation Group, VCP 21.

The purpose of VCP 21 is to optimize the volume- sampling interval for changing radar echo patterns, but its
primary application is for shallow precipitation. VCP 21 scans nine elevation cuts in six minutes. (See Table 5-3 and

> **Figure 5-3). The contiguous scans from about 4.3° downward and the slower update rate are appropriate for shallow**

> and slowly changing precipitation situations. If deep convection develops with significant storms, the user may wish

and slowly changing precipitation situations. If deep convection develops with significant storms, the user may wish
to switch from VCP 21 to VCP 11 or VCP 12.
5.3.2.1 Parameters.
• Short pulse (1.57 µs PRF 318 to 1304 Hz)
• Unambiguous (Nyquist) velocity range 8 to 32.8 m/s (15.5 to 63.7 knots)
• Nine elevation angles
• 6-minute update rate
• Velocity data within 300 km (162 nm) radius
• Hydrometeorological data within 230 km (124 nm) radius
• Reflectivity data within 460 km (248 nm) radius
• Separate Surveillance and Doppler scans are taken at the two lowest elevation angles to improve clutter filtering
performance, maximize the velocity accuracy and unambiguous interval
• Lowest five elevation angles are contiguous
5-7

October 2017

5.3.2.2 Strengths/Applications. Slightly higher accuracy of estimates for reflectivity and velocity because of a larger number of pulses in the sample estimate
5.3.2.3 Limitations.
• Coarse vertical resolution above the fifth elevation scan
• Degraded performance of algorithms that depend on volumetric information, relative to VCPs 11 and 12
• Pronounced banding of legacy Enhanced Echo Tops and high-layer legacy products
• Small-scale phenomena such as storm initiation may not be detected at the earliest opportunity

> **Table 5-3: VCP 21 Characteristics**

> Scan

Scan

Surveillance

Angle

AZ Rate

Period

WF

PRF

(°)

(°/sec)

(sec)

Type

No.

0.5
0.5
1.45
1.45
2.4
3.4
4.3
6.0
9.9
14.6
19.5

11.34
11.36
11.34
11.36
11.18
11.18
11.18
11.18
14.26
14.33
14.41

31.93
31.87
31.93
31.87
32.37
32.37
32.37
32.37
25.39
25.27
25.11

CS
CD
CS
CD
B
B
B
B
CDX
CDX
CDX

–
–
–
–
–

Doppler PRF No.





#
Pulses

#
Pulse

#
Pulse

#
Pulse

#
Pulse

#
Pulse

–
–
–
–
–

–
–

–
–

–
–

–
–

–
–

Notes:
• Default Doppler PRF numbers are underlined and bold; Doppler PRFs are editable <7.0 degrees
• Sum of periods, which is “data collection” time = 332.80 secs / 5.55 mins, transition times will vary
• Volume scan update time is about 6 minutes
• See Table 5-10 for PRF No. information

5-8

FMH-11-Part C


> **Figure 5-3:VCP 21.**

> VCP 21 samples nine elevation angles with eleven antenna rotations in about 6 minutes. The lowest

VCP 21 samples nine elevation angles with eleven antenna rotations in about 6 minutes. The lowest
two angles use Split Cut (CS/CD), middle angles use Batch (B), and higher angles use Contiguous
Doppler (CDX) mode. The lines representing the beam elevation with height as a function of range
assume standard atmospheric refraction of the beam.

#### 5.3.3 Multiple PRF Dealiasing Algorithm and Sachidananda-Zrnic Group, VCPs 121, 211, 212, and 221.

The purpose of VCPs 121, 211, 212, and 221 is to reduce range-obscured velocity and spectrum width data when
compared to other VCPs. Only VCP 121 uses the Multiple PRF Dealiasing Algorithm (MPDA) (Section 4.3.5.3)
while all four of these VCPs use Sachidananda- Zrnic (SZ)-2 processing (Section 4.2.10) to range unfold velocity
data on Split Cuts. Specific usages of each VCP can be found in the subsequent sections.
5.3.3.1 VCP 121.
The purpose of VCP 121 is to provide rapid volume-sampling updates to monitor changing radar echo patterns
while at the same time significantly reducing the amount of range folded (50-70% reduction) and incorrectly
dealiased velocity data. This VCP is intended to be a short-term solution to the problem of both range and velocity
folding.
VCP 121 is the MPDA version of VCP 21 (Section 5.3.2). To accomplish the dealiasing goal, VCP 121 uses multiple
Contiguous Doppler (CD) rotations at the lower elevation scans. For example, for the 0.5° and 1.5° elevations, the
Split Cut technique includes one Contiguous Surveillance (CS) scan and three CD scans (PRFs 8, 6 and 4). PRF 8
is used first in conjunction with the SZ-2 algorithm for range unfolding at the RDA. Then, data from PRFs 6 and
4 are used at the RPG for additional range unfolding and velocity dealiasing. Similarly, three PRFs (4, 8 and 6) are
used at 2.4° and 3.3° to range unfold and velocity dealias the data at the RPG while two PRFs (4 and 7) are used at
4.3°.
5-9

October 2017

VCP 121 can be effective in most meteorological situations. However, due to the increased scanning time required
to complete the lower elevations with respect to VCP 12, VCP 121 may not be appropriate for fast moving or rapidly evolving convective storms. VCP 121 is especially useful for widespread precipitation and high wind velocities
(e.g., tropical cyclones). (See Table 5-4 and Figure 5-4.)
5.3.3.1.1 Parameters.
• Short pulse (1.57 µs PRF 318 to 1304 Hz)
• Unambiguous (Nyquist) velocity range 8 to 32.8 m/s (15.5 to 63.7 knots)
• Nine elevation angles
• 5.75-minute update rate
• Velocity data within 300 km (162 nm) radius
• Hydrometeorological data within 230 km (124 nm) radius
• Reflectivity data within 460 km (248 nm) radius
• Separate Surveillance and Doppler scans are taken at the five lowest elevation angles to improve clutter filtering
performance, maximize the velocity accuracy and unambiguous interval
• Lowest five elevation angles are contiguous
5.3.3.1.2 Strengths/Applications.
• Provides base data with far less range overlaid echoes and velocity aliased data
• Recommended for sampling widespread non-severe echo coverage and hurricanes and other tropical systems
while still offshore when large scale structure is the priority
5.3.3.1.3 Limitations.
• Lower accuracy of estimates (based on fewer samples) as compared to VCP 21
• Should be used with caution for rapidly changing storms or deep convective storms and only when extensive
range folding is a problem

5-10

FMH-11-Part C


> **Table 5-4: VCP 121 Characteristics**

> Scan

Scan

Surveillance

Doppler PRF No.

AZ
Angle

Rate

Period

WF

PRF

(°)

(°/sec)

(sec)

Type

No.

#
Pulses






#
Pulse

#
Pulse

#
Pulse

#
Pulse

#
Pulse

0.5

18.68

19.38


0.5

19.75

18.33

0.5
0.5
1.5

27.40
21.40
19.84

13.21
16.92
18.24

1.5

19.75

18.33

1.5
1.5
2.4
2.4
2.4
3.4
3.4
3.4
4.3
4.3
6.0
9.9
14.6
19.5

27.40
21.40
16.30
29.05
27.40
16.30
29.05
27.40
16.30
29.50
20.20
29.50
29.80
29.80

13.21
16.92
22.20
12.46
13.21
22.20
12.46
13.21
22.20
12.27
17.92
12.27
12.15
12.15

CS/
SZ-2
CD/
SZ-2
CD
CD
CS/
SZ-2
CD/
SZ-2
CD
CD
B
CDX
CDX
B
CDX
CDX
B
CDX
B
CDX
CDX
CDX


–

–

–

–

–

–

–






–
–

–
–

–

–

–

–

–

–

–






–
–
–
–
–
–
–
–
–
–

–
–
–
–
–
–
–
–
–
–






Notes:
• Default Doppler PRF numbers used for each rotation are underlined and bold; Doppler PRFs are not editable
• Sum of periods, which is “data collection” time = 319.25 secs / 5.32 mins, transition times will vary
• Volume scan update time is about 5.75 minutes
• See Table 5-10 for PRF No. information
• VCP 121 utilizes the VCP 21 elevation angles with the MPDA and SZ-2

5-11

October 2017


> **Figure 5-4: VCP 121.**

> VCP 121 samples nine elevation angles with twenty antenna rotations in 5 minutes. The lowest five

VCP 121 samples nine elevation angles with twenty antenna rotations in 5 minutes. The lowest five
angles apply the MPDA technique, while 6.0° uses only Batch (B), and higher angles use Contiguous
Doppler (CDX) mode. The lines representing the beam elevation with height as a function of range
assume standard atmospheric refraction of the beam.
5.3.3.2 VCP 211.
The purpose of VCP 211 is the same as that described for VCP 11 in Section 5.3.1.1. In addition, this VCP provides significantly reduced range-obscured velocity and spectrum width data when compared to VCP 11. (See Table
5-5 and Figure 5-5.)
5.3.3.2.1 Parameters.
• Short pulse (1.57 µs; PRF 318 to 1304 Hz)
• Unambiguous (Nyquist) velocity range 8 to 32.8 m/s (15.5 to 63.7 knots)
• Fourteen elevation angles
• 5-minute update rate
• Velocity data within 300 km (162 nm) radius
• Hydrometeorology data within 230 km (124 nm) radius
• Reflectivity data within 460 km (248 nm) radius
• Separate Surveillance and Doppler data are taken at the two lowest elevation angles to improve clutter filtering
performance, maximize the velocity accuracy and unambiguous interval
• Lowest seven elevation angles are contiguous
5.3.3.2.2 Strengths/Advantages.
• Reduced range-folded data as compared to VCP 11
5-12

FMH-11-Part C

•
•
•
•

More rapid update rate than VCP 221 (6 minutes)
More elevation cuts than VCP 221 allowing better operator storm evaluation and producing better algorithm
output
Better for monitoring convection than VCP 221
Provides better vertical definition of storm structure than VCP 221

5.3.3.2.3 Limitations.
• Slower update rate than VCP 212 (4.2 minutes)
• Lacks the low-level overlapping beams of VCP 212
• More widespread overlaid echo and velocity dealiasing failures as compared to VCP 121
• Stored only at RPG and not at RDA, it must be downloaded to RDA

> **Table 5-5: VCP 211 Characteristics**

> Scan

Scan

Surveillance

Doppler PRF No.

AZ
Angle

Rate

Period

WF

PRF

(°)

(°/sec)

(sec)

Type

No.

0.5

18.68

19.38

0.5

19.75

18.33

1.5

19.84

18.25

1.5

19.75

18.33

2.4
3.4
4.3
5.3
6.2
7.5
8.7
10.0
12.0
14.0
16.7
19.5

16.12
17.90
17.90
17.46
17.47
25.17
25.40
25.42
25.47
25.51
25.60
25.70

22.46
20.23
20.23
20.74
20.72
14.38
14.25
14.24
14.22
14.19
14.14
14.09

CS/
SZ-2
CD/
SZ-2
CS/
SZ-2
CD/
SZ-2
B
B
B
B
B
CDX
CDX
CDX
CDX
CDX
CDX
CDX

#
Pulses






#
Pulse

#
Pulse

#
Pulse

#
Pulse

#
Pulse



–

–

–

–

–

–

–








–

–

–

–

–

–

–






–
–
–
–
–
–
–

–
–
–
–
–
–
–






Notes:
• Default Doppler PRFs are underlined and bold; Doppler PRFs are editable for <7.0 degrees, since mid-2014.
Prior to 2014 Doppler PRFs are not editable for SZ-2 cuts
• Sum of periods, which is “data collection” time = 278.16 secs / 4.64 mins, transition times will vary
• Volume scan update time is about 5 minutes
• See Table 5-10 for PRF No. information
• VCP 211 has the VCP 11 elevation angles with SZ-2 applied to Split Cuts
5-13

October 2017


> **Figure 5-5: VCP 211.**

> VCP 211 samples fourteen elevation angles with sixteen antenna rotations in 5 minutes. The lowest

VCP 211 samples fourteen elevation angles with sixteen antenna rotations in 5 minutes. The lowest
two angles use Split Cut (CS/CD) and SZ-2, middle angles use Batch (B), and higher angles use
Contiguous Doppler (CDX) mode. The lines representing the beam elevation with height as a function
of range assume standard atmospheric refraction of the beam.
5.3.3.3 VCP 212.
The purpose of VCP 212 is the same as that described for VCP 12 in Section 5.3.1.2. In addition, this VCP provides significantly reduced range-obscured velocity and spectrum width data when compared to VCP 12. (See Table 5-6 and Figure 5-6.)
5.3.3.3.1 Parameters.
• Short pulse (1.57 µs; PRF 318 to 1304 Hz)
• Unambiguous (Nyquist) velocity range 8 to 32.8 m/s (15.5 to 63.7 knots)
• Fourteen elevation angles
• 4.75-minute update rate
• Velocity data within 300 km (162 nm) radius
• Hydrometeorology data within 230 km (124 nm) radius
• Reflectivity data within 460 km (248 nm) radius
• Separate Surveillance and Doppler data are taken at the three lowest elevation angles to improve clutter filtering performance, maximize the velocity accuracy and unambiguous interval
• Lowest seven elevation angles are contiguous
5.3.3.3.2 Strengths/Advantages.
• Reduced range-folded data as compared to VCP 12
• Algorithms that depend on low-level data perform better with VCP 212
• Provides more data in the lowest levels of the atmosphere by increasing the low-level vertical resolution
• Fast update rate
5-14

FMH-11-Part C
•
•
•
•

Provides better vertical definition of storm structure
Better rainfall and snowfall estimates than for VCP 211 or VCP 221 due to increased low-level scanning
Improved storm characterization by algorithms
Six elevation scans below 4° (overlapping beam sampling)

5.3.3.3.3 Limitations.
• Faster antenna rotation rates, especially above the lowest three elevation angles, slightly degrades velocity and reflectivity
estimates as compared to VCP 211 due to fewer pulses in the sample estimate
• Stored only at RPG and not at RDA, it must be downloaded to RDA
• Product availability is limited for some display systems due to communications bandwidth limitations


> **Table 5-6: VCP 212 Characteristics**

> Scan

Scan

Surveillance

Doppler PRF No.

AZ
Angle

Rate

Period

WF

PRF

(°)

(°/sec)

(sec)

Type

No.

#
Pulses






#
Pulse

#
Pulse

#
Pulse

#
Pulse

#
Pulse

0.5

21.15

17.12


0.5

16.90

21.42

0.9

21.15

17.12

0.9

16.90

21.42

1.3

21.15

17.12

1.3

16.90

21.42

1.8
2.4
3.1
4.0
5.1
6.4
8.0
10.0
12.5
15.6
19.5

24.64
26.40
26.40
26.40
28.00
28.00
28.40
28.88
28.74
28.74
28.74

14.69
13.71
13.71
13.71
12.93
12.93
12.75
12.53
12.60
12.60
12.60

CS/
SZ-2
CD/
SZ-2
CS/
SZ-2
CD/
SZ-2
CS/
SZ-2
CD/
SZ-2
B
B
B
B
B
B
CDX
CDX
CDX
CDX
CDX


–

–

–

–

–

–

–








–

–

–

–

–

–

–








–

–

–

–

–

–

–






–
–
–
–
–

–
–
–
–
–






Notes:
• Default Doppler PRFs are underlined and bold; Doppler PRFs are editable for <7.0 degrees, since mid-2014.
Prior to 2014 Doppler PRFs are not editable for SZ-2 cuts
• Sum of periods, which is “data collection” time = 260.37 secs / 4.34 mins, transition times will vary
• Volume scan update time is about 4.75 minutes
• See Table 5-10 for PRF No. information
• VCP 212 has the VCP 12 elevation angles with SZ-2 on Split Cuts

> **Figure 5-6: VCP 212.**


5-15

October 2017

VCP 212 samples fourteen elevation angles with seventeen antenna rotations in 4.75 minutes. The
lowest three angles use Split Cut (CS/CD) and SZ-2, middle angles use Batch (B), and higher angles
use Contiguous Doppler (CDX) mode. The lines representing the beam elevation with height as a
function of range assume standard atmospheric refraction of the beam.
5.3.3.4 VCP 221.
The purpose of VCP 221 is the same as that described for VCP 21 in Section 5.3.2. In addition, this VCP provides
significantly reduced range-obscured velocity and spectrum width data when compared to VCP 21. (See Table 5-7
and Figure 5-7.)
5.3.3.4.1 Parameters.
• Short pulse (1.57 µs; PRF 318 to 1304 Hz)
• Unambiguous (Nyquist) velocity range 8 to 32.8 m/s (15.5 to 63.7 knots)
• Nine elevation angles
• 6-minute update rate
• Velocity data within 300 km (162 nm) radius
• Hydrometeorology data within 230 km (124 nm) radius
• Reflectivity data within 460 km (248 nm) radius
• Separate Surveillance and Doppler data are taken at the two lowest elevation angles to improve clutter filtering
performance, maximize the velocity accuracy and unambiguous interval
• Lowest five elevation angles are continuous
5.3.3.4.2 Strengths/Advantages.

5-16

FMH-11-Part C

•
•

Reduced range-folded data as compared to VCP 21
Slightly higher accuracy of estimates for reflectivity and velocity because of a larger number of pulses in the
sample estimate

5.3.3.4.2 Limitations.
• Coarse vertical resolution above the fifth elevation scan
• Degraded performance of algorithms that depend on volumetric information, relative to VCPs 211 and 212
• Pronounced banding of legacy Enhanced Echo Tops and high-layer legacy products
• Small-scale phenomena such as storm initiation may not be detected at the earliest opportunity

> **Table 5-7: VCP 221 Characteristics**

> Scan

Scan

Surveillance

Doppler PRF No.

AZ
Angle
(°)

Rate

Period
(sec)

0.5

(°/
sec)
11.34

31.93

0.5

15.61

23.19

1.5

11.34

31.93

1.5

15.61

23.19

2.4
3.4
4.3
6.0
9.9
14.6
19.5

10.75
10.75
10.75
11.18
12.13
12.13
12.13

33.69
33.69
33.69
32.37
29.85
29.85
29.85

WF

PRF

Type

No.

#
Pulses






#
Pulse

#
Pulse

#
Pulse

#
Pulse

#
Pulse

CS/
SZ-2
CD/
SZ-2
CS/
SZ-2
CD/
SZ-2
B
B
B
B
CDX
CDX
CDX



–

–

–

–

–

–

–








–

–

–

–

–











–
–
–

–
–
–

Notes:
• Default Doppler PRFs are underlined and bold; Doppler PRFs are editable for <7.0 degrees, since mid-2014.
Prior to 2014 Doppler PRFs are not editable for SZ-2 cuts
• Sum of periods, which is “data collection” time = 333.21 secs / 5.55 mins, transition times will vary
• Volume scan update time is about 6 minutes
• See Table 5-10 for PRF No. information
• VCP 221 has the VCP 21 elevation angles with SZ-2 on Split Cuts

5-17

October 2017


> **Figure 5-7: VCP 221.**

> VCP 221 samples nine elevation angles with eleven antenna rotations in about 6 minutes. The lowest two angles use Split Cut (CS/CD) and SZ-2, middle angles use Batch (B), and higher angles use

VCP 221 samples nine elevation angles with eleven antenna rotations in about 6 minutes. The lowest two angles use Split Cut (CS/CD) and SZ-2, middle angles use Batch (B), and higher angles use
Contiguous Doppler (CDX) mode. The lines representing the beam elevation with height as a function
of range assume standard atmospheric refraction of the beam.

#### 5.3.4 Clear Air Group, VCPs 31 and 32.

The purpose of VCPs 31 and 32 is to scan the atmosphere with enhanced sensitivity in order to detect low signalto-noise echoes (e.g., snow, refractive index gradients, smoke, and insects). In practice the VCPs are used when no
precipitation is detected or when only snow or very light precipitation is detectable.
5.3.4.1 Long Pulse VCP 31.
The purposes of VCP 31 are to: 1) detect early formation of convective precipitation, 2) detect air mass discontinuities, 3) determine the depth of the mixing layer, 4) monitor precipitation onset, and 5) obtain wind profiles to
supplement soundings. (See Table 5-8 and Figure 5-8.)
5.3.4.1.1 Parameters.
• Long pulse (4.7 µs PRF 318 to 452 Hz)
• Unambiguous (Nyquist) velocity range 8 to 12.4 m/s (15.5 to 24.1 knots)
• Five elevation angles
• 10-minute update rate
• Surveillance coverage of the lowest elevation angle within 460 km (248 nm) radius
• Hydrometeorological data within 230 km (124 nm) radius

5-18

FMH-11-Part C

•

Separate Surveillance and Doppler PRF on lowest three elevation angles

5.3.4.1.2 Strengths/Applications.
• Long pulse provides maximum sensitivity to low signals because of the larger number of sample estimates per
volume sample
• Excellent for detection of weak returns such as boundaries, fronts, ice crystals and non-meteorological returns
such as smoke and insects and birds
• Algorithms can function in clear air mode
• Often used effectively for dry snow situations
5.3.4.1.3 Limitations.
• May not provide unambiguous estimates of the velocity within a low-level jet
• Velocity dealiasing failures occur more frequently, except during light wind conditions, because of the large percentage of velocity dealiasing errors due to a small Nyquist co-interval
• Lack of vertical resolution will restrict usefulness in sampling storms or in
• supplementing upper air soundings
• Cone of silence above 4.5°
• Lower spatial resolution (750 m pulse width vs. 250 m pulse width for short pulse)
• Volume scan algorithms requiring the full volume data will not function well

> **Table 5-8: VCP 31 Characteristics, Long Pulse**

> Scan

Scan

Surveillance

Angle

AZ
Rate

Period

WF

PRF

(°)

(°/sec)

(sec)

Type

No.

#
Pulses

0.5
0.5
1.5
1.5
2.5
2.5
3.5
4.5

5.04
5.07
5.04
5.07
5.04
5.07
5.07
5.07

71.78
71.47
71.78
71.47
71.78
71.47
71.47
71.47

CS
CD
CS
CD
CS
CD
CDX
CDX



Doppler PRF No.





#
Pulse

#
Pulse

#
Pulse

#
Pulse

#
Pulse

–
–
–
–
–
–
–
–

–
–
–
–
–
–
–
–

–
–
–
–
–
–
–
–

–
–
–
–
–
–
–
–

–
–
–
–
–
–
–
–

Notes:
• PRF #2 is used for CD mode throughout; this VCP and is not editable
• Sum of periods, which is “data collection” time = 572.96 secs / 9.55 mins, transition times will vary
• Volume scan update time is about 10 minutes
• See Table 5-10 for PRF No. information

5-19

October 2017


> **Figure 5-8: VCP 31.**

> VCP 31 samples five elevation angles (0.5, 1.5, 2.5, 3.5, and 4.5 degrees) in 10 minutes. The lowest

VCP 31 samples five elevation angles (0.5, 1.5, 2.5, 3.5, and 4.5 degrees) in 10 minutes. The lowest
three angles use Split Cut (CS/CD), and highest two angles use Contiguous Doppler (CDX) mode.
The lines representing the beam elevation with height as a function of range assume standard atmospheric refraction of the beam.
5.3.4.2 Short Pulse VCP 32.
The purpose of VCP 32 is the same as that described for VCP 31 in Section 5.3.4.1. (See Table 5-9 and Figure 5-9.)
5.3.4.2.1 Parameters.
• Short pulse (1.57 µs; PRF 318 to 1304 Hz)
• Unambiguous (Nyquist) velocity range 8 to 28.2 m/s (15.5 to 54.8 knots)
• Five elevation angles
• Ten-minute update rate
• Surveillance coverage of the lowest elevation angle within 460 km (248 nm) radius
• Hydrometeorological data within 230 km (124 nm) radius
• Separate surveillance and Doppler PRF on the two lowest elevation angles
5.3.4.2.2 Strengths/Applications.
• Clear air sampling as with VCP 31 but with short pulse
• Fewer velocity dealiasing failures than with VCP 31
• Almost as many phenomena detected as detected with VCP 31

5-20

FMH-11-Part C

5.3.4.2.3 Limitations.
• Less sensitive than VCP 31
• Lack of vertical resolution will restrict usefulness in sampling storms or in supplementing upper air soundings
• Cone of silence above 4.5° elevation
• Algorithms requiring the full volume data will not function well

> **Table 5-9: VCP 32 Characteristics, Short Pulse**

> Scan

Scan

Surveillance

Angle

AZ
Rate

Period

WF

PRF

(°)

(°/sec)

(sec)

Type

No.

#
Pulses

0.5
0.5
1.5
1.5
2.5
3.5
4.5

4.97
4.55
4.97
4.55
4.07
4.07
4.07

72.90
79.60
72.90
79.60
89.05
89.05
89.05

CS
CD
CS
CD
B
B
B

–
–

–
–

Doppler PRF No.





#
Pulse

#
Pulse

#
Pulse

#
Pulse

#
Pulse

–
–

–
–

–
–

–
–

–
–

Notes:
• Default Doppler PRF numbers are underlined and bold; Doppler PRFs are editable
• Sum of periods, which is “data collection” time = 572.63 secs / 9.54 mins, transition times will vary
• Volume scan update time is about 10 minutes
• See Table 5-10 for PRF No. information

5-21

October 2017


> **Figure 5-9: VCP 32.**

> VCP 32 samples five elevation angles (0.5, 1.5, 2.5, 3.5, and 4.5 degrees) in 10 minutes. The lowest

VCP 32 samples five elevation angles (0.5, 1.5, 2.5, 3.5, and 4.5 degrees) in 10 minutes. The lowest
two angles use Split Cut (CS/CD), and highest three angles use Batch (B) mode. The lines representing the beam elevation with height as a function of range assume standard atmospheric refraction of
the beam.

#### 5.3.5 Site-Specific VCPs.

Newer scanning strategies also include site-specific VCPs. These VCPs are based on the previously defined VCPs
but may include additional elevation slices. For example, the Langley Hill, WA radar (KLGX) has an additional
elevation slice of 0.2°. This additional slice uses Split Cut (CD and CS) for VCPs 11, 12, 21, 31, and 32; MPDA and
SZ-2 for VCP 121; and Split Cut with SZ-2 for VCPs 211, 212, and 221. With the additional slice, the update rate
is increased. Increased rates range from ~ 32 seconds for VCP 12 and up to ~153 seconds for VCP 31. Again, if
AVSET and/or SAILS are enabled, the VCP characteristics (i.e. update rate) may be different than those previously
stated.

### 5.4 Mode Selection Function.

The Mode Selection Function (MSF) was first introduced in 2006. It replaced functional remnants of the Precipitation Detection Function (PDF) that remained after the Enhanced PPS Preprocessing (EPRE) sub-algorithm was
first deployed while retaining some of the PDF logic. The MSF offers the operator automatic and manual selection of either mode, Precipitation or Clear Air; and it allows for a default VCP to be selected by the operator for
both modes. The MSF determines a recommended mode as soon as the hybrid scan (Section 3.3.1) is generated by
EPRE. The recommended mode is determined based on the areal coverage exceeding the reflectivity threshold. The
areal coverage and reflectivity thresholds are adaptable. The mode chosen by the operator can be in conflict with the
recommended mode for a specified time not to exceed 48 hours. However, the operator can choose to ignore this
conflict.
5-22

FMH-11-Part C


### 5.5 Volume Coverage Pattern Adaptable Parameters.

Most characteristics of operational VCPs are fixed; however, operators are able to make some parameter changes.
One changeable parameter is the Doppler PRF for a selectable range of elevation cuts. An exception exists for VCP
31 and VCP 121 whose Doppler PRFs are not editable, to ensure proper processing. For all other VCPs, users can
control three sector boundaries to allow localized PRF changes. (Note: Initially the Doppler PRFs for SZ-2 Split
Cuts of VCPs 211, 212, and 221 were not modifiable. However, after 2013, PRF changes to these Split Cuts became
possible.) Another editable VCP parameter is the velocity increment. Each type of change is discussed below as well
as the fixed characteristics of selectable PRFs.

#### 5.5.1 Doppler PRF Changes.

Users can select the best PRF in order to minimize the amount of obscuration for a specific meteorological target
(e.g., the maximum unambiguous range can be modified to a limited extent by changing the Doppler PRF). Normally, the Auto PRF function will minimize the amount of obscuration for an entire elevation cut; however, velocity
values within a particular storm or region of interest might be range folded. In those instances, an operator can
change the PRF to alter the unambiguous range and thus reveal velocity values that were previously overlaid.

#### 5.5.2 Velocity Measurement Increment Changes.

The Velocity Measurement Increment (VMI) selection defines the resolution of the available base velocity data. Velocity measurement increments are available for resolutions of 0.97 knots (0.5 m/s) and 1.94 knots (1.0 m/s). Radial
wind velocities can be viewed ranging from ±122 knots using the default VMI resolution of 0.97 knots. In order to
view radial wind speeds ranging from ±244 knots, the VMI resolution must be changed to 1.94 kt resolution. This is
usually done when a tropical storm approaches or when very strong divergence at the summit of convective storms
occurs where the flow is sometimes greater than 200 knots. When 1.94 knots is selected as the VMI, the velocity
resolution reduces from a 1- knot increment to a 2-knot increment. The selection of a velocity increment also dictates which velocity display threshold table will be used to quantize the data for display on the Velocity (V) product
(see Section 2.3.2).

#### 5.5.3 PRF Characteristics.

Available PRFs are coded with PRF numbers ranging from 1 through 8 (Table 5-10). In addition, each WSR-88D
uses one of five sets known as Delta PRF Indices (PRI) A through E that supply small shifts to numbered PRFs
to avoid interference with neighboring radars. Over 90 percent of operational WSR-88D sites use Delta PRI Set C.
The unambiguous range and Nyquist Velocity for each

5-23

October 2017

5.5.4
PRF of Delta PRI C are shown in the following table.

> **Table 5-10: An Example of Typical WSR-88D PRF Characteristics Used Operationally (Delta PRI**

> Set C)

Set C)
PRF No.

PRF (s-1)



252/467

Vmax (kts/
ms-1)
16/8



181/336

22/11



126/233

32/16



95/176

43/22



80/148

51/26



74/137

55/28



69/128

59/30



63/117

64/33

Rmax (nm/
km)

5-24

Appendix A: Acronyms And Abbreviations
2D-VDA......................................................................................................Two-Dimensional Velocity Dealiasing Algorithm
AEL...............................................................................................................Algorithm Enunciation Language
AGL.............................................................................................................Above Ground Level
AP.................................................................................................................Anomalous Propagation
ARL..............................................................................................................Above Radar Level
APR..............................................................................................................Layer Composite Reflectivity – Anomalous Propagation
Removed Product
ARTCC........................................................................................................Air Route Traffic Control Center
ASD..............................................................................................................Autocorrelation Spectral Density
ASP...............................................................................................................Archive Status Product
ASCII...........................................................................................................American Standard Code for Information Interchange
AWIPS.........................................................................................................Advanced Weather Interactive Processing System
AVSET.........................................................................................................Automated Volume Scan Evaluation and Termination
AZ/RAN.....................................................................................................Azimuth/Range
B....................................................................................................................Batch
BD................................................................................................................Big Drops
BI..................................................................................................................Biological
BWER..........................................................................................................Bounded Weak Echo Region
CAPPI..........................................................................................................Constant Altitude Plan Position Indicator
CAT..............................................................................................................Combined Attribute Table
CBT..............................................................................................................Coherency-Based Threshold
CC.................................................................................................................Correlation Coefficient Product
CD................................................................................................................Contiguous Doppler
CDX.............................................................................................................Contiguous Doppler without range unfolding
CFC..............................................................................................................Clutter Filter Control Product
CLD.............................................................................................................Clutter Likelihood Doppler Product
CLEAN-AP................................................................................................Clutter Environment Analysis using Adaptive Processing
CLR..............................................................................................................Clutter Likelihood Reflectivity Product
CMD............................................................................................................Clutter Mitigation Decision
CONUS.......................................................................................................Conterminous United States
CS.................................................................................................................Contiguous Surveillance
CS/CD.........................................................................................................Split Cut (Contiguous Surveillance/Contiguous Doppler)
CS.................................................................................................................Combined Shear Product
CR.................................................................................................................Composite Reflectivity Product
A-1

October 2017
CRE

Composite Reflectivity Edited for Anomalous Propagation
Product

CVG.............................................................................................................CODEview Graphics
CWSU..........................................................................................................Central Weather Service Unit
DAA.............................................................................................................Digital Accumulation Array Product
dB.................................................................................................................Decibel
dBR...............................................................................................................Decibel (Reflectivity)
dBZ..............................................................................................................Radar Reflectivity Factor (Logarithmic Unit)
dBZ..............................................................................................................Decibels Relative to an Equivalent Reflectivity Factor
DBV.............................................................................................................Integrated Terminal Weather System Digital Base Velocity
Product
DCC.............................................................................................................Digital Correlation Coefficient Product
DHC.............................................................................................................Digital Hydrometeor Classification Product
DHR.............................................................................................................Digital Hybrid Scan Reflectivity Product
DIV..............................................................................................................Divergence
DKD............................................................................................................Digital Specific Differential Phase
DMD............................................................................................................Digital Mesocyclone Detection Data Array Product
DOD............................................................................................................Digital One-Hour Accumulation Difference Product
DPA..............................................................................................................Hourly Digital Precipitation Array Product
DPR..............................................................................................................Digital Instantaneous Precipitation Rate Product
DQA.............................................................................................................Data Quality Assurance Algorithm
DR................................................................................................................Base Reflectivity Data Array Product
DRQ.............................................................................................................Base Reflectivity Data Array Edited with DQA Product
DS.................................................................................................................Dry Snow
DSA..............................................................................................................Digital Storm Total Accumulation Product
DSD.............................................................................................................Digital Storm Total Difference Product
DSP..............................................................................................................Digital Storm Total Precipitation Product
DUA.............................................................................................................Digital User-Selectable Accumulation Product
DV................................................................................................................Mean Radial Velocity Data Array Product
DV7..............................................................................................................Mean Radial Velocity Enhanced Resolution Product
DVL.............................................................................................................High Resolution Digital Vertically Integrated Liquid
DZD............................................................................................................Digital Differential Reflectivity Product
E...................................................................................................................Flux of Kinetic Energy of Hailstones
EDC.............................................................................................................Eddy Dissipation Confidence Product
EDR.............................................................................................................Eddy Dissipation Rate Product
A-2

FMH-11-Part C
ET.................................................................................................................Tops Product
EET..............................................................................................................High Resolution Enhanced Echo Tops Product
EPRE...........................................................................................................Enhanced PPS Preprocessing Sub-algorithm
ETVS...........................................................................................................Elevated Tornado Vortex Signature
EWT.............................................................................................................Environmental Wind Table
FAA..............................................................................................................Federal Aviation Administration
FAR..............................................................................................................False Alarm Ratio
FCST............................................................................................................Forecast
FMH.............................................................................................................Federal Meteorological Handbook
FTM.............................................................................................................Free Text Message Product
GC................................................................................................................Ground Clutter
GFM.............................................................................................................Gust Front MIGFA Product
GMAP.........................................................................................................Gaussian Model Adaptive Processing
GR................................................................................................................Graupel
GSM.............................................................................................................General Status Message
HA................................................................................................................Rain mixed with Hail
HC................................................................................................................Hydrometeor Classification Product
HCA.............................................................................................................Hydrometeor Classification Algorithm
HCI...............................................................................................................Computer Interface
HDA.............................................................................................................Hail Detection Algorithm
HHC.............................................................................................................Hybrid Hydrometeor Classification Product
HHL.............................................................................................................Hail Hazard Layer Product
HI.................................................................................................................Hail Index Product
HKE.............................................................................................................Hailfall Kinetic Energy
HR................................................................................................................Heavy Rain
HSR..............................................................................................................Hybrid Scan Reflectivity Product
IC..................................................................................................................Ice Crystals
ICD..............................................................................................................Interface Control Document
ID.................................................................................................................Identifier
IHL...............................................................................................................Icing Hazard Level Product
ITWS............................................................................................................Integrated Terminal Weather System
ITWSDBV..................................................................................................Integrated Terminal Weather System Digital Base Velocity
KDP.............................................................................................................Specific Differential Phase Product
LEWP..........................................................................................................Line Echo Wave Pattern
A-3

October 2017
LFM.............................................................................................................Limited-Area Fine Mesh (model)
LRA..............................................................................................................Layer Composite Reflectivity (Average) Product
LRM.............................................................................................................Layer Composite Reflectivity (Maximum) Product
M...................................................................................................................Mesocyclone Product
MD...............................................................................................................Mesocyclone Detection Product
MDA............................................................................................................Mesocyclone Detection Algorithm
MEHS..........................................................................................................Maximum Expected Hail Size
MIGFA........................................................................................................Machine Intelligent Gust Front Algorithm
ML................................................................................................................Melting Layer Product
MLDA..........................................................................................................Melting Layer Detection Algorithm
MOA............................................................................................................Memorandum of Agreement
MPDA..........................................................................................................Multiple Pulse Repetition Frequency Dealiasing Algorithm
MPE.............................................................................................................Multisensor Precipitation Estimator
MRU.............................................................................................................Mesocyclone Rapid Update Product
MSCF...........................................................................................................Master System Control Function
MSF..............................................................................................................Mode Selection Function
MSI...............................................................................................................Mesocyclone Strength Index
MSL..............................................................................................................Mean Sea Level
NA................................................................................................................Not Applicable
ND...............................................................................................................No Data
NCEI...........................................................................................................National Centers for Environmental Information
NEXRAD...................................................................................................Next Generation Weather Radar
NGRPS........................................................................................................Minimum Number Gage-Radar Pairs
NOAA.........................................................................................................National Oceanic and Atmospheric Administration
NSE..............................................................................................................Near Storm Environment
NSSL............................................................................................................National Severe Storms Laboratory
NTDA..........................................................................................................NEXRAD Turbulence Detection Algorithm
NWS.............................................................................................................National Weather Service
OFCM..........................................................................................................Office of the Federal Coordinator for Meteorological
Services and Supporting Research
OHA............................................................................................................One-Hour Accumulation Product
OHP.............................................................................................................One-Hour Rainfall Accumulation Product
OPUP...........................................................................................................Open System Principal User Processor
ORPG..........................................................................................................Open Systems Radar Product Generator
OSD.............................................................................................................One-Hour Snow Depth Accumulation Product
A-4

FMH-11-Part C
OSW.............................................................................................................One-Hour Snow Water Equivalent Accumulation Product
PDF..............................................................................................................Precipitation Detection Function
PHI...............................................................................................................Differential Phase
POD.............................................................................................................Probability of Detection
POH.............................................................................................................Probability of Hail
POSH...........................................................................................................Probability of Severe Hail
PPS...............................................................................................................Precipitation Processing System
PRI...............................................................................................................PRF Indices
PRF..............................................................................................................Pulse Repetition Frequency
PRT..............................................................................................................Pulse Repetition Time
PSP...............................................................................................................Programmable Signal Processor
QC................................................................................................................Quality Control
QPE.............................................................................................................Quantitative Precipitation Estimation
R....................................................................................................................Reflectivity Product
RA.................................................................................................................Rain
RAN.............................................................................................................Range
RCM.............................................................................................................Radar Coded Message Product
RCS...............................................................................................................Reflectivity Cross Section Product
RDA.............................................................................................................Radar Data Acquisition
RDASC........................................................................................................RDA Status and Control
REC..............................................................................................................Radar Echo Classifier
RFD..............................................................................................................Rear Flank Downdraft
RHI...............................................................................................................Range Height Indicator
RMS..............................................................................................................Root Mean Square
ROC.............................................................................................................Radar Operations Center
RPG..............................................................................................................Radar Product Generator
S....................................................................................................................Snow Water Equivalent
SAA..............................................................................................................Snow Accumulation Algorithm Product
SAILS...........................................................................................................Supplemental Adaptive Intra-Volume Low-Level Scan
SCIT.............................................................................................................Storm Cell Identification and Tracking
SD.................................................................................................................Snow Fall
SDR..............................................................................................................Reflectivity Super Resolution Data Array Product
SDV..............................................................................................................Velocity Super Resolution Data Array Product
SDW.............................................................................................................Spectrum Width Super Resolution Data Array Product
A-5

October 2017
SHI...............................................................................................................Severe Hail Index
SNR..............................................................................................................Signal-to-Noise Ratio
SO.................................................................................................................SuperOb
SPD..............................................................................................................Supplemental Precipitation Data Product
SPG..............................................................................................................Supplemental Product Generator
SPRT............................................................................................................Staggered Pulse Repetition Time
SRM..............................................................................................................Storm Relative Mean Radial Velocity (Map) Product
SRR...............................................................................................................Storm Relative Mean Radial Velocity (Region) Product
SS..................................................................................................................Storm Structure Product
SSD...............................................................................................................Storm Total Snow Depth Accumulation Product
SSW..............................................................................................................Storm Total Snow Water Equivalent Accumulation Product
STA...............................................................................................................Storm Total Accumulation Product
STI................................................................................................................Storm Tracking Information Product
STP...............................................................................................................Storm Total Rainfall Accumulation Product
SW................................................................................................................Spectrum Width Product
SWA..............................................................................................................Severe Weather Analysis Product
SWP..............................................................................................................Severe Weather Probability Product
SWR.............................................................................................................Severe Weather Analysis (Reflectivity) Product
SWS..............................................................................................................Severe Weather Analysis (Radial Shear) Product
SWV.............................................................................................................Severe Weather Analysis (Mean Radial Velocity) Product
SWW............................................................................................................Severe Weather Analysis (Spectrum Width) Product
SZ.................................................................................................................Sachidananda-Zrnic (Range Unfolding Algorithm)
TDA.............................................................................................................Tornado Detection Algorithm
THP..............................................................................................................Three-Hour Rainfall Accumulation Product
TRU..............................................................................................................Tornado Vortex Signature Rapid Update Product
TVS..............................................................................................................Tornado Vortex Signature Product
UAM.............................................................................................................User Alert Message Product
UK................................................................................................................Unknown
ULR..............................................................................................................User Selectable Layer Reflectivity Maximum Product
URC..............................................................................................................Unit Radar Committee
USD..............................................................................................................User Selectable Snow Depth Accumulation Product
USP..............................................................................................................User Selectable Rainfall Accumulation Product
USW.............................................................................................................User Selectable Snow Water Equivalent Accumulation
UTC..............................................................................................................Universal Time (Coordinated)
A-6

FMH-11-Part C
V...................................................................................................................Mean Radial Velocity Product
VAD.............................................................................................................Velocity Azimuth Display Product
VCP..............................................................................................................Volume Coverage Pattern
VCS..............................................................................................................Mean Radial Velocity Cross Section
VDA.............................................................................................................Velocity Dealiasing Algorithm
VDTF..........................................................................................................Velocity Difference Transfer Function
VIL...............................................................................................................Vertically Integrated Liquid Water Product
VMI..............................................................................................................Velocity Measurement Increment
VWP.............................................................................................................Velocity Azimuth Display Wind Profile Product
W..................................................................................................................Vertical Velocity
WCT.............................................................................................................Weather & Climate Toolkit
WDTB.........................................................................................................Warning Decision Training Branch
WER.............................................................................................................Weak Echo Region
WFO............................................................................................................Weather Forecast Office
WS................................................................................................................Wet Snow
WSR-88D....................................................................................................Weather Surveillance Radar-1988, Doppler
ZDR.............................................................................................................Differential Reflectivity Product
Zr..................................................................................................................Recombined Refectivity
Z-R...............................................................................................................Reflectivity – Rain Rate
Z-S................................................................................................................Reflectivity – Snow Rate

A-7

October 2017

A-8

Appendix B: Glossary
Adaptation Data: Adaptable parameter settings for WSR-88D hardware and software that change their operating
characteristics. These changes enable system optimization based on meteorological, climatological, and regional
variations, as well as user preferences.
Adiabatic Process: A process in which a system does not interact with its surroundings by virtue of a temperature
difference between them. In an adiabatic process any change in internal energy (for a system of fixed mass) is solely
a consequence of work. For an ideal gas and for most atmospheric systems, compression results in warming, expansion results in cooling.
Alert: An operational condition or message created when a user-selected product value or algorithm output is detected during an elevation or volume scan. An alert is usually requested by a user in order to be notified of a condition related to a meteorological event.
Algorithm: A fixed step-by-step procedure, usually within system software, designed to accomplish a given result;
usually a simplified procedure for solving a complex problem; also a full statement of a finite number of steps. Meteorological algorithms are designed to recognize data patterns related to weather phenomena or threats.
Aliasing: The process by which frequencies too high to be analyzed with the given sampling interval appear at a
frequency less than the Nyquist frequency.
Anomalous Propagation: A propagation path of electromagnetic radiation that deviates from the path expected
from refractive conditions in a standard atmosphere.
Antenna: (Also called aerial; sometimes the more general term radiator is used.) A conductor or system of conductors for radiating and/or receiving radio energy. As used in radar, the antenna is usually “directional,” that is, it
has the property of radiating or receiving radio waves in larger proportion in a given direction.
Archive Level I: The analog, time-domain output from the receiver. These data are useful for detailed engineering
studies. The data recording interface is located at the RDA.
Archive Level II: The digital base data output from the signal processor. The output also includes status information required to properly interpret the data (e.g., information on synchronization, calibration, date, time, antenna
position, clutter and notchwidth maps, and operational mode).
Archive Level III: The base products and derived products/algorithm output produced by the RPG. The Level III
products are defined in Part A of this handbook.
Archive Level IV: The base products and derived products/algorithm output produced by the RPG. Data recorded
are selected by at the user display system operator.
Aspect Ratio: The ratio of height-to-length scales (D/L) characteristic of a fluid flow or for radar, the ratio of the
actual physical size of the reflectivity or flow-field entity (e.g., hook echo, mesocyclone, TVS) and the size of the
radar illuminated volume.
Atmospheric Boundary Layer: (Also called boundary layer, planetary boundary layer.) The bottom layer of the
troposphere that is in contact with the surface of the earth.
Azimuth: A direction in terms of the 360° compass.
Base Data: Those digital fields of reflectivity, mean radial velocity, and spectrum width data in spherical coordinates provided at the finest resolution available from the radar. (Also known as Archive Level II.)
Batch Waveform: Contains both Surveillance and Doppler pulse trains alternating low and high PRFs within each
radial at each low elevation angle scanned to allow resolution of range ambiguities. It is used where a high degree of
clutter suppression is required, but where contiguous waveforms are not needed.

B-1

October 2017

Beam Filling: The measure of variation of hydrometer density throughout the radar sampling volume. If there is
no variation in density, the beam is considered to be filled.
Beam Width: Angular width of the antenna pattern. Usually the width where the power density is one-half that of
the axis of the beam.
Bias: A systematic difference between an estimate of and the true value of the parameter.
Bin: Radar sample volume.
Biological Target: Airborne living particulates such as insects and birds that backscatter incident radar signals.
Book-End Vortices: (Also line-end vortices.) With time, mesoscale corrective systems tend to develop vortex pairs
with opposite sense rotation at the ends of the convective line. Primarily created when the system updraft tilts the
easterly shear generated at the system’s cold pool/updraft interface.
Boundary Layer: The layer of a fluid adjacent to a physical boundary in which the fluid motion is affected by the
boundary and has a mean velocity less than the free-stream value.
Bounded Weak Echo Region (BWER): A nearly vertical channel of weak radar echo, surrounded on the sides
and top by significantly stronger echoes. The BWER, sometimes called a vault, is related to the strong updraft in a
severe convective storm. The BWER has long been found to be associated with the supercell thunderstorm.
Bow Echo: A bow-shaped line of convective cells that is often associated with swaths of damaging straight-line
winds and small tornadoes. Key structural features include an intense rear-inflow jet impinging on the core of the
bow, with book-end or line-end vortices on both sides of the rear-inflow jet, behind the ends of the bowed convective segment. Bow echoes have been observed with scales between 20 and 200 km, and often have lifetimes between
3 and 6 hours.
Bright Band: The enhanced radar echo caused by the difference in radar reflectivity of ice and water particles. This
echo is interpreted as the delineation on a radar display between frozen and liquid precipitation.
Bypass Map: In the absence of any operator-defined Clutter Suppression Regions, this map (built by the RDA
System Operability Test (RDASOT) software) specifies where to apply clutter suppression.
Calibration Constant: One of several known test signals injected into the radar system for the purpose of adjusting radar systems such as signal processor and receiver to conform to the output predicted by the radar equation.
Cell: A compact region of relatively strong vertical air motion (at least 10 ms-1; 19 kts). In radar, sometimes applied
to individual radar echoes or radar echo cores of higher reflectivity.
Centroid: The center of mass of a storm echo or storm echo component.
Channel: In radar, often pertaining to an elongated or linear weak echo feature.
Clear Air Mode: System scanning in order to gather data which will facilitate the detection of precursors to precipitation development and non-meteorological echoes. This mode uses slower scan rates than are used in the precipitation mode to provide increased sensitivity.
Cloud Street: Linear cloud organization occurring atop the updraft branches of horizontal convective rolls when
sufficient moisture is present.
Clutter (or Ground Clutter): The pattern of radar echoes from fixed ground targets.
Clutter Filter Bypass Map: See Bypass Map.
Clutter Suppression Region: An area defined by the user or by adaptation data where clutter suppression is to be
applied.
Combined Attribute Table: A table affixed to the Composite Reflectivity product composed of the outputs of
other products and meteorological algorithms pertaining to severe convective storms.
B-2

FMH-11-Part C

Complex Signal: In radar, a representation of the time-varying amplitude and phase of the received signal as the
real and imaginary parts of a time-varying complex number. These parts are called the in-phase and quadrature
components and are measured by coherent detection of the received signal.
Cone of Silence: A conical shaped region directly above the radar left un-scanned when the rotating radar antenna
has a fixed upper limit to its elevation. A typical value is 20o , leaving the 70o region above un-scanned.
Contiguous Doppler (CD) Scan: A constant high PRF (short Rmax and high Vmax) employed for the entire 360°
sweep at low elevation angles where range ambiguity resolution is required to accurately determine “1st guess” velocity and spectrum width array estimates.
Contiguous Surveillance (CS) Scan: A constant low PRF (long Rmax and low Vmax) scan employed for the entire
360° sweep at low elevation angles to determine proper target location, returned power, and to permit optimum
clutter filtering.
Contiguous Waveform: Immediately adjacent waves of the same character.
Convergence: A measure of the contraction of a vector field.
Correlated Shear: An output of the Mesocyclone Detection Algorithm indicating a 3-dimensional shear region
(i.e., vertically correlated) that is not symmetrical.
Correlation Coefficient: A measure of how similarly the reflected horizontal and vertical power returns of a dual
polarized signal are behaving from pulse to pulse in a resolution volume.
Covariance: A measure of the degree of association between two variables. In Doppler radars, the argument (or
angle) of the covariance of the complex signal is a measure of the Doppler frequency.
Data Level: The specific range of data values represented by a single pixel when the data are presented in a pixel
image format; the specific value a datum may assume.
Data Level Code: A code representing a specific data level; used to assign color values when such are assigned.
Data Resolution: The resolution of the base data as produced by the signal processor, nominally 1 km (0.54 nm)
x 1°(AZ) x 1°(ELEV) for the reflectivity values and 0.25 km (0.13 nm) x 1°(AZ) x 1°(ELEV) for radial velocity and
spectrum width values. Averaging and additional processing may reduce these resolutions. A measure of the degree
of association between two variables. In Doppler radars, the argument (or angle) of the covariance of the complex
signal is a measure of the thing.
Dealiasing: Process of correcting for aliases in the velocity field.
Decibel (dB): A logarithmic expression for ratio of two quantities. DBm is a decibel with respect to 1 milliwatt.
Mathematically:
dB

= 10 Log (P1/P2)

dBa

= 10 Log [(accumulation)/1 mm]

dBm

= 10 Log (P/10-3)

dBR

= 10 Log [(precipitation rate)/(1 mm/hr)]

dBZe = 10 Log (Ze)
Deep Convergence Zone: A narrow and deep velocity signature characterized by strong convergence along a nearly vertical interface extending from the radar horizon upward to altitudes as high as 50,000 ft. Often associated with
very damaging surface winds and related to the Mid-Altitude Radial Convergence.
Differential Reflectivity: The logarithm of the ratio of the reflected horizontal and vertical power returns.
B-3

October 2017

Digital Data Resolution: Establishes the number of unique values that can be associated with a parameter of
interest. The data resolution is set by the number of binary bits contained within the digital words that represent the
values. The number of unique values is equal to 2n, where n is the number of bits. For example if 4 bits are used,
then 24 = 16, thus 16 unique values, or levels, data can be represented. In the case of the base WSR-88D moment
data, 4 bits are used to establish the 16 data levels. An 8 bit product would have 28 = 256 unique levels.
Display Resolution: The area or two-dimensional product of the X and Y coordinates represented by one picture
element (pixel) of a raster scan display.
Divergence: A measure of the expansion or spreading out in a vector field.
Doppler Radar: A radar that detects and interprets the Doppler effect in terms of the radial velocity of a target.
The signal received by radar from a moving target differs in frequency from the transmitted frequency by an amount
that is proportional to the radial component of the velocity relative to the radar.
Downburst: A strong downdraft that induces an outburst of damaging winds on or near the ground.
Downdraft: Small-scale downward moving air current in a cumulonimbus cloud.
Dual Polarization: In terms of Doppler Radar, the system is capable of transmitting and receiving two orthogonal
polarizations.
Echo: Energy backscattered from a target as seen on the radar display.
Echo Tops: The height of the greatest (in altitude) non-zero reflectivity value (greater than the minimum significant reflectivity set in adaptation data, 18.5 dBZ is the default) for each 4 x 4 km (2.2 x 2.2. nm) grid box above the
surface of the Earth.
Elevation Scan: The process of the radar completing a full 360° rotation in azimuth for a specific elevation angle.
Elevation Slice: The full 360o rotation in azimuth for a specific elevation angle.
Equivalent Radar Reflectivity (Ze): When all the assumptions (e.g., uniformly distributed liquid water particles
whose diameters meet the Rayleigh approximation) do not apply, the radar reflectivity, Z, is expressed as Ze, the
equivalent radar reflectivity. Typically expressed as: dBZ = 10 Log Ze.
Estimate: A statement of the value of a quantity or function based on a finite number of samples.
Extratropical Cyclone: (Sometimes called extratropical low, extratropical storm.) Any cyclonic-scale storm that is
not a tropical cyclone, usually referring only to the migratory frontal cyclones of middle and high latitudes.
Feature: A set of pattern vectors in close proximity.
Flash Flood: A flood that rises and falls quite rapidly, usually as the result of intense rainfall over a relatively small
area.
Frequency: The number of recurrences of a periodic phenomenon per unit time. Electromagnetic energy is usually specified in Hertz (Hz), which is a unit of frequency equal to one cycle per second.
Fuzzy Logic: A system of logic dealing with the concept of partial truth with values ranging between “completely
true” and “completely false.”
Gust Front: The boundary between the horizontally propagating cold air outflow from a thunderstorm and the
surrounding environmental air.
Helicity: One-half the scalar product of the velocity and vorticity vectors. It is a conserved quantity if the flow is
inviscid and homogeneous in density, but is not conserved in more general viscous flows with buoyancy effects. The
concept is useful in understanding severe convective storms and tornadoes, since in strong updrafts the velocity and
vorticity vectors tend to be aligned, yielding high helicity.
B-4

FMH-11-Part C

Hook Echo: A pendant, curve-shaped region of reflectivity caused when precipitation is drawn into the cyclonic
spiral of a mesocyclone. The hook echo is a fairly shallow feature, typically extending only up to 3–4 km in height
within a supercell storm before becoming part of a bounded weak echo region.
Horizontal Convective Rolls: (Also known as horizontal roll vortices, boundary layer rolls.) Counter- rotating
horizontal vortices that commonly occur within the convective boundary layer; their major axes are aligned with the
mean boundary layer wind-shear vector.
Hybrid Scan: An approach in which different elevation angles (normally the lowest four) are used to minimize the
effects of ground clutter and data voids on radar based observations such as precipitation estimates.
Hydrometeor: Any product of condensation or deposition of atmospheric water vapor, whether formed in the
free atmosphere or at the earth’s surface; also, any water particle blown by the wind from the earth’s surface.
In-phase (signal): The signal obtained by demodulating the received signal with a local oscillator having the same
phase and frequency as the transmitted signal.
Isolated Storm: An individual cell or group of cells that are identifiable and separate from other cells in a given
geographic area.
Kalman Filter: A linear system in which the mean squared error between the desired output and the actual output
is minimized when the input is a random signal generated by white noise.
Klystron: A power amplifier tube used to amplify weak microwave energy (provided by a radar- frequency exciter)
to a high power level for a radar transmitter.
Lake-effect Snow: Localized, convective snow bands that occur in the lee of lakes when relatively cold airflows
over warm water. In the United States this phenomenon is most noted along the south and east shores of the Great
Lakes during arctic cold-air outbreaks.
Lapse Rate: The decrease of an atmospheric variable with height, the variable being temperature, unless otherwise
specified.
Limited-Area Fine Mesh (LFM): A rectangular grid based on a polar stereographic projection. The grid mesh
length of the LFM, 1/4 LFM and 1/40 LFM at 60° N (standard latitude) and 105° W (standard longitude) are
190.5, 47.625, and 4.7625 km (102.9, 25.7 and 2.6 nmi), respectively.
Line Echo Wave Pattern (LEWP): A special configuration in a line of convective storms configured like a wave
and that may indicate the presence of a low pressure area and the possibility of damaging winds and tornadoes. In
response to very strong outflow winds behind it, a portion of the line may bulge outward forming a bow echo.
Low-Level Jet (LLJ): (Also called low-level jet stream.) A jet stream that is typically found in the lower 2–3 km of
the troposphere. At night, sometimes called a nocturnal jet. Examples are the Great Plains low-level jet, extratropical cyclone low-level jet, African jet and the Somali jet.
Master System Control Function (MSCF): The computer-user interface at the WSR-88D system RPG.
Mesoscale Convective System (MCS): A cloud system that occurs in connection with an ensemble of thunderstorms and produces a contiguous precipitation area on the order of 100 km or more in horizontal scale in at least
one direction. An MCS exhibits deep, moist convective overturning contiguous with or embedded within a mesoscale vertical circulation that is at least partially driven by the convective overturning.
Mean Radial Velocity: The component of motion of the target toward or away from the radar.
Melting Layer: The region where hydrometers are in mixed phase. It represents the transition from solid to liquid
phase and may be several hundred meters deep.
Mesocyclone: A 3-dimensional region in a storm that contains strong cyclonic vertical vorticity (rotates cyclonically) and is closely correlated with severe weather.
B-5

October 2017

Mesocyclone Strength Index: A non-dimensional value based on the vertical integration of the three strength
parameters incorporated into Rank calculations. The vertical integration is divided by the depth of the circulation
and is weighted by density.
Mesoscale: On a scale of 4 km to 400 km (2.2 nm to 215 nm).
Microburst: Small downburst, 1 to 4 km (0.54 to 2.2 nm) in outflow size, with peak winds lasting 2 to 15 minutes.
Mid-Altitude Radial Convergence: A deep, mid-level, convergent, velocity signature related to the Deep Convergence Zone , but confined to mid-levels, and found to be a Doppler radar-based precursor of damaging straight-line
winds in a linear MCS or bowing convective system.
Mie Scattering: Scattering of electromagnetic waves by homogeneous spheres of arbitrary size (smaller, comparable to, or larger than the wavelength), named after Gustav Mie (1868–1957), whose theory of 1908 explains the
process.
Mini-Supercell: Convective storm that contains similar radar characteristics to those of a supercell (e.g., mesocyclone, hook echo, WER, BWER), but is significantly smaller in height and width.
Misoscale: On a scale of 40 m to 4 km (130 ft to 2.2 nm).
Nyquist Co-Interval: The full range of the Nyquist interval, e.g., +/- 50 kts.
Nyquist Interval: (Also Nyquist velocity). The maximum time interval between equally spaced samples of a signal
that will enable the signal waveform to be completely determined. Also known as the (absolute value) of the maximum unambiguous velocity that can be measured by a Doppler radar, e.g., 50 kts.
Operational Mode: A combination of one or more volume coverage patterns and products mixes tailored to one
or more meteorological situations.
Overhang: A storm has overhang if the edge of the storm component at a given height range (mid-levels) extends
outward beyond the edge of the storm component at the lowest elevation by a specified distance.
Particulate Matter: (Also called Particulates.) The term for solid or liquid matter in the form of particles found in
the air.
Pattern Vector: A pattern vector is formed by a series of azimuthally adjacent sample volumes of increasing or
decreasing Doppler velocity.
Pedestal: In radar, a device for supporting and positioning the antenna. Typically, the pedestal allows the azimuth
and elevation angles of the antenna to be controlled separately or in a coordinated way to permit different methods
of scanning.
Planetary Boundary Layer: The bottom layer of the troposphere that is in contact with the surface of the earth.
Point Clutter Rejection: The rejection or removal of echoes having the characteristics of point targets.
Polar Coordinates: A system of coordinates in which a point is isolated by its distance (range) and angular direction (azimuth) from a fixed reference point. For radars, this reference point is usually the location of the antenna.
Polarization: With respect to a transverse electromagnetic wave, the correlation between two orthogonal components of its electric (or, equivalently, magnetic) field.
Precipitation Mode: System scanning at variable rates to accommodate the greatest number of elevation angles,
thus sampling to full radar volume in order to detect precipitation in all its forms.
Product: Output of the WSR-88D receiver in the form of the three base moments (reflectivity, mean radial velocity, and velocity spectrum width) formatted by the RPG as base, derived, or algorithm processed alphanumeric
messages, graphic images, or graphic overlays for presentation on a user display system.
Product Resolution: The smallest spatial increment of a data element that is distinguishable in a product.
B-6

FMH-11-Part C

Propagation: Transmission of electromagnetic energy as waves through or along a medium.
Pulse: A single short duration transmission of electromagnetic energy.
Pulse Width: The linear distance in range occupied by an individual broadcast from a radar.
Quadrature (signal): In radar systems, an orthogonal relationship between two coherent signals in which the phase
of one signal is offset by 90° from the phase of the other. Two signals in quadrature may be regarded as a single
complex signal. In Doppler radar, the signal is composed of the in-phase and quadrature components.
Radar: (Coined word for radio detection and ranging.) An electronic instrument used for the detection and ranging
of distant objects of such composition that they scatter or reflect microwave radio energy. A radar consists of a
transmitter, receiver, antenna, display, and associated equipment for control and signal processing.
Radar Data Acquisition System Operability Test (RDASOT): An off-line program designed to determine the
status of the hardware and to enhance its maintainability. RDASOT, executed from the system console, consists of
diagnostic tests, calibration tests, and maintainability aids.
Radar Horizon: The locus of points at which direct rays from a radar transmitter/antenna become tangential to
the earth’s surface. The radar horizon extends beyond the geometric and visible horizons in conditions of normal
atmospheric refraction. It may be decreased or increased in particular cases as standard propagation is replaced by
substandard or superstandard propagation respectively. Beyond the radar horizon, surface targets cannot be detected under rf atmospheric conditions although significant radar power is sometimes detected in the diffraction zone
below the horizon.
Range Aliasing: (Also called range folding.) In radar meteorology, a sampling problem that arises when echoes
located beyond the maximum unambiguous range (Rmax) are received as if they were within this radar range. A radar
ordinarily computes range to targets by measuring the time interval between the transmission of a pulse and the
receipt of the returned signal, assuming that the signal was associated with the pulse just transmitted. However, depending on the pulse, the returned signal may be associated with one of several pulses transmitted prior to the latest
one. Therefore, a returned signal, indicated as originating at range r, could have originated at r + Rmax (second-trip
echo), or r + 2Rmax (third-trip echo), etc.
Range Dealiasing: (Also known as range unfolding.) The process or processes of removing range ambiguity in
apparent range of a multitrip target; that is to assign the correct target range.
Rayleigh Scattering: Approximate theory for electromagnetic scattering by small particles named for Lord
Rayleigh (John William Strutt, 1842–1919). Commonly used in radar theory referring to particles small as compared
to radar wavelength.
Rear Flank Downdraft: A downdraft almost exclusively associated with supercell storms found along the rear portion (facing in the direction of storm motion) of the storm and associated with the mesocyclone and often, tornadoes. Sometimes responsible for damaging surface winds.
Rear Inflow Jet: A mesoscale circulation feature in which a system-relative current of air enters and flows through
the stratiform precipitation region of mesoscale convective systems from the rear. The rear-inflow jet forms in response to the upshear-tilting of the convective circulation, as the horizontal buoyancy gradients along the back edge
of the system create a circulation that draws midlevel air in from the rear. The rear-inflow jet supplies potentially
cold and dry midlevel air that aids in the production of convective and system-scale downdrafts.
Receiver: An instrument used to detect the presence of and to determine the information carried by electromagnetic radiation. A receiver includes circuits designed to detect, amplify, rectify, and shape the incoming radio-frequency signals received at the antenna.
Reflectivity: A measure of the fraction of radiation reflected by a given surface; defined as a ratio of the radiant
energy reflected to the total that is incident upon that surface.
B-7

October 2017

Refraction: Changes in the direction of energy propagation (due to changes in speed) as a result of density changes
within the propagating medium.
Refractive Index: A measure of the amount of refraction. Numerically equal to the ratio of wave velocity in a
vacuum to wave speed in the medium.
Scatterer: Any object capable of reflecting the radar signal.
Segment: As applied in the Storm Segments algorithm, segments are defined as runs of contiguous radar sample
volumes greater than or equal to the minimum reflectivity threshold and having a combined length greater than or
equal to the segment length threshold.
Severe Storm: A storm with a tornado, surface hail >3/4 inch, or wind gusts >50 knots, or a combination of them.
Shear: The rate of change of the vector wind in a specified direction normal to the wind direction. Vertical shear is
the variation of the horizontal wind in the vertical direction.
Sidelobe: Secondary radiated energy maximum other than the radar main beam. Typically contains a small percentage of energy compared to the mainlobe.
Signal Processor: A computer processor used to apply a series of algorithms to the output of the receiver in order
to estimate the spectral moments contained in the received backscattered signal.
Signal to Noise Ratio: A ratio that measures the comprehensibility of data, usually expressed as the signal power
divided by the noise power.
Specific Differential Phase: The along-the radial range derivative of the differential phase (PHI) shift.
Spectrum Width: A measure of dispersion of velocities within the radar sample volume. Standard deviation of the
mean radial velocity spectrum.
Split Cut (CS/CD) Scan: While staying at a particular elevation angle, there is one full rotation using the CS waveform, followed by one full rotation using the CD waveform.
Spot Blanking: The ability of the RDA to selectively stop radiation of energy along specific azimuths so as not to
cause interference with another facility.
Storm: Any disturbed state of the atmosphere, especially as affecting the Earth’s surface, and strongly implying
destructive and otherwise unpleasant weather. Storms range in scale from tornadoes and thunderstorms, through
tropical cyclones, to widespread extra-tropical cyclones.
Stratiform: Descriptive of clouds or precipitation of extensive horizontal development, as contrasted to the vertically developed convective clouds or precipitation types.
Supercell: An often dangerous convective storm that contains radar characteristics such as the hook echo, WER,
BWER but that also contains a deep, persistent mesocyclone characterized most often by cyclonic vorticity and
closely associated with the dominant storm updraft and Rear Flank Downdraft. Variations include the “Low Precipitation” (LP), “Classic” (C), and “Heavy Precipitation” (HP) supercells. These storms are often long-lived, often
move somewhat differently than other non-severe storms in their environment, and commonly produce severe
weather.
Super Resolution: Increased resolution with gate spacing of 250 m and azimuthal resolution of 0.5°.
Surveillance Waveform: A constant low PRF (long Rmax and low Vmax) scan employed for the entire 360° sweep to
determine proper target location and returned power. This PRF is unambiguous over the observing domain (range).
Generally used as part of a split cut consisting of a surveillance waveform followed immediately by a range-ambiguous Doppler scan. (See also Contiguous Surveillance (CS) Scan).
Target: Precipitation or other phenomena that produce echoes.
B-8

FMH-11-Part C

Three-body Scattering: Radiation from a radar scattered toward the ground that is scattered back to hydrometeors, which then scatter some of the radiation back to the radar.
Tilt: A storm is said to have tilt if a line connecting the centroid of a midlevel storm component to the centroid of
the lowest storm component is to the right or rear of the direction of movement of the storm.
Tornado Vortex Signature (TVS): The Doppler velocity signature of a tornado or incipient tornado-like circulation within any scanned elevation angle. As the signature occurs when the radar beam is wider than the vortex, the
measured Doppler velocities are weaker than the rotational velocities within the vortex and the apparent core diameter is larger than that of the vortex. The signature, which may extend throughout a considerable vertical depth, is
ideally characterized by extreme Doppler velocity values of opposite sign separated in azimuth by the equivalent of
one beamwidth.
TOVER: An adaptable parameter (power difference) used in the radar range dealiasing algorithm. The parameter is
applied in comparing power returned from a range gate and those separated by the unambiguous range or multiples
of that range from the range gate in question.
Transmitter: A device used for the generation of signals of any type and form that are to be transmitted. In radio
and radar, it is that portion of the equipment that includes electronic circuits designed to generate, amplify, and
shape the radio frequency energy that is delivered to the antenna where it is radiated out into space.
Tropical Cyclone: The general term for a cyclone that originates over the tropical oceans. This term encompasses
tropical depressions, tropical storms, hurricanes, and typhoons.
Turbulence: Random and continuously changing air motions that are superposed on the mean motion of the air.
Unambiguous Range: The range to which a transmitted pulse wave can travel and return to the radar before the
next pulse is transmitted.
Uncorrelated Shear: An output of the Mesocyclone Detection Algorithm indicating a region of shear that is large
and symmetrical but not vertically correlated.
Unit Radar Committee: A committee formed at WSR-88D sites where there is more than one Associated Principal User.
Updraft: A small- scale current of air with marked vertical motion that is upward moving.
Velocity Aliasing: (Also called velocity folding.) A basic sampling problem arises when the unambiguous velocity
sampling interval is less than the full range of naturally occurring velocities, causing the erroneous appearance of
higher velocities within the sampling interval. This phenomenon occurs in Doppler velocity measurements when
the maximum unambiguous velocity interval (±Vmax) is less than the full range of velocities being measured. Any
true velocity, V, appears within the interval from -Vmax to +Vmax, with the value V , which is related to the true
velocity by V = V ± 2nVmax where n is an integer.
Velocity Dealiasing: (Also called velocity unfolding.) A process or processes by which ambiguous velocities are
assigned their correct unambiguous value.
Vertically Integrated Liquid (VIL): Vertical integral of liquid water content obtained from radar observations at
different elevation angles within a precipitation volume; has dimensions of mass per unit area. Liquid water content
M is computed from the equivalent reflectivity factor Ze using the Marshall-Palmer drop-size distribution
VIL Density: VIL divided by the echo top (m) and multiplied by 1000, units are g m-3.
VIL of the Day: A threshold VIL value associated with hail of ¾ inch diameter or larger on a given day. This value
will change from day to day, or even during the same day.
Volume Coverage Pattern: A volumetric sampling procedure designed for the surveillance of one or more particular meteorological phenomena.
B-9

October 2017

Volume Scan: The process of completing a series of specified scans in a specific sequence.
Vortex: In its most general use, any flow possessing vorticity. More often the term refers to a flow with closed
streamlines.
Warning: A message or condition created when an adverse situation is detected by the system in the WSR-88D
hardware or software.
Waveform: The pictorial representation of the shape of a wave showing the amplitude variations as a function of
time. Often used to also represent other wave properties.
Weak Echo Region (WER): Within a convective echo a localized region of weak radar echo that is bounded on
one side and above by strong echo and associated with the strong updraft region. It is located on the low-altitude
inflow or updraft side of the storm.
Wind Shear: The local variation of the wind vector or any of its components in a given direction.
WSR-88D System: The summation of all hardware, software, facilities, communications, logistics, staffing, training,
operations, and procedures specifically associated with the collection, processing, analysis, dissemination, and application of data from the WSR-88D unit.
WSR-88D Unit: The combination of one RDA, one RPG, and interconnecting communications.
Ze: See Equivalent Radar Reflectivity.

B-10



---

## Relevance to rustywx

The most directly relevant FMH-11 part for rustywx. Defines the complete product suite including product numbers, resolution, and format. Understanding the product structure (especially the difference between base data arrays and derived products) is essential for correctly interpreting and displaying NEXRAD Level II data. The algorithm descriptions (clutter filtering, range unfolding, dual-pol processing) explain the data quality characteristics visible in rustywx's display.
