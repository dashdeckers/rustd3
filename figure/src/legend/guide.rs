//! [`Guide`]: what a figure slot or a panel corner holds.

use crate::colorbar;
use crate::legend::Draw;
use crate::scale::ScaleRef;

/// What a figure slot ([`Figure::legend`](crate::Figure::legend)) or a
/// panel corner ([`Panel::legend`](crate::Panel::legend)) holds: a
/// [`Legend`](crate::legend::Legend) of an ordinal palette's keys or a
/// [`Colorbar`](crate::colorbar::Colorbar) of a continuous or binned color
/// scale. Implemented by those two only.
pub trait Guide<'a> {
    /// The guide as the figure keeps it. Its type cannot be named outside
    /// this crate, so no other crate implements `Guide`.
    #[doc(hidden)]
    fn erase(self) -> Erased<'a>;
}

/// A guide as the figure keeps it, its type erased.
pub struct Erased<'a>(pub(crate) Stored<'a>);

/// A guide with its type erased: a legend's keys or a colorbar.
pub(crate) enum Stored<'a> {
    /// A legend of an ordinal palette's keys.
    Keys(Box<dyn Draw>),
    /// A colorbar.
    Bar(Box<dyn colorbar::Draw<'a> + 'a>),
}

impl Stored<'_> {
    /// The handle the guide is drawn from.
    pub(crate) fn handles(&self) -> [ScaleRef<()>; 1] {
        match self {
            Stored::Keys(keys) => [keys.scale()],
            Stored::Bar(bar) => [bar.scale()],
        }
    }
}
