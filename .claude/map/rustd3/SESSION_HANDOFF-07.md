# rustd3 — session handoff 07

2026-10-27 · Phase 7 complete (D7.3 open; D7.1, D7.2 decided) · next: **Phase 8 — Lines, areas and curves**

## 1. Project

rustd3 is a Rust plotting library that speaks d3's vocabulary and adds
matplotlib's tier-1 pieces (typst text and math, legends, colorbars,
layout from measured text, statistical charts), rendering SVG, PNG, PDF
and GIF from one drawing list, with an iced integration. The
specification is `rustd3.md` next to this file; `CLAUDE.md` at the
repository root is the entry point.

## 2. Where things stand

Phase 7 is complete; `scripts/check.sh quick` and `full` pass (full:
23 s warm; no cold build this session).

- `rustd3-core`:
  - `scale::Band<K>` / `scale::band(keys)` (d3's scaleBand: builders
    recompute step, start and bandwidth as band.js's `rescale`; lookup
    is a linear search, `None` outside the domain); `scale::Point<K>` /
    `scale::point(keys)` wraps a band with inner padding 1 (`padding` is
    the outer padding; no inner/outer builders).
  - `scale::Ordinal<K, V>` / `scale::ordinal(keys, range)`: explicit,
    deduplicated domain; `apply` (range[i % len] or the unknown value)
    and the opt-in `apply_or_insert(&mut self)`.
  - `Ticks` with `Step = ()` for `Band<K>`, `Point<K>` and
    `Ordinal<K, f64>` (`K: Clone + PartialEq + Display`): every key,
    labels by `Display`, band positions through d3-axis's `center`
    (point delegates to it, D7.3 (1)).
  - `color::{Rgb, Hsl, ParseColorError}`: d3-color 3.1.0's rgb and hsl
    as plain data, both conversions, brighter/darker, clamp,
    displayable, `Display` (formatRgb), `format_hex`, `format_hex8`,
    `format_hsl`; a hand-written port of `color(format)` in
    `color/parse.rs` (JavaScript's whitespace, ASCII lowercase, hex,
    rgb/rgba, hsl/hsla, 148 names in `color/named.rs`, `transparent`,
    d3's alpha/lightness/saturation rules). `BRIGHTER`/`DARKER` are
    private consts in `color.rs` (D7.3 (4)).
  - `Interpolate for Rgb`: d3's `interpolateRgb` at gamma 1 (`nogamma`);
    `scale::linear().range([white, steel])` is a `Linear<Rgb>`.
  - `scheme::{CATEGORY10, OBSERVABLE10, DEEP, COLORBLIND, OKABE_ITO}`.
- Tests: core 27 unit + 244 in `main` (new: `band.rs`, `point.rs`,
  `ordinal.rs`, `color.rs`, `scheme.rs`) + 36 doctests + 6
  `compile_fail` with twins (new: `axis::bottom` on a `Band<Bin>`
  without `Display`, E0277; `invert` on `Linear<Rgb>`, E0599; both
  error codes checked with rustc by hand).
- Harness: `rules/band.mjs` (band and point; the shared `chain`,
  `calls` and `list` helpers), `rules/ordinal.mjs` (key/value types
  resolved by the `finish` hook), `rules/color.mjs` (rgb, hsl, color,
  asserts.js helpers, field writes, `copy({…})`, interpolateRgb);
  189 code lines (228 with comments) against the budget of 200.
  `expr.mjs` gained `combine`, test-file assertion helpers, `finish`
  and `assign` hooks, `strictEqual(x, null)` as an `Err` check, and a
  string literal's `raw`; `rules/scale.mjs` translates color ranges and
  shadows a variable whose range changes type. New suite `color`
  (d3-color); `interpolate` adds `rgb-test.js`; `scale` adds band,
  point, ordinal. Prelude: `rgb_equal`, `rgb_approx_equal`,
  `hsl_equal`, `interpolate_rgb`.

Harness counts (translated = passing; every passing assertion also
passes d3's semantics):

| Suite | Assertions | Passing | Skipped |
| --- | --- | --- | --- |
| path | 79 | 74 | js 3, types 2 |
| ease | 431 | 365 | js 66 |
| ticks | 241 | 241 | — |
| format | 729 | 639 | gap 51, js 21, types 18 |
| scale | 1,104 | 723 | api 106, gap 6, js 174, types 95 |
| interpolate | 49 | 38 | gap 8, js 3 |
| color | 333 | 298 | js 29, types 6 |
| total | 2,966 | 2,378 | 588 |

Floors: 57 (was 50). Raised: linear 153 → 171, pow 155 → 161, log
121 → 126. New: band 77/104, point 14/22 (D7.1), ordinal 63/87, color
79/79, rgb 97/116, hsl 122/138, interpolate rgb 11/22. linear (69 %)
and pow (66 %) stay under 70 % in D4.2's skip-table rows; the
measurements section of `decisions/phase-07.md` says why D4.2's
estimate (71 %, 72 %) was not reached.

Decisions (`.claude/decisions/phase-07.md`):

- **D7.1** (decided, the spec's answer) — `point-test.js` 14 of 22:
  getters `api` 6, absent inner/outer padding `types` 2; floor 14.
- **D7.2** (decided, the spec's answer) — `.unknown(undefined)`: no
  call, `s(x)` maps to `apply`. Note: `apply_or_insert` appends even
  when an unknown value is set (as specified); d3 would not, and no
  translated test reaches it.
- **D7.3** (open, recommending what is built) — (1) point positions
  through d3-axis's `center`; (2) the `Hsl::from("#808080")` spot row
  as `"#808080".parse::<Hsl>()`; (3) JavaScript's whitespace for trim;
  (4) `BRIGHTER`/`DARKER` private.

Commits: db97b1c (band, point), 81f33a0 (ordinal), 4b939e2 (Rgb, Hsl,
parsing), ae2979a (rgb interpolation, schemes), bf31f49 (private
factors), c218bd9 (harness), 21c1ae4 (D7.1–D7.3), and this handoff.

## 3. Project management

- `goon.yaml` still has `repo: none`; no pushing in agent sessions.
- Decisions: `.claude/decisions/phase-NN.md`; index in
  `.claude/DECISIONS.md`.

## 4. The plan

| Milestone | Phases | Status |
| --- | --- | --- |
| M0 Foundation | 1–3 | done |
| M1 Core contract | 4–10 | 7 done (D7.3 open); 8 next |
| M2–M5 | 11–40 | pending |

## 5. What to do next

Phase 8: read `rustd3.md`'s front matter and "### Phase 8 —" (it
builds on Phase 3 only), and section 6 below.

## 6. Notes for later phases

- Phase 12 (drawing list) and 14+ (figure): colors are `color::Rgb`
  values; `Display` is d3's `rgb(…)`/`rgba(…)` string, `format_hex`
  the `#rrggbb` one.
- Phase 14+ (band handles): `Band::apply` is the band start; the
  center is `Ticks::position(v, 0.0)`. Lookups are linear (`K:
  PartialEq`); a hashed lookup stays out of v0.1.
- Phase 17 (legends): ordinal scales over colors have no `Ticks`; read
  `get_domain()` and `apply`.
- Phase 23: `Interpolate for Rgb` is gamma 1; `interpolateRgb.gamma`
  (8 skips in `d3-interpolate/test/rgb-test.js`), `interpolateHsl` (6
  in `log-test.js`, `gap: phase 23 (interpolate of a color range)`) and
  Lab/HCL/Cubehelix/Gray come then. The color suite is
  `translate.mjs`'s `color` (d3-color); add `lab`, `hcl`, `lch`,
  `cubehelix`, `gray` there. `rules/color.mjs`'s `into(e, space)`
  converts strings and colors for a target space; `FIELDS`/`TYPES`
  extend per space. `BRIGHTER`/`DARKER` are reachable inside `color`.
- Harness: a scale chain on a variable assigns it (`chain` in
  `rules/band.mjs`, as in `rules/scale.mjs`); a rules file can name a
  test file's own assertion helpers (`assertions`) and rewrite lines
  once a test is read (`finish`).
- Phase 10: still owes `pub(crate) fn forward(&self) -> &[f64; N]` on
  `Continuous` with `Rescale` and `Pad` (D6.1).

## 7. Key files

`CLAUDE.md`; `.claude/decisions/phase-07.md`;
`core/src/scale/{band,point,ordinal}.rs`, `core/src/color.rs`,
`core/src/color/{rgb,hsl,parse,named,error}.rs`, `core/src/scheme.rs`;
`core/tests/main/{band,point,ordinal,color,scheme}.rs`;
`dev/harness/translate/rules/{band,ordinal,color,scale}.mjs`,
`dev/harness/translate/expr.mjs`, `dev/harness/src/prelude.rs`,
`dev/harness/passing.txt`.

## 8. Commands

```bash
scripts/check.sh quick                           # every commit (runs the ratchet)
scripts/check.sh full                            # end of a phase (background)
cargo test -p rustd3-core --test main color
node dev/harness/translate/translate.mjs [--check]
cargo run -p rustd3-harness [-- --failing [prefix] | --check | --bless [--allow-drop] [--bless-floors]]
node --import ./dev/harness/translate/d3-resolve.mjs script.mjs   # d3 in Node
```

## 9. End-of-session checklist

- `scripts/check.sh quick` green; `full` green (23 s).
- D7.3 open with its recommendation; D7.1 and D7.2 decided.
- Conventional commits, no attribution.
