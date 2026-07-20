# Docs gap: net API — same-ID re-fire behavior is undocumented

**Type:** Documentation gap
**Severity:** Low-medium — causes defensive workarounds

## Summary

The `net` module uses string IDs to identify in-flight requests. The
behavior when `net::get("foo", url, ...)` is called while a request with
ID `"foo"` is already in-flight is not documented. Does it:

- Cancel the previous request and start a new one?
- Queue the new request after the previous completes?
- Silently overwrite the previous request's handle?
- Return an error?

## What we observed

We needed to refresh data periodically (re-fetch the same endpoint). Not
knowing the same-ID behavior, we worked around it by using unique IDs per
fetch cycle (appending a sequence number). This works but is unnecessary
boilerplate if the intended behavior is "cancel and replace."

## What needs to change

Document the same-ID re-fire behavior in the `net` module docs. If the
current behavior is "last-write-wins" (overwrite), that's fine — just
state it. If it's undefined or platform-dependent, that should be called
out so developers know to use unique IDs.
