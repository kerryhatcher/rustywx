# Docs gap: u_resolution and u_position auto-uniforms are required but undocumented

**Type:** Documentation gap
**Severity:** Medium — developers discover this only through errors

## Summary

The Ply renderer automatically sets two uniforms on every shader material:

- `u_resolution` (`vec2`) — the render target dimensions
- `u_position` (`vec2`) — the element's position

Custom shaders **must** declare these uniforms even if they don't use
them. Failing to declare them results in compilation or runtime errors.

This requirement is not documented in the custom shader guide or the
`ShaderAsset` API docs. Developers discover it only when their shader
fails to compile or emits "non-existing uniform" warnings.

## What we observed

Our custom blur shader initially omitted these declarations and produced
warnings. Adding them resolved the issue:

```glsl
uniform vec2 u_resolution;  // auto-set by renderer
uniform vec2 u_position;    // auto-set by renderer
```

## What needs to change

- Document `u_resolution` and `u_position` as required auto-uniforms in
  the custom shader guide
- Include them in any shader starter template / example
- Optionally: make the renderer gracefully handle missing uniforms
  (declare them as unused if the shader doesn't reference them)
