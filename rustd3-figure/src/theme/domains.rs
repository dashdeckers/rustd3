//! [`Domains`]: how fitted domains end.

/// How the domains of fitted continuous and time handles end.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Domains {
    /// This share of the span added per side, stopped by sticky values
    /// (bars and areas at their base), as matplotlib's and seaborn's
    /// margins; `Margin(0.05)` by default. Shares outside [0, 0.5) are
    /// rejected by [`Theme::domains`](crate::Theme::domains).
    Margin(f64),
    /// Rounded outward to nice tick values, as d3's `nice(count)`, with
    /// the count of the first axis that shows the handle. That count is
    /// its `.ticks(n)`, or the one the first layout pass gives its length,
    /// or d3's 10 for a handle no axis shows. Sticky values still stop the
    /// rounding. Time handles floor and ceil to the tick interval for that
    /// count.
    /// Windows (`Fit::Window`) and `Fit::Visible` handles fit without a
    /// margin and are not rounded.
    Nice,
}
