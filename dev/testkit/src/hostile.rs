//! Hostile inputs for property tests, without a dependency: Marsaglia's
//! xorshift64 generator, values from the edges of `f64` (NaN, ±infinity,
//! −0.0, subnormals, 1e±300, `f64::MAX`), counts from 0 to `usize::MAX`,
//! strings from an alphabet mixed with arbitrary Unicode, and [`run`],
//! which runs a fixed number of cases from a fixed seed and names the seed
//! and the case of the first one that panics.

use std::any::Any;
use std::panic::{self, AssertUnwindSafe};

/// The values every hostile draw favours.
const EDGES: [f64; 24] = [
    0.0,
    -0.0,
    5e-324,
    -5e-324,
    f64::MIN_POSITIVE,
    1e-300,
    -1e-300,
    0.5,
    -0.5,
    1.0,
    -1.0,
    2.0,
    5.0,
    10.0,
    100.0,
    1e9,
    1e300,
    -1e300,
    f64::MAX,
    -f64::MAX,
    f64::INFINITY,
    f64::NEG_INFINITY,
    f64::NAN,
    -f64::NAN,
];

/// The counts every count draw favours.
const COUNTS: [usize; 12] = [
    0,
    1,
    2,
    3,
    5,
    10,
    37,
    1000,
    1_000_000,
    usize::MAX / 2,
    usize::MAX - 1,
    usize::MAX,
];

/// A reproducible source of hostile values: xorshift64 from a seed.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    /// A generator from `seed`; a zero seed, which xorshift never leaves,
    /// is replaced by a fixed odd constant.
    pub fn new(seed: u64) -> Rng {
        Rng(if seed == 0 {
            0x9E37_79B9_7F4A_7C15
        } else {
            seed
        })
    }

    /// The next 64 random bits.
    pub fn bits(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// An index below `n`; 0 when `n` is 0.
    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        // n fits in u64 on every supported target, and the remainder is below n.
        usize::try_from(self.bits() % n as u64).unwrap_or(0)
    }

    /// A value in [0, 1).
    pub fn unit(&mut self) -> f64 {
        (self.bits() >> 11) as f64 / (1_u64 << 53) as f64
    }

    /// True with probability `p`.
    pub fn chance(&mut self, p: f64) -> bool {
        self.unit() < p
    }

    /// One of `items`, which must not be empty.
    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len())]
    }

    /// A hostile `f64`: an edge value, a value near an edge, uniform in
    /// ±1e6, or arbitrary bits.
    pub fn f64(&mut self) -> f64 {
        match self.below(8) {
            0..=3 => *self.pick(&EDGES),
            4 => {
                let edge = *self.pick(&EDGES);
                edge * (1.0 + (self.unit() - 0.5) * 1e-12)
            }
            5 | 6 => (self.unit() * 2.0 - 1.0) * 1e6,
            _ => f64::from_bits(self.bits()),
        }
    }

    /// A hostile but finite `f64`.
    pub fn finite(&mut self) -> f64 {
        let v = self.f64();
        if v.is_finite() {
            v
        } else {
            (self.unit() * 2.0 - 1.0) * 1e3
        }
    }

    /// An ordinary value in ±1e3, for the parts of a case that are not
    /// under test.
    pub fn plain(&mut self) -> f64 {
        (self.unit() * 2.0 - 1.0) * 1e3
    }

    /// A hostile count: an edge count or one below 100.
    pub fn count(&mut self) -> usize {
        if self.chance(0.5) {
            *self.pick(&COUNTS)
        } else {
            self.below(100)
        }
    }

    /// A small count, below `n`, with 0 and 1 favoured.
    pub fn small(&mut self, n: usize) -> usize {
        match self.below(4) {
            0 => 0,
            1 => 1.min(n.saturating_sub(1)),
            _ => self.below(n),
        }
    }

    /// Hostile values: empty, one, or up to `max` of them; sometimes sorted,
    /// sometimes all equal, sometimes with repeats.
    pub fn values(&mut self, max: usize) -> Vec<f64> {
        let n = self.small(max + 1);
        let mut v: Vec<f64> = (0..n)
            .map(|_| {
                if self.chance(0.3) {
                    self.f64()
                } else {
                    self.plain()
                }
            })
            .collect();
        match self.below(5) {
            0 => v.sort_by(f64::total_cmp),
            1 if !v.is_empty() => {
                let first = v[0];
                v.fill(first);
            }
            2 if v.len() > 1 => {
                let i = self.below(v.len());
                let j = self.below(v.len());
                v[j] = v[i];
            }
            _ => {}
        }
        v
    }

    /// A string of up to `max` characters, mostly from `alphabet`, the rest
    /// arbitrary Unicode scalar values.
    pub fn string(&mut self, alphabet: &str, max: usize) -> String {
        let chars: Vec<char> = alphabet.chars().collect();
        let n = self.small(max + 1);
        (0..n)
            .map(|_| {
                if !chars.is_empty() && self.chance(0.85) {
                    *self.pick(&chars)
                } else {
                    self.char()
                }
            })
            .collect()
    }

    /// An arbitrary Unicode scalar value, favouring ASCII, combining marks,
    /// line separators and astral characters.
    pub fn char(&mut self) -> char {
        const ODD: [char; 10] = [
            '\0',
            '\n',
            '\r',
            '\u{2028}',
            '\u{0301}',
            '\u{200B}',
            '\u{FEFF}',
            '\u{FFFD}',
            '\u{1F600}',
            '\u{10FFFF}',
        ];
        match self.below(4) {
            0 => *self.pick(&ODD),
            1 => char::from(u8::try_from(self.below(128)).unwrap_or(b'?')),
            _ => loop {
                let code = u32::try_from(self.bits() % 0x11_0000).unwrap_or(0);
                if let Some(c) = char::from_u32(code) {
                    break c;
                }
            },
        }
    }
}

/// Runs `cases` cases of `property`, each with a generator seeded from
/// `seed` and the case's number, so that a case reruns alone. The first
/// case that panics fails the run with `name`, the seed, the case and the
/// panic's message. `RUSTD3_FUZZ=1` runs ten times as many cases.
pub fn run(name: &str, seed: u64, cases: usize, mut property: impl FnMut(&mut Rng)) {
    let cases = match std::env::var("RUSTD3_FUZZ").as_deref() {
        Ok("1") => cases.saturating_mul(10),
        Ok(_) | Err(_) => cases,
    };
    for case in 0..cases {
        let mut rng = case_rng(seed, case);
        let outcome = panic::catch_unwind(AssertUnwindSafe(|| property(&mut rng)));
        if let Err(payload) = outcome {
            panic!(
                "{name}: seed {seed:#x}, case {case} (rerun with hostile::case({seed:#x}, {case})): {}",
                message(payload.as_ref())
            );
        }
    }
}

/// The generator [`run`] gives case `case` of a run from `seed`.
pub fn case(seed: u64, case: usize) -> Rng {
    case_rng(seed, case)
}

fn case_rng(seed: u64, case: usize) -> Rng {
    // splitmix64 of the case number, so neighbouring cases start far apart.
    let mut z = (case as u64).wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    let mut rng = Rng::new(seed ^ z);
    rng.bits();
    rng
}

fn message(payload: &(dyn Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_owned()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "a panic without a message".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_case_reruns_alone_from_its_seed_and_number() {
        let mut seen = Vec::new();
        run("collect", 7, 5, |rng| seen.push(rng.bits()));
        assert_eq!(seen[3], case(7, 3).bits());
    }

    #[test]
    fn a_panicking_case_is_named_with_its_seed() {
        let outcome = panic::catch_unwind(|| {
            run("named", 0x2a, 10, |rng| {
                assert!(rng.below(4) != 3, "drew a three");
            });
        });
        let payload = outcome.expect_err("some case draws a three");
        let text = message(payload.as_ref());
        assert!(text.starts_with("named: seed 0x2a, case "), "{text}");
        assert!(text.ends_with("drew a three"), "{text}");
    }

    #[test]
    fn draws_reach_the_edges() {
        let mut rng = Rng::new(1);
        let values: Vec<f64> = (0..2000).map(|_| rng.f64()).collect();
        assert!(values.iter().any(|v| v.is_nan()));
        assert!(values.contains(&f64::INFINITY));
        assert!(values.iter().any(|v| *v == 0.0 && v.is_sign_negative()));
        let counts: Vec<usize> = (0..2000).map(|_| rng.count()).collect();
        assert!(counts.contains(&usize::MAX));
        assert!(counts.contains(&0));
    }
}
