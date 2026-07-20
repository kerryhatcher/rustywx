# Stage 6: "Observatory Look" — Visual Design

**Status:** 🔲 Not started
**Tag:** `v0.4.0-stage6`

## Goal

The full "Observatory" visual identity — frosted glass, animations,
typography, responsive layout.

## Scope

- `widgets/glass_panel.rs` — reusable frosted glass panel wrapper
- Frosted glass styling on all panels. Ply's `built-in-shaders` has GLOW but
  no Gaussian blur — **author a custom GLSL ES 1.00 fragment shader** for the
  blur effect. Apply via `.shader(&BLUR_SHADER, |s| s.uniform("u_radius", 8.0))`.
  Validated by Spike S1.
- Dark gradient background with noise texture (use `GRADIENT_RADIAL` built-in
  or a custom shader)
- Teal/cyan accent color on all interactive elements
- Inter font for UI chrome, JetBrains Mono for scope/data labels.
  Use `FontAsset::Path("assets/fonts/Inter.ttf")` — verify font loading
  works on all platforms. Bundle fonts in `assets/fonts/`.
- Rich animations:
  - Panel slide-in/out with spring physics
  - Hover glow on buttons and interactive elements (use `GLOW` built-in shader)
  - Pulse animation on new data arrival
  - Staggered entrance on app launch
  - Radar sweep line (optional visual flourish)
- Auto-hiding control bar (fades after 3s inactivity)
- Responsive layout:
  - Desktop: top control bar, side NHC panel, bottom status
  - Mobile (<900px): bottom control bar, full-screen NHC panel, larger touch targets
- Loading skeleton while data loads
- Empty states ("No active storms", "No alerts", etc.)
- **Spline-based color tables** — replace the current step-based color lookup
  in `colors.rs` with cubic Hermite spline (Cardinal Spline) interpolation for
  smoother, more professional-looking color ramps. The thesis by Yi Ru (2007)
  uses this approach for interactive transfer functions; the same technique
  applies to static NWS-style color tables. See
  `docs/post-v1-multi-site-animation.md` for the thesis reference.
- **Bilinear range interpolation** — replace the current single-gate range
  lookup in `scope.rs` rasterization with bilinear interpolation across both
  azimuth and range. The current code already interpolates between two
  nearest radials (azimuth axis) but uses a hard gate index for range,
  producing blocky gate-aligned artifacts. Kvasov et al. show that 4-gate
  bilinear interpolation (weighted blend of the curvilinear trapezoid
  formed by 2 adjacent radials × 2 adjacent range gates) gives ~90%
  improvement over nearest-neighbor, with max error only 4.3–6.7%. The
  formula is a simple weighted blend:
  `Z = (1-ξ)(1-η)·Z_ij + (1-ξ)η·Z_i(j+1) + ξ(1-η)·Z_(i+1)j + ξη·Z_(i+1)(j+1)`
  where ξ = relative azimuth, η = relative range. See
  `docs/research/Weatherradardatavisualizationusingfirst-orderinterpolation.md`
  for the full paper.

## Notes from Spike S1

- The blur shader was validated during spike S1 and removed from Stage 1
  as a spike artifact. The shader file (`assets/shaders/blur.frag`) remains
  in the repo. Re-add the `BLUR_SHADER` static and glass panel UI in this
  stage.
- **GLSL ES 1.00** is the target version (not 3.00 as docs suggest — see
  Ply issue #4). Use `#version 100` with `attribute`/`varying`/`gl_FragColor`.
- Custom shaders **must** declare `u_resolution` (vec2) and `u_position`
  (vec2) auto-uniforms even if unused (see Ply issue #5).
- Use `ShaderAsset::Source` with `include_str!` to embed the shader in the
  binary (avoids runtime path issues).
- Constant loop bounds are required in GLSL ES 1.00 on some drivers.

## Dependencies to add

- `ply-engine` features: add `built-in-shaders` and `text-styling`

## Deliverable

App matches the `observatory-mockup.html` look and feel.

## Validation

- [ ] Frosted glass effect visible on panels (custom blur shader working)
- [ ] Control bar auto-hides after 3s, reappears on mouse move
- [ ] Buttons glow on hover with accent color
- [ ] NHC panel slides with spring animation
- [ ] New data triggers subtle pulse
- [ ] Inter font used in UI, JetBrains Mono on scope
- [ ] Resize to <900px — controls move to bottom, NHC goes full-screen
- [ ] Touch targets ≥44px on mobile
- [ ] Loading state shows skeleton/spinner
- [ ] Empty states show helpful messages
- [ ] Spline-based color tables produce smooth gradients (no visible banding)
- [ ] Bilinear range interpolation eliminates gate-aligned blockiness in scope
- [ ] `git push` → CI passes → `git tag v0.4.0-stage6` → `git push --tags`
