# Windows Packaging & Distribution for rustywx

Status: research / recommendation. No Microsoft Store distribution — installer only.

## 1. Executive summary

Ship an **MSI built with `cargo-wix`**, unsigned at first, on a `windows-latest` GitHub Actions
runner triggered off release-please tags. `cargo-wix` is Rust-native, needs no new CI dependency
(the legacy WiX Toolset v3.14.1 is preinstalled on the runner today), and produces a real MSI with
a Start Menu shortcut and an uninstall entry — the right default for an AGPL-3.0 Rust GUI app with
no existing Windows packaging.

Ship the normal dynamic-CRT build; do **not** add `+crt-static`. The static-CRT flag is a known
hazard on the MSVC target: as soon as any dependency links the CRT dynamically, the two linkage
models collide (duplicate allocators, mismatched runtime state), and the only thing it buys is
dropping the one runtime DLL a dynamic build needs — `vcruntime140.dll`, which is already present
on effectively every Windows 10/11 machine. DirectX 12, the GPU backend, needs no redistributable
at all on Windows 10 1607+/Windows 11 (see §5). The size and dependency savings are marginal and
not worth debugging linker failures for.

Sign later, not never: unsigned OSS binaries trigger a SmartScreen "Windows protected your PC"
warning until enough people run the hash. Skip EV certs — as of March 2024 Microsoft removed EV's
instant-reputation shortcut, so EV no longer buys faster SmartScreen trust than a cheaper OV cert
(see §4). **Azure Artifact Signing** (Microsoft's rename of "Trusted Signing," Basic tier ~$9.99/mo)
is the cheapest path to a real Authenticode signature once the project wants one.

Effort: ~4 hours / same-week for the unsigned MSI pipeline; +1–2 weeks and a recurring ~$10/mo if
signing is added.

## 2. Installer tooling comparison

| Tool | Output | Language | CI fit | Notes |
|---|---|---|---|---|
| **cargo-wix** | MSI | Rust cargo subcommand, wraps WiX XML | Legacy WiX v3.14.1 preinstalled on `windows-latest` (currently the `windows-2025` image); zero extra install | Recommended. Native to the Rust toolchain, formal MSI (org/enterprise-friendly): `cargo install cargo-wix`, then `cargo wix init` once and `cargo wix` to build |
| Inno Setup | .exe installer | Pascal script (`.iss`) | No official GitHub-hosted preinstall; must `choco install innosetup` or download in CI | Smaller installer (~2-3MB overhead vs MSI), very popular for indie/OSS Windows apps, more CI setup work |
| NSIS | .exe installer | NSIS scripting | Cross-compilable from Linux (`makensis` on Ubuntu), used by Tauri and RustDesk | Good if a maintainer ever wants to build the Windows installer from a non-Windows CI runner |
| cargo-packager | MSI/NSIS/deb/AppImage/dmg | TOML config, CrabNebula-maintained | Wraps WiX or NSIS under the hood; single tool for multi-platform config | Worth a look if rustywx later wants one config for macOS/Linux too — but it's an added dependency for a Windows-only need today |
| cargo-bundle | — | — | — | Notably weak/unmaintained for Windows target; not recommended here |

**Recommendation:** `cargo-wix` now. Revisit `cargo-packager` only if/when macOS and Linux
installers are built from one shared config — not needed for this task.

## 3. Build pipeline

- **Target:** `x86_64-pc-windows-msvc` (the standard MSVC ABI target; GNU target is not worth the
  complexity for a GUI app that only needs to run on Windows).
- **Static vs. dynamic CRT — use dynamic (the default).** Enabling `+crt-static` (via
  `RUSTFLAGS`/`.cargo/config.toml`) statically links the MSVC CRT so the binary no longer needs
  `vcruntime140.dll`. The trade is not worth it here: static and dynamic CRT linkage cannot be
  mixed safely, so the flag risks runtime and linker breakage the moment a dependency links the
  CRT dynamically, and the only benefit is dropping a DLL that ships on essentially every modern
  Windows install. Leave the flag off and ship the standard dynamic-CRT build. Do **not** pin the
  target in a repo-wide `.cargo/config.toml` either — that would force the Windows target onto the
  Linux CI jobs (§6) and break them; select the target per-command instead.
- **Icon + version embedding:** use the `winresource` crate in `build.rs` to embed the `.ico` and
  version info (`FileVersion`/`ProductVersion`) into `rustywx.exe`. Use `winresource`, **not** the
  original `winres` crate — `winres` is unmaintained and fails to build on Rust 1.61 and newer,
  which rules it out for this edition-2024 crate. The repo's icons live in `app/assets/icon/` as
  PNGs only, with no `.ico` yet. Convert once from the largest source with ImageMagick 7:
  `magick app/assets/icon/icon_1024.png -define icon:auto-resize=256,128,64,48,32,16 app/assets/icon/rustywx.ico`
  (or any multi-resolution PNG-to-ICO tool), then commit the generated `.ico` alongside the PNGs.
- **Shortcuts/uninstaller:** cargo-wix's generated `main.wxs` template already wires up a Start
  Menu shortcut and registers the app in "Apps & features" (standard MSI uninstall), so no work is
  needed beyond filling in the template (§7).

## 4. Code signing & SmartScreen

| Path | Cost | UX impact | Notes |
|---|---|---|---|
| Unsigned | $0 | SmartScreen "Windows protected your PC" click-through warning until reputation builds (typically ~6–12 weeks of real downloads) | Normal for OSS; fine for v1 |
| OV certificate | ~$200–300/yr | Signed, but builds SmartScreen reputation the same organic way as EV since March 2024 | Traditional CA-issued cert, needs private-key custody (HSM or USB token) |
| **Azure Artifact Signing** (formerly "Trusted Signing") | Basic tier ~$9.99/mo (5,000 signatures/mo), Premium ~$99.99/mo (100,000/mo) — [Microsoft pricing](https://azure.microsoft.com/en-us/pricing/details/artifact-signing/) | Same reputation-building path as OV, no local key custody | Microsoft's own recommended replacement for standalone code-signing certs; signs via `azuresigntool`/Azure CLI in GitHub Actions, no hardware token to manage. Renamed from "Trusted Signing" to "Artifact Signing" (GA in the US, Canada, EU, and UK); the identity-validation step now accepts self-employed individuals, not only organizations with 3+ years of history — so a solo maintainer can enroll |

**Verified: EV no longer instant-bypasses SmartScreen.** Since a March 2024 Microsoft Trusted Root
Program change, EV and OV certificates build SmartScreen reputation identically, based on download
volume — EV's old "signed = instantly trusted" shortcut was retired specifically because malware
operators had turned EV certs into a commodity. Paying an EV premium purely for faster SmartScreen
trust is no longer justified. ([ToDesktop: "EV Certs do not grant immediate reputation anymore"](https://www.todesktop.com/blog/posts/windows-apps-psa-ev-certs-do-not-grant-immediate-reputation-anymore),
[Microsoft Learn: SmartScreen reputation for Windows app developers](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation))

Sigstore is a code-transparency system for supply-chain provenance, not an Authenticode signer —
it does not satisfy Windows SmartScreen/Authenticode and is not a substitute here.

**Recommendation:** ship unsigned for the first release(s); add Azure Artifact Signing
(~$10/mo) once there's a release cadence worth protecting — it is cheaper and less operationally
heavy than an OV cert (no key custody) and gives up nothing EV would have provided post-2024.

## 5. wgpu on Windows

- rustywx's `ply-engine`/wgpu stack targets DirectX 12 as its native Windows backend. DX12 has
  shipped in-box since Windows 10 version 1607 and is present in Windows 11, so **no
  redistributable is required** for the graphics backend itself. This is a separate question from
  the CRT (§3): the dynamic-CRT build depends on `vcruntime140.dll`, which is present on
  essentially every Windows 10/11 machine but is technically supplied by the VC++ redistributable
  rather than the OS. In practice that DLL is ubiquitous enough that shipping without bundling the
  redistributable is the norm for Rust GUI apps; if a truly pristine target image is ever a
  concern, bundle the VC++ redistributable rather than reaching for `+crt-static`.
  Vulkan remains available as an optional fallback backend if DX12 is ever unavailable, but it is
  not the default path.
- wgpu's `static-dxc` feature statically links the DXC shader compiler if that dependency ever
  needs pinning down; not required unless a runtime DXC-loading issue actually shows up.

## 6. GitHub Actions CI

Build on `windows-latest`, triggered by the tag `release-please` pushes on a version bump:

```yaml
release-windows:
  runs-on: windows-latest
  needs: release-please   # or: on tag push from release-please's release
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - run: cargo install cargo-wix --locked
    # cargo-wix runs `cargo build --release` itself, so no separate build step is needed.
    # `windows-latest` is already an x86_64-pc-windows-msvc host, so no --target is required;
    # run from app/ so cargo-wix picks the rustywx package out of the virtual workspace.
    - run: cargo wix
      working-directory: app
    - uses: softprops/action-gh-release@v2
      with:
        files: target/wix/*.msi
```

This repo's existing `.github/workflows/ci.yml` runs fmt/clippy/check/test on Ubuntu only — the
Windows MSI job above is new and additive, not a replacement for the existing CI. No
`release-please` config currently exists in the repo; adding one (or wiring this job to
`push: tags: ['v*']`) is a prerequisite, not part of this packaging task.

## 7. Concrete config

No `.cargo/config.toml` is needed. Do not pin the build target there: this is a cross-platform
repo whose CI also builds on Linux (§6), and a repo-wide `target = "x86_64-pc-windows-msvc"` would
force the Windows target onto those jobs and break them. The Windows runner is already an MSVC
host, so it builds the right target with no configuration.

**`app/build.rs`** (new file) — embed the icon and version via `winresource`. Add
`winresource = "0.1"` under `[target.'cfg(target_os = "windows")'.build-dependencies]` in
`app/Cargo.toml`. The icon path is relative to `app/`, where `build.rs` lives:

```rust
fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon/rustywx.ico");
        res.set("FileVersion", env!("CARGO_PKG_VERSION"));
        res.set("ProductVersion", env!("CARGO_PKG_VERSION"));
        res.compile().expect("failed to embed Windows resources");
    }
}
```

**`app/wix/main.wxs`** — generate it once by running `cargo wix init` from inside `app/` (so the
`wix/` folder is created in the `rustywx` package, not at the virtual-workspace root), then
hand-edit: set `<Product Name>`, `Manufacturer`, and `UpgradeCode` (generate one GUID and keep it
stable across releases — this is what lets MSI upgrades replace rather than duplicate-install),
and the Start Menu shortcut component pointing at `rustywx.exe`.

## 8. Effort & phased rollout

- **Phase 1 (unsigned MSI, ~4 hours / same week):** PNG-to-ICO conversion, `winresource` in
  `app/build.rs`, `cargo wix init` plus `main.wxs` edits, GitHub Actions job, attach the MSI to the
  GitHub Release.
- **Phase 2 (signed, +1–2 weeks, ~$10/mo recurring):** set up Azure Artifact Signing, wire
  `azuresigntool`/Azure CLI signing step into the CI job before the MSI/artifact upload, verify the
  signature with `signtool verify`.
- Do not attempt `+crt-static` (§3) or Microsoft Store packaging (out of scope per project facts)
  in either phase.

## 9. Sources

- [volks73/cargo-wix](https://github.com/volks73/cargo-wix) — cargo subcommand for WiX-based MSIs; the README documents `cargo wix init` / `cargo wix`, the `target\wix` output path, the `-p/--package` (Builder `package()`) workspace selector, and notes that as of April 2026 `windows-latest` still ships only the legacy WiX v3.14.1
- [WiX Toolset v4/v5 tutorial (FireGiant)](https://docs.firegiant.com/wix/tutorial/) — modern `wix.exe` is a .NET tool, separate from the legacy v3.14 toolchain that ships on `windows-latest`
- [jrsoftware.com/isinfo.php](https://jrsoftware.com/isinfo.php) — Inno Setup
- [docs.crabnebula.dev/packager](https://docs.crabnebula.dev/packager) — cargo-packager
- [v2.tauri.app/distribute/windows-installer](https://v2.tauri.app/distribute/windows-installer/) — NSIS/WiX usage precedent (Tauri)
- [BenjaminRi/winresource](https://github.com/BenjaminRi/winresource) — maintained fork of the unmaintained `winres` crate; `winres` no longer builds on Rust 1.61+
- [Azure Artifact Signing pricing](https://azure.microsoft.com/en-us/pricing/details/artifact-signing/) — Basic $9.99/mo (5,000 sig/mo), Premium $99.99/mo (100,000 sig/mo); service renamed from "Trusted Signing"
- [Artifact Signing FAQ (Microsoft Learn)](https://learn.microsoft.com/en-us/azure/artifact-signing/faq) — rename from Trusted Signing, availability, and individual/self-employed eligibility
- [ToDesktop: EV certs do not grant immediate reputation anymore](https://www.todesktop.com/blog/posts/windows-apps-psa-ev-certs-do-not-grant-immediate-reputation-anymore) — March 2024 Microsoft Trusted Root Program change
- [Microsoft Learn: SmartScreen reputation for Windows app developers](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation)
- [The Rust Reference: `crt-static`](https://doc.rust-lang.org/reference/linkage.html#static-and-dynamic-c-runtimes) — static vs. dynamic CRT linkage on MSVC
- [users.rust-lang.org: Statically linking to CRT on MSVC](https://users.rust-lang.org/t/statically-linking-to-crt-on-msvc/9755)
