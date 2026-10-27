# rustd3 — session handoff 06

2026-10-27 · Phase 6 complete (D6.1 open; D6.2, D6.3 decided) · next: **Phase 7 — Categorical scales and basic color**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

Phase 6 is complete; `scripts/check.sh quick` and `full` pass (full:
22 s warm; no cold build this session).

- `rustd3-core`:
  - `locale::Locale` (`NEUTRAL`, `EN_US`, `number()`, `Default`), the
    number half of the figure locale; private field for Phase 28's time
    half.
  - `Format::format_all`: the batch grouping rule (all or nothing under
    a locale grouping from N digits, digits counted as printed);
    `format` and `format_all` share `parts` and `finish`. Crate-private
    `Format::with_locale` rebinds a format (same specifier and fixed SI
    prefix; the SI factor and symbol are kept in the `si` field).
  - `TickFormat::format_all` delegates to `Format::format_all`, then
    blanks log labels; `TickFormat::labels(&Set, &locale::Locale)`
    rebinds when the locales differ.
  - `ticks::Set<D, S>` (`new`, `values`, `step`, `count`,
    `with_values`); `scale::Ticks` (final) with impls for every
    `Continuous<T, f64, N>` (`scale/ticks/continuous.rs`) and
    `scale::Identity` (`scale/ticks/identity.rs`); the shared minor rule
    is `scale::ticks::minor` (one-step extension in transformed space,
    extensions not finite or not past their end left out).
  - Crate-private `Continuous::transform()`.
  - `scale::Identity` and `scale::identity()` (d3's scaleIdentity, two
    stops).
  - `axis::{Axis, new, top, right, bottom, left, Orient, Label,
    Anchor}`: `draw` into any `PathContext`, `labels()` for the host.
  - `Symlog::minor` also takes the decade rule when every major but the
    first and last is a decade and two inner ones share a sign (D6.1
    (3)); unchanged on the raw majors of every built-in rule.
  - `Transform::tick_format` documents that its default specifier must
    parse (D6.3).
- Tests: core 22 unit + 189 in `main` (new: `locale.rs`, `axis.rs` with
  the recording `PathContext` and the snapshot parser,
  `scale/ticks.rs`, `scale/identity.rs`; the fuzz also checks the axis
  minor ticks and one label per tick) + 26 doctests + 4 `compile_fail`
  with twins (new: `axis::bottom(&"not a scale")`, E0277 checked).
- The fuzz grew with labels: 2,000 debug cases 0.3 s → 4.5 s; 20,000
  release cases (`RUSTD3_FUZZ=1`) 0.85 s → 10.9 s, under the 60 s budget.
- Harness: `rules/scale.mjs` 174 lines (+11, −4):
  `scaleIdentity` (one argument is the domain; domain and range one
  array; other than two stops is a `types:` skip); `rules/array.mjs`:
  `reverse()` on string arrays; `translate.mjs` adds `identity` to the
  scale suite.

Harness counts (every passing assertion also passes d3's semantics):

| Suite | Assertions | Passing | Skipped |
| --- | --- | --- | --- |
| path | 79 | 74 | js 3, types 2 |
| ease | 431 | 365 | js 66 |
| ticks | 241 | 241 | — |
| format | 729 | 639 | gap 51, js 21, types 18 |
| scale | 891 | 540 | api 67, gap 59, js 142, types 83 |
| interpolate | 27 | 27 | — |
| total | 2,398 | 1,886 | 512 |

Floors: 50 (was 49): `identity-test.js` 44 of 90 added; the others
unchanged. `axis-test.js` and `snapshot-test.js` stay untranslated (the
skip table); `axis_snapshots_of_d3_are_reproduced` grades the two
snapshots.

Decisions (`.claude/decisions/phase-06.md`):

- **D6.1** (open, recommending what is built) — (1) `group_from(digits)`
  and (2) `Continuous::forward()` not added: no caller (forward waits for
  Phase 10); (3) the one-step extension broke symlog's decade minors, so
  `Symlog::minor` accepts free first and last majors (verified
  bit-identical to Phase 5's rule on 200,000 random raw major lists);
  (4) extensions not past their end are left out (fuzz); (5) the inherent
  `Continuous::minor(count)` shadows `Ticks::minor` on concrete scales
  (E0308); call `Ticks::minor(&scale, &set)`.
- **D6.2** (decided, the spec's answer) — `identity-test.js` 44 of 90:
  js 34, types 12 (three-stop domains); floor 44.
- **D6.3** (decided, the spec's answer) — a transform's default tick
  format must parse; `Ticks::labels` panics otherwise.

Commits: 21913c7 (locale, batch labels), cb380f0 (Set, Ticks), 5fcf122
(identity, harness), 09d348f (axes), eaac87b (axis snapshots), 4ea0646
(Transform doc), b0b3535 (SI rebinding test), 39c2ef5 (symlog fix),
eab1f7b (D6.1–D6.3), and this handoff.

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4–10 | 6 done (D6.1 open); 7 next |
| M2–M5 | 11–40 | pending |

## 5. What to do next

Phase 7: read `rustd3.md`'s front matter and "### Phase 7 —",
`decisions/phase-04.md` to `phase-06.md`, and section 6 below.

## 6. Notes for later phases

- Phase 7: `Ticks` for `Band<K>`, `Point<K>`, `Ordinal<K, f64>` (`Step =
  ()`); `position(value, offset)` is where d3's `center` applies the
  bandwidth and offset; the axis already passes its offset. Color ranges
  re-run the scale suite (D4.2 floors of linear and pow).
- Inside generic code `scale.minor(&set)` is the trait's; on a concrete
  `Continuous` write `Ticks::minor(&scale, &set)` (D6.1 (5)).
- Phase 10: add `pub(crate) fn forward(&self) -> &[f64; N]` on
  `Continuous` with `Rescale` and `Pad`; `Rescale` and `Pad` for
  `scale::Identity`.
- Phase 14+: `axis::Axis` is the core contract; figure axes compute
  their own tick counts. `Axis` is generic over `S: Ticks` and borrows
  the scale and the format closure for `'s`.
- Phase 26: `locale::Locale` gains named tables through its private
  field; `Format::format_all` already applies to any `group_from`.
- Phase 28: `Locale`'s time half; `Ticks` for time scales with `Step =
  time::Interval`.
- Phase 39: symlog decade minors come from `Ticks::minor` (D6.1 (3)).

## 7. Key files

`CLAUDE.md`; `.claude/decisions/phase-06.md`; `core/src/locale.rs`,
`core/src/format.rs`, `core/src/scale/{tick_format,identity,ticks}.rs`,
`core/src/scale/ticks/{continuous,identity}.rs`,
`core/src/ticks/set.rs`, `core/src/axis.rs`,
`core/src/axis/{orient,label}.rs`,
`core/src/scale/transform/symlog.rs`;
`core/tests/main/{locale,axis,fuzz}.rs`,
`core/tests/main/scale/{ticks,identity}.rs`;
`dev/harness/translate/rules/{scale,array}.mjs`;
`dev/harness/passing.txt`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
RUSTD3_FUZZ=1 scripts/check.sh quick             # plus 20,000 fuzz cases in release
scripts/check.sh full                            # end of a phase (background)
cargo test -p rustd3-core --test main axis_snapshots
node dev/harness/translate/translate.mjs [--check]
cargo run -p rustd3-harness [-- --failing [prefix] | --check | --bless [--allow-drop] [--bless-floors]]
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green; `full` green (22 s).
- D6.1 open with its recommendation; D6.2 and D6.3 decided.
- Conventional commits, no attribution.
