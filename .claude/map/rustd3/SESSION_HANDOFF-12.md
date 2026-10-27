# rustd3 — session handoff 12

2026-10-27 · the D11.2 core contract fixes are done in one session (D11.3 open) · next: **Phase 12 — Drawing list and raster outputs**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

Phase 11 is complete (handoff 11), and so are D11.2's fixes to the core
contract, items 1–7, one commit each; `scripts/check.sh quick` passes on
every commit and `full` on the last one (57 s warm).

- **(1) Labels follow the ticks** (78ef1ca, 98d49c7). `Symlog`'s tick
  format follows its tick branch: decades `",.12~g"` (a given specifier
  as is), linear ticks d3's linear format. A scale with `.tick_rule(…)`
  labels by default (`Ticks::labels`, `tick_format(count, None)`) with
  `,f` to the places of the smallest gap between `ticks(count)`, no log
  blanking (`TickFormat::spaced`, crate-private); the gap is read to 12
  significant digits of the largest tick (`format::precision_fixed_between`,
  crate-private), and fewer than two ticks or no gap at that resolution
  keep the transform's format. Tests in `core/tests/main/scale/ticks.rs`
  (the four rows of D11.2's issue, float noise at [0, 1] and [1e9, 1e9 +
  1], the fallbacks).
- **(2) `Continuous::with_range`** (ac867ec): d3's `range(r)` for the
  same type, keeping interpolator, clamp, unknown and tick rule;
  `range_round` is built on it; `range` always resets. The translator
  still maps d3's `range` to `.range` (Phase 4's table); no d3 test sets
  a range after `unknown`, `interpolate` or `rangeRound`.
- **(3) No minors across thinned decades** (84e09a7): `ticks::thinned`
  (crate-private); `Log::minor` walks neighbouring majors and skips gaps
  above one decade, `Symlog::minor` likewise; unchanged elsewhere, bit
  for bit (checked on 20,000 random scales). matplotlib's
  `LogLocator(subs='auto', numticks=5)` gives no minors on [1, 1e12],
  [1e-4, 1e4], [1, 1e9]; ours now too (tests in `scale/ticks.rs`).
- **(4) Width bound** (b9587fb): a specifier width above 536,870,888
  (V8's 2²⁹ − 24) is `InvalidFormat` (`MAX_WIDTH` in
  `format/specifier.rs`).
- **(5) Additive** (d014ed9): `From<format::Locale> for locale::Locale`;
  `Default for interact::Transform`; `Clone` (X, Y: Clone) and a
  bound-free `Debug` (kind, len) for `series::Points`; aliases
  `scale::{Symlog, Logit, Asinh}`.
- **(6) Style** (20e1f3a): the trait lives in the private
  `scale/mapping.rs`, one path `scale::Transform`; `scale::transform`
  holds the types only. `LogKeep` → `Keep`. `InvalidFormat(String)` has
  a private field and `input()`, as `ParseColorError`.
- **(7)** (7d6f91b): Band, Point and Ordinal docs say keys compare with
  `PartialEq` (a NaN key never matches).

Harness: unchanged, 9 suites, 3,314 assertions, 2,623 passing, 691
skipped, 80 floors; nothing blessed. figure, iced and the dev crates
named none of the changed items.

Decisions (`.claude/decisions/phase-11.md`):

- **D11.3** (open) — (1) the tick gap read at the ticks' precision
  (built, recommended); (2) `func(f, g, rule)` with a rule other than
  `Data` keeps d3's linear labels, the third row's defect (not built;
  recommended: `Func::tick_format` uses `TickFormat::spaced`); (3)
  matplotlib's other cut, no minors once ten or more decades are in view
  (not built; 96 minors on log [1, 1e12] at count 10); (4) the private
  module's name `mapping`.

Commits: 78ef1ca, ac867ec, 84e09a7, b9587fb, d014ed9, 20e1f3a, 7d6f91b,
98d49c7, 06385ce (D11.3), and this handoff.

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4–10 | done; D11.2 fixes done (D11.3 open) |
| M2 Figure layer and widget | 11–22 | 11 done; 12 next |
| M3–M5 | 23–40 | pending |

## 5. What to do next

Phase 12 (two sessions, cut after Step 4): read `rustd3.md`'s front
matter and "### Phase 12 —"; it builds on Phases 1 and 7 and starts
`rustd3-figure`. If the architect answers D11.3 (2) or (3) with B, do
that first, in core, one commit each.

## 6. Notes for later phases

- Phase 14: `Scale::with_range` delegates to `Continuous::with_range`
  (keeps a user's unknown, rounding and interpolator).
- Phase 26: the `const` specifier checker applies the same width bound
  (`MAX_WIDTH`, 536,870,888).
- Phase 28: `From<format::Locale> for locale::Locale` must default the
  time half to the neutral one.
- Phase 39: minor ticks come from `Ticks::minor` as built: none across
  gaps above one decade.
- Errors: `InvalidFormat::input()` reads the specifier; tests compare it
  (`map_err(|e| e.input().to_owned())`), not the tuple.
- Handoff 11's notes for Phases 18–21 and 32 stand.

## 7. Key files

`core/src/scale/{continuous,tick_format,mapping,transform}.rs`,
`core/src/scale/transform/{log,symlog}.rs`, `core/src/ticks/rules.rs`,
`core/src/format/{specifier,precision}.rs`, `core/src/locale.rs`,
`core/src/series/points.rs`; `core/tests/main/scale/ticks.rs`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
TZ=Asia/Kathmandu cargo test -p rustd3-core --test main scale::ticks
cargo run -p rustd3-harness -- --check
python3 -c "import matplotlib.ticker as t, inspect; print(inspect.getsourcefile(t))"
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green on every commit; `full` green (57 s).
- D11.3 open with its recommendations.
- Conventional commits, no attribution.
