//! [`Placed`]: a brush as a panel stores it, its handles' types erased.

use super::Dim;
use crate::scale::{Invert, Kind, Position, Scale, ScaleRef};

/// A selection's ends over `over`'s range as a scale of the target's type,
/// from a template of that type.
type Window = fn(&dyn Scale, &dyn Scale, Option<[f64; 2]>) -> Option<Box<dyn Scale>>;

/// Where a scale of the target's type puts its domain's ends along
/// `over`'s range.
type Place = fn(&dyn Scale, &dyn Scale) -> Option<[f64; 2]>;

/// A brush as a panel stores it ([`Panel::brush`](crate::Panel::brush)),
/// with its handles' types erased. It holds the axis it selects along and
/// the table positions of the handle it brushes over and of the handle its
/// selection sets, if any. It also holds the conversions between a
/// selection in pixels and a domain of the target's type, which a widget
/// needs without knowing the handles' types. A brush with no target
/// converts to scales of its own handle's type, so a widget keeps its
/// selection as a domain too.
#[derive(Debug, Clone, Copy)]
pub struct Placed {
    dim: Dim,
    over: ScaleRef<()>,
    target: Option<ScaleRef<()>>,
    window: Window,
    place: Place,
}

impl Placed {
    /// A brush along `dim` over `over`, a handle of type `A`, setting
    /// `target`, a handle of type `B`, if any.
    pub(super) fn new<A, B>(dim: Dim, over: ScaleRef<()>, target: Option<ScaleRef<()>>) -> Placed
    where
        A: Invert,
        B: Position<Domain = <A as Kind>::Domain>,
    {
        Placed {
            dim,
            over,
            target,
            window: window::<A, B>,
            place: place::<A, B>,
        }
    }

    /// The axis it selects along.
    #[must_use]
    pub fn get_dim(&self) -> Dim {
        self.dim
    }

    /// The table position of the handle it brushes over.
    #[must_use]
    pub fn get_over(&self) -> usize {
        self.over.get_index()
    }

    /// The table position of the handle its selection sets
    /// ([`Brush::sets`](crate::brush::Brush::sets)), if any.
    pub fn get_target(&self) -> Option<usize> {
        self.target.map(ScaleRef::get_index)
    }

    /// The selection `selection` as a scale of the target's type, or of
    /// `over`'s type without a target. The selection's ends lie in `over`'s
    /// range, in either order; `None` is an empty selection.
    ///
    /// The scale is `template`, a scale of that type, with its domain
    /// fitted without a margin to the values `over` inverts the ends to, in
    /// `template`'s orientation. An empty selection gives `over`'s whole
    /// domain. `over` is the handle's resolved scale set to the panel's
    /// range (`Scale::with_range`). `None` when `over` or `template` is not
    /// of the brush's types, the selection has no width, or an end does not
    /// invert to a value `template` admits.
    pub fn window(
        &self,
        over: &dyn Scale,
        template: &dyn Scale,
        selection: Option<[f64; 2]>,
    ) -> Option<Box<dyn Scale>> {
        (self.window)(over, template, selection)
    }

    /// Where the ends of `window`'s domain lie in `over`'s range, in the
    /// domain's order. This is the selection that
    /// [`window`](Placed::window) inverts. As there, `over` is set to the
    /// panel's range, and `window` is of the target's type (`over`'s
    /// without a target). `None` when either is not of the brush's types
    /// or an end does not lie at a finite position.
    pub fn place(&self, over: &dyn Scale, window: &dyn Scale) -> Option<[f64; 2]> {
        (self.place)(over, window)
    }

    /// The handles the brush is over and sets.
    pub(crate) fn handles(&self) -> Vec<ScaleRef<()>> {
        [Some(self.over), self.target]
            .into_iter()
            .flatten()
            .collect()
    }
}

/// [`Placed::window`] for an `A` over a `B` target.
fn window<A, B>(
    over: &dyn Scale,
    template: &dyn Scale,
    selection: Option<[f64; 2]>,
) -> Option<Box<dyn Scale>>
where
    A: Invert,
    B: Position<Domain = <A as Kind>::Domain>,
{
    let over = over.as_any().downcast_ref::<A>()?;
    let template = template.as_any().downcast_ref::<B>()?;
    let ends = match selection {
        Some([a, b]) if a == b => return None,
        Some([a, b]) => [over.invert(a)?, over.invert(b)?],
        None => Kind::ends(over)?,
    };
    let mut extent = None;
    for end in &ends {
        if !template.admits(end) {
            return None;
        }
        B::merge(&mut extent, end);
    }
    Some(Box::new(template.fitted(extent.as_ref(), &[], 0.0)))
}

/// [`Placed::place`] for an `A` over a `B` target.
fn place<A, B>(over: &dyn Scale, window: &dyn Scale) -> Option<[f64; 2]>
where
    A: Invert,
    B: Position<Domain = <A as Kind>::Domain>,
{
    let over = over.as_any().downcast_ref::<A>()?;
    let window = window.as_any().downcast_ref::<B>()?;
    let [a, b] = Kind::ends(window)?;
    let ends = [over.position(&a)?, over.position(&b)?];
    ends.iter().all(|end| end.is_finite()).then_some(ends)
}

#[cfg(test)]
mod tests {
    use rustd3_core::jiff::{SignedDuration, Timestamp};
    use rustd3_core::scale::{self, Linear, Log};
    use rustd3_core::time::{self, Time};

    use crate::scale::{Kind, Scale};
    use crate::{Figure, Panel, brush};

    /// The domain of `scale`, a linear one.
    fn linear(scale: &dyn Scale) -> [f64; 2] {
        let scale = scale.as_any().downcast_ref::<Linear>().expect("linear");
        Kind::ends(scale).expect("continuous")
    }

    /// A figure with two linear handles, [0, 100] and [10, 0], and two
    /// brushes over the first: one along x that sets the second, and one
    /// along y alone.
    fn handles() -> (Figure<'static>, brush::Placed, brush::Placed) {
        let mut fig = Figure::new();
        let over = fig.scale(scale::linear().domain([0.0, 100.0]));
        let target = fig.scale(scale::linear().domain([10.0, 0.0]));
        let sets: brush::Placed = brush::x(over).sets(target).into();
        let alone: brush::Placed = brush::y(over).into();
        (fig, sets, alone)
    }

    #[test]
    fn a_selection_becomes_the_target_domain_in_its_orientation() {
        let (_, sets, _) = handles();
        let over = scale::linear().domain([0.0, 100.0]).range([0.0, 400.0]);
        let target = scale::linear().domain([10.0, 0.0]);
        let window = sets
            .window(&over, &target, Some([100.0, 200.0]))
            .expect("a window");
        assert_eq!(linear(&*window), [50.0, 25.0]);
        // Placed back: the domain's ends in its order.
        assert_eq!(sets.place(&over, &*window), Some([200.0, 100.0]));
        // An empty selection: over's whole domain.
        let whole = sets.window(&over, &target, None).expect("a window");
        assert_eq!(linear(&*whole), [100.0, 0.0]);
        // No width, or a scale of another type: none.
        assert!(sets.window(&over, &target, Some([5.0, 5.0])).is_none());
        let log = scale::log().domain([1.0, 10.0]);
        assert!(sets.window(&over, &log, Some([1.0, 2.0])).is_none());
        assert!(sets.place(&log, &target).is_none());
    }

    #[test]
    fn a_brush_without_a_target_keeps_its_selection_as_its_own_domain() {
        let (_, _, alone) = handles();
        assert_eq!(alone.get_dim(), brush::Dim::Y);
        assert_eq!((alone.get_over(), alone.get_target()), (0, None));
        // Up a 300 px plot area: y runs from its bottom.
        let over = scale::linear().domain([0.0, 100.0]).range([300.0, 0.0]);
        let window = alone
            .window(&over, &over, Some([30.0, 150.0]))
            .expect("a window");
        assert_eq!(linear(&*window), [50.0, 90.0]);
        let [bottom, top] = alone.place(&over, &*window).expect("placed");
        assert!((bottom - 150.0).abs() < 1e-12 && (top - 30.0).abs() < 1e-12);
    }

    #[test]
    fn a_log_selection_inverts_through_the_log_scale() {
        let mut fig = Figure::new();
        let over = fig.scale(scale::log().domain([1.0, 1000.0]));
        let target = fig.scale(scale::log().domain([1.0, 1000.0]));
        let placed: brush::Placed = brush::x(over).sets(target).into();
        let ranged = scale::log().domain([1.0, 1000.0]).range([0.0, 300.0]);
        let window = placed
            .window(&ranged, &scale::log(), Some([100.0, 200.0]))
            .expect("a window");
        let window = window.as_any().downcast_ref::<Log>().expect("log");
        let [a, b] = window.get_domain();
        assert!(
            (a - 10.0).abs() < 1e-12 && (b - 100.0).abs() < 1e-12,
            "{a}, {b}"
        );
    }

    #[test]
    fn a_time_selection_is_a_window_of_instants() {
        let start: Timestamp = "2026-09-01T00:00:00Z".parse().expect("an instant");
        let end = start + SignedDuration::from_hours(719);
        let mut fig = Figure::new();
        let full = fig.scale(time::utc().domain([start, end]));
        let focus = fig.fit(time::utc());
        let placed: brush::Placed = brush::x(full).sets(focus).into();
        assert_eq!((placed.get_over(), placed.get_target()), (0, Some(1)));
        // 719 px: one hour per pixel.
        let over = time::utc().domain([start, end]).range([0.0, 719.0]);
        let window = placed
            .window(&over, &time::utc(), Some([179.75, 359.5]))
            .expect("a window");
        let [a, b] = window
            .as_any()
            .downcast_ref::<Time>()
            .expect("time")
            .get_domain();
        let hours = |t: Timestamp| t.duration_since(start).as_secs_f64() / 3600.0;
        assert!((hours(a) - 179.75).abs() < 1e-9 && (hours(b) - 359.5).abs() < 1e-9);
    }

    #[test]
    fn a_second_brush_replaces_the_first() {
        let mut fig = Figure::new();
        let x = fig.scale(scale::linear().domain([0.0, 1.0]));
        let y = fig.scale(scale::linear().domain([0.0, 1.0]));
        let panel: &mut Panel<'_> = fig.panel(0, 0);
        panel.brush(brush::x(x)).brush(brush::y(y));
        let placed = panel.brushing().expect("a brush");
        assert_eq!((placed.get_dim(), placed.get_over()), (brush::Dim::Y, 1));
    }
}
