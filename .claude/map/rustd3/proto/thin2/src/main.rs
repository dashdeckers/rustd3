//! Problem #5 evidence: thinning a long series for a
//! panel `cols` device pixels wide. Fidelity is judged
//! by rasterizing the full series and the thinned one
//! with the same stroke and diffing the pixels.

use std::hint::black_box;
use std::time::Instant;
use tiny_skia as ts;

/// A vibration-like trace: drift, a 7 Hz tone, noise,
/// periodic bursts, and 50 one-sample impacts.
fn signal(n: usize) -> (Vec<f64>, Vec<f64>, Vec<usize>) {
    let mut seed = 42u64;
    let mut u = move || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 11) as f64 / (1u64 << 53) as f64
    };
    let fs = 10_000.0;
    let x: Vec<f64> = (0..n).map(|i| i as f64 / fs).collect();
    let mut y: Vec<f64> = x
        .iter()
        .map(|t| {
            let burst = if (t % 10.0) < 0.2 {
                0.8 * (std::f64::consts::TAU * 120.0 * t).sin()
            } else {
                0.0
            };
            0.3 * (std::f64::consts::TAU * 0.05 * t).sin()
                + 0.4 * (std::f64::consts::TAU * 7.0 * t).sin()
                + 0.25 * (u() - 0.5)
                + burst
        })
        .collect();
    let mut spikes: Vec<usize> =
        (0..50).map(|_| (u() * n as f64) as usize).collect();
    spikes.sort_unstable();
    for (k, &s) in spikes.iter().enumerate() {
        let sign = if k % 2 == 0 { 1.0 } else { -1.0 };
        y[s] = sign * (1.4 + 0.6 * u());
    }
    (x, y, spikes)
}

/// Column boundaries in sample indices for [x0, x1).
fn columns(x: &[f64], x0: f64, x1: f64, cols: usize) -> Vec<usize> {
    (0..=cols)
        .map(|c| {
            let v = x0 + (x1 - x0) * c as f64 / cols as f64;
            x.partition_point(|t| *t < v)
        })
        .collect()
}

fn argminmax(y: &[f64], a: usize, b: usize) -> (usize, usize) {
    let (mut lo, mut hi) = (a, a);
    for i in a..b {
        if y[i] < y[lo] {
            lo = i;
        }
        if y[i] > y[hi] {
            hi = i;
        }
    }
    (lo, hi)
}

fn push_sorted(out: &mut Vec<usize>, mut idx: Vec<usize>) {
    idx.sort_unstable();
    idx.dedup();
    out.extend(idx);
}

fn minmax(x: &[f64], y: &[f64], v: [f64; 2], cols: usize) -> Vec<usize> {
    let b = columns(x, v[0], v[1], cols);
    let mut out = Vec::new();
    for w in b.windows(2) {
        if w[0] < w[1] {
            let (lo, hi) = argminmax(y, w[0], w[1]);
            push_sorted(&mut out, vec![lo, hi]);
        }
    }
    out
}

fn m4(x: &[f64], y: &[f64], v: [f64; 2], cols: usize) -> Vec<usize> {
    let b = columns(x, v[0], v[1], cols);
    let mut out = Vec::new();
    for w in b.windows(2) {
        if w[0] < w[1] {
            let (lo, hi) = argminmax(y, w[0], w[1]);
            push_sorted(&mut out, vec![w[0], lo, hi, w[1] - 1]);
        }
    }
    out
}

/// Exact argmin/argmax over any index range: a segment
/// tree over blocks of B samples, edges scanned raw.
struct Tree {
    size: usize,
    lo: Vec<u32>,
    hi: Vec<u32>,
}

const B: usize = 64;
const NONE: u32 = u32::MAX;

impl Tree {
    fn build(y: &[f64]) -> Self {
        let blocks = y.len().div_ceil(B);
        let size = blocks.next_power_of_two();
        let mut lo = vec![NONE; 2 * size];
        let mut hi = vec![NONE; 2 * size];
        for k in 0..blocks {
            let (a, b) = (k * B, ((k + 1) * B).min(y.len()));
            let (l, h) = argminmax(y, a, b);
            lo[size + k] = l as u32;
            hi[size + k] = h as u32;
        }
        for i in (1..size).rev() {
            lo[i] = pick(y, lo[2 * i], lo[2 * i + 1], true);
            hi[i] = pick(y, hi[2 * i], hi[2 * i + 1], false);
        }
        Tree { size, lo, hi }
    }

    fn bytes(&self) -> usize {
        (self.lo.len() + self.hi.len()) * 4
    }

    fn query(&self, y: &[f64], a: usize, b: usize) -> (usize, usize) {
        let (ba, bb) = (a.div_ceil(B), b / B);
        if ba >= bb {
            return argminmax(y, a, b);
        }
        let (mut l, mut h) = argminmax(y, a, ba * B);
        if ba * B == a {
            (l, h) = (NONE as usize, NONE as usize);
        }
        let mut lo = l as u32;
        let mut hi = h as u32;
        let (e0, e1) = argminmax(y, bb * B, b);
        if bb * B < b {
            lo = pick(y, lo, e0 as u32, true);
            hi = pick(y, hi, e1 as u32, false);
        }
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

    fn m4(&self, x: &[f64], y: &[f64], v: [f64; 2], cols: usize) -> Vec<usize> {
        let b = columns(x, v[0], v[1], cols);
        let mut out = Vec::with_capacity(4 * cols);
        for w in b.windows(2) {
            if w[0] < w[1] {
                let (lo, hi) = self.query(y, w[0], w[1]);
                push_sorted(&mut out, vec![w[0], lo, hi, w[1] - 1]);
            }
        }
        out
    }
}

/// The smaller (or larger) of two indices by value; the
/// earlier index wins ties, as the raw scan does.
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

fn render(
    pts: impl Iterator<Item = (f64, f64)>,
    v: [f64; 2],
    w: u32,
    scale: f32,
) -> ts::Pixmap {
    let h = 240u32;
    let (pw, ph) = ((w as f32 * scale) as u32, (h as f32 * scale) as u32);
    let mut px = ts::Pixmap::new(pw, ph).unwrap();
    px.fill(ts::Color::WHITE);
    let mut pb = ts::PathBuilder::new();
    for (i, (x, y)) in pts.enumerate() {
        let sx = ((x - v[0]) / (v[1] - v[0]) * w as f64) as f32;
        let sy = (h as f64 / 2.0 - y * 50.0) as f32;
        if i == 0 {
            pb.move_to(sx, sy);
        } else {
            pb.line_to(sx, sy);
        }
    }
    let path = pb.finish().unwrap();
    let mut paint = ts::Paint::default();
    paint.set_color_rgba8(30, 60, 140, 255);
    paint.anti_alias = true;
    let mut s = ts::Stroke::default();
    s.width = 1.0;
    px.stroke_path(&path, &paint, &s, ts::Transform::from_scale(scale, scale), None);
    px
}

/// M4's line, plus each column's min..max filled as a
/// 1-px bar widened by half the stroke: dense columns
/// become solid, as the full trace's overlapping
/// segments make them.
fn render_band(
    x: &[f64],
    y: &[f64],
    idx: &[usize],
    v: [f64; 2],
    cols: usize,
    scale: f32,
) -> ts::Pixmap {
    let mut px = render(idx.iter().map(|&i| (x[i], y[i])), v, cols as u32, scale);
    let b = columns(x, v[0], v[1], cols);
    let mut pb = ts::PathBuilder::new();
    let h = 240.0;
    for (c, w) in b.windows(2).enumerate() {
        if w[1] - w[0] < 8 {
            continue;
        }
        let (lo, hi) = argminmax(y, w[0], w[1]);
        let top = (h / 2.0 - y[hi] * 50.0) as f32 - 0.5;
        let bot = (h / 2.0 - y[lo] * 50.0) as f32 + 0.5;
        // Each column's own x extent of samples.
        let xa = ((x[w[0]] - v[0]) / (v[1] - v[0]) * cols as f64) as f32;
        let xb = ((x[w[1] - 1] - v[0]) / (v[1] - v[0]) * cols as f64) as f32;
        let _ = c;
        if let Some(r) = ts::Rect::from_ltrb(xa - 0.5, top, xb + 0.5, bot) {
            pb.push_rect(r);
        }
    }
    if let Some(path) = pb.finish() {
        let mut paint = ts::Paint::default();
        paint.set_color_rgba8(30, 60, 140, 255);
        paint.anti_alias = true;
        px.fill_path(&path, &paint, ts::FillRule::Winding, ts::Transform::from_scale(scale, scale), None);
    }
    px
}

fn diff(a: &ts::Pixmap, b: &ts::Pixmap) -> (u8, f64) {
    let (mut max, mut over) = (0u8, 0usize);
    for (p, q) in a.data().chunks(4).zip(b.data().chunks(4)) {
        let d = p.iter().zip(q).map(|(s, t)| s.abs_diff(*t)).max().unwrap();
        max = max.max(d);
        if d > 16 {
            over += 1;
        }
    }
    (max, 100.0 * over as f64 / (a.data().len() / 4) as f64)
}

fn ms<T>(runs: u32, mut f: impl FnMut() -> T) -> f64 {
    let t = Instant::now();
    for _ in 0..runs {
        black_box(f());
    }
    t.elapsed().as_secs_f64() * 1e3 / runs as f64
}

fn main() {
    std::fs::create_dir_all("out").unwrap();
    let n = 1_000_000;
    let (x, y, spikes) = signal(n);
    let full_v = [x[0], x[n - 1] + 1e-4];
    let cols = 1000;
    let pts: Vec<minmaxlttb::Point> = x
        .iter()
        .zip(&y)
        .map(|(a, b)| minmaxlttb::Point::new(*a, *b))
        .collect();
    let by_x = |xs: &[f64]| -> Vec<usize> {
        xs.iter().map(|v| x.partition_point(|t| t < v)).collect()
    };

    println!("== fidelity: 1M samples into a {cols}-px panel");
    let t = Instant::now();
    let reference = render(x.iter().cloned().zip(y.iter().cloned()), full_v, cols as u32, 1.0);
    println!("full polyline stroked in {:.0} ms", t.elapsed().as_secs_f64() * 1e3);
    reference.save_png("out/full.png").unwrap();
    let methods: Vec<(&str, Vec<usize>)> = vec![
        ("min/max", minmax(&x, &y, full_v, cols)),
        ("M4", m4(&x, &y, full_v, cols)),
        ("LTTB 2000", by_x(&minmaxlttb::lttb(&pts, 2000, minmaxlttb::Binning::ByCount).unwrap().iter().map(|p| p.x()).collect::<Vec<_>>())),
        ("LTTB 4000", by_x(&minmaxlttb::lttb(&pts, 4000, minmaxlttb::Binning::ByCount).unwrap().iter().map(|p| p.x()).collect::<Vec<_>>())),
        ("MinMaxLTTB 2000", by_x(&minmaxlttb::minmaxlttb(&pts, 2000, 4).unwrap().iter().map(|p| p.x()).collect::<Vec<_>>())),
        ("MinMaxLTTB 4000", by_x(&minmaxlttb::minmaxlttb(&pts, 4000, 4).unwrap().iter().map(|p| p.x()).collect::<Vec<_>>())),
    ];
    for (name, idx) in &methods {
        let img = render(idx.iter().map(|&i| (x[i], y[i])), full_v, cols as u32, 1.0);
        let (max, pct) = diff(&reference, &img);
        let kept = spikes.iter().filter(|s| idx.binary_search(s).is_ok()).count();
        println!(
            "{name:>16}: {:>5} points, diff max {max:>3}, {pct:.3}% px >16, impacts kept {kept}/50",
            idx.len()
        );
        img.save_png(format!("out/{}.png", name.replace([' ', '/'], "_"))).unwrap();
    }

    println!("\n== M4 variants against the full trace at 1×");
    for k in [2, 4] {
        let idx = m4(&x, &y, full_v, k * cols);
        let img = render(idx.iter().map(|&i| (x[i], y[i])), full_v, cols as u32, 1.0);
        let (max, pct) = diff(&reference, &img);
        println!("M4, {k} columns per px: {} points, diff max {max}, {pct:.3}% px >16", idx.len());
        img.save_png(format!("out/M4_x{k}.png")).unwrap();
    }
    // M4 with each column's points drawn at the
    // column's center, so a dense column is one
    // vertical run inside its own pixel.
    let idx = m4(&x, &y, full_v, cols);
    let bounds = columns(&x, full_v[0], full_v[1], cols);
    let col_of = |i: usize| bounds.partition_point(|b| *b <= i) - 1;
    let w = full_v[1] - full_v[0];
    let snapped = idx.iter().map(|&i| {
        let c = col_of(i);
        let dense = bounds[c + 1] - bounds[c] > 4;
        let xs = if dense { full_v[0] + (c as f64 + 0.5) / cols as f64 * w } else { x[i] };
        (xs, y[i])
    });
    let img = render(snapped, full_v, cols as u32, 1.0);
    let (max, pct) = diff(&reference, &img);
    println!("M4 snapped to column centers: diff max {max}, {pct:.3}% px >16");
    img.save_png("out/M4_snapped.png").unwrap();
    let img = render_band(&x, &y, &idx, full_v, cols, 1.0);
    let (max, pct) = diff(&reference, &img);
    println!("M4 line + per-column min/max band: diff max {max}, {pct:.3}% px >16");
    img.save_png("out/M4_band.png").unwrap();

    println!("\n== columns must be device pixels: rendered at 2×");
    let ref2 = render(x.iter().cloned().zip(y.iter().cloned()), full_v, cols as u32, 2.0);
    for c in [cols, 2 * cols] {
        let idx = m4(&x, &y, full_v, c);
        let img = render(idx.iter().map(|&i| (x[i], y[i])), full_v, cols as u32, 2.0);
        let (max, pct) = diff(&ref2, &img);
        println!("M4 with {c} columns: diff max {max}, {pct:.3}% px >16");
    }

    println!("\n== cost per frame (median-free mean of runs)");
    for n in [1_000_000usize, 10_000_000] {
        let (x, y, _) = signal(n);
        let v = [x[0], x[n - 1] + 1e-4];
        let zoom = [x[n / 2], x[n / 2 + n / 20]];
        let pts: Vec<minmaxlttb::Point> = x.iter().zip(&y).map(|(a, b)| minmaxlttb::Point::new(*a, *b)).collect();
        let t = Instant::now();
        let tree = Tree::build(&y);
        let build = t.elapsed().as_secs_f64() * 1e3;
        // Exactness against the raw scan.
        let mut same = 0;
        for k in 0..50 {
            let a = x[k * n / 60];
            let b = x[(k * n / 60 + 1 + k * 997 * (n / 1_000_000)).min(n - 1)];
            let v = [a, b.max(a + 2e-4)];
            same += usize::from(tree.m4(&x, &y, v, cols) == m4(&x, &y, v, cols));
        }
        same += usize::from(tree.m4(&x, &y, full_v_of(&x), cols) == m4(&x, &y, full_v_of(&x), cols));
        println!("{n} samples:");
        println!("  raw M4, full view      {:>8.3} ms", ms(5, || m4(&x, &y, v, cols)));
        println!("  raw M4, 5% view        {:>8.3} ms", ms(20, || m4(&x, &y, zoom, cols)));
        println!("  LTTB 2000              {:>8.3} ms", ms(3, || minmaxlttb::lttb(&pts, 2000, minmaxlttb::Binning::ByCount).unwrap()));
        println!("  MinMaxLTTB 2000        {:>8.3} ms", ms(3, || minmaxlttb::minmaxlttb(&pts, 2000, 4).unwrap()));
        println!("  tree build             {build:>8.1} ms, {:.1}% of the f64 data", 100.0 * tree.bytes() as f64 / (8 * n) as f64);
        println!("  tree M4, full view     {:>8.3} ms", ms(50, || tree.m4(&x, &y, v, cols)));
        println!("  tree M4, 5% view       {:>8.3} ms", ms(50, || tree.m4(&x, &y, zoom, cols)));
        println!("  tree M4 equals raw M4  {same}/51 views");
        let deep = [x[n / 2], x[n / 2 + 2000]];
        println!("  tree M4, 2,000-sample view {:>6.3} ms ({} points)", ms(50, || tree.m4(&x, &y, deep, cols)), tree.m4(&x, &y, deep, cols).len());
    }

    println!("\n== stroking cost, 1000 px wide");
    let (x, y, _) = signal(1_000_000);
    let v = full_v_of(&x);
    let idx = m4(&x, &y, v, cols);
    println!("  1M points  {:>8.1} ms", ms(3, || render(x.iter().cloned().zip(y.iter().cloned()), v, 1000, 1.0)));
    println!("  M4 ({} pts) {:>7.2} ms", idx.len(), ms(20, || render(idx.iter().map(|&i| (x[i], y[i])), v, 1000, 1.0)));
}

fn full_v_of(x: &[f64]) -> [f64; 2] {
    [x[0], x[x.len() - 1] + 1e-4]
}
