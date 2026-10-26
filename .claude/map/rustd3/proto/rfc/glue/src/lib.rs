//! The thin iced layer: a `PathContext` over iced's path builder, and pointer events for core.

use d3::interact::Pointer;
use d3::path::PathContext;
use iced::mouse;
use iced::widget::canvas::{self, Event, path};
use iced::{Point, Radians, Rectangle};
use std::f64::consts::TAU;

/// d3 generators draw into iced paths through this newtype.
pub struct IcedPath<'a>(pub &'a mut path::Builder);

impl PathContext for IcedPath<'_> {
    fn move_to(&mut self, x: f64, y: f64) {
        self.0.move_to(Point::new(x as f32, y as f32));
    }
    fn line_to(&mut self, x: f64, y: f64) {
        self.0.line_to(Point::new(x as f32, y as f32));
    }
    fn quadratic_curve_to(&mut self, x1: f64, y1: f64, x: f64, y: f64) {
        self.0.quadratic_curve_to(Point::new(x1 as f32, y1 as f32), Point::new(x as f32, y as f32));
    }
    fn bezier_curve_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) {
        let p = |x: f64, y: f64| Point::new(x as f32, y as f32);
        self.0.bezier_curve_to(p(x1, y1), p(x2, y2), p(x, y));
    }
    fn arc(&mut self, x: f64, y: f64, r: f64, a0: f64, a1: f64, ccw: bool) {
        // iced sweeps from start to end; canvas semantics pick the direction with `ccw`.
        let mut end = a1;
        if !ccw && end < a0 {
            end += TAU * ((a0 - end) / TAU).ceil();
        } else if ccw && end > a0 {
            end -= TAU * ((end - a0) / TAU).ceil();
        }
        self.0.arc(path::Arc {
            center: Point::new(x as f32, y as f32),
            radius: r as f32,
            start_angle: Radians(a0 as f32),
            end_angle: Radians(end as f32),
        });
    }
    fn close_path(&mut self) {
        self.0.close();
    }
}

/// Builds an iced path from anything that draws into a `PathContext`.
pub fn path(draw: impl FnOnce(&mut IcedPath<'_>)) -> canvas::Path {
    canvas::Path::new(|b| draw(&mut IcedPath(b)))
}

/// Translates an iced canvas event into core's renderer-agnostic pointer input.
pub fn pointer(event: &Event, bounds: Rectangle, cursor: mouse::Cursor) -> Option<Pointer> {
    let at = cursor.position_in(bounds)?;
    let (x, y) = (at.x as f64, at.y as f64);
    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => Some(Pointer::Pressed { x, y }),
        Event::Mouse(mouse::Event::CursorMoved { .. }) => Some(Pointer::Moved { x, y }),
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => Some(Pointer::Released),
        Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
            let lines = match delta {
                mouse::ScrollDelta::Lines { y, .. } => -*y as f64,
                mouse::ScrollDelta::Pixels { y, .. } => -*y as f64 / 20.0,
            };
            Some(Pointer::Wheel { x, y, lines })
        }
        _ => None,
    }
}
