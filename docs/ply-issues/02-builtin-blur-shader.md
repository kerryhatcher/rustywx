# Feature request: built-in Gaussian blur shader

**Type:** Feature request
**Severity:** Medium — commonly needed UI effect

## Summary

The `built-in-shaders` feature includes FOIL, HOLOGRAPHIC, DISSOLVE, GLOW,
CRT, and three GRADIENT_* shaders — but no Gaussian blur. Blur is one of
the most common UI effects (frosted glass, background blur, depth-of-field,
shadow softening). Every app needing these effects must author a custom
GLSL fragment shader from scratch.

## What we need

A built-in `BLUR` shader (or `GAUSSIAN_BLUR`) that works with the existing
`.shader()` API:

```rust
ui.element()
    .shader(&BLUR_SHADER, |s| {
        s.uniform("u_radius", 8.0);
    })
```

A `u_radius` uniform controlling blur strength would cover the majority of
use cases.

## Validation

We prototyped a 5×5 single-pass Gaussian blur during spike work. It compiles
and runs cleanly in Ply's `.shader()` pipeline (which captures the element
to an offscreen buffer, then applies the fragment shader). The existing
renderer infrastructure already supports everything needed — it's just a
matter of shipping the shader as a built-in.

## Design considerations

- **5×5 kernel** (25 samples) is a good default — moderate blur, good perf.
- **Separable two-pass** (horizontal + vertical) would give stronger blur
  with fewer samples, but may require two nested `.shader()` calls.
- **GLSL ES 1.00** is the target (see issue: "Shader docs say GLSL ES 3.00
  but engine uses 1.00"). Constant loop bounds are required for some
  drivers; the radius uniform should control sample spread, not loop count.
