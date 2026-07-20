# Spike S3 Report: Ply Composite Widget — Searchable Dropdown

**Date:** 2025-07-19
**Status:** ✅ Complete

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

- Extract dropdown logic into `widgets/dropdown.rs` as a reusable component
- The current implementation iterates all 160+ sites for click detection
  each frame. For Stage 3, only check visible options.
- Add scroll wheel support for the dropdown list
- The filter display is basic (just shows the string). Stage 3 should add
  a proper text input field with cursor.
- Consider adding a scrollbar indicator for long lists
- The dropdown panel height is fixed at 300px. Make it dynamic based on
  available space in Stage 3.
