//! [`Modifiers`]: the keyboard modifiers held during pointer input.

/// The keyboard modifiers held: shift, ctrl, alt and logo (the command or
/// windows key). Built from the constants with [`union`](Modifiers::union).
///
/// ```
/// use rustd3_core::interact::Modifiers;
///
/// let held = Modifiers::SHIFT.union(Modifiers::CTRL);
/// assert!(held.get_shift() && held.get_ctrl() && !held.get_alt());
/// assert_eq!(Modifiers::default(), Modifiers::NONE);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Modifiers {
    shift: bool,
    ctrl: bool,
    alt: bool,
    logo: bool,
}

impl Modifiers {
    /// No modifier held.
    pub const NONE: Self = Modifiers {
        shift: false,
        ctrl: false,
        alt: false,
        logo: false,
    };
    /// Shift alone.
    pub const SHIFT: Self = Modifiers {
        shift: true,
        ..Modifiers::NONE
    };
    /// Ctrl alone.
    pub const CTRL: Self = Modifiers {
        ctrl: true,
        ..Modifiers::NONE
    };
    /// Alt alone.
    pub const ALT: Self = Modifiers {
        alt: true,
        ..Modifiers::NONE
    };
    /// Logo (command or windows key) alone.
    pub const LOGO: Self = Modifiers {
        logo: true,
        ..Modifiers::NONE
    };

    /// The modifiers held in either.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Modifiers {
            shift: self.shift || other.shift,
            ctrl: self.ctrl || other.ctrl,
            alt: self.alt || other.alt,
            logo: self.logo || other.logo,
        }
    }

    /// Whether shift is held.
    #[must_use]
    pub fn get_shift(self) -> bool {
        self.shift
    }

    /// Whether ctrl is held.
    #[must_use]
    pub fn get_ctrl(self) -> bool {
        self.ctrl
    }

    /// Whether alt is held.
    #[must_use]
    pub fn get_alt(self) -> bool {
        self.alt
    }

    /// Whether logo is held.
    #[must_use]
    pub fn get_logo(self) -> bool {
        self.logo
    }
}
