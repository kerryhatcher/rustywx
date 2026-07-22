# Linux Packaging for rustywx

`rustywx` is a Rust GPU-GUI desktop app (ply-engine/wgpu, Vulkan backend on Linux),
crate/binary `rustywx` v0.7.1, edition 2024, licensed AGPL-3.0-only. Geolocation is
macOS-only — no Linux geoloc path to worry about. App ID for all store/sandbox
formats: `io.github.kerryhatcher.rustywx`. Repo: github.com/kerryhatcher/rustywx.
CI is GitHub Actions with release-please managing versions.

This doc consolidates three independent format investigations (Flatpak/Flathub,
AppImage+.deb+.rpm, Snap) into one plan, and resolves the one real contradiction
between reports: how GPU access works under a sandbox.

## 1. Executive summary

| Format | Effort (first ship) | Reach | Sandbox | GPU/Vulkan risk | Store review |
|---|---|---|---|---|---|
| **.deb** | Low (~half day) — `cargo-deb` | Debian/Ubuntu + derivatives (largest desktop-Linux slice) | None (system install) | Low — links host Vulkan loader directly | None (self-hosted on GitHub Releases) |
| **AppImage** | Low-Medium (~1 day) | Distro-agnostic, run-anywhere | None (portable binary) | Low-Medium — must NOT bundle libGL/libvulkan/Mesa, resolve from host | None; zsync for updates |
| **.rpm** | Low (~half day) — `cargo-generate-rpm` | Fedora/RHEL/openSUSE | None | Low — same as .deb, host Vulkan loader | None (self-hosted) |
| **Flatpak/Flathub** | Medium (1-3 days + review wait) | Broad, cross-distro, discoverable via Flathub/GNOME Software | Strict, opt-in via finish-args (`--device=dri`) | Medium — sandboxed DRI passthrough; known NVIDIA proprietary-driver friction | Yes — Flathub PR + bot build + volunteer review |
| **Snap** | Medium (3-4 hrs first publish, then fast) | Ubuntu-first, growing elsewhere | Strict, via `gnome` extension (`gpu-2404` plug) | Medium — driver detection under confinement across Intel/AMD/NVIDIA needs real testing | Yes — Snap Store name registration now requires manual review (~2 business days) |

**Recommendation for rustywx:** ship `.deb` first (least work, biggest immediate
audience, direct Vulkan access with no sandbox variable to debug), AppImage second
(same low GPU risk, zero-install distro-agnostic reach), `.rpm` third (same shape as
.deb, smaller audience). Flatpak and Snap are both worth doing for discoverability
(Flathub/GNOME Software, Snap Store) but carry the same category of risk — GPU
under sandbox — and should follow only once the non-sandboxed builds are stable and
the CI matrix exists, since both require real hardware testing (Intel/AMD/NVIDIA)
that the plain builds don't.

## 2. Cross-cutting concerns

**Workspace layout.** The repo is a virtual Cargo workspace (root `Cargo.toml` has
`members = ["app"]` and no `[package]`); the crate/binary `rustywx` lives in `app/`.
Two consequences for every packager: all `[package.metadata.*]` blocks (`deb`,
`generate-rpm`, `appimage`) must live in `app/Cargo.toml`, not the workspace root,
so the tools that read them must run against that manifest (`cargo deb -p rustywx`
or `--manifest-path app/Cargo.toml`). The compiled binary still lands in the shared
workspace target directory at the repo root, i.e. `target/release/rustywx` (not
`app/target/...`), which is the path Flatpak and AppImage install from.

**glibc portability.** Only matters for .deb/.rpm/AppImage (Flatpak and Snap bundle
their own runtime/base). glibc is forward-compatible only, so build on the oldest
base you intend to support — Ubuntu 22.04 (glibc 2.35) is a pragmatic floor. Use
`rustup`'s toolchain rather than the distro's `rustc` so the OS stays old but the
compiler stays current.

**Shared assets, built once, reused everywhere:**
- `.desktop` file — valid freedesktop `Categories=` (e.g. `Science` or `Utility`;
  Flathub explicitly rejects made-up categories like `GUI`/`Application`/`GTK`).
- AppStream metainfo XML (`io.github.kerryhatcher.rustywx.metainfo.xml`) — required
  by Flathub (must pass `appstreamcli validate` with zero warnings — warnings are
  fatal there) and reusable as-is for the Snap's `snap/gui/*.appdata.xml` and as
  general metadata for .deb/.rpm/AppImage.
- Icons — already present at `app/assets/icon/` (16 through 1024 PNG + `rustywx.svg`);
  install into `hicolor` icon theme paths for every format.
- Common `common-id`/app-id: `io.github.kerryhatcher.rustywx` ties Flatpak, Snap
  AppStream, and the desktop file together.

**One CI matrix.** A single GitHub Actions workflow can produce all five artifacts:
`.deb`+AppImage from an `ubuntu-22.04` job, `.rpm` from a Fedora container job,
Flatpak from `flatpak-builder-tools`/`flatpak-builder` (needs the offline
vendoring step below), and Snap via `snapcore/action-build` +
`snapcore/action-publish`. release-please owns the `Cargo.toml` version; every
packager (`cargo-deb`, `cargo-generate-rpm`, `cargo-appimage`, Flatpak manifest,
`snapcraft.yaml`) reads that version at build time rather than duplicating it.

## 3. Per-format detail

### Flatpak / Flathub
- **Tooling:** `org.freedesktop.Platform`//25.08 runtime, `org.freedesktop.Sdk` +
  `rust-stable` SDK extension (`append-path /usr/lib/sdk/rust-stable/bin`).
- **Offline build (biggest maintenance item):** the Flatpak build sandbox has no
  network access, so all crates must be vendored ahead of time with
  `flatpak-cargo-generator.py Cargo.lock -o cargo-sources.json`, then build with
  `CARGO_NET_OFFLINE=true` / `cargo --offline build --release` from the workspace
  root (which builds the `app` member) and install the resulting
  `target/release/rustywx` into `/app/bin`. **`cargo-sources.json` must be
  regenerated every time `Cargo.lock` changes** — this is the most common Flatpak
  build failure and the main ongoing cost of this format. Generate it from the
  Cargo.lock that ships with the tagged release, not a moving branch.
- **finish-args (least privilege):** `--device=dri` (Vulkan/wgpu — NOT
  `--device=all`), `--socket=wayland`, `--socket=fallback-x11`, `--share=ipc`,
  `--share=network` (needed for NEXRAD/NHC data fetch), `--filesystem=xdg-cache/rustywx:create`,
  `--filesystem=xdg-config/rustywx:create`. No geoclue permission needed since
  geolocation is macOS-only.
- **Flathub submission requirements:** reverse-DNS app-id verified via GitHub
  login, valid `.desktop`, hicolor icons, AppStream metainfo with id/name/summary/
  description/launchable/url/content_rating/releases, AGPL-3.0-only accepted with
  correct SPDX in `project_license`.
- **Distribution flow:** fork `flathub/flathub`, PR against `new-pr`, comment
  "bot, build" to trigger the build bot, volunteer review, merge creates a
  per-app repo, maintainer enables 2FA. Auto-update PRs come from
  `flatpak-external-data-checker` via `x-checker-data` (github-releases checker).
- **Gotcha:** NVIDIA proprietary driver + `--device=dri` renderD128 passthrough has
  open upstream issues (flatpak/flatpak#6672) that are not fixable from the app
  side — see section 4.
- **Effort:** ~1-3 focused days plus unpredictable volunteer-review wait time.

### AppImage
- **Tooling:** `cargo-appimage` (fast path, config via `[package.metadata.appimage]`
  in `app/Cargo.toml`) or manual `linuxdeploy` + `appimagetool` fed the binary at
  `target/release/rustywx` (more control over what gets bundled — the recommended
  route here because of the GPU rule below).
- **Critical rule:** never bundle `libGL.so`, `libvulkan.so`, or Mesa inside the
  AppDir — these must resolve against the host's installed driver, or the app will
  render with software/mismatched GPU state on the user's machine.
- **AppDir layout:** `usr/bin`, `usr/share/applications/*.desktop`,
  `usr/share/icons/hicolor/*/apps`, `usr/share/metainfo`.
- **Build base:** Ubuntu 22.04 (glibc 2.35 floor), rustup toolchain for a current
  compiler.
- **Updates:** `appimagetool -u "gh-releases-zsync|kerryhatcher|rustywx|latest|rustywx-*x86_64.AppImage.zsync"`
  produces a `.zsync` file so AppImageUpdate can delta-update from GitHub Releases.
- **Distribution:** ship `.AppImage` + `.zsync` as GitHub Release assets. No store.

### .deb
- **Tooling:** `cargo-deb`, configured via `[package.metadata.deb]` in
  `app/Cargo.toml`; build with `cargo deb -p rustywx`.
- **Fields:** maintainer, copyright, license-file, `section = "science"`.
- **Depends:** cargo-deb's `depends` field takes `$auto` (which runs
  `dpkg-shlibdeps` on the binary to infer transitively-linked libraries) plus a
  comma-separated manual list for the libraries loaded dynamically at runtime,
  which `$auto` cannot see (the Vulkan loader and GPU stack):

  ```toml
  [package.metadata.deb]
  depends = "$auto, libvulkan1, mesa-vulkan-drivers, libgl1, libx11-6, libxkbcommon0, libwayland-client0"
  ```

  These are separate packages, not alternatives: `libvulkan1` is the Vulkan loader,
  `mesa-vulkan-drivers` provides the Mesa Vulkan ICDs, and `libgl1` is the GL loader.
- **Assets:** binary (`target/release/rustywx`) + icons (hicolor sizes) + `.desktop`;
  no maintainer scripts needed since `.desktop`/icon-cache updates are handled by
  standard triggers.
- **Build:** `ubuntu-22.04` GitHub Actions runner.
- **Distribution:** standalone `.deb` on GitHub Releases. A real APT repo (for
  `apt upgrade` support) is extra hosting infrastructure — skip until there's
  demand.

### .rpm
- **Tooling:** `cargo-generate-rpm` — no hand-written spec file or `rpmbuild`
  needed. It reads `[package.metadata.generate-rpm]` from `app/Cargo.toml` and
  packages a binary you have already built (`cargo build --release`), so run
  `cargo generate-rpm --package rustywx` after the release build (confirm the
  workspace-member flag against your installed cargo-generate-rpm version).
- **Config:** `[package.metadata.generate-rpm]` for `assets`, plus a
  `[package.metadata.generate-rpm.requires]` table listing the runtime
  dependencies under Fedora/RHEL naming (which differs from the .deb equivalents by
  distro convention):

  ```toml
  [package.metadata.generate-rpm.requires]
  vulkan-loader = "*"
  mesa-libGL = "*"
  libX11 = "*"
  libxkbcommon = "*"
  libwayland-client = "*"
  ```
- **Build:** Fedora container (`fedora:40`) in CI.
- **Distribution:** standalone `.rpm` on GitHub Releases; COPR repo later if there's
  demand.

### Snap
- **Tooling:** `snapcraft.yaml`, `plugin: rust`, `base: core24`,
  `confinement: strict`, `license: AGPL-3.0-only`.
- **Extension:** `extensions: [gnome]` — the extension key is literally `gnome`;
  on a `core24` base it wires in the `gnome-46-2404` platform snap and the
  `gpu-2404` plug (backed by the `mesa-2404` content snap), and auto-provides the
  `desktop`/`opengl`/`wayland`/`x11` plugs plus the GTK/graphics runtime libraries
  so you don't hand-wire them. (`gnome-46-2404` and `gpu-2404`/`mesa-2404` are the
  content snaps it pulls in, not the value you put under `extensions:`.)
- **Plugs:** `network`, `home` (read: all).
- **build-packages:** `libglib2.0-dev, libwayland-dev, libx11-dev,
  libxkbcommon-dev, libssl-dev, pkg-config`.
- **Desktop integration:** `snap/gui/rustywx.desktop` (`Icon=${SNAP}/...`), icon,
  `appdata.xml` (reuse the AppStream metainfo from section 2), `common-id:
  io.github.kerryhatcher.rustywx`.
- **Distribution flow:** Ubuntu One account, `snapcraft register rustywx` (name is
  globally unique and permanent — **the Snap Store now requires manual review of
  new name registrations, roughly 2 business days**; confirm current wait time
  before relying on a launch date), build locally or via Launchpad
  `remote-build` for multi-arch, publish to `edge`/`beta`/`candidate`/`stable`
  channels. CI via `snapcore/action-build` + `snapcore/action-publish` with a
  `SNAPCRAFT_TOKEN` secret.
- **Effort:** ~3-4 hours for first publish once the manual review clears, 5-10
  minutes per subsequent release.

## 4. GPU under sandbox: the wgpu/Vulkan reality

Both sandboxed formats (Flatpak, Snap) work — this is not a blocker for either —
but each solves host-GPU access differently, and each carries its own real risk.
Neither report's claim about the other's GPU story is wrong; they're describing
different mechanisms:

- **Flatpak:** GPU access is opt-in via `--device=dri` in `finish-args`, which
  passes through the host's DRI render node (`/dev/dri/renderD128` etc.) for
  Vulkan/wgpu. This is deliberately minimal (avoid `--device=all`). The known
  failure mode is **NVIDIA's proprietary driver**, which has open, long-standing
  passthrough issues under this model (tracked upstream at
  flatpak/flatpak#6672) that are not fixable from rustywx's side — Mesa-based
  drivers (Intel, AMD, and NVIDIA's open Nouveau/nvidia-open path) are the
  reliable case.
- **Snap:** GPU access comes from the `gnome` extension's `gpu-2404` plug plus the
  `mesa-2404` content provider snap, which injects Mesa library paths into the
  strictly-confined runtime at launch. This lets Vulkan/wgpu work under
  `confinement: strict` without falling back to `confinement: classic`. The
  remaining risk here is **driver detection consistency across Intel/AMD/NVIDIA
  hosts under confinement** — this needs to be tested on real hardware for each
  vendor before calling it stable, not assumed from the extension alone.

**Bottom line:** Flatpak's specific known gap is NVIDIA proprietary; Snap's is
general cross-vendor confinement testing. Both are solvable, neither has a
one-line fix, and both should be validated on real GPUs (particularly NVIDIA)
before either format is promoted out of a beta channel.

## 5. Effort estimate and phased rollout

1. **Phase 1 (fastest wins, no sandbox risk):** `.deb` via `cargo-deb`, then
   AppImage via manual `linuxdeploy`+`appimagetool` (excluding GPU libs), built in
   one `ubuntu-22.04` CI job. ~1-1.5 days combined.
2. **Phase 2:** `.rpm` via `cargo-generate-rpm` in a Fedora container job, reusing
   the same shared `.desktop`/icon/metainfo assets from Phase 1. ~0.5 day.
3. **Phase 3 (sandboxed, discoverability-driven):** Flatpak manifest + offline
   vendoring pipeline (`flatpak-cargo-generator.py` regenerated on every
   `Cargo.lock` bump), submit to Flathub. ~1-3 days plus unpredictable volunteer
   review wait. Test on Intel/AMD/NVIDIA before submitting.
4. **Phase 4:** Snap via the `gnome` extension (which supplies the `gnome-46-2404`
   platform and `gpu-2404`/`mesa-2404` GPU stack on core24), register the name early
   (manual review lead time), publish to `edge` first and soak-test GPU behavior
   across vendors before promoting to `stable`. ~3-4 hours once the name is
   approved.
5. Fold all four phases into one GitHub Actions matrix workflow once Phase 2 is
   stable, gated on the same release-please version bump.

Total: roughly 1 work-week spread across the four phases, dominated by review
wait times (Flathub, Snap Store name approval) rather than engineering effort.

## 6. Sources

- github.com/flatpak/flatpak-builder-tools (cargo generator)
- belmoussaoui.com/blog/8-how-to-flatpak-a-rust-application
- develop.kde.org/docs/getting-started/rust/rust-flatpak
- docs.flathub.org (submission, metainfo, verification, external-data-checker docs)
- github.com/flatpak/flatpak/issues/6672 (NVIDIA DRI passthrough)
- github.com/kornelski/cargo-deb
- github.com/cat-in-136/cargo-generate-rpm
- github.com/linuxdeploy/linuxdeploy
- docs.appimage.org/reference/best-practices.html
- github.com/AppImageCommunity/AppImageUpdate
- v2.tauri.app/distribute/appimage (glibc/build-base guidance)
- ubuntu.com/docs/snapcraft/9.0 (craft-a-rust-app, gpu-extension, gnome-extension)
- snapcraft.io/docs (channels, confinement)
- github.com/canonical/action-build / action-publish
- linuxiac.com (Snap Store manual-review-on-registration reporting)

**Flagged as time-sensitive, verify before relying on them for a launch date:**
the Snap Store's "manual review, ~2 business days" figure and the Flathub bot
build/review workflow are both process details that Canonical/Flathub can and do
change; re-check current process docs immediately before submission rather than
trusting the timeline here.
