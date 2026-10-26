//! Port of d3-format, built on `std`'s float formatting rather than JS's `toFixed` family.

use std::fmt;

/// A format specifier that does not parse.
#[derive(Debug, Clone, PartialEq)]
pub struct InvalidFormat(pub String);

impl fmt::Display for InvalidFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid format: {}", self.0)
    }
}

impl std::error::Error for InvalidFormat {}

/// `[[fill]align][sign][symbol][0][width][,][.precision][~][type]`
#[derive(Debug, Clone, PartialEq)]
pub struct Specifier {
    pub fill: char,
    pub align: char,
    pub sign: char,
    pub symbol: Option<char>,
    pub zero: bool,
    pub width: Option<usize>,
    pub comma: bool,
    pub precision: Option<usize>,
    pub trim: bool,
    pub kind: Option<char>,
}

impl std::str::FromStr for Specifier {
    type Err = InvalidFormat;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c: Vec<char> = s.chars().collect();
        let invalid = || InvalidFormat(s.to_owned());
        let is_align = |ch: char| "<>=^".contains(ch);
        let mut spec = Specifier {
            fill: ' ', align: '>', sign: '-', symbol: None, zero: false,
            width: None, comma: false, precision: None, trim: false, kind: None,
        };
        let mut i = 0;
        if c.len() >= 2 && is_align(c[1]) {
            spec.fill = c[0];
            spec.align = c[1];
            i = 2;
        } else if !c.is_empty() && is_align(c[0]) {
            spec.align = c[0];
            i = 1;
        }
        let digits = |i: &mut usize| {
            let from = *i;
            while *i < c.len() && c[*i].is_ascii_digit() {
                *i += 1;
            }
            (from < *i).then(|| c[from..*i].iter().collect::<String>().parse::<usize>().ok()).flatten()
        };
        if i < c.len() && "+-( ".contains(c[i]) { spec.sign = c[i]; i += 1; }
        if i < c.len() && "$#".contains(c[i]) { spec.symbol = Some(c[i]); i += 1; }
        if i < c.len() && c[i] == '0' { spec.zero = true; i += 1; }
        spec.width = digits(&mut i);
        if i < c.len() && c[i] == ',' { spec.comma = true; i += 1; }
        if i < c.len() && c[i] == '.' {
            i += 1;
            spec.precision = Some(digits(&mut i).ok_or_else(invalid)?);
        }
        if i < c.len() && c[i] == '~' { spec.trim = true; i += 1; }
        if i < c.len() && (c[i].is_ascii_alphabetic() || c[i] == '%') { spec.kind = Some(c[i]); i += 1; }
        if i == c.len() { Ok(spec) } else { Err(invalid()) }
    }
}

impl fmt::Display for Specifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}", self.fill, self.align, self.sign)?;
        if let Some(s) = self.symbol { write!(f, "{s}")?; }
        if self.zero { f.write_str("0")?; }
        if let Some(w) = self.width { write!(f, "{}", w.max(1))?; }
        if self.comma { f.write_str(",")?; }
        if let Some(p) = self.precision { write!(f, ".{p}")?; }
        if self.trim { f.write_str("~")?; }
        if let Some(k) = self.kind { write!(f, "{k}")?; }
        Ok(())
    }
}

/// The parts of a locale definition; anything left `None` takes d3's fallback.
#[derive(Debug, Clone, Default)]
pub struct Definition {
    pub decimal: Option<String>,
    pub thousands: Option<String>,
    pub grouping: Option<Vec<usize>>,
    pub currency: Option<[String; 2]>,
    pub numerals: Option<Vec<String>>,
    pub percent: Option<String>,
    pub minus: Option<String>,
    pub nan: Option<String>,
}

/// Locale-specific number formatting rules.
#[derive(Debug, Clone)]
pub struct Locale {
    grouping: Option<(Vec<usize>, String)>,
    currency: [String; 2],
    decimal: String,
    numerals: Option<Vec<String>>,
    percent: String,
    minus: String,
    nan: String,
}

impl Default for Locale {
    /// d3's default, en-US.
    fn default() -> Self {
        Self::new(Definition {
            thousands: Some(",".into()),
            grouping: Some(vec![3]),
            currency: Some(["$".into(), String::new()]),
            ..Definition::default()
        })
    }
}

const PREFIXES: [&str; 17] = ["y", "z", "a", "f", "p", "n", "µ", "m", "", "k", "M", "G", "T", "P", "E", "Z", "Y"];

impl Locale {
    pub fn new(d: Definition) -> Self {
        Self {
            grouping: d.grouping.zip(d.thousands),
            currency: d.currency.unwrap_or_default(),
            decimal: d.decimal.unwrap_or_else(|| ".".into()),
            numerals: d.numerals,
            percent: d.percent.unwrap_or_else(|| "%".into()),
            minus: d.minus.unwrap_or_else(|| "\u{2212}".into()),
            nan: d.nan.unwrap_or_else(|| "NaN".into()),
        }
    }

    /// A formatter for `specifier`.
    pub fn format(&self, specifier: &str) -> Result<Format, InvalidFormat> {
        Ok(Format::new(self.clone(), specifier.parse()?, ""))
    }

    /// A fixed-point formatter that scales values to the SI prefix of `value`.
    pub fn format_prefix(&self, specifier: &str, value: f64) -> Result<PrefixFormat, InvalidFormat> {
        let mut spec: Specifier = specifier.parse()?;
        spec.kind = Some('f');
        let e = exponent(value).map_or(0, |x| (x.div_euclid(3)).clamp(-8, 8) * 3);
        let suffix = PREFIXES[(8 + e / 3) as usize];
        Ok(PrefixFormat { scale: 10f64.powi(-e), inner: Format::new(self.clone(), spec, suffix) })
    }

    fn group(&self, value: &str, width: usize) -> String {
        let Some((grouping, thousands)) = &self.grouping else { return value.to_owned() };
        let chars: Vec<char> = value.chars().collect();
        let (mut i, mut j, mut length) = (chars.len(), 0, 0);
        let mut g = grouping[0];
        let mut parts = Vec::new();
        while i > 0 && g > 0 {
            if length + g + 1 > width {
                g = width.saturating_sub(length).max(1);
            }
            let from = i.saturating_sub(g);
            parts.push(chars[from..i].iter().collect::<String>());
            i = from;
            length += g + 1;
            if length > width {
                break;
            }
            j = (j + 1) % grouping.len();
            g = grouping[j];
        }
        parts.reverse();
        parts.join(thousands)
    }
}

/// d3's default-locale `format(specifier)`.
pub fn format(specifier: &str) -> Result<Format, InvalidFormat> {
    Locale::default().format(specifier)
}

/// d3's default-locale `formatPrefix(specifier, value)`.
pub fn format_prefix(specifier: &str, value: f64) -> Result<PrefixFormat, InvalidFormat> {
    Locale::default().format_prefix(specifier, value)
}

/// A number formatter.
#[derive(Debug, Clone)]
pub struct Format {
    locale: Locale,
    spec: Specifier,
    kind: char,
    precision: usize,
    trim: bool,
    comma: bool,
    zero: bool,
    fill: char,
    align: char,
    prefix: String,
    suffix: String,
}

impl Format {
    fn new(locale: Locale, spec: Specifier, extra_suffix: &str) -> Self {
        let (mut kind, mut comma, mut trim, mut precision) = (spec.kind.unwrap_or(' '), spec.comma, spec.trim, spec.precision);
        if kind == 'n' {
            comma = true;
            kind = 'g';
        } else if !"%bcdefgoprsXx".contains(kind) {
            precision.get_or_insert(12);
            trim = true;
            kind = 'g';
        }
        let (mut fill, mut align, mut zero) = (spec.fill, spec.align, spec.zero);
        if zero || (fill == '0' && align == '=') {
            (zero, fill, align) = (true, '0', '=');
        }
        let prefix = match spec.symbol {
            Some('$') => locale.currency[0].clone(),
            Some('#') if "boxX".contains(kind) => format!("0{}", kind.to_ascii_lowercase()),
            _ => String::new(),
        };
        let suffix = match spec.symbol {
            Some('$') => locale.currency[1].clone(),
            _ if "%p".contains(kind) => locale.percent.clone(),
            _ => String::new(),
        } + extra_suffix;
        let precision = match precision {
            None => 6,
            Some(p) if "gprs".contains(kind) => p.clamp(1, 21),
            Some(p) => p.min(20),
        };
        Self { locale, spec, kind, precision, trim, comma, zero, fill, align, prefix, suffix }
    }

    pub fn format(&self, value: f64) -> String {
        let (kind, p) = (self.kind, self.precision);
        let mut value_prefix = self.prefix.clone();
        let mut value_suffix = self.suffix.clone();
        let mut value_str;
        if kind == 'c' {
            value_suffix = format!("{value}{value_suffix}");
            value_str = String::new();
        } else {
            let mut negative = value < 0.0 || 1.0 / value < 0.0;
            let mut si = None;
            value_str = if value.is_nan() {
                self.locale.nan.clone()
            } else if value.is_infinite() {
                // JS prints "Infinity" for every format type; std would print "inf".
                "Infinity".to_owned()
            } else {
                let x = value.abs();
                match kind {
                    '%' => format!("{:.*}", p, x * 100.0),
                    'b' => format!("{:b}", x.round() as u128),
                    'd' => format!("{}", x.round()),
                    'e' => format!("{:.*e}", p, x),
                    'f' => format!("{:.*}", p, x),
                    'g' => to_precision(x, p),
                    'o' => format!("{:o}", x.round() as u128),
                    'p' => format_rounded(x * 100.0, p),
                    'r' => format_rounded(x, p),
                    's' => {
                        let (s, e) = format_prefix_auto(x, p);
                        si = e;
                        s
                    }
                    'X' => format!("{:X}", x.round() as u128),
                    'x' => format!("{:x}", x.round() as u128),
                    _ => unreachable!("kind normalized in Format::new"),
                }
            };
            if self.trim {
                value_str = format_trim(&value_str);
            }
            if negative && value_str.parse::<f64>().is_ok_and(|v| v == 0.0) && self.spec.sign != '+' {
                negative = false;
            }
            let sign = self.spec.sign;
            let sign_str = if negative {
                if sign == '(' { "(".to_owned() } else { self.locale.minus.clone() }
            } else if sign == '-' || sign == '(' {
                String::new()
            } else {
                sign.to_string()
            };
            value_prefix = sign_str + &value_prefix;
            let si_suffix = match si {
                Some(e) if kind == 's' => PREFIXES[(8 + e / 3) as usize],
                _ => "",
            };
            value_suffix = format!("{si_suffix}{value_suffix}{}", if negative && sign == '(' { ")" } else { "" });
            if "defgprs%".contains(kind) {
                if let Some(i) = value_str.find(|c: char| !c.is_ascii_digit()) {
                    let rest = if value_str[i..].starts_with('.') {
                        format!("{}{}", self.locale.decimal, &value_str[i + 1..])
                    } else {
                        value_str[i..].to_owned()
                    };
                    value_suffix = rest + &value_suffix;
                    value_str.truncate(i);
                }
            }
        }
        if self.comma && !self.zero {
            value_str = self.locale.group(&value_str, usize::MAX);
        }
        let width = self.spec.width.unwrap_or(0);
        let length = value_prefix.chars().count() + value_str.chars().count() + value_suffix.chars().count();
        let mut padding = if length < width { self.fill.to_string().repeat(width - length) } else { String::new() };
        if self.comma && self.zero {
            let pad_len = padding.chars().count();
            let w = if pad_len > 0 { width.saturating_sub(value_suffix.chars().count()) } else { usize::MAX };
            value_str = self.locale.group(&(padding + &value_str), w);
            padding = String::new();
        }
        let out = match self.align {
            '<' => format!("{value_prefix}{value_str}{value_suffix}{padding}"),
            '=' => format!("{value_prefix}{padding}{value_str}{value_suffix}"),
            '^' => {
                let pad: Vec<char> = padding.chars().collect();
                let half = pad.len() / 2;
                format!("{}{value_prefix}{value_str}{value_suffix}{}", pad[..half].iter().collect::<String>(), pad[half..].iter().collect::<String>())
            }
            _ => format!("{padding}{value_prefix}{value_str}{value_suffix}"),
        };
        match &self.locale.numerals {
            Some(n) => out.chars().map(|c| c.to_digit(10).map_or(c.to_string(), |d| n[d as usize].clone())).collect(),
            None => out,
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.spec.fmt(f)
    }
}

/// A fixed-point formatter with an SI-prefix suffix chosen up front.
#[derive(Debug, Clone)]
pub struct PrefixFormat {
    scale: f64,
    inner: Format,
}

impl PrefixFormat {
    pub fn format(&self, value: f64) -> String {
        self.inner.format(self.scale * value)
    }
}

/// Significant digits and decimal exponent of `x`, e.g. `1.23` → `("123", 0)`.
fn decimal_parts(x: f64, p: Option<usize>) -> Option<(String, i32)> {
    if !x.is_finite() || x == 0.0 {
        return None;
    }
    // As in d3, a precision of 0 means "as many digits as needed".
    let s = match p {
        Some(p) if p > 0 => format!("{:.*e}", p - 1, x),
        _ => format!("{x:e}"),
    };
    let (mantissa, exp) = s.split_once('e').expect("{:e} output has an exponent");
    Some((mantissa.replace('.', "").trim_start_matches('-').to_owned(), exp.parse().expect("integer exponent")))
}

fn exponent(x: f64) -> Option<i32> {
    decimal_parts(x.abs(), None).map(|d| d.1)
}

/// JS `Number.prototype.toPrecision` for a non-negative `x`.
fn to_precision(x: f64, p: usize) -> String {
    if !x.is_finite() {
        return format!("{x}");
    }
    let Some((digits, e)) = decimal_parts(x, Some(p)) else {
        return if p > 1 { format!("0.{}", "0".repeat(p - 1)) } else { "0".into() };
    };
    if e < -6 || e >= p as i32 {
        let frac = if digits.len() > 1 { format!(".{}", &digits[1..]) } else { String::new() };
        format!("{}{frac}e{}{}", &digits[..1], if e < 0 { '-' } else { '+' }, e.abs())
    } else if e >= 0 {
        let e = e as usize;
        if digits.len() > e + 1 { format!("{}.{}", &digits[..=e], &digits[e + 1..]) } else { digits }
    } else {
        format!("0.{}{digits}", "0".repeat((-e - 1) as usize))
    }
}

fn format_rounded(x: f64, p: usize) -> String {
    let Some((c, e)) = decimal_parts(x, Some(p)) else { return format!("{x}") };
    if e < 0 {
        format!("0.{}{c}", "0".repeat((-e - 1) as usize))
    } else if c.len() > e as usize + 1 {
        format!("{}.{}", &c[..=e as usize], &c[e as usize + 1..])
    } else {
        format!("{c}{}", "0".repeat(e as usize + 1 - c.len()))
    }
}

/// SI-prefixed significant digits; returns the prefix exponent if one applies.
fn format_prefix_auto(x: f64, p: usize) -> (String, Option<i32>) {
    let Some((c, e)) = decimal_parts(x, Some(p)) else { return (to_precision(x, p), None) };
    let pe = e.div_euclid(3).clamp(-8, 8) * 3;
    let i = e - pe + 1;
    let n = c.len() as i32;
    let s = if i == n {
        c
    } else if i > n {
        format!("{c}{}", "0".repeat((i - n) as usize))
    } else if i > 0 {
        format!("{}.{}", &c[..i as usize], &c[i as usize..])
    } else {
        let more = decimal_parts(x, Some((p as i32 + i - 1).max(0) as usize)).map(|d| d.0).unwrap_or_default();
        format!("0.{}{more}", "0".repeat((-i) as usize))
    };
    (s, Some(pe))
}

/// Trims insignificant zeros: `1.2000k` → `1.2k`.
fn format_trim(s: &str) -> String {
    let c: Vec<char> = s.chars().collect();
    let (mut i0, mut i1) = (-1i64, 0usize);
    for i in 1..c.len() {
        match c[i] {
            '.' => { i0 = i as i64; i1 = i; }
            '0' => { if i0 == 0 { i0 = i as i64; } i1 = i; }
            ch if ch.is_ascii_digit() => { if i0 > 0 { i0 = 0; } }
            _ => break,
        }
    }
    if i0 > 0 {
        c[..i0 as usize].iter().chain(&c[i1 + 1..]).collect()
    } else {
        s.to_owned()
    }
}

/// Decimal places needed for a fixed-point format at `step`.
pub fn precision_fixed(step: f64) -> f64 {
    exponent(step.abs()).map_or(f64::NAN, |e| (-e).max(0) as f64)
}

/// Decimal places for an SI-prefix format of `value` at `step`.
pub fn precision_prefix(step: f64, value: f64) -> f64 {
    match (exponent(value), exponent(step.abs())) {
        (Some(v), Some(s)) => (v.div_euclid(3).clamp(-8, 8) * 3 - s).max(0) as f64,
        _ => f64::NAN,
    }
}

/// Significant digits for a rounded format of values up to `max` at `step`.
pub fn precision_round(step: f64, max: f64) -> f64 {
    let step = step.abs();
    let max = max.abs() - step;
    match (exponent(max), exponent(step)) {
        (Some(m), Some(s)) => ((m - s).max(0) + 1) as f64,
        _ => f64::NAN,
    }
}
