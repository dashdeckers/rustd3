//! `interact`: pointer input, d3-zoom's transform and the zoom behavior
//! against d3-zoom 3.0.0 (d3's `wheeled`, `mousedowned` and `dblclicked`
//! computed in Node from the vendored sources).

use rustd3_core::interact::{self, Button, Delta, Modifiers, Pointer, Zoom};
use rustd3_testkit::obs::{ToObs, Tolerance};

/// The transform with these fields.
fn t(k: f64, x: f64, y: f64) -> interact::Transform {
    interact::Transform { k, x, y }
}

#[test]
fn scaling_twice_multiplies_the_factors() {
    let transform = interact::Transform::IDENTITY.scale(2.5);
    assert_eq!(transform.scale(2.0), t(5.0, 0.0, 0.0));
}

#[test]
fn translation_is_in_data_units_scaled_by_k() {
    let transform = interact::Transform::IDENTITY.translate(2.0, 3.0);
    assert_eq!(transform.translate(-4.0, 4.0), t(1.0, -2.0, 7.0));
    assert_eq!(
        transform.scale(2.0).translate(-4.0, 4.0),
        t(2.0, -6.0, 11.0)
    );
    assert_eq!(
        interact::Transform::IDENTITY
            .scale(2.0)
            .translate(1.0, -3.0),
        t(2.0, 2.0, -6.0)
    );
}

#[test]
fn apply_and_invert_are_inverse_maps() {
    let transform = interact::Transform::IDENTITY.translate(2.0, 3.0).scale(2.0);
    assert_eq!(transform.apply((4.0, 5.0)), (10.0, 13.0));
    assert_eq!(transform.invert((4.0, 5.0)), (1.0, 1.0));
    assert_eq!(transform.apply_x(4.0), 10.0);
    assert_eq!(transform.apply_y(5.0), 13.0);
    assert_eq!(transform.invert_x(4.0), 1.0);
    assert_eq!(transform.invert_y(5.0), 1.0);
}

#[test]
fn transforms_print_as_an_svg_transform_with_negative_zero_as_zero() {
    let transform = interact::Transform::IDENTITY.translate(2.0, 3.0).scale(2.0);
    assert_eq!(transform.to_string(), "translate(2,3) scale(2)");
    assert_eq!(
        interact::Transform::IDENTITY.to_string(),
        "translate(0,0) scale(1)"
    );
    assert_eq!(t(1.0, -0.0, -0.0).to_string(), "translate(0,0) scale(1)");
    assert_eq!(
        t(0.5, -12.25, 1e-7).to_string(),
        "translate(-12.25,0.0000001) scale(0.5)"
    );
}

#[test]
fn modifiers_union_holds_each_key_of_either() {
    let held = Modifiers::SHIFT.union(Modifiers::CTRL);
    assert!(held.shift() && held.ctrl() && !held.alt() && !held.logo());
    let all = held.union(Modifiers::ALT).union(Modifiers::LOGO);
    assert!(all.shift() && all.ctrl() && all.alt() && all.logo());
    assert_eq!(Modifiers::NONE.union(Modifiers::NONE), Modifiers::NONE);
    assert_eq!(held.union(Modifiers::SHIFT), held);
    assert_eq!(Modifiers::default(), Modifiers::NONE);
    assert!(!Modifiers::NONE.shift());
}

#[test]
fn pointer_input_is_plain_data_matched_with_a_wildcard_arm() {
    let wheel = Pointer::Wheel {
        x: 450.0,
        y: 150.0,
        delta: Delta::Lines { x: 0.0, y: -20.0 },
        modifiers: Modifiers::NONE,
    };
    let lines = match wheel {
        Pointer::Wheel {
            delta: Delta::Lines { y, .. },
            ..
        } => Some(y),
        Pointer::Wheel { .. }
        | Pointer::Pressed { .. }
        | Pointer::Moved { .. }
        | Pointer::Released { .. }
        | Pointer::DoubleClicked { .. }
        | Pointer::Modifiers(_)
        | Pointer::Left => None,
        // Pointer is #[non_exhaustive]: other crates need this arm, which
        // only variants added later reach.
        _ => None,
    };
    assert_eq!(lines, Some(-20.0));
    assert_ne!(
        Pointer::Released {
            button: Button::Left
        },
        Pointer::Released {
            button: Button::Other(4)
        }
    );
}

/// A wheel step at (450, 150), the spot checks' pointer.
fn wheel(delta: Delta, modifiers: Modifiers) -> Pointer {
    Pointer::Wheel {
        x: 450.0,
        y: 150.0,
        delta,
        modifiers,
    }
}

/// A left press at (`x`, `y`) with `modifiers`.
fn press(x: f64, y: f64, button: Button, modifiers: Modifiers) -> Pointer {
    Pointer::Pressed {
        x,
        y,
        button,
        modifiers,
    }
}

/// The transform one wheel step makes from the identity.
fn wheeled(delta: Delta, modifiers: Modifiers) -> Option<interact::Transform> {
    Zoom::default().handle(wheel(delta, modifiers), interact::Transform::IDENTITY)
}

#[test]
fn a_pixel_wheel_step_zooms_around_the_pointer_as_d3() {
    let up = wheeled(Delta::Pixels { x: 0.0, y: -120.0 }, Modifiers::NONE);
    let d3 = t(1.1809926614295305, -81.44669764328876, -27.148899214429576);
    // V8's Math.pow(2, 0.24) is one ulp above the correctly rounded
    // 1.1809926614295303 that f64::powf returns.
    let up = up.expect("a step that zooms");
    assert_eq!(up.k, 1.1809926614295303);
    let fields = |t: interact::Transform| [t.k, t.x, t.y].to_obs();
    assert!(fields(up).approx_eq(&fields(d3), &Tolerance::relative(1e-12)));
}

#[test]
fn line_wheel_steps_zoom_in_and_out_as_d3() {
    let up = wheeled(Delta::Lines { x: 0.0, y: -1.0 }, Modifiers::NONE);
    assert_eq!(
        up,
        Some(t(
            1.0352649238413776,
            -15.869215728619906,
            -5.289738576206645
        ))
    );
    let down = wheeled(Delta::Lines { x: 0.0, y: 3.0 }, Modifiers::NONE);
    assert_eq!(
        down,
        Some(t(0.9012504626108302, 44.43729182512641, 14.812430608375479))
    );
    let twenty = wheeled(Delta::Lines { x: 0.0, y: -20.0 }, Modifiers::NONE);
    assert_eq!(twenty.map(|t| t.k), Some(2.0));
}

#[test]
fn ctrl_multiplies_a_wheel_step_by_ten() {
    let zoomed = wheeled(Delta::Pixels { x: 0.0, y: -10.0 }, Modifiers::CTRL);
    let expected = t(1.148698354997035, -66.91425974866581, -22.30475324955526);
    assert_eq!(zoomed, Some(expected));
}

#[test]
fn a_horizontal_wheel_step_pans_by_25_pixels_a_line() {
    let pixels = wheeled(Delta::Pixels { x: 30.0, y: 0.0 }, Modifiers::NONE);
    assert_eq!(pixels, Some(t(1.0, -30.0, 0.0)));
    let lines = wheeled(Delta::Lines { x: 1.0, y: 0.0 }, Modifiers::NONE);
    assert_eq!(lines, Some(t(1.0, -25.0, 0.0)));
}

#[test]
fn a_wheel_step_that_changes_nothing_is_ignored() {
    assert_eq!(
        wheeled(Delta::Lines { x: 0.0, y: 0.0 }, Modifiers::NONE),
        None
    );
    let mut zoom = Zoom::default().scale_extent([1.0, 1.0]);
    let up = wheel(Delta::Lines { x: 0.0, y: -3.0 }, Modifiers::NONE);
    assert_eq!(zoom.handle(up, interact::Transform::IDENTITY), None);
}

#[test]
fn the_scale_extent_clamps_a_wheel_burst() {
    let mut zoom = Zoom::default().scale_extent([1.0, 50.0]);
    let up = wheel(Delta::Lines { x: 0.0, y: -200.0 }, Modifiers::NONE);
    let top = zoom.handle(up, interact::Transform::IDENTITY);
    assert_eq!(top.map(|t| t.k), Some(50.0));
    let top = top.unwrap();
    assert_eq!(top.apply((450.0, 150.0)), (450.0, 150.0));
    assert_eq!(zoom.handle(up, top), None);
}

#[test]
fn a_wheel_burst_keeps_its_anchor_until_the_pointer_moves() {
    let mut zoom = Zoom::default();
    let step = wheel(Delta::Lines { x: 0.0, y: -1.0 }, Modifiers::NONE);
    let first = zoom.handle(step, interact::Transform::IDENTITY).unwrap();
    let second = zoom.handle(step, first).unwrap();
    assert_eq!(second.k, first.k * first.k);
    assert_eq!(second.invert((450.0, 150.0)), (450.0, 150.0));
    let elsewhere = Pointer::Wheel {
        x: 0.0,
        y: 0.0,
        delta: Delta::Lines { x: 0.0, y: -1.0 },
        modifiers: Modifiers::NONE,
    };
    let third = zoom.handle(elsewhere, second).unwrap();
    assert_eq!(third.apply(second.invert((0.0, 0.0))), (0.0, 0.0));
}

#[test]
fn a_left_drag_pans_until_the_button_is_released() {
    let mut zoom = Zoom::default();
    let identity = interact::Transform::IDENTITY;
    assert!(!zoom.is_active());
    assert_eq!(
        zoom.handle(press(10.0, 10.0, Button::Left, Modifiers::NONE), identity),
        None
    );
    assert!(zoom.is_active());
    let moved = zoom.handle(Pointer::Moved { x: 40.0, y: 25.0 }, identity);
    assert_eq!(moved, Some(t(1.0, 30.0, 15.0)));
    let moved = moved.unwrap();
    assert_eq!(zoom.handle(Pointer::Left, moved), None);
    assert!(zoom.is_active());
    let right = Pointer::Released {
        button: Button::Right,
    };
    assert_eq!(zoom.handle(right, moved), None);
    assert!(zoom.is_active());
    let released = Pointer::Released {
        button: Button::Left,
    };
    assert_eq!(zoom.handle(released, moved), None);
    assert!(!zoom.is_active());
    assert_eq!(
        zoom.handle(Pointer::Moved { x: 90.0, y: 90.0 }, moved),
        None
    );
}

#[test]
fn secondary_buttons_and_ctrl_presses_do_not_drag() {
    for pressed in [
        press(10.0, 10.0, Button::Right, Modifiers::NONE),
        press(10.0, 10.0, Button::Middle, Modifiers::NONE),
        press(10.0, 10.0, Button::Left, Modifiers::CTRL),
    ] {
        let mut zoom = Zoom::default();
        let identity = interact::Transform::IDENTITY;
        assert_eq!(zoom.handle(pressed, identity), None);
        assert!(!zoom.is_active());
        assert_eq!(
            zoom.handle(Pointer::Moved { x: 40.0, y: 25.0 }, identity),
            None
        );
    }
}

#[test]
fn a_translate_extent_keeps_a_drag_inside_the_world() {
    let bounds = [[0.0, 0.0], [960.0, 500.0]];
    let mut zoom = Zoom::default().extent(bounds).translate_extent(bounds);
    let zoomed = t(2.0, -480.0, -250.0);
    zoom.handle(press(480.0, 250.0, Button::Left, Modifiers::NONE), zoomed);
    let to = |x, y| Pointer::Moved { x, y };
    let zoom_at = |x, y| zoom.clone().handle(to(x, y), zoomed);
    assert_eq!(zoom_at(580.0, 250.0), Some(t(2.0, -380.0, -250.0)));
    assert_eq!(zoom_at(1080.0, 250.0), Some(t(2.0, 0.0, -250.0)));
    assert_eq!(zoom_at(1080.0, 600.0), Some(t(2.0, 0.0, 0.0)));
    assert_eq!(zoom_at(-2000.0, -2000.0), Some(t(2.0, -960.0, -500.0)));
    let wide = t(0.5, 0.0, 0.0);
    let mut out = Zoom::default().extent(bounds).translate_extent(bounds);
    out.handle(press(0.0, 0.0, Button::Left, Modifiers::NONE), wide);
    let centred = out.handle(to(100.0, 40.0), wide);
    assert_eq!(centred, Some(t(0.5, 240.0, 125.0)));
}

#[test]
fn a_double_click_zooms_in_and_with_shift_out() {
    let mut zoom = Zoom::default();
    let identity = interact::Transform::IDENTITY;
    let click = Pointer::DoubleClicked { x: 100.0, y: 50.0 };
    assert_eq!(zoom.handle(click, identity), Some(t(2.0, -100.0, -50.0)));
    assert_eq!(
        zoom.handle(Pointer::Modifiers(Modifiers::SHIFT), identity),
        None
    );
    assert_eq!(zoom.handle(click, identity), Some(t(0.5, 50.0, 25.0)));
    zoom.handle(Pointer::Modifiers(Modifiers::NONE), identity);
    assert_eq!(zoom.handle(click, identity), Some(t(2.0, -100.0, -50.0)));
}

#[test]
fn a_ctrl_double_click_is_filtered_as_in_d3() {
    let mut zoom = Zoom::default();
    let identity = interact::Transform::IDENTITY;
    zoom.handle(Pointer::Modifiers(Modifiers::CTRL), identity);
    let click = Pointer::DoubleClicked { x: 100.0, y: 50.0 };
    assert_eq!(zoom.handle(click, identity), None);
}

#[test]
fn the_scale_extent_bounds_a_double_click() {
    let mut zoom = Zoom::default().scale_extent([0.5, 1.0]);
    let click = Pointer::DoubleClicked { x: 100.0, y: 50.0 };
    assert_eq!(zoom.handle(click, interact::Transform::IDENTITY), None);
}
