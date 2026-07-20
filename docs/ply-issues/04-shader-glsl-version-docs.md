# Docs bug: ShaderAsset docs say GLSL ES 3.00 but engine uses 1.00

**Type:** Documentation bug
**Severity:** Medium — causes wasted development time

## Summary

The `ShaderAsset` documentation suggests GLSL ES 3.00, but the engine
actually uses GLSL ES 1.00 (`#version 100`). All built-in shaders (GLOW,
FOIL, HOLOGRAPHIC, etc.) use `#version 100`. Custom shaders written with
3.00 features (e.g., `in`/`out` qualifiers, integer attributes, uniform
blocks) will fail to compile.

## What we observed

During spike work to author a custom Gaussian blur shader, we initially
targeted GLSL ES 3.00 based on the docs. The shader failed to compile.
Switching to `#version 100` with `attribute`/`varying`/`gl_FragColor`
resolved all compilation issues.

## What needs to change

- Update `ShaderAsset` docs to state GLSL ES 1.00 as the target version
- Note the specific 1.00 constraints: `attribute`/`varying` (not
  `in`/`out`), `gl_FragColor` (not custom out variables), constant loop
  bounds on some drivers
- Optionally: if GLSL ES 3.00 is actually supported under certain
  conditions (e.g., WebGL2 only), document those conditions explicitly
