# Docs gap: net API — 60-frame auto-cleanup is surprising

**Type:** Documentation gap
**Severity:** Medium — can cause silent data loss

## Summary

The `net` module automatically cleans up requests after 60 frames of not
being polled via `net::request(id)`. If a developer fires a request and
then doesn't poll for 60 frames (e.g., during a loading screen, a modal
overlay, or any state where the request isn't checked), the response
silently disappears.

## What we observed

We discovered this behavior in the API surface research but did not
trigger it during spike work (our game loop polls every frame). However,
it's a likely source of bugs in more complex apps where the polling code
path isn't always executed — for example, when a settings modal is open
and the main render loop skips the net-polling section.

## What needs to change

- Prominently document the 60-frame cleanup in the `net` module docs
- Consider making the timeout configurable (e.g., `net::get(id, url,
  |r| r, timeout_frames: 120)`)
- Consider logging a warning (at least in debug builds) when a request
  is cleaned up without being consumed
