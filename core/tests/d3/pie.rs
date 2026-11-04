//! d3's tests of d3-shape, as assertions on rustd3-core; each names d3's file, line and column.

#![allow(
    clippy::approx_constant,
    reason = "d3's tests write these constants as decimal literals"
)]

use rustd3_core::shape;

use super::support::{assert_path_eq, drawn, js_round, slices};

// d3-shape/test/pie-test.js: pie(data) returns arcs in input order
#[test]
fn pie_data_returns_arcs_in_input_order() {
    assert_eq!(
        slices(&shape::pie().layout(vec![1.0, 3.0, 2.0])),
        slices(&[
            shape::Slice {
                datum: 0,
                index: 2,
                value: 1.0,
                start_angle: 5.235987755982988,
                end_angle: 6.283185307179585,
                pad_angle: 0.0
            },
            shape::Slice {
                datum: 1,
                index: 0,
                value: 3.0,
                start_angle: 0.000000000000000,
                end_angle: 3.141592653589793,
                pad_angle: 0.0
            },
            shape::Slice {
                datum: 2,
                index: 1,
                value: 2.0,
                start_angle: 3.141592653589793,
                end_angle: 5.235987755982988,
                pad_angle: 0.0
            }
        ])
    ); // d3-shape/test/pie-test.js:19:3
}

// d3-shape/test/pie-test.js: pie(data) treats negative values as zero
#[test]
fn pie_data_treats_negative_values_as_zero() {
    assert_eq!(
        slices(&shape::pie().layout(vec![1.0, 0.0, -1.0])),
        slices(&[
            shape::Slice {
                datum: 0,
                index: 0,
                value: 1.0,
                start_angle: 0.000000000000000,
                end_angle: 6.283185307179586,
                pad_angle: 0.0
            },
            shape::Slice {
                datum: 1,
                index: 1,
                value: 0.0,
                start_angle: 6.283185307179586,
                end_angle: 6.283185307179586,
                pad_angle: 0.0
            },
            shape::Slice {
                datum: 2,
                index: 2,
                value: -1.0,
                start_angle: 6.283185307179586,
                end_angle: 6.283185307179586,
                pad_angle: 0.0
            }
        ])
    ); // d3-shape/test/pie-test.js:47:3
}

// d3-shape/test/pie-test.js: pie(data) puts everything at the startAngle when the sum is zero
#[test]
fn pie_data_puts_everything_at_the_startangle_when_the_sum_is_zero() {
    assert_eq!(
        slices(&shape::pie().layout(vec![0.0, 0.0])),
        slices(&[
            shape::Slice {
                datum: 0,
                index: 0,
                value: 0.0,
                start_angle: 0.0,
                end_angle: 0.0,
                pad_angle: 0.0
            },
            shape::Slice {
                datum: 1,
                index: 1,
                value: 0.0,
                start_angle: 0.0,
                end_angle: 0.0,
                pad_angle: 0.0
            }
        ])
    ); // d3-shape/test/pie-test.js:72:3
    assert_eq!(
        slices(&shape::pie().start_angle(1.0).layout(vec![0.0, 0.0])),
        slices(&[
            shape::Slice {
                datum: 0,
                index: 0,
                value: 0.0,
                start_angle: 1.0,
                end_angle: 1.0,
                pad_angle: 0.0
            },
            shape::Slice {
                datum: 1,
                index: 1,
                value: 0.0,
                start_angle: 1.0,
                end_angle: 1.0,
                pad_angle: 0.0
            }
        ])
    ); // d3-shape/test/pie-test.js:76:3
}

// d3-shape/test/pie-test.js: pie(data) restricts |endAngle - startAngle| to τ
#[test]
fn pie_data_restricts_endangle_startangle_to() {
    assert_eq!(
        slices(
            &shape::pie()
                .start_angle(0.0)
                .end_angle(7.0)
                .layout(vec![1.0, 2.0])
        ),
        slices(&[
            shape::Slice {
                datum: 0,
                index: 1,
                value: 1.0,
                start_angle: 4.1887902047863905,
                end_angle: 6.283185307179586,
                pad_angle: 0.0
            },
            shape::Slice {
                datum: 1,
                index: 0,
                value: 2.0,
                start_angle: 0.0000000000000000,
                end_angle: 4.1887902047863905,
                pad_angle: 0.0
            }
        ])
    ); // d3-shape/test/pie-test.js:84:3
    assert_eq!(
        slices(
            &shape::pie()
                .start_angle(0.0)
                .end_angle(7.0)
                .start_angle(7.0)
                .end_angle(0.0)
                .layout(vec![1.0, 2.0])
        ),
        slices(&[
            shape::Slice {
                datum: 0,
                index: 1,
                value: 1.0,
                start_angle: 2.8112097952136095,
                end_angle: 0.7168146928204142,
                pad_angle: 0.0
            },
            shape::Slice {
                datum: 1,
                index: 0,
                value: 2.0,
                start_angle: 7.0000000000000000,
                end_angle: 2.8112097952136095,
                pad_angle: 0.0
            }
        ])
    ); // d3-shape/test/pie-test.js:88:3
    assert_eq!(
        slices(
            &shape::pie()
                .start_angle(0.0)
                .end_angle(7.0)
                .start_angle(7.0)
                .end_angle(0.0)
                .start_angle(1.0)
                .end_angle(8.0)
                .layout(vec![1.0, 2.0])
        ),
        slices(&[
            shape::Slice {
                datum: 0,
                index: 1,
                value: 1.0,
                start_angle: 5.1887902047863905,
                end_angle: 7.283185307179586,
                pad_angle: 0.0
            },
            shape::Slice {
                datum: 1,
                index: 0,
                value: 2.0,
                start_angle: 1.0000000000000000,
                end_angle: 5.1887902047863905,
                pad_angle: 0.0
            }
        ])
    ); // d3-shape/test/pie-test.js:92:3
    assert_eq!(
        slices(
            &shape::pie()
                .start_angle(0.0)
                .end_angle(7.0)
                .start_angle(7.0)
                .end_angle(0.0)
                .start_angle(1.0)
                .end_angle(8.0)
                .start_angle(8.0)
                .end_angle(1.0)
                .layout(vec![1.0, 2.0])
        ),
        slices(&[
            shape::Slice {
                datum: 0,
                index: 1,
                value: 1.0,
                start_angle: 3.8112097952136095,
                end_angle: 1.7168146928204142,
                pad_angle: 0.0
            },
            shape::Slice {
                datum: 1,
                index: 0,
                value: 2.0,
                start_angle: 8.0000000000000000,
                end_angle: 3.8112097952136095,
                pad_angle: 0.0
            }
        ])
    ); // d3-shape/test/pie-test.js:96:3
}

// d3-shape/test/pie-test.js: pie().startAngle(θ)(data) observes the specified start angle
#[test]
fn pie_startangle_data_observes_the_specified_start_angle() {
    assert_eq!(
        slices(
            &shape::pie()
                .start_angle(std::f64::consts::PI)
                .layout(vec![1.0, 2.0, 3.0])
        ),
        slices(&[
            shape::Slice {
                datum: 0,
                index: 2,
                value: 1.0,
                start_angle: 5.759586531581287,
                end_angle: 6.283185307179586,
                pad_angle: 0.0
            },
            shape::Slice {
                datum: 1,
                index: 1,
                value: 2.0,
                start_angle: 4.71238898038469,
                end_angle: 5.759586531581287,
                pad_angle: 0.0
            },
            shape::Slice {
                datum: 2,
                index: 0,
                value: 3.0,
                start_angle: 3.141592653589793,
                end_angle: 4.71238898038469,
                pad_angle: 0.0
            }
        ])
    ); // d3-shape/test/pie-test.js:125:3
}

// d3-shape/test/pie-test.js: pie().endAngle(θ)(data) observes the specified end angle
#[test]
fn pie_endangle_data_observes_the_specified_end_angle() {
    assert_eq!(
        slices(
            &shape::pie()
                .end_angle(std::f64::consts::PI)
                .layout(vec![1.0, 2.0, 3.0])
        ),
        slices(&[
            shape::Slice {
                datum: 0,
                index: 2,
                value: 1.0,
                start_angle: 2.617993877991494,
                end_angle: 3.1415926535897927,
                pad_angle: 0.0
            },
            shape::Slice {
                datum: 1,
                index: 1,
                value: 2.0,
                start_angle: 1.5707963267948966,
                end_angle: 2.617993877991494,
                pad_angle: 0.0
            },
            shape::Slice {
                datum: 2,
                index: 0,
                value: 3.0,
                start_angle: 0.0000000000000000,
                end_angle: 1.5707963267948966,
                pad_angle: 0.0
            }
        ])
    ); // d3-shape/test/pie-test.js:133:3
}

// d3-shape/test/pie-test.js: pie().padAngle(δ)(data) observes the specified pad angle
#[test]
fn pie_padangle_data_observes_the_specified_pad_angle() {
    assert_eq!(
        slices(&shape::pie().pad_angle(0.1).layout(vec![1.0, 2.0, 3.0])),
        slices(&[
            shape::Slice {
                datum: 0,
                index: 2,
                value: 1.0,
                start_angle: 5.185987755982988,
                end_angle: 6.283185307179585,
                pad_angle: 0.1
            },
            shape::Slice {
                datum: 1,
                index: 1,
                value: 2.0,
                start_angle: 3.0915926535897933,
                end_angle: 5.185987755982988,
                pad_angle: 0.1
            },
            shape::Slice {
                datum: 2,
                index: 0,
                value: 3.0,
                start_angle: 0.0000000000000000,
                end_angle: 3.0915926535897933,
                pad_angle: 0.1
            }
        ])
    ); // d3-shape/test/pie-test.js:141:3
}

// d3-shape/test/pie-test.js: pie().startAngle(θ₀).endAngle(θ₁).padAngle(δ)(data) restricts the pad angle to |θ₁ - θ₀| / data.length
#[test]
fn pie_startangle_endangle_padangle_data_restricts_the_pad_angle_to_data_length() {
    assert_eq!(
        slices(
            &shape::pie()
                .start_angle(0.0)
                .end_angle(std::f64::consts::PI)
                .pad_angle(f64::INFINITY)
                .layout(vec![1.0, 2.0, 3.0])
        ),
        slices(&[
            shape::Slice {
                datum: 0,
                index: 2,
                value: 1.0,
                start_angle: 2.0943951023931953,
                end_angle: 3.141592653589793,
                pad_angle: 1.0471975511965976
            },
            shape::Slice {
                datum: 1,
                index: 1,
                value: 2.0,
                start_angle: 1.0471975511965976,
                end_angle: 2.0943951023931953,
                pad_angle: 1.0471975511965976
            },
            shape::Slice {
                datum: 2,
                index: 0,
                value: 3.0,
                start_angle: 0.0000000000000000,
                end_angle: 1.0471975511965976,
                pad_angle: 1.0471975511965976
            }
        ])
    ); // d3-shape/test/pie-test.js:163:3
    assert_eq!(
        slices(
            &shape::pie()
                .start_angle(0.0)
                .end_angle(-std::f64::consts::PI)
                .pad_angle(f64::INFINITY)
                .layout(vec![1.0, 2.0, 3.0])
        ),
        slices(&[
            shape::Slice {
                datum: 0,
                index: 2,
                value: 1.0,
                start_angle: -2.0943951023931953,
                end_angle: -3.141592653589793,
                pad_angle: 1.0471975511965976
            },
            shape::Slice {
                datum: 1,
                index: 1,
                value: 2.0,
                start_angle: -1.0471975511965976,
                end_angle: -2.0943951023931953,
                pad_angle: 1.0471975511965976
            },
            shape::Slice {
                datum: 2,
                index: 0,
                value: 3.0,
                start_angle: 0.0000000000000000,
                end_angle: -1.0471975511965976,
                pad_angle: 1.0471975511965976
            }
        ])
    ); // d3-shape/test/pie-test.js:168:3
}

// d3-shape/test/pie-test.js: pie.sortValues(f) sorts arcs by value per the specified comparator function f
#[test]
fn pie_sortvalues_f_sorts_arcs_by_value_per_the_specified_comparator_function_f() {
    assert_eq!(
        slices(
            &shape::pie()
                .sort(shape::pie::Sort::Ascending)
                .layout(vec![1.0, 3.0, 2.0])
        ),
        slices(&[
            shape::Slice {
                datum: 0,
                index: 0,
                value: 1.0,
                start_angle: 0.0000000000000000,
                end_angle: 1.0471975511965976,
                pad_angle: 0.0
            },
            shape::Slice {
                datum: 1,
                index: 2,
                value: 3.0,
                start_angle: 3.141592653589793,
                end_angle: 6.283185307179586,
                pad_angle: 0.0
            },
            shape::Slice {
                datum: 2,
                index: 1,
                value: 2.0,
                start_angle: 1.0471975511965976,
                end_angle: 3.141592653589793,
                pad_angle: 0.0
            }
        ])
    ); // d3-shape/test/pie-test.js:177:3
    assert_eq!(
        slices(
            &shape::pie()
                .sort(shape::pie::Sort::Ascending)
                .sort(shape::pie::Sort::Descending)
                .layout(vec![1.0, 3.0, 2.0])
        ),
        slices(&[
            shape::Slice {
                datum: 0,
                index: 2,
                value: 1.0,
                start_angle: 5.235987755982988,
                end_angle: 6.283185307179585,
                pad_angle: 0.0
            },
            shape::Slice {
                datum: 1,
                index: 0,
                value: 3.0,
                start_angle: 0.0000000000000000,
                end_angle: 3.141592653589793,
                pad_angle: 0.0
            },
            shape::Slice {
                datum: 2,
                index: 1,
                value: 2.0,
                start_angle: 3.141592653589793,
                end_angle: 5.235987755982988,
                pad_angle: 0.0
            }
        ])
    ); // d3-shape/test/pie-test.js:182:3
}

// d3-shape/test/arc-test.js: arc().centroid(…) computes the midpoint of the center line of the arc
#[test]
fn arc_centroid_computes_the_midpoint_of_the_center_line_of_the_arc() {
    let round = |x: f64| js_round(x * 1e6) / 1e6;
    assert_eq!(
        {
            let (x, y) =
                shape::arc()
                    .inner_radius(0.0)
                    .outer_radius(100.0)
                    .centroid(&shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: std::f64::consts::PI,
                        pad_angle: 0.0,
                    });
            vec![x, y]
        }
        .into_iter()
        .map(round)
        .collect::<Vec<f64>>(),
        vec![50.0, 0.0]
    ); // d3-shape/test/arc-test.js:56:3
    assert_eq!(
        {
            let (x, y) =
                shape::arc()
                    .inner_radius(0.0)
                    .outer_radius(100.0)
                    .centroid(&shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: (std::f64::consts::PI / 2.0),
                        pad_angle: 0.0,
                    });
            vec![x, y]
        }
        .into_iter()
        .map(round)
        .collect::<Vec<f64>>(),
        vec![35.355339, -35.355339]
    ); // d3-shape/test/arc-test.js:57:3
    assert_eq!(
        {
            let (x, y) = shape::arc()
                .inner_radius(50.0)
                .outer_radius(100.0)
                .centroid(&shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: 0.0,
                    end_angle: -std::f64::consts::PI,
                    pad_angle: 0.0,
                });
            vec![x, y]
        }
        .into_iter()
        .map(round)
        .collect::<Vec<f64>>(),
        vec![-75.0, -0.0]
    ); // d3-shape/test/arc-test.js:58:3
    assert_eq!(
        {
            let (x, y) = shape::arc()
                .inner_radius(50.0)
                .outer_radius(100.0)
                .centroid(&shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: 0.0,
                    end_angle: (-std::f64::consts::PI / 2.0),
                    pad_angle: 0.0,
                });
            vec![x, y]
        }
        .into_iter()
        .map(round)
        .collect::<Vec<f64>>(),
        vec![-53.033009, -53.033009]
    ); // d3-shape/test/arc-test.js:59:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(0).outerRadius(0) renders a point
#[test]
fn arc_innerradius_0_outerradius_0_renders_a_point() {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(0.0).outer_radius(0.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: 0.0,
                    end_angle: (2.0 * std::f64::consts::PI),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,0Z",
    ); // d3-shape/test/arc-test.js:92:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(0.0).outer_radius(0.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: 0.0,
                    end_angle: 0.0,
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,0Z",
    ); // d3-shape/test/arc-test.js:93:3
}

// d3-shape/test/arc-test.js: a negative angle span proceeds anticlockwise
#[test]
fn a_negative_angle_span_proceeds_anticlockwise() {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(0.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: 0.0,
                    end_angle: (-std::f64::consts::PI / 2.0),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,0,0,-100,0L0,0Z",
    ); // d3-shape/test/arc-test.js:98:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(0).outerRadius(r).startAngle(θ₀).endAngle(θ₁) renders a clockwise circle if r > 0 and θ₁ - θ₀ ≥ τ
#[test]
fn arc_innerradius_0_outerradius_r_startangle_endangle_renders_a_clockwise_circle_if_r_0_and() {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(0.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: 0.0,
                    end_angle: (2.0 * std::f64::consts::PI),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,1,1,0,100A100,100,0,1,1,0,-100Z",
    ); // d3-shape/test/arc-test.js:103:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(0.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: 0.0,
                    end_angle: (3.0 * std::f64::consts::PI),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,1,1,0,100A100,100,0,1,1,0,-100Z",
    ); // d3-shape/test/arc-test.js:104:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(0.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: (-2.0 * std::f64::consts::PI),
                    end_angle: 0.0,
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,1,1,0,100A100,100,0,1,1,0,-100Z",
    ); // d3-shape/test/arc-test.js:105:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(0.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: -std::f64::consts::PI,
                    end_angle: std::f64::consts::PI,
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,100A100,100,0,1,1,0,-100A100,100,0,1,1,0,100Z",
    ); // d3-shape/test/arc-test.js:106:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(0.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: (-3.0 * std::f64::consts::PI),
                    end_angle: 0.0,
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,100A100,100,0,1,1,0,-100A100,100,0,1,1,0,100Z",
    ); // d3-shape/test/arc-test.js:107:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(0).outerRadius(r).startAngle(θ₀).endAngle(θ₁) renders an anticlockwise circle if r > 0 and θ₀ - θ₁ ≥ τ
#[test]
fn arc_innerradius_0_outerradius_r_startangle_endangle_renders_an_anticlockwise_circle_if_r_0_and()
{
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(0.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: 0.0,
                    end_angle: (-2.0 * std::f64::consts::PI),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,1,0,0,100A100,100,0,1,0,0,-100Z",
    ); // d3-shape/test/arc-test.js:112:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(0.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: 0.0,
                    end_angle: (-3.0 * std::f64::consts::PI),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,1,0,0,100A100,100,0,1,0,0,-100Z",
    ); // d3-shape/test/arc-test.js:113:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(0.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: (2.0 * std::f64::consts::PI),
                    end_angle: 0.0,
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,1,0,0,100A100,100,0,1,0,0,-100Z",
    ); // d3-shape/test/arc-test.js:114:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(0.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: std::f64::consts::PI,
                    end_angle: -std::f64::consts::PI,
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,100A100,100,0,1,0,0,-100A100,100,0,1,0,0,100Z",
    ); // d3-shape/test/arc-test.js:115:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(0.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: (3.0 * std::f64::consts::PI),
                    end_angle: 0.0,
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,100A100,100,0,1,0,0,-100A100,100,0,1,0,0,100Z",
    ); // d3-shape/test/arc-test.js:116:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(r₀).outerRadius(r₁).startAngle(θ₀).endAngle(θ₁) renders a clockwise annulus if r₀ > 0, r₁ > 0 and θ₀ - θ₁ ≥ τ
#[test]
fn arc_innerradius_r_outerradius_r_startangle_endangle_renders_a_clockwise_annulus_if_r_0_r_0_and()
{
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(50.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: 0.0,
                    end_angle: (2.0 * std::f64::consts::PI),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,1,1,0,100A100,100,0,1,1,0,-100M0,-50A50,50,0,1,0,0,50A50,50,0,1,0,0,-50Z",
    ); // d3-shape/test/arc-test.js:123:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(50.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: 0.0,
                    end_angle: (3.0 * std::f64::consts::PI),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,1,1,0,100A100,100,0,1,1,0,-100M0,50A50,50,0,1,0,0,-50A50,50,0,1,0,0,50Z",
    ); // d3-shape/test/arc-test.js:124:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(50.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: (-2.0 * std::f64::consts::PI),
                    end_angle: 0.0,
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,1,1,0,100A100,100,0,1,1,0,-100M0,-50A50,50,0,1,0,0,50A50,50,0,1,0,0,-50Z",
    ); // d3-shape/test/arc-test.js:125:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(50.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: -std::f64::consts::PI,
                    end_angle: std::f64::consts::PI,
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,100A100,100,0,1,1,0,-100A100,100,0,1,1,0,100M0,50A50,50,0,1,0,0,-50A50,50,0,1,0,0,50Z",
    ); // d3-shape/test/arc-test.js:126:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(50.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: (-3.0 * std::f64::consts::PI),
                    end_angle: 0.0,
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,100A100,100,0,1,1,0,-100A100,100,0,1,1,0,100M0,-50A50,50,0,1,0,0,50A50,50,0,1,0,0,-50Z",
    ); // d3-shape/test/arc-test.js:127:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(r₀).outerRadius(r₁).startAngle(θ₀).endAngle(θ₁) renders an anticlockwise annulus if r₀ > 0, r₁ > 0 and θ₁ - θ₀ ≥ τ
#[test]
fn arc_innerradius_r_outerradius_r_startangle_endangle_renders_an_anticlockwise_annulus_if_r_0_r_0_and()
 {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(50.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: 0.0,
                    end_angle: (-2.0 * std::f64::consts::PI),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,1,0,0,100A100,100,0,1,0,0,-100M0,-50A50,50,0,1,1,0,50A50,50,0,1,1,0,-50Z",
    ); // d3-shape/test/arc-test.js:134:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(50.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: 0.0,
                    end_angle: (-3.0 * std::f64::consts::PI),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,1,0,0,100A100,100,0,1,0,0,-100M0,50A50,50,0,1,1,0,-50A50,50,0,1,1,0,50Z",
    ); // d3-shape/test/arc-test.js:135:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(50.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: (2.0 * std::f64::consts::PI),
                    end_angle: 0.0,
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,1,0,0,100A100,100,0,1,0,0,-100M0,-50A50,50,0,1,1,0,50A50,50,0,1,1,0,-50Z",
    ); // d3-shape/test/arc-test.js:136:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(50.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: std::f64::consts::PI,
                    end_angle: -std::f64::consts::PI,
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,100A100,100,0,1,0,0,-100A100,100,0,1,0,0,100M0,50A50,50,0,1,1,0,-50A50,50,0,1,1,0,50Z",
    ); // d3-shape/test/arc-test.js:137:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(50.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: (3.0 * std::f64::consts::PI),
                    end_angle: 0.0,
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,100A100,100,0,1,0,0,-100A100,100,0,1,0,0,100M0,-50A50,50,0,1,1,0,50A50,50,0,1,1,0,-50Z",
    ); // d3-shape/test/arc-test.js:138:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(0).outerRadius(r).startAngle(θ₀).endAngle(θ₁) renders a small clockwise sector if r > 0 and π > θ₁ - θ₀ ≥ 0
#[test]
fn arc_innerradius_0_outerradius_r_startangle_endangle_renders_a_small_clockwise_sector_if_r_0_and_0()
 {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(0.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: 0.0,
                    end_angle: (std::f64::consts::PI / 2.0),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,0,1,100,0L0,0Z",
    ); // d3-shape/test/arc-test.js:143:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(0.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: (2.0 * std::f64::consts::PI),
                    end_angle: ((5.0 * std::f64::consts::PI) / 2.0),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,0,1,100,0L0,0Z",
    ); // d3-shape/test/arc-test.js:144:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(0.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: -std::f64::consts::PI,
                    end_angle: (-std::f64::consts::PI / 2.0),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,100A100,100,0,0,1,-100,0L0,0Z",
    ); // d3-shape/test/arc-test.js:145:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(0).outerRadius(r).startAngle(θ₀).endAngle(θ₁) renders a small anticlockwise sector if r > 0 and π > θ₀ - θ₁ ≥ 0
#[test]
fn arc_innerradius_0_outerradius_r_startangle_endangle_renders_a_small_anticlockwise_sector_if_r_0_and_0()
 {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(0.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: 0.0,
                    end_angle: (-std::f64::consts::PI / 2.0),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,0,0,-100,0L0,0Z",
    ); // d3-shape/test/arc-test.js:150:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(0.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: (-2.0 * std::f64::consts::PI),
                    end_angle: ((-5.0 * std::f64::consts::PI) / 2.0),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,0,0,-100,0L0,0Z",
    ); // d3-shape/test/arc-test.js:151:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(0.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: std::f64::consts::PI,
                    end_angle: (std::f64::consts::PI / 2.0),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,100A100,100,0,0,0,100,0L0,0Z",
    ); // d3-shape/test/arc-test.js:152:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(0).outerRadius(r).startAngle(θ₀).endAngle(θ₁) renders a large clockwise sector if r > 0 and τ > θ₁ - θ₀ ≥ π
#[test]
fn arc_innerradius_0_outerradius_r_startangle_endangle_renders_a_large_clockwise_sector_if_r_0_and()
{
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(0.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: 0.0,
                    end_angle: ((3.0 * std::f64::consts::PI) / 2.0),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,1,1,-100,0L0,0Z",
    ); // d3-shape/test/arc-test.js:157:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(0.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: (2.0 * std::f64::consts::PI),
                    end_angle: ((7.0 * std::f64::consts::PI) / 2.0),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,1,1,-100,0L0,0Z",
    ); // d3-shape/test/arc-test.js:158:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(0.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: -std::f64::consts::PI,
                    end_angle: (std::f64::consts::PI / 2.0),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,100A100,100,0,1,1,100,0L0,0Z",
    ); // d3-shape/test/arc-test.js:159:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(0).outerRadius(r).startAngle(θ₀).endAngle(θ₁) renders a large anticlockwise sector if r > 0 and τ > θ₀ - θ₁ ≥ π
#[test]
fn arc_innerradius_0_outerradius_r_startangle_endangle_renders_a_large_anticlockwise_sector_if_r_0_and()
 {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(0.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: 0.0,
                    end_angle: ((-3.0 * std::f64::consts::PI) / 2.0),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,1,0,100,0L0,0Z",
    ); // d3-shape/test/arc-test.js:164:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(0.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: (-2.0 * std::f64::consts::PI),
                    end_angle: ((-7.0 * std::f64::consts::PI) / 2.0),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,1,0,100,0L0,0Z",
    ); // d3-shape/test/arc-test.js:165:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(0.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: std::f64::consts::PI,
                    end_angle: (-std::f64::consts::PI / 2.0),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,100A100,100,0,1,0,-100,0L0,0Z",
    ); // d3-shape/test/arc-test.js:166:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(r₀).outerRadius(r₁).startAngle(θ₀).endAngle(θ₁) renders a small clockwise annular sector if r₀ > 0, r₁ > 0 and π > θ₁ - θ₀ ≥ 0
#[test]
fn arc_innerradius_r_outerradius_r_startangle_endangle_renders_a_small_clockwise_annular_sector_if_r_0_r_0_and_0()
 {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(50.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: 0.0,
                    end_angle: (std::f64::consts::PI / 2.0),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,0,1,100,0L50,0A50,50,0,0,0,0,-50Z",
    ); // d3-shape/test/arc-test.js:172:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(50.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: (2.0 * std::f64::consts::PI),
                    end_angle: ((5.0 * std::f64::consts::PI) / 2.0),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,0,1,100,0L50,0A50,50,0,0,0,0,-50Z",
    ); // d3-shape/test/arc-test.js:173:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(50.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: -std::f64::consts::PI,
                    end_angle: (-std::f64::consts::PI / 2.0),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,100A100,100,0,0,1,-100,0L-50,0A50,50,0,0,0,0,50Z",
    ); // d3-shape/test/arc-test.js:174:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(r₀).outerRadius(r₁).startAngle(θ₀).endAngle(θ₁) renders a small anticlockwise annular sector if r₀ > 0, r₁ > 0 and π > θ₀ - θ₁ ≥ 0
#[test]
fn arc_innerradius_r_outerradius_r_startangle_endangle_renders_a_small_anticlockwise_annular_sector_if_r_0_r_0_and_0()
 {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(50.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: 0.0,
                    end_angle: (-std::f64::consts::PI / 2.0),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,0,0,-100,0L-50,0A50,50,0,0,1,0,-50Z",
    ); // d3-shape/test/arc-test.js:180:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(50.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: (-2.0 * std::f64::consts::PI),
                    end_angle: ((-5.0 * std::f64::consts::PI) / 2.0),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,0,0,-100,0L-50,0A50,50,0,0,1,0,-50Z",
    ); // d3-shape/test/arc-test.js:181:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(50.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: std::f64::consts::PI,
                    end_angle: (std::f64::consts::PI / 2.0),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,100A100,100,0,0,0,100,0L50,0A50,50,0,0,1,0,50Z",
    ); // d3-shape/test/arc-test.js:182:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(r₀).outerRadius(r₁).startAngle(θ₀).endAngle(θ₁) renders a large clockwise annular sector if r₀ > 0, r₁ > 0 and τ > θ₁ - θ₀ ≥ π
#[test]
fn arc_innerradius_r_outerradius_r_startangle_endangle_renders_a_large_clockwise_annular_sector_if_r_0_r_0_and()
 {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(50.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: 0.0,
                    end_angle: ((3.0 * std::f64::consts::PI) / 2.0),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,1,1,-100,0L-50,0A50,50,0,1,0,0,-50Z",
    ); // d3-shape/test/arc-test.js:188:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(50.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: (2.0 * std::f64::consts::PI),
                    end_angle: ((7.0 * std::f64::consts::PI) / 2.0),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,1,1,-100,0L-50,0A50,50,0,1,0,0,-50Z",
    ); // d3-shape/test/arc-test.js:189:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(50.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: -std::f64::consts::PI,
                    end_angle: (std::f64::consts::PI / 2.0),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,100A100,100,0,1,1,100,0L50,0A50,50,0,1,0,0,50Z",
    ); // d3-shape/test/arc-test.js:190:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(r₀).outerRadius(r₁).startAngle(θ₀).endAngle(θ₁) renders a large anticlockwise annular sector if r₀ > 0, r₁ > 0 and τ > θ₀ - θ₁ ≥ π
#[test]
fn arc_innerradius_r_outerradius_r_startangle_endangle_renders_a_large_anticlockwise_annular_sector_if_r_0_r_0_and()
 {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(50.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: 0.0,
                    end_angle: ((-3.0 * std::f64::consts::PI) / 2.0),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,1,0,100,0L50,0A50,50,0,1,1,0,-50Z",
    ); // d3-shape/test/arc-test.js:196:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(50.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: (-2.0 * std::f64::consts::PI),
                    end_angle: ((-7.0 * std::f64::consts::PI) / 2.0),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,-100A100,100,0,1,0,100,0L50,0A50,50,0,1,1,0,-50Z",
    ); // d3-shape/test/arc-test.js:197:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc().inner_radius(50.0).outer_radius(100.0).draw(
                &shape::Slice {
                    datum: 0,
                    index: 0,
                    value: 0.0,
                    start_angle: std::f64::consts::PI,
                    end_angle: (-std::f64::consts::PI / 2.0),
                    pad_angle: 0.0,
                },
                ctx,
            )
        }),
        &"M0,100A100,100,0,1,0,-100,0L-50,0A50,50,0,1,1,0,50Z",
    ); // d3-shape/test/arc-test.js:198:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(0).outerRadius(0).cornerRadius(r) renders a point
#[test]
fn arc_innerradius_0_outerradius_0_cornerradius_r_renders_a_point() {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(0.0)
                .outer_radius(0.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: (2.0 * std::f64::consts::PI),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,0Z",
    ); // d3-shape/test/arc-test.js:203:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(0.0)
                .outer_radius(0.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: 0.0,
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,0Z",
    ); // d3-shape/test/arc-test.js:204:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(0).outerRadius(r).startAngle(θ₀).endAngle(θ₁).cornerRadius(rᵧ) renders a clockwise circle if r > 0 and θ₁ - θ₀ ≥ τ
#[test]
fn arc_innerradius_0_outerradius_r_startangle_endangle_cornerradius_r_renders_a_clockwise_circle_if_r_0_and()
 {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(0.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: (2.0 * std::f64::consts::PI),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-100A100,100,0,1,1,0,100A100,100,0,1,1,0,-100Z",
    ); // d3-shape/test/arc-test.js:209:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(0.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: (3.0 * std::f64::consts::PI),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-100A100,100,0,1,1,0,100A100,100,0,1,1,0,-100Z",
    ); // d3-shape/test/arc-test.js:210:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(0.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: (-2.0 * std::f64::consts::PI),
                        end_angle: 0.0,
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-100A100,100,0,1,1,0,100A100,100,0,1,1,0,-100Z",
    ); // d3-shape/test/arc-test.js:211:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(0.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: -std::f64::consts::PI,
                        end_angle: std::f64::consts::PI,
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,100A100,100,0,1,1,0,-100A100,100,0,1,1,0,100Z",
    ); // d3-shape/test/arc-test.js:212:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(0.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: (-3.0 * std::f64::consts::PI),
                        end_angle: 0.0,
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,100A100,100,0,1,1,0,-100A100,100,0,1,1,0,100Z",
    ); // d3-shape/test/arc-test.js:213:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(0).outerRadius(r).startAngle(θ₀).endAngle(θ₁).cornerRadius(rᵧ) renders an anticlockwise circle if r > 0 and θ₀ - θ₁ ≥ τ
#[test]
fn arc_innerradius_0_outerradius_r_startangle_endangle_cornerradius_r_renders_an_anticlockwise_circle_if_r_0_and()
 {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(0.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: (-2.0 * std::f64::consts::PI),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-100A100,100,0,1,0,0,100A100,100,0,1,0,0,-100Z",
    ); // d3-shape/test/arc-test.js:218:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(0.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: (-3.0 * std::f64::consts::PI),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-100A100,100,0,1,0,0,100A100,100,0,1,0,0,-100Z",
    ); // d3-shape/test/arc-test.js:219:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(0.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: (2.0 * std::f64::consts::PI),
                        end_angle: 0.0,
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-100A100,100,0,1,0,0,100A100,100,0,1,0,0,-100Z",
    ); // d3-shape/test/arc-test.js:220:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(0.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: std::f64::consts::PI,
                        end_angle: -std::f64::consts::PI,
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,100A100,100,0,1,0,0,-100A100,100,0,1,0,0,100Z",
    ); // d3-shape/test/arc-test.js:221:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(0.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: (3.0 * std::f64::consts::PI),
                        end_angle: 0.0,
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,100A100,100,0,1,0,0,-100A100,100,0,1,0,0,100Z",
    ); // d3-shape/test/arc-test.js:222:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(r₀).outerRadius(r₁).startAngle(θ₀).endAngle(θ₁).cornerRadius(rᵧ) renders a clockwise annulus if r₀ > 0, r₁ > 0 and θ₀ - θ₁ ≥ τ
#[test]
fn arc_innerradius_r_outerradius_r_startangle_endangle_cornerradius_r_renders_a_clockwise_annulus_if_r_0_r_0_and()
 {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(50.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: (2.0 * std::f64::consts::PI),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-100A100,100,0,1,1,0,100A100,100,0,1,1,0,-100M0,-50A50,50,0,1,0,0,50A50,50,0,1,0,0,-50Z",
    ); // d3-shape/test/arc-test.js:229:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(50.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: (3.0 * std::f64::consts::PI),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-100A100,100,0,1,1,0,100A100,100,0,1,1,0,-100M0,50A50,50,0,1,0,0,-50A50,50,0,1,0,0,50Z",
    ); // d3-shape/test/arc-test.js:230:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(50.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: (-2.0 * std::f64::consts::PI),
                        end_angle: 0.0,
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-100A100,100,0,1,1,0,100A100,100,0,1,1,0,-100M0,-50A50,50,0,1,0,0,50A50,50,0,1,0,0,-50Z",
    ); // d3-shape/test/arc-test.js:231:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(50.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: -std::f64::consts::PI,
                        end_angle: std::f64::consts::PI,
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,100A100,100,0,1,1,0,-100A100,100,0,1,1,0,100M0,50A50,50,0,1,0,0,-50A50,50,0,1,0,0,50Z",
    ); // d3-shape/test/arc-test.js:232:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(50.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: (-3.0 * std::f64::consts::PI),
                        end_angle: 0.0,
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,100A100,100,0,1,1,0,-100A100,100,0,1,1,0,100M0,-50A50,50,0,1,0,0,50A50,50,0,1,0,0,-50Z",
    ); // d3-shape/test/arc-test.js:233:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(r₀).outerRadius(r₁).startAngle(θ₀).endAngle(θ₁).cornerRadius(rᵧ) renders an anticlockwise annulus if r₀ > 0, r₁ > 0 and θ₁ - θ₀ ≥ τ
#[test]
fn arc_innerradius_r_outerradius_r_startangle_endangle_cornerradius_r_renders_an_anticlockwise_annulus_if_r_0_r_0_and()
 {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(50.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: (-2.0 * std::f64::consts::PI),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-100A100,100,0,1,0,0,100A100,100,0,1,0,0,-100M0,-50A50,50,0,1,1,0,50A50,50,0,1,1,0,-50Z",
    ); // d3-shape/test/arc-test.js:240:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(50.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: (-3.0 * std::f64::consts::PI),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-100A100,100,0,1,0,0,100A100,100,0,1,0,0,-100M0,50A50,50,0,1,1,0,-50A50,50,0,1,1,0,50Z",
    ); // d3-shape/test/arc-test.js:241:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(50.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: (2.0 * std::f64::consts::PI),
                        end_angle: 0.0,
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-100A100,100,0,1,0,0,100A100,100,0,1,0,0,-100M0,-50A50,50,0,1,1,0,50A50,50,0,1,1,0,-50Z",
    ); // d3-shape/test/arc-test.js:242:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(50.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: std::f64::consts::PI,
                        end_angle: -std::f64::consts::PI,
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,100A100,100,0,1,0,0,-100A100,100,0,1,0,0,100M0,50A50,50,0,1,1,0,-50A50,50,0,1,1,0,50Z",
    ); // d3-shape/test/arc-test.js:243:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(50.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: (3.0 * std::f64::consts::PI),
                        end_angle: 0.0,
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,100A100,100,0,1,0,0,-100A100,100,0,1,0,0,100M0,-50A50,50,0,1,1,0,50A50,50,0,1,1,0,-50Z",
    ); // d3-shape/test/arc-test.js:244:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(0).outerRadius(r).startAngle(θ₀).endAngle(θ₁).cornerRadius(rᵧ) renders a small clockwise sector if r > 0 and π > θ₁ - θ₀ ≥ 0
#[test]
fn arc_innerradius_0_outerradius_r_startangle_endangle_cornerradius_r_renders_a_small_clockwise_sector_if_r_0_and_0()
 {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(0.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: (std::f64::consts::PI / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-94.868330A5,5,0,0,1,5.263158,-99.861400A100,100,0,0,1,99.861400,-5.263158A5,5,0,0,1,94.868330,0L0,0Z",
    ); // d3-shape/test/arc-test.js:249:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(0.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: (2.0 * std::f64::consts::PI),
                        end_angle: ((5.0 * std::f64::consts::PI) / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-94.868330A5,5,0,0,1,5.263158,-99.861400A100,100,0,0,1,99.861400,-5.263158A5,5,0,0,1,94.868330,0L0,0Z",
    ); // d3-shape/test/arc-test.js:250:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(0.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: -std::f64::consts::PI,
                        end_angle: (-std::f64::consts::PI / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,94.868330A5,5,0,0,1,-5.263158,99.861400A100,100,0,0,1,-99.861400,5.263158A5,5,0,0,1,-94.868330,0L0,0Z",
    ); // d3-shape/test/arc-test.js:251:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(0).outerRadius(r).startAngle(θ₀).endAngle(θ₁).cornerRadius(rᵧ) renders a small anticlockwise sector if r > 0 and π > θ₀ - θ₁ ≥ 0
#[test]
fn arc_innerradius_0_outerradius_r_startangle_endangle_cornerradius_r_renders_a_small_anticlockwise_sector_if_r_0_and_0()
 {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(0.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: (-std::f64::consts::PI / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-94.868330A5,5,0,0,0,-5.263158,-99.861400A100,100,0,0,0,-99.861400,-5.263158A5,5,0,0,0,-94.868330,0L0,0Z",
    ); // d3-shape/test/arc-test.js:256:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(0.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: (-2.0 * std::f64::consts::PI),
                        end_angle: ((-5.0 * std::f64::consts::PI) / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-94.868330A5,5,0,0,0,-5.263158,-99.861400A100,100,0,0,0,-99.861400,-5.263158A5,5,0,0,0,-94.868330,0L0,0Z",
    ); // d3-shape/test/arc-test.js:257:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(0.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: std::f64::consts::PI,
                        end_angle: (std::f64::consts::PI / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,94.868330A5,5,0,0,0,5.263158,99.861400A100,100,0,0,0,99.861400,5.263158A5,5,0,0,0,94.868330,0L0,0Z",
    ); // d3-shape/test/arc-test.js:258:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(0).outerRadius(r).startAngle(θ₀).endAngle(θ₁).cornerRadius(rᵧ) renders a large clockwise sector if r > 0 and τ > θ₁ - θ₀ ≥ π
#[test]
fn arc_innerradius_0_outerradius_r_startangle_endangle_cornerradius_r_renders_a_large_clockwise_sector_if_r_0_and()
 {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(0.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: ((3.0 * std::f64::consts::PI) / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-94.868330A5,5,0,0,1,5.263158,-99.861400A100,100,0,1,1,-99.861400,5.263158A5,5,0,0,1,-94.868330,0L0,0Z",
    ); // d3-shape/test/arc-test.js:263:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(0.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: (2.0 * std::f64::consts::PI),
                        end_angle: ((7.0 * std::f64::consts::PI) / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-94.868330A5,5,0,0,1,5.263158,-99.861400A100,100,0,1,1,-99.861400,5.263158A5,5,0,0,1,-94.868330,0L0,0Z",
    ); // d3-shape/test/arc-test.js:264:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(0.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: -std::f64::consts::PI,
                        end_angle: (std::f64::consts::PI / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,94.868330A5,5,0,0,1,-5.263158,99.861400A100,100,0,1,1,99.861400,-5.263158A5,5,0,0,1,94.868330,0L0,0Z",
    ); // d3-shape/test/arc-test.js:265:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(0).outerRadius(r).startAngle(θ₀).endAngle(θ₁).cornerRadius(rᵧ) renders a large anticlockwise sector if r > 0 and τ > θ₀ - θ₁ ≥ π
#[test]
fn arc_innerradius_0_outerradius_r_startangle_endangle_cornerradius_r_renders_a_large_anticlockwise_sector_if_r_0_and()
 {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(0.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: ((-3.0 * std::f64::consts::PI) / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-94.868330A5,5,0,0,0,-5.263158,-99.861400A100,100,0,1,0,99.861400,5.263158A5,5,0,0,0,94.868330,0L0,0Z",
    ); // d3-shape/test/arc-test.js:270:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(0.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: (-2.0 * std::f64::consts::PI),
                        end_angle: ((-7.0 * std::f64::consts::PI) / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-94.868330A5,5,0,0,0,-5.263158,-99.861400A100,100,0,1,0,99.861400,5.263158A5,5,0,0,0,94.868330,0L0,0Z",
    ); // d3-shape/test/arc-test.js:271:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(0.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: std::f64::consts::PI,
                        end_angle: (-std::f64::consts::PI / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,94.868330A5,5,0,0,0,5.263158,99.861400A100,100,0,1,0,-99.861400,-5.263158A5,5,0,0,0,-94.868330,0L0,0Z",
    ); // d3-shape/test/arc-test.js:272:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(r₀).outerRadius(r₁).startAngle(θ₀).endAngle(θ₁).cornerRadius(rᵧ) renders a small clockwise annular sector if r₀ > 0, r₁ > 0 and π > θ₁ - θ₀ ≥ 0
#[test]
fn arc_innerradius_r_outerradius_r_startangle_endangle_cornerradius_r_renders_a_small_clockwise_annular_sector_if_r_0_r_0_and_0()
 {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(50.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: (std::f64::consts::PI / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-94.868330A5,5,0,0,1,5.263158,-99.861400A100,100,0,0,1,99.861400,-5.263158A5,5,0,0,1,94.868330,0L54.772256,0A5,5,0,0,1,49.792960,-4.545455A50,50,0,0,0,4.545455,-49.792960A5,5,0,0,1,0,-54.772256Z",
    ); // d3-shape/test/arc-test.js:278:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(50.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: (2.0 * std::f64::consts::PI),
                        end_angle: ((5.0 * std::f64::consts::PI) / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-94.868330A5,5,0,0,1,5.263158,-99.861400A100,100,0,0,1,99.861400,-5.263158A5,5,0,0,1,94.868330,0L54.772256,0A5,5,0,0,1,49.792960,-4.545455A50,50,0,0,0,4.545455,-49.792960A5,5,0,0,1,0,-54.772256Z",
    ); // d3-shape/test/arc-test.js:279:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(50.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: -std::f64::consts::PI,
                        end_angle: (-std::f64::consts::PI / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,94.868330A5,5,0,0,1,-5.263158,99.861400A100,100,0,0,1,-99.861400,5.263158A5,5,0,0,1,-94.868330,0L-54.772256,0A5,5,0,0,1,-49.792960,4.545455A50,50,0,0,0,-4.545455,49.792960A5,5,0,0,1,0,54.772256Z",
    ); // d3-shape/test/arc-test.js:280:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(r₀).outerRadius(r₁).startAngle(θ₀).endAngle(θ₁).cornerRadius(rᵧ) renders a small anticlockwise annular sector if r₀ > 0, r₁ > 0 and π > θ₀ - θ₁ ≥ 0
#[test]
fn arc_innerradius_r_outerradius_r_startangle_endangle_cornerradius_r_renders_a_small_anticlockwise_annular_sector_if_r_0_r_0_and_0()
 {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(50.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: (-std::f64::consts::PI / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-94.868330A5,5,0,0,0,-5.263158,-99.861400A100,100,0,0,0,-99.861400,-5.263158A5,5,0,0,0,-94.868330,0L-54.772256,0A5,5,0,0,0,-49.792960,-4.545455A50,50,0,0,1,-4.545455,-49.792960A5,5,0,0,0,0,-54.772256Z",
    ); // d3-shape/test/arc-test.js:286:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(50.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: (-2.0 * std::f64::consts::PI),
                        end_angle: ((-5.0 * std::f64::consts::PI) / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-94.868330A5,5,0,0,0,-5.263158,-99.861400A100,100,0,0,0,-99.861400,-5.263158A5,5,0,0,0,-94.868330,0L-54.772256,0A5,5,0,0,0,-49.792960,-4.545455A50,50,0,0,1,-4.545455,-49.792960A5,5,0,0,0,0,-54.772256Z",
    ); // d3-shape/test/arc-test.js:287:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(50.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: std::f64::consts::PI,
                        end_angle: (std::f64::consts::PI / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,94.868330A5,5,0,0,0,5.263158,99.861400A100,100,0,0,0,99.861400,5.263158A5,5,0,0,0,94.868330,0L54.772256,0A5,5,0,0,0,49.792960,4.545455A50,50,0,0,1,4.545455,49.792960A5,5,0,0,0,0,54.772256Z",
    ); // d3-shape/test/arc-test.js:288:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(r₀).outerRadius(r₁).startAngle(θ₀).endAngle(θ₁).cornerRadius(rᵧ) renders a large clockwise annular sector if r₀ > 0, r₁ > 0 and τ > θ₁ - θ₀ ≥ π
#[test]
fn arc_innerradius_r_outerradius_r_startangle_endangle_cornerradius_r_renders_a_large_clockwise_annular_sector_if_r_0_r_0_and()
 {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(50.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: ((3.0 * std::f64::consts::PI) / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-94.868330A5,5,0,0,1,5.263158,-99.861400A100,100,0,1,1,-99.861400,5.263158A5,5,0,0,1,-94.868330,0L-54.772256,0A5,5,0,0,1,-49.792960,4.545455A50,50,0,1,0,4.545455,-49.792960A5,5,0,0,1,0,-54.772256Z",
    ); // d3-shape/test/arc-test.js:294:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(50.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: (2.0 * std::f64::consts::PI),
                        end_angle: ((7.0 * std::f64::consts::PI) / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-94.868330A5,5,0,0,1,5.263158,-99.861400A100,100,0,1,1,-99.861400,5.263158A5,5,0,0,1,-94.868330,0L-54.772256,0A5,5,0,0,1,-49.792960,4.545455A50,50,0,1,0,4.545455,-49.792960A5,5,0,0,1,0,-54.772256Z",
    ); // d3-shape/test/arc-test.js:295:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(50.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: -std::f64::consts::PI,
                        end_angle: (std::f64::consts::PI / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,94.868330A5,5,0,0,1,-5.263158,99.861400A100,100,0,1,1,99.861400,-5.263158A5,5,0,0,1,94.868330,0L54.772256,0A5,5,0,0,1,49.792960,-4.545455A50,50,0,1,0,-4.545455,49.792960A5,5,0,0,1,0,54.772256Z",
    ); // d3-shape/test/arc-test.js:296:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(r₀).outerRadius(r₁).startAngle(θ₀).endAngle(θ₁).cornerRadius(rᵧ) renders a large anticlockwise annular sector if r₀ > 0, r₁ > 0 and τ > θ₀ - θ₁ ≥ π
#[test]
fn arc_innerradius_r_outerradius_r_startangle_endangle_cornerradius_r_renders_a_large_anticlockwise_annular_sector_if_r_0_r_0_and()
 {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(50.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: ((-3.0 * std::f64::consts::PI) / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-94.868330A5,5,0,0,0,-5.263158,-99.861400A100,100,0,1,0,99.861400,5.263158A5,5,0,0,0,94.868330,0L54.772256,0A5,5,0,0,0,49.792960,4.545455A50,50,0,1,1,-4.545455,-49.792960A5,5,0,0,0,0,-54.772256Z",
    ); // d3-shape/test/arc-test.js:302:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(50.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: (-2.0 * std::f64::consts::PI),
                        end_angle: ((-7.0 * std::f64::consts::PI) / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-94.868330A5,5,0,0,0,-5.263158,-99.861400A100,100,0,1,0,99.861400,5.263158A5,5,0,0,0,94.868330,0L54.772256,0A5,5,0,0,0,49.792960,4.545455A50,50,0,1,1,-4.545455,-49.792960A5,5,0,0,0,0,-54.772256Z",
    ); // d3-shape/test/arc-test.js:303:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(50.0)
                .outer_radius(100.0)
                .corner_radius(5.0)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: std::f64::consts::PI,
                        end_angle: (-std::f64::consts::PI / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,94.868330A5,5,0,0,0,5.263158,99.861400A100,100,0,1,0,-99.861400,-5.263158A5,5,0,0,0,-94.868330,0L-54.772256,0A5,5,0,0,0,-49.792960,-4.545455A50,50,0,1,1,4.545455,49.792960A5,5,0,0,0,0,54.772256Z",
    ); // d3-shape/test/arc-test.js:304:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(r₀).outerRadius(r₁).cornerRadius(rᵧ) restricts rᵧ to |r₁ - r₀| / 2
#[test]
fn arc_innerradius_r_outerradius_r_cornerradius_r_restricts_r_to_r_r_2() {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(90.0)
                .outer_radius(100.0)
                .corner_radius(f64::INFINITY)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: (std::f64::consts::PI / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-94.868330A5,5,0,0,1,5.263158,-99.861400A100,100,0,0,1,99.861400,-5.263158A5,5,0,0,1,94.868330,0L94.868330,0A5,5,0,0,1,89.875260,-4.736842A90,90,0,0,0,4.736842,-89.875260A5,5,0,0,1,0,-94.868330Z",
    ); // d3-shape/test/arc-test.js:309:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(100.0)
                .outer_radius(90.0)
                .corner_radius(f64::INFINITY)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: (std::f64::consts::PI / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-94.868330A5,5,0,0,1,5.263158,-99.861400A100,100,0,0,1,99.861400,-5.263158A5,5,0,0,1,94.868330,0L94.868330,0A5,5,0,0,1,89.875260,-4.736842A90,90,0,0,0,4.736842,-89.875260A5,5,0,0,1,0,-94.868330Z",
    ); // d3-shape/test/arc-test.js:310:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(r₀).outerRadius(r₁).cornerRadius(rᵧ) merges adjacent corners when rᵧ is relatively large
#[test]
fn arc_innerradius_r_outerradius_r_cornerradius_r_merges_adjacent_corners_when_r_is_relatively_large()
 {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(10.0)
                .outer_radius(100.0)
                .corner_radius(f64::INFINITY)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: (std::f64::consts::PI / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-41.421356A41.421356,41.421356,0,1,1,41.421356,0L24.142136,0A24.142136,24.142136,0,0,1,0,-24.142136Z",
    ); // d3-shape/test/arc-test.js:315:3
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(100.0)
                .outer_radius(10.0)
                .corner_radius(f64::INFINITY)
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: (std::f64::consts::PI / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-41.421356A41.421356,41.421356,0,1,1,41.421356,0L24.142136,0A24.142136,24.142136,0,0,1,0,-24.142136Z",
    ); // d3-shape/test/arc-test.js:316:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(0).outerRadius(0).startAngle(0).endAngle(τ).padAngle(δ) does not pad a point
#[test]
fn arc_innerradius_0_outerradius_0_startangle_0_endangle_padangle_does_not_pad_a_point() {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(0.0)
                .outer_radius(0.0)
                .pad_angle(Some(0.1))
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: (2.0 * std::f64::consts::PI),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,0Z",
    ); // d3-shape/test/arc-test.js:321:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(0).outerRadius(r).startAngle(0).endAngle(τ).padAngle(δ) does not pad a circle
#[test]
fn arc_innerradius_0_outerradius_r_startangle_0_endangle_padangle_does_not_pad_a_circle() {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(0.0)
                .outer_radius(100.0)
                .pad_angle(Some(0.1))
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: (2.0 * std::f64::consts::PI),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-100A100,100,0,1,1,0,100A100,100,0,1,1,0,-100Z",
    ); // d3-shape/test/arc-test.js:326:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(r₀).outerRadius(r₁).startAngle(0).endAngle(τ).padAngle(δ) does not pad an annulus
#[test]
fn arc_innerradius_r_outerradius_r_startangle_0_endangle_padangle_does_not_pad_an_annulus() {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(50.0)
                .outer_radius(100.0)
                .pad_angle(Some(0.1))
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: (2.0 * std::f64::consts::PI),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M0,-100A100,100,0,1,1,0,100A100,100,0,1,1,0,-100M0,-50A50,50,0,1,0,0,50A50,50,0,1,0,0,-50Z",
    ); // d3-shape/test/arc-test.js:331:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(0).outerRadius(r).startAngle(θ₀).endAngle(θ₁).padAngle(δ) pads the outside of a circular sector
#[test]
fn arc_innerradius_0_outerradius_r_startangle_endangle_padangle_pads_the_outside_of_a_circular_sector()
 {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(0.0)
                .outer_radius(100.0)
                .pad_angle(Some(0.1))
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: (std::f64::consts::PI / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M4.997917,-99.875026A100,100,0,0,1,99.875026,-4.997917L0,0Z",
    ); // d3-shape/test/arc-test.js:336:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(r₀).outerRadius(r₁).startAngle(θ₀).endAngle(θ₁).padAngle(δ) pads an annular sector
#[test]
fn arc_innerradius_r_outerradius_r_startangle_endangle_padangle_pads_an_annular_sector() {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(50.0)
                .outer_radius(100.0)
                .pad_angle(Some(0.1))
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: (std::f64::consts::PI / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M5.587841,-99.843758A100,100,0,0,1,99.843758,-5.587841L49.686779,-5.587841A50,50,0,0,0,5.587841,-49.686779Z",
    ); // d3-shape/test/arc-test.js:341:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(r₀).outerRadius(r₁).startAngle(θ₀).endAngle(θ₁).padAngle(δ) may collapse the inside of an annular sector
#[test]
fn arc_innerradius_r_outerradius_r_startangle_endangle_padangle_may_collapse_the_inside_of_an_annular_sector()
 {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(10.0)
                .outer_radius(100.0)
                .pad_angle(Some(0.2))
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: (std::f64::consts::PI / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M10.033134,-99.495408A100,100,0,0,1,99.495408,-10.033134L7.071068,-7.071068Z",
    ); // d3-shape/test/arc-test.js:346:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(0).outerRadius(r).startAngle(θ₀).endAngle(θ₁).padAngle(δ).cornerRadius(rᵧ) rounds and pads a circular sector
#[test]
fn arc_innerradius_0_outerradius_r_startangle_endangle_padangle_cornerradius_r_rounds_and_pads_a_circular_sector()
 {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(0.0)
                .outer_radius(100.0)
                .corner_radius(10.0)
                .pad_angle(Some(0.1))
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: (std::f64::consts::PI / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M4.470273,-89.330939A10,10,0,0,1,16.064195,-98.701275A100,100,0,0,1,98.701275,-16.064195A10,10,0,0,1,89.330939,-4.470273L0,0Z",
    ); // d3-shape/test/arc-test.js:351:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(r₀).outerRadius(r₁).startAngle(θ₀).endAngle(θ₁).padAngle(δ).cornerRadius(rᵧ) rounds and pads an annular sector
#[test]
fn arc_innerradius_r_outerradius_r_startangle_endangle_padangle_cornerradius_r_rounds_and_pads_an_annular_sector()
 {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(50.0)
                .outer_radius(100.0)
                .corner_radius(10.0)
                .pad_angle(Some(0.1))
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: (std::f64::consts::PI / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M5.587841,-88.639829A10,10,0,0,1,17.319823,-98.488698A100,100,0,0,1,98.488698,-17.319823A10,10,0,0,1,88.639829,-5.587841L57.939790,-5.587841A10,10,0,0,1,48.283158,-12.989867A50,50,0,0,0,12.989867,-48.283158A10,10,0,0,1,5.587841,-57.939790Z",
    ); // d3-shape/test/arc-test.js:356:3
}

// d3-shape/test/arc-test.js: arc().innerRadius(r₀).outerRadius(r₁).startAngle(θ₀).endAngle(θ₁).padAngle(δ).cornerRadius(rᵧ) rounds and pads a collapsed annular sector
#[test]
fn arc_innerradius_r_outerradius_r_startangle_endangle_padangle_cornerradius_r_rounds_and_pads_a_collapsed_annular_sector()
 {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(10.0)
                .outer_radius(100.0)
                .corner_radius(10.0)
                .pad_angle(Some(0.2))
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: 0.0,
                        end_angle: (std::f64::consts::PI / 2.0),
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M9.669396,-88.145811A10,10,0,0,1,21.849183,-97.583878A100,100,0,0,1,97.583878,-21.849183A10,10,0,0,1,88.145811,-9.669396L7.071068,-7.071068Z",
    ); // d3-shape/test/arc-test.js:361:3
}

// d3-shape/test/arc-test.js: arc() handles a very small arc with rounded corners
#[test]
fn arc_handles_a_very_small_arc_with_rounded_corners() {
    assert_path_eq(
        &drawn(|ctx| {
            shape::arc()
                .inner_radius(15.0)
                .outer_radius(24.0)
                .corner_radius(4.0)
                .pad_angle(Some(0.0))
                .draw(
                    &shape::Slice {
                        datum: 0,
                        index: 0,
                        value: 0.0,
                        start_angle: (1.2 - 1e-8),
                        end_angle: 1.2,
                        pad_angle: 0.0,
                    },
                    ctx,
                )
        }),
        &"M22.369,-8.697L13.981,-5.435Z",
    ); // d3-shape/test/arc-test.js:366:3
}
