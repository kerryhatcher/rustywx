# Agent Instructions

## Mandatory: smoke-test before claiming done

Before reporting a task as complete, you MUST verify the app actually
builds and runs — not just that `cargo check` passes. A clean check does
not guarantee the binary launches without panicking.

Run:

```sh
just run &
sleep 3
kill %1 2>/dev/null || true
```

If `just run` fails to compile, panics on startup, or exits immediately
with an error, **do not claim the task is done** — fix the issue first.

This takes 3 seconds and catches:
- Link errors that `cargo check` skips
- Runtime panics (missing assets, bad window config, GPU init failures)
- Feature flag combinations that break the binary

No exceptions. "It compiled" is not the same as "it works."

## Icons

UI icons render from a Nerd Fonts symbol font. The file is
`app/assets/fonts/SymbolsNerdFontMono-Regular.ttf`. It is wired as
`SYMBOL_FONT` in `app/src/widgets/mod.rs`, and the codepoints in use are
named in that file's `nf` module. Inter carries no symbol glyphs, so every
icon must go through this font. Emoji never render, because no color-emoji
font is bundled.

Need an icon that `nf` does not already have? First look for a fitting Nerd
glyph. If none fits, https://iconoir.com/ is a good source. It is
open-source under the MIT license and its thin-line style suits this app.
