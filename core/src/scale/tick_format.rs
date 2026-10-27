//! d3-scale's tickFormat.js, and [`TickFormat`]: the labels of one scale's
//! ticks.

use crate::format::{self, Format, InvalidFormat, Kind, Locale, Specifier};
use crate::scale::transform;
use crate::{js, locale, ticks};

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

    /// The labels of a whole tick set, one per value: the number format's
    /// batch rule ([`Format::format_all`]) over every value, then a log
    /// scale's blanking, so a blanked value still counts towards grouping.
    pub fn format_all(&self, values: &[f64]) -> Vec<String> {
        let mut labels = self.format.format_all(values);
        if let Some(keep) = &self.keep {
            for (label, &value) in labels.iter_mut().zip(values) {
                if !keep.keeps(value) {
                    label.clear();
                }
            }
        }
        labels
    }

    /// Labels a whole tick set as one axis under `locale`: the same
    /// specifier and fixed SI prefix, written in `locale`'s number half,
    /// with [`format_all`](TickFormat::format_all)'s batch rule and log
    /// blanking.
    ///
    /// ```
    /// use rustd3_core::locale::Locale;
    /// use rustd3_core::scale::{self, Ticks};
    ///
    /// let x = scale::linear().domain([0.0, 12000.0]);
    /// let set = x.tick_set(5);
    /// let neutral = x.tick_format(5, None)?;
    /// assert_eq!(neutral.labels(&set, &Locale::EN_US)[6], "12,000");
    /// assert_eq!(neutral.labels(&set, &Locale::NEUTRAL)[6], "12\u{202f}000");
    /// # Ok::<(), rustd3_core::format::InvalidFormat>(())
    /// ```
    pub fn labels(&self, set: &ticks::Set<f64, f64>, locale: &locale::Locale) -> Vec<String> {
        if self.format.locale() == locale.number() {
            return self.format_all(set.values());
        }
        TickFormat {
            format: self.format.with_locale(locale.number()),
            keep: self.keep,
        }
        .format_all(set.values())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_blanked_value_still_decides_the_grouping_of_the_others() {
        let format = Locale::NEUTRAL.format(",").expect("a valid specifier");
        let thinned = TickFormat::thinned(format, transform::Log::new(10.0), 1.0);
        assert_eq!(
            thinned.format_all(&[1000.0, 90_000.0]),
            ["1\u{202f}000", ""]
        );
        assert_eq!(thinned.format(1000.0), "1000");
    }
}
