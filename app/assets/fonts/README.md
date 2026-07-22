# Bundled fonts

rustywx ships the following font files. Each is redistributed under its own
license; rustywx's own AGPL-3.0-only license does not apply to these assets.

| File | Family | License | Source |
|------|--------|---------|--------|
| `Inter-Regular.ttf`, `Inter-Bold.ttf` | Inter | SIL Open Font License 1.1 | https://github.com/rsms/inter |
| `SymbolsNerdFontMono-Regular.ttf` | Symbols Nerd Font Mono (v3.4.0) | MIT (Nerd Fonts patches) + component icon sets under their own licenses | https://github.com/ryanoasis/nerd-fonts |

## Notes

- **Inter** — © The Inter Project Authors. SIL OFL 1.1. Used for all body and
  label text.
- **Symbols Nerd Font Mono** — the Nerd Fonts "Symbols Only" build. The Nerd
  Fonts patch tooling is MIT-licensed; the glyphs are aggregated from upstream
  icon sets, each under its own license (SIL OFL 1.1, Apache-2.0, MIT, and
  CC BY 4.0), e.g. Font Awesome, Material Design Icons, Weather Icons,
  Octicons, Devicons, and Powerline. See the upstream LICENSE files:
  https://github.com/ryanoasis/nerd-fonts/tree/master/LICENSE
  Used only for UI icon glyphs (see `src/widgets/mod.rs` — the `nf` module).
