# Ply Engine — Feedback Issues

Drafted and revised during the rustywx → Ply port. The current review target is
**Ply 1.1.1**. Recheck the installed/current upstream version before filing,
because several early spike findings were resolved or superseded by newer APIs.

Each numbered file is intended to become a self-contained upstream issue, but
files marked “needs revalidation” should not be filed unchanged.

## Issues

| # | File | Type | Severity | Status | Title |
|---|---|---|---|---|---|
| 1 | `01-ttf-parser-unmaintained.md` | Bug / supply chain | High | Ready | ttf-parser 0.21.1 is unmaintained (RUSTSEC-2026-0192) |
| 2 | `02-builtin-blur-shader.md` | Feature request | Medium | Ready | Built-in Gaussian blur shader |
| 3 | `03-text-input-widget.md` | Docs / ergonomics | Low-medium | Reproduce first | Text input state and callbacks are hard to compose |
| 4 | `04-shader-glsl-version-docs.md` | Docs bug | Medium | Revalidate | ShaderAsset docs say GLSL ES 3.00 but engine uses 1.00 |
| 5 | `05-shader-auto-uniforms-undocumented.md` | Docs gap | Medium | Revalidate | Required shader auto-uniforms are undocumented |
| 6 | `06-net-same-id-behavior-undocumented.md` | Docs gap | Low-medium | Revalidate | net same-ID re-fire behavior is undocumented |
| 7 | `07-net-auto-cleanup-undocumented.md` | Docs gap | Medium | Revalidate | net 60-frame auto-cleanup is surprising |
| 8 | `08-headless-testing-mode.md` | Docs / testing ergonomics | Medium | Ready | Public headless UI testing guide and interaction harness |
| 9 | `09-dropdown-widget.md` | Feature / docs | Medium | Ready | Dropdown/combobox widget or composition guide |
| 10 | `10-scrollable-list-container.md` | Feature / docs | Low-medium | Confirm gap | Virtualized list support and scroll-to-item guidance |
| 11 | `11-color-i32-overflow.md` | Ergonomics | Low | Revalidate | Color literals use i32; values above 0x7FFFFFFF overflow |
| 12 | `12-storage-new-signature-docs.md` | Docs gap | Low | Revalidate | Storage::new() signature mismatch in docs |
| 13 | `13-borderless-window-conf.md` | Feature request | Medium | Ready | Borderless/undecorated window option in Conf (macOS style mask hardcoded) |

## Stage 3 corrections

Reviewing Ply 1.1.1 during the custom-widget stage invalidated three original
premises:

- Ply **does** have a text-input primitive and focus/value/cursor/selection APIs.
- Ply **does** have `Ply::new_headless` and low-level interaction simulation.
- Ply **does** have overflow scroll containers, scrollbars, drag control, and
  programmatic scroll positions.

Issues #3, #8, and #10 have therefore been reframed around the remaining gaps
rather than requesting capabilities that already exist.

No separate Stage 3 issue was added for composite-widget input routing. The
findings about the `Ui` declaration → `show` → `Ply` query lifecycle, stable
dynamic IDs, focus, outside-click behavior, and scroll/input isolation are
included in issue #9.

## Filing priority

1. **#1** — supply-chain issue affecting all downstream applications.
2. **#4–#7 and #12 after revalidation** — focused documentation corrections.
3. **#9** — broad-value combobox/composition request backed by a working
   143-option implementation.
4. **#8** — document and stabilize the existing headless testing path.
5. **#2** — high-value visual feature with a validated downstream shader.
6. **#3 after a minimal reproduction** — clarify the intended state/event
   pattern for reusable text-input components.
7. **#10 after confirming the gap** — virtualization and scroll-to-item support,
   not basic scrolling.
8. **#11 after revalidation** — minor color ergonomics issue.

## Filing checklist

Before filing any draft:

1. Check the latest Ply release and changelog.
2. Reproduce against a minimal project using that version.
3. Verify the relevant API is public and not already documented.
4. Replace rustywx-specific wording with the smallest reproducible example.
5. Include exact crate version, platform, and observed/expected behavior.
