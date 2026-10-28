//! [`Slots`]: where a chart's groups sit across their key axis.

use crate::scale::Position;

/// The share of the gap between neighbouring keys a group takes on a scale
/// without bands, as bars take it.
const SHARE: f64 = 0.8;

/// Where a chart's groups sit across their key axis, as bars sit: each
/// key's band on band scales, else a width centred on the key, 0.8 of the
/// smallest gap between the keys' positions (of the axis for keys at one
/// position); dodged into a palette key's part of it when asked.
pub(super) struct Slots {
    /// Each group's width in px.
    width: f64,
    /// Whether the scale has bands, whose start `position` gives.
    banded: bool,
    /// The dodge slot `(k, n)`: the k-th of n equal parts.
    dodge: Option<(usize, usize)>,
}

impl Slots {
    /// How groups over `keys` sit across `scale`, an axis `length` px long,
    /// in part `dodge` of their slot when given.
    pub(super) fn new<'k, S: Position>(
        scale: &S,
        keys: impl Iterator<Item = &'k S::Domain>,
        length: f64,
        dodge: Option<(usize, usize)>,
    ) -> Slots {
        let bandwidth = scale.bandwidth();
        if bandwidth > 0.0 {
            return Slots {
                width: bandwidth,
                banded: true,
                dodge,
            };
        }
        let mut at: Vec<f64> = keys
            .filter_map(|k| scale.position(k))
            .filter(|p| p.is_finite())
            .collect();
        at.sort_by(f64::total_cmp);
        let gap = at
            .windows(2)
            .map(|w| w[1] - w[0])
            .filter(|d| *d > 0.0)
            .fold(f64::INFINITY, f64::min);
        Slots {
            width: SHARE * if gap.is_finite() { gap } else { length },
            banded: false,
            dodge,
        }
    }

    /// The slot `(start, width)` in px of the group over `key`; `None` for
    /// a key outside the domain or a position that is not finite.
    pub(super) fn of<S: Position>(&self, scale: &S, key: &S::Domain) -> Option<(f64, f64)> {
        let p = scale.position(key).filter(|p| p.is_finite())?;
        let start = if self.banded { p } else { p - self.width / 2.0 };
        Some(match self.dodge {
            Some((k, n)) if n > 0 => {
                let w = self.width / n as f64;
                (start + k as f64 * w, w)
            }
            Some(_) | None => (start, self.width),
        })
    }
}

#[cfg(test)]
mod tests {
    use rustd3_core::scale;

    use super::*;

    #[test]
    fn groups_take_their_band_or_its_dodged_part() {
        let band = scale::band(["a", "b"]).range([0.0, 100.0]);
        let slots = Slots::new(&band, ["a", "b"].iter(), 100.0, None);
        assert_eq!(slots.of(&band, &"b"), Some((50.0, 50.0)));
        let dodged = Slots::new(&band, ["a"].iter(), 100.0, Some((1, 2)));
        assert_eq!(dodged.of(&band, &"a"), Some((25.0, 25.0)));
        assert_eq!(slots.of(&band, &"z"), None);
    }

    #[test]
    fn groups_on_a_continuous_scale_take_most_of_the_smallest_gap() {
        let x = scale::linear().domain([0.0, 10.0]).range([0.0, 100.0]);
        let slots = Slots::new(&x, [1.0, 3.0, 8.0].iter(), 100.0, None);
        let (start, width) = slots.of(&x, &3.0).expect("placed");
        assert!((width - 16.0).abs() < 1e-12 && (start - 22.0).abs() < 1e-12);
        let one = Slots::new(&x, [5.0].iter(), 100.0, None);
        assert_eq!(one.of(&x, &5.0), Some((10.0, 80.0)));
    }
}
