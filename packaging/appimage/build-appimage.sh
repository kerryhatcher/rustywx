#!/usr/bin/env bash
# Assemble a release AppDir and package it as a signed, updateable AppImage.
# Tool paths can be overridden for local builds; CI supplies verified binaries.
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: packaging/appimage/build-appimage.sh --binary PATH --version VERSION --arch ARCH [options]

Options:
  --output DIR       Artifact directory (default: dist/appimage)
  --appdir DIR       AppDir to create (default: build/appimage/AppDir)
  --unsigned         Do not sign (intended for local validation only)
EOF
}

binary=''
version=''
arch=''
output_dir='dist/appimage'
appdir='build/appimage/AppDir'
sign=true
while (($#)); do
  case "$1" in
    --binary) binary="$2"; shift 2 ;;
    --version) version="$2"; shift 2 ;;
    --arch) arch="$2"; shift 2 ;;
    --output) output_dir="$2"; shift 2 ;;
    --appdir) appdir="$2"; shift 2 ;;
    --unsigned) sign=false; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown option: $1" >&2; usage >&2; exit 2 ;;
  esac
done

[[ -x "$binary" ]] || { echo "Release executable not found: $binary" >&2; exit 1; }
[[ -n "$version" ]] || { echo "--version is required" >&2; exit 2; }
[[ "$arch" == x86_64 || "$arch" == aarch64 ]] || {
  echo "--arch must be x86_64 or aarch64" >&2; exit 2;
}
[[ "$appdir" != / && "$appdir" != . ]] || {
  echo "--appdir must not be / or ." >&2; exit 2;
}

linuxdeploy="${LINUXDEPLOY:-linuxdeploy}"
appimagetool="${APPIMAGETOOL:-appimagetool}"
command -v "$linuxdeploy" >/dev/null || { echo "linuxdeploy is required" >&2; exit 1; }
command -v "$appimagetool" >/dev/null || { echo "appimagetool is required" >&2; exit 1; }
command -v zsyncmake >/dev/null || { echo "zsyncmake is required to generate update metadata" >&2; exit 1; }

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
desktop_file="$root_dir/packaging/appimage/rustywx.desktop"
metainfo_file="$root_dir/packaging/appimage/com.github.kerryhatcher.rustywx.metainfo.xml"
icon_file="$root_dir/app/assets/icon/rustywx.svg"

rm -rf "$appdir"
mkdir -p \
  "$appdir/usr/bin" \
  "$appdir/usr/share/applications" \
  "$appdir/usr/share/icons/hicolor/scalable/apps" \
  "$appdir/usr/share/metainfo"
install -m 0755 "$binary" "$appdir/usr/bin/rustywx"
install -m 0644 "$desktop_file" "$appdir/rustywx.desktop"
install -m 0644 "$desktop_file" "$appdir/usr/share/applications/rustywx.desktop"
install -m 0644 "$icon_file" "$appdir/rustywx.svg"
install -m 0644 "$icon_file" "$appdir/usr/share/icons/hicolor/scalable/apps/rustywx.svg"
install -m 0644 "$metainfo_file" "$appdir/usr/share/metainfo/com.github.kerryhatcher.rustywx.metainfo.xml"

# Package ordinary runtime libraries. GPU and display drivers are selected by
# the user's system and must remain outside the AppImage.
APPIMAGE_EXTRACT_AND_RUN=1 "$linuxdeploy" \
  --appdir "$appdir" \
  --executable "$appdir/usr/bin/rustywx" \
  --desktop-file "$desktop_file" \
  --icon-file "$icon_file" \
  --deploy-deps-only "$appdir/usr/bin/rustywx" \
  --exclude-library='libGL.so.*' \
  --exclude-library='libEGL.so.*' \
  --exclude-library='libGLX.so.*' \
  --exclude-library='libvulkan.so.*' \
  --exclude-library='libdrm.so.*'

mkdir -p "$output_dir"
artifact="$output_dir/rustywx-${version}-${arch}.AppImage"
zsync_file="${artifact}.zsync"
# zsyncmake writes next to its current working directory, using only the
# AppImage basename, even when appimagetool receives a destination with a
# directory component. Remove both possible stale paths before packaging.
generated_zsync="$(basename "$artifact").zsync"
rm -f "$artifact" "$zsync_file" "$generated_zsync"
update_information="gh-releases-zsync|kerryhatcher|rustywx|latest|$(basename "$artifact")"
appimagetool_args=(--updateinformation "$update_information")
if "$sign"; then
  : "${APPIMAGE_GPG_KEY_ID:?APPIMAGE_GPG_KEY_ID is required for signed release builds}"
  : "${APPIMAGETOOL_SIGN_PASSPHRASE:?APPIMAGETOOL_SIGN_PASSPHRASE is required for signed release builds}"
  appimagetool_args+=(--sign --sign-key "$APPIMAGE_GPG_KEY_ID")
fi

ARCH="$arch" APPIMAGE_EXTRACT_AND_RUN=1 "$appimagetool" \
  "${appimagetool_args[@]}" "$appdir" "$artifact"

if [[ -s "$generated_zsync" && "$generated_zsync" != "$zsync_file" ]]; then
  mv "$generated_zsync" "$zsync_file"
fi
[[ -s "$zsync_file" ]] || { echo "appimagetool did not produce $zsync_file" >&2; exit 1; }
[[ -s "$artifact" ]] || { echo "appimagetool did not produce $artifact" >&2; exit 1; }
(cd "$output_dir" && sha256sum "$(basename "$artifact")" > "$(basename "$artifact").sha256")
printf 'Created %s, %s, and %s.sha256\n' "$artifact" "$zsync_file" "$artifact"
