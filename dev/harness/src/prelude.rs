//! What the generated tests call: [`check`], [`skip`], [`throws`] and the
//! assertion [`Kind`]s. Each call is recorded for the run to fold.

use std::fmt;

use rustd3_testkit::obs::{Obs, ToObs, Token, Tolerance, tokenize};

use crate::record::{self, Mark, Record};

pub(crate) use crate::stub;

/// The d3 assertion an assertion was written with, which sets its d3 verdict.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    /// `strictEqual`, `deepStrictEqual`, `equal`: exact in d3 (`Object.is`).
    Strict,
    /// `assertInDelta`: within an absolute tolerance.
    InDelta(f64),
    /// `assertPathEqual`: equal after d3-shape's `normalizePath` (integers
    /// within 1e-6, everything else to 3 decimals).
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "the stub suite has no path assertions; Phase 3's path suite does"
        )
    )]
    PathEq,
    /// `assert.throws`: the call must return `Err`.
    Throws,
}

/// Records one assertion under d3's semantics and under the tolerant
/// format (`Tolerance::default()`, or `at_least(d)` for `InDelta(d)`).
/// Under `Throws`, both pass iff `actual` is the [`Obs`] of an `Err`.
pub fn check<A: ToObs + ?Sized, B: ToObs + ?Sized>(
    id: &'static str,
    kind: Kind,
    actual: &A,
    expected: &B,
) {
    let (actual, expected) = (actual.to_obs(), expected.to_obs());
    let (d3, tolerant) = match kind {
        Kind::Strict => (
            actual.strict_eq(&expected),
            actual.approx_eq(&expected, &Tolerance::default()),
        ),
        Kind::InDelta(delta) => (
            in_delta(&actual, &expected, delta),
            actual.approx_eq(&expected, &Tolerance::at_least(delta)),
        ),
        Kind::PathEq => (
            d3_normalize(&actual) == d3_normalize(&expected),
            actual.approx_eq(&expected, &Tolerance::default()),
        ),
        Kind::Throws => {
            let thrown = if let Obs::Str(s) = &actual {
                s.starts_with("error: ")
            } else {
                false
            };
            (thrown, thrown)
        }
    };
    record::push(Record {
        id,
        mark: Mark::Checked {
            kind,
            d3,
            tolerant,
            actual,
            expected,
        },
    });
}

/// Marks an assertion the translator skipped. `reason` repeats its
/// `ASSERTS` entry, which the run verifies.
pub fn skip(id: &'static str, reason: &'static str) {
    record::push(Record {
        id,
        mark: Mark::Skipped(reason),
    });
}

/// `assert.throws`: records a `Kind::Throws` record that passes under both
/// verdicts iff `f()` returns `Err`; an `Ok(v)` fails and reports `v`.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the stub suite has no throws assertions; Phase 3's path suite does"
    )
)]
pub fn throws<T: ToObs, E: fmt::Debug>(id: &'static str, f: impl FnOnce() -> Result<T, E>) {
    let (thrown, actual) = match f() {
        Ok(value) => (false, value.to_obs()),
        Err(error) => (true, Obs::Str(format!("error: {error:?}"))),
    };
    let expected = Obs::Str("an error".to_owned());
    let mark = Mark::Checked {
        kind: Kind::Throws,
        d3: thrown,
        tolerant: thrown,
        actual,
        expected,
    };
    record::push(Record { id, mark });
}

/// d3's `assertInDelta`: numbers within `delta`, arrays element by element.
fn in_delta(actual: &Obs, expected: &Obs, delta: f64) -> bool {
    match (actual, expected) {
        (Obs::Num(a), Obs::Num(e)) => *a >= e - delta && *a <= e + delta,
        (Obs::List(a), Obs::List(e)) => {
            a.len() == e.len() && a.iter().zip(e).all(|(p, q)| in_delta(p, q, delta))
        }
        (Obs::Null | Obs::Bool(_) | Obs::Num(_) | Obs::Str(_) | Obs::List(_), _) => {
            actual == expected
        }
    }
}

/// d3-shape's `normalizePath`: numbers within 1e-6 of an integer print as
/// that integer, every other number with 3 decimals.
fn d3_normalize(path: &Obs) -> Obs {
    let Obs::Str(s) = path else {
        return path.clone();
    };
    let mut out = String::new();
    for token in tokenize(s) {
        match token {
            Token::Text(text) => out.push_str(&text),
            Token::Num { value, .. } => {
                let integer = value.round();
                if (value - integer).abs() < 1e-6 {
                    let integer = if integer == 0.0 { 0.0 } else { integer };
                    out.push_str(&integer.to_string());
                } else {
                    out.push_str(&format!("{value:.3}"));
                }
            }
        }
    }
    Obs::Str(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::{Mark, take};

    fn verdicts() -> Vec<(bool, bool)> {
        take()
            .into_iter()
            .map(|r| match r.mark {
                Mark::Checked { d3, tolerant, .. } => (d3, tolerant),
                Mark::Skipped(reason) => panic!("unexpected skip: {reason}"),
            })
            .collect()
    }

    #[test]
    fn strict_is_object_is_while_the_tolerant_verdict_allows_rounding() {
        check("a:1:1", Kind::Strict, &(0.1 + 0.2), &0.3);
        check("a:2:1", Kind::Strict, &f64::NAN, &f64::NAN);
        check("a:3:1", Kind::Strict, &-0.0, &0.0);
        check("a:4:1", Kind::Strict, "1e+21", "1000000000000000000000");
        assert_eq!(
            verdicts(),
            [(false, true), (true, true), (false, true), (false, true)]
        );
    }

    #[test]
    fn in_delta_keeps_d3s_delta_in_both_verdicts() {
        check("a:1:1", Kind::InDelta(1e-6), &0.1000005, &0.1);
        check("a:2:1", Kind::InDelta(1e-6), &0.100002, &0.1);
        check(
            "a:3:1",
            Kind::InDelta(0.5),
            &vec![1.0, 2.4],
            &vec![1.0, 2.0],
        );
        assert_eq!(verdicts(), [(true, true), (false, false), (true, true)]);
    }

    #[test]
    fn path_equality_normalizes_numbers_to_three_decimals() {
        check(
            "a:1:1",
            Kind::PathEq,
            "M0.0000001,1.00049L2",
            "M0,1.0004L2.0000000001",
        );
        check("a:2:1", Kind::PathEq, "M0,1.0006", "M0,1.0004");
        assert_eq!(verdicts(), [(true, false), (false, false)]);
    }

    #[test]
    fn throws_passes_only_on_err() {
        throws("a:1:1", || Err::<f64, &str>("negative radius"));
        throws("a:2:1", || Ok::<f64, &str>(2.0));
        check("a:3:1", Kind::Throws, &Err::<(), &str>("bad"), &());
        check("a:4:1", Kind::Throws, &Ok::<(), &str>(()), &());
        let records = take();
        let marks: Vec<_> = records
            .iter()
            .map(|r| match &r.mark {
                Mark::Checked {
                    kind,
                    d3,
                    tolerant,
                    actual,
                    ..
                } => (*kind, *d3, *tolerant, actual.clone()),
                Mark::Skipped(reason) => panic!("unexpected skip: {reason}"),
            })
            .collect();
        assert_eq!(
            marks,
            [
                (
                    Kind::Throws,
                    true,
                    true,
                    Obs::Str("error: \"negative radius\"".to_owned())
                ),
                (Kind::Throws, false, false, Obs::Num(2.0)),
                (Kind::Throws, true, true, Obs::Str("error: bad".to_owned())),
                (Kind::Throws, false, false, Obs::Null),
            ]
        );
    }

    #[test]
    fn skip_records_its_reason() {
        skip("a:1:1", "js: coercion");
        let records = take();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id, "a:1:1");
        assert!(matches!(records[0].mark, Mark::Skipped("js: coercion")));
    }

    #[test]
    fn d3_normalize_rounds_like_d3_shape() {
        let normalized = d3_normalize(&Obs::Str("M-0.0000001,2.00049Z".to_owned()));
        assert_eq!(normalized, Obs::Str("M0,2.000Z".to_owned()));
        assert_eq!(d3_normalize(&Obs::Num(1.5)), Obs::Num(1.5));
    }
}
