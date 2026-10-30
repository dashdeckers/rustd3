//! `#[derive(Interpolate)]`: spot checks, and the update pattern's `Bar`
//! against its hand-written impl.

use std::marker::PhantomData;

use rustd3_core::interpolate::Interpolate;

#[derive(Debug, Clone, PartialEq, Interpolate)]
struct Bar {
    y: f64,
    width: f64,
    opacity: f64,
    value: f64,
    #[interpolate(skip)]
    name: String,
}

#[derive(Debug, Clone, PartialEq, Interpolate)]
struct Mark<T> {
    v: T,
    w: f64,
    #[interpolate(skip)]
    label: String,
}

/// Generic over a parameter that only a nested field names.
#[derive(Debug, Clone, PartialEq, Interpolate)]
struct Nested<T> {
    m: Mark<T>,
    k: f64,
}

/// A parameter that only a skipped `PhantomData` names.
#[derive(Debug, Clone, PartialEq, Interpolate)]
struct Tagged<U> {
    x: f64,
    #[interpolate(skip)]
    unit: PhantomData<U>,
}

/// The trait is not imported here; the container attribute names it.
mod plain {
    #[derive(Clone, rustd3_core::interpolate::Interpolate)]
    #[interpolate(crate = "rustd3_core")]
    pub struct S {
        pub x: f64,
    }
}

/// The update pattern's `Bar` with a hand-written impl.
mod hand {
    use rustd3_core::interpolate::Interpolate;

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Bar {
        pub y: f64,
        pub width: f64,
        pub opacity: f64,
        pub value: f64,
    }

    impl Interpolate for Bar {
        fn lerp(&self, to: &Self, t: f64) -> Self {
            let f = |a: f64, b: f64| a.lerp(&b, t);
            Bar {
                y: f(self.y, to.y),
                width: f(self.width, to.width),
                opacity: f(self.opacity, to.opacity),
                value: f(self.value, to.value),
            }
        }
    }
}

/// The update pattern's `Bar`, derived.
mod derived {
    use rustd3_core::interpolate::Interpolate;

    #[derive(Debug, Clone, Copy, PartialEq, Interpolate)]
    pub struct Bar {
        pub y: f64,
        pub width: f64,
        pub opacity: f64,
        pub value: f64,
    }
}

fn bar(y: f64, width: f64, opacity: f64, value: f64, name: &str) -> Bar {
    let name = name.to_owned();
    Bar {
        y,
        width,
        opacity,
        value,
        name,
    }
}

fn mark(v: f64, w: f64, label: &str) -> Mark<f64> {
    let label = label.to_owned();
    Mark { v, w, label }
}

#[test]
fn bar_blends_its_numbers_and_keeps_its_name_until_the_end() {
    let (a, b) = (
        bar(0.0, 0.0, 0.0, 0.0, "a"),
        bar(40.0, 200.0, 1.0, 10.0, "b"),
    );
    assert_eq!(a.lerp(&b, 0.25), bar(10.0, 50.0, 0.25, 2.5, "a"));
    assert_eq!(a.lerp(&b, 1.0).name, "b");
    assert_eq!(a.lerp(&b, 0.999).name, "a");
}

#[test]
fn a_generic_mark_blends_through_its_parameter() {
    let (a, b) = (mark(0.0, 1.0, "a"), mark(10.0, 3.0, "b"));
    assert_eq!(a.lerp(&b, 0.5), mark(5.0, 2.0, "a"));
}

#[test]
fn a_nested_mark_blends_through_its_own_impl() {
    let a = Nested {
        m: mark(0.0, 1.0, "a"),
        k: 0.0,
    };
    let b = Nested {
        m: mark(10.0, 3.0, "b"),
        k: 4.0,
    };
    let quarter = Nested {
        m: mark(2.5, 1.5, "a"),
        k: 1.0,
    };
    assert_eq!(a.lerp(&b, 0.25), quarter);
}

#[test]
fn a_parameter_only_a_skipped_phantom_names_still_derives() {
    #[derive(Clone)]
    struct Unit;
    let a = Tagged::<Unit> {
        x: 0.0,
        unit: PhantomData,
    };
    let b = Tagged::<Unit> {
        x: 8.0,
        unit: PhantomData,
    };
    assert_eq!(a.lerp(&b, 0.25).x, 2.0);
}

#[test]
fn the_crate_attribute_works_without_the_trait_in_scope() {
    let s = plain::S { x: 0.0 }.lerp(&plain::S { x: 2.0 }, 0.5);
    assert_eq!(s.x, 1.0);
}

#[test]
fn use_case_two_bar_derived_equals_its_hand_written_impl() {
    let from = (0.0, 0.0, 0.0, 0.0);
    let to = (125.0, 310.5, 1.0, 42.0);
    let hand = |(y, width, opacity, value)| hand::Bar {
        y,
        width,
        opacity,
        value,
    };
    let derived = |(y, width, opacity, value)| derived::Bar {
        y,
        width,
        opacity,
        value,
    };
    for t in [0.0, 0.25, 0.5, 1.0] {
        let h = hand(from).lerp(&hand(to), t);
        let d = derived(from).lerp(&derived(to), t);
        assert_eq!(
            (h.y, h.width, h.opacity, h.value),
            (d.y, d.width, d.opacity, d.value)
        );
    }
}
