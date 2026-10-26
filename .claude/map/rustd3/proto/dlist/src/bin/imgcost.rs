//! Review: option 4's cost, the whole figure as one
//! vello_cpu image, by output size and thread count.
use dlist::shape::Shaper;
use dlist::{vcpu, wheels};
use std::time::Instant;
fn main() {
    let samples = wheels::simulate();
    let mut l = wheels::figure(&samples, false).draw([960.0, 760.0]);
    Shaper::new().shape(&mut l);
    let t = Instant::now();
    let fig = wheels::figure(&samples, false);
    let mut l2 = fig.draw([960.0, 760.0]);
    Shaper::new().shape(&mut l2);
    println!("draw list + shape (cold shaper): {:.1} ms", t.elapsed().as_secs_f64() * 1e3);
    for s in [1.0, 2.0, 4.0] {
        for th in [0u16, 2] {
            let best = (0..3)
                .map(|_| {
                    let t = Instant::now();
                    std::hint::black_box(vcpu::render_threads(&l, s, th));
                    t.elapsed().as_secs_f64() * 1e3
                })
                .fold(f64::MAX, f64::min);
            let mpx = 960.0 * 760.0 * s * s / 1e6;
            println!("{s}x ({mpx:.1} Mpx, {:.1} MB RGBA), threads {th}: {best:.1} ms", mpx * 4.0);
        }
    }
}
