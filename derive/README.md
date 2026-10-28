# rustd3-derive

`#[derive(Interpolate)]` for [rustd3](https://crates.io/crates/rustd3):
interpolates a struct field by field, so a mark with several animated
attributes transitions as one value. Enable it through the `derive`
feature of `rustd3` or `rustd3-core`, which re-export it beside the
trait as `rustd3::interpolate::Interpolate`.

Documentation: [docs.rs](https://docs.rs/rustd3-derive).
