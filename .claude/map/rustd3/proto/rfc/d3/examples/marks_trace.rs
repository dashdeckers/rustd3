//! Use case 2's core: keyed enter/update/exit transitions, sampled at fixed instants.

use d3::anim::{Marks, Timing};
use std::time::{Duration, Instant};

fn main() {
    let t0 = Instant::now();
    let ms = |n: u64| t0 + Duration::from_millis(n);
    let timing = Timing { duration: Duration::from_millis(100), stagger: Duration::from_millis(20), ..Timing::default() };
    let enter = |w: &f64| 0.0 * w; // new bars grow from zero width
    let exit = |_: &f64| 0.0; // leaving bars shrink to zero

    let mut bars: Marks<&str, f64> = Marks::default();
    bars.update([("a", 10.0), ("b", 20.0)], ms(0), timing, enter, exit);
    let show = |bars: &Marks<&str, f64>, at: u64| {
        let v: Vec<String> = bars.values(ms(at)).map(|(k, w, p)| format!("{k}={w:.1}{}", if p == d3::anim::Phase::Exiting { "(exit)" } else { "" })).collect();
        println!("t={at:>3}ms  {}", v.join("  "));
    };
    for at in [0, 50, 120] { show(&bars, at); }

    // New data mid-flight at 60 ms: "a" leaves, "b" grows, "c" enters.
    bars.update([("b", 40.0), ("c", 30.0)], ms(60), timing, enter, exit);
    for at in [60, 110, 180] { show(&bars, at); }
    bars.prune(ms(180));
    show(&bars, 180);
    println!("animating at 180 ms: {}", bars.is_animating(ms(180)));
}
