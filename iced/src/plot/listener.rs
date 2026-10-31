//! [`Listener`]: an `on_change` closure, its scale type erased, keyed by
//! the handle it listens to.

use rustd3_figure::scale::{Kind, Scale, ScaleRef};

use crate::plot::domains::Key;

/// A scale to the app's message, its type erased.
type Publish<'a, Message> = Box<dyn Fn(&dyn Scale) -> Option<Message> + 'a>;

/// Turns changes to one handle into the app's messages: the widget keeps
/// the handle's domain only while a gesture publishes it.
pub(crate) struct Listener<'a, Message> {
    key: Key,
    publish: Publish<'a, Message>,
}

impl<'a, Message> Listener<'a, Message> {
    /// Listens to `handle`: a scale of its type becomes `f` of its domain's
    /// end stops, in the scale's own type.
    pub(crate) fn new<S>(handle: ScaleRef<S>, f: impl Fn([S::Domain; 2]) -> Message + 'a) -> Self
    where
        S: Kind,
    {
        Listener {
            key: Key::of::<S>(handle.get_index()),
            publish: Box::new(move |scale: &dyn Scale| {
                let scale = scale.as_any().downcast_ref::<S>()?;
                Kind::domain(scale).map(&f)
            }),
        }
    }

    /// The handle listened to.
    pub(crate) fn key(&self) -> Key {
        self.key
    }

    /// The message for `scale`; `None` when it is not of the handle's type
    /// or has no continuous domain.
    pub(crate) fn publish(&self, scale: &dyn Scale) -> Option<Message> {
        (self.publish)(scale)
    }
}

#[cfg(test)]
mod tests {
    use rustd3_core::scale;
    use rustd3_figure::Figure;

    use super::Listener;
    use crate::plot::domains::Key;

    #[test]
    fn a_listener_publishes_its_handles_domain_and_ignores_other_types() {
        let mut fig = Figure::new();
        let _ = fig.scale(scale::linear().domain([0.0, 1.0]));
        let x = fig.scale(scale::linear().domain([0.0, 1.0]));
        let listener = Listener::new(x, |d: [f64; 2]| d);
        assert_eq!(listener.key(), Key::of::<scale::Linear>(1));
        let zoomed = scale::linear().domain([2.0, 3.0]);
        assert_eq!(listener.publish(&zoomed), Some([2.0, 3.0]));
        assert_eq!(listener.publish(&scale::log().domain([1.0, 10.0])), None);
    }
}
