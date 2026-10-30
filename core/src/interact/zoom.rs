//! [`Zoom`], d3-zoom's behavior as a state machine: wheel, drag-to-pan and
//! double-click on an `interact::Transform` the caller stores, within a
//! scale extent and a translate extent (d3-zoom 3.0.0, `zoom.js`).

use crate::interact::{self, Button, Delta, Modifiers, Pointer};
use crate::js;

/// d3's `defaultWheelDelta`: the zoom exponent of one line…
const PER_LINE: f64 = 0.05;
/// …and of one pixel.
const PER_PIXEL: f64 = 0.002;
/// Pixels a horizontal line step pans: d3's own line-to-pixel ratio.
const LINE_PX: f64 = PER_LINE / PER_PIXEL;

/// d3.zoom(): wheel, drag-to-pan and double-click within limits. It keeps
/// only the gesture in progress; the caller stores the transform, passes
/// it to [`handle`](Zoom::handle) with each event and keeps what comes
/// back.
///
/// ```
/// use rustd3_core::interact::{self, Delta, Modifiers, Pointer, Zoom};
///
/// let mut zoom = Zoom::default().scale_extent([1.0, 50.0]);
/// let wheel = Pointer::Wheel {
///     x: 450.0,
///     y: 150.0,
///     delta: Delta::Lines { x: 0.0, y: -20.0 },
///     modifiers: Modifiers::NONE,
/// };
/// let t = zoom.handle(wheel, interact::Transform::IDENTITY);
/// // k = 2^(20 · 0.05) = 2, around the pointer.
/// assert_eq!(t, Some(interact::Transform { k: 2.0, x: -450.0, y: -150.0 }));
/// ```
///
/// Defaults are d3's: scale extent [0, ∞] and an unbounded translate
/// extent. The viewport `extent` is [[0, 0], [0, 0]] until set, since d3
/// reads it from a DOM element, so an anchor of `None` is (0, 0) until then.
///
/// Departures from d3:
///
/// - The wheel anchor (the point a wheel burst zooms around) ends at the
///   next event that is not a wheel step, or when the pointer has moved,
///   not after 150 ms without a step: core reads no clock. The results
///   are equal unless the translate extent constrains.
/// - A horizontal wheel step pans, by 25 px per line (d3's own
///   line-to-pixel ratio, 0.05 / 0.002) and 1 px per pixel; d3 ignores
///   `deltaX`.
/// - A double click applies its ×2 (×0.5 with shift) at once, without
///   d3's 250 ms transition; the app animates if it wants. Double clicks
///   carry no modifiers, so the zoom reads them from
///   [`Pointer::Modifiers`] events.
/// - A drag ends when the left button is released.
#[derive(Debug, Clone, PartialEq)]
pub struct Zoom {
    scale_extent: [f64; 2],
    translate_extent: [[f64; 2]; 2],
    extent: [[f64; 2]; 2],
    gesture: Gesture,
}

/// What a zoom remembers between events.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Gesture {
    /// While the left button is held: the data location under the pointer
    /// when it was pressed.
    grab: Option<(f64, f64)>,
    /// During a wheel burst: where it zooms around.
    wheel: Option<Anchor>,
    /// The keyboard modifiers last reported.
    modifiers: Modifiers,
}

/// A screen point and the data location kept under it (d3's `g.mouse`).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Anchor {
    point: (f64, f64),
    location: (f64, f64),
}

impl Default for Zoom {
    fn default() -> Self {
        Zoom {
            scale_extent: [0.0, f64::INFINITY],
            translate_extent: [
                [f64::NEG_INFINITY, f64::NEG_INFINITY],
                [f64::INFINITY, f64::INFINITY],
            ],
            extent: [[0.0, 0.0], [0.0, 0.0]],
            gesture: Gesture::default(),
        }
    }
}

impl Zoom {
    /// d3's `zoom.scaleExtent([k0, k1])`: the allowed scale factors.
    pub fn scale_extent(self, extent: [f64; 2]) -> Self {
        Zoom {
            scale_extent: extent,
            ..self
        }
    }

    /// d3's `zoom.translateExtent([[x0, y0], [x1, y1]])`: the world, in
    /// data units, the viewport must stay within.
    pub fn translate_extent(self, extent: [[f64; 2]; 2]) -> Self {
        Zoom {
            translate_extent: extent,
            ..self
        }
    }

    /// d3's `zoom.extent([[x0, y0], [x1, y1]])`: the viewport, in screen
    /// units.
    pub fn extent(self, extent: [[f64; 2]; 2]) -> Self {
        Zoom { extent, ..self }
    }

    /// Feeds one event; the new transform if it changed.
    ///
    /// A left press without ctrl grabs the data point under the pointer,
    /// and moves keep it there until the left button is released (d3's
    /// default filter drops other buttons and ctrl-presses); a wheel step
    /// scales by 2^(−δy · 0.05 per line or 0.002 per pixel, × 10 with ctrl)
    /// around the pointer, and its horizontal step pans; a double click
    /// scales by 2 around the pointer, or 0.5 with shift held (ignored with
    /// ctrl held, as d3's filter does).
    pub fn handle(
        &mut self,
        input: Pointer,
        t: interact::Transform,
    ) -> Option<interact::Transform> {
        // Every event but a wheel step ends a wheel burst.
        let burst = self.gesture.wheel.take();
        match input {
            Pointer::Wheel {
                x,
                y,
                delta,
                modifiers,
            } => self.wheeled(burst, (x, y), delta, modifiers, t),
            Pointer::Pressed {
                x,
                y,
                button,
                modifiers,
            } => {
                if button == Button::Left && !modifiers.ctrl() {
                    self.gesture.grab = Some(t.invert((x, y)));
                }
                None
            }
            Pointer::Moved { x, y } => {
                let location = self.gesture.grab?;
                changed(self.constrain(translate(t, (x, y), location)), t)
            }
            Pointer::Released { button } => {
                if button == Button::Left {
                    self.gesture.grab = None;
                }
                None
            }
            Pointer::DoubleClicked { x, y } => {
                let held = self.gesture.modifiers;
                if held.ctrl() {
                    return None;
                }
                let factor = if held.shift() { 0.5 } else { 2.0 };
                let p = (x, y);
                let next = translate(self.scale(t, t.k * factor), p, t.invert(p));
                changed(self.constrain(next), t)
            }
            Pointer::Modifiers(modifiers) => {
                self.gesture.modifiers = modifiers;
                None
            }
            Pointer::Left => None,
        }
    }

    /// A drag is in progress (a focus-and-context view syncs the brush
    /// only between gestures).
    pub fn is_active(&self) -> bool {
        self.gesture.grab.is_some()
    }

    /// d3's `zoom.scaleBy(selection, k, p)`: [`scale_to`](Zoom::scale_to)
    /// `t.k · k`.
    pub fn scale_by(
        &self,
        t: interact::Transform,
        k: f64,
        at: Option<(f64, f64)>,
    ) -> interact::Transform {
        self.scale_to(t, t.k * k, at)
    }

    /// d3's `zoom.scaleTo(selection, k, p)`: scale factor `k`, within the
    /// scale extent, keeping the data point under `at` (the centroid of
    /// the extent for `None`) in place; constrained.
    pub fn scale_to(
        &self,
        t: interact::Transform,
        k: f64,
        at: Option<(f64, f64)>,
    ) -> interact::Transform {
        let p = at.unwrap_or_else(|| self.centroid());
        self.constrain(translate(self.scale(t, k), p, t.invert(p)))
    }

    /// d3's `zoom.translateBy(selection, x, y)`: translated by (`x`, `y`)
    /// in data units; constrained.
    pub fn translate_by(&self, t: interact::Transform, x: f64, y: f64) -> interact::Transform {
        self.constrain(t.translate(x, y))
    }

    /// d3's `zoom.translateTo(selection, x, y, p)`: the data point (`x`,
    /// `y`) shown at `at` (the centroid of the extent for `None`), the
    /// scale factor kept; constrained.
    pub fn translate_to(
        &self,
        t: interact::Transform,
        x: f64,
        y: f64,
        at: Option<(f64, f64)>,
    ) -> interact::Transform {
        let (px, py) = at.unwrap_or_else(|| self.centroid());
        let placed = interact::Transform::IDENTITY
            .translate(px, py)
            .scale(t.k)
            .translate(-x, -y);
        self.constrain(placed)
    }

    /// d3's defaultConstrain against `extent` and `translate_extent`: the
    /// transform translated so the viewport stays within the translate
    /// extent, centred where the viewport is the larger. With the default
    /// unbounded translate extent it returns `t`.
    pub fn constrain(&self, t: interact::Transform) -> interact::Transform {
        let [[ex0, ey0], [ex1, ey1]] = self.extent;
        let [[tx0, ty0], [tx1, ty1]] = self.translate_extent;
        t.translate(
            offset(t.invert_x(ex0) - tx0, t.invert_x(ex1) - tx1),
            offset(t.invert_y(ey0) - ty0, t.invert_y(ey1) - ty1),
        )
    }

    /// d3's `wheeled`: the step scales around the anchor of the `burst` in
    /// progress, or a new one at `p`, then pans by its horizontal part.
    fn wheeled(
        &mut self,
        burst: Option<Anchor>,
        p: (f64, f64),
        delta: Delta,
        modifiers: Modifiers,
        t: interact::Transform,
    ) -> Option<interact::Transform> {
        let (dx, w) = match delta {
            Delta::Lines { x, y } => (x * LINE_PX, -y * PER_LINE),
            Delta::Pixels { x, y } => (x, -y * PER_PIXEL),
        };
        let w = if modifiers.ctrl() { w * 10.0 } else { w };
        let k = self.clamped(t.k * 2f64.powf(w));
        let mut anchor = match burst {
            Some(anchor) if anchor.point == p => anchor,
            // d3 ignores a step that starts no burst and changes nothing.
            None if k == t.k && dx == 0.0 => return None,
            Some(_) | None => Anchor {
                point: p,
                location: t.invert(p),
            },
        };
        let mut next = self.constrain(translate(self.scale(t, k), p, anchor.location));
        if dx != 0.0 {
            next = self.constrain(interact::Transform {
                x: next.x - dx,
                ..next
            });
            anchor.location = next.invert(p);
        }
        self.gesture.wheel = Some(anchor);
        changed(next, t)
    }

    /// d3's `scale(transform, k)`: `k` within the scale extent, the
    /// translation kept.
    fn scale(&self, t: interact::Transform, k: f64) -> interact::Transform {
        interact::Transform {
            k: self.clamped(k),
            ..t
        }
    }

    /// `k` within the scale extent, as JavaScript's `Math.max` and
    /// `Math.min` clamp it (NaN stays NaN).
    fn clamped(&self, k: f64) -> f64 {
        let [low, high] = self.scale_extent;
        js::max(low, js::min(high, k))
    }

    /// d3's `centroid(extent)`: the middle of the viewport.
    fn centroid(&self) -> (f64, f64) {
        let [[x0, y0], [x1, y1]] = self.extent;
        ((x0 + x1) / 2.0, (y0 + y1) / 2.0)
    }
}

/// d3's `translate(transform, p0, p1)`: the transform that shows the data
/// point `location` at the screen point `p`, its scale factor kept.
fn translate(t: interact::Transform, p: (f64, f64), location: (f64, f64)) -> interact::Transform {
    interact::Transform {
        x: p.0 - location.0 * t.k,
        y: p.1 - location.1 * t.k,
        ..t
    }
}

/// defaultConstrain's correction along one axis, from how far the
/// viewport's low (`d0`) and high (`d1`) edges lie past the translate
/// extent's: centred when the viewport is the larger, else the smaller
/// correction (JavaScript's `Math.min(0, d0) || Math.max(0, d1)`).
fn offset(d0: f64, d1: f64) -> f64 {
    if d1 > d0 {
        (d0 + d1) / 2.0
    } else {
        let low = js::min(0.0, d0);
        if js::truthy(low) {
            low
        } else {
            js::max(0.0, d1)
        }
    }
}

/// `next` if it differs from `t`.
fn changed(next: interact::Transform, t: interact::Transform) -> Option<interact::Transform> {
    (next != t).then_some(next)
}

#[cfg(test)]
mod tests {
    //! d3-zoom's `zoom-test.js` exercises the behavior through the DOM;
    //! its numeric sequences are ported here by hand.

    use super::*;

    fn t(k: f64, x: f64, y: f64) -> interact::Transform {
        interact::Transform { k, x, y }
    }

    #[test]
    fn a_transform_set_on_the_zoom_is_kept_as_given() {
        assert_eq!(
            interact::Transform::IDENTITY
                .scale(2.0)
                .translate(1.0, -3.0),
            t(2.0, 2.0, -6.0)
        );
    }

    #[test]
    fn scale_by_keeps_the_given_point_in_place() {
        let zoom = Zoom::default();
        let a = zoom.scale_by(interact::Transform::IDENTITY, 2.0, Some((0.0, 0.0)));
        assert_eq!(a, t(2.0, 0.0, 0.0));
        let b = zoom.scale_by(a, 2.0, Some((2.0, -2.0)));
        assert_eq!(b, t(4.0, -2.0, 2.0));
        let c = zoom.scale_by(b, 0.25, Some((2.0, -2.0)));
        assert_eq!(c, t(1.0, 1.0, -1.0));
    }

    #[test]
    fn scale_to_anchors_at_the_centroid_of_the_empty_extent() {
        let zoom = Zoom::default();
        let a = zoom.scale_to(interact::Transform::IDENTITY, 2.0, None);
        assert_eq!(a, t(2.0, 0.0, 0.0));
        let b = zoom.scale_to(a, 2.0, None);
        assert_eq!(b, t(2.0, 0.0, 0.0));
        assert_eq!(zoom.scale_to(b, 1.0, None), t(1.0, 0.0, 0.0));
    }

    #[test]
    fn translate_by_moves_in_data_units() {
        let zoom = Zoom::default();
        let a = zoom.translate_by(interact::Transform::IDENTITY, 10.0, 10.0);
        assert_eq!(a, t(1.0, 10.0, 10.0));
        let b = zoom.scale_by(a, 2.0, None);
        assert_eq!(b, t(2.0, 20.0, 20.0));
        assert_eq!(zoom.translate_by(b, -10.0, -10.0), t(2.0, 0.0, 0.0));
    }

    #[test]
    fn the_function_argument_sequences_reduce_to_plain_values() {
        let zoom = Zoom::default();
        let identity = interact::Transform::IDENTITY;
        assert_eq!(
            zoom.scale_by(identity, 2.0, Some((0.0, 0.0))),
            t(2.0, 0.0, 0.0)
        );
        assert_eq!(
            zoom.scale_to(identity, 2.0, Some((0.0, 0.0))),
            t(2.0, 0.0, 0.0)
        );
        assert_eq!(zoom.translate_by(identity, 2.0, 3.0), t(1.0, 2.0, 3.0));
    }

    #[test]
    fn the_default_constrain_returns_its_transform() {
        let zoom = Zoom::default();
        assert_eq!(zoom.extent, [[0.0, 0.0], [0.0, 0.0]]);
        assert_eq!(zoom.translate_extent[0][0], f64::NEG_INFINITY);
        let moved = zoom.translate_by(interact::Transform::IDENTITY, 10.0, 10.0);
        assert_eq!(zoom.constrain(moved), t(1.0, 10.0, 10.0));
        let odd = t(3.5, -0.0, 1e300);
        let kept = zoom.constrain(odd);
        assert_eq!(kept, odd);
        assert!(kept.x.is_sign_negative());
    }

    #[test]
    fn translate_to_shows_the_data_point_at_the_anchor() {
        let zoom = Zoom::default().extent([[0.0, 0.0], [960.0, 500.0]]);
        let from = t(2.0, 7.0, 9.0);
        assert_eq!(
            zoom.translate_to(from, 10.0, 20.0, None),
            t(2.0, 460.0, 210.0)
        );
        let at = zoom.translate_to(from, 10.0, 20.0, Some((0.0, 0.0)));
        assert_eq!(at.apply((10.0, 20.0)), (0.0, 0.0));
    }

    #[test]
    fn offset_centres_a_larger_viewport_and_otherwise_takes_the_smaller_correction() {
        assert_eq!(offset(-10.0, 30.0), 10.0);
        assert_eq!(offset(-60.0, -540.0), -60.0);
        assert_eq!(offset(190.0, -290.0), 0.0);
        assert_eq!(offset(25.0, 5.0), 5.0);
        assert_eq!(offset(f64::INFINITY, f64::NEG_INFINITY), 0.0);
        assert_eq!(offset(f64::NAN, 3.0), 3.0);
    }
}
