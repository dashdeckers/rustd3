//! Follow-up 7: what sharing column boundaries saves.
use std::hint::black_box;
use std::time::Instant;
fn columns(x: &[f64], x0: f64, x1: f64, cols: usize) -> Vec<usize> {
    (0..=cols).map(|c| { let v = x0 + (x1 - x0) * c as f64 / cols as f64; x.partition_point(|t| *t < v) }).collect()
}
fn main() {
    for n in [1_000_000usize, 10_000_000] {
        let x: Vec<f64> = (0..n).map(|i| i as f64 * 1e-4 + ((i * 7919) % 13) as f64 * 1e-6).collect();
        let (a, b) = (x[0], x[n - 1]);
        let t = Instant::now();
        for _ in 0..1000 { black_box(columns(black_box(&x), a, b, 1000)); }
        let us = t.elapsed().as_secs_f64() * 1e6 / 1000.0;
        println!("{n} samples: 1000 column boundaries by binary search {us:.1} µs");
    }
}
