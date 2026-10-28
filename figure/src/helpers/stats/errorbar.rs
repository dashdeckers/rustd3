//! An error bar's reach: [`Err`]; the chart is
//! [`stats::errorbar`](super::errorbar()).

pub(super) mod chart;

/// How far an error bar reaches from its value, as magnitudes
/// (matplotlib's `yerr`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Err {
    /// The same distance below and above.
    Sym(f64),
    /// `(below, above)`.
    Asym(f64, f64),
}

impl Err {
    /// The bar's ends around `value`; `None` unless both distances are
    /// finite and not negative (matplotlib rejects negative errors).
    pub(super) fn ends(self, value: f64) -> Option<[f64; 2]> {
        let (below, above) = match self {
            Err::Sym(e) => (e, e),
            Err::Asym(below, above) => (below, above),
        };
        let valid = |e: f64| e.is_finite() && e >= 0.0;
        (valid(below) && valid(above)).then_some([value - below, value + above])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bars_reach_their_magnitudes_below_and_above() {
        assert_eq!(Err::Sym(0.5).ends(2.0), Some([1.5, 2.5]));
        assert_eq!(Err::Asym(1.0, 0.25).ends(2.0), Some([1.0, 2.25]));
        assert_eq!(Err::Sym(-1.0).ends(2.0), None);
        assert_eq!(Err::Asym(1.0, f64::NAN).ends(2.0), None);
    }
}
