//! NaN and null break the line: M4 per NaN-free run,
//! from a tree whose nodes ignore NaN.

use std::time::Instant;
use tiny_skia as ts;

const B: usize = 64;
const NONE: u32 = u32::MAX;

struct Series {
    x: Vec<f64>,
    y: Vec<f64>,
    /// NaN-free runs [a, b) of samples.
    runs: Vec<(usize, usize)>,
    lo: Vec<u32>,
    hi: Vec<u32>,
    size: usize,
}

fn argminmax(y: &[f64], a: usize, b: usize) -> (u32, u32) {
    let (mut lo, mut hi) = (NONE, NONE);
    for i in a..b {
        if y[i].is_nan() {
            continue;
        }
        if lo == NONE || y[i] < y[lo as usize] {
            lo = i as u32;
        }
        if hi == NONE || y[i] > y[hi as usize] {
            hi = i as u32;
        }
    }
    (lo, hi)
}

fn pick(y: &[f64], a: u32, b: u32, min: bool) -> u32 {
    if a == NONE {
        return b;
    }
    if b == NONE {
        return a;
    }
    let (va, vb) = (y[a as usize], y[b as usize]);
    let better = if min { vb < va } else { vb > va };
    if better || (vb == va && b < a) { b } else { a }
}

impl Series {
    /// `None` and NaN both become gaps.
    fn new(x: Vec<f64>, y: impl IntoIterator<Item = Option<f64>>) -> Self {
        let y: Vec<f64> = y
            .into_iter()
            .map(|v| v.unwrap_or(f64::NAN))
            .collect();
        let mut runs = Vec::new();
        let mut start = None;
        for (i, v) in y.iter().enumerate() {
            match (v.is_nan(), start) {
                (false, None) => start = Some(i),
                (true, Some(s)) => {
                    runs.push((s, i));
                    start = None;
                }
                _ => {}
            }
        }
        if let Some(s) = start {
            runs.push((s, y.len()));
        }
        let blocks = y.len().div_ceil(B);
        let size = blocks.next_power_of_two();
        let (mut lo, mut hi) = (vec![NONE; 2 * size], vec![NONE; 2 * size]);
        for k in 0..blocks {
            let (l, h) = argminmax(&y, k * B, ((k + 1) * B).min(y.len()));
            lo[size + k] = l;
            hi[size + k] = h;
        }
        for i in (1..size).rev() {
            lo[i] = pick(&y, lo[2 * i], lo[2 * i + 1], true);
            hi[i] = pick(&y, hi[2 * i], hi[2 * i + 1], false);
        }
        Series { x, y, runs, lo, hi, size }
    }

    fn query(&self, a: usize, b: usize) -> (usize, usize) {
        let y = &self.y;
        let (ba, bb) = (a.div_ceil(B), b / B);
        if ba >= bb {
            let (l, h) = argminmax(y, a, b);
            return (l as usize, h as usize);
        }
        let (mut lo, mut hi) = argminmax(y, a, ba * B);
        let (e0, e1) = argminmax(y, bb * B, b);
        lo = pick(y, lo, e0, true);
        hi = pick(y, hi, e1, false);
        let (mut i, mut j) = (ba + self.size, bb + self.size);
        while i < j {
            if i & 1 == 1 {
                lo = pick(y, lo, self.lo[i], true);
                hi = pick(y, hi, self.hi[i], false);
                i += 1;
            }
            if j & 1 == 1 {
                j -= 1;
                lo = pick(y, lo, self.lo[j], true);
                hi = pick(y, hi, self.hi[j], false);
            }
            i >>= 1;
            j >>= 1;
        }
        (lo as usize, hi as usize)
    }

    /// One polyline per visible run; `raw` scans instead
    /// of using the tree, for the exactness check.
    fn m4(&self, v: [f64; 2], cols: usize, raw: bool) -> Vec<Vec<usize>> {
        let x = &self.x;
        let bounds: Vec<usize> = (0..=cols)
            .map(|c| {
                let t = v[0] + (v[1] - v[0]) * c as f64 / cols as f64;
                x.partition_point(|s| *s < t)
            })
            .collect();
        let (va, vb) = (bounds[0], bounds[cols]);
        let mut out = Vec::new();
        let first = self.runs.partition_point(|r| r.1 <= va);
        for &(ra, rb) in &self.runs[first..] {
            if ra >= vb {
                break;
            }
            let mut line = Vec::new();
            let c0 = bounds.partition_point(|b| *b <= ra.max(va)) - 1;
            for c in c0..cols {
                let a = bounds[c].max(ra);
                let b = bounds[c + 1].min(rb);
                if a >= b {
                    if bounds[c] >= rb {
                        break;
                    }
                    continue;
                }
                let (l, h) = if raw {
                    let (l, h) = argminmax(&self.y, a, b);
                    (l as usize, h as usize)
                } else {
                    self.query(a, b)
                };
                let mut idx = [a, l, h, b - 1];
                idx.sort_unstable();
                for i in idx {
                    if line.last() != Some(&i) {
                        line.push(i);
                    }
                }
            }
            out.push(line);
        }
        out
    }
}

fn render(s: &Series, lines: &[Vec<usize>], v: [f64; 2], w: u32) -> ts::Pixmap {
    let mut px = ts::Pixmap::new(w, 160).unwrap();
    px.fill(ts::Color::WHITE);
    let mut pb = ts::PathBuilder::new();
    for line in lines {
        for (k, &i) in line.iter().enumerate() {
            let sx = ((s.x[i] - v[0]) / (v[1] - v[0]) * w as f64) as f32;
            let sy = (80.0 - s.y[i] * 40.0) as f32;
            if k == 0 {
                pb.move_to(sx, sy);
                // A lone sample still shows, as a dot-sized dash.
                if line.len() == 1 {
                    pb.line_to(sx + 0.01, sy);
                }
            } else {
                pb.line_to(sx, sy);
            }
        }
    }
    let mut paint = ts::Paint::default();
    paint.set_color_rgba8(30, 60, 140, 255);
    let mut st = ts::Stroke::default();
    st.width = 1.0;
    st.line_cap = ts::LineCap::Round;
    if let Some(p) = pb.finish() {
        px.stroke_path(&p, &paint, &st, ts::Transform::identity(), None);
    }
    px
}

fn main() {
    let n = 1_000_000;
    let mut seed = 9u64;
    let mut u = move || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 11) as f64 / (1u64 << 53) as f64
    };
    let x: Vec<f64> = (0..n).map(|i| i as f64 / 10_000.0).collect();
    let mut y: Vec<Option<f64>> = x
        .iter()
        .map(|t| Some(0.6 * (std::f64::consts::TAU * 0.1 * t).sin() + 0.3 * (u() - 0.5)))
        .collect();
    // Gaps of 1, 10, 100, 1,000, 10,000 and 50,000
    // samples, four of each, plus the first and last.
    let mut gaps = 0;
    for len in [1usize, 10, 100, 1_000, 10_000, 50_000] {
        for _ in 0..4 {
            let s = (u() * (n - len) as f64) as usize;
            for v in &mut y[s..s + len] {
                *v = None;
            }
            gaps += 1;
        }
    }
    y[0] = None;
    y[n - 1] = Some(f64::NAN);
    let t = Instant::now();
    let s = Series::new(x, y);
    println!(
        "build with run detection: {:.1} ms, {} runs ({gaps} gaps inserted, some overlap)",
        t.elapsed().as_secs_f64() * 1e3,
        s.runs.len()
    );
    let full = [s.x[0], s.x[n - 1] + 1e-4];
    let mut views = vec![full];
    for k in 0..40 {
        let a = s.x[k * n / 41];
        views.push([a, a + 0.05 + 2.0 * (k as f64)]);
    }
    let mut same = 0;
    let mut crossing = 0;
    let mut nan_out = 0;
    for v in &views {
        let a = s.m4(*v, 1000, false);
        let b = s.m4(*v, 1000, true);
        same += usize::from(a == b);
        for line in &a {
            nan_out += line.iter().filter(|i| s.y[**i].is_nan()).count();
            for w in line.windows(2) {
                // A segment crosses a gap if any sample
                // between its ends is NaN.
                if s.y[w[0]..=w[1]].iter().any(|v| v.is_nan()) {
                    crossing += 1;
                }
            }
        }
    }
    println!(
        "{} views: tree equals raw scan in {same}, segments across a gap {crossing}, NaN points emitted {nan_out}",
        views.len()
    );
    let t = Instant::now();
    let reps = 50;
    let mut lines = Vec::new();
    for _ in 0..reps {
        lines = s.m4(full, 1000, false);
    }
    println!(
        "full view: {} polylines, {} points, {:.3} ms",
        lines.len(),
        lines.iter().map(Vec::len).sum::<usize>(),
        t.elapsed().as_secs_f64() * 1e3 / reps as f64
    );
    std::fs::create_dir_all("out").unwrap();
    render(&s, &lines, full, 1000).save_png("out/gaps.png").unwrap();
    // Zoom onto one 1,000-sample gap.
    let r = s.runs.iter().find(|r| {
        s.runs.iter().any(|q| q.0 > r.1 && q.0 - r.1 >= 900 && q.0 - r.1 <= 1100)
    });
    if let Some(&(_, end)) = r {
        let v = [s.x[end - 3000], s.x[(end + 4000).min(n - 1)]];
        let lines = s.m4(v, 1000, false);
        println!(
            "zoom on a 1,000-sample gap: {} polylines, {} points",
            lines.len(),
            lines.iter().map(Vec::len).sum::<usize>()
        );
        render(&s, &lines, v, 1000).save_png("out/gap_zoom.png").unwrap();
    }
}
