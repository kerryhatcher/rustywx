# Docs/ergonomics: text input state and callbacks are hard to compose

**Type:** Documentation / API ergonomics

**Severity:** Low-medium — the primitive exists, but custom-widget integration is unclear

**Status:** Ready to file after reproducing with a minimal example on Ply 1.1.1

## Summary

Ply 1.1.1 provides `ElementBuilder::text_input`, focus management, and methods
for reading or setting text value, cursor position, and selection. The original
version of this draft incorrectly claimed that no text-input primitive existed.

The remaining problem is composability when a text input belongs to a reusable
stateful widget. `on_changed` and `on_submit` callbacks are stored as `'static`
closures, while the widget's state normally lives in the application state
borrowed for the current frame. It is not obvious from the public examples
whether the intended pattern is:

- Poll `Ply::get_text_value(id)` after the frame.
- Route callbacks through `Rc<RefCell<_>>` or a channel.
- Keep text state inside Ply and synchronize it manually.
- Use another supported event/state pattern.

A documented pattern would help users avoid unnecessary raw macroquad input or
interior-mutability plumbing.

## Context

Our searchable dropdown only needs immediate type-to-filter behavior, so it
currently uses:

```rust
while let Some(c) = get_char_pressed() {
    if c.is_ascii_alphanumeric() || c == ' ' || c == '-' {
        state.filter.push(c);
    }
}
```

That is appropriate for this limited control, but a future accessible search
field will need Ply's cursor, selection, clipboard, focus, and IME behavior.
The migration path should be clear.

## Current API

The builder supports callbacks:

```rust
ui.element()
    .id("search-box")
    .text_input(|t| {
        t.placeholder("Filter sites…")
            .max_length(64)
            .on_changed(|text| println!("changed: {text}"))
            .on_submit(|text| println!("submitted: {text}"))
    })
    .empty();
```

`Ply` also exposes:

- `get_text_value` / `set_text_value`
- `get_cursor_pos` / `set_cursor_pos`
- `get_selection_range` / `set_selection`
- `focused_element`, `set_focus`, and `clear_focus`

## Requested documentation

Please add an example showing a text input embedded in a reusable component
whose value updates ordinary application state. Ideally it should cover:

1. Declaring the input during the `Ui` phase.
2. Reading or receiving changes after `show`.
3. Keeping application state and Ply's stored value synchronized.
4. Submitting a value without `Rc<RefCell<_>>` when possible.
5. Focus and outside-click behavior.
6. Which editing behaviors are supported: clipboard, Unicode/IME, selection,
   multiline input, max length, and password masking.

## Optional API improvement

If polling is the intended approach, a change indicator would make it easier to
avoid comparing strings every frame:

```rust
if ply.text_value_changed("search-box") {
    state.filter = ply.get_text_value("search-box").to_owned();
}
```

Alternatively, an event-returning widget API could make owned application state
the normal path without requiring `'static` callbacks.
