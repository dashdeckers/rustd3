//! [`Grid`]: a thinning column grid anchored in output space.

/// A thinning column grid anchored in output space (plain data): column
/// `c` spans `[origin + c / density, origin + (c + 1) / density)` list px
/// along a plot area, from its left edge (the x scale's output in a
/// panel), for every integer `c`.
///
/// A widget that pans by whole device pixels moves `origin` by the same
/// amount, so the data boundaries of the columns that stay in view repeat
/// and their M4 samples with them; outside gestures a line thins on the
/// grid anchored at its plot area's left edge at the target's density.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Grid {
    /// Output px of the data anchor.
    pub origin: f64,
    /// Columns per output px.
    pub density: f64,
}

impl Grid {
    /// The columns that intersect the view `[lo, hi]` (output px), first
    /// and last; `None` when the grid is not finite and positive or no
    /// column does. Products within 1e-9 of a whole column count as on it,
    /// so a view whose ends lie on column edges takes no extra column.
    pub(crate) fn span(self, [lo, hi]: [f64; 2]) -> Option<(i64, i64)> {
        const SLACK: f64 = 1e-9;
        let finite = self.origin.is_finite() && self.density.is_finite();
        if !finite || self.density <= 0.0 {
            return None;
        }
        let first = ((lo - self.origin) * self.density + SLACK).floor();
        let past = ((hi - self.origin) * self.density - SLACK).ceil();
        let limit = f64::from(u32::MAX);
        let finite = first.is_finite() && past.is_finite();
        if !finite || past <= first || past - first > limit {
            return None;
        }
        Some((first as i64, past as i64 - 1))
    }

    /// Output px of the left edge of column `c`.
    pub(crate) fn edge(self, c: i64) -> f64 {
        self.origin + c as f64 / self.density
    }
}

#[cfg(test)]
mod tests {
    use super::Grid;

    #[test]
    fn a_view_on_column_edges_takes_no_extra_column() {
        let grid = Grid {
            origin: 0.0,
            density: 1.25,
        };
        // 720 list px at 1.25 are 900 device columns, 0 to 899.
        assert_eq!(grid.span([0.0, 720.0]), Some((0, 899)));
        let shifted = Grid {
            origin: 37.0 / 1.25,
            ..grid
        };
        assert_eq!(shifted.span([0.0, 720.0]), Some((-37, 862)));
        assert_eq!(shifted.edge(-37), 0.0);
    }

    #[test]
    fn partly_covered_columns_count() {
        let grid = Grid {
            origin: 0.3,
            density: 1.0,
        };
        assert_eq!(grid.span([0.0, 10.0]), Some((-1, 9)));
        // A view of no width inside a column still intersects it.
        assert_eq!(grid.span([5.0, 5.0]), Some((4, 4)));
        assert_eq!(grid.span([5.0, 4.0]), None);
        let flat = Grid {
            origin: 0.0,
            density: 0.0,
        };
        assert_eq!(flat.span([0.0, 10.0]), None);
    }
}
