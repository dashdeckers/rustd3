# Phase 1 — Workspace, tooling and fonts

## Measurements

2026-10-26, `nproc` = 2 (7 GB RAM), Rust 1.97.0, debug profile with
`debug = "line-tables-only"`, at commit ddfdcb4. Reference values
(rustd3.md, Phase 1): about 6 minutes for a cold build of the whole
dependency set, about 1.8 GB in `target`, 0.6 s for a no-op rebuild.
Recorded only; there is no threshold.

| Run | Wall | `target` after |
| --- | --- | --- |
| `cargo check --workspace --all-features`, cold (crates already downloaded; step 1) | 100 s | — |
| `cargo clean`, then `scripts/check.sh quick` with all ten packages (cold; what a manifest change asks for) | 377 s | 3.3 GB |
| `scripts/check.sh quick`, no changes (fmt and clippy; the last commit touched no package) | 0.5 s | — |
| `scripts/check.sh quick` with all ten packages, warm | 2.4 s | — |
| `scripts/check.sh full`, after the cold `quick` | 67 s | 3.8 GB |
| `cargo check --workspace --all-features`, no-op | 0.3 s | — |

`Cargo.lock` holds 327 packages (`grep -c '^name = ' Cargo.lock`), none
of them iced. `target` is larger than the reference's 1.8 GB because
`quick` builds clippy's metadata for the whole workspace and then test
binaries per package, and `full` adds the no-default-feature builds of
each package.

## D1.1 — d3-ease is BSD-3-Clause: what `core/LICENSE-d3` carries

**Date:** 2026-10-26 · **Context:** Phase 1, step 1 (`core/LICENSE-d3`); the skeleton was committed without it (0728bd2) · **Status:** open

### Issue

rustd3.md, Phase 1, lists `proto/d3-src/*/LICENSE` as ISC ("Builds on")
and specifies `core/LICENSE-d3` as "the ISC text once, preceded by the
copyright line of each module core ports", d3-ease 2010-2021 among them
("Licences"). d3-ease is not ISC. `proto/d3-src/d3-ease/LICENSE`
(d3-ease 3.0.1, identical in `proto/d3-tests/d3-ease/`) is the BSD
3-Clause licence over two notices, `Copyright 2010-2021 Mike Bostock`
and `Copyright 2001 Robert Penner` / `All rights reserved.`, and its
`package.json` declares `"license": "BSD-3-Clause"`.

The ISC text with d3-ease's first copyright line would present d3-ease,
which Phase 3 ports (`core/src/ease.rs`), as ISC, and drop Penner's
notice and the conditions and disclaimer that the BSD licence's first
clause requires redistributions of source to keep.

### Investigation

- The other sixteen modules core ports (d3-array, -axis, -brush, -color,
  -dispatch, -drag, -format, -interpolate, -path, -scale, -shape, -time,
  -time-format, -timer, -zoom, internmap) have byte-identical ISC bodies
  (`tail -n +2 LICENSE | md5sum` = `7181fab1…` for all sixteen) and the
  copyright lines rustd3.md lists.
- d3-scale-chromatic 3.1.0's LICENSE is ISC plus the ColorBrewer notice
  (Apache-2.0); rustd3.md has Phase 23 append that notice to
  `core/LICENSE-d3`, so the file already holds more than the ISC text.
- Phase 40's root README credits read "d3, ISC, Mike Bostock".

### Options

| | A — as written | **B — ISC block, then d3-ease's own notice** | C — every module's LICENSE verbatim |
| --- | --- | --- | --- |
| Approach | The ISC text once, preceded by seventeen copyright lines, d3-ease's Bostock line among them | The sixteen ISC modules' copyright lines and the ISC text once; then a line naming d3-ease 3.0.1 and its LICENSE verbatim (both notices, three conditions, disclaimer). Phase 23 appends ColorBrewer after it | Seventeen sections, each a module's LICENSE verbatim |
| Size | about 30 lines | about 60 lines | about 280 lines, the ISC body sixteen times |
| Correctness | Drops Penner's notice and the BSD terms | Complete | Complete |
| Architectural fit | Contradicts the source | The shape Phase 23 already uses for ColorBrewer; "the ISC text once" still holds for the ISC modules | Departs from "the ISC text once" |
| Elsewhere | — | Phase 40's credits add "d3-ease: BSD-3-Clause (Robert Penner's easing equations)" | As B |

### Decision

Recommended: B, with rustd3-core's `license = "MIT OR Apache-2.0"`
unchanged: the upstream notices travel in `LICENSE-d3`, as the ISC and
ColorBrewer ones do.

### Why

B is the smallest file that keeps every notice d3-ease's licence
requires, and it follows the structure rustd3.md already uses for the
one other non-ISC notice.

### Followups

- On the answer: write `core/LICENSE-d3` in its own commit, then
  `scripts/check.sh quick` and `full`.
- Phase 40: the README credit line names d3-ease's licence.

## D1.2 — The font licence texts in `proto/fonts/` are incomplete

**Date:** 2026-10-26 · **Context:** Phase 1, step 3 (`fonts/LICENSE*`); the fonts were committed without them (c16e92b) · **Status:** open

### Issue

rustd3.md, Phase 1, says `proto/fonts/LICENSE-DejaVu` holds "the
Bitstream Vera and Arev terms" and `LICENSE-NewCMMath.txt` "the GUST
Font License 1.0", that both are copied into `fonts/` byte for byte, and
that `fonts/LICENSE` is a summary paragraph followed by both texts.
Neither file is what the spec says:

- `LICENSE-DejaVu` (78 lines) is Debian's copyright file for
  fonts-dejavu-core: byte-identical to
  `/usr/share/doc/fonts-dejavu-core/copyright` on this machine, whose
  `DejaVuSans.ttf` and `DejaVuSans-Bold.ttf` are byte-identical to ours.
  It has the Bitstream Vera terms and "DejaVu changes are in public
  domain", **no Arev notice**, and a `Files: debian/*` section under
  GPL-2+ that covers Debian's packaging, not the fonts. DejaVu's own
  terms say "Glyphs imported from Arev fonts are (c) Tavmjong Bah", and
  the Arev licence requires its notice "be included in all copies of one
  or more of the Font Software typefaces".
- `LICENSE-NewCMMath.txt` (100 lines) is exactly lines 101–200 of
  typst-assets 0.15.1's `NOTICE` (our `NewCMMath-Regular.otf` is
  byte-identical to typst-assets 0.15.1's `files/fonts/`). Its lines
  1–30 are the GUST Font License 1.0, complete, without the section's
  heading (`NOTICE` line 99); lines 31–100 run on into typst-assets'
  DejaVu section and **stop mid-sentence** inside the Arev notice
  ("…and to permit").

Byte-for-byte copies would ship a truncated licence and a DejaVu notice
without Arev.

### Investigation

- typst-assets 0.15.1 is in the lock (a typst dependency);
  `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/typst-assets-0.15.1/NOTICE`
  (sha256 `80ac46fb…`) holds both texts whole: lines 103–130, the GUST
  Font License 1.0; lines 139–236, DejaVu's notice ("Fonts are (c)
  Bitstream (see below). DejaVu changes are in public domain. Glyphs
  imported from Arev fonts are (c) Tavmjong Bah (see below)"), the
  Bitstream Vera licence and the Arev licence. Lines 238–325 (TeX Gyre
  DJV Math, AMSFonts) cover DejaVu Math TeX Gyre, which rustd3 does not
  ship.
- DejaVu's upstream LICENSE is on GitHub, outside the allowed network.

### Options

| | A — byte for byte, as written | **B — whole sections from typst-assets 0.15.1** | C — patch the proto texts |
| --- | --- | --- | --- |
| Approach | Copy both proto files; `fonts/LICENSE` = summary + both | `fonts/LICENSE-DejaVu` = `NOTICE` lines 139–236; `fonts/LICENSE-NewCMMath.txt` = `NOTICE` lines 103–130; `fonts/LICENSE` = summary + both. `proto/` stays as it is; the sha256 comparison applies to the font files | Keep Debian's file and append the Arev notice (`NOTICE` 190–236); cut `LICENSE-NewCMMath.txt` after its line 30 |
| Correctness | Truncated text; no Arev notice | Complete for the three fonts shipped | Complete, but carries Debian's GPL-2+ packaging section into the crate's licence |
| Provenance | Debian plus a truncated excerpt | One pinned source, a crate typst depends on; reproducible with `sed -n` | Two sources, edited by hand |

### Decision

Recommended: B.

### Why

It is the only option whose texts are both whole and from one pinned
source, and the extraction is a reproducible `sed -n` over a file the
lock already pins.

### Followups

- On the answer: add `fonts/LICENSE`, `LICENSE-DejaVu` and
  `LICENSE-NewCMMath.txt` in one `chore(fonts)` commit, then
  `scripts/check.sh quick` and `full`.
- An SPDX expression for rustd3-fonts stays in Later (0.2).
