# Fixtures

Reference values that rustd3's tests compare against, for behavior that
d3's own test files do not cover or that comes from matplotlib. Each
fixture is a JSON file committed together with the script that writes
it; tests read the JSON and compare through `rustd3_testkit`'s `Obs`
values, within tolerance, never byte for byte. Fixtures are regenerated
only when a phase section says so, and a rerun of an unchanged script
writes byte-identical JSON (sorted keys, floats through `repr` or
JavaScript's own number formatting), so `git diff --exit-code
dev/fixtures` shows whether a regeneration changed anything.

## d3 fixtures (`*.mjs`)

Plain Node scripts, with no `node_modules`: they import d3 from the
sources vendored under `dev/harness/d3/` (from Phase 2) through the
resolve hook, so the values are those of the measured d3 versions.

```bash
node --import ./dev/harness/translate/d3-resolve.mjs dev/fixtures/<name>.mjs
```

A script that formats or parses time runs with
`TZ=America/Los_Angeles`, as d3's time tests do.

## matplotlib fixtures (`*.py`)

Python 3 scripts against matplotlib 3.11.2 (with numpy, and scipy or
seaborn where a fixture needs them). Each asserts
`matplotlib.__version__ == "3.11.2"` before writing, so a different
version fails instead of silently changing the values.

```bash
python3 dev/fixtures/<name>.py
```

## Written by a table generator

`format_locales.json` is written by `dev/harness/gen/format_locales.mjs`
together with the number-locale tables it checks (d3's own
`formatLocale` over a fixed probe list per locale), and
`time_locales.json` by `dev/harness/gen/time_locales.mjs` (under
`TZ=America/Los_Angeles`) together with the time-locale tables (d3's own
`timeFormatLocale` over seven specifiers on four instants); `scripts/check.sh
full` reruns both generators and fails on any difference.
