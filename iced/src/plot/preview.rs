//! [`Affine1`]: the scaled-preview map of one axis.
//!
//! While a plot-area cell's sharp render is pending, its previous image is
//! drawn at mapped bounds: per axis, `p_current = scale · p_rendered +
//! offset`, from the scales the image was rendered with to those just
//! resolved. Figure scales apply pixel-space operations affinely in output
//! space, so two points, the cell's edges, determine the map for every
//! continuous transform. The object-safe figure scale cannot map a
//! position, so the points come from the gestures' transforms instead
//! (D20.3): each handle keeps the [`View`] its gestures applied since its
//! epoch began, in shares of its range, and each shown plot-area image
//! keeps, per axis, the [`Basis`] its cell was planned at; the map is the
//! current view after the inverse of the planned one, `T_c ∘ T_r⁻¹`.

use rustd3_figure::kurbo::Rect;

use crate::plot::domains::Key;
use crate::plot::region::Along;

/// One axis of a preview map: `p_current = scale · p_rendered + offset`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Affine1 {
    scale: f64,
    offset: f64,
}

/// A handle's preview view: the affine map from range shares (0 at the
/// range's start, 1 at its end) at its epoch's start to range shares now,
/// which its gestures composed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct View {
    /// Distinct for every view a handle starts afresh (first use, reset).
    pub(crate) epoch: u64,
    /// Shares at the epoch's start to shares now.
    pub(crate) map: Affine1,
}

/// The view of the handle that leads one axis of a panel, as a cell was
/// planned (or as it is now).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Basis {
    pub(crate) key: Key,
    pub(crate) view: View,
}

impl Basis {
    /// The map from range shares as planned at `self` to range shares at
    /// `now`; `None` when `now` leads with another handle or epoch.
    pub(crate) fn to(self, now: Basis) -> Option<Affine1> {
        if self.key != now.key || self.view.epoch != now.view.epoch {
            return None;
        }
        if self.view.map == now.view.map {
            return Some(Affine1::IDENTITY);
        }
        self.view.map.inverse().map(|back| now.view.map.after(back))
    }
}

impl Affine1 {
    /// The map that moves nothing.
    pub(crate) const IDENTITY: Affine1 = Affine1 {
        scale: 1.0,
        offset: 0.0,
    };

    /// `p ↦ scale · p + offset`.
    pub(crate) fn new(scale: f64, offset: f64) -> Affine1 {
        Affine1 { scale, offset }
    }

    /// The map that takes `a.0` to `a.1` and `b.0` to `b.1`; `None` when
    /// the two points coincide or the map is not finite.
    pub(crate) fn through(a: (f64, f64), b: (f64, f64)) -> Option<Affine1> {
        let scale = (b.1 - a.1) / (b.0 - a.0);
        let offset = a.1 - scale * a.0;
        (scale.is_finite() && offset.is_finite()).then_some(Affine1 { scale, offset })
    }

    /// The map's scale: 1 for a translation.
    pub(crate) fn scale(self) -> f64 {
        self.scale
    }

    /// Where the map takes `p`.
    pub(crate) fn apply(self, p: f64) -> f64 {
        self.scale * p + self.offset
    }

    /// `self` after `first`: `p ↦ self(first(p))`.
    pub(crate) fn after(self, first: Affine1) -> Affine1 {
        Affine1 {
            scale: self.scale * first.scale,
            offset: self.scale * first.offset + self.offset,
        }
    }

    /// The map back; `None` for a map that collapses the axis.
    pub(crate) fn inverse(self) -> Option<Affine1> {
        let scale = 1.0 / self.scale;
        let offset = -self.offset / self.scale;
        (scale.is_finite() && offset.is_finite()).then_some(Affine1 { scale, offset })
    }

    /// Whether the map moves nothing.
    pub(crate) fn is_identity(self) -> bool {
        self == Affine1::IDENTITY
    }

    /// `shares`, a map of range shares, as a map of device px along
    /// `along` for a cell whose device edges along it are `edges`, in the
    /// plot area `area` (list px) at `factor` device px per list px: two
    /// points, the cell's edges, taken through the shares and back.
    pub(crate) fn device(
        shares: Affine1,
        along: Along,
        area: Rect,
        factor: f64,
        edges: [i32; 2],
    ) -> Option<Affine1> {
        if shares.is_identity() {
            return Some(Affine1::IDENTITY);
        }
        let [a, b] = edges.map(f64::from);
        let mapped = |p: f64| {
            let share = share(along, area, factor, p);
            px(along, area, factor, shares.apply(share))
        };
        Affine1::through((a, mapped(a)), (b, mapped(b)))
    }
}

/// The share of the range at device px `p` along `along` of the plot area
/// `area` (list px) at `factor`: x runs from the area's left, y up from
/// its bottom.
fn share(along: Along, area: Rect, factor: f64, p: f64) -> f64 {
    match along {
        Along::X => (p / factor - area.x0) / area.width(),
        Along::Y => 1.0 - (p / factor - area.y0) / area.height(),
    }
}

/// The device px of range share `u` along `along`, as [`share`] reads it.
fn px(along: Along, area: Rect, factor: f64, u: f64) -> f64 {
    match along {
        Along::X => factor * (area.x0 + u * area.width()),
        Along::Y => factor * (area.y0 + (1.0 - u) * area.height()),
    }
}

#[cfg(test)]
mod tests {
    use rustd3_figure::kurbo::Rect;

    use super::{Affine1, Basis, View};
    use crate::plot::domains::Key;
    use crate::plot::region::Along;

    #[test]
    fn affine_map_from_two_points() {
        // x [0, 100] over 400 device px, zoomed ×2 about 200 px to
        // [25, 75]: p_c = 8 (p_r / 4 − 25) = 2 p_r − 200.
        let current = |p: f64| 8.0 * (p / 4.0 - 25.0);
        let map = Affine1::through((0.0, current(0.0)), (400.0, current(400.0)))
            .expect("two distinct points");
        assert_eq!(map, Affine1::new(2.0, -200.0));
        // A cell over device columns [0, 256) previews at [−200, 312).
        assert_eq!((map.apply(0.0), map.apply(256.0)), (-200.0, 312.0));
        assert_eq!(map.apply(200.0), 200.0, "the zoom's anchor stays");
        assert!(Affine1::through((3.0, 1.0), (3.0, 2.0)).is_none());
    }

    #[test]
    fn shares_map_to_device_px_along_each_axis() {
        // A plot area of 400 × 300 list px at (50, 20), factor 2; the
        // shares zoomed ×2 about their middle.
        let area = Rect::new(50.0, 20.0, 450.0, 320.0);
        let zoom = Affine1::new(2.0, -0.5);
        let x = Affine1::device(zoom, Along::X, area, 2.0, [100, 900]).expect("a map");
        // Device x 500 is the area's middle: it stays; its left edge
        // (100) goes 400 px further left.
        assert_eq!((x.apply(500.0), x.apply(100.0)), (500.0, -300.0));
        let y = Affine1::device(zoom, Along::Y, area, 2.0, [40, 640]).expect("a map");
        assert_eq!((y.apply(340.0), y.apply(640.0)), (340.0, 940.0));
        // Shares that move nothing move no device px.
        let still = Affine1::device(Affine1::IDENTITY, Along::Y, area, 2.0, [40, 100]);
        assert_eq!(still, Some(Affine1::IDENTITY));
    }

    #[test]
    fn a_basis_maps_only_to_the_same_handle_and_epoch() {
        let key = Key::of::<rustd3_core::scale::Linear>(0);
        let other = Key::of::<rustd3_core::scale::Linear>(1);
        let view = |epoch, map| View { epoch, map };
        let planned = Basis {
            key,
            view: view(1, Affine1::new(2.0, 0.0)),
        };
        let now = Basis {
            key,
            view: view(1, Affine1::new(4.0, 1.0)),
        };
        assert_eq!(planned.to(now), Some(Affine1::new(2.0, 1.0)));
        assert_eq!(planned.to(planned), Some(Affine1::IDENTITY));
        assert_eq!(planned.to(Basis { key: other, ..now }), None);
        assert_eq!(
            planned.to(Basis {
                view: view(2, now.view.map),
                ..now
            }),
            None
        );
    }

    #[test]
    fn maps_compose_and_invert() {
        let zoom = Affine1::new(2.0, -200.0);
        let pan = Affine1::new(1.0, 40.0);
        let both = pan.after(zoom);
        assert_eq!(both.apply(100.0), pan.apply(zoom.apply(100.0)));
        let back = both.inverse().expect("invertible");
        assert_eq!(back.after(both), Affine1::IDENTITY);
        assert!(Affine1::new(0.0, 1.0).inverse().is_none());
        assert!(Affine1::IDENTITY.is_identity() && !pan.is_identity());
    }
}
