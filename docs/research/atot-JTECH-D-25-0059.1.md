# Improving NEXRAD Velocity Retrievals Using Multi-PRT Scans with Regression Processing

**Authors:** John C. Hubbertᵃ, David Schvartzmanᵇ, Ulrike Romatschkeᵃ,ᶜ

ᵃ NCAR, Boulder, Colorado
ᵇ University of Oklahoma, Norman, Oklahoma
ᶜ GeoSphere Austria, Vienna, Austria

**Journal:** Journal of Atmospheric and Oceanic Technology, Volume 43, April 2026
**DOI:** 10.1175/JTECH-D-25-0059.1
**Received:** 12 May 2025 | **Final form:** 2 January 2026 | **Accepted:** 17 March 2026

**Source / Citation:**
- AMS Journals: https://journals.ametsoc.org/view/journals/atot/43/4/JTECH-D-25-0059.1.xml
- DOI: https://doi.org/10.1175/JTECH-D-25-0059.1

---

## Abstract

A fundamental data quality issue for pulsed Doppler radars is range–velocity ambiguity, especially at low elevation angles where weather echoes out to greater than 400 km in range need to be detected for effective long-range weather surveillance while also retrieving unambiguous radial velocities of around 30 m s⁻¹. The U.S. National Weather Service (NWS) Next Generation Weather Radars (NEXRADs) mitigate the range–velocity ambiguities by employing a so-called "split-cut" scan whereby the atmosphere is interrogated twice at the same low elevation angle: once with a surveillance scan and once with a Doppler scan. Due to range overlays in the Doppler scan, the SZ(8/64) systematic phase coding technique is used, but frequently velocities still cannot be resolved, which results in velocity voids (referred to as "purple haze"). There are two new techniques to recover data in the velocity void regions: 1) Doppler velocity recovery and dealiasing (VRAD) and 2) SZ(8/64) regression processing. These two algorithms are combined here for the first time for enhanced Doppler velocity recovery, and it is shown that this combination delivers superior velocity estimates as compared to either algorithm individually.

## Significance Statement

Critical to the observation, forecasting, and warnings of storms are the accurate estimation and wide spatial coverage of the Doppler velocity field, especially at low elevation angles where long-range unambiguous reflectivity estimates are required. Next Generation Weather Radar (NEXRAD) employs a uniform pulse repetition time (PRT) Doppler scan where the typical unambiguous range is about 120–150 km. The result is frequent overlaid echoes where the velocity cannot be estimated for each overlay, thus producing voids in velocity data. The presented new velocity recovery algorithm by and large makes it possible to significantly reduce the velocity data voids and thereby potentially reveal critical weather signatures. Having velocity estimates in regions that have been previously unattainable would be a significant and consequential improvement for forecasters and other users of radar data.

**Keywords:** Weather radar signal processing; Filtering techniques; Regression analysis

---

## 1. Introduction

A fundamental data quality issue for pulsed Doppler radar systems is range–velocity ambiguity. The equations that govern the unambiguous velocity (Va) and unambiguous range (Ra) are

> **Va = λ / (4Ts)**  &nbsp;&nbsp;&nbsp;&nbsp; (1)

> **Ra = cTs / 2**  &nbsp;&nbsp;&nbsp;&nbsp; (2)

where Ts is the pulse repetition time (PRT), λ is the wavelength, and c is the speed of light. Increasing Ts increases Ra but reduces Va, whereas decreasing Ts increases Va but reduces Ra. This issue affects, for example, the Next Generation Weather Radar (NEXRAD) network, which is the weather monitoring radar network operated by the National Weather Service (NWS) in the United States. For effective weather surveillance, at low elevation angles, the NWS stipulates an Ra of about 465 km (Ts ≈ 3.1 ms) which implies a Va of approximately 8.5 m s⁻¹. However, storms can have velocities in excess of 50 m s⁻¹. Clearly from (1) and (2), there is no single value of Ts that can satisfy both range and velocity requirements.

A well-known way to combat the range–velocity problem is to use multiple pulse repetition frequencies (PRFs) (PRF = 1/Ts) (Sirmans et al. 1976; Zrnić and Mahapatra 1985; Dazhang et al. 1984; Sachidananda and Zrnić 2003; Pirttilä et al. 2005; Cho and Chornoboy 2005; Tabary et al. 2005). In general, these techniques combine velocity estimates from the used PRFs to estimate a velocity using the Chinese remainder theorem (Torres et al. 2004). The combined PRF estimates permit an effective Va greater than that of the individual PRFs. The Ra is determined from the smallest PRF.

Here, we focus on the range–velocity mitigation technique employed by NEXRAD on the lowest elevation angles where the atmosphere is scanned using the so-called "split-cut" scan (OFCM 2006). The atmosphere is first interrogated with a long-PRT surveillance scan to obtain an Ra of about 465 km, and then, the same elevation angle is immediately scanned with a shorter PRT (short PRT, also often termed the Doppler scan) for higher Va values of approximately 25–34 m s⁻¹. However, the PRT of the Doppler scan yields a relatively small Ra of about 135–150 km so that there are frequently overlaid echoes that degrade the accuracy of velocity estimates. To separate the overlaid echoes and improve velocity estimates, the transmit pulses are modulated with the SZ(8/64) phase code (Sachidananda and Zrnić 1999) that allows each echo trip (up to four trips) to possess a unique phase sequence. Thus, each trip can be cohered, while the remaining three uncohered trips behave as noise so that velocity estimates for the cohered trip are unbiased by the overlaid uncohered echoes.

In this paper, we focus on cases with significant overlaid echoes from only two trips (first and second) to illustrate the proposed velocity retrieval technique. The stronger of the two trips [strong trip (ST) echo] is cohered using the conjugate of its phase code. The accompanying weak trip (WT) echo is then phase coded with a so-called modulation code that will cause the WT spectrum to be comprised of eight replicas of itself so that the WT signal will behave as noise. For example, the first-lag autocorrelation is zero, and thus, estimates of the ST velocity are unbiased by the overlaid WT signal. The ST signal is then removed by a filter. To filter the ST signal, typically a von Hann window is first applied to the time series for legacy processing. A discrete Fourier transform (DFT) is used to generate the power spectrum. Because the velocity of the ST echo is known, the power spectrum can be notched to eliminate the ST signal. The filter bandwidth of the notch is 0.75 of the total Nyquist bandwidth. At this point, the filtered signal is transformed back to the time domain where the WT signal can be recohered. The WT velocity may then be accurately estimated if sufficient WT echo remains.

A brief outline of the legacy SZ(8/64) processing steps is as follows. Let E_k be the received phase-coded time series with c_k phases. Let the first trip be the strong trip and j = √(−1).

1. Cohere the ST echo: E1_k = E_k · e^(−jc_k). The weak second trip echo now has the modulation code f_k.
2. Multiply the time series with the von Hann window weights.
3. Clutter filter with Gaussian model adaptive processing (GMAP) if necessary.
4. Estimate the ST velocity, Vs.
5. Use the DFT to calculate the power spectrum.
6. Set 0.75 of the power spectrum points to zero centered around Vs. This ideally eliminates the ST echo though residuals can remain.
7. Transform the filtered spectrum to the time domain with an inverse DFT.
8. Cohere the WT echo by multiplying the time series with e^(−jf_k), the conjugate of the modulation code.
9. Calculate the WT velocity.

For details of the SZ(8/64) algorithm, see Sachidananda and Zrnić (1999) and Sachidananda et al. (1998). In this paper, a regression filter replaces both the legacy ST filter and the GMAP clutter filter. Importantly, the regression filter does not require the use of a window function.

There are many situations where significant portions of the velocity field cannot be recovered due to insufficient WT signal-to-noise ratio (SNR). Furthermore, ground clutter, especially strong clutter close to the radar location, can make it difficult to recover accurate velocities since clutter requires additional filtering, which further attenuates the WT signal and degrades recoverability. In the NEXRAD display of the standard level-2 data products, the SZ(8/64) (SZ) processed unrecoverable velocities are typically shown in purple in the NEXRAD velocity color scale and are colloquially referred to as "purple haze."

Recently, two very different algorithms have been proposed that can significantly reduce the large areas of NEXRAD purple haze when compared to the standard NEXRAD processing (which we call the legacy method). The first technique uses the velocities from the long-PRT scan to populate the purple haze regions and is termed Doppler velocity recovery and dealiasing (VRAD) (Schvartzman and Palmer 2024). The velocities from the long-PRT scan have a Va of about 8.5 m s⁻¹ and thus can have significant velocity aliasing. That is, the velocity estimates are V̂ = Vt ± n·2Va, where V̂ is the estimated velocity, Vt is the true velocity, and n is an integer. An important aspect of VRAD is its ability to dealias the long-PRT velocity estimates.

The second technique is based on regression filtering (Hubbert et al. 2025, 2021). In Hubbert and Romatschke (2025) (hereafter referred to as HR25), it is shown how a high-pass regression filter can operate as an arbitrary-frequency bandstop filter. The new filter, termed the regression frequency shift filter (RFSF), is used to attenuate the ST echo in SZ processing. In HR25, it is shown that in the SZ WT velocity area, the RFSF can recover more velocities as compared to the legacy SZ processing employed by NEXRAD, and in general, the regression-processed velocities have lower areal standard deviation.

In this paper, the VRAD velocity retrieval technique is combined with the SZ RFSF velocity retrieval technique and is termed REG-VRAD. Two cases are analyzed in detail from the NEXRAD split cut: one from KFTG, the Denver, Colorado, NEXRAD, and the other one from KLWX, the Sterling, Virginia, NEXRAD, which has a mesocyclone signature obscured by purple haze. It is shown that REG-VRAD can recover more velocity estimates with lower standard error as compared to the individual algorithms or the legacy method. Two additional cases from the KBGM (Binghamton, New York) and KBOX (Boston, Massachusetts) radars further corroborate these results.

The contribution of this work is primarily the integration of two recently developed techniques—regression-based SZ velocity retrieval and the VRAD velocity recovery framework—into a unified processing methodology termed REG-VRAD. This study does not introduce a new Doppler velocity recovery paradigm. Rather, regression processing provides higher-quality and more spatially extensive short-PRT velocity estimates, which in turn reduces the extent of purple haze regions and enables VRAD to operate more robustly. In addition, a revised censoring strategy is introduced to better identify and retain reliable regression-based velocity estimates prior to VRAD processing. While this censoring approach relies on standard spatial variability metrics and is not novel in isolation, it is essential for the effective integration of the two algorithms. The VRAD methodology itself remains conceptually unchanged, although velocity dealiasing is now performed using the Unfold Radar Velocity (UNRAVEL) algorithm (Louf et al. 2020) to improve robustness, representing an implementation-level refinement rather than a conceptual modification.

The paper is organized as follows. Section 2 gives a brief overview of the RFSF, and a description of the VRAD algorithm is given in section 3. Section 4 describes the combined algorithm of VRAD with SZ RFSF processing. Section 5 shows the four NEXRAD cases processed using REG-VRAD, and those velocities are compared to both level-2 velocities and VRAD using level-2 velocities. In section 6 are the conclusions.

---

## 2. Overview of Regression Processing

Regression filtering and analysis are mathematical tools used for estimating the trends of time series data or scatterplots of data. Typically, the applications are modeling and prediction for the purpose of understanding the relationships between variables (Harrell 2015; Sen and Srivastava 1990). Regression fits are also used to smooth data sequences and obtain the residuals. Much less frequently is regression regarded as a filter with a frequency response that can be used as a high-pass (low-pass) filter as would be infinite impulse response (IIR) or finite impulse response (FIR) filters. This is because a polynomial regression fit cannot be represented as a convolution of the input signal with the impulse response of the regression filter (as in traditional signals and systems theory), and thus, the regression filter is not in the same class as linear time-invariant (LTI) filters. Despite this, the regression filter can be characterized by a frequency response (Torp 1997; Bjaerum et al. 2002; Torres and Zrnić 1999; Hubbert et al. 2025), which can be used for filter design (i.e., selecting the polynomial fit order). Also, because the regression filter does not have an impulse response, the output of the regression filter cannot be represented as a product of the frequency response of the filter and the Fourier transform of the input signal. Nevertheless, regression can be thought of as a high-pass filter (for analyzing the "residuals") or as a low-pass filter (for analyzing the "trend" of a signal) with the trend and the residuals being a function of the selected polynomial order.

One filter application is the automated detection and adaptive global polynomial clutter elimination (ADVANCE) clutter filter. ADVANCE is a regression-based clutter filter which has been shown to be effective in removing ground clutter in weather radar signals (Hubbert et al. 2025, 2024, 2021). It is used as the first step in the REG-VRAD algorithm introduced in this study (see section 4).

Regression can also be used for bandstop filtering as has been shown in HR25. This is accomplished by first selecting the center frequency of the band of frequencies desired to be eliminated. The center frequency is then shifted to 0 Hz using the Fourier frequency shift property (Oppenheim and Schafer 1989):

> **x(t)e^(jω₀t) ↔ X(ω − ω₀)**  &nbsp;&nbsp;&nbsp;&nbsp; (3)

where x(t) is the time signal, X(ω) is the Fourier transform of x(t), ω is the frequency, ω₀ is the frequency shift, j is the square root of −1, and t is the time. After the signal is filtered using the appropriate polynomial order, the frequencies of the filtered signal are then shifted again to their original locations using the Fourier frequency shift property. This filter is called the RFSF, and it can be used to eliminate the ST signal in SZ processing. The RFSF has been shown to provide superior recovery of the WT velocity as compared to the legacy SZ processing (HR25).

The RFSF is used as the second step in the REG-VRAD filter (see section 4). The process of first applying the ADVANCE clutter filter followed by the application of the RFSF will be referred to as "regression processing" throughout the rest of this study. In this paper, the RFSF always operates on 64-point time series as required by the SZ(8/64) algorithm. In HR25, simulations are performed to compare the ability of the RFSF SZ technique to the legacy technique to recover the WT velocity. Those simulations showed that an order-37 regression filter produced a WT velocity recovery region, as a function of WT SW, ST SW, and WT SNR, that is larger than the recovery region for the legacy SZ algorithm and with lower WT velocity SD statistics. In this paper, the RFSF is always order 37.

An example of the ability of the regression processing to recover velocities is demonstrated in Fig. 1 on observations from the KFTG radar. The level-2 velocities (NOAA NWS Radar Operations Center 1991b) obtained with the NEXRAD legacy processing show significant regions of purple haze (black in Fig. 1a). Regression processing of the level-1 time series data (NOAA NWS Radar Operations Center 1991a) is able to recover most of the velocities in the purple haze region (Fig. 1b), and the regression velocities are, in general, spatially smoother (see discussion in HR25). The reasons for the data quality improvements of the regression processing over the legacy processing are outlined in the appendix and in HR25.

> **Figure 1.** A PPI comparison of SZ recovered velocities from (a) legacy processing and (b) regression processing for observations from the KDDC radar collected at 0516:15 UTC 25 May 2020 near Dodge City, Kansas. The black color represents the purple haze region. As discussed in HR25, the RFSF processing recovers more velocities and the areal SD of the RFSF velocities is, in general, lower (HR25).

---

## 3. Overview of the VRAD Algorithm

The VRAD algorithm is designed to extend the availability of valid Doppler velocity estimates in weather radar systems, particularly addressing range–velocity ambiguities and recovering obscured velocities in purple haze regions (Schvartzman and Palmer 2024). VRAD operates on data collected using the split-cut scanning strategy of the NEXRAD Weather Surveillance Radar-1988 Doppler (WSR-88D) system, blending information from the long-PRT surveillance scan and the short-PRT Doppler scan to produce more complete and seamless velocity fields.

VRAD consists of four main stages: preprocessing, dealiasing of the short-PRT velocity field, recovery of missing velocities using long-PRT estimates, and final smoothing of the reconstructed field. The initial preprocessing step ensures baseline data quality. Consistent with NEXRAD radar data acquisition practices, minimum SNR thresholds are applied: at least 2 dB for reflectivity and 3.5 dB for Doppler velocity. This removes low-quality gates that could degrade performance in the subsequent stages.

VRAD addresses two fundamental challenges: correcting aliasing in the short-PRT scan where measured velocities exceed the unambiguous velocity and recovering missing velocity estimates in regions of overlaid returns.

### 3a. Stage 1: Dealiasing the short-PRT velocity field

The first stage focuses on correcting aliasing artifacts in the short-PRT Doppler velocity field. It employs a two-step approach:

**Velocity difference dealiasing:** By leveraging the different PRTs of the surveillance and Doppler scans, VRAD compares the measured velocities from both scans and evaluates their difference against a precomputed set of thresholds. These thresholds are based on the theoretical properties of the velocity difference transfer function, similar to the principles applied in staggered-PRT dealiasing algorithms. The observed velocity differences cluster around discrete levels, allowing for inference of the correct number of ±2Va wraps to restore the true velocity.

**Median filter dealiasing:** While effective over broad regions, the velocity difference approach can fail near the radar where the temporal offset between scans introduces decorrelation. Therefore, VRAD applies a secondary spatial continuity-based dealiasing step. A two-dimensional median filter is used sequentially with window sizes of 5×5, 10×10, and 30×30 gates. This filter identifies isolated aliased gates or small clusters and corrects them based on surrounding valid estimates, ensuring a physically consistent velocity field.

The dealiased short-PRT velocity field then serves as the trusted reference for recovering missing values in the next stage.

In the present implementation, the original VRAD dealiasing procedure described in Schvartzman and Palmer (2024) has been replaced with the UNRAVEL algorithm (Louf et al. 2020). UNRAVEL is a modular, continuity-based dealiasing framework that employs multiple passes and neighborhood consistency checks to robustly correct aliased Doppler velocities. This substitution does not alter the underlying VRAD recovery concept but improves the robustness of the short-PRT reference velocity field, particularly in regions of enhanced shear or reduced spatial continuity. All subsequent VRAD processing stages operate on the dealiased velocity field produced by UNRAVEL.

### 3b. Stage 2: Recovery of missing velocities in range-folded regions

The second stage targets recovery of missing velocities caused by range overlays. These missing purple haze regions often obscure important meteorological signatures. VRAD populates these regions using long-PRT velocities from the surveillance scan, which are heavily aliased but contain usable information. The recovery is governed by a boundary matching condition: Velocities at the edges of missing regions are compared to adjacent valid short-PRT velocities. If the difference exceeds a threshold (typically 5 m s⁻¹), an appropriate multiple of 2Va is added or subtracted to align the long-PRT velocity smoothly with the short-PRT field. The recovery is performed using a "spiral" pattern: Beginning at the perimeter of the missing region, the algorithm progressively unwraps the long-PRT velocities inward, ensuring that continuity is preserved. If no neighboring valid gate is available, the recovery of that particular gate is deferred until subsequent iterations.

### 3c. Stage 3: Final dealiasing

Following recovery, a final median filtering pass is applied to the hybrid velocity field. This suppresses small-scale residual inconsistencies and enhances the spatial coherence of the recovered Doppler velocities.

### 3d. Performance summary

Application of VRAD significantly increases the spatial coverage of Doppler velocity observations in split-cut scans. Experimental evaluation using level-2 and level-1 data indicates an approximate 25% increase in valid velocity estimates in conventional scans and a 12% increase in phase-coded scans (Sachidananda and Zrnić 1999). Such improvements can be operationally significant, particularly for early detection of storm signatures obscured by range-folded returns.

> **Figure 2.** Illustration of the VRAD algorithm applied to data from the KLWX radar collected at approximately 2257:16 UTC 28 Apr 2023. (a) Radar reflectivity, shown for reference. (b) Doppler velocity field from the short-PRT scan, showing censored and range-folded regions in black. (c) Aliased velocities from the long-PRT surveillance scan in the corresponding areas. (d) Recovered VRAD field, showing enhanced spatial coverage of valid Doppler velocities (i.e., no censored or range-overlaid data).

Figure 2 illustrates the VRAD algorithm applied to data collected by the KLWX radar on 28 April 2023. The top-left panel shows reflectivity from the long-PRT surveillance scan for reference, highlighting the precipitation structures associated with the velocity fields. The top-right panel displays the Doppler velocities obtained through legacy processing from the short-PRT Doppler scan, in which significant regions (black areas) indicate velocity data voids resulting from range-overlaid echoes. The bottom-left panel depicts the aliased velocities from the long-PRT surveillance scan, which have a lower Nyquist velocity but provide continuous velocity coverage throughout the range domain. Although these velocities are typically unused operationally, they contain essential velocity information.

The bottom-right panel shows the VRAD-processed velocity field, which integrates the dealiased short-PRT velocities with the appropriately corrected long-PRT velocities to fill in the previously obscured regions. Of particular interest are the outflow velocities associated with storm cells in the northeastern sector of the scan; these velocities, initially obscured by range-overlay contamination, become clearly discernible following VRAD processing. Such enhanced velocity recovery can significantly improve radar-based monitoring of severe weather phenomena.

The modular structure of VRAD permits future enhancements. For example, its integration with regression-based clutter filtering, described in section 2, can further improve data quality by increasing the number of usable velocity estimates and reducing the standard deviation of measurements across meteorological targets.

---

## 4. Combining VRAD with Regression Filtering: REG-VRAD

The performance of VRAD is dependent on the quality and extent of the velocities estimated in the short-PRT scan. Since it has been shown that velocity estimates obtained from the Doppler scan when using regression processing (e.g., Fig. 1) yield more areal coverage and a smoother velocity field (HR25), using VRAD with regression-processed data further enhances VRAD velocity recovery.

The REG-VRAD algorithm consists of six steps:

1. Clutter filter both the long- and short-PRT scans using ADVANCE (if required).
2. Process the SZ-coded short-PRT scan using the RFSF.
3. Stage 1 censor the regression short-PRT scan velocities and long-PRT velocities for VRAD processing (see section 4a).
4. VRAD process the censored regression-processed short-PRT and long-PRT scan velocities.
5. Stage 2 censor the original regression-processed short-PRT scan from step 1.
6. Fill in regions in the VRAD-processed data that were removed with stage 1 censoring but retained in stage 2 censoring.

### 4a. Censoring Algorithms

In the REG-VRAD algorithm, two censoring processes are applied to the regression-processed short-PRT scan. Stage 1 censoring needs to be aggressive to eliminate faulty velocities that could otherwise mislead the VRAD algorithm. Stage 2 censoring can be less aggressive because it is applied at the very end and marginally noisy velocity estimates around the edges of the weather echoes are no longer detrimental to the algorithm.

#### Stage 1 Censoring

Stage 1 censoring is a critical step designed to ensure that only high-quality Doppler velocity observations are provided to the VRAD algorithm for recovery processing. Because VRAD relies on the smoothness of velocity transitions to determine dealiasing and reconstruction, faulty or highly noisy velocity gates could severely degrade its performance if not identified. Thus, a more aggressive censoring approach is adopted during this stage compared to typical quality control procedures. The censoring is applied independently to both the regression-processed short-PRT Doppler scan and the long-PRT surveillance scan. Two primary criteria are used:

1. **SNR thresholds:** For the regression-processed Doppler velocities, the SNR threshold is lowered to 1 dB (compared to 3.5 dB in the legacy processing). This adjustment reflects the improved quality of velocity estimates produced by the regression processing, even at relatively low SNR values. Although lower thresholds (near 0 dB) could be feasible, 1 dB is adopted here as a conservative choice to maintain robustness. For reflectivity, a minimum SNR threshold of 2 dB is retained. Gates failing to meet these criteria are masked and excluded from further VRAD processing.

2. **Velocity spatial smoothness:** To suppress noisy estimates, a local standard deviation check is performed. For each gate, the standard deviation of velocities is computed over a nine-beam azimuthal by five-gate range kernel. If the standard deviation exceeds 8 m s⁻¹, the central gate is flagged and removed.

These criteria collectively ensure that only reliable, meteorological velocity measurements are passed to the VRAD reconstruction. The result is a censored velocity field where noisy, inconsistent, or low-quality gates have been masked, minimizing the risk of dealiasing errors or unphysical transitions when recovering velocities in range-folded regions.

It is important to note that this aggressive stage 1 censoring is only applied during the preparation for VRAD processing. A more relaxed stage 2 censoring, described in the following section, is used later to fill in additional marginal regions after VRAD has reconstructed the core velocity field.

#### Stage 2 Censoring

Stage 2 censoring is based on known statistics that velocity measurements from meteorological scatterers are by and large spatially smooth, while noise is not. We, therefore, calculate the standard deviation of the velocity estimates over a moving kernel of nine rays in azimuth and five range gates. If the standard deviation is higher than 7 m s⁻¹, the data point is deemed noise and, therefore, censored. Note that this is a very basic censoring algorithm that was developed for demonstration purposes only. For operational use, it would likely need to be refined, but this is beyond the scope of the current study.

---

## 5. Experimental Examples of REG-VRAD

Here, we give several examples of REG-VRAD processing using NEXRAD data. Two cases, one from KFTG in Denver, Colorado (CO), and one from KLWX in Sterling, Virginia (VA), are analyzed in detail, and in addition to the REG-VRAD processing, we also apply VRAD to the level-2 data for comparison. Cases from the KBGM, Binghamton, New York (NY), and KBOX, Boston, Massachusetts (MA), radars are also given that further demonstrate how REG-VRAD processing increases the recovered velocity regions over the legacy processing.

### 5a. KFTG Observations of a Spring Rainstorm

KFTG observations were collected around Denver, CO, at 1905:31 UTC 29 March 2022 at 0.488° elevation angle. KFTG was running volume coverage pattern (VCP) 215 which uses a PRT of 3.1067 ms (Va = 8.347 m s⁻¹) and 0.9133 ms (Va = 28.39 m s⁻¹) for the long-PRT and short-PRT scans, respectively. The KFTG long-PRT reflectivity (Fig. 3) shows a spring rainstorm which is mostly stratiform in nature. In the level-2 velocities, the purple haze region is clearly visible as a ring of missing observations at the boundary from the first to the second trip echo (Fig. 4a). VRAD applied to the level-2 velocities is able to recover most of the velocities in the purple haze region (Fig. 4b).

> **Figure 3.** KFTG level-2 long-PRT reflectivity from 1905:31 UTC 29 Mar 2022 at 0.488° elevation.

The regression-processed, but uncensored, velocities (after steps 1 and 2 of the REG-VRAD algorithm have been applied) are shown in Fig. 4c. They show increased velocity coverage not only in the purple haze region but also around the edges of the precipitation region. As mentioned before, VRAD needs low variance, accurate velocities as input and, therefore, requires the more aggressive stage 1 censoring. Using the stage 1 censored velocities obtained in step 3 of the algorithm (Fig. 4d), VRAD is able to completely fill in the purple haze region with the regression-processed long-PRT velocities (Fig. 4e, step 4 of the algorithm). After the less aggressive stage 2 censoring is applied to the original regression-processed short-PRT scan (step 5 of the algorithm), the stage 2 censored velocities are used to fill in regions around the edges of the precipitation region in the final step of the algorithm (Fig. 4f).

> **Figure 4.** KFTG velocity PPIs corresponding to Fig. 3 that demonstrate and compare VRAD with level-2 and REG-VRAD processing: (a) level-2 velocities, (b) level-2 velocities processed with VRAD, (c) regression-processed velocities, (d) regression-processed and stage 1 censored velocities for VRAD processing, (e) REG-VRAD-processed velocities, and (f) REG-VRAD-processed velocities infilled with stage 2 censored velocities. The black rectangle in (a) denotes the area shown in Fig. 5.

Comparing the final product of REG-VRAD (Fig. 4f) with the operationally used level-2 velocities (Fig. 4a) demonstrates a significant difference in data quality and quantity between REG-VRAD and level-2 velocities. **About 25% more velocity data points are recovered** (compared to a 6% increase when using VRAD on level-2 data), and the velocities are spatially smoother with REG-VRAD. The reintroduction of the RFSF-processed data of Fig. 4c into Fig. 4f does allow some velocities that have higher spatial variance than what some NEXRAD data users might like. This would need to be evaluated against the value of having a more complete picture of the velocity field. The censoring techniques offered here are preliminary and need further investigation.

> **Figure 5.** As in Fig. 4, but zoomed in on the region in the black rectangle in Fig. 4a.

To further illustrate the improvement in data quality and quantity from the level-2 velocities to the REG-VRAD velocities, Fig. 5 shows the area encompassed by the black rectangle in Fig. 4a. Because fewer velocity data points are available for processing in the level-2 velocities (Fig. 5a), VRAD applied to the level-2 velocities encounters difficulties around the edges of the echo, which results in an erroneously dealiased region at around 130 km north and 90–120 km east (Fig. 5b). The regression processing is able to recover more velocities (Fig. 5c), and after the stage 1 censoring has been applied (Fig. 5d), it provides more and higher-quality velocities for VRAD to operate on, therefore avoiding the erroneous dealiasing (Fig. 5e). Even more velocities are recovered when the less aggressive stage 2 censored velocities are used to fill in regions around the edges (Fig. 5f).

### 5b. KLWX Observations of a Severe Thunderstorm

KLWX observations were collected around Sterling, VA, at 2206:21 UTC 7 August 2023. KLWX was running VCP 212, which uses a PRT of 3.1067 ms (Va = 8.17 m s⁻¹) and a PRT of 0.986 ms (Va = 25.75 m s⁻¹, Ra = 148 km) for the long-PRT and short-PRT scans, respectively. In the level-2 velocities (Fig. 6a), the purple haze obscures the velocity couplet signature of potential tornado development at around 100 km north and 110 km east. VRAD applied to the level-2 velocities of Fig. 6a, along the accompanying long-PRT velocities (not shown), can recover most of the velocities in the purple haze region (Fig. 6b). The regression-processed short-PRT scan is shown in Fig. 6c with no censoring and shows that regression processing also recovers most missing velocities, and overall, the velocities are smoother. An aggressive censoring (stage 1 censor) is applied to the regression-processed velocities (Fig. 6c), and the result (Fig. 6d) is used as input for VRAD along with the regression-processed long-PRT velocities (not given here). Comparing the resulting velocities from the REG-VRAD algorithm (Fig. 6e) with those from the legacy processing (Fig. 6b) shows that the regression processing allows more velocities to be recovered, and in general, they appear spatially smoother. Figure 6f demonstrates how regression-processed velocities that were only censored with the milder stage 2 censoring can be used to fill in more velocities, mostly around the edges of the precipitation region shown in Fig. 6e. The additional stage 2 censoring restored velocities, combined with the filled in purple haze region velocities (Fig. 6f), result in a **29% increase in velocity data points** as compared to the legacy processing (Fig. 6a) (VRAD applied to the level-2 data leads to an 8% increase (Fig. 6b)).

> **Figure 6.** As in Fig. 4, but for KLWX velocity PPIs at around 2206:21 UTC 7 Aug 2023. The black rectangle in (a) denotes the region shown in Fig. 7.

Zooming in on the velocity couplet (black rectangle in Fig. 6a) shows that REG-VRAD is not only able to recover significantly more velocities but that those velocities are of improved quality (Fig. 7f) as compared to the level-2 velocities (Fig. 7a).

> **Figure 7.** As in Fig. 6, but zoomed in on the region in the black rectangle in Fig. 6a.

To quantify the improvement of data quality, we calculate areal velocity standard deviations (SDs) of Figs. 6b and 6e. The SDs are created by passing a nine beam by five range-bin kernel over the PPIs and calculating the SD over the kernel at each PPI point [as was done for the stage 2 censoring and in Hubbert et al. (2025) and HR25]. Figure 8a shows the difference between the areal SD PPIs (not given here) for REG-VRAD processing minus the VRAD processing using level-2 velocities. Thus, negative SD values indicate that REG-VRAD yields lower SDs. As can be seen, the blue colors for negative SDs dominate the PPI in Fig. 8a. Figure 8b is a histogram of the SD values in Fig. 8a. The histogram shows that there are more negative SD differences and further demonstrates the lower velocity spatial SDs when using regression processing.

> **Figure 8.** A KLWX areal SD difference comparison of VRAD with level-2 and REG-VRAD velocities in Figs. 6b and 6e, respectively. (a) PPI of the areal SD of REG-VRAD-processed velocities minus the areal SD of VRAD with level-2 velocities and (b) the histogram of the SDs in (a). Negative SDs indicate that REG-VRAD has lower SDs.

### 5c. KBGM and KBOX Observations

Two additional cases are now shown that further demonstrate the velocity recovery and data quality improvement when using the new REG-VRAD processing. KBGM observations were collected near Binghamton, NY, at 1838:14 UTC 9 November 2018, and the KBOX case was observed at 1314:24 UTC 8 May 2024 near Boston, MA. While the purple haze region is clearly visible in the level-2 velocities (Figs. 9a,c), it is completely filled in by the REG-VRAD processing (Figs. 9b,d). Both the data quality but especially the quantity of the recovered velocities are significantly improved with the new processing method.

> **Figure 9.** (a),(c) Level-2 velocities and (b),(d) REG-VRAD-processed velocities infilled with stage 2 censored velocities for cases from the (top) KBGM and (bottom) KBOX radars.

---

## 6. Conclusions

Achieving a large unambiguous range for radar variable estimates while simultaneously obtaining a large unambiguous velocity is difficult and is termed the range–velocity dilemma for weather radar. NEXRAD's approach has been to scan a low elevation angle twice, once with a long-PRT scan for unambiguous radar variable estimates out to about 465-km range, followed by a shorter PRT Doppler scan that yields a larger unambiguous velocity interval but is prone to range-overlaid echoes. The overlaid echoes can be separated to some extent by coding the transmit pulses with the SZ(8/64) phases. However, many times the overlaid echoes cannot be separated sufficiently to allow for valid estimates of the weaker trip velocity. Such areas are labeled in purple, i.e., the well-known purple haze, which potentially can obscure consequential velocity signatures.

Recently, two new approaches were described in the literature that significantly reduce the purple haze regions: 1) VRAD processing which uses the velocities from the long-PRT scan to fill in the missing velocities in the short-PRT scan (Schvartzman and Palmer 2024) and 2) SZ velocity recover processing using a regression filter (i.e., the RFSF) (HR25). The SZ RFSF processing has been shown to recover more usable, valid velocities than the legacy processing technique (Sachidananda and Zrnić 1999), and the velocities have lower standard deviations, in general. This study has shown that combining these two techniques can expand the velocity recovery region further than either technique by itself.

The regression-processed velocities, using the RFSF for the short-PRT scan data and the new ADVANCE clutter filter algorithm on both the long-PRT and short-PRT scans, provide the VRAD algorithm more velocities that, in general, have lower SDs. Four NEXRAD cases, KFTG, KLWX, KBGM, and KBOX, were shown that compared VRAD using level-2 velocities and VRAD using regression-processed data (REG-VRAD). **About 25%–30% more velocities could be recovered with REG-VRAD**, and the retrieved velocities were of superior quality (i.e., they had lower spatial standard deviations). The KLWX case demonstrated a scenario where the purple haze in the level-2 data obscured a tornado signature. Using REG-VRAD restored nearly all the velocities in the purple haze region so that the potential tornado signature was clearly visible. The additional cases from the KBGM and KBOX radars further supported the ability of REG-VRAD processing to almost completely fill in purple haze regions. These results demonstrate that the new REG-VRAD algorithm shows great potential to solve the purple haze problem.

---

## Acknowledgments

This work was supported in part by the Radar Operations Center (ROC) of Norman, Oklahoma (Drs. Hubbert and Romatschke). Recently, Dr. Romatschke moved to GeoSphere in Wien, Austria. This material is based upon work supported by the NSF National Center for Atmospheric Research, which is a major facility sponsored by the National Science Foundation under Cooperative Agreement 1852977. Support for Dr. Schvartzman was provided by the University of Oklahoma's Advanced Radar Research Center (ARRC), the School of Meteorology, and the School of Electrical and Computer Engineering. Any opinions, findings, and conclusions or recommendations expressed in this publication are those of the authors and do not necessarily reflect the views of the National Science Foundation.

## Data Availability Statement

The NOAA Next Generation Weather Radar (NEXRAD) level-2 base data are available for download at https://www.ncei.noaa.gov/metadata/geoportal/rest/metadata/item/gov.noaa.ncdc:C00345/html (NOAA NWS Radar Operations Center 1991b). The NOAA NEXRAD level-1 event data (time series) are available for download at https://www.ncei.noaa.gov/metadata/geoportal/rest/metadata/item/gov.noaa.ncdc:C01597/html (NOAA NWS Radar Operations Center 1991a).

---

## Appendix: The Advantages of Regression Processing over NEXRAD Legacy SZ Processing

The legacy SZ algorithm requires the use of a time domain window function such as the von Hann (4.19-dB attenuation) or the Blackman (5.23-dB attenuation) if clutter filtering is required. Because the legacy filters operate on the spectra, the window functions are required to contain spectral leakage (Harris 1978). Thus, one advantage of regression processing is that no window functions are required since processing is done in the time domain, and thereby, window function attenuation is avoided.

It is of interest to first compare the GMAP filter, a window and notch filter, with the regression clutter filter. The regression filter represents the signal in the time domain as a sum of orthogonal polynomial basis functions, which in our case are the Forsythe discrete polynomials (Forsythe 1957; Ruckdeschel 1981; Hubbert et al. 2025). Spectral filters, like NEXRAD's GMAP (Siggia and Passarelli 2004), operate on the discrete Fourier transform (DFT) of the input signal that represents the input signal as a sum of N sinusoids corresponding to the length of the signal, N points. If there is a frequency component in the input signal that does not correspond to one of the N Fourier basis functions, the energy of that frequency component is spread, in general, across the N Fourier basis frequencies. To mitigate this spread of energy, window functions are used. The lowest frequency Fourier basis function (other than a constant) is a one period sinusoid across the N points in the discreet time period. Clutter signals can be very slow varying over the N points and can, for example, resemble a half or quarter period of a sinusoid. Such slow varying signals cannot be represented by one of the Fourier basis functions, and thus, the energy will be spread across all the Fourier basis functions, unless a window function is used. In contrast, such slow varying signals are easily captured by a second- or third-order polynomial fit and then subsequently subtracted from the time series to remove the trend (clutter). Thus, the regression clutter filter, as part of the ADVANCE algorithm, does not use window functions and thus provides a significant advantage for ADVANCE compared to GMAP and other spectral-based filters.

Another effect of the windowing function is that the true underlying SW of the signal is increased, i.e., the underlying signal spectrum is smeared. This is due to the fundamental Fourier property that multiplication in the time domain is equivalently represented as convolution in the frequency domain (Oppenheim and Schafer 1989; Proakis and Manolakis 1988).

A fundamental requirement of SZ processing is that the ST echo is sufficiently attenuated so that estimates of the WT velocity are obtained with a standard deviation (SD) lower than 2 m s⁻¹. It was shown in HR25 that the RFSF can also remove the ST echo without using a window function.

In HR25's Fig. 3, the frequency response of an order-37 and order-47 regression filter are compared to the legacy 0.75 spectrum notch filter. The attenuations of the legacy 0.75 spectrum notch and RFSF for orders 37 and 47 are 6, 3.51, and 4.86 dB, respectively. In this paper, we use an order-37 RFSF, and thus, there is a 3.51-dB attenuation factor. Combining the 0.75 SZ notch width with the von Hann window gives 10.19-dB attenuation for the legacy technique. This attenuation difference between the two techniques, 3.51 dB for the RFSF and 10.19 for the Legacy filter, is the primary reason that the RFSF is able to recover more WT velocity estimates than the legacy technique at a lower SD.

For legacy SZ processing, SZ99 recommended a spectral notch width of 0.75 based on simulations where the WT SW was 4 m s⁻¹ and the WT SNR was 20 dB. In HR25, a broader set of simulation parameters were considered: WT SW of 5, 6, and 7 m s⁻¹ and a WT SNR of 10 dB. Over this broader simulation space, the order-37 RFSF has a larger WT velocity recovery region than the legacy technique.

---

## References

1. Bjaerum, S., H. Torp, and K. Kristoffersen, 2002: Clutter filter design for ultrasound color flow imaging. *IEEE Trans. Ultrason. Ferroelectr. Freq. Control*, 49, 204–216, https://doi.org/10.1109/58.985705.
2. Cho, J., and E. Chornoboy, 2005: Multi-PRI signal processing for the terminal Doppler weather radar. Part I: Clutter filtering. *J. Atmos. Oceanic Technol.*, 22, 575–582, https://doi.org/10.1175/JTECH1730.1.
3. Dazhang, T., S. G. Geotis, R. E. Passarelli Jr., A. L. Hansen, and C. L. Frush, 1984: Evaluation of an alternating-PRF method for extending the range of unambiguous Doppler velocity. *22nd Conf. on Radar Meteorology*, Zurich, Switzerland, Amer. Meteor. Soc., 523–527.
4. Forsythe, G. E., 1957: Generation and use of orthogonal polynomials for data-fitting with a digital computer. *J. Soc. Ind. Appl. Math.*, 5, 74–88, https://doi.org/10.1137/0105007.
5. Harrell, F. E., 2015: *Regression Modeling Strategies, with Applications to Linear Models, Logistic and Ordinal Regression, and Survival Analysis*. 2nd ed. Springer, 582 pp.
6. Harris, F. J., 1978: On the use of windows for harmonic analysis with the discrete Fourier transform. *Proc. IEEE*, 66, 51–83.
7. Hubbert, J. C., and U. Romatschke, 2025: A novel band-stop regression filter with application to range-velocity mitigation for weather radar data. *J. Atmos. Oceanic Technol.*, 42, 1381–1402, https://doi.org/10.1175/JTECH-D-24-0115.1.
8. Hubbert, J. C., G. Meymaris, U. Romatschke, and M. Dixon, 2021: Using a regression ground clutter filter to improve weather radar signal statistics: Theory and simulations. *J. Atmos. Oceanic Technol.*, 38, 1353–1375, https://doi.org/10.1175/JTECH-D-20-0026.1.
9. Hubbert, J. C., S. Ellis, and G. Meymaris, 2024: The mitigation of ground clutter. *Advances in Weather Radar, Volume 1: Precipitation Sensing Platforms*, V. N. Bringi, K. V. Mishra, and M. Thurai, Eds., Institution of Engineering and Technology, 189–230.
10. Hubbert, J. C., G. Meymaris, U. Romatschke, S. Ellis, and M. Dixon, 2025: A new paradigm for automated ground clutter removal: Global regression filtering. *J. Atmos. Oceanic Technol.*, 42, 589–620, https://doi.org/10.1175/JTECH-D-24-0029.1.
11. Louf, V., A. Protat, R. C. Jackson, S. M. Collis, and J. Helmus, 2020: UNRAVEL: A robust modular velocity dealiasing technique for Doppler radar. *J. Atmos. Oceanic Technol.*, 37, 741–758, https://doi.org/10.1175/JTECH-D-19-0020.1.
12. NOAA NWS Radar Operations Center, 1991a: NOAA Next Generation Radar (NEXRAD) Level 1 event data. NOAA National Centers for Environmental Information, accessed 19 April 2022, https://doi.org/10.25921/wj1j-aj43.
13. NOAA NWS Radar Operations Center, 1991b: NOAA Next Generation Radar (NEXRAD) Level 2 base data. NOAA National Centers for Environmental Information, accessed 19 April 2022, https://doi.org/10.7289/V5W9574V.
14. OFCM, 2006: WSR-88D unit description and operational applications. Part D, Doppler radar meteorological observations. Office of the Federal Coordinator for Meteorological Services and Supporting Research, U.S. Department of Commerce/NOAA FCM-H11D-2006, 218 pp.
15. Oppenheim, A., and R. Schafer, 1989: *Discrete-Time Signal Processing*. Prentice Hall, 478 pp.
16. Pirttilä, J., M. S. Lehtinen, A. Huuskonen, and M. Markkanen, 2005: A proposed solution to the range-Doppler dilemma of weather radar measurements by using the SMPRF codes, practical results, and a comparison with operational measurements. *J. Appl. Meteor.*, 44, 1375–1390.
17. Proakis, J. G., and D. G. Manolakis, 1988: *Introduction to Digital Signal Processing*. Macmillan, 944 pp.
18. Ruckdeschel, F., 1981: *Basic Scientific Routines*. Vol. 2. Byte/McGraw-Hill, 790 pp.
19. Sachidananda, M., and D. S. Zrnić, 1999: Systematic phase codes for resolving range overlaid signals in a Doppler weather radar. *J. Atmos. Oceanic Technol.*, 16, 1351–1363.
20. Sachidananda, M., and D. S. Zrnić, 2003: Unambiguous range extension by overlay resolution in staggered PRT technique. *J. Atmos. Oceanic Technol.*, 20, 673–684.
21. Sachidananda, M., D. Zrnić, R. Doviak, and S. Torres, 1998: Signal design and processing techniques the WSR-88D ambiguity resolution: Part 2. NOAA NSSL Tech. Rep., 109 pp.
22. Schvartzman, D., and R. D. Palmer, 2024: Doppler velocity recovery and dealiasing algorithm for multi-PRT scans in weather radars. *IEEE Trans. Geosci. Remote Sens.*, 62, 1–14, https://doi.org/10.1109/TGRS.2024.3406445.
23. Sen, A., and M. Srivastava, 1990: *Regression Analysis: Theory, Methods, and Applications*. 2nd ed. Springer-Verlag, 347 pp.
24. Siggia, A. D., and R. E. Passarelli Jr., 2004: Gaussian model adaptive processing (GMAP) for improved ground clutter cancellation and moment calculation. *Proc. of Third European Conf. on Radar in Meteorology and Hydrology*, Visby, Sweden, ERAD, 67–73.
25. Sirmans, D., D. Zrnić, and B. Bumgarner, 1976: Extension of maximum unambiguous Doppler velocity by use of two sampling rates. *17th Conf. on Radar Meteorology*, Seattle, Amer. Meteor. Soc., 23–28.
26. Tabary, P., L. Perier, J. Gagneux, and J. Parent-du-Chatelet, 2005: Test of a staggered PRT scheme for the French radar network. *J. Atmos. Oceanic Technol.*, 22, 352–364.
27. Torp, H., 1997: Clutter rejection filters in color flow imaging: A theoretical approach. *IEEE Trans. Ultrason. Ferroelectr. Freq. Control*, 44, 417–424.
28. Torres, S. M., and D. S. Zrnić, 1999: Ground clutter canceling with a regression filter. *J. Atmos. Oceanic Technol.*, 16, 1364–1372.
29. Torres, S. M., Y. F. Dubel, and D. S. Zrnić, 2004: Design, implementation, and demonstration of a staggered PRT algorithm for the WSR-88D. *J. Atmos. Oceanic Technol.*, 21, 1389–1399.
30. Zrnić, D., and P. Mahapatra, 1985: Two methods of ambiguity resolution in pulse Doppler weather radars. *IEEE Trans. Aerosp. Electron. Syst.*, AES-21, 470–483.

---

## Relevance to rustywx

Directly addresses the velocity "purple haze" problem visible in NEXRAD Level II data that rustywx displays. Understanding the split-cut scanning strategy and its limitations explains why velocity data may have voids at certain ranges. The clutter filtering techniques (ADVANCE, GMAP) and velocity dealiasing (UNRAVEL) are relevant to improving data quality in the display pipeline. Key findings:
- **KFTG (Denver):** ~25% more velocity data points recovered vs. level-2 (6% with VRAD alone)
- **KLWX (Sterling, VA):** ~29% more velocity data points; revealed tornado signature obscured by purple haze
- **KBGM (Binghamton, NY) and KBOX (Boston, MA):** Purple haze nearly completely filled
- REG-VRAD velocities have lower spatial standard deviations than VRAD with level-2 data
