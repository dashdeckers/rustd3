// invert exists only where the range is numeric.
use d3::{color::Rgb, scale};
fn main() {
    let heat = scale::linear().range([Rgb::hex(0xffffff), Rgb::hex(0x4682b4)]);
    let _ = heat.invert(0.5);
}
