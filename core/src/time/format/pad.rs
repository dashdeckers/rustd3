//! [`Pad`], how a directive pads its number: d3's `pad` (`locale.js:393–398`).

/// How a directive pads its number to its width: d3's flags `-` (none),
/// `_` (spaces) and `0` (zeros).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Pad {
    /// `%-d`: no padding.
    None,
    /// `%_d`, and `%e` by default: spaces.
    Space,
    /// `%0d`, and every directive but `%e` by default: zeros.
    Zero,
}

impl Pad {
    /// The pad a flag character stands for.
    pub(crate) fn from_flag(c: char) -> Option<Pad> {
        match c {
            '-' => Some(Pad::None),
            '_' => Some(Pad::Space),
            '0' => Some(Pad::Zero),
            _ => None,
        }
    }

    /// d3's `pad(value, fill, width)`: the sign, then the digits of
    /// `|value|` filled on the left to `width`; `NaN` for a value that left
    /// jiff's range, as d3 writes an Invalid Date's.
    pub(crate) fn write(self, value: Option<i64>, width: usize, out: &mut String) {
        let Some(value) = value else {
            out.push_str("NaN");
            return;
        };
        if value < 0 {
            out.push('-');
        }
        let digits = value.unsigned_abs().to_string();
        let fill = match self {
            Pad::None => None,
            Pad::Space => Some(' '),
            Pad::Zero => Some('0'),
        };
        if let Some(fill) = fill {
            out.extend(std::iter::repeat_n(
                fill,
                width.saturating_sub(digits.len()),
            ));
        }
        out.push_str(&digits);
    }
}
