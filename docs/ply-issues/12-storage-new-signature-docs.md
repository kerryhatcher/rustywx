# Docs gap: Storage::new() signature mismatch in docs

**Type:** Documentation gap
**Severity:** Low — easily worked around but confusing

## Summary

The Ply API surface documentation shows `Storage::new()` with no
arguments:

```rust
let storage = Storage::new().await?;
```

But actual usage requires a namespace string:

```rust
let storage = Storage::new("my_app/data").await?;
```

## What we observed

During spike S8 (storage async integration), we called `Storage::new()`
without a namespace and encountered a compilation error. Adding the
namespace argument resolved it. The namespace determines the storage
subdirectory (e.g., `~/.local/share/my_app/data` on Linux).

## What needs to change

- Update the `Storage::new()` documentation to show the required
  namespace argument
- Document what the namespace maps to on each platform:
  - Linux: `~/.local/share/<namespace>`
  - macOS: `~/Library/Application Support/<namespace>`
  - Windows: `%APPDATA%/<namespace>`
  - Web: OPFS root `/<namespace>`
  - Android: app-specific storage `/<namespace>`
- Clarify whether the namespace is a path (can contain `/`) or a flat
  identifier
