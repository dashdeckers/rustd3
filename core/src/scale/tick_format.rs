//! d3-scale's tickFormat.js, and [`TickFormat`]: the labels of one scale's
//! ticks.

use crate::format::{self, Format, InvalidFormat, Kind, Locale, Specifier};
use crate::scale::transform;
use crate::{js, ticks};

/// Labels for one scale's ticks: d3's `scale.tickFormat(count, specifier)`
/// as a value, a number format and, for log scales, which ticks get one.
#[derive(Debug, Clone, PartialEq)]
pub struct TickFormat {
    format: Format,
    keep: Option<LogKeep>,
}

/// Which ticks of a log scale keep their label (log.js `tickFormat`): the
/// value over the power of the base nearest it, times the base when below
/// `base − 0.5`, is at most `k`.
#[derive(Debug, Clone, Copy, PartialEq)]
struct LogKeep {
    log: transform::Log,
    k: f64,
}

impl From<Format> for TickFormat {
    /// A tick format that labels every tick with `format`.
    fn from(format: Format) -> TickFormat {
        TickFormat { format, keep: None }
    }
}

impl TickFormat {
    /// `format` for the ticks of a log scale under `log` (set for its
    /// domain), labelling only those whose leading digit is at most `k`.
    pub(crate) fn thinned(format: Format, log: transform::Log, k: f64) -> TickFormat {
        TickFormat {
            format,
            keep: Some(LogKeep { log, k }),
        }
    }

    /// One tick's label; `""` for a tick a log scale leaves unlabelled.
    pub fn format(&self, value: f64) -> String {
        match &self.keep {
            Some(keep) if !keep.keeps(value) => String::new(),
            Some(_) | None => self.format.format(value),
        }
    }

    /// The labels of a whole tick set, one per value, value by value, with
    /// a log scale's blanking.
    pub fn format_all(&self, values: &[f64]) -> Vec<String> {
        values.iter().map(|&v| self.format(v)).collect()
    }
}

impl LogKeep {
    /// Whether `value` keeps its label.
    fn keeps(&self, value: f64) -> bool {
        let base = self.log.base();
        let mut digit = value / self.log.pows(js::round(self.log.logs(value)));
        if digit * base < base - 0.5 {
            digit *= base;
        }
        digit <= self.k
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
