//! d3-scale's tickFormat.js, and [`TickFormat`]: the labels of one scale's
//! ticks.

use crate::format::{self, Format, InvalidFormat, Kind, Locale, Specifier};
use crate::{js, ticks};

/// Labels for one scale's ticks: d3's `scale.tickFormat(count, specifier)`
/// as a value.
#[derive(Debug, Clone, PartialEq)]
pub struct TickFormat {
    format: Format,
}

impl From<Format> for TickFormat {
    /// A tick format that labels every tick with `format`.
    fn from(format: Format) -> TickFormat {
        TickFormat { format }
    }
}

impl TickFormat {
    /// One tick's label.
    pub fn format(&self, value: f64) -> String {
        self.format.format(value)
    }

    /// The labels of a whole tick set, one per value.
    pub fn format_all(&self, values: &[f64]) -> Vec<String> {
        values.iter().map(|&v| self.format(v)).collect()
    }
}

/// d3-scale's tickFormat(start, stop, count, specifier): a format for
/// about `count` ticks over `domain` under `locale`. The specifier defaults
/// to `",f"`; when it sets no precision, the precision follows from the
/// tick step: significant digits for no type, `e`, `g`, `p` and `r`,
/// decimal places for `f` and `%`, and for `s` decimal places under the SI
/// prefix of the larger end.
#[doc(alias = "tickFormat")]
pub fn tick_format(
    locale: &Locale,
    domain: [f64; 2],
    count: usize,
    specifier: Option<&str>,
) -> Result<Format, InvalidFormat> {
    let [start, stop] = domain;
    let step = ticks::tick_step(start, stop, count as f64);
    let largest = js::max(start.abs(), stop.abs());
    let specifier: Specifier = specifier.unwrap_or(",f").parse()?;
    // A precision the specifier sets wins; d3's NaN precision is None.
    let precise = |precision: Option<usize>, less: usize| match precision {
        Some(p) if specifier.precision().is_none() => {
            specifier.clone().with_precision(p.saturating_sub(less))
        }
        Some(_) | None => specifier.clone(),
    };
    Ok(match specifier.kind() {
        Some(Kind::Si) => Format::prefixed(
            locale,
            precise(format::precision_prefix(step, largest), 0),
            largest,
        ),
        None | Some(Kind::General | Kind::PercentRounded | Kind::Rounded) => {
            Format::new(locale, precise(format::precision_round(step, largest), 0))
        }
        Some(Kind::Exponent) => {
            Format::new(locale, precise(format::precision_round(step, largest), 1))
        }
        Some(Kind::Fixed) => Format::new(locale, precise(format::precision_fixed(step), 0)),
        // d3 subtracts 2 and the format clamps a negative precision to 0.
        Some(Kind::Percent) => Format::new(locale, precise(format::precision_fixed(step), 2)),
        Some(
            Kind::Binary
            | Kind::Octal
            | Kind::Decimal
            | Kind::Hex
            | Kind::HexUpper
            | Kind::Char
            | Kind::Number
            | Kind::Other(_),
        ) => Format::new(locale, specifier),
    })
}
