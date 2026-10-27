//! `interact`: pointer input, d3-zoom's transform, the zoom behavior and
//! `Rescale` against d3-zoom 3.0.0 and d3-scale 4.0.2 (d3's `wheeled`,
//! `mousedowned`, `dblclicked` and `rescaleX` computed in Node from the
//! vendored sources), and the pixel-space gestures against their rule.

use rustd3_core::interact::{self, Button, Delta, Modifiers, Pointer, Rescale, Zoom};
use rustd3_core::scale::{self, AnyContinuous, TickRule};
use rustd3_testkit::obs::{ToObs, Tolerance};

/// The transform with these fields.
fn t(k: f64, x: f64, y: f64) -> interact::Transform {
    interact::Transform { k, x, y }
}

#[test]
fn the_default_transform_is_the_identity() {
    assert_eq!(
        interact::Transform::default(),
        interact::Transform::IDENTITY
    );
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
fn ctrl_multiplies_a_wheel_step_by_ten_and_the_scale_rescales_as_d3() {
    let zoomed = wheeled(Delta::Pixels { x: 0.0, y: -10.0 }, Modifiers::CTRL);
    let expected = t(1.148698354997035, -66.91425974866581, -22.30475324955526);
    assert_eq!(zoomed, Some(expected));
    let visible = x().rescale_x(&expected).get_domain();
    assert_eq!(visible, [6.067942345494186, 93.12299867510657]);
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

/// The spot checks' scale: [0, 100] over 960 px.
fn x() -> scale::Linear {
    scale::linear().domain([0.0, 100.0]).range([0.0, 960.0])
}

/// d3's `log().domain([1, 1000]).range([0, 300])`.
fn log() -> scale::Log {
    scale::log().domain([1.0, 1000.0]).range([0.0, 300.0])
}

/// Whether `a` and `b` agree within 1e-12 relative.
fn near<const N: usize>(a: [f64; N], b: [f64; N]) -> bool {
    a.to_obs()
        .approx_eq(&b.to_obs(), &Tolerance::relative(1e-12))
}

#[test]
fn rescale_is_d3s_on_linear_and_log_scales() {
    let half = interact::Transform::IDENTITY
        .translate(-480.0, 0.0)
        .scale(2.0);
    assert_eq!(x().rescale_x(&half).get_domain(), [25.0, 75.0]);
    let log_half = interact::Transform::IDENTITY
        .translate(-150.0, 0.0)
        .scale(2.0);
    assert_eq!(
        log().rescale_x(&log_half).get_domain(),
        [5.62341325190349, 177.82794100389225]
    );
    let y = scale::linear().domain([0.0, 10.0]).range([500.0, 0.0]);
    let vertical = interact::Transform::IDENTITY
        .translate(0.0, -250.0)
        .scale(2.0);
    assert_eq!(y.rescale_y(&vertical).get_domain(), [2.5, 7.5]);
    assert_eq!(y.rescale_y(&vertical).get_range(), &[500.0, 0.0]);
}

#[test]
fn pixel_gestures_pan_and_zoom_the_domain() {
    let zoomed = x().zoom_px(480.0, 2.0).map(|s| s.get_domain());
    assert_eq!(zoomed, Some([25.0, 75.0]));
    assert_eq!(
        x().pan_px(96.0).map(|s| s.get_domain()),
        Some([-10.0, 90.0])
    );
    let out = x().zoom_px(0.0, 0.5).map(|s| s.get_domain());
    assert_eq!(out, Some([0.0, 200.0]));
    let flipped = scale::linear().domain([100.0, 0.0]).range([0.0, 960.0]);
    let panned = flipped.pan_px(96.0).expect("a valid pan").get_domain();
    assert!(near(panned, [110.0, 10.0]));
}

#[test]
fn zoom_px_equals_rescale_x_for_an_unclamped_scale() {
    let half = |anchor: f64| t(2.0, -anchor, 0.0);
    let log_zoomed = log().zoom_px(150.0, 2.0).map(|s| s.get_domain());
    assert_eq!(log_zoomed, Some(log().rescale_x(&half(150.0)).get_domain()));
    assert_eq!(log_zoomed, Some([5.62341325190349, 177.82794100389225]));
    // t = (k, a·(1 − k), 0) shows at r what zoom_px(a, k) does.
    for (anchor, factor) in [(480.0, 2.0), (100.0, 3.5), (900.0, 0.25), (-50.0, 1.7)] {
        let t = t(factor, anchor * (1.0 - factor), 0.0);
        for scale in [x().into(), AnyContinuous::from(log())] {
            let gesture = scale.zoom_px(anchor, factor).expect("a valid zoom");
            assert!(near(gesture.get_domain(), scale.rescale_x(&t).get_domain()));
        }
    }
}

#[test]
fn a_clamped_scale_cannot_zoom_out_by_rescale_but_can_by_gesture() {
    let clamped = x().clamp(true);
    let out = t(0.5, 240.0, 0.0);
    assert_eq!(clamped.rescale_x(&out).get_domain(), [0.0, 100.0]);
    let gesture = clamped.zoom_px(480.0, 0.5).map(|s| s.get_domain());
    assert_eq!(gesture, Some([-50.0, 150.0]));
}

#[test]
fn log_gestures_move_in_transformed_space() {
    let panned = log().pan_px(100.0).expect("a valid pan").get_domain();
    assert!(near(panned, [0.10000000000000002, 99.99999999999996]));
    let zoomed = log().zoom_px(0.0, 3.0).expect("a valid zoom").get_domain();
    assert!(near(zoomed, [1.0, 10.0]));
}

#[test]
fn piecewise_stops_move_together() {
    let s = scale::piecewise([0.0, 10.0, 100.0], [0.0, 480.0, 960.0]);
    assert_eq!(
        s.pan_px(48.0).map(|s| s.get_domain()),
        Some([-1.0, 9.0, 91.0])
    );
    let zoomed = s.zoom_px(480.0, 2.0).map(|s| s.get_domain());
    assert_eq!(zoomed, Some([5.0, 10.0, 55.0]));
    let half = interact::Transform::IDENTITY
        .translate(-480.0, 0.0)
        .scale(2.0);
    assert_eq!(s.rescale_x(&half).get_domain(), [5.0, 10.0, 55.0]);
}

#[test]
fn the_identity_scale_moves_its_one_array() {
    let s = scale::identity().domain([0.0, 960.0]);
    let panned = s.pan_px(96.0).expect("a valid pan");
    assert_eq!(panned.get_domain(), [-96.0, 864.0]);
    assert_eq!(panned.get_range(), [-96.0, 864.0]);
    let zoomed = s.zoom_px(480.0, 2.0).map(|s| s.get_domain());
    assert_eq!(zoomed, Some([240.0, 720.0]));
    let half = interact::Transform::IDENTITY
        .translate(-480.0, 0.0)
        .scale(2.0);
    let rescaled = s.rescale_x(&half);
    assert_eq!(rescaled.get_domain(), [240.0, 720.0]);
    assert_eq!(rescaled.get_range(), [240.0, 720.0]);
    let down = interact::Transform::IDENTITY.translate(0.0, 60.0);
    assert_eq!(s.rescale_y(&down).get_domain(), [-60.0, 900.0]);
}

#[test]
fn any_continuous_rescales_as_its_transform() {
    let any: AnyContinuous = log().into();
    let zoomed = any.zoom_px(150.0, 2.0).map(|s| s.get_domain());
    assert_eq!(zoomed, log().zoom_px(150.0, 2.0).map(|s| s.get_domain()));
    let panned = any.pan_px(-30.0).map(|s| s.get_domain());
    assert_eq!(panned, log().pan_px(-30.0).map(|s| s.get_domain()));
    let half = t(2.0, -150.0, 0.0);
    assert_eq!(
        any.rescale_x(&half).get_domain(),
        log().rescale_x(&half).get_domain()
    );
}

#[test]
fn a_span_below_two_to_the_minus_forty_of_its_centre_is_rejected() {
    let unit = scale::linear().range([0.0, 960.0]);
    let deep = unit.zoom_px(480.0, 1e12).expect("a span of 1e-12");
    let [a, b] = deep.get_domain();
    // 1e-12 up to the rounding of 0.5 ± 5e-13.
    assert!((b - a - 1e-12).abs() < 1e-15, "{}", b - a);
    assert!(unit.zoom_px(480.0, 1e13).is_none());
    let identity = scale::identity();
    assert!(identity.zoom_px(0.5, 1e12).is_some());
    assert!(identity.zoom_px(0.5, 1e13).is_none());
}

#[test]
fn factors_that_are_not_finite_and_positive_are_rejected() {
    for factor in [-1.0, 0.0, -0.0, f64::INFINITY, f64::NAN] {
        assert!(x().zoom_px(480.0, factor).is_none(), "{factor}");
        assert!(scale::identity().zoom_px(0.5, factor).is_none(), "{factor}");
    }
    assert!(x().zoom_px(f64::NAN, 2.0).is_none());
}

#[test]
fn stops_that_are_not_finite_are_rejected() {
    for dx in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(x().pan_px(dx).is_none(), "{dx}");
        assert!(scale::identity().pan_px(dx).is_none(), "{dx}");
    }
    // e^(±3450): the far end overflows.
    assert!(log().zoom_px(150.0, 1e-3).is_none());
}

#[test]
fn stops_whose_transformed_value_is_not_finite_are_rejected() {
    // The near end underflows to 0, which log maps to −∞.
    assert!(log().zoom_px(300.0, 0.009).is_none());
    assert!(log().zoom_px(300.0, 0.01).is_some());
}

#[test]
fn stops_that_leave_the_domains_order_are_rejected() {
    let square = scale::func(f64::sqrt, |u| u * u, TickRule::Data)
        .domain([0.0, 100.0])
        .range([0.0, 100.0]);
    assert!(square.pan_px(60.0).is_none());
    assert!(square.pan_px(-60.0).is_some());
}

#[test]
fn flat_domains_and_ranges_cannot_move() {
    assert!(scale::linear().domain([5.0, 5.0]).pan_px(10.0).is_none());
    assert!(scale::linear().range([0.0, 0.0]).pan_px(10.0).is_none());
    assert!(scale::identity().domain([3.0, 3.0]).pan_px(1.0).is_none());
}
