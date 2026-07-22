# Docs/feature: public headless UI testing guide and interaction harness

**Type:** Documentation / testing ergonomics

**Severity:** Medium — the low-level capability exists, but downstream usage is unclear

**Status:** Ready to file against Ply 1.1.1

## Summary

Ply 1.1.1 already provides a headless engine through
`Ply::new_headless(Dimensions)` and uses it extensively in the engine's own
unit tests. The original version of this draft incorrectly requested a
headless mode as though none existed.

What downstream applications still lack is a documented, stable testing story
for custom controls. The useful upstream request is a public guide and, if
needed, small helpers for simulating interaction without reaching into Ply's
private context.

## Existing capability

The public API includes:

- `Ply::new_headless(dimensions)`
- `Ply::begin()` and `Ui::eval()`
- `Ply::pointer_state(position, is_down)`
- `Ply::bounding_box(id)`
- `Ply::is_pressed`, `is_just_pressed`, and `is_just_released`
- `Ply::set_focus` and keyboard activation support
- Programmatic text values, cursor positions, selections, and scroll positions

Ply's own tests use these primitives to validate layout, callbacks, pointer
presses, keyboard activation, and render commands.

## Downstream problem

The APIs are discoverable in source and generated references, but there is no
end-to-end example for testing an application widget. During Stage 3 of our
port, pure filtering and cursor logic could be unit-tested, but mouse/keyboard
integration was validated by launching the release binary under X11 and driving
it with `xdotool`.

That works, but it is slower and more fragile than a headless component test.
It is also unclear which low-level methods are intended as stable public test
interfaces.

## Requested documentation

Please add a testing guide with examples for:

### Layout validation

```rust
let mut ply = Ply::<()>::new_headless(Dimensions::new(800.0, 600.0));
ply.set_measure_text_function(|_, _| Dimensions::new(100.0, 20.0));

let mut ui = ply.begin();
// Declare component.
let commands = ui.eval();

let bounds = ply.bounding_box("site-dropdown-btn").unwrap();
assert_eq!(bounds.height, 24.0);
assert!(!commands.is_empty());
```

### Pointer interaction

Document the frame sequence for:

1. Declaring and evaluating elements so bounding boxes exist.
2. Moving the pointer over an element.
3. Pressing and releasing it.
4. Advancing frames.
5. Querying `is_just_pressed` and callbacks.

### Keyboard and text input

Show how to:

- Focus an element.
- Trigger Enter/Space activation.
- Send characters and editing actions to a focused text input.
- Assert text, cursor, and selection state.

### Scroll containers

Show how to inject wheel/drag input and assert `scroll_container_data` or a
programmatic scroll position.

## Optional helper API

Small public helpers could make tests less coupled to frame-order details:

```rust
harness.click("site-dropdown-btn");
harness.key_press(KeyCode::Enter);
harness.type_text("KTLX");
harness.scroll("site-list", 120.0);
```

A helper crate or documented test utility would be sufficient; a GPU renderer
or pixel-comparison system is not required for the core request.

## Value

A supported headless harness would let downstream projects test dropdowns,
menus, forms, focus behavior, and responsive layout in normal `cargo test`
runs while reserving screenshot tests for visual regressions.
