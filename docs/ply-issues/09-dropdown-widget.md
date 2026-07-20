# Feature request: dropdown / combobox widget or composition guide

**Type:** Feature request / documentation

**Severity:** Medium — common control with substantial interaction boilerplate

**Status:** Ready to file against Ply 1.1.1

## Summary

We implemented reusable searchable dropdowns from Ply primitives for a
143-site radar selector and a scan-dependent tilt selector. Ply provides the
necessary foundations—floating elements, indexed IDs, focus APIs, text input,
scroll containers, and pointer-state queries—but composing them into a robust
combobox still requires significant application code.

A built-in dropdown/combobox would be valuable. If that is outside Ply's scope,
a documented composite-widget example plus a small outside-click helper would
address most of the difficulty.

## What the application had to implement

- Button showing the selected value and open/closed indicator.
- Floating panel positioning and z-index.
- Optional type-to-filter state.
- Filtered source-index calculation.
- Visible-window calculation for long lists.
- Stable IDs based on source indices rather than visible-row positions.
- Up/Down, wheel, Enter, Escape, and Backspace behavior.
- Mouse selection and active-row highlighting.
- Outside-click closing.
- Coordination so only one dropdown remains open.
- Input isolation so dropdown wheel/drag events do not also zoom or pan the
  underlying radar view.
- A two-phase API: declare elements through `Ui`, then process
  `Ply::is_just_pressed` after `show`.

The current data set has 143 options. The application renders and hit-tests at
most 12 site rows per frame.

## Option A: built-in widget

```rust
let event = ui.combobox("site-selector", selected_site, &sites, |combo| {
    combo
        .searchable(true)
        .placeholder("Select a radar site…")
        .visible_rows(12)
        .label(|site| format!("{} — {}", site.id, site.name))
});

if let ComboboxEvent::Selected(site) = event {
    select_site(site);
}
```

Desired behavior:

- Mouse, keyboard, and touch interaction.
- Searchable and non-searchable modes.
- Correct focus and accessibility roles.
- Automatic placement above/below according to available space.
- Scroll-to-highlighted item.
- Close on Escape, outside click, selection, or focus loss.
- Stable option identity independent of filtering/virtualization.

An event-returning API is preferable for application state because Ply's
rendering and press-query phases are separate.

## Option B: composition guide and helper primitives

If a full widget is not planned, please document a canonical combobox built
from current APIs. It should explain:

1. The `Ui` declaration → `show` → `Ply` interaction-query lifecycle.
2. Indexed IDs such as `("site-option", source_index as u32)`.
3. Floating-panel anchoring and available-space placement.
4. Focus ownership and keyboard routing.
5. Scroll-container integration and scroll-to-highlighted behavior.
6. Accessibility roles for the button, popup, and options.

A helper for outside-click detection would remove repetitive and error-prone
checks:

```rust
if ply.was_clicked_outside(["site-dropdown-btn", "site-dropdown-panel"]) {
    state.close();
}
```

The helper should treat descendants of the panel as inside and should work with
overlapping floating elements in z-order.

## Reusability

This pattern applies to site selectors, tilt/elevation selectors, settings,
context menus, autocomplete fields, command palettes, and any choose-from-a-
list control.
