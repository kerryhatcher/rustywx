# Feature request: borderless / undecorated window option in `Conf`

**Type:** Feature request
**Severity:** Medium — blocks custom-chrome / kiosk UIs
**Status:** Ready (validated against the bundled `miniquad-ply` 0.4.8)

## Summary

`miniquad::conf::Conf` (via the `miniquad-ply` 0.4.8 fork Ply depends on)
exposes `fullscreen`, `window_resizable`, `window_width/height`, and Wayland
CSD hints — but no way to create a **borderless / undecorated** window. On
macOS the native window style mask is hardcoded in
`src/native/macos.rs`:

```rust
let mut window_masks = NSWindowStyleMask::NSTitledWindowMask as u64
    | NSWindowStyleMask::NSClosableWindowMask as u64
    | NSWindowStyleMask::NSMiniaturizableWindowMask as u64;
if conf.window_resizable {
    window_masks |= NSWindowStyleMask::NSResizableWindowMask as u64;
}
```

There is no code path that omits `NSTitledWindowMask`, so an application can
never get a chromeless windowed surface. The only way to hide OS chrome today
is `window::set_fullscreen(true)`, which takes the whole screen — it can't
produce a *windowed* viewport with custom in-app title/close controls.

## What we need

A `Conf` flag to create the window without OS decorations, e.g.:

```rust
Conf {
    platform: Platform { borderless: true, ..Default::default() },
    ..Default::default()
}
```

On macOS this maps to dropping `NSTitledWindowMask` (borderless style mask);
on Windows to `WS_POPUP`; on X11 to Motif `_MOTIF_WM_HINTS` /
`_NET_WM_WINDOW_TYPE`; Wayland already has CSD hints.

Ideally paired with a runtime `window::set_decorations(bool)` so apps can
toggle chrome without recreating the window.

## Why

Custom-chrome desktop apps (dashboards, kiosk/scope displays, media players)
want to draw their own title bar and window controls. rustywx wants a
chromeless NEXRAD scope with an in-app fullscreen/close control cluster;
right now we can only offer a fullscreen toggle, not a decoration-free
windowed mode.

## Notes / considerations

- Borderless windows lose the OS drag region, so a companion **drag-region**
  hint (or a `window::begin_drag()` call an app invokes from a hit-tested
  title area) is needed for the window to remain movable. Worth designing
  alongside this flag rather than after.
- Prior art: `winit`'s `WindowBuilder::with_decorations(false)` and
  `Window::set_decorations` cover the same surface across all platforms and
  are a good reference for the API shape.

## Workaround in use

rustywx ships a fullscreen toggle (`window::set_fullscreen`) plus an in-app
close button (`window::order_quit`) as the chrome-hiding path. True borderless
windowed mode is deferred pending this upstream capability.
