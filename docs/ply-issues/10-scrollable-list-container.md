# Feature/docs: virtualized list support and scroll-to-item guidance

**Type:** Feature request / documentation

**Severity:** Low-medium — scroll containers exist; large dynamic lists remain manual

**Status:** Ready to file after confirming virtualization is not already available

## Summary

Ply 1.1.1 already supports overflow scrolling and scrollbars through
`overflow(|o| o.scroll_y().scrollbar(...))`. It also provides drag-scrolling
control, `scroll_container_data`, and `set_scroll_position`. The original
version of this draft incorrectly claimed that Ply had no scrollable container.

The remaining gap is efficient, keyboard-coordinated rendering of large dynamic
lists. Our 143-site dropdown manually calculates a visible index window,
renders only 12 rows, and keeps the keyboard cursor within that window.

## Current manual pattern

```rust
let max_scroll = filtered.len().saturating_sub(visible_rows);
scroll = scroll.min(max_scroll);
let visible = &filtered[scroll..(scroll + visible_rows).min(filtered.len())];

for &source_index in visible {
    ui.element()
        .id(("site-option", source_index as u32))
        // render row
        .empty();
}
```

The application must also:

- Convert pixel/wheel movement into an item window.
- Keep the keyboard-highlighted row visible.
- Preserve stable IDs while filtering.
- Render a meaningful scrollbar for content that was not declared.
- Scroll to a selected item when the popup opens.
- Handle variable-height rows if they are introduced later.

A normal overflow container cannot automatically virtualize children that were
never declared.

## Requested documentation

Please document the recommended approach for a long selectable list:

- Whether applications should declare every row and rely on clipping/culling.
- The expected cost of declaring hundreds or thousands of children.
- How to call `set_scroll_position` to reveal a keyboard-selected row.
- How to derive a row's offset from `bounding_box` and
  `scroll_container_data`.
- How scrollbars should represent application-virtualized content.
- Touch and drag-scrolling behavior for list selection.

If declaring hundreds of rows is the intended and performant path, a benchmark
or guideline may be enough and this feature request can be closed.

## Optional virtual-list API

For larger data sets, a virtualized list could declare only visible rows while
Ply manages total extent and scroll position:

```rust
ui.virtual_list("site-list", sites.len(), 22.0, |ui, range| {
    for index in range {
        draw_site_row(ui, index, &sites[index]);
    }
});
```

Useful capabilities would include:

- Fixed-height rows initially; variable heights can be deferred.
- Total content extent and a correct scrollbar thumb.
- Mouse wheel and touch drag scrolling.
- `scroll_to_index(index, alignment)` for keyboard navigation.
- Overscan rows to avoid visible popping.
- Stable indexed IDs for rendered children.

## Why this remains separate from a combobox

Virtualized lists also apply to logs, tables, search results, file browsers,
and timelines. A combobox may use this facility, but the underlying capability
has broader value.
