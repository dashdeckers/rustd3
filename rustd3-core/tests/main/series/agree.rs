//! Regular and explicit clocks of the same x agree, and randomized
//! recordings agree with a plain model of their samples: appends between
//! queries, drops at chunk boundaries, NaN runs, ties, ±∞, and columns
//! narrower and wider than a block. It runs 12 cases by default and 300
//! with `RUSTD3_FUZZ=1`, in release
//! (`cargo test --release -p rustd3-core --test main fuzz`).

use rustd3_core::series::{Clock, Recording};

use super::{Lcg, m4_scan, regular, thinned_by_index, xs};

/// Samples per chunk.
const CHUNK: usize = 65_536;

impl Lcg {
    /// An index below `n`.
    fn below(&mut self, n: usize) -> usize {
        ((self.next() * n as f64) as usize).min(n.saturating_sub(1))
    }

    /// A length from 1 to `max`, log-uniform.
    fn len(&mut self, max: usize) -> usize {
        ((max as f64).powf(self.next()) as usize).clamp(1, max)
    }
}

/// The predicates the queries pick from.
const PREDICATES: [fn(f64) -> bool; 4] = [|_| true, |y| y > 0.0, |y| y.abs() < 0.5, |y| y != 1.0];

/// A recording and the plain vectors it should hold.
struct Model {
    rec: Recording<f64>,
    /// The clock: `Some((start, period))` for a regular one.
    regular: Option<(f64, f64)>,
    offset: usize,
    x: Vec<f64>,
    channels: [Vec<f64>; 2],
}

impl Model {
    fn new(random: &mut Lcg) -> Self {
        let regular =
            (random.next() < 0.5).then(|| (random.next() * 100.0 - 50.0, 1e-3 + random.next()));
        let clock = match regular {
            Some((start, period)) => Clock::Regular { start, period },
            None => Clock::Explicit(Vec::new()),
        };
        let rec = Recording::new(clock, [[0.0; 0]; 2]).expect("an empty recording");
        Self {
            rec,
            regular,
            offset: 0,
            x: Vec::new(),
            channels: [Vec::new(), Vec::new()],
        }
    }

    /// Appends `n` samples: values with NaN runs, ties and ±∞; explicit x
    /// with repeats.
    fn append(&mut self, n: usize, random: &mut Lcg) {
        let first = self.offset + self.x.len();
        let x: Vec<f64> = match self.regular {
            Some((start, period)) => (first..first + n)
                .map(|i| start + i as f64 * period)
                .collect(),
            None => {
                let mut last = self.x.last().copied().unwrap_or(-10.0);
                (0..n)
                    .map(|_| {
                        last += [0.0, 1e-3, 0.25][random.below(3)];
                        last
                    })
                    .collect()
            }
        };
        let channels: [Vec<f64>; 2] = std::array::from_fn(|_| values(n, random));
        let given = if self.regular.is_some() {
            &[][..]
        } else {
            &x[..]
        };
        self.rec.extend(given, &channels).expect("a valid append");
        self.x.extend(x);
        for (model, new) in self.channels.iter_mut().zip(channels) {
            model.extend(new);
        }
    }

    /// Drops what lies before `before`, checks what went and returns it.
    fn drop_front(&mut self, before: usize) -> usize {
        let dropped = self.rec.drop_front(before);
        assert_eq!(dropped, before.min(self.x.len()) / CHUNK * CHUNK);
        self.offset += dropped;
        self.x.drain(..dropped);
        for channel in &mut self.channels {
            channel.drain(..dropped);
        }
        dropped
    }

    /// Checks the recording against the model with random queries.
    fn check(&self, random: &mut Lcg) {
        let len = self.x.len();
        assert_eq!(
            (self.rec.len(), self.rec.get_offset()),
            (len, self.offset as u64)
        );
        if len == 0 {
            return;
        }
        assert_eq!(xs(&self.rec), self.x);
        for _ in 0..4 {
            let edges = self.edges(random);
            let valid = PREDICATES[random.below(PREDICATES.len())];
            for (channel, y) in self.channels.iter().enumerate() {
                let rec = self.rec.series(channel).expect("a channel");
                let columns = self.rec.columns(&edges).expect("valid edges");
                let lines = rec.m4(&columns, valid).expect("fresh columns");
                let lines: Vec<Vec<usize>> = lines.iter().map(<[usize]>::to_vec).collect();
                assert_eq!(
                    (columns.get_edges().to_vec(), lines),
                    m4_scan(&self.x, y, &edges, valid)
                );
                let a = random.below(len);
                let b = a + random.len(len - a + 10);
                let found = rec
                    .extremes(a..b, valid)
                    .map(|e| (e.get_min(), e.get_max()));
                assert_eq!(
                    found.map(bits),
                    scan(&y[a..b.min(len)], a, valid).map(bits),
                    "{a}..{b}"
                );
            }
        }
    }

    /// Random data-space column edges around the x extent: 1 to 3,000
    /// columns over 2 to all samples.
    fn edges(&self, random: &mut Lcg) -> Vec<f64> {
        let (lo, hi) = (self.x[0], self.x[self.x.len() - 1]);
        let span = (hi - lo).max(1e-3);
        let a = lo - 0.05 * span + random.next() * span;
        let b = a + random.next() * (hi + 0.05 * span - a).max(0.0);
        let columns = random.len(3_000);
        (0..=columns)
            .map(|c| a + (b - a) * c as f64 / columns as f64)
            .collect()
    }
}

/// Values with NaN runs, ties, signed zeros and ±∞.
fn values(n: usize, random: &mut Lcg) -> Vec<f64> {
    let mut gap = 0;
    (0..n)
        .map(|_| {
            let u = random.next();
            if gap > 0 || u < 2e-4 {
                gap = if gap > 0 {
                    gap - 1
                } else {
                    (u * 1e9) as usize % 700
                };
                return f64::NAN;
            }
            match (u * 1e9) as u64 % 1_000 {
                0 => f64::INFINITY,
                1 => f64::NEG_INFINITY,
                2 => -0.0,
                _ => (u * 16.0).floor() / 8.0 - 1.0,
            }
        })
        .collect()
}

/// The earliest minimum and maximum of `y` among values that are not NaN
/// and pass `valid`, with indices from `first`.
fn scan(y: &[f64], first: usize, valid: fn(f64) -> bool) -> Option<((usize, f64), (usize, f64))> {
    let mut found: Option<((usize, f64), (usize, f64))> = None;
    for (i, &v) in (first..).zip(y).filter(|(_, v)| !v.is_nan() && valid(**v)) {
        let (min, max) = found.get_or_insert(((i, v), (i, v)));
        if v < min.1 {
            *min = (i, v);
        }
        if v > max.1 {
            *max = (i, v);
        }
    }
    found
}

/// Extremes with their values as bits, so ±0 and ±∞ compare exactly.
fn bits(((a, x), (b, y)): ((usize, f64), (usize, f64))) -> (usize, u64, usize, u64) {
    (a, x.to_bits(), b, y.to_bits())
}

#[test]
fn regular_and_explicit_clocks_of_the_same_x_agree() {
    let mut random = Lcg(31);
    let n = 2 * CHUNK + 4_321;
    let y = values(n, &mut random);
    let rec = regular(0.1, 1e-4, &[&y]);
    let explicit = Recording::new(Clock::Explicit(xs(&rec)), [&y]).expect("the same x");
    let (lo, hi) = (0.1, 0.1 + n as f64 * 1e-4);
    for columns in [1, 2, 999, 4_000] {
        for (a, b) in [
            (lo - 1.0, hi + 1.0),
            (lo, hi),
            (6.5537, 6.5541),
            (lo + 3.0, lo + 3.3),
        ] {
            let edges: Vec<f64> = (0..=columns)
                .map(|c| a + (b - a) * c as f64 / columns as f64)
                .collect();
            let regular = thinned_by_index(&rec, &edges, |_| true);
            assert_eq!(regular, thinned_by_index(&explicit, &edges, |_| true));
            assert_eq!(regular, m4_scan(&xs(&rec), &y, &edges, |_| true));
        }
    }
}

#[test]
fn fuzzed_recordings_agree_with_a_raw_scan_of_their_samples() {
    let cases = match std::env::var("RUSTD3_FUZZ").as_deref() {
        Ok("1") => 300,
        Ok(_) | Err(_) => 12,
    };
    let mut random = Lcg(2026);
    let mut drops = 0;
    for _ in 0..cases {
        let mut model = Model::new(&mut random);
        for _ in 0..6 {
            let n = match random.below(4) {
                0 => random.len(70),
                1 => random.len(5_000),
                2 => CHUNK - 64 + random.below(130),
                _ => random.len(150_000),
            };
            model.append(n, &mut random);
            model.check(&mut random);
            if random.next() < 0.3 {
                let len = model.x.len();
                let before = match random.below(3) {
                    0 => len / CHUNK * CHUNK,
                    1 => random.below(len + 1),
                    _ => usize::MAX,
                };
                drops += usize::from(model.drop_front(before) > 0);
                model.check(&mut random);
            }
        }
    }
    assert!(drops > 0, "no case dropped a chunk");
}
