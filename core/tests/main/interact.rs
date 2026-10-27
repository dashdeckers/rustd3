//! `interact`: pointer input and d3-zoom's transform against d3-zoom
//! 3.0.0.

use rustd3_core::interact::{self, Button, Delta, Modifiers, Pointer};

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
