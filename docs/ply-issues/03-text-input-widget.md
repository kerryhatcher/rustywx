# Feature request: text input widget

**Type:** Feature request
**Severity:** Medium-high — fundamental UI primitive

## Summary

Ply has no built-in text input field. We needed a filter input for a
searchable dropdown and had to build it using raw macroquad APIs:

```rust
// Character input
if let Some(c) = get_char_pressed() {
    if c.is_ascii_alphanumeric() || c == ' ' || c == '-' {
        state.filter.push(c);
    }
}
// Backspace
if is_key_pressed(KeyCode::Backspace) && !state.filter.is_empty() {
    state.filter.pop();
}
```

This is the most basic possible text input and it still required reaching
below Ply's abstraction layer into macroquad. A real text input needs
cursor positioning, selection, clipboard support, placeholder text, max
length, and IME handling — none of which are trivial to implement
correctly.

## What we need

A Ply-native text input element:

```rust
ui.element()
    .id("search-box")
    .text_input(&mut state.filter, |t| {
        t.placeholder("Filter sites…")
         .font_size(13)
    });
```

Minimal features for a first version:
- Accepts a `&mut String` as the buffer
- Keyboard character input (printable ASCII)
- Backspace, Delete, arrow-left/right (cursor movement)
- Enter key (submit / on_submit callback)
- Placeholder text when empty
- Focus / blur states (click to focus, click-outside to blur)

Nice-to-have for a later version:
- Selection (shift+arrow, double-click word)
- Clipboard (Ctrl+C / Ctrl+V)
- Max length validation
- IME / multi-byte input
- Password mode (masked characters)
