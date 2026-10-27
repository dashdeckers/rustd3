//! [`Gesture`]: the active drag or wheel gesture, its [`Track`]s (one per
//! handle it rescales) and the layout it holds.
//!
//! Each pointer is fed to every track's core `Zoom` with the pointer
//! relative to the gesture panel's held plot area and the track's
//! transform since the gesture began: an x track reads (k, x) of the
//! result, a y track (k, y). A track's scale is its scale at the start
//! rescaled by that transform, through the figure scale's pixel
//! operations (core's `rescale_x` and `rescale_y` semantics): for every
//! output position p of the start scale, the new scale shows the same
//! value at p · k + offset. Transforms accumulate from the start, never
//! step to step, so rounding does not compound; a step the scale rejects
//! leaves the track's transform as it was.

use iced_core::time::{Duration, Instant};
use rustd3_core::interact::{self, Pointer};
use rustd3_figure::kurbo::{Point, Rect};
use rustd3_figure::layout;
use rustd3_figure::scale::Scale;

use crate::plot::domains::{Domains, Key};
use crate::plot::preview::Affine1;
use crate::plot::region::{Along, Region};

/// How long after its last step a wheel gesture ends: d3's own wheel idle
/// is 150 ms (`zoom.js:68`); 300 ms keeps the layout held across a burst.
pub(crate) const WHEEL_END: Duration = Duration::from_millis(300);

/// The gesture in progress.
pub(crate) struct Gesture {
    region: Region,
    kind: Kind,
    /// Per handle: its axis, the transform since the gesture began and the
    /// scale at that moment.
    tracks: Vec<Track>,
    /// Plot areas at the gesture's start (held layout).
    held: Option<layout::Areas>,
    /// The plot area of the region's panel at the start, list px:
    /// pointers are fed relative to its corner.
    area: Rect,
}

/// A drag pans until its release; a wheel gesture ends `WHEEL_END` after
/// its last step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Drag,
    Wheel { last: Instant },
}

/// One handle a gesture rescales.
pub(crate) struct Track {
    key: Key,
    along: Along,
    transform: interact::Transform,
    start: Box<dyn Scale>,
    /// The handle's preview map at the start, range shares.
    base: Affine1,
}

/// An accepted step of one track: the handle, its new scale, and its
/// preview map (range shares since its epoch began).
pub(crate) struct Step {
    pub(crate) key: Key,
    pub(crate) scale: Box<dyn Scale>,
    pub(crate) view: Affine1,
}

impl Track {
    /// A track of `key` along `along` that starts from `start`, whose
    /// preview map is `base` then.
    pub(crate) fn new(key: Key, along: Along, start: Box<dyn Scale>, base: Affine1) -> Track {
        Track {
            key,
            along,
            transform: interact::Transform::IDENTITY,
            start,
            base,
        }
    }
}

impl Gesture {
    /// A gesture of `kind` over `region`, whose panel's plot area is
    /// `area` (list px), rescaling `tracks`, holding the layout `held`.
    pub(crate) fn new(
        region: Region,
        kind: Kind,
        tracks: Vec<Track>,
        held: Option<layout::Areas>,
        area: Rect,
    ) -> Gesture {
        Gesture {
            region,
            kind,
            tracks,
            held,
            area,
        }
    }

    /// What the gesture is over.
    pub(crate) fn region(&self) -> Region {
        self.region
    }

    /// Whether it is a drag.
    pub(crate) fn is_drag(&self) -> bool {
        self.kind == Kind::Drag
    }

    /// The plot areas it holds.
    pub(crate) fn held(&self) -> Option<&layout::Areas> {
        self.held.as_ref()
    }

    /// A wheel step at `now`: a wheel gesture ends `WHEEL_END` after it.
    pub(crate) fn wheeled(&mut self, now: Instant) {
        if let Kind::Wheel { last } = &mut self.kind {
            *last = now;
        }
    }

    /// When a wheel gesture ends: `WHEEL_END` after its last step.
    pub(crate) fn ends_at(&self) -> Option<Instant> {
        match self.kind {
            Kind::Wheel { last } => Some(last + WHEEL_END),
            Kind::Drag => None,
        }
    }

    /// Feeds `pointer` (list px) to every track's zoom in `domains`; the
    /// steps accepted.
    pub(crate) fn feed(&mut self, pointer: Pointer, domains: &mut Domains) -> Vec<Step> {
        let local = local(pointer, self.area.origin());
        let (width, height) = (self.area.width(), self.area.height());
        let mut changed = Vec::new();
        for track in &mut self.tracks {
            let Some(t) = domains.zoom(track.key).handle(local, track.transform) else {
                continue;
            };
            let (before, after) = match track.along {
                Along::X => ((track.transform.k, track.transform.x), (t.k, t.x)),
                Along::Y => ((track.transform.k, track.transform.y), (t.k, t.y)),
            };
            if before == after {
                track.transform = t;
                continue;
            }
            let (range, anchor) = match track.along {
                Along::X => ([0.0, width], position(local).map(|p| p.x)),
                Along::Y => ([height, 0.0], position(local).map(|p| p.y)),
            };
            let (k, offset) = after;
            let rescaled =
                anchor.and_then(|anchor| rescale(&*track.start, range, k, offset, anchor));
            if let Some(scale) = rescaled {
                track.transform = t;
                // The transform in shares of the range: x runs from 0 to
                // the width, y from the height to 0.
                let shares = match track.along {
                    Along::X => Affine1::new(k, offset / width),
                    Along::Y => Affine1::new(k, 1.0 - k - offset / height),
                };
                changed.push(Step {
                    key: track.key,
                    scale,
                    view: shares.after(track.base),
                });
            }
        }
        changed
    }
}

/// `start` over `range` rescaled so that the value it shows at position p
/// is shown at `k` · p + `offset`: zoomed by `k` around `anchor` (a
/// position the gesture is at, so that the zoom moves little), then
/// panned by what remains of `offset`. `None` when the scale rejects
/// either step.
fn rescale(
    start: &dyn Scale,
    range: [f64; 2],
    k: f64,
    offset: f64,
    anchor: f64,
) -> Option<Box<dyn Scale>> {
    if k == 1.0 {
        return if offset == 0.0 {
            Some(start.boxed())
        } else {
            start.pan_px(range, offset)
        };
    }
    let zoomed = start.zoom_px(range, anchor, k)?;
    let rest = offset - anchor * (1.0 - k);
    if rest == 0.0 {
        Some(zoomed)
    } else {
        zoomed.pan_px(range, rest)
    }
}

/// Where `pointer` is, if it says.
fn position(pointer: Pointer) -> Option<Point> {
    match pointer {
        Pointer::Pressed { x, y, .. }
        | Pointer::Moved { x, y }
        | Pointer::Wheel { x, y, .. }
        | Pointer::DoubleClicked { x, y } => Some(Point::new(x, y)),
        Pointer::Released { .. } | Pointer::Modifiers(_) | Pointer::Left => None,
        _ => None,
    }
}

/// `pointer` with its position relative to `origin`.
fn local(pointer: Pointer, origin: Point) -> Pointer {
    let (dx, dy) = (origin.x, origin.y);
    match pointer {
        Pointer::Pressed {
            x,
            y,
            button,
            modifiers,
        } => Pointer::Pressed {
            x: x - dx,
            y: y - dy,
            button,
            modifiers,
        },
        Pointer::Moved { x, y } => Pointer::Moved {
            x: x - dx,
            y: y - dy,
        },
        Pointer::Wheel {
            x,
            y,
            delta,
            modifiers,
        } => Pointer::Wheel {
            x: x - dx,
            y: y - dy,
            delta,
            modifiers,
        },
        Pointer::DoubleClicked { x, y } => Pointer::DoubleClicked {
            x: x - dx,
            y: y - dy,
        },
        Pointer::Released { .. } | Pointer::Modifiers(_) | Pointer::Left => pointer,
        _ => pointer,
    }
}

#[cfg(test)]
mod tests {
    use rustd3_core::interact::{Delta, Modifiers, Pointer};
    use rustd3_core::scale::{self, Linear, Log};
    use rustd3_figure::kurbo::Rect;
    use rustd3_figure::scale::Scale;

    use super::{Gesture, Kind, Step, Track, rescale};
    use crate::plot::domains::{Domains, Key};
    use crate::plot::preview::Affine1;
    use crate::plot::region::{Along, Region};

    /// A wheel of `lines` (iced's sign: positive scrolls up) at `x`, `y`.
    fn wheel(x: f64, y: f64, lines: f64) -> Pointer {
        Pointer::Wheel {
            x,
            y,
            delta: Delta::Lines { x: 0.0, y: -lines },
            modifiers: Modifiers::NONE,
        }
    }

    /// A gesture over a 400 × 300 plot area at (50, 20) on one track of
    /// `start` along `along`.
    fn gesture(start: Box<dyn Scale>, along: Along) -> Gesture {
        let key = Key::new(0, &*start);
        Gesture::new(
            Region::Area { panel: 0 },
            Kind::Drag,
            vec![Track::new(key, along, start, Affine1::IDENTITY)],
            None,
            Rect::new(50.0, 20.0, 450.0, 320.0),
        )
    }

    /// The domain of a linear scale among `changed`.
    fn domain(changed: &[Step]) -> [f64; 2] {
        let [Step { scale, .. }] = changed else {
            panic!("one change, got {}", changed.len());
        };
        scale
            .as_any()
            .downcast_ref::<Linear>()
            .expect("a linear scale")
            .get_domain()
    }

    #[test]
    fn a_zoom_by_two_at_100_px_of_0_to_100_over_400_px_gives_12_5_to_62_5() {
        let x = scale::linear().domain([0.0, 100.0]);
        let zoomed = rescale(&x, [0.0, 400.0], 2.0, -100.0, 100.0).expect("accepted");
        let zoomed = zoomed.as_any().downcast_ref::<Linear>().unwrap();
        assert_eq!(zoomed.get_domain(), [12.5, 62.5]);
        // The value at 100 px stays there.
        let ranged = zoomed.clone().range([0.0, 400.0]);
        assert_eq!(ranged.apply(25.0), 100.0);
    }

    #[test]
    fn a_drag_of_40_px_pans_0_to_100_over_400_px_to_minus_10_to_90() {
        let x = scale::linear().domain([0.0, 100.0]);
        let panned = rescale(&x, [0.0, 400.0], 1.0, 40.0, 0.0).expect("accepted");
        let panned = panned.as_any().downcast_ref::<Linear>().unwrap();
        assert_eq!(panned.get_domain(), [-10.0, 90.0]);
    }

    #[test]
    fn the_new_scale_shows_each_value_at_k_p_plus_the_offset() {
        let starts: [Box<dyn Scale>; 2] = [
            Box::new(scale::linear().domain([-3.0, 7.0])),
            Box::new(scale::log().domain([1.0, 1e4])),
        ];
        for start in starts {
            for (range, k, offset, anchor) in [
                ([0.0, 400.0], 1.7, -55.0, 120.0),
                ([300.0, 0.0], 0.6, 31.0, 250.0),
                ([0.0, 400.0], 1.0, 12.5, 0.0),
            ] {
                let new = rescale(&*start, range, k, offset, anchor).expect("accepted");
                for p in [0.0, 37.0, 150.0, 300.0] {
                    let (seen, expected) = if let Some(s) = start.as_any().downcast_ref::<Linear>()
                    {
                        let s = s.clone().range(range);
                        let n = new.as_any().downcast_ref::<Linear>().unwrap().clone();
                        (n.range(range).apply(s.invert(p)), p * k + offset)
                    } else {
                        let s = start.as_any().downcast_ref::<Log>().unwrap().clone();
                        let s = s.range(range);
                        let n = new.as_any().downcast_ref::<Log>().unwrap().clone();
                        (n.range(range).apply(s.invert(p)), p * k + offset)
                    };
                    assert!(
                        (seen - expected).abs() <= 1e-9 * expected.abs().max(1.0),
                        "{range:?} k {k} offset {offset}: {p} → {seen}, expected {expected}"
                    );
                }
            }
        }
    }

    #[test]
    fn wheel_steps_accumulate_from_the_start_around_the_cursor() {
        let mut domains = Domains::default();
        let mut g = gesture(Box::new(scale::linear().domain([0.0, 100.0])), Along::X);
        // At x = 150 in the area (list x 200): data 37.5 stays there.
        let mut last = [0.0, 100.0];
        for _ in 0..5 {
            last = domain(&g.feed(wheel(200.0, 100.0, 1.0), &mut domains));
        }
        let k = 2f64.powf(0.25);
        let width = 100.0 / k;
        let expected = [37.5 - 0.375 * width, 37.5 + 0.625 * width];
        for (got, want) in last.iter().zip(expected) {
            assert!((got - want).abs() < 1e-12, "{last:?} against {expected:?}");
        }
        // Released carries no position: no change.
        assert!(
            g.feed(
                Pointer::Released {
                    button: rustd3_core::interact::Button::Left
                },
                &mut domains
            )
            .is_empty()
        );
    }

    #[test]
    fn a_y_track_ignores_horizontal_motion() {
        let mut domains = Domains::default();
        let start: Box<dyn Scale> = Box::new(scale::linear().domain([0.0, 10.0]));
        let key = Key::new(0, &*start);
        let mut g = Gesture::new(
            Region::Area { panel: 0 },
            Kind::Drag,
            vec![Track::new(key, Along::Y, start, Affine1::IDENTITY)],
            None,
            Rect::new(0.0, 0.0, 400.0, 300.0),
        );
        let press = Pointer::Pressed {
            x: 100.0,
            y: 100.0,
            button: rustd3_core::interact::Button::Left,
            modifiers: Modifiers::NONE,
        };
        assert!(g.feed(press, &mut domains).is_empty());
        assert!(
            g.feed(Pointer::Moved { x: 160.0, y: 100.0 }, &mut domains)
                .is_empty()
        );
        // 30 px down over 300 px of [0, 10]: content follows, so the
        // domain moves up by one unit.
        let moved = domain(&g.feed(Pointer::Moved { x: 160.0, y: 130.0 }, &mut domains));
        assert!((moved[0] - 1.0).abs() < 1e-12 && (moved[1] - 11.0).abs() < 1e-12);
    }
}
