#!/usr/bin/env bash
# Start rustywx against a real Wayland compositor (Weston's headless backend).
# This is intentionally strict: no X11 fallback is allowed in this test.
set -euo pipefail

binary="${1:-target/release/rustywx}"
[[ -x "$binary" ]] || { echo "Release executable not found: $binary" >&2; exit 1; }
command -v weston >/dev/null || { echo "weston is required" >&2; exit 1; }

log_dir="${RUSTYWX_WAYLAND_LOG_DIR:-$(mktemp -d)}"
runtime_dir="$(mktemp -d)"
chmod 700 "$runtime_dir"
export XDG_RUNTIME_DIR="$runtime_dir"
export WAYLAND_DISPLAY="rustywx-wayland-0"
export RUSTYWX_LINUX_BACKEND=wayland

weston_log="$log_dir/weston.log"
app_log="$log_dir/rustywx-wayland.log"
cleanup() {
  [[ -n "${app_pid:-}" ]] && kill "$app_pid" 2>/dev/null || true
  [[ -n "${weston_pid:-}" ]] && kill "$weston_pid" 2>/dev/null || true
  wait "${app_pid:-}" 2>/dev/null || true
  wait "${weston_pid:-}" 2>/dev/null || true
  rm -rf "$runtime_dir"
}
trap cleanup EXIT

mkdir -p "$log_dir"
# miniquad initializes the Wayland clipboard and input paths, which require a
# wl_seat. Weston's headless backend provides one only when explicitly asked.
weston --backend=headless-backend.so --fake-seat --socket="$WAYLAND_DISPLAY" --log="$weston_log" &
weston_pid=$!
for _ in $(seq 1 50); do
  [[ -S "$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY" ]] && break
  kill -0 "$weston_pid" 2>/dev/null || {
    cat "$weston_log" >&2
    exit 1
  }
  sleep 0.1
done
[[ -S "$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY" ]] || {
  echo "Weston did not create its Wayland socket" >&2
  cat "$weston_log" >&2
  exit 1
}

"$binary" >"$app_log" 2>&1 &
app_pid=$!
sleep 5
if ! kill -0 "$app_pid" 2>/dev/null; then
  echo "rustywx exited before the Wayland smoke-test deadline" >&2
  cat "$app_log" >&2
  exit 1
fi
grep -F 'Linux window backend: WaylandOnly' "$app_log"
echo "Native Wayland smoke test passed; logs: $log_dir"
