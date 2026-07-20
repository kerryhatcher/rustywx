# Ergonomics: color literals use i32 — values above 0x7FFFFFFF overflow

**Type:** Ergonomics / API design
**Severity:** Low — workaround exists but is a footgun

## Summary

Ply's color system accepts `i32` for hex color literals. Values above
`0x7FFFFFFF` (2,147,483,647) overflow into negative territory and produce
incorrect colors. This affects common RGBA hex values:

| Intended color | Hex literal | Overflows? |
|---|---|---|
| Opaque white | `0xFFFFFFFF` | ✅ Yes |
| Light gray, 80% alpha | `0xCCAAAAAA` | ✅ Yes |
| Semi-transparent red | `0x80FF0000` | ✅ Yes |
| Opaque dark gray | `0x12161e` | ❌ No (fits in i32) |

## Workarounds

```rust
// Workaround 1: Omit alpha, use 0xRRGGBB (always fits in i32)
.background_color(0xFFFFFF)

// Workaround 2: Split the hex to keep high byte small
.background_color(0x1A_FFFFFF)  // works because 0x1A < 0x80

// Workaround 3: Use rgba construction if available
// (not confirmed in Ply API)
```

## Suggested fix

### Option A: Use u32 for color values
Change color parameters from `i32` to `u32`. This is the simplest fix —
`u32` covers the full `0x00000000`–`0xFFFFFFFF` range. May require
updating internal color parsing/conversion.

### Option B: Provide a color! macro
```rust
color!(0xFF, 0xFF, 0xFF, 0xFF)  // explicit RGBA
color!(0xFFFFFF, 0xFF)           // RGB + alpha
color!(0xFFFFFFFF)               // full u32 hex
```

### Option C: Accept hex strings
```rust
.background_color("#FFFFFFFF")
```

## Note

This is a minor issue — the workaround (omit alpha or use `0xRRGGBB`) is
usable for most cases. But it's a classic footgun that will catch
developers off-guard when they try `0xFFFFFFFF` for white.
