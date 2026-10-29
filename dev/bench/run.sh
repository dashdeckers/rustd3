#!/usr/bin/env bash
# The benchmark's one entry point (D22.9), from a checkout on Linux or
# macOS: `cargo run --release -p rustd3-bench -- "$@"`, the same as
# `cargo rustd3-bench "$@"`. It builds the release bench, then runs a
# level: --quick (about 10 minutes, what a performance issue needs),
# --default (about 75 minutes, the default) or --full (hours), into
# bench-results/<UTC stamp>-<level>/, packed into
# bench-results/rustd3-bench-<level>-<UTC stamp>.tar.gz (dev/bench/README.md).
#
#   dev/bench/run.sh [--quick | --default | --full] [--out DIR]
#                    [--refresh-hz HZ] [--native-factor F]
#
# HZ and FACTOR, when set, pass --refresh-hz and --native-factor, e.g.
# HZ=144 FACTOR=1.25 dev/bench/run.sh --quick. The display measurement
# gives both otherwise (the factor is the one the window reports, GNOME's
# included), so the script no longer reads GNOME's settings.
set -euo pipefail
cd "$(dirname "$(readlink -f "$0")")/../.."

for arg in "$@"; do
  case "$arg" in
    --smoke-only | --focus)
      echo "run.sh: $arg is gone; run a level: --quick, --default or --full (dev/bench/README.md)" >&2
      exit 2 ;;
  esac
done
args=("$@")
if [ -n "${HZ:-}" ]; then args+=(--refresh-hz "$HZ"); fi
if [ -n "${FACTOR:-}" ]; then args+=(--native-factor "$FACTOR"); fi

command -v cargo >/dev/null || { echo "cargo not found: install Rust from https://rustup.rs" >&2; exit 1; }
exec cargo run --release -p rustd3-bench -- "${args[@]}"
