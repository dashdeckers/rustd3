# Phase 6 — Ticks, locale, core axes and the identity scale

## D6.1 — Statements of Phase 6 that do not hold as written

**Date:** 2026-10-27 · **Context:** Phase 6, steps 1–3 (21913c7, cb380f0, 39c2ef5) · **Status:** decided

### Issue

1. **`Locale::group_from(digits)` has no caller.** Phase 3, Design,
   `format`: `pub(crate) fn group_from(self, digits: usize) -> Locale`,
   whose only caller was to be Phase 6's `Format::format_all` (D3.2,
   Followups). `format_all` as built decides the grouping once from the
   printed integer digits of every value and hands that decision to the
   tail `format` shares; no locale is rebound, so the builder would be
   dead code (D3.2's rule).
2. **`Continuous::forward()` has no caller in Phase 6.** D4.1 (2),
   Followups: "Phase 6 (axes) adds `transform()` and `forward()` with
   their first caller". The axes read `Ticks::extent` (the range ends)
   and `Ticks::position` (`apply`); nothing in Phase 6 reads the forward
   cache. `transform()` is added: `Ticks::minor` reads it.
3. **Symlog loses its decade minors under the extension rule.** Phase 6,
   Design, `impl Ticks for Continuous`, `minor(set)`: extend the majors by
   one step on each side in transformed space, then call
   `Transform::minor` on the extended list. Phase 5's `Symlog::minor`
   takes the decade rule only when every nonzero major is ±c·10ᵏ, and an
   extension of decade majors is not a decade: over `symlog().domain([0,
   5000])` the majors 0, 1, 10, 100, 1000 extend to −1 and 9919.80, so the
   whole list falls back to five equal steps in data space: 0.2, 0.4,
   0.6, 0.8, 2.8, 4.6, 6.4, 8.2, 28, 46, …, 820, 2784.0, 4567.9. Part I,
   Scales ("so log and symlog provide 2–9 per decade … Axes draw minor
   ticks with `.minor(true)`") and Phase 39, step 2 ("the symlog decade
   minors", drawn from core's `Ticks::minor`) need 2–9 per decade.
4. **An extension may not lie past its end.** Not in the rule as
   written. Under a transform undefined beyond an end, 2u₀ − u₁ maps back
   inside the domain: `func(sqrt, |y| y * y, …)` with
   `TickRule::Transformed` over `[140869.4, 0]` extends 0 to a positive
   value (the square of a negative u), the extended list leaves domain
   order, and the minor ticks come out of order (the fuzz, fixed seed,
   case 101).
5. **`Ticks::minor` is shadowed on concrete continuous scales.** The
   Design explains why the trait says `tick_set` and not `ticks` (an
   inherent method shadows a trait method), but keeps the trait's `minor`
   beside Phase 4's inherent `Continuous::minor(count)`. On a concrete
   `Continuous`, `scale.minor(&set)` resolves to the inherent method and
   does not type-check (E0308); generic code (the axes) and
   `Ticks::minor(&scale, &set)` work.

### Investigation

- (1), (2): `grep -rn 'group_from\|fn forward' core/src` at cb380f0:
  `group_from` is only the locale field; `Continuous` has no `forward()`.
- (3): `core/src/scale/transform/symlog.rs` at 51fddb7 (Phase 5), `minor`:
  `exponents.collect::<Option<…>>()` fails on any non-decade nonzero
  value and returns `ticks::subdivide(&transform::Identity, major, 5)`.
  The extension values above are computed with symlog's `forward` and
  `inverse` (constant 1). Log is not affected: `Log::minor` works from
  the magnitudes of the first and last value, so the extension only
  widens its range.
- (3), as built: a first rule (cb380f0) freed the ends whenever two
  majors were decades and so also caught evenly spaced majors:
  `symlog().domain([-20, 20]).minor(4)` over [−20, −10, 0, 10, 20] lost
  its 16 fifths (`rules.rs`,
  `symlog_minor_ticks_subdivide_symmetric_linear_majors_through_decades`).
  39c2ef5 asks for two inner decades of one sign, which evenly spaced
  majors never hold (two decades of one sign are at least a factor 10
  apart, neighbouring multiples of a step at most a factor 2). A
  throwaway test compared the built rule with a copy of Phase 5's on the
  raw majors of 200,000 random symlog scales (constants 0.1–10, domains
  across 0, counts 1–20, the own, `Data` and `Transformed` rules; 36,022
  in decade mode with minor ticks): the same values, bit for bit.
- (4): `core/tests/main/fuzz.rs` at cb380f0 asserts the axis minor ticks
  (finite, in domain order, inside the ends) and one label per tick for
  every case; 2,000 debug and 20,000 release cases pass.

### Options

| | **A — as built** | B — as written | C — per-segment |
| --- | --- | --- | --- |
| (1), (2) | Not added; `forward()` arrives with Phase 10's `Rescale` and `Pad` | Dead code, rejected by `-D warnings` | — |
| (3) | `Symlog::minor` takes the decade rule, as before, when every nonzero major is ±c·10ᵏ, and also when every one but the first and last is and two of those share a sign; m·c·10ᵏ strictly between neighbouring majors of one sign. [0, 5000] gets 2–9, 20–90, 200–900, 2000–5000 | Data-space fifths on every symlog decade axis | `Ticks::minor` calls `Transform::minor` on the majors and on each edge pair: decades inside, data-space fifths at the edges (2784.0, 4567.9 in [1000, 5000]) |
| (4) | An extension that is not finite or not past its end is left out | Minor ticks out of order | — |
| (5) | As written; the trait's doc names `Ticks::minor(&scale, &set)` | — | Rename the inherent `Continuous::minor` (a Phase 4 public item) |

### Decision

A for all five (architect, 2026-10-27): `Format::format_all` reads the
locale's `group_from` itself, so no `group_from(digits)` method is
added; `Continuous::forward()` arrives with Phase 10; symlog's minors
use the decade rule when the inner majors are decades, as Part I and
Phase 39 expect; extensions that are not finite or not past their end
are dropped; `Ticks::minor` is called through the trait where the
inherent `Continuous::minor(count)` would shadow it.

Recommended: A for all five, already built (21913c7, cb380f0, 39c2ef5).
For (3), the majors of symlog's own, `Data` and `Transformed` rules get
Phase 5's minor ticks unchanged; only lists with two inner decades of
one sign and an end off the decades change, which the extension and
custom tick rules make: a custom rule's majors [5, 10, 100, 250] get
6–9, 20–90, 200 where they got data-space fifths.

### Why

(1), (2) and (5) follow D3.2's and D4.1's rule for code without a caller
and keep the API as specified; (3) keeps both the stated extension rule
and Part I's symlog decade minors without a new `Transform` method, where
C keeps the decades only between the outer majors; (4) is Phase 3's
bail-out rule applied to the property the fuzz asserts.

### Followups

- Phase 10: `pub(crate) fn forward(&self) -> &[f64; N]` on `Continuous`
  with `Rescale` and `Pad`.
- Phase 39: the symlog decade minors come from `Ticks::minor` as built.

## D6.2 — `identity-test.js` passes under 70 %

**Date:** 2026-10-27 · **Context:** Phase 6, step 3 (5fcf122) · **Status:** decided

### Issue

Phase 6, Grading: `identity-test.js`'s floor "must reach 70 %". "Decisions
this phase may meet" foresees a shortfall and recommends blessing the
passing ids and recording the counts and causes. The file has 90
assertion calls; 44 translate and pass (49 %), each also under d3's
semantics.

| Category | Count | Assertions (lines of `identity-test.js`) |
| --- | --- | --- |
| js | 24 | Dates, strings and `new Number` as domain or range values (81–109) |
| js | 3 | strings as range values, then Dates and colors (50, 52, 54) |
| js | 5 | `s("2")`, `s(undefined)`, `s(null)`, `s("N/A")`, `s.invert("2")` (27, 32, 33, 35, 59) |
| js | 2 | a domain from a `Set` (114, 115) |
| types | 8 | a three-stop "polyidentity" domain (120–128) |
| types | 4 | the alias test, whose setup sets a three-stop domain (64–69) |

### Investigation

`scale::Identity` holds two stops (`stops: [f64; 2]`, Phase 6, Design),
so the twelve `types:` skips follow from the specified type; the
translator skips a whole test when a setup statement skips, which takes
the alias test's `deepStrictEqual(s.domain(), s.range())` (line 65) with
it. No skip is a translator gap.

### Options

| | **A — skip-table row, floor 44** | B — reach 70 % | C — drop the file |
| --- | --- | --- | --- |
| Approach | Row: `identity-test.js`, "coercion of Dates, strings, `Number` objects, `null` and iterables (`js:`), three-stop domains (`types:`)" | N-stop identity scales and string coercion: beyond the design | Contradicts Exit criteria 3 |
| Correctness | Counts stay visible in the report and the floor | Tests of behaviour Rust does not have | Hides 90 assertions |

### Decision

A, the spec's recommended answer: Phase 6's skip table gains
`identity-test.js` (90 assertions; coercion `js:` 34, three-stop domains
`types:` 12), floor 44, blessed in 5fcf122 after reading the diff (44 ids
added, none removed).

### Why

As D4.2 and D5.2: every shortfall follows from the two-stop type or from
JavaScript coercion.

### Followups

None.

## D6.3 — A transform whose default tick format fails

**Date:** 2026-10-27 · **Context:** Phase 6, step 2 (cb380f0, 4ea0646) · **Status:** decided

### Issue

`Ticks::labels` for continuous scales calls the transform's
`tick_format(…, None)` and treats an `Err` as a broken invariant. A user
`Transform` overriding `tick_format` with a default specifier that does
not parse makes axis labels panic.

### Options

| | **A — document and keep the panic** | B — fall back to a default format |
| --- | --- | --- |
| Approach | `Transform::tick_format` states that with no specifier the result must be `Ok`; `labels` panics with "invariant: a transform's default tick format specifier parses" | Labels silently switch to `,f` |
| Correctness | A bug in the override surfaces at once | Hides the override's bug |

### Decision

A, the spec's recommended answer (4ea0646).

### Why

The built-in transforms' defaults are literals that parse; a user
override is the only way to fail, and failing loudly names it.

### Followups

None.
