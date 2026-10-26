//! Does keeping vello_cpu's Resources between renders
//! (its glyph caches) make repeated frames cheaper?
use dlist::shape::Shaper;
use dlist::{vcpu, wheels};
use figmodel::plot::draw::Command;
use std::time::Instant;
fn main() {
    let samples = wheels::simulate();
    let mut l = wheels::figure(&samples, false).draw([960.0, 760.0]);
    Shaper::new().shape(&mut l);
    let mut text = l.clone();
    text.commands.retain(|c| !matches!(c, Command::Markers { .. } | Command::Fill { .. } | Command::Stroke { .. }));
    for (name, list) in [("whole figure", &l), ("text only", &text)] {
        for s in [1.0, 2.0] {
            let fresh = (0..5).map(|_| { let t = Instant::now(); std::hint::black_box(vcpu::render(list, s)); t.elapsed().as_secs_f64() * 1e3 }).fold(f64::MAX, f64::min);
            let mut res = vello_cpu::Resources::new();
            let _ = vcpu::render_res(list, s, 0, &mut res);
            let kept = (0..5).map(|_| { let t = Instant::now(); std::hint::black_box(vcpu::render_res(list, s, 0, &mut res)); t.elapsed().as_secs_f64() * 1e3 }).fold(f64::MAX, f64::min);
            let a = vcpu::render(list, s);
            let b = vcpu::render_res(list, s, 0, &mut res);
            let same = a.data_as_u8_slice() == b.data_as_u8_slice();
            println!("{name} at {s}x: fresh resources {fresh:.2} ms, kept {kept:.2} ms, identical pixels {same}");
        }
    }
}
