// d3's ordinal scale grows its domain when called; in Rust that is a mutable borrow.
use d3::{color::scheme, scale};
fn main() {
    let mut color = scale::ordinal(Vec::<&str>::new(), scheme::CATEGORY10);
    println!("{} {}", color.apply_or_insert("disk"), color.apply_or_insert("net"));
}
