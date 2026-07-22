# Investigating the outer spiral tornado mechanism in Typhoon Mujigae

**Authors:** Yuchen Liu a,b,c, Qin Su a,b, Lingkun Ran c,d,\*, Zhaoming Li e, Baofeng Jiao c

**Affiliations:**
a. Yunnan Key Laboratory of Meteorological Disasters and Climate Resources in the Great Mekong Subregion, Yunnan University, Kunming, China
b. Department of Atmospheric Sciences, Yunnan University, Kunming, China
c. Laboratory of Cloud-Precipitation Physics and Severe Storms, Institute of Atmospheric Physics (LACS), Chinese Academy of Sciences, Beijing, China
d. University of Chinese Academy of Sciences, Beijing, China
e. Foshan Meteorological Bureau / Foshan Tornado Research Center, Foshan, China

*Corresponding author: Lingkun Ran — rlk@mail.iap.ac.cn*

**Journal:** Atmospheric and Oceanic Science Letters 19 (2026) 100671
**DOI:** https://doi.org/10.1016/j.aosl.2025.100671
Received 8 April 2025; Revised 24 May 2025; Accepted 26 June 2025; Available online 2 July 2025
ISSN 1674-2834 / © 2025 The Authors. Published by Elsevier B.V. on behalf of KeAi Communications Co. Ltd.
Open access under CC BY-NC-ND license (http://creativecommons.org/licenses/by-nc-nd/4.0/)

---

> **⚠️ Source-file notice (read before using this sidecar):**
> The source PDF (`Investigating_the_outer_spiral_tornado_mechanism_in_Typhoon_Mujigae.pdf`, 7 pages per its own metadata) has a **broken/corrupted page tree**. Every extraction tool used (poppler `pdftotext`, `pdfimages`, `pdftoppm`, and the Read tool) throws:
> ```
> Internal Error: xref num 1 not found but needed, try to reconstruct
> Syntax Error: Kid object (page 2) is wrong type (null)
> ```
> After reconstruction, the tools can only recover **5 pages of real content**: the title/abstract page, followed directly by the page bearing the printed page number "4" (Fig. 2 and Eq. 4), page "5" (Fig. 3 and the SRH section), page "6" (Fig. 4, Conclusions, Funding, References start), and page "7" (References, continued). Requesting "page 2" or "page 3" from the file returns the *same* reconstructed content as pages "4"/"5" — there is no recoverable content for the physical pages that should hold printed pages **2 and 3**.
> That means the following material, which the paper's own cross-references and abstract prove existed, **is missing from this PDF file and could not be recovered**:
> - The remainder of Section 1 (Introduction) beyond the first column of page 1.
> - **Section 2 (Data and Methods)** in full — no formulas 1–3 (Eqs. 1–3, e.g. the vorticity-tube / tilting-term derivation referenced by "Eq. (4)" in the recovered text, which explicitly says "See Supplementary Material" implying Eqs. 1–3 preceded it), and no description of the WRF configuration, domain, grid nesting, resolution steps down to 49 m, microphysics/PBL schemes, or initialization/boundary data.
> - **Section 3, subsections 3.1–3.3** (Results) — likely covering the synoptic/typhoon-rainband setup, the observed tornado (4 October 2015, Foshan), model verification against observations, and the vorticity-tube/tilting-term analysis that Eq. (4) and Fig. 3's discussion presuppose.
> - **Figure 1** and its caption — almost certainly the typhoon-track/radar or satellite overview figure and/or the tornado damage/observation figure, standard for this kind of case-study paper. No image object for a "Figure 1" exists anywhere in the file's embedded image list, confirming it is genuinely absent, not just unextracted.
> - Table 1 is referenced once at the end of the SRH discussion ("...the imminent weakening of the tornado Table 1.") but no Table 1 content, caption, or values appear anywhere in the recoverable file.
>
> Everything below is a verbatim, full-fidelity transcription of what **could** be recovered (roughly pages 1, 4, 5, 6, 7 of the original page numbering). Nothing has been invented to fill the gap — per instructions, if the PDF's content is unavailable, this Markdown preserves everything that could be salvaged and flags the loss explicitly rather than guessing. Do not delete/re-fetch the PDF: the same corruption will reproduce on any future re-extraction attempt with these tools; a fresh copy of the source PDF (e.g., re-downloaded from the publisher, see Reference/Citation section below) would be required to recover the missing pages.

---

## Keywords

Tornado; Horizontal vortices; Vorticity equation; Vertical motion equation; Storm relative helicity

关键词: 龙卷风; 水平涡管; 涡度方程; 垂直运动方程; 风暴相对螺旋度

---

## Abstract

To investigate how horizontal vortex tubes influenced the developmental stages of the tornado that formed in the outer rainbands of Typhoon Mujigae in Foshan, China, on 4 October 2015, the authors carried out a high-resolution simulation with the WRF model and derived a pressure-gradient force equation containing horizontal vortex terms. The results show that during the developmental stage of the tornado, the coupling between horizontal vortex tubes and vertical wind shear redistributes the tilting term, whose amplification further enhances the vertical vorticity and intensifies the tornado's horizontal rotation. The vorticity associated with horizontal vortex tubes also modifies the vertical-gradient term of horizontal divergence in the vertical pressure gradient force equation, increasing the vertical pressure gradient force and indirectly fostering stronger upward motion. Throughout the event, persistent positive low-level storm relative helicity indicates that the inflow is largely aligned with the environmental shear vector, providing favorable conditions for the tilting and stretching of horizontal vorticity. Taken together, horizontal vortex tubes affect not only the rotational but also the vertical dynamics of the tornado, while positive storm relative helicity emerges as a potential early-warning indicator for outer-rainband tornadoes spawned by landfalling typhoons.

### 摘要 (Chinese abstract)

基于 WRF 高分辨率模拟及含水平涡管的压力梯度力方程，本文分析 2015年10月4日台风"彩虹"外围雨带龙卷中水平涡管的作用。在龙卷的发展阶段，水平涡管与垂直切变耦合影响倾斜项，进一步增强垂直涡度并增强水平旋转；水平涡管的旋涡度又改变水平辐合的垂直梯度项，增大垂直气压梯度力，促进上升。全过程低层正风暴相对螺旋度与环境切变大体同向，有利水平涡度的倾斜和拉伸。综上，水平涡管既影响龙卷旋转和垂直运动，正风暴相对螺旋度(SRH)或可作为外围雨带龙卷早期预警指标。

*(Note: the Chinese abstract above was reconstructed from a partially garbled/font-substituted OCR layer in the recovered PDF text stream — some characters in the source extraction were unresolved glyphs. The transcription here reflects the readable Chinese characters recovered; a handful of characters that rendered as blank/unrecognizable in the raw extraction were reconstructed from context and may not be pixel-exact to the original typesetting.)*

---

## 1. Introduction

*(Only the first column of page 1 is recoverable; the rest of Section 1 is lost — see the source-file notice above.)*

Tornadoes induced by typhoons are typically embedded within spiral rainbands, where strong low-level shear, buoyant updrafts, and mesoscale convergence coexist (Rasmussen and Blanchard, 1998). The parent convective cells inside these rainbands are short-lived and deeply obscured by clouds and precipitation, making direct observations extremely challenging. Tornadoes themselves are characterized by their small spatial scale, sudden onset, and brief duration; wind speeds within the vortex core can reach or even exceed 140 m s⁻¹ (Davies-Jones et al., 2001).

Previous studies have demonstrated the pivotal role of horizontal vortex tubes in tornado genesis and development. Under the influence of vertical wind shear, these tubes are rapidly tilted and stretched, fostering tornado-scale vortices (Roberts and Xue, 2017; Roberts et al., 2016; Schenkman et al., 2014). Because high-resolution observations were not available in the early years, seminal numerical investigations of supercells laid the dynamical foundation for tornado research and enabled quantitative analyses of the governing physical processes (Klemp and Wilhelmson, 1978; Rotunno, 1979; Schlesinger, 1975). Subsequent work linked near-surface storm relative helicity (SRH) to tornado formation, showing that both low-level SRH and composite tornado parameters can effectively discriminate tornadic from non-tornadic storms (Thompson et al., 2003).

Today, high-resolution simulations have successfully reproduced tornado funnels and identified frictionally generated near-surface shear vortices as a key source of vertical vorticity (Hanley et al., 2016; Schenkman et al., 2014). In China, Wang et al. (2022) reported col-

*[Column 1 of page 1 ends here — the article continues onto the missing pages 2–3, presumably completing the Introduction, then Section 2 "Data and Methods" and Section 3.1–3.3 "Results," none of which are recoverable.]*

### Footnotes (page 1)

- Peer review under the responsibility of Editorial Board of Atmospheric and Oceanic Science Letters.
- \* Corresponding author.
- E-mail address: rlk@mail.iap.ac.cn (L. Ran).

---

## [MISSING: Section 1 continued, Section 2 "Data and Methods," Section 3.1–3.3, and Figure 1]

*(Not recoverable from this PDF — see the source-file notice at the top of this document.)*

---

## 3.3 (continued, recovered) — Vertical pressure-gradient-force equation and horizontal vortex tubes

*(The recovered text below picks up mid-discussion; it references "the Supplementary Material" for the derivation and presents Eq. (4). The lead-in sentence fragment "...plementary Material):" is the tail end of a sentence such as "...the full derivation is provided in the Supplementary Material):" that began on a preceding, missing page.)*

...plementary Material):

**Equation (4):**

```
dη/dt = (ηc_p·λ)/c_v + η(∂v/∂y) + η(∂u/∂x)
        − (∂u/∂z)α − (∂v/∂z)β
        − (p·c_p/ρ·c_v)·[∂/∂z(∂u/∂x + ∂v/∂y)]
        − (p·c_p/ρ·c_v)·[∂/∂z(∂w/∂z)]
        − (∂/∂z)(S_p/ρ)                                          (4)
```

where the first term on the right-hand side of the equation represents the three-dimensional divergence contribution term, λ = ∂u/∂x + ∂v/∂y + ∂w/∂z is the three-dimensional divergence, c_v is the heat capacity ratio at constant volume, and c_p is the heat capacity ratio at constant pressure. The second and third terms on the right-hand side of the equation represent the horizontal divergence contribution term. The fourth and fifth terms on the right-hand side of the equation are the horizontal air pressure gradient force and vertical wind shear coupling term, α = −(1/ρ)(∂p/∂x) is the horizontal pressure gradient force in the x direction, and β = −(1/ρ)(∂p/∂y) is the horizontal pressure gradient force in the y direction. The sixth term on the right-hand side of the equation is the vertical gradient contribution term of the horizontal divergence. The seventh term on the right-hand side of the equation is the vertical gradient contribution term of the vertical divergence. The eighth term on the right-hand side of the equation is the pressure source and sink term: S_p = −(ST·c_p)/(θ·c_v), ST = dθ/dt, θ is potential temperature.

A 49 m resolution simulation and 0.5 km height data were used to calculate the above formula. During the tornado development stage, the individual changing terms of the vertical pressure gradient force — that is, the left end of the equation — is a positive-value area in the low-pressure center, and there are positive and negative large-value areas alternately appearing outside the tornado spiral (Fig. 3(a)). The magnitudes of the three-dimensional divergence contribution term and the horizontal divergence contribution term are similar, which is 10⁻² (Fig. 3(b, c)). The distribution of the three-dimensional divergence contribution term (Fig. 3(b)) is similar to the individual changing terms of the vertical pressure gradient force (Fig. 3(a)). The horizontal divergence contribution term exhibits a significant positive-value region near the low-pressure center and its northern side, whereas a substantial negative-value region is seen on the southern side of the low-pressure center and the outer regions of the tornado spiral (Fig. 3(c)). The magnitude of the coupling term of the horizontal air pressure gradient force and vertical wind shear is 10⁻³, and the regions of highest value are mostly concentrated in the central region of the tornado (Fig. 3(d)). Its phase matches well with the positive and negative magnitudes of the vertical velocity of the tornado. In the rising region, the primary direction of the horizontal pressure gradient force and the vertical wind shear coupling term is negative. Conversely, in the descending region, this term is mainly positive. The magnitude of the vertical gradient contribution term of horizontal divergence and the vertical gradient contribution term of vertical divergence are both 10⁻¹ *(printed in the source as "10°" — likely a typesetting artifact for 10⁻¹, retained here as printed with this note)*. The vertical gradient of horizontal divergence is a positive contribution at the low-pressure center, a negative contribution at the periphery of the low-pressure center, and a negative contribution at the periphery of the tornado spiral. The inside is a positive contribution, and the outside is a negative contribution (Fig. 3(e)). This term is mainly affected by the curl of the horizontal vortex tube. Its distribution range matches the distribution of the horizontal vortex tube, but the positive- and negative-value areas are opposite. The vertical gradient contribution of vertical divergence is opposite to the vertical gradient distribution of horizontal divergence, but the magnitude is smaller than the vertical gradient contribution of horizontal divergence (Fig. 3(f)). The magnitude of the pressure source and sink term is 10⁻³. It has a weak positive contribution at the low-pressure center, a weak negative contribution on the inside of the tornado spiral periphery, and a positive contribution on the outside (Fig. 3(g)).

The vorticity of horizontal vortex tubes influences the vertical gradient term of horizontal divergence in the vertical pressure gradient force equation, forcing the growth of the vertical pressure gradient force and indirectly promoting the development of vertical motion.

### Figure 2

![Fig. 2: vertical vorticity, mean divergence, torsion terms, vertical velocity, pressure gradient force, and buoyancy](media/Investigating_the_outer_spiral_tornado_mechanism_in_Typhoon_Mujigae/fig2-vorticity-terms.png)

**Caption (verbatim):** Fig. 2. Simulated tornado at 49 m resolution and 0.5 km height: (a) vertical vorticity (green solid line; units: s⁻¹), mean divergence term (red solid line; units: 10⁻⁴ s⁻²), and mean torsion term (purple solid line; units: 10⁻⁴ s⁻²) during the tornado's initial stage (0640 UTC), developing stage (0644 UTC), and mature stage (0648 UTC); (b) 0644 UTC vertical velocity (colored; units: m s⁻¹) superimposed on the horizontal wind field (vectors; m s⁻¹); (c) 0644 UTC vertical disturbance pressure gradient force (colored; units: m s⁻²) superimposed perturbed air pressure (contours; units: hPa); (d) 0644 UTC buoyancy (colored; units: m s⁻²) superimposed perturbed air pressure (contours; units: hPa).

**Description:** A four-panel figure. Panel (a) is a line chart with time on the x-axis (0640, 0644, 0648 UTC) and three overlaid series: vertical vorticity (green, roughly flat near 0), mean divergence term (red, rising steeply from about 0 at 0640 to nearly 3.0 at 0648, with a dip near 0644), and mean torsion term (purple, declining from about 1.0 at 0640 to about −1.0 by 0648). Panels (b)–(d) are map plots centered near 22.72–22.76°N, 113.37–113.41°E, each showing the tornado's cyclonic vortex signature: (b) a spiral horizontal wind field (vectors, 30 m/s reference) with a blue-to-red vertical velocity color fill showing a tight rising/sinking couplet at the vortex core; (c) a vertical disturbance pressure gradient force color field (blue negative to red positive, range −0.5 to 0.5 m s⁻²) with black perturbed-pressure contours spiraling into a low center; (d) a buoyancy color field (blue to red, −0.5 to 0.5 m s⁻² range) with the same perturbed-pressure contours, showing a dipole of negative buoyancy north/west and positive buoyancy south/east of the pressure minimum.

---

## 3.4 SRH

The analysis above indicates that horizontal vorticity tubes affect both the rotation and vertical movement of tornadoes, and are important for the dynamical changes in tornadoes. Therefore, does horizontal vorticity have an indicative significance for the development of tornadoes? Coffer et al. (2019) found through experimental analyses that using the 0–500 m SRH can improve the accuracy of tornado forecasts. SRH can quantify the strength of fluid rotation and movement along the rotation direction and plays a vital role in tornado warnings and forecasting. The calculation formula is given as follows (Flournoy and Rasmussen, 2021):

**Equation (5):**

```
SRH = ∫₀ʰ (V_h − C) · ζ_h  dz                                     (5)
```

where **V**_h is the horizontal wind field and **C** = (C_x − C_y) is the relay speed of the storm. Typically, **C** takes 75 % of the average wind speed between the 1.5–7 km air layer and is deflected 40° to the right. **ζ**_h is the horizontal vorticity vector, and h is the air layer thickness, conventionally assumed to be 0.5 km.

Fig. 4 depicts the 0–0.5 km SRH field throughout the tornado life cycle. Positive SRH consistently exceeds its negative counterpart in both magnitude and areal coverage, with the principal positive-SRH lobe persisting in the northeastern quadrant of the tornado, in agreement with previous findings for typhoon-borne tornadoes (Huang et al., 2024). In the genesis stage (Fig. 4(a)), an extensive positive-SRH region appears north of the southeast-oriented reflectivity appendage; its core extends southward and branches, indicating that the near-surface inflow is largely aligned with the ambient shear vector and that vorticity tilting and stretching within the updraft are intensifying (Coffer et al., 2017). During the developing stage (Fig. 4(b)), both the magnitude and coverage of positive SRH increase further, fully encompassing the tornado vortex. Simultaneously, a broader swath of weak negative SRH emerges to the east and northeast. The presence of negative SRH suggests that, under the combined influence of the typhoon rainband and its shear environment, part of the low-level horizontal vorticity becomes antiparallel to the storm-relative flow; nevertheless, its amplitude remains insufficient to curb vorticity amplification, thereby favoring continued tornado intensification. At maturity (Fig. 4(c)), positive SRH strengthens and wraps around the vortex core, while the intensity and extent of negative SRH continue to grow, encircling the positive-SRH region. This configuration diverges from that typically observed in supercell tornadoes, implying a more complex redistribution of vorticity near the tornado base that merits further investigation. In the dissipation stage (Fig. 4(d)), the vortex center is once again embedded within positive SRH, whereas the negative-SRH area shifts to the southeast, signifying the re-establishment of near-surface directional convergence and the imminent weakening of the tornado (Table 1 — *referenced here in the source text but the table itself is not recoverable; see source-file notice*).

### Figure 3

![Fig. 3: individual terms of the vertical pressure gradient force equation](media/Investigating_the_outer_spiral_tornado_mechanism_in_Typhoon_Mujigae/fig3-pressure-gradient-terms.png)

**Caption (verbatim):** Fig. 3. Simulated tornado at 49 m resolution and 0.5 km height at 0644 UTC: (a) individual change terms of vertical pressure gradient force (colored; units: m s⁻³) superimposed pressure field (contours; units: hPa); (b) three-dimensional divergence contribution term (colored; units: 10⁻² m s⁻³) superimposed pressure field (contours; units: hPa); (c) horizontal divergence contribution term (colored; units: 10⁻² m s⁻³) superimposed pressure field (contours; units: hPa); (d) horizontal pressure gradient force and vertical wind shear coupling term (colored; units: 10⁻² m s⁻³) superimposed pressure field (contours; units: hPa); (e) the vertical gradient contribution term (colored; units: m s⁻³) of the horizontal divergence is superimposed on the pressure field (contours; units: hPa); (f) the vertical gradient contribution term (colored; units: m s⁻³) of the vertical divergence superimposed on the pressure field (contours; units: hPa); (g) pressure source and sink terms (colored; units: 10⁻² m s⁻³) superimposed on the pressure field (contours; units: hPa).

**Description:** A seven-panel (a–g, arranged 3×3 with the last cell empty) map figure, all over the same domain (~22.72–22.76°N, 113.37–113.41°E), each panel a colored field (red-white-blue diverging scale, ranges roughly ±0.5 or ±20×10⁻² depending on panel) overlaid with black pressure contours spiraling into a central low. Panels (a)–(d) show strong red/blue dipoles straddling the low-pressure center (the individual pressure-gradient change term, 3-D divergence term, horizontal divergence term, and the pressure-gradient/shear coupling term, respectively); panels (e) and (f) show smaller-magnitude, more diffuse dipole patterns wrapping around the spiral arms (vertical-gradient contributions of horizontal and vertical divergence); panel (g) shows a weaker, more mottled pattern (the pressure source/sink term).

---

## 4. Conclusions

This study employed the WRF model to conduct a high-resolution numerical simulation of the tornado that occurred in Foshan, China, on 4 October 2015, aimed at examining how horizontal vortex tubes influence the different stages of tornado development. The main conclusions drawn from the simulation are as follows:

Horizontal vortex tubes have a pronounced impact on tornado evolution in two primary ways. Firstly, coupling between the horizontal vortex tubes and the horizontal shear of vertical velocity alters the distribution of the tilting terms. These changes further enhance the vertical vorticity, thereby strengthening the tornado's horizontal rotation. Secondly, by deriving and analyzing the vertical pressure-gradient-force equation, it was found that the vorticity associated with horizontal vortex tubes affects the vertical-gradient term of horizontal divergence in this equation, increasing the vertical pressure-gradient force and indirectly fostering the development of vertical motion.

Analysis of the SRH indicated that, from the early stage of tornado genesis, a broad area of positive SRH exists in the study region. This implied that the low-level inflow is generally aligned with the environmental shear vector, favoring the tilting and stretching of horizontal vorticity.

### Figure 4

![Fig. 4: simulated tornado SRH and horizontal vortex tubes through the tornado life cycle](media/Investigating_the_outer_spiral_tornado_mechanism_in_Typhoon_Mujigae/fig4-srh.png)

**Caption (verbatim):** Fig. 4. Simulated tornado at 49 m resolution and 0–0.5 km height SRH (colored; units: m² s⁻²) superimposed with 0.5 km height horizontal vortex tubes (vectors; units: s⁻¹) at (a) 0640 UTC, (b) 0644 UTC, (c) 0648 UTC, and (d) 0652 UTC.

**Description:** A four-panel figure (a–d), each a larger-domain map (roughly 0.2° × 0.2°, centered progressively south/southwest across the four panels as the storm/tornado tracks) showing an SRH color field (blue negative through white/orange to dark red positive, range about −600 to +600 m² s⁻²) overlaid with black horizontal-vortex-tube vectors (scale bar 0.05 s⁻¹) and a thin black contour outlining a low-level reflectivity/pressure feature. Across the four panels (0640→0652 UTC), a positive-SRH lobe (orange/red) is visible tracking with a hook/appendage-shaped feature, growing in areal coverage and intensity from panel (a) to (c) and beginning to separate/weaken by panel (d), consistent with the genesis → developing → mature → dissipating stages described in Section 3.4.

---

## Funding

This research was funded by the Beijing Municipal Science and Technology Commission [grant number Z221100005222012], the Department of Science and Technology of Hebei Province [grant number 22375404D], the National Natural Science Foundation of China [grant numbers U2233218 and 42275010], and the Basic Scientific Program of the Institute of Atmospheric Physics supporting the 14th Five-Year Plan [grant number 7–224151].

## Supplementary materials

Supplementary material associated with this article can be found, in the online version, at doi:10.1016/j.aosl.2025.100671.

## References

- Coffer, B.E., Parker, M.D., Dahl, J.M.L., Wicker, L.J., Clark, A.J., 2017. Volatility of tornadogenesis: An ensemble of simulated nontornadic and tornadic supercells in vortex2 environments. Mon. Wea. Rev. 145 (11), 4605–4625. doi:10.1175/MWR-D-17-0152.1.
- Coffer, B.E., Parker, M.D., Thompson, R.L., Smith, B.T., Jewell, R.E., 2019. Using near-ground storm relative helicity in supercell tornado forecasting. Wea. Forecast. 34 (5), 1417–1435. doi:10.1175/WAF-D-19-0115.1.
- Davies-Jones, R., Trapp, R.J., Bluestein, H.B., 2001. Tornadoes and tornadic storms. Meteorol. Monogr. 28 (50), 167–221. doi:10.1007/978-1-935704-06-5_5.
- Fan, J., Wang, J., Lin, Y., 2023. Urbanization may enhance tornado potential: A single case report. Front. Earth Sci. 11, 1148506. doi:10.3389/feart.2023.1148506.
- Flournoy, M.D., Rasmussen, E.N., 2021. The influence of ground-relative flow and friction on near-surface storm-relative helicity. J. Atmos. Sci. 78 (7), 2135–2142. doi:10.1175/JAS-D-20-0320.1.
- Hanley, K.E., Barrett, A.I., Lean, H.W., 2016. Simulating the 20 may 2013 moore, oklahoma tornado with a 100-metre grid-length nwp model. Atmos. Sci. Lett. 17 (8), 453–461. doi:10.1002/asl.678.
- Houser, J.L., Bluestein, H.B., Snyder, J.C., 2016. A finescale radar examination of the tornadic debris signature and weak-echo reflectivity band associated with a large, violent tornado. Mon. Wea. Rev. 144 (11), 4101–4130. doi:10.1175/MWR-D-15-0408.1.
- Huang, X., Bai, L., Yan, L., Zhang, Z., Cai, K., Zhi, J., 2024. Climatic characteristics and environmental conditions of the tornado occurrences within tropical cyclones over guangdong province, china. Acta Meteorol. Sin. 82 (3), 319–339. doi:10.11676/qxxb2024.20230125.
- Jiao, B., Ran, L., Li, N., Cai, R., Qu, T., Zhou, Y., 2023. Comparative analysis of the generalized omega equation and generalized vertical motion equation. Adv. Atmos. Sci. 40 (5), 856–873. doi:10.1007/s00376-022-1435-5.
- Klemp, J.B., Wilhelmson, R.B., 1978. Simulations of right- and left-moving storms produced through storm splitting. J. Atmos. Sci. 35 (6), 1097–1110. doi:10.1175/1520-0469(1978)035<1097:SORALM>2.0.CO;2.
- Mashiko, W., Niino, H., Kato, T., 2009. Numerical simulation of tornadogenesis in an outer-rainband minisupercell of typhoon shanshan on 17 september 2006. Mon. Wea. Rev. 137 (12), 4238–4260. doi:10.1175/2009MWR2959.1.
- Oliveira, M.I., Xue, M., Roberts, B.J., Wicker, L.J., Yussouf, N., 2019. Horizontal vortex tubes near a simulated tornado: three-dimensional structure and kinematics. Atmosphere 10 (11), 716. doi:10.3390/atmos10110716.
- Rasmussen, E.N., Blanchard, D.O., 1998. A baseline climatology of sounding-derived supercell and tornado forecast parameters. Wea. Forecast. 13 (4), 1148–1164. doi:10.1175/1520-0434(1998)013<1148:ABCOSD>2.0.CO;2.
- Roberts, B., Xue, M., 2017. The role of surface drag in mesocyclone intensification leading to tornadogenesis within an idealized supercell simulation. J. Atmos. Sci. 74 (9), 3055–3077. doi:10.1175/JAS-D-16-0364.1.
- Roberts, B., Xue, M., Schenkman, A.D., Dawson, D.T., 2016. The role of surface drag in tornadogenesis within an idealized supercell simulation. J. Atmos. Sci. 73 (9), 3371–3395. doi:10.1175/JAS-D-15-0332.1.
- Rotunno, R., 1979. A study in tornado-like vortex dynamics. J. Atmos. Sci. 36 (1), 140–155. doi:10.1175/1520-0469(1979)036<0140:ASITLV>2.0.CO;2.
- Schenkman, A.D., Xue, M., Hu, M., 2014. Tornadogenesis in a high-resolution simulation of the 8 may 2003 oklahoma city supercell. J. Atmos. Sci. 71 (1), 130–154. doi:10.1175/JAS-D-13-073.1.
- Schlesinger, R.E., 1975. A three-dimensional numerical model of an isolated deep convective cloud: Preliminary results. J. Atmos. Sci. 32 (5), 934–957. doi:10.1175/1520-0469(1975)032<0934:ATDNMO>2.0.CO;2.
- Tang, J., Tang, X., Xu, F., Zhang, F., 2022. Multi-scale interaction between a squall line and a supercell and its impact on the genesis of the "0612" gaoyou tornado. Atmosphere 13 (2), 272. doi:10.3390/atmos13020272.
- Thompson, R.L., Edwards, R., Hart, J.A., Elmore, K.L., Markowski, P., 2003. Close proximity soundings within supercell environments obtained from the rapid update cycle. Wea. Forecast. 18 (6), 1243–1261. doi:10.1175/1520-0434(2003)018<1243:CPSWSE>2.0.CO;2.
- Wang, Y., Wang, T., Yang, P., Xue, W., 2022. A numerical simulation of the "1907" kaiyuan tornado weather process in liaoning, northeast China. Atmosphere 13 (2), 219. doi:10.3390/atmos13020219.

*(Note: Houser et al. 2016 is listed among the references but was not cited in-text within the recoverable portions of the paper; it may have been cited in the missing Sections 1–3.3.)*

---

## Reference / Citation

Liu, Y., Su, Q., Ran, L., Li, Z., Jiao, B. (2025/2026). Investigating the outer spiral tornado mechanism in Typhoon Mujigae. *Atmospheric and Oceanic Science Letters*, 19, 100671. https://doi.org/10.1016/j.aosl.2025.100671

- **Journal:** Atmospheric and Oceanic Science Letters (KeAi/Elsevier, on behalf of the Institute of Atmospheric Physics, Chinese Academy of Sciences)
- **DOI:** 10.1016/j.aosl.2025.100671
- **Article number:** 100671, Volume 19 (2026)
- **Publication history:** Received 8 April 2025; Revised 24 May 2025; Accepted 26 June 2025; Available online 2 July 2025
- **Open access:** CC BY-NC-ND 4.0
- **Publisher landing page:** https://doi.org/10.1016/j.aosl.2025.100671 (resolves via Elsevier/ScienceDirect and KeAi)
- **ScienceDirect article page:** https://www.sciencedirect.com/science/article/pii/S1674283425000911
- **Elsevier PII:** S1674283425000911
- Also covered in general-audience science press: [Phys.org — "Horizontal vortex tubes may have a significant impact on tornado development"](https://phys.org/news/2025-09-horizontal-vortex-tubes-significant-impact.html); [The Watchers — "New simulations reveal how vortex tubes intensify tornadoes in typhoons"](https://watchers.news/epicenter/new-simulations-reveal-how-vortex-tubes-intensify-tornadoes-in-typhoons/)
- Note: this 2025/2026 paper is a follow-up case study on the same 4 Oct 2015 Foshan tornado event first characterized in Zhao, Y. et al., "Characteristics and Possible Formation Mechanisms of Severe Storms in the Outer Rainbands of Typhoon Mujigae (1522)," *Journal of Meteorological Research*, doi:10.1007/s13351-017-6043-4 — a useful companion reference if more observational/synoptic context is needed to reconstruct the missing Sections 2–3.3 discussed above.

## Media files retained

Located in `media/Investigating_the_outer_spiral_tornado_mechanism_in_Typhoon_Mujigae/`:

- `fig2-vorticity-terms.png` — Figure 2 (native-resolution extracted raster; vertical vorticity/divergence/torsion time series + vertical velocity, pressure-gradient force, and buoyancy maps).
- `fig3-pressure-gradient-terms.png` — Figure 3 (native-resolution extracted raster; seven-panel vertical pressure-gradient-force term decomposition).
- `fig4-srh.png` — Figure 4 (native-resolution extracted raster; four-panel 0–0.5 km SRH + horizontal vortex tube evolution).
- `page-fig2-2.png`, `page-fig3-3.png`, `page-fig4-4.png` — full-page 150 dpi renders of the pages containing Figures 2, 3, and 4 respectively (kept for layout/caption context alongside the cropped figure rasters above).

Three small junk rasters (KeAi logo icon, journal-cover thumbnail icon, and a tiny index-color sliver, all under 35 KB) were extracted by `pdfimages` and discarded as non-content decorative assets.
