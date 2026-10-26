// A malformed format specifier is a compile error.
fn main() {
    let _f = d3::fmt!("10..f");
}
