#!/usr/bin/env bash
# D22.8's plain-iced reproduction (`rustd3-bench repro-software`, see
# dev/bench/src/repro.rs) four times, about 15 s each: tiny-skia and wgpu,
# each on Wayland and on X11 through XWayland (WAYLAND_DISPLAY removed).
# Then a table of the frames drawn in 10 s, the frame interval's p50, p90
# and p99 and the CPU seconds per second; the outputs go to
# bench-results/repro-<UTC stamp>/. A row whose display the session lacks
# is skipped. Linux only: it compares Wayland with X11.
#
#   dev/bench/repro-software.sh
set -euo pipefail
if [ "$(uname -s)" != Linux ]; then
  echo "repro-software.sh: Linux only (it compares Wayland with X11); this is $(uname -s)" >&2
  exit 1
fi
cd "$(dirname "$(readlink -f "$0")")/../.."

command -v cargo >/dev/null || { echo "cargo not found: install Rust from https://rustup.rs" >&2; exit 1; }
cargo build --release -p rustd3-bench
bench="${CARGO_TARGET_DIR:-target}/release/rustd3-bench"
dir="bench-results/repro-$(date -u +%Y%m%dT%H%M%SZ)"
mkdir -p "$dir"

# The value after "NAME: " in FILE, up to the first space.
field() { sed -n "s|^$1: \([^ ]*\).*|\1|p" "$2"; }
row() { printf '%-10s %-8s %7s %8s %8s %8s %8s\n' "$@"; }

table="$(row backend session frames "p50 ms" "p90 ms" "p99 ms" "CPU s/s")"$'\n'
for backend in tiny-skia wgpu; do
  for session in wayland x11; do
    if [ "$session" = wayland ]; then
      need=WAYLAND_DISPLAY
      run=(env)
    else
      need=DISPLAY
      run=(env -u WAYLAND_DISPLAY)
    fi
    if [ -z "${!need:-}" ]; then
      table+="$(row "$backend" "$session" "skipped: $need is unset")"$'\n'
      continue
    fi
    out="$dir/$backend-$session.txt"
    echo "== $backend on $session"
    if "${run[@]}" ICED_BACKEND="$backend" "$bench" repro-software --out "$out" && [ -s "$out" ]; then
      table+="$(row "$backend" "$session" "$(field frames "$out")" \
        "$(field 'frame interval p50' "$out")" "$(field 'frame interval p90' "$out")" \
        "$(field 'frame interval p99' "$out")" "$(field 'CPU s/s' "$out")")"$'\n'
    else
      table+="$(row "$backend" "$session" "failed (see above)")"$'\n'
    fi
  done
done
echo
printf '%s' "$table"
echo "Outputs: $dir"
