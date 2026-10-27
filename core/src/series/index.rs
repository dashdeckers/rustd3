//! One channel's exact M4 index: the minimum and maximum of every complete
//! block of [`BLOCK`] samples and of every power-of-two group of blocks.

use std::fmt;
use std::mem;

use crate::series::BLOCK;
use crate::series::chunks::Chunks;

/// Levels `0..=L`: level 0 holds one node per complete block, and node j of
/// level k covers blocks `[j·2ᵏ, (j+1)·2ᵏ)`. Only complete nodes are
/// stored, so level k has ⌊complete blocks / 2ᵏ⌋ of them and an append
/// only pushes.
#[derive(Clone, Default)]
pub(crate) struct Index {
    levels: Vec<Level>,
}

/// The nodes of one level, by extreme.
#[derive(Clone, Default)]
struct Level {
    /// NaN when every sample of the node is NaN.
    min: Vec<f64>,
    max: Vec<f64>,
    /// Offsets of the extremes from the node's first sample.
    at_min: Vec<u32>,
    at_max: Vec<u32>,
}

/// One extreme of a node or a block: its offset from the first sample and
/// its value, NaN when every sample is NaN.
#[derive(Debug, Clone, Copy)]
struct Node {
    at: u32,
    value: f64,
}

/// Which extreme a node or a scan keeps.
#[derive(Debug, Clone, Copy)]
enum Extreme {
    Min,
    Max,
}

/// A summary: the levels, nodes and bytes rather than every node.
impl fmt::Debug for Index {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Index")
            .field("levels", &self.levels.len())
            .field("nodes", &self.levels.iter().map(Level::len).sum::<usize>())
            .field("bytes", &self.len_bytes())
            .finish()
    }
}

impl Index {
    /// Pushes a leaf for every block of `values` completed since the last
    /// call, and every parent each leaf completes, left to right.
    pub(crate) fn extend(&mut self, values: &Chunks<f64>) {
        let done = self.levels.first().map_or(0, Level::len);
        for block in done..values.len() / BLOCK {
            let samples = values.slice(block * BLOCK..(block + 1) * BLOCK);
            let mut nodes = (Extreme::Min.of(samples), Extreme::Max.of(samples));
            for k in 0.. {
                if self.levels.len() == k {
                    self.levels.push(Level::default());
                }
                let level = &mut self.levels[k];
                level.push(nodes);
                let n = level.len();
                if n % 2 == 1 {
                    break;
                }
                let size = (BLOCK << k) as u32;
                nodes = (
                    level.parent(Extreme::Min, n - 2, size),
                    level.parent(Extreme::Max, n - 2, size),
                );
            }
        }
    }

    /// The bytes the levels' vectors hold: their lengths, not capacities.
    pub(crate) fn len_bytes(&self) -> usize {
        self.levels
            .iter()
            .map(|level| level.len() * Level::NODE)
            .sum()
    }

    /// Every level's four vectors, values as bits, for exact comparisons.
    #[cfg(test)]
    pub(crate) fn bits(&self) -> Vec<[Vec<u64>; 4]> {
        let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect();
        let wide = |v: &[u32]| v.iter().map(|&x| u64::from(x)).collect();
        let level = |level: &Level| {
            [
                bits(&level.min),
                bits(&level.max),
                wide(&level.at_min),
                wide(&level.at_max),
            ]
        };
        self.levels.iter().map(level).collect()
    }
}

impl Level {
    /// The bytes of one node across the four vectors.
    const NODE: usize = 2 * mem::size_of::<f64>() + 2 * mem::size_of::<u32>();

    fn len(&self) -> usize {
        self.min.len()
    }

    fn node(&self, extreme: Extreme, j: usize) -> Node {
        match extreme {
            Extreme::Min => Node {
                at: self.at_min[j],
                value: self.min[j],
            },
            Extreme::Max => Node {
                at: self.at_max[j],
                value: self.max[j],
            },
        }
    }

    fn push(&mut self, (min, max): (Node, Node)) {
        self.min.push(min.value);
        self.at_min.push(min.at);
        self.max.push(max.value);
        self.at_max.push(max.at);
    }

    /// The parent of nodes `left` and `left + 1`, each `size` samples.
    fn parent(&self, extreme: Extreme, left: usize, size: u32) -> Node {
        let right = self.node(extreme, left + 1);
        let right = Node {
            at: right.at + size,
            ..right
        };
        extreme.keep(self.node(extreme, left), right)
    }
}

impl Extreme {
    /// Whether `value` replaces `kept`: strictly smaller (min) or larger (max).
    fn beats(self, value: f64, kept: f64) -> bool {
        match self {
            Extreme::Min => value < kept,
            Extreme::Max => value > kept,
        }
    }

    /// The earlier node `kept` unless `next` beats it or `kept` is NaN, so a
    /// left-to-right fold keeps the earliest extreme and skips NaN.
    fn keep(self, kept: Node, next: Node) -> Node {
        if kept.value.is_nan() || self.beats(next.value, kept.value) {
            next
        } else {
            kept
        }
    }

    /// The extreme of `samples`, the earliest on ties.
    fn of(self, samples: &[f64]) -> Node {
        (0..)
            .zip(samples)
            .map(|(at, &value)| Node { at, value })
            .fold(
                Node {
                    at: 0,
                    value: f64::NAN,
                },
                |kept, next| self.keep(kept, next),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Values with NaN runs, repeated values and ±∞, from a fixed seed.
    fn values(n: usize, seed: u64) -> Vec<f64> {
        let mut state = seed;
        let mut nan = 0;
        (0..n)
            .map(|_| {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                let u = (state >> 11) as f64 / (1_u64 << 53) as f64;
                if nan > 0 || u < 0.002 {
                    nan = if nan > 0 {
                        nan - 1
                    } else {
                        (u * 1e5) as usize % 300
                    };
                    return f64::NAN;
                }
                match (u * 1e6) as u64 % 997 {
                    0 => f64::INFINITY,
                    1 => f64::NEG_INFINITY,
                    _ => (u * 40.0).round() - 20.0,
                }
            })
            .collect()
    }

    fn built(pieces: &[&[f64]]) -> (Chunks<f64>, Index) {
        let (mut chunks, mut index) = (Chunks::default(), Index::default());
        for piece in pieces {
            chunks.extend(piece);
            index.extend(&chunks);
        }
        (chunks, index)
    }

    #[test]
    fn levels_after_split_appends_equal_a_one_shot_build() {
        let all = values(1 + 63 + 64 * 1023 + 65_537 + 70_000, 3);
        let mut pieces = Vec::new();
        let mut at = 0;
        for len in [1, 63, 64 * 1023, 65_537] {
            pieces.push(&all[at..at + len]);
            at += len;
        }
        let mut state = 11_u64;
        while at < all.len() {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let len = ((state >> 33) as usize % 5_000).min(all.len() - at);
            pieces.push(&all[at..at + len]);
            at += len;
        }
        let (_, split) = built(&pieces);
        let (_, once) = built(&[&all]);
        assert_eq!(split.bits(), once.bits());
        assert_eq!(split.levels[0].len(), all.len() / BLOCK);
    }

    #[test]
    fn every_node_holds_the_earliest_extreme_of_its_samples() {
        let all = values(64 * 300 + 17, 8);
        let (_, index) = built(&[&all]);
        let leaves = &index.levels[0];
        assert!(leaves.min.iter().any(|v| v.is_nan()), "an all-NaN block");
        assert!(leaves.max.contains(&f64::INFINITY) && leaves.min.contains(&f64::NEG_INFINITY));
        for (k, level) in index.levels.iter().enumerate() {
            assert_eq!(level.len(), (all.len() / BLOCK) >> k);
            for j in 0..level.len() {
                let size = BLOCK << k;
                let samples = &all[j * size..(j + 1) * size];
                let valid = samples.iter().enumerate().filter(|(_, v)| !v.is_nan());
                let min = valid
                    .clone()
                    .fold(None, |m: Option<(usize, f64)>, (i, &v)| match m {
                        Some((_, kept)) if v >= kept => m,
                        _ => Some((i, v)),
                    });
                let max = valid.fold(None, |m: Option<(usize, f64)>, (i, &v)| match m {
                    Some((_, kept)) if v <= kept => m,
                    _ => Some((i, v)),
                });
                for (extreme, expected) in [(Extreme::Min, min), (Extreme::Max, max)] {
                    let node = level.node(extreme, j);
                    match expected {
                        Some((at, value)) => {
                            assert_eq!(
                                (node.at as usize, node.value.to_bits()),
                                (at, value.to_bits())
                            );
                        }
                        None => assert!(node.value.is_nan()),
                    }
                }
            }
        }
    }

    #[test]
    fn index_memory_stays_under_ten_percent_of_data() {
        let n = 1_000_000;
        let (_, index) = built(&[&vec![0.5; n]]);
        let nodes: usize = index.levels.iter().map(Level::len).sum();
        assert_eq!(nodes, 31_243);
        assert_eq!(index.len_bytes(), 749_832);
        let share = index.len_bytes() as f64 / (n * mem::size_of::<f64>()) as f64;
        assert!(share <= 0.10, "{share}");
        assert_eq!(
            format!("{index:?}"),
            "Index { levels: 14, nodes: 31243, bytes: 749832 }"
        );
    }
}
