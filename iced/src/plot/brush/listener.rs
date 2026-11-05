//! [`Listener`]: an `on_brush` closure, its scale type erased, keyed by
//! the handle the brush is over.

use rustd3_figure::scale::{Invert, Kind, Scale, ScaleRef};

use crate::plot::domains::Key;

/// A function from a selection in pixels, inverted through the brush's
/// scale, to the app's message.
type Publish<'a, Message> = Box<dyn Fn(&dyn Scale, Option<[f64; 2]>) -> Option<Message> + 'a>;

/// Turns the selections of brushes over one handle into the app's
/// messages.
pub(crate) struct Listener<'a, Message> {
    key: Key,
    publish: Publish<'a, Message>,
}

impl<'a, Message> Listener<'a, Message> {
    /// A listener to brushes over `over`. A selection becomes `f` of the
    /// values the handle's scale inverts its ends to, in their order, and
    /// no selection becomes `f(None)`.
    pub(crate) fn new<A>(
        over: ScaleRef<A>,
        f: impl Fn(Option<[<A as Kind>::Domain; 2]>) -> Message + 'a,
    ) -> Self
    where
        A: Invert,
    {
        Listener {
            key: Key::of::<A>(over.get_index()),
            publish: Box::new(move |scale: &dyn Scale, selection| {
                let Some([a, b]) = selection else {
                    return Some(f(None));
                };
                let scale = scale.as_any().downcast_ref::<A>()?;
                Some(f(Some([scale.invert(a)?, scale.invert(b)?])))
            }),
        }
    }

    /// The handle the brushes listened to are over.
    pub(crate) fn key(&self) -> Key {
        self.key
    }

    /// The message for `selection` (list px along the brush's axis) of a
    /// brush over `over`, the handle's scale set to the plot area's range.
    /// `None` when `over` is not of the handle's type or an end does not
    /// invert.
    pub(crate) fn publish(&self, over: &dyn Scale, selection: Option<[f64; 2]>) -> Option<Message> {
        (self.publish)(over, selection)
    }
}

#[cfg(test)]
mod tests {
    use rustd3_core::scale;
    use rustd3_figure::Figure;

    use super::Listener;
    use crate::plot::domains::Key;

    #[test]
    fn a_selection_is_published_in_the_handles_domain() {
        let mut fig = Figure::new();
        let x = fig.scale(scale::linear().domain([0.0, 100.0]));
        let listener = Listener::new(x, |d: Option<[f64; 2]>| d);
        assert_eq!(listener.key(), Key::of::<scale::Linear>(0));
        let over = scale::linear().domain([0.0, 100.0]).range([0.0, 400.0]);
        assert_eq!(
            listener.publish(&over, Some([100.0, 200.0])),
            Some(Some([25.0, 50.0]))
        );
        assert_eq!(listener.publish(&over, None), Some(None));
        let log = scale::log().domain([1.0, 10.0]);
        assert_eq!(listener.publish(&log, Some([1.0, 2.0])), None);
    }
}
