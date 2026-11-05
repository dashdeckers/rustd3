//! [`Input`]: pointer events queued until the next `RedrawRequested`,
//! which applies them in order. iced can deliver several events before the
//! app's `update` runs and the view is rebuilt.

use rustd3_core::interact::{Button, Pointer};

/// Pointer input waiting for the next redraw, in arrival order.
#[derive(Debug, Default)]
pub(crate) struct Input {
    queue: Vec<Pointer>,
}

impl Input {
    /// Queues `pointer`.
    pub(crate) fn push(&mut self, pointer: Pointer) {
        self.queue.push(pointer);
    }

    /// Whether a left press waits in the queue. Moves after it belong to
    /// the drag it starts.
    pub(crate) fn has_press(&self) -> bool {
        self.queue.iter().any(|pointer| {
            matches!(
                pointer,
                Pointer::Pressed {
                    button: Button::Left,
                    ..
                }
            )
        })
    }

    /// Takes everything queued, oldest first.
    pub(crate) fn take(&mut self) -> Vec<Pointer> {
        std::mem::take(&mut self.queue)
    }
}

#[cfg(test)]
mod tests {
    use rustd3_core::interact::{Button, Modifiers, Pointer};

    use super::Input;

    #[test]
    fn queued_input_comes_out_in_order_once() {
        let mut input = Input::default();
        assert!(!input.has_press());
        let press = Pointer::Pressed {
            x: 1.0,
            y: 2.0,
            button: Button::Left,
            modifiers: Modifiers::NONE,
        };
        input.push(press);
        input.push(Pointer::Moved { x: 3.0, y: 4.0 });
        assert!(input.has_press());
        assert_eq!(input.take(), [press, Pointer::Moved { x: 3.0, y: 4.0 }]);
        assert!(input.take().is_empty());
        assert!(!input.has_press());
    }
}
