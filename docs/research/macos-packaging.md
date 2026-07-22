# macOS Packaging for rustywx

Consolidated from two independent research passes (Mac App Store track, Homebrew track), fact-checked
against the repo and current Apple/Homebrew documentation as of 2026-07-22.

**Subject:** `rustywx` v0.7.1, Rust/edition 2024, GPU GUI via ply-engine/wgpu (Metal backend on macOS),
CoreLocation via `objc2-core-location`, AGPL-3.0-only, github.com/kerryhatcher/rustywx. Icons already
exist at `app/assets/icon/` (16–1024px PNG + source SVG). No `Info.plist`, entitlements file, or signing
config exists in the repo yet — this is a greenfield packaging effort.

## 1. Executive summary and recommendation

**Ship it two ways: a Developer ID-signed, notarized, universal `.dmg` attached to GitHub Releases, and
a Homebrew Cask on a custom tap (`kerryhatcher/homebrew-rustywx`) that installs that same `.dmg`. Skip
the Mac App Store unless the project relicenses.**

- The AGPL-3.0-only license is **not compatible with Mac App Store distribution as-is** — this is a real,
  well-documented conflict, not a formality. Both source reports agree on this, and it holds up under
  verification (see §2).
- Homebrew Cask is the low-friction, license-clean path: it doesn't touch Apple's store terms at all,
  just Gatekeeper. The one blocking prerequisite for *both* paths is the same: an Apple Developer Program
  membership ($99/yr, which is what lets you issue the Developer ID Application certificate) and a working
  `codesign` → `notarytool` → `stapler` pipeline. Build that once, reuse it everywhere.
- rustywx will not qualify for the **upstream** `homebrew-cask` repo on notability grounds unless/until it
  gains a real user base — use a custom tap instead, which has no such gate and is trivial to automate.
- Phased rollout (detail in §6): (1) custom tap + unsigned dev builds → (2) signing/notarization in CI →
  (3) automated version-bump PRs to the tap on every `release-please` release.

## 2. The AGPL-3.0 licensing constraint

**Claim verified.** Apple's App Store Terms require the license granted to end users be a non-transferable
license restricted to Apple-branded hardware, effectively capped at whatever device-family limit Apple
sets. The GPL family (GPLv2/v3/AGPLv3) forbids adding downstream restrictions like this on top of the
license — you can't relicense-by-side-agreement what the GPL already grants unconditionally. This is the
same conflict that has kept VLC and other GPL/AGPL projects off Apple's official stores; it is a genuine
legal incompatibility, not App Store bureaucracy that can be argued around.

Both source reports agree on the conflict and on the option set. Consolidated options, ranked by fit for
a solo/small-maintainer AGPL project:

| Option | What it means | Verdict for rustywx |
|---|---|---|
| **C. Skip the App Store, distribute Developer ID direct** | No relicensing, no App Store review, full AGPL compliance | **Recommended** — lowest friction, zero legal risk |
| B. Add a GPL §7 "App Store exception" (additional permission) | An explicit clause added to the AGPL grant permitting distribution under the store's restrictive terms, provided the same source stays available unrestricted elsewhere (the pattern several AGPL iOS apps use) | Viable later if App Store reach becomes a business goal; legally clean, though the FSF discourages it on principle |
| A. Dual-license (proprietary for MAS, AGPL for GitHub) | Maintainer retains enough copyright to offer a second, non-copyleft license for the store build | Only works cleanly if there are zero external contributors under AGPL (true today); revisit if contributors are accepted |
| D. Relicense the whole project to MIT/Apache | Drops copyleft entirely | Not recommended — throws away the AGPL network-copyleft guarantee for no benefit specific to this problem |

No contradiction between the two reports here; Report A's framing (options A–D) and Report B's framing
(App Store is a non-starter under AGPL) are the same conclusion from two angles.

## 3. Distribution paths

### (a) Homebrew Cask via custom tap — recommended primary path

Use a **Cask**, not a Formula: rustywx is a GUI `.app`, and Casks install pre-built binaries to
`/Applications`, whereas Formulae are for CLI tools built from source. This is unambiguous in Homebrew's
own docs and both reports agree.

Upstream `homebrew-cask` has real notability gates verified against current docs: self-submitted casks
(PR author == repo owner, which is the case here) need **90 forks / 90 watchers / 225 stars** — 3x the
general-submission bar of 30/30/75, specifically to filter out low-traffic self-promotion. rustywx will
not clear this today. Use a personal tap instead:

```bash
brew tap-new kerryhatcher/rustywx
# Casks/rustywx.rb
```

```ruby
cask "rustywx" do
  arch arm: "aarch64", intel: "x86_64"

  version "0.7.1"
  sha256 arm:   "REPLACE_WITH_ARM64_DMG_SHA256",
          intel: "REPLACE_WITH_X86_64_DMG_SHA256"

  url "https://github.com/kerryhatcher/rustywx/releases/download/v#{version}/rustywx-#{version}-#{arch}.dmg"
  name "rustywx"
  desc "GPU-accelerated weather GUI"
  homepage "https://github.com/kerryhatcher/rustywx"

  app "rustywx.app"

  zap trash: [
    "~/Library/Application Support/rustywx",
    "~/Library/Preferences/com.github.kerryhatcher.rustywx.plist",
    "~/Library/Caches/com.github.kerryhatcher.rustywx",
  ]
end
```

Users: `brew tap kerryhatcher/rustywx && brew install --cask rustywx`.

**Notarization is a hard requirement.** Homebrew 5.0.0 (released November 2025) deprecated unsigned and
un-notarized casks in the **official** `homebrew-cask` tap; casks that fail a Gatekeeper check are
scheduled to be disabled there in **September 2026**. A **custom tap is not bound by that upstream
policy**, but that distinction does not help in practice: Gatekeeper on the end user's machine will block
or warn on an unsigned or un-notarized `.app` regardless of which tap installed it. Notarization is
therefore a practical requirement either way, not merely a Homebrew policy checkbox.

### (b) Developer ID direct-distribution DMG — recommended secondary/parallel path

Attach a signed, notarized, universal `.dmg` to GitHub Releases. This is the artifact the Cask above
points at, so building this pipeline serves both distribution paths simultaneously. Fully AGPL-compliant:
no store terms involved at all.

### (c) Mac App Store — only if relicensed

Do not pursue unless the project adopts option A or B from §2. If it ever does: separate signing identity
(3rd-Party Mac Developer, not Developer ID), App Store Connect listing, sandboxing is *mandatory* (not
optional as with Developer ID), and full App Review — realistically 3–7 weeks of added work on top of the
Developer ID pipeline, not instead of it.

## 4. Technical pipeline

### Universal binary

```bash
rustup target add aarch64-apple-darwin x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin
lipo -create \
  target/aarch64-apple-darwin/release/rustywx \
  target/x86_64-apple-darwin/release/rustywx \
  -output rustywx-universal
```

For `.app` bundling and DMG creation, `cargo-packager` (CrabNebula, actively maintained — 0.11.x as of
late 2025) is a reasonable off-the-shelf choice over hand-rolling bundle scripts: it assembles the bundle
from the universal binary and icon set and produces the `.dmg`. It is still in public preview, and it does
not run notarization for you; treat signing and notarization as a separate `codesign` → `notarytool` →
`stapler` step (below) even if you let packager handle bundling. Confirm the current config surface against
`docs.crabnebula.dev/packager/configuration` before wiring into CI, since the config schema still moves
between releases. Hand-rolling the bundle (a `.app` is just a directory tree plus `Info.plist`) is a fine
alternative for an app this simple and removes the preview-tool dependency.

### Signing identity

Two distinct certs for two distinct purposes — do not conflate them:

| Cert | Path | Notarization |
|---|---|---|
| Developer ID Application | Cask + direct DMG distribution | **Required** |
| 3rd Party Mac Developer (Mac App Store) | App Store only | Not applicable — App Review replaces it |

### Entitlements

The two `com.apple.security.*` sandbox keys below only take effect when the App Sandbox is enabled. The
recommended Developer ID path does **not** sandbox, so it needs a different (and much smaller) entitlements
file than the Mac App Store path. Do not copy the sandbox list onto the Developer ID build — it buys
nothing there and can only add friction.

**Developer ID path (recommended).** With the hardened runtime (`--options runtime`, required for
notarization) and no sandbox, this app needs **no entitlements file at all** for its current feature set:

- Location works from the `NSLocationWhenInUseUsageDescription` string in `Info.plist` alone; the
  `com.apple.security.personal-information.location` entitlement is a *sandbox* entitlement and is not
  consulted outside the sandbox.
- Network access is unrestricted outside the sandbox, so `com.apple.security.network.client` is not needed.
- Metal/wgpu GPU access needs no entitlement.

Add an entitlements file here only if a hardened-runtime exception turns out to be required (for example
`com.apple.security.cs.disable-library-validation`, needed only if the app loads third-party unsigned
dylibs — this app does not). Sign with `--options runtime` and skip `--entitlements` if the file is empty.

**Mac App Store path (only if relicensed).** The sandbox is mandatory here, so this variant is required
(`app/rustywx.entitlements`):

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.app-sandbox</key>
    <true/>
    <key>com.apple.security.network.client</key>
    <true/>
    <key>com.apple.security.personal-information.location</key>
    <true/>
    <key>com.apple.security.files.user-selected.read-write</key>
    <true/>
</dict>
</plist>
```

Notes on the sandbox list, reconciled against current Apple docs:

- The location key is `com.apple.security.personal-information.location`, per Apple's current
  [Location entitlement documentation](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.personal-information.location).
  The shorter `com.apple.security.location` seen in one source report is not a documented key.
- `com.apple.security.network.client` is required for any weather-data HTTP fetch under the sandbox.
- Metal/wgpu GPU access needs no separate entitlement even under the sandbox — verify once against a live
  sandboxed build, since GPU entitlement requirements have shifted across macOS versions before.
- `com.apple.security.files.user-selected.read-write` is only needed if the app presents open/save
  dialogs; drop it if it does not.

### `Info.plist` additions

```xml
<key>CFBundleIdentifier</key>
<string>com.github.kerryhatcher.rustywx</string>
<key>CFBundleShortVersionString</key>
<string>0.7.1</string>
<key>NSLocationWhenInUseUsageDescription</key>
<string>rustywx uses your location to show local weather conditions.</string>
<key>LSMinimumSystemVersion</key>
<string>11.0</string>
<key>LSApplicationCategoryType</key>
<string>public.app-category.weather</string>
<key>NSHighResolutionCapable</key>
<true/>
```

`NSLocationWhenInUseUsageDescription` is required — without it, `CLLocationManager` silently returns nil
instead of prompting, and this is one of the more common CoreLocation gotchas. Confirmed correct key name
against Apple's CoreLocation docs.

### Notarization flow (Developer ID path)

Do **not** use `codesign --deep`; it has been deprecated for signing since macOS 13 and applies every
signing option indiscriminately to nested content. Sign from the inside out instead: any nested binaries
first, then the outer `.app` last. For a plain Rust `.app` with no bundled frameworks or helpers, that is
just the app bundle itself.

```bash
# 1. Sign the .app with the hardened runtime (required for notarization).
#    Add --entitlements app/rustywx.entitlements only if a non-empty file exists (see above).
codesign --force --timestamp --options runtime \
  --sign "Developer ID Application: NAME (TEAMID)" rustywx.app

# 2. Build the .dmg around the signed .app, then sign the .dmg too.
#    (Use cargo-packager, create-dmg, or hdiutil to build it.)
codesign --force --timestamp \
  --sign "Developer ID Application: NAME (TEAMID)" rustywx-0.7.1-aarch64.dmg

# 3. Submit the .dmg for notarization and wait for the result.
xcrun notarytool submit rustywx-0.7.1-aarch64.dmg \
  --apple-id "$APPLE_ID" --password "$APP_SPECIFIC_PASSWORD" \
  --team-id "$TEAM_ID" --wait

# 4. Staple the ticket to the .dmg (and to the .app before packaging, for offline first launch).
xcrun stapler staple rustywx-0.7.1-aarch64.dmg

# 5. Sanity check — should report "accepted" / "source=Notarized Developer ID".
spctl -a -t open --context context:primary-signature -vv rustywx-0.7.1-aarch64.dmg
```

Notarization operates on the `.dmg` here, which covers the `.app` inside it. To also make the extracted
`.app` pass Gatekeeper offline, staple the `.app` (`xcrun stapler staple rustywx.app`) **before** building
the `.dmg` in step 2. Submitting a zipped app (`ditto -c -k --keepParent rustywx.app rustywx.zip`) instead
of the `.dmg` is an equally valid input to `notarytool` if you prefer to notarize the app directly.

## 5. CI automation

- GitHub Actions macOS runner builds both targets, `lipo`s, bundles, signs, notarizes, staples, and
  attaches the `.dmg`(s) to the GitHub Release created by `release-please`.
- On the `release-please` `release: published` event, fire a tap-bump action that opens a PR to
  `kerryhatcher/homebrew-rustywx` updating `version` and both `sha256`s:
  [`mislav/bump-homebrew-formula-action`](https://github.com/mislav/bump-homebrew-formula-action) (works
  for Casks too, despite the "formula" name) or
  [`dawidd6/action-homebrew-bump-formula`](https://github.com/dawidd6/action-homebrew-bump-formula) as an
  alternative. Both integrate cleanly after `release-please` since they trigger off the release event,
  not off a push to main.
- Store Apple ID, app-specific password/API key, Team ID, and the Developer ID cert (as a base64 secret,
  imported into a CI keychain) in GitHub Actions secrets.

## 6. Effort, cost, and phased rollout

| Item | Cost/effort |
|---|---|
| Apple Developer Program | $99/yr — the one hard blocking cost for *every* path except AGPL-only source distribution |
| Notarization service | Free (included with Developer Program membership) |
| Signing + notarization CI pipeline | ~1–2 days first time; near-zero maintenance after |
| Custom Homebrew tap + Cask | ~half a day |
| Tap version-bump automation | ~half a day |
| Developer ID direct distribution, end to end | 2–5 weeks elapsed (mostly cert provisioning + CI debugging, not raw effort) |
| Mac App Store (only if relicensed) | 3–7 weeks elapsed — dual-license/exception decision + sandboxing rework + App Review cycles, on top of the Developer ID pipeline |

**Recommended phasing:**

1. **Now:** stand up `kerryhatcher/homebrew-rustywx`, publish an unsigned/dev Cask pointing at existing
   GitHub Release assets. Gets the tap and install UX working end to end.
2. **Next:** get the Apple Developer cert, wire codesign → notarytool → stapler into the release CI job,
   ship signed/notarized universal DMGs.
3. **Then:** add the tap-bump GitHub Action so every `release-please` release auto-PRs the Cask update —
   no more manual sha256 edits.
4. **Only if it becomes a real goal:** revisit Mac App Store via a GPL §7 exception or dual-license,
   understanding it's materially more calendar time than the two steps above combined.

**Gotchas, verified relevant:**

- Test the CoreLocation prompt on both Apple Silicon and Intel builds. `NSLocationWhenInUseUsageDescription`
  in `Info.plist` is the easy one to forget — without it, `CLLocationManager` fails silently instead of
  prompting.
- Do not carry the sandbox entitlements onto the Developer ID build (see §4). They do nothing outside the
  sandbox and can mask real permission bugs.
- Do not use `codesign --deep` — deprecated since macOS 13. Sign inside out.
- Expect occasional `notarytool` scanning delays; set generous CI step timeouts (Apple's scan is usually
  minutes but is not guaranteed).

## 7. Sources

- [FSF — More about the App Store GPL Enforcement](https://www.fsf.org/blogs/licensing/more-about-the-app-store-gpl-enforcement)
- [appfair.org — GPL and the App Stores](https://appfair.org/blog/gpl-and-the-app-stores)
- [Network World — Solving the Apple App Store Incompatibility with the GPL](https://www.networkworld.com/article/2228190/solving-the-apple-app-store-incompatibility-with-the-gpl.html)
- [Apple Developer — App Sandbox](https://developer.apple.com/documentation/security/app-sandbox)
- [Apple Developer — `NSLocationWhenInUseUsageDescription`](https://developer.apple.com/documentation/bundleresources/information-property-list/nslocationwheninuseusagedescription)
- [Apple Developer — Location entitlement (`com.apple.security.personal-information.location`)](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.personal-information.location)
- [Apple — `codesign(1)` man page (`--deep` deprecated for signing since macOS 13)](https://keith.github.io/xcode-man-pages/codesign.1.html)
- [Homebrew — 5.0.0 release announcement (cask notarization)](https://brew.sh/2025/11/12/homebrew-5.0.0/)
- [CrabNebula — cargo-packager configuration](https://docs.crabnebula.dev/packager/configuration/)
- [Homebrew — Acceptable Casks](https://docs.brew.sh/Acceptable-Casks)
- [Homebrew — How to Create and Maintain a Tap](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap)
- [Workbrew — What Homebrew 5.0.0 means for your Mac fleet](https://workbrew.com/blog/homebrew-5-0-0)
- [GitHub — Homebrew notarization discussion](https://github.com/orgs/Homebrew/discussions/4582)
- [mislav/bump-homebrew-formula-action](https://github.com/mislav/bump-homebrew-formula-action)
- [dawidd6/action-homebrew-bump-formula](https://github.com/dawidd6/action-homebrew-bump-formula)
