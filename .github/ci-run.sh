#!/usr/bin/env bash
# Runs a command for CI. When it fails, the output lines that name the
# failure (errors, failed tests, panics, rustfmt diffs) become GitHub error
# annotations, which the checks API reports even where the raw logs are not
# reachable.
#
#   .github/ci-run.sh cargo test --workspace
#   .github/ci-run.sh --retry cargo test -p rustd3-iced
#
# With --retry, a command that fails runs a second time, and only the
# second failure is annotated: the runners' software GPU adapters (lavapipe
# on Linux, WARP on Windows) now and then crash in the first test that
# opens a device, and the rerun passes.
set -uo pipefail
retry=0
if [ "${1:-}" = --retry ]; then retry=1; shift; fi
log=$(mktemp)
"$@" 2>&1 | tee "$log"
status=${PIPESTATUS[0]}
if [ "$status" -ne 0 ] && [ "$retry" -eq 1 ]; then
  echo "::warning::$* failed with exit status $status; running it again"
  "$@" 2>&1 | tee "$log"
  status=${PIPESTATUS[0]}
fi
if [ "$status" -ne 0 ]; then
  echo "::error::$* failed with exit status $status"
  plain=$(sed 's/\x1b\[[0-9;]*m//g' "$log")
  # The lines that name the failure, then the log's last 3000 characters
  # as one annotation (newlines encoded, as workflow commands need).
  printf '%s\n' "$plain" \
    | grep -nE '^(error|warning)(\[E[0-9]+\])?:|^Diff in |panicked at|^test .* FAILED$|^failures:|^---- .* stdout ----|^thread .* panicked|^assertion ' \
    | head -n 8 | while IFS= read -r line; do
      echo "::error::${line:0:400}"
    done
  tail=$(printf '%s' "$plain" | tail -c 3000 | sed 's/%/%25/g' | sed ':a;N;$!ba;s/\n/%0A/g')
  echo "::error::log tail: $tail"
fi
rm -f "$log"
exit "$status"
