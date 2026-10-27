//! Part identities of a figure's elements.

use crate::list::part;

/// What a figure element is: the top byte of its part id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Role {
    /// The figure's background.
    Background = 0,
    /// The figure title.
    Title = 1,
    /// A panel's background.
    Panel = 2,
    /// A panel's border.
    Border = 3,
    /// An axis of a panel.
    Axis = 4,
    /// A panel's title.
    Heading = 5,
}

/// The id of element `index` of `role` in panel `panel`: `role << 56 |
/// panel << 32 | index`, stable across draws of the same figure.
pub(super) fn id(role: Role, panel: usize, index: usize) -> part::Id {
    debug_assert!(
        panel < 1 << 24 && index < 1 << 32,
        "a part index fits its field"
    );
    part::Id::new((role as u64) << 56 | (panel as u64) << 32 | index as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_part_id_packs_role_panel_and_index() {
        assert_eq!(id(Role::Axis, 3, 1), part::Id::new(0x0400_0003_0000_0001));
        assert_eq!(id(Role::Background, 0, 0), part::Id::new(0));
    }
}
