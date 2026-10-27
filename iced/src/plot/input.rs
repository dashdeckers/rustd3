//! [`Input`]: pointer events queued until the next `RedrawRequested`,
//! which applies them in order: iced can deliver several events before the
//! app's `update` runs and the view is rebuilt.

use rustd3_core::interact::Pointer;

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
        let press = Pointer::Pressed {
            x: 1.0,
            y: 2.0,
            button: Button::Left,
            modifiers: Modifiers::NONE,
        };
        input.push(press);
        input.push(Pointer::Moved { x: 3.0, y: 4.0 });
        assert_eq!(input.take(), [press, Pointer::Moved { x: 3.0, y: 4.0 }]);
        assert!(input.take().is_empty());
    }
}
