# Phase 34 — Figure marks breadth

## D34.1 — `series::Error` gains `Recordings`

**Date:** 2026-10-28 · **Context:** Phase 34, step 1 · **Status:** decided

### Decision

The section's answer (Design, Core: `series::m4_joint`; Conventions,
item 7): Phase 9's public `series::Error<X>` gains `Recordings { left:
series::Id, right: series::Id }`, returned by `Series::m4_joint` for
channels of two recordings, checked before the columns. No crate of the
workspace matches on `series::Error` exhaustively (figure, iced, examples
and bench only name it in doctests' `Ok::<(), …>`), so no call site
changes. `m4::Lines::thin`, crate-private, now takes up to two channels'
extremes per column, which keeps `Series::m4` as it was.
