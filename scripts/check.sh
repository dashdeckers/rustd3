#!/usr/bin/env bash
# quick: every commit's gate. full: end of every phase (run in the background).
set -euo pipefail
cd "$(dirname "$0")/.."
PACKAGES=(rustd3 rustd3-core rustd3-derive rustd3-figure rustd3-fonts rustd3-iced
          rustd3-testkit rustd3-harness rustd3-examples rustd3-bench)

touched() {  # packages with uncommitted changes or changed in HEAD
  { git diff --name-only HEAD; git ls-files --others --exclude-standard;
    git diff --name-only HEAD~1 HEAD 2>/dev/null || true; } | while read -r f; do
    case "$f" in
      core/*) echo rustd3-core ;;        derive/*) echo rustd3-derive ;;
      figure/*) echo rustd3-figure ;;    fonts/*) echo rustd3-fonts ;;
      iced/*) echo rustd3-iced ;;        dev/testkit/*) echo rustd3-testkit ;;
      dev/harness/*) echo rustd3-harness ;; dev/examples/*) echo rustd3-examples ;;
      dev/bench/*) echo rustd3-bench ;;  src/*) echo rustd3 ;;
      Cargo.toml|Cargo.lock|clippy.toml|dev/clippy.toml) printf '%s\n' "${PACKAGES[@]}" ;;
    esac
  done | sort -u
}

quick() {
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  local pkgs=("$@")
  if [ ${#pkgs[@]} -eq 0 ]; then mapfile -t pkgs < <(touched); fi
  for p in "${pkgs[@]}"; do TZ=Asia/Kathmandu cargo test -q -p "$p" --all-features; done
  if [ "${RUSTD3_FUZZ:-0}" = 1 ]; then TZ=Asia/Kathmandu cargo test -q -p rustd3-core --release --test main fuzz; fi
  TZ=Asia/Kathmandu cargo run -q -p rustd3-harness -- --check
}

full() {
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  for p in "${PACKAGES[@]}"; do
    TZ=Asia/Kathmandu cargo test -q -p "$p" --no-default-features
    TZ=Asia/Kathmandu cargo test -q -p "$p" --all-features
  done
  TZ=Asia/Kathmandu cargo test -q -p rustd3-figure   # default features: fonts
  TZ=Asia/Kathmandu cargo test -q -p rustd3
  node dev/harness/translate/translate.mjs --check
  RUSTDOCFLAGS="-D warnings" cargo doc -q --workspace --no-deps --all-features
}

case "${1:-quick}" in
  quick) shift || true; quick "$@" ;;
  full) full ;;
  *) echo "usage: scripts/check.sh quick [package…] | full" >&2; exit 2 ;;
esac
