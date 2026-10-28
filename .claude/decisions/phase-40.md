# Phase 40 — Facade, documentation and release preparation

## Measurements

2026-10-28, one session (526f512 … the handoff). cargo and rustc 1.97.0.

**Before starting.** `cargo search rustd3` (outside the workspace) lists
no crate; `cargo info rustd3` outside the workspace: "error: could not
find `rustd3` in registry". Inside the workspace `cargo info rustd3`
describes the local facade, so the check runs elsewhere (RELEASING.md
item 1 is run from any directory but the repository's).

**Harness** unchanged (no core change): 16 suites, 6,722 assertions,
5,292 passing, 1,430 skipped, 0 failing, 179 floors; nothing blessed.

**D31.10's addition** (526f512): `land` applies a check before
calibrating. `verify_checks_leave_the_cost_model_as_it_was` holds every
prediction of a refresh of checks only to the bit; with the old order the
same refresh moved them (0.49 → 0.53, 1.83 → 2.94, 9.31 → 15.5 ms at
64 Ki px, 64 Ki px + 400 items, 600 Ki px + 2,000 items).

**The weak `fonts` feature.** `cargo tree -p rustd3 --no-default-features
--features figure -e features -i rustd3-fonts`: "package ID specification
`rustd3-fonts` did not match any packages" (not in the tree); with
`--features figure,fonts` it is, through `rustd3-figure feature "fonts"`.
`cargo check -p rustd3` passes with no features, each of the nine
features alone, the default and all features.

**Crates each feature builds** (`cargo tree -p rustd3 -e normal`, unique
packages): no features 10 (rustd3, rustd3-core and 8 more: jiff,
jiff-core, thiserror, thiserror-impl, syn 3, quote, proc-macro2,
unicode-ident); `derive` +2; `figure` 273; then `fonts` +1, `raster` +10
(Part I says +7), `png` and `gif` +10 (nothing beyond `raster`), `pdf`
+5, `tiny-skia` +1, `helpers` 0; all features 291.

**MSRV floors** (`cargo metadata … | sort -V`): the published crates'
highest are 1.92 (typst 0.15.1 and its 12 sibling crates, typst-layout,
krilla 0.8.2, hayro 0.7 and its 5 crates), then ordered-float 5.5.1
1.90, pic-scale 0.7.12 and uuid 1.29.0 1.89. iced 0.15.0-dev, iced_program,
iced_selector and iced_test declare 1.93; only rustd3-iced's apps and the
dev crates build them.

**Packages** (`scripts/check.sh release`, the dry run's `.crate` files):

| Crate | Files | Packaged | Compressed |
| --- | --- | --- | --- |
| rustd3-fonts | 14 | 2.7 MiB | 1.5 MiB (1,611,321 B) |
| rustd3-derive | 8 | 29.5 KiB | 10.1 KiB |
| rustd3-core | 200 | 756.5 KiB | 197.8 KiB |
| rustd3-figure | 184 | 1.0 MiB | 287.9 KiB |
| rustd3 | 9 | 112.5 KiB | 34.4 KiB |

Each list is `src/**/*.rs` and exactly the include table's other files
plus `.cargo_vcs_info.json`, `Cargo.lock`, `Cargo.toml` and
`Cargo.toml.orig`; none under `.claude/`, `dev/` or `target/`. The
published manifests of core, figure and fonts name no `rustd3-testkit`.
Each packaged `Cargo.lock` holds 0 `git+` sources.

**The dry run** (`cargo publish --dry-run --workspace --exclude
rustd3-iced` inside `scripts/check.sh release`, on the clean tree at
8ae8622, 4 min 2 s; again at 7a03794, 16 s with the verify builds
cached): packages derive, core, fonts, figure, rustd3; verifies them in
that order (figure 3 min 31 s the first time, building typst's graph
again under the package's own dev profile);
"Uploading" and "aborting upload due to dry run" for rustd3-derive,
rustd3-fonts, rustd3-core, rustd3-figure, rustd3. Its only warnings are
the four `ignoring test main` notes (core, fonts, figure, rustd3); with
`documentation` set, cargo does not warn about the missing `repository`.

**Viewed** (scratch, not committed): the README's figure as PNG at 2×
(the damped wave, its dots, the typst math label); the canvas example
through `iced_test` on tiny-skia in the light and dark themes (the wave
in the palette's primary color, the axis and its labels in the
background's text color); the `Plot` example on tiny-skia at 2×.

## D40.1 — Include entries without a leading slash match files at any depth

**Date:** 2026-10-28 · **Context:** Phase 40, step 1, 71442c3 · **Status:** decided

### Issue

rustd3.md, Phase 40, "Include lists and licences per package", gives the
facade `include = ["src/**/*.rs", "Cargo.toml", "README.md",
"CHANGELOG.md", "LICENSE-MIT", "LICENSE-APACHE"]`, and exit criterion 2
asks that `cargo package --list -p rustd3` print exactly those files and
no path under `.claude/` or `dev/`. With the list as written, it printed
`.claude/map/rustd3/proto/README.md` and `dev/fixtures/README.md` too.

### Investigation

cargo reads `include` as gitignore patterns: an entry without a slash
(other than a trailing one) matches a file of that name in any directory
below the package root. The facade's root is the repository's, so
`README.md` matches every README that is not inside a nested package.
The four other packages have no nested file of a listed name today, but
`fonts/LICENSE` would, for instance, match a `LICENSE` added under
`fonts/tests/`.

### Options

| | **A — anchor every plain file name** | B — anchor only the facade's | C — list as written, exclude the extras |
| --- | --- | --- | --- |
| Approach | `"/README.md"`, `"/Cargo.toml"`, `"/LICENSE-MIT"`, … in all five lists; `src/**/*.rs` stays (it holds a slash, so it is anchored already) | The facade's entries only | Add `exclude` entries for `.claude/` and `dev/` |
| Correctness | Each entry matches one file | The other lists stay open to a future nested match | Does not work: cargo ignores `exclude` when `include` is set |
| Fit | The table's files, one to one | Two styles | Not possible as stated |

### Decision

A (architect, 2026-10-28), as built in 71442c3: every plain file name in
the five `include` lists is anchored with a leading slash.

### Recommendation

A, as built in 71442c3.

### Why

The table lists root files; anchoring states exactly that, and the
release script's path check stays a backstop rather than the only guard.

## D40.2 — Details of Phase 40 as built

**Date:** 2026-10-28 · **Context:** Phase 40, steps 1–5 · **Status:** decided

### Issue

Details the section leaves open, or statements that do not hold against
the built crates:

1. **The README figure doctest's `cfg`.** The section wraps README blocks
   that use `rustd3::figure` in `#[cfg(feature = "figure")]`;
   `Figure::save_svg` and `save_png` also need `fonts`.
2. **The iced blocks compile.** The section makes them `text` blocks; the
   architect asks for code that compiles.
3. **The repository URL** in the README's git lines is not known yet.
4. **figure's `LICENSE-d3`**: whether any figure code is ported from d3.
5. **The derive crate's runnable example.** A proc-macro crate's
   doctests cannot use rustd3-core without a dev-dependency on it.
6. **The facade's tests.** The section's spot checks of the facade's
   paths need a place to be asserted.
7. **When the `readme` keys come.**
8. **`check.sh quick`'s package map** for the root `tests/` and
   `README.md`.
9. **The changelog link** from the README, which is also the docs.rs
   page.
10. **Feature costs.** Part I's table says `raster` adds 7 crates.
11. **`fmt!` and grouping.** `fmt!` formats in `Locale::NEUTRAL`, which
    groups only from five integer digits with U+202F.
12. **The changelog's Later list.**
13. **The canvas example's `view`.** iced at 3cbdacc has no
    `From<Canvas> for Element`.
14. **Where the dry run leaves its packages.** The section's script reads
    `target/package/<crate>-0.1.0.crate`.
15. **Keywords of fonts and derive**: the section gives none.
16. **RELEASING.md** beyond the section's checklist.

### Options

| | **A — as built** |
| --- | --- |
| (1) | `# #[cfg(all(feature = "figure", feature = "fonts"))]` and its negation; the PNG line under `png` as the section says |
| (2) | The README's two `text` blocks are whole `main.rs` files, compiled as rustd3-examples' examples `readme_canvas` and `readme_plot` with `rustd3_core` and `rustd3_figure` for the facade's `rustd3` and `rustd3::figure` (the dev crate cannot depend on the facade without a manifest change); `tests/main/readme.rs` asserts the README's blocks equal the examples' code with the paths mapped back. Both were rendered through `iced_test` and viewed |
| (3) | The section's `https://…/rustd3`; RELEASING.md item 2 replaces it with the URL when `repository` is set |
| (4) | Yes: `path::Bez`'s arc follows d3-path's (the 1e-6 join to the current point, the sweep normalised through `da % tau + tau`, the full-circle cutoff), so figure ships `LICENSE-d3`, a link to core's file, whose first line names rustd3-core as the porter |
| (5) | The crate doc's example derives against a stand-in `plot::interpolate` module through `#[interpolate(crate = "plot")]`, and points to core's documented examples with the real trait |
| (6) | `tests/main` in the facade (`[[test]] main`, outside `include`, hence the dry run's allowed note): the scale spot row, `rustd3::axis::bottom` as core's `Axis<'_, Linear>` and `rustd3::figure::axis::bottom` as figure's, `fmt!`, `jiff`, the time items, the derive with `crate = "::rustd3"` and no trait import, a figure drawn with the bundled fonts, a PDF and a GIF written |
| (7) | Step 3, with the README files; step 1's `include` lists name them already |
| (8) | `tests/*` maps to rustd3; `README.md` to rustd3 and rustd3-examples |
| (9) | `https://docs.rs/crate/rustd3/latest/source/CHANGELOG.md`: a relative link breaks on docs.rs, and the repository is not public yet |
| (10) | The README states what `cargo tree` counts (Measurements): `raster` +10, `png`/`gif` nothing beyond it, `figure` about 270 |
| (11) | The README shows a locale's format (`Locale::EN_US.format("$,.2f")`, "$1,234.50") and `fmt!(".1%")` ("12.3%") |
| (12) | The Later section's items, plus building the `Plot` widget's drawing list off the UI thread, which D22.1 and Phase 31's measurements recorded for 0.2 |
| (13) | `view` returns `impl Widget<()>`, as the use cases' views do |
| (14) | cargo 1.97.0 leaves the dry run's `.crate` files in `target/package/tmp-crate/`; the script reads the crate from there or from `target/package/`, fails when neither holds it or its lockfile, and names each lockfile it read (7a03794 fixed a first version that passed on a missing file). Exit criterion 4's command, run against `tmp-crate/`, prints 0 for all five |
| (15) | None; categories as the section gives them |
| (16) | Item 1 adds this session's check and that `cargo info` runs outside the repository; item 4 names the dry run's expected tail and warnings; item 9 adds updating the README's iced sections; the MSRV section is the section's text with the floors the lockfile shows |

### Decision

A for all sixteen (architect, 2026-10-28), as built.

### Recommendation

A for all sixteen.

### Followups

- When the user gives the repository URL: `repository` in
  `[workspace.package]` and the README's git lines (RELEASING.md item 2).
- Phase 31's rerun, when it returns, may change the README's performance
  paragraph only if its outcome is "GPU data layer recommended"; the
  paragraph states no numbers.
