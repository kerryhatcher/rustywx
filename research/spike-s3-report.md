# Spike S3 Report: Ply Composite Widget — Searchable Dropdown

**Date:** 2025-07-19
**Status:** ✅ Complete

**Stage 3 update:** Implemented in `05e4fb7`; some pre-implementation notes below
are superseded by Ply 1.1.1 findings recorded at the end of this report.

## Approach

Built a searchable site selector dropdown using **Ply-native** elements
(Approach A from the de-risking plan). No raw macroquad draw calls needed.

The dropdown consists of:
- A **button** in the top bar showing the current site ID with a ▾ indicator
- A **floating panel** that appears below the button when clicked
- A **filter text** display showing the current filter string
- A **scrollable list** of filtered sites, each as a clickable element
- Keyboard input via macroquad's `get_char_pressed()` and `is_key_pressed()`

## Key Findings

### Ply Id System
Ply's `Id` type implements `From<&'static str>` and `From<(&str, u32)>`.
For dynamically-indexed elements (like dropdown options), use the tuple
form: `.id(("site-opt", idx as u32))`. The same form works with
`ply.is_just_pressed(("site-opt", idx as u32))`.

### Floating Elements
Floating elements work well for dropdown panels. The `.floating()` builder
accepts `.offset((x, y))` for positioning and `.z_index(n)` for layering.
The panel renders above other content without affecting layout flow.

### Keyboard Input
`get_char_pressed()` returns `Option<char>` for the last pressed character.
Works for alphanumeric filter input. `is_key_pressed()` handles special
keys (Backspace, Escape, arrows, Enter).

### Outside-Click Detection
Detecting clicks outside the dropdown is done by checking if neither the
dropdown panel nor the button was pressed on the same frame:
```rust
if is_mouse_button_pressed(MouseButton::Left) {
    if !ply.is_just_pressed("site-dropdown-panel")
        && !ply.is_just_pressed("site-dropdown-btn") {
        // close dropdown
    }
}
```

### Color Literals
Ply's color system uses i32 for hex colors. Values above `0x7FFFFFFF`
overflow. Keep hex literals within i32 range (use `0xRRGGBB` without
alpha, or use separate alpha via `.opacity()`).

## Results

| Metric | Value |
|---|---|
| Approach | Ply-native (no raw macroquad draw calls) |
| Filter performance | Instant with 160+ sites (string comparison, no lag) |
| Keyboard nav | Arrow keys scroll, Enter selects, Escape closes |
| Outside click | Works — closes dropdown |
| Type-to-filter | Works — Backspace to delete, alphanumeric + space + dash |
| Build | Compiles cleanly |

## Code Location

- Branch: `spike/ply-radar-scope`
- File: `ply-spike/src/main.rs` — dropdown state in AppState, UI in top bar, input handling in handle_input()

## Verdict

**Ready for Stage 3.** The Ply-native approach works for composite widgets.
No need for raw macroquad draw calls. The dropdown pattern (button + floating
panel + keyboard input + outside-click detection) is reusable for the tilt
selector and other dropdowns in Stage 3.

## Notes for Stage 3

- [x] Extract dropdown logic into `widgets/dropdown.rs` as a reusable component.
- [x] Check only visible option IDs rather than the full radar-site list.
- [x] Add scroll-wheel support for dropdown navigation.
- [ ] Add a full cursor-bearing text field only if future UX requires it.
- [ ] Consider a scrollbar indicator if usability testing shows it is needed.
- [~] Panel dimensions and offsets are configurable, but available-space-aware
  placement is deferred to Stage 6 responsive layout work.

## Post-Implementation Findings (Stage 3)

- The production radar-site table contains **143** sites, not “160+”. The
  implementation renders and hit-tests at most 12 site rows at once.
- Ply 1.1.1 exposes a native text-input primitive plus focus/value APIs. The
  dropdown retained `get_char_pressed()` because the desired interaction is
  immediate type-to-filter, not a general text editor. The draft text-input
  issue should be updated before filing.
- Reusable Ply widgets work best with a two-phase API: declare elements while
  holding `Ui`, then process `Ply::is_just_pressed(...)` after `show`.
- Dynamic option IDs should use stable source indices. Visible-row indices are
  unsuitable because filtering and scrolling would remap identity each frame.
- An open dropdown must gate global radar input. Otherwise wheel navigation
  also zooms the scope and pointer interaction can pan it.
- Product selection and tilt selection are coupled: switching products resets
  the tilt index because Reflectivity and Velocity may have different sweeps.
- Local end-to-end validation used the release binary under X11, `xdotool` for
  interaction, and screenshots in addition to the mandatory startup smoke test.
