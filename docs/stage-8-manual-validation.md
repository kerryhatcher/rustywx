# Stage 8: Manual Validation Checklist

This document lists hardware-gated validation items that require manual testing on real hardware. These items cannot be verified in CI or by automated agents.

## HiDPI Rendering

**Platforms:** Linux (fractional scaling), macOS (Retina)

**How to test:**
- **Linux:** Enable fractional scaling (e.g., 125%, 150%) in Display settings; launch the app and verify UI renders sharply (no blurry text or icons)
- **macOS:** Run on a Retina display; verify text sizes (16–18px) are legible and crisp, no pixelated rendering

**Pass criteria:** UI elements render without pixelation or blur; text is clearly legible at target sizes; buttons and icons scale smoothly with the display scale factor.

---

## Wayland Native Support (Linux)

**Platform:** Linux with Wayland compositor (GNOME 40+, KDE Plasma 5.20+)

**How to test:**
1. Launch session with `XDG_SESSION_TYPE=wayland`
2. Run the app and check terminal/systemd logs for X11 fallback warnings
3. Verify window positioning, resizing, and fullscreen mode work correctly

**Pass criteria:** No X11 fallback warnings in logs; window chrome (title bar, resize handles) works smoothly; no graphical glitches or crashes.

---

## macOS Native Window (Finder/Dock Launch)

**Platform:** macOS

**How to test:**
1. Build release: `cargo build --release`
2. Locate `target/release/rustywx` (or `.app` bundle if packaged)
3. Launch from Finder (double-click or drag to Dock)
4. Verify:
   - Asset paths resolve correctly (no "file not found" errors for icons, data files)
   - Current working directory is set correctly for data loading
   - Window appears with proper Cocoa chrome (title bar, close/minimize/maximize buttons)

**Pass criteria:** App launches without file-not-found errors; assets load correctly; window behaves like a native macOS app.

---

## Screen Reader Accessibility

**Platforms:** Linux (Orca via AT-SPI), macOS (VoiceOver)

**How to test:**

### Linux (Orca):
1. Enable Orca: `orca --setup` or via Accessibility settings
2. Launch the app with `XDG_SESSION_TYPE=wayland` (preferred)
3. Tab through all interactive elements (buttons, sliders, text inputs if any)
4. Verify Orca announces:
   - Control type (e.g., "button", "slider")
   - Label or purpose
   - Current state (e.g., "toggled", "focused")

### macOS (VoiceOver):
1. Enable VoiceOver: Cmd+F5 or System Preferences → Accessibility → VoiceOver
2. Launch the app
3. Use VO+Right Arrow to navigate; VO+Space to interact
4. Verify all interactive controls are reachable and announced

**Pass criteria:**
- All interactive elements are keyboard-reachable
- Screen reader announces control type, label, and state
- Tab order is logical (left-to-right, top-to-bottom)
- No unlabeled buttons or orphaned controls

---

## Frame Time and Performance

**Platforms:** Linux and macOS on target hardware

**How to observe:**
- **Option 1 (Ply debugging):** Enable frame-time logging if built with debug features
- **Option 2 (Manual observation):** Watch for frame drops or stuttering when:
  - Panning the radar scope
  - Updating sweep data
  - Resizing the window
- **Option 3 (System tools):**
  - Linux: `perf`, Flamegraph, or `htop` to monitor CPU during rendering
  - macOS: Activity Monitor (% CPU), Instruments (Metal profiler)

**Pass criteria:** Consistent 60 FPS (frame time <16.7ms) on target hardware; no visible stuttering during normal interaction. Occasional frame drops on UI resize are acceptable; sustained drops during data display are failures.

---

## Alert-Status Indicator (Optional, Deferred)

**Platforms:** Linux (system tray / notification area), macOS (menu bar)

**Status:** Deferred to post-v1.0 release; marked as optional.

**How to test (future):**
- **Linux:** Add a system tray icon showing alert status (e.g., green = no alerts, red = active); implement via DBus system tray spec or AppIndicator
- **macOS:** Add a menu bar item (NSStatusBar) showing alert count or status

**Pass criteria:** Icon appears in system tray / menu bar; clicking shows alert summary or toggles to full app window.

---

## Platform-Specific Notes

### Linux
- Test on both X11 and Wayland; Wayland is preferred for v1.0
- Verify fractional scaling at 100%, 125%, 150%
- Check Orca integration with both GNOME and KDE Plasma (if resources permit)

### macOS
- Test on Apple Silicon (M1+) and Intel hardware if available
- Verify native window launch via Finder, not just `cargo run`
- Test VoiceOver with both trackpad and keyboard navigation

---

## Sign-Off

When all items pass, update this file with the test date and hardware specs:

- **Tested by:** [name]
- **Date:** [YYYY-MM-DD]
- **Linux hardware:** [e.g., "GNOME 45 on Fedora 39, Intel i7-8700K, 1080p + 4K fractional scaling"]
- **macOS hardware:** [e.g., "macOS 14.2 on M1 Pro, Retina display"]

All items: [✓ Pass / ✗ Fail with notes]
