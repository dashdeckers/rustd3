#!/usr/bin/env bash
# The benchmark in one command: build, the smoke plan (about 5 minutes, a
# check of the windowed path), then the default plan (about 46 minutes, the
# verdict's run), into bench-results/<UTC stamp>/{smoke,default}/.
#
#   dev/bench/run.sh               smoke, then default
#   dev/bench/run.sh --smoke-only  smoke only
#
# HZ (the display's refresh rate) and FACTOR (its scale factor) override
# detection, e.g. HZ=144 FACTOR=1.25 dev/bench/run.sh.
set -euo pipefail
cd "$(dirname "$(readlink -f "$0")")/../.."

smoke_only=false
case "${1:-}" in
  "") ;;
  --smoke-only) smoke_only=true ;;
  *) echo "usage: dev/bench/run.sh [--smoke-only]" >&2; exit 2 ;;
esac

plans="the smoke plan (about 5 min; its numbers do not count)"
$smoke_only || plans="$plans, then the default plan (about 46 min, the verdict's run)"
echo "rustd3-bench: build, then $plans, into bench-results/<UTC stamp>/."
echo "Keep hands off the mouse and keyboard while windows open and close."

command -v cargo >/dev/null || { echo "cargo not found: install Rust from https://rustup.rs" >&2; exit 1; }

# The refresh rate: the current (*) mode of xrandr's first output, else the
# (current) mode of cosmic-randr, whose output carries colour codes.
if [ -z "${HZ:-}" ] && command -v xrandr >/dev/null; then
  HZ=$(xrandr --current 2>/dev/null | grep -o '[0-9]\+\.[0-9]\+\*' | head -n1 | tr -d '*' || true)
fi
if [ -z "${HZ:-}" ] && command -v cosmic-randr >/dev/null; then
  HZ=$(cosmic-randr list 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' | grep '(current)' \
    | grep -o '[0-9.]\+ Hz' | head -n1 | cut -d' ' -f1 || true)
fi
if [ -z "${HZ:-}" ]; then
  echo "No refresh rate found (neither xrandr nor cosmic-randr reported one)." >&2
  echo "Rerun with your display's rate: HZ=144 dev/bench/run.sh" >&2
  exit 1
fi

# The scale factor: GNOME's integer scaling-factor ("uint32 2") when it is
# set; the smoke run reports the factor the windows really get.
if [ -z "${FACTOR:-}" ]; then
  FACTOR=1
  scaling=$(gsettings get org.gnome.desktop.interface scaling-factor 2>/dev/null | awk '{print $NF}' || true)
  if [[ "$scaling" =~ ^[0-9]+$ ]] && [ "$scaling" -gt 0 ]; then FACTOR=$scaling; fi
fi
echo "Refresh rate $HZ Hz, scale factor $FACTOR."

cargo build --release -p rustd3-bench
bench="${CARGO_TARGET_DIR:-target}/release/rustd3-bench"

if command -v system76-power >/dev/null; then
  previous=$(system76-power profile 2>/dev/null | sed -n 's/^Power Profile: *//p' | tr 'A-Z' 'a-z' || true)
  system76-power profile performance >/dev/null 2>&1 || echo "Could not set the performance profile; continuing."
  if [ -n "$previous" ]; then
    trap 'system76-power profile "$previous" >/dev/null 2>&1 || true' EXIT
  fi
fi

out="bench-results/$(date -u +%Y%m%dT%H%M%SZ)"
mkdir -p bench-results && mkdir "$out"

# plan NAME ARGS…: one plan into $out/NAME, its console in $out/run-NAME.log.
# The bench writes into a stamped directory under --out, moved up here.
plan() {
  local name=$1 status=0
  shift
  "$bench" run "$@" --out "$out/$name" 2>&1 | tee "$out/run-$name.log" || status=$?
  local stamped=("$out/$name"/*/)
  if [ -d "${stamped[0]}" ]; then mv "${stamped[0]}"* "$out/$name/" && rmdir "${stamped[0]}"; fi
  return "$status"
}

# Smoke at the native factor, so its windows have the default plan's size.
plan smoke --plan smoke --refresh-hz "$HZ" --native-factor "$FACTOR" --factor "$FACTOR" \
  || { echo "The smoke plan failed (see $out/run-smoke.log); the default plan was not run." >&2; exit 1; }

# The factor iced reported for the smoke windows (each raw file's first line).
observed=$(head -qn1 "$out"/smoke/raw/*.jsonl | grep -o '"window_factor":[0-9.]\+' | cut -d: -f2 \
  | sort | uniq -c | sort -rn | awk 'NR == 1 {print $2}' || true)
factor=$FACTOR
if [ -n "$observed" ] && awk -v a="$observed" -v b="$FACTOR" 'BEGIN {exit !(a - b > 1e-3 || b - a > 1e-3)}'; then
  echo "The windows' scale factor is $observed, not $FACTOR: the default plan uses $observed."
  factor=$observed
fi

{
  echo "# dev/bench/run.sh: HZ=$HZ FACTOR=$FACTOR, observed window factor ${observed:-none}, default plan at $factor"
  echo "# power profile before the run: ${previous:-unknown}"
  echo
  cat "$out/smoke/system.txt"
} > "$out/system.txt"

status=0
if ! $smoke_only; then
  plan default --plan default --refresh-hz "$HZ" --native-factor "$factor" || status=$?
fi

echo
echo "Results: $out"
for name in smoke default; do
  if [ -f "$out/$name/report.md" ]; then
    echo "== $name: rule outcome"
    sed -n '/^## Rule outcome/,$p' "$out/$name/report.md" | sed '1d;/^$/d'
    resized=$(grep -c 'resized by the window manager' "$out/$name/report.md" || true)
    [ "$resized" = 0 ] || echo "($resized runs had a window of another size: see Flags in report.md)"
  fi
done
echo "To send: tar czf bench.tgz -C bench-results $(basename "$out")"
exit "$status"
