#!/usr/bin/env bash
# Runs a command for CI. When it fails, the lines of its output that name
# the failure (errors, failed tests, panics, rustfmt diffs) are emitted as
# GitHub error annotations, which the checks API reports, since the raw
# logs are not always reachable from where the results are read.
#
#   scripts/ci-run.sh cargo test --workspace
set -uo pipefail
log=$(mktemp)
"$@" 2>&1 | tee "$log"
status=${PIPESTATUS[0]}
if [ "$status" -ne 0 ]; then
  echo "::error::$* failed with exit status $status"
  grep -nE '^(error|warning)(\[E[0-9]+\])?:|^Diff in |panicked at|^test .* FAILED|^failures:|^---- .* stdout ----|^thread .* panicked|FAILED|assertion' "$log" \
    | sed 's/\x1b\[[0-9;]*m//g' | head -n 9 | while IFS= read -r line; do
      echo "::error::${line:0:400}"
    done
fi
rm -f "$log"
exit "$status"
