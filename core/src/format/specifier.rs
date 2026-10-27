//! d3-format's specifier mini-language,
//! `[[fill]align][sign][symbol][0][width][,][.precision][~][type]`.

use std::fmt;
use std::str::FromStr;

/// The widest padding a specifier may ask for: V8's maximum string length
/// on 64-bit targets, 2²⁹ − 24, beyond which d3's padding throws a
/// `RangeError` instead of allocating.
const MAX_WIDTH: usize = (1 << 29) - 24;

/// A parsed `[[fill]align][sign][symbol][0][width][,][.precision][~][type]`.
#[derive(Debug, Clone, PartialEq)]
pub struct Specifier {
    fill: char,
    align: Align,
    sign: Sign,
    symbol: Option<Symbol>,
    zero: bool,
    width: Option<usize>,
    comma: bool,
    precision: Option<usize>,
    trim: bool,
    kind: Option<Kind>,
}

/// Where padding goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    /// `<`: after the value.
    Left,
    /// `>`: before the value (the default).
    Right,
    /// `^`: split around the value.
    Center,
    /// `=`: between the sign and the digits.
    AfterSign,
}

/// Which signs are written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    /// `-`: a minus for negative values only (the default).
    Minus,
    /// `+`: a plus for zero and positive values.
    Plus,
    /// `(`: parentheses around negative values.
    Parens,
    /// ` `: a space for zero and positive values.
    Space,
}

/// The `$` and `#` options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symbol {
    /// `$`: the locale's currency prefix and suffix.
    Currency,
    /// `#`: `0b`, `0o` or `0x` before binary, octal and hexadecimal.
    Base,
}

/// The format type: the letter at the end of a specifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// `e`: exponent notation.
    Exponent,
    /// `f`: fixed point.
    Fixed,
    /// `g`: decimal or exponent notation, rounded to significant digits.
    General,
    /// `r`: decimal notation, rounded to significant digits.
    Rounded,
    /// `s`: decimal notation with an SI prefix, rounded to significant digits.
    Si,
    /// `%`: times 100, fixed point, with a percent sign.
    Percent,
    /// `p`: times 100, rounded to significant digits, with a percent sign.
    PercentRounded,
    /// `b`: binary, rounded to an integer.
    Binary,
    /// `o`: octal, rounded to an integer.
    Octal,
    /// `d`: decimal, rounded to an integer.
    Decimal,
    /// `x`: lowercase hexadecimal, rounded to an integer.
    Hex,
    /// `X`: uppercase hexadecimal, rounded to an integer.
    HexUpper,
    /// `c`: the value as text, unformatted.
    Char,
    /// `n`: `,g`.
    Number,
    /// Any other letter: d3 formats it as `~g` with precision 12.
    Other(char),
}

/// A format specifier that does not parse; it carries the specifier as
/// given.
///
/// ```
/// use rustd3_core::format;
///
/// let error = format::format("10..f").unwrap_err();
/// assert_eq!(error.input(), "10..f");
/// assert_eq!(error.to_string(), "invalid format: 10..f");
/// ```
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("invalid format: {0}")]
pub struct InvalidFormat(String);

impl Default for Specifier {
    /// The specifier `""` parses to.
    fn default() -> Self {
        Specifier {
            fill: ' ',
            align: Align::Right,
            sign: Sign::Minus,
            symbol: None,
            zero: false,
            width: None,
            comma: false,
            precision: None,
            trim: false,
            kind: None,
        }
    }
}

impl FromStr for Specifier {
    type Err = InvalidFormat;

    /// d3's formatSpecifier: the regular expression
    /// `^(?:(.)?([<>=^]))?([+\-( ])?([$#])?(0)?(\d+)?(,)?(\.\d+)?(~)?([a-z%])?$`
    /// (case-insensitive), with `.` matching no line terminator. A width
    /// above 2²⁹ − 24 = 536,870,888 is an error too, where d3 parses it
    /// and throws a `RangeError` when it pads.
    fn from_str(text: &str) -> Result<Self, InvalidFormat> {
        let c: Vec<char> = text.chars().collect();
        let at = |i: usize| c.get(i).copied();
        let mut spec = Specifier::default();
        let mut i = 0;
        if let (Some(fill), Some(align)) = (at(0), at(1).and_then(Align::parse))
            && !matches!(fill, '\n' | '\r' | '\u{2028}' | '\u{2029}')
        {
            (spec.fill, spec.align, i) = (fill, align, 2);
        } else if let Some(align) = at(0).and_then(Align::parse) {
            (spec.align, i) = (align, 1);
        }
        if let Some(sign) = at(i).and_then(Sign::parse) {
            (spec.sign, i) = (sign, i + 1);
        }
        if let Some(symbol) = at(i).and_then(Symbol::parse) {
            (spec.symbol, i) = (Some(symbol), i + 1);
        }
        if at(i) == Some('0') {
            (spec.zero, i) = (true, i + 1);
        }
        spec.width = digits(&c, &mut i);
        if spec.width.is_some_and(|width| width > MAX_WIDTH) {
            return Err(InvalidFormat::new(text));
        }
        if at(i) == Some(',') {
            (spec.comma, i) = (true, i + 1);
        }
        if at(i) == Some('.') {
            i += 1;
            spec.precision = Some(digits(&c, &mut i).ok_or_else(|| InvalidFormat::new(text))?);
        }
        if at(i) == Some('~') {
            (spec.trim, i) = (true, i + 1);
        }
        if let Some(kind) = at(i).and_then(Kind::parse) {
            (spec.kind, i) = (Some(kind), i + 1);
        }
        if i == c.len() {
            Ok(spec)
        } else {
            Err(InvalidFormat::new(text))
        }
    }
}

impl fmt::Display for Specifier {
    /// d3's FormatSpecifier.toString: the normalized specifier.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}", self.fill, self.align, self.sign)?;
        if let Some(symbol) = self.symbol {
            write!(f, "{symbol}")?;
        }
        if self.zero {
            f.write_str("0")?;
        }
        if let Some(width) = self.width {
            write!(f, "{}", width.max(1))?;
        }
        if self.comma {
            f.write_str(",")?;
        }
        if let Some(precision) = self.precision {
            write!(f, ".{precision}")?;
        }
        if self.trim {
            f.write_str("~")?;
        }
        if let Some(kind) = self.kind {
            write!(f, "{kind}")?;
        }
        Ok(())
    }
}

impl fmt::Display for Align {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self {
            Align::Left => '<',
            Align::Right => '>',
            Align::Center => '^',
            Align::AfterSign => '=',
        };
        write!(f, "{c}")
    }
}

impl fmt::Display for Sign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self {
            Sign::Minus => '-',
            Sign::Plus => '+',
            Sign::Parens => '(',
            Sign::Space => ' ',
        };
        write!(f, "{c}")
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self {
            Symbol::Currency => '$',
            Symbol::Base => '#',
        };
        write!(f, "{c}")
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self {
            Kind::Exponent => 'e',
            Kind::Fixed => 'f',
            Kind::General => 'g',
            Kind::Rounded => 'r',
            Kind::Si => 's',
            Kind::Percent => '%',
            Kind::PercentRounded => 'p',
            Kind::Binary => 'b',
            Kind::Octal => 'o',
            Kind::Decimal => 'd',
            Kind::Hex => 'x',
            Kind::HexUpper => 'X',
            Kind::Char => 'c',
            Kind::Number => 'n',
            Kind::Other(c) => *c,
        };
        write!(f, "{c}")
    }
}

impl InvalidFormat {
    /// The error for `specifier`, as given.
    fn new(specifier: &str) -> Self {
        InvalidFormat(specifier.to_owned())
    }

    /// The specifier that did not parse.
    pub fn input(&self) -> &str {
        &self.0
    }
}

impl Specifier {
    /// The fill character (default ` `).
    pub fn fill(&self) -> char {
        self.fill
    }

    /// The alignment (default [`Align::Right`]).
    pub fn align(&self) -> Align {
        self.align
    }

    /// The sign option (default [`Sign::Minus`]).
    pub fn sign(&self) -> Sign {
        self.sign
    }

    /// The `$` or `#` option.
    pub fn symbol(&self) -> Option<Symbol> {
        self.symbol
    }

    /// Whether `0` asks for zero padding.
    pub fn zero(&self) -> bool {
        self.zero
    }

    /// The minimum width.
    pub fn width(&self) -> Option<usize> {
        self.width
    }

    /// Whether `,` asks for grouping.
    pub fn comma(&self) -> bool {
        self.comma
    }

    /// The precision.
    pub fn precision(&self) -> Option<usize> {
        self.precision
    }

    /// Whether `~` asks for insignificant zeros to be trimmed.
    pub fn trim(&self) -> bool {
        self.trim
    }

    /// The type, if one is given.
    pub fn kind(&self) -> Option<Kind> {
        self.kind
    }

    /// d3-scale's tickFormat sets these on a parsed specifier.
    pub fn with_precision(self, precision: usize) -> Self {
        Specifier {
            precision: Some(precision),
            ..self
        }
    }

    /// The specifier with `~` set or cleared.
    pub fn with_trim(self, trim: bool) -> Self {
        Specifier { trim, ..self }
    }

    /// The specifier with another type: formatPrefix's `f`.
    pub(super) fn with_kind(self, kind: Option<Kind>) -> Self {
        Specifier { kind, ..self }
    }
}

impl Align {
    fn parse(c: char) -> Option<Align> {
        match c {
            '<' => Some(Align::Left),
            '>' => Some(Align::Right),
            '^' => Some(Align::Center),
            '=' => Some(Align::AfterSign),
            _ => None,
        }
    }
}

impl Sign {
    fn parse(c: char) -> Option<Sign> {
        match c {
            '-' => Some(Sign::Minus),
            '+' => Some(Sign::Plus),
            '(' => Some(Sign::Parens),
            ' ' => Some(Sign::Space),
            _ => None,
        }
    }
}

impl Symbol {
    fn parse(c: char) -> Option<Symbol> {
        match c {
            '$' => Some(Symbol::Currency),
            '#' => Some(Symbol::Base),
            _ => None,
        }
    }
}

impl Kind {
    /// d3's type letters are case-sensitive; any other ASCII letter is kept
    /// as [`Kind::Other`], as d3's case-insensitive pattern accepts it.
    fn parse(c: char) -> Option<Kind> {
        let kind = match c {
            'e' => Kind::Exponent,
            'f' => Kind::Fixed,
            'g' => Kind::General,
            'r' => Kind::Rounded,
            's' => Kind::Si,
            '%' => Kind::Percent,
            'p' => Kind::PercentRounded,
            'b' => Kind::Binary,
            'o' => Kind::Octal,
            'd' => Kind::Decimal,
            'x' => Kind::Hex,
            'X' => Kind::HexUpper,
            'c' => Kind::Char,
            'n' => Kind::Number,
            c if c.is_ascii_alphabetic() => Kind::Other(c),
            _ => return None,
        };
        Some(kind)
    }
}

/// A run of ASCII digits at `*i` as a number (saturating), advancing `*i`.
fn digits(c: &[char], i: &mut usize) -> Option<usize> {
    let start = *i;
    let mut n = 0usize;
    while let Some(d) = c.get(*i).and_then(|c| c.to_digit(10)) {
        let d = usize::try_from(d).unwrap_or(usize::MAX);
        n = n.saturating_mul(10).saturating_add(d);
        *i += 1;
    }
    (*i > start).then_some(n)
}
