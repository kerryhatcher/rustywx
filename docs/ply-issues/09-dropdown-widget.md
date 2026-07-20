# Feature request: dropdown / combobox widget

**Type:** Feature request
**Severity:** Medium — commonly needed, significant boilerplate

## Summary

We built a searchable site-selector dropdown from Ply primitives: a
button + floating panel + keyboard navigation + outside-click detection +
type-to-filter. This required ~80 lines of boilerplate that would be
reusable across the ecosystem.

## What we built

The dropdown consists of:
- A **button** showing the current selection with a ▾ indicator
- A **floating panel** that appears below the button when clicked
- A **filter input** (type to narrow the list)
- A **scrollable list** of options, each as a clickable element
- **Keyboard navigation**: arrow keys to scroll, Enter to select, Escape
  to close
- **Outside-click detection** to close the dropdown

## What we need

Even if Ply doesn't ship a full combobox, providing the building blocks
would help:

### Option A: Full built-in widget
```rust
ui.dropdown("site-selector", &mut state.selected_site, &sites, |d| {
    d.searchable(true)
     .placeholder("Select a radar site…")
     .on_select(|site| { /* callback */ });
});
```

### Option B: Helper primitives
- **Outside-click detection**: `ply.is_outside_clicked("my-element")` —
  currently requires manual checking of all element IDs
- **Scrollable container**: a built-in scroll view with scroll wheel
  support (see separate issue: "Scrollable list container")
- **Focus management**: `ply.set_focus("my-element")` / `ply.has_focus("id")`
  for keyboard navigation between elements

## Reusability

This pattern (button + floating panel + list + outside-click) is needed
for: site selectors, tilt/elevation selectors, settings dropdowns,
context menus, autocomplete inputs, and any "choose from a list" UI.
