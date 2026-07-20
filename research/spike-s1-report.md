# Spike S1 Report: Custom GLSL Blur Shader

**Date:** 2025-07-19
**Status:** ✅ Complete

## Approach

Built a single-pass Gaussian blur fragment shader in GLSL ES 1.00 for the
frosted glass effect. Applied via Ply's `.shader()` API which captures the
element and its children to an offscreen buffer, then applies the fragment
shader — exactly what's needed for frosted glass.

Key findings during development:
- Ply's shader system uses GLSL ES 1.00 (`#version 100`), not 3.00 as the
  ShaderAsset docs suggest. The built-in shaders (GLOW, FOIL, etc.) all use
  `#version 100`.
- Auto-uniforms `u_resolution` (vec2) and `u_position` (vec2) are set by
  the renderer on every shader material. Custom shaders MUST declare these.
- `ShaderAsset::Path` loads from the filesystem at runtime. For development
  this works, but `ShaderAsset::Source` with `include_str!` is more robust
  (embeds shader in binary, no runtime path issues).
- The `.shader()` closure must return the builder (no trailing semicolon),
  same as `.floating()`.
- Non-constant loop bounds in GLSL ES 1.00 can cause compilation failures
  on some drivers. Fixed by using constant loop bounds and deriving the
  sample spread from the radius uniform.

## Shader Design

- 5×5 Gaussian kernel (25 samples) with fixed loop bounds
- Radius uniform controls sample spread in UV space
- Blends blurred background with subtle white overlay (glass_alpha = 0.25)
- Preserves child element content alpha on top of the glass

## Results

| Metric | Value |
|---|---|
| Build | Compiles cleanly, no shader warnings |
| Runtime | No "non-existing uniform" warnings (shader compiles successfully) |
| Approach | `ShaderAsset::Source` with `include_str!` (embedded) |
| Kernel size | 5×5 (25 texture samples per fragment) |
| Radius tested | 8.0 (default) |

## Code Location

- Branch: `spike/ply-radar-scope`
- Files:
  - `ply-spike/assets/shaders/blur.frag` — GLSL ES 1.00 Gaussian blur shader
  - `ply-spike/src/main.rs` — glass panel test with G key toggle

## Verdict

**Ready for Stage 6.** The custom GLSL blur shader compiles and runs without
errors. The `.shader()` API works as expected for group-level effects.

## Notes for Stage 6

- The current shader uses a 5×5 kernel which gives moderate blur. For a
  stronger frosted glass effect, consider:
  - Increasing to 7×7 or 9×9 kernel (at cost of more texture samples)
  - Two-pass separable blur (horizontal + vertical) using two nested
    `.shader()` calls — Ply supports multiple shaders on one element
  - Adding noise texture overlay for realistic frosted glass
- The `ShaderAsset::Source` approach embeds the shader in the binary.
  For hot-reloading during development, switch to `ShaderAsset::Path`.
- Frame time impact should be measured on target hardware. The 5×5 kernel
  is conservative; mobile/WASM may need a smaller kernel.
- The glass panel test uses a floating element. In Stage 6, the glass
  effect will be applied to the control bar, NHC panel, and settings modal.
