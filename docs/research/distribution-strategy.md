# Distribution Strategy for rustywx

Status: research / open decision. **No licensing or platform decision has been made yet** — this
document captures the maintainer's intent, the constraints discovered during packaging research, and
the options on the table so the decision can be made deliberately later.

This is the strategic layer above the three platform packaging docs in this directory
([`macos-packaging.md`](./macos-packaging.md), [`linux-packaging.md`](./linux-packaging.md),
[`windows-packaging.md`](./windows-packaging.md)). Those docs assume the current AGPL-3.0-only license
and a "skip the Mac App Store" recommendation. **If the license changes, those recommendations must be
revisited** (see [Consequences](#consequences-if-the-license-changes)).

---

## 1. Maintainer intent (the goals that drive everything)

Stated by the maintainer:

1. **Primary concern: ease of discovery and installation by end users.**
2. **The primary userbase is non-technical.**
3. **iOS and Android versions are eventually required** (not immediately, but a firm future goal).

These three goals, taken together, are the reason the license question below becomes the gating
decision for the whole project rather than a per-platform footnote.

---

## 2. How non-technical users actually discover and install software

Package managers (Homebrew, apt/deb, dnf/rpm, AppImage, even Flatpak to a degree) serve a **technical
minority**. A non-technical audience discovers and installs almost exclusively through **application
stores** and, secondarily, one-click installers downloaded from a website.

| Platform | Real channel for a non-technical user | Technical-only routes (secondary) |
|----------|---------------------------------------|-----------------------------------|
| **iOS** | **App Store — the only option** | none (sideloading is not realistic for this audience) |
| **Android** | **Google Play** (sideloading an APK will not happen for non-technical users) | direct APK |
| **macOS** | **Mac App Store** (best discovery) > notarized DMG (double-click, drag to Applications) | Homebrew Cask |
| **Windows** | Signed installer from the project website; **Microsoft Store** would materially help discovery | winget |
| **Linux** | Flathub (closest thing to an app store); niche for a non-technical audience | .deb / .rpm / AppImage |

Implication: the Homebrew / .deb / .rpm / AppImage work scoped in the platform docs is worth keeping
for the technical minority, but **it is not the primary channel** for the stated userbase. The primary
channels are stores.

---

## 3. The gating problem: AGPL-3.0 is incompatible with the required stores

`rustywx` is currently licensed **AGPL-3.0-only**. The GPL/AGPL family (via GPL §6, inherited by AGPL)
forbids imposing "further restrictions" on downstream users. Every major app store the strategy
depends on imposes exactly such restrictions (device caps tied to an account, DRM wrapping,
no-redistribution terms), creating an unresolvable conflict for the copyright holder.

| Store | Needed for the strategy? | AGPL status |
|-------|--------------------------|-------------|
| Mac App Store | Best macOS discovery for non-technical users | **Blocked** (App Store EULA vs GPL §6; VLC/GNU Go precedent) |
| iOS App Store | **Mandatory** — the only iOS channel | **Blocked** (same Apple EULA conflict) |
| Google Play | **Primary** Android channel | **Blocked** (Play Developer Distribution Agreement adds anti-redistribution terms that clash with GPL/AGPL) |
| Microsoft Store | Optional discovery boost (Windows) | Historically friendlier, but not required (Windows ships via installer) |
| Flathub | Linux (secondary audience) | **Allowed** — Flathub is copyleft-friendly |

**Conclusion:** the stated strategy (non-technical audience + eventual mobile) cannot be delivered
under AGPL-3.0. iOS is store-only and Android is store-primary, and both stores block AGPL. This is a
hard, structural conflict, not a tooling problem.

For the full mechanics of the App Store / GPL conflict, see the "AGPL-3.0 licensing constraint" section
of [`macos-packaging.md`](./macos-packaging.md).

---

## 4. Options on the table (no decision made)

### Option A — Relicense the whole project to a permissive license
Switch to **Apache-2.0**, or the Rust-ecosystem-standard **MIT OR Apache-2.0** dual.

- **Pros:** removes the store conflict on all five platforms at once; no dual build, no per-store
  proprietary variant, no contributor CLA required for relicensing; cheapest to do **now** while the
  project is solo-maintained at v0.7.1 with no external contributors whose consent would be needed.
- **Fit:** AGPL's purpose is to force source disclosure from network *services*. A desktop/mobile
  weather GUI is not that, so AGPL's main cost (store lockout) buys a protection the project does not
  use. Permissive is the natural fit for a "maximize reach" goal.
- **Cons:** gives up copyleft protection — anyone (including a competitor) could ship a closed
  derivative. May disappoint a copyleft-minded userbase.

### Option B — Dual-license (AGPL public + proprietary store builds)
Keep AGPL for the public source; ship proprietary-licensed builds to the stores.

- **Pros:** retains copyleft on the open version while still reaching stores.
- **Cons:** requires a **CLA/DCO with a relicensing grant** before accepting any external contribution
  (retrofitting contributor consent later is painful); two build variants to maintain; still requires
  the dependency audit; more legal and CI overhead. Only worth it if copyleft protection is genuinely
  wanted **and** store reach is needed.

### Option C — Keep AGPL-3.0-only
Stay copyleft; forgo the blocked stores.

- **Consequence:** **no iOS, no Google Play, no Mac App Store — ever.** Distribution limited to
  notarized DMG + Homebrew (macOS), signed installer (Windows), and Flathub / deb / rpm / AppImage
  (Linux). This caps non-technical **mobile** reach at zero, directly contradicting goal #3.

---

## 5. Dependency-license caveat (applies to Options A and B)

Relicensing *your own* code does not neutralize a copyleft *dependency*. `rustywx` statically links
`ply-engine`, `nexrad-data`, `nexrad-model`, `tokio`, `objc2*`, `image`, `zip`, and their transitive
deps.

- **MIT / Apache-2.0 / BSD / Zlib / MPL-2.0** — fine to include in a permissive or proprietary build.
- **GPL / AGPL dependency** — fatal; the same conflict re-appears regardless of your own license.
- **LGPL dependency** — a trap for a statically-linked Rust binary: LGPL permits proprietary use only
  if the user can relink against a modified library, which a static App Store binary under DRM cannot
  satisfy. Treat static-linked LGPL as near-fatal.

**Action required before any store plan is final:** run `cargo deny check licenses` (the repo already
has a `deny.toml`) or `cargo license` and audit the **full** dependency tree, not just direct deps.
**Check `ply-engine`'s license first** — it is the rendering core and the highest-risk single
dependency.

---

## 6. Signing matters more for a non-technical audience

For non-technical users, an unsigned artifact is not just a warning — it is an abandonment point.

- **Windows:** an unsigned installer triggers SmartScreen's "Windows protected your PC" screen.
  Technical users click through; non-technical users quit. Budget **Azure Artifact Signing**
  (~$9.99/mo, formerly Trusted Signing) sooner rather than "later." Details in
  [`windows-packaging.md`](./windows-packaging.md).
- **macOS:** Gatekeeper blocks un-notarized apps outright on default settings. A **Developer ID**
  certificate + notarization is non-optional for a non-technical audience even outside the App Store.
  Details in [`macos-packaging.md`](./macos-packaging.md).

The "ship unsigned first, sign later" advice in the platform docs was written for a technical
early-adopter audience. **For a non-technical userbase, signing moves up the priority list.**

---

## 7. Mobile (iOS / Android) — future, but the license must be settled now

- **Technical feasibility:** `wgpu` (via `ply-engine`) runs on iOS (Metal) and Android (Vulkan), so the
  rendering core is not a blocker in principle. However, a desktop GUI is **not** automatically a
  mobile app — touch input, app lifecycle, screen density, and platform packaging (an Xcode project for
  iOS, a Gradle/AAB build for Android) are substantial separate efforts. This is future work, out of
  scope for the current packaging pass.
- **Why it forces the license decision now:** iOS is store-only and Android is store-primary, and both
  stores block AGPL. Whatever license the mobile builds ship under must be settled before that work
  starts — and the cheapest time to relicense is now, while solo. Deferring the license decision is
  fine; deferring it *past* the arrival of external contributors is not.

---

## 8. Consequences if the license changes

If the project moves off AGPL (Option A or B), the existing platform docs need revisiting:

- **macOS:** the "skip the Mac App Store" recommendation **reverses** — the App Store becomes the best
  macOS discovery channel for non-technical users, and the sandbox/entitlements/App-Store-Connect
  track in `macos-packaging.md` moves from "not applicable" to "primary path."
- **Windows:** Microsoft Store becomes a viable discovery add-on to the installer (currently scoped
  out per the original "installer only" instruction — worth reconsidering for discovery).
- **Linux:** unchanged (Flathub already works under AGPL).
- **New work:** iOS and Android packaging docs would be added once the license and mobile effort are
  greenlit.

---

## 9. Open decisions / next steps

1. **Licensing direction** — Option A (relicense permissive), B (dual-license), or C (keep AGPL).
   *Deferred by maintainer; documented here for a deliberate later decision.*
2. **Dependency license audit** — run `cargo deny check licenses`; confirm `ply-engine` and the full
   tree are store-compatible. Can be done independently of the license decision and de-risks Options A/B.
3. **Signing budget** — decide when Developer ID (macOS) and Azure Artifact Signing (Windows) come
   online; earlier is better for a non-technical audience.
4. **Doc updates** — once the license direction is chosen, revise the three platform docs per
   [Consequences](#consequences-if-the-license-changes) and add mobile docs when that work is scoped.
