//! Backends for the figure's drawing list, measured on
//! the wheel figure (problem #4 in the RFC).

pub mod anyr;
pub mod lower;
pub mod measure;
pub mod outline;
pub mod pdf;
pub mod shape;
pub mod skia;
pub mod svg;
pub mod vcpu;
pub mod wheels;

/// Per-pixel comparison of two straight RGBA buffers.
pub struct Diff {
    pub max: u8,
    pub mean: f64,
    /// Share of pixels with any channel off by > 16.
    pub over16: f64,
}

pub fn diff(a: &[u8], b: &[u8]) -> Diff {
    assert_eq!(a.len(), b.len());
    let (mut max, mut sum, mut over) = (0u8, 0u64, 0usize);
    for (p, q) in a.chunks(4).zip(b.chunks(4)) {
        let d = p
            .iter()
            .zip(q)
            .map(|(x, y)| x.abs_diff(*y))
            .max()
            .unwrap();
        max = max.max(d);
        sum += d as u64;
        if d > 16 {
            over += 1;
        }
    }
    let n = a.len() / 4;
    Diff {
        max,
        mean: sum as f64 / n as f64,
        over16: over as f64 / n as f64,
    }
}
