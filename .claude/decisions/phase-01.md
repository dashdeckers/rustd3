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
