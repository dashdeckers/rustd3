# Phase 24 — `#[derive(Interpolate)]`

## Measurements

2026-10-27, 4f669da … e870a57, one session (Phase 25 follows in it).
`scripts/check.sh full` green at e870a57 in about 9 min (warm, the
benchmark smoke included), no warnings.
Harness unchanged: 9 suites, 3,822 assertions, 3,040 passing, 782
skipped, 97 floors (no d3 tests here). Doctests on the re-export: six
passing twins and five `compile_fail` examples, each compiled once
unmarked and seen to fail for its stated reason:

| Example | Error |
| --- | --- |
| `struct Bad { name: String }` | E0277, the trait bound `String: Interpolate` is not satisfied |
| a field of a user type without an impl | E0277, `Weight: Interpolate` |
| `lerp` on `Mark<String>` | E0599, the method `lerp` exists for `Mark<String>`, but its trait bounds were not satisfied |
| `enum Shape { Dot { r: f64 } }` | the derive's error, "Interpolate derives only for structs with named fields" (no code) |
| `struct P(f64, f64);` | the same |

A twin broken on purpose (`struct Dot(` for `struct Dot {`) failed
`cargo test -p rustd3-core --features derive --doc`, so the doctests on
the `pub use` run.

## D24.1 — The field bounds do not make `Self: Clone` hold

**Date:** 2026-10-27 · **Context:** Phase 24, step 1 (4f669da) · **Status:** decided

### Issue

Phase 24, Builds on: "The `Clone` supertrait means a deriving struct also
derives `Clone`; the generated bounds make that hold (a type parameter in
a non-skipped field is bounded by `Interpolate`, hence `Clone`; one in a
skipped field by `Clone`)". Design, Bounds: the bound is on the *field
type* (`FieldType: Interpolate`, `FieldType: Clone`). The two agree only
when a parameter is a whole field's type. A derived `Clone` needs every
type parameter `Clone`, and the compiler cannot get `T: Clone` from
`Mark<T>: Interpolate` or from `PhantomData<U>: Clone`, so with the
bounds as written

```rust
#[derive(Clone, Interpolate)]
struct Nested<T> { m: Mark<T>, k: f64 }

#[derive(Clone, Interpolate)]
struct Tagged<U> { x: f64, #[interpolate(skip)] unit: PhantomData<U> }
```

fail at the derive site with `error[E0277]: the trait bound T: Clone is
not satisfied` (and `U: Clone`), although "`PhantomData<T>` fields need
`skip`" says the second one works.

### Investigation

Compiled both through the derive as written (rustc 1.97.0): E0277 on
each, from the supertrait obligation `Nested<T>: Clone`. With one more
predicate, `Self: ::core::clone::Clone`, on structs that have a type
parameter, both compile and interpolate (tests
`a_nested_mark_blends_through_its_own_impl`,
`a_parameter_only_a_skipped_phantom_names_still_derives`). Wherever the
written bounds compiled, they already proved `Self: Clone`, so the extra
predicate changes no impl that existed; it only admits the ones that
failed.

### Options

| | **A — add `Self: Clone` on generic structs** | B — also bound each mentioned parameter by `Clone` | C — as written |
| --- | --- | --- | --- |
| Approach | One predicate after the field bounds when the struct has a type parameter | `T: Clone` for every parameter a field names | Field bounds only |
| Correctness | Exact: the impl exists when the fields blend and the struct is `Clone`, whatever its `Clone` impl requires | Too strong for a hand-written `Clone` with weaker bounds; still misses nothing derived `Clone` needs | Nested generics and phantom parameters do not compile |
| Cost | Two lines; the unit test's expansion of `Mark<T>` gains `Self: ::core::clone::Clone,` | A second walk collecting parameters | — |

### Decision

A (architect, 2026-10-27), as built: the derive bounds every field type
as the spec says and adds `Self: Clone` for generic structs, so a type
parameter that appears only nested or in a skipped `PhantomData<T>`
compiles.

### Why

It is the supertrait obligation itself, stated once, so the impl's
applicability is exactly "every field blends or clones, and the struct
is `Clone`". It follows the compiler where the written rule cannot
compile, and changes nothing that compiled.

### Followups

None; Phase 30's `anim::Marks` needs only the trait.

## D24.2 — Details of Phase 24 as built

**Date:** 2026-10-27 · **Context:** Phase 24, 4f669da … e870a57 · **Status:** decided

### Issue

1. **Error codes.** "Each failing doctest is `compile_fail`"; the
   dispatch asks for exact codes. `lerp` on `Mark<String>` is E0599 (a
   method whose trait bounds fail), not E0277; the enum and tuple-struct
   examples fail with the derive's own `compile_error!`, which has no
   code.
2. **A duplicate `crate` key** (`#[interpolate(crate = "a", crate =
   "b")]`) is not covered by the rules.
3. **Error collection.** `parse_nested_meta` stops an attribute at its
   first bad key, so errors are combined across attributes and fields,
   not within one attribute's list.
4. **Shapes the type walk does not open** (bare `fn` types, trait
   objects, `impl Trait`, macros, verbatim tokens, associated-type
   constraints): the section lists the shapes to walk, not the others.
5. **Rustdoc order.** The inlined page shows the `pub use` doc comment
   before the derive function's own.
6. **Unit tests** beyond the one the section names.

### Options

| | **A — as built** |
| --- | --- |
| (1) | `compile_fail,E0277` twice, `compile_fail,E0599`, plain `compile_fail` for the two shape errors; the measurements table above records each message |
| (2) | A spanned error, "duplicate interpolate attribute `crate`" |
| (3) | Every attribute's first error and every field's, combined with `syn::Error::combine` |
| (4) | Counted as naming a parameter: the bound that adds is then either trivially true, or trivially false and fails with E0277 exactly as the field would without it, so no struct changes outcome; skipped when the struct has no type parameter |
| (5) | The re-export's comment opens with the summary line and the examples; the derive function's comment states the rules and closes the page |
| (6) | Bounds and where-clause merging on a struct with a lifetime, a const parameter and its own `where`; the `crate` path; unknown keys on fields and the container reported together; the four rejected shapes; the duplicate `crate` |

### Decision

A for all six (architect, 2026-10-27), as built.

### Why

(1) records what the compiler emits; (2) and (3) keep the rule that
malformed attributes are errors, never ignored; (4) is the conservative
reading that cannot change a struct's outcome; (5) and (6) are
documentation and tests only.

### Followups

None.
