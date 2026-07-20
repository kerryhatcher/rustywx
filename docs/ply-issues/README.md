# Ply Engine — Feedback Issues

Drafted during the rustywx → Ply port (Stage 1, 2025-07-19).
Each file is a self-contained issue ready to file upstream.

## Issues

| # | File | Type | Severity | Title |
|---|---|---|---|---|
| 1 | `01-ttf-parser-unmaintained.md` | Bug / supply chain | High | ttf-parser 0.21.1 is unmaintained (RUSTSEC-2026-0192) |
| 2 | `02-builtin-blur-shader.md` | Feature request | Medium | Built-in Gaussian blur shader |
| 3 | `03-text-input-widget.md` | Feature request | Medium-high | Text input widget |
| 4 | `04-shader-glsl-version-docs.md` | Docs bug | Medium | ShaderAsset docs say GLSL ES 3.00 but engine uses 1.00 |
| 5 | `05-shader-auto-uniforms-undocumented.md` | Docs gap | Medium | u_resolution / u_position auto-uniforms required but undocumented |
| 6 | `06-net-same-id-behavior-undocumented.md` | Docs gap | Low-medium | net API — same-ID re-fire behavior undocumented |
| 7 | `07-net-auto-cleanup-undocumented.md` | Docs gap | Medium | net API — 60-frame auto-cleanup is surprising |
| 8 | `08-headless-testing-mode.md` | Feature request | Medium | Headless testing mode |
| 9 | `09-dropdown-widget.md` | Feature request | Medium | Dropdown / combobox widget |
| 10 | `10-scrollable-list-container.md` | Feature request | Low-medium | Scrollable list container |
| 11 | `11-color-i32-overflow.md` | Ergonomics | Low | Color literals use i32 — values above 0x7FFFFFFF overflow |
| 12 | `12-storage-new-signature-docs.md` | Docs gap | Low | Storage::new() signature mismatch in docs |

## Filing priority

1. **#1** (ttf-parser) — file immediately; affects all downstream apps
2. **#2** (blur shader) — high-value, low-effort for maintainers
3. **#3** (text input) — fundamental primitive, broad demand
4. **#4–#7** (docs) — quick wins, improve developer experience
5. **#8** (headless testing) — larger scope, but high ecosystem value
6. **#9–#10** (widgets) — nice-to-have, can be community-contributed
7. **#11–#12** (ergonomics/docs) — minor, file when convenient
