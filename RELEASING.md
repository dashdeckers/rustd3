# Releasing rustd3

How to publish a release of the five crates on crates.io: `rustd3-fonts`,
`rustd3-derive`, `rustd3-core`, `rustd3-figure` and the facade `rustd3`.
`rustd3-iced` stays on git until iced 0.15 is on crates.io, and the dev
crates under `dev/` are never published. Publishing is permanent: a
version can be yanked, never replaced or deleted. This file is not
packaged.

## Checklist for 0.1.0

1. **The names are free.** From a directory outside this repository
   (inside it, `cargo info` describes the local facade):
   `cargo search rustd3` lists no crate named `rustd3*`, and
   `cargo info rustd3` reports that it could not find `rustd3` in the
   registry. Both held on 2026-10-17 and on 2026-10-28.
2. **The repository.** Choose the public URL; set `repository` (and
   `homepage`, if any) under `[workspace.package]` in the root
   `Cargo.toml`; replace the placeholder `https://…/rustd3` in the git
   lines of `README.md`'s iced sections with it; make the repository
   public. Commit.
3. **The account.** A crates.io account with a verified email address,
   and `cargo login` with an API token whose scopes allow publishing new
   crates.
4. **The checks**, on a clean clone of the release commit:

   ```sh
   scripts/check.sh full       # about 20 minutes warm, longer cold
   scripts/check.sh release    # docs, doctests, package lists, dry run
   ```

   `release` writes `target/package-<crate>.txt`, the files each crate
   ships; read the five lists once. Its dry run must end with
   `Uploading` and `aborting upload due to dry run` for each of the five
   crates, then one line per crate saying its packaged `Cargo.lock` has
   no git source, then `release: ready`. The only other warnings it may
   print are a missing `repository` (until step 2) and, for core, fonts,
   figure and the facade, that the `main` integration test is not
   included in the package.
5. **Optional: a lower minimum Rust version** (see below).
6. **Tag** the release commit and push the tag:

   ```sh
   git tag v0.1.0 && git push origin v0.1.0
   ```

7. **Publish**, from the tagged commit:

   ```sh
   cargo publish --workspace --exclude rustd3-iced
   ```

   cargo publishes `rustd3-fonts` and `rustd3-derive`, then
   `rustd3-core`, `rustd3-figure` and `rustd3`, waiting for each to
   appear in the index before the next.
8. **Check the result.** docs.rs builds all five crates with all
   features; read their front pages. In a scratch project, run
   `cargo add rustd3 --features figure,png` and build the README's
   figure example.
9. **rustd3-iced stays on git.** When iced 0.15 is on crates.io: switch
   rustd3-iced's iced dependencies (and the dev crates') from the git
   revision to the version, rerun the iced tests on both backends
   (`ICED_TEST_BACKEND=tiny-skia` and `wgpu`), set `publish = true` on
   rustd3-iced, add the facade feature `iced`, update the README's iced
   sections, and release 0.1.1.
10. **Optional:** `cargo owner --add <user>` for co-maintainers.

## Minimum Rust version

Every crate declares `rust-version = "1.97"`, the toolchain the release
is built and tested with. The dependencies' own floors are lower:
typst 0.15.1, typst-layout 0.15.1 and krilla 0.8.2 need 1.92; vello_cpu
0.3.0 1.89; kurbo 0.13.1 and peniko 0.6.1 1.85; thiserror 2 1.77; png
0.18 1.73; syn 2 1.71; jiff 0.2.38 1.70; edition 2024 itself 1.85. So
`rustd3-figure`, and the facade with `figure`, cannot go below 1.92, and
`rustd3-core` alone cannot go below 1.85. The lockfile may hold
transitive crates with higher floors; list the highest with:

```sh
cargo metadata --format-version 1 | jq -r '.packages[] | select(.rust_version != null) | "\(.rust_version) \(.name) \(.version)"' | sort -V | tail
```

On 2026-10-28 the highest floor among the published crates' dependencies
was 1.92 (typst, krilla and hayro); iced at the pinned revision declares
1.93, which binds apps that use rustd3-iced, not the published crates.

To lower it: install the toolchain (`rustup toolchain install 1.92.0`),
then

```sh
cargo +1.92.0 check -p rustd3-figure --all-features
cargo +1.92.0 test -p rustd3-core -p rustd3-figure --all-features
```

and set `rust-version` per package to the version these pass on.
