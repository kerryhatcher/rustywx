# Feature request: scrollable list container

**Type:** Feature request
**Severity:** Low-medium — manual implementation works but is tedious

## Summary

We needed a scrollable list for 160+ radar sites. Ply has no built-in
scrollable container, so we implemented manual scroll tracking:

- Maintain a `scroll_offset: usize` in app state
- Calculate the visible window: `items[scroll..scroll + visible_count]`
- Render only the visible slice as elements
- Arrow Up/Down to adjust `scroll_offset`
- Clamp to `max_scroll = items.len() - visible_count`

This works but lacks:
- **Scroll wheel support** (would need `mouse_wheel().1` integration)
- **Scrollbar indicator** (visual feedback for scroll position)
- **Virtualized rendering** (only render visible items — we do this
  manually, but it should be automatic)
- **Touch scrolling** for mobile (drag to scroll)
- **Smooth scrolling** (animated scroll position)

## What we need

A built-in scrollable container:

```rust
ui.scroll_view("site-list", |s| {
    s.height(300.0)
     .scrollable(true)
})
.children(|ui| {
    for site in &sites {
        ui.element()
            .id(("site-opt", site.idx))
            .text(&site.label, |t| t.font_size(12))
            .empty();
    }
});
```

The container would handle:
- Virtualized rendering (only create elements for visible items)
- Scroll wheel input
- Scrollbar rendering
- Touch drag scrolling on mobile
- Programmatic scroll-to (for keyboard navigation)
