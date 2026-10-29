#!/usr/bin/env bash
# The benchmark in one command on Linux: builds the release bench, then
# runs `rustd3-bench suite` (dev/bench/README.md), which does the rest on
# every platform: the smoke plan (about 6 minutes, a check of the windowed
# path, whose windows give the refresh rate and scale factor), the default
# plan (about 47 minutes, the verdict's run), then D22.4's weak plan three
# times (about 7 minutes each): without a GPU (tiny-skia), and on wgpu
# limited to 4 and to 2 cores. In a Wayland session with XWayland the
# no-GPU profile runs again on X11 (weak-nogpu-x11, report-only, D22.8),
# 7 minutes more. About 76 minutes in all (83 on Wayland), into
# bench-results/<UTC stamp>/{smoke,default,weak-nogpu,weak-nogpu-x11,
# weak-4cores,weak-2cores}/. With --focus, D22.5's focus plan (about 17
# minutes, report-only) replaces smoke and default, after one run of its
# first cell that measures the display: about 40 minutes (47 on Wayland),
# into bench-results/<UTC stamp>/{probe,focus,weak-nogpu,weak-nogpu-x11,
# weak-4cores,weak-2cores}/.
#
#   dev/bench/run.sh               smoke, default, the weak profiles
#   dev/bench/run.sh --smoke-only  smoke only
#   dev/bench/run.sh --focus       the focus plan, the weak profiles
#
# HZ (the display's refresh rate) and FACTOR (its scale factor) override
# what the first windows report, e.g. HZ=144 FACTOR=1.25 dev/bench/run.sh.
set -euo pipefail
cd "$(dirname "$(readlink -f "$0")")/../.."

args=()
case "${1:-}" in
  "") ;;
  --smoke-only | --focus) args+=("$1") ;;
  *) echo "usage: dev/bench/run.sh [--smoke-only | --focus]" >&2; exit 2 ;;
esac
if [ -n "${HZ:-}" ]; then args+=(--refresh-hz "$HZ"); fi
# GNOME's integer scaling factor ("uint32 2"), when set, is what the first
# windows ask for; the factor they report is used after them.
if [ -z "${FACTOR:-}" ]; then
  scaling=$(gsettings get org.gnome.desktop.interface scaling-factor 2>/dev/null | awk '{print $NF}' || true)
  if [[ "$scaling" =~ ^[0-9]+$ ]] && [ "$scaling" -gt 0 ]; then FACTOR=$scaling; fi
fi
if [ -n "${FACTOR:-}" ]; then args+=(--native-factor "$FACTOR"); fi

command -v cargo >/dev/null || { echo "cargo not found: install Rust from https://rustup.rs" >&2; exit 1; }
cargo build --release -p rustd3-bench
exec "${CARGO_TARGET_DIR:-target}/release/rustd3-bench" suite "${args[@]}" --out bench-results
