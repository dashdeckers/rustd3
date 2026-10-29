#!/usr/bin/env bash
# quick: every commit's gate. full: end of every phase (run in the background).
# release: before publishing, on a clean tree (RELEASING.md).
set -euo pipefail
cd "$(dirname "$0")/.."
PACKAGES=(rustd3 rustd3-core rustd3-derive rustd3-figure rustd3-fonts rustd3-iced
          rustd3-testkit rustd3-harness rustd3-examples rustd3-bench)
# Every file a table generator under dev/harness/gen writes.
GENERATED=(core/src/scheme/stops.rs core/src/scheme/viridis.rs core/src/scheme/magma.rs
           core/src/scheme/inferno.rs core/src/scheme/plasma.rs core/src/scheme/categorical.rs
           core/src/format/locales.rs core/src/format/locales dev/fixtures/format_locales.json
           core/src/time/locales.rs core/src/time/locales dev/fixtures/time_locales.json)

touched() {  # packages with uncommitted changes or changed in HEAD
  { git diff --name-only HEAD; git ls-files --others --exclude-standard;
    git diff --name-only HEAD~1 HEAD 2>/dev/null || true; } | while read -r f; do
    case "$f" in
      core/*) echo rustd3-core ;;        derive/*) echo rustd3-derive ;;
      figure/*) echo rustd3-figure ;;    fonts/*) echo rustd3-fonts ;;
      iced/*) echo rustd3-iced ;;        dev/testkit/*) echo rustd3-testkit ;;
      dev/harness/*) echo rustd3-harness ;; dev/examples/*) echo rustd3-examples ;;
      dev/bench/*) echo rustd3-bench ;;  src/*|tests/*) echo rustd3 ;;
      README.md) echo rustd3; echo rustd3-examples ;;
      Cargo.toml|Cargo.lock|clippy.toml|dev/clippy.toml) printf '%s\n' "${PACKAGES[@]}" ;;
    esac
  done | sort -u
}

# The benchmark built without the widget, as `cargo rustd3-bench-figure`
# builds it (D22.10): no iced, winit or wgpu in its tree, clippy on it and
# the examples it uses, its tests.
no_widget() {
  if cargo tree -q -p rustd3-bench --no-default-features -e normal --prefix none \
      | grep -E '^(iced|winit|wgpu)'; then
    echo "rustd3-bench without the widget builds the crates above" >&2; return 1
  fi
  cargo clippy -p rustd3-bench -p rustd3-examples --all-targets --no-default-features -- -D warnings
  TZ=Asia/Kathmandu cargo test -q -p rustd3-bench --no-default-features
}

quick() {
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  local pkgs=("$@")
  if [ ${#pkgs[@]} -eq 0 ]; then mapfile -t pkgs < <(touched); fi
  for p in "${pkgs[@]}"; do TZ=Asia/Kathmandu cargo test -q -p "$p" --all-features; done
  if printf '%s\n' "${pkgs[@]}" | grep -qxE 'rustd3-(bench|examples)'; then no_widget; fi
  if [ "${RUSTD3_FUZZ:-0}" = 1 ]; then TZ=Asia/Kathmandu cargo test -q -p rustd3-core --release --test main fuzz; fi
  TZ=Asia/Kathmandu cargo run -q -p rustd3-harness -- --check
}

# The generated tables are what is committed and what their generators
# write now: a hand edit fails here, committed or not.
fresh() {
  git diff --exit-code HEAD -- "${GENERATED[@]}"
  node --import ./dev/harness/translate/d3-resolve.mjs dev/harness/gen/scheme_tables.mjs
  node --import ./dev/harness/translate/d3-resolve.mjs dev/harness/gen/format_locales.mjs
  TZ=America/Los_Angeles node --import ./dev/harness/translate/d3-resolve.mjs dev/harness/gen/time_locales.mjs
  git diff --exit-code HEAD -- "${GENERATED[@]}"
  local extra; extra=$(git ls-files --others --exclude-standard -- "${GENERATED[@]}")
  if [ -n "$extra" ]; then echo "generated but not committed: $extra" >&2; return 1; fi
}

full() {
  fresh
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  no_widget
  for p in "${PACKAGES[@]}"; do
    TZ=Asia/Kathmandu cargo test -q -p "$p" --no-default-features
    TZ=Asia/Kathmandu cargo test -q -p "$p" --all-features
  done
  TZ=Asia/Kathmandu cargo test -q -p rustd3-figure   # default features: fonts
  TZ=Asia/Kathmandu cargo test -q -p rustd3
  # iced's tests once per backend; with ICED_TEST_BACKEND set iced does not
  # fall back, and a missing Vulkan device fails unless RUSTD3_ALLOW_NO_GPU=1.
  for backend in tiny-skia wgpu; do
    if [ "$backend" = wgpu ] && [ "${RUSTD3_ALLOW_NO_GPU:-0}" = 1 ]; then
      echo "RUSTD3_ALLOW_NO_GPU=1: iced tests skip wgpu when it finds no device"
    fi
    for p in rustd3-iced rustd3-examples rustd3-bench; do
      ICED_TEST_BACKEND=$backend TZ=Asia/Kathmandu cargo test -q -p "$p" --all-features -- --nocapture
    done
  done
  node dev/harness/translate/translate.mjs --check
  RUSTDOCFLAGS="-D warnings" cargo doc -q --workspace --no-deps --all-features
  # The benchmark's headless smoke plan: a report that fails on errors
  # (a run that crashes or writes nothing), never on numbers.
  cargo build -q --release -p rustd3-bench
  target/release/rustd3-bench run --headless --plan smoke --backend tiny-skia \
    --out target/bench-smoke
}

# The five published crates, in the order cargo publishes them.
PUBLISHED=(rustd3-fonts rustd3-derive rustd3-core rustd3-figure rustd3)

# Before a release (RELEASING.md): the published crates' docs and the
# README's doctests, each package's file list, then on a clean tree the
# dry run, which packages, verifies and stops short of uploading, and no
# git source in any packaged Cargo.lock.
release() {
  RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features \
    -p rustd3 -p rustd3-core -p rustd3-figure -p rustd3-fonts -p rustd3-derive
  cargo test -p rustd3 --all-features --doc
  cargo test -p rustd3 --no-default-features --doc
  mkdir -p target
  for p in "${PUBLISHED[@]}"; do
    cargo package --list -p "$p" > "target/package-$p.txt"
    if grep -E '^(\.claude/|dev/|target/)' "target/package-$p.txt"; then
      echo "release: $p packages files it must not" >&2; return 1
    fi
  done
  if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "release: the dry run needs a clean tree" >&2; return 1
  fi
  cargo publish --dry-run --workspace --exclude rustd3-iced
  # cargo 1.97's dry run leaves the packages it built in tmp-crate/;
  # `cargo package` writes them to target/package/ itself.
  for p in "${PUBLISHED[@]}"; do
    local file="$p-0.1.0.crate" crate="" lock n
    for at in target/package/tmp-crate target/package; do
      if [ -f "$at/$file" ]; then crate="$at/$file"; break; fi
    done
    if [ -z "$crate" ] || ! lock=$(tar -xzOf "$crate" "$p-0.1.0/Cargo.lock"); then
      echo "release: no packaged Cargo.lock for $p" >&2; return 1
    fi
    n=$(grep -c 'git+' <<< "$lock" || true)
    if [ "$n" != 0 ]; then
      echo "release: $p's packaged Cargo.lock has $n git sources" >&2; return 1
    fi
    echo "release: $crate: Cargo.lock without git sources"
  done
  echo "release: ready; the five package lists are target/package-*.txt"
}

case "${1:-quick}" in
  quick) shift || true; quick "$@" ;;
  full) full ;;
  release) release ;;
  *) echo "usage: scripts/check.sh quick [package…] | full | release" >&2; exit 2 ;;
esac
