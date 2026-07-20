# ttf-parser 0.21.1 is unmaintained (RUSTSEC-2026-0192)

**Type:** Bug / supply chain health
**Severity:** High — affects every downstream app

## Summary

`ply-engine` depends on `ttf-parser 0.21.1` (transitive via
`macroquad-ply` → `fontdue 0.9.3`), which has been flagged as unmaintained
by RustSec ([RUSTSEC-2026-0192](https://rustsec.org/advisories/RUSTSEC-2026-0192)).
There is no safe upgrade available within the current dependency chain.

## Dependency path

```
ttf-parser 0.21.1
  └── fontdue 0.9.3
      └── macroquad-ply 0.4.14
          └── ply-engine 1.1.1
```

## Impact on downstream apps

Every app using `ply-engine` inherits this advisory. Developers must add
an ignore entry in `cargo-deny`, `cargo-audit`, and CI audit jobs — and
re-justify it every time someone reviews the security posture.

## Suggested fix

The RustSec advisory recommends [`skrifa`](https://crates.io/crates/skrifa)
(from Google Fonts' `fontations` project) as an actively maintained
alternative. If `fontdue` is pinned by `macroquad-ply`, the fix may need
to happen there first.

## Workaround

Ignore the advisory in all audit tooling:

```toml
# deny.toml
[advisories]
ignore = ["RUSTSEC-2026-0192"]
```

```yaml
# .github/workflows/ci.yml (rustsec/audit-check action)
ignore: RUSTSEC-2026-0192
```
