//! [`Layers`]: a figure's drawing list split into what each plot area's
//! tiles show, which moves with the area's data, and the rest, which stays
//! put. The cells outside the plot areas show the rest, and each plot area
//! shows its part of it over its tiles.
//!
//! The widget draws its list with an overscan ([`Held::overscan`]). Each
//! panel's background and grid lines cover its plot area grown by the
//! overscan, and its marks are clipped to the grown area, so they reach
//! into the gutters around it. A tile must show only its plot area's own
//! content there. It must not carry what stays put when the content moves
//! (the axis lines whose inner half lies on the plot area's edge pixels, a
//! border, an inset legend), or a translated tile would move it too. So
//! the list is split by part:
//!
//! - a panel's **layer**: the figure's background, the panel's background,
//!   its grid lines and its marks, in the group the figure translates to
//!   the plot area and clips to it, as the list has them (one run of
//!   commands from the panel's background to the group's end);
//! - the **chrome**: every other part, all drawn outside any group.
//!
//! Parts are told apart by their ids, which rustd3-figure composes as
//! `role << 56 | panel << 32 | index`
//! (`rustd3-figure/src/figure/part.rs`): role 0 is the figure's background,
//! 2 a panel's background, 6 a mark and 9 a panel's grid lines.
//!
//! [`Held::overscan`]: rustd3_figure::Held::overscan

use std::sync::Arc;

use rustd3_figure::list::{self, Bounds, Command, List, Outlines, Part, part};

use crate::plot::style;

/// The role of the figure's background, a panel's background, an axis's
/// ticks and labels, a mark, a legend and a panel's grid lines in a part
/// id's top byte.
const BACKGROUND: u64 = 0;
const PANEL: u64 = 2;
pub(crate) const AXIS: u64 = 4;
const MARK: u64 = 6;
pub(crate) const LEGEND: u64 = 7;
const GRID: u64 = 9;

/// A list a job renders, with its bounds.
#[derive(Clone)]
pub(crate) struct Source {
    pub(crate) list: Arc<List>,
    pub(crate) bounds: Arc<Bounds>,
}

/// One plot area's layer: the list its tiles render, a hash of its
/// parts, and a hash of their looks without their positions
/// ([`style::hash`]).
pub(crate) struct Layer {
    pub(crate) source: Source,
    pub(crate) hash: u64,
    pub(crate) style: u64,
}

/// A figure's list split into the plot areas' layers and the chrome.
pub(crate) struct Layers {
    /// Everything but the layers, which the cells outside the plot areas
    /// show.
    pub(crate) chrome: Source,
    /// The chrome without the figure's background, transparent elsewhere.
    /// Each plot area shows it over its tiles.
    pub(crate) fixed: Source,
    /// Per panel, its layer.
    pub(crate) layers: Vec<Layer>,
}

impl Layers {
    /// Splits `list`, drawn for `panels` panels, its glyph outlines read
    /// through `outlines`.
    pub(crate) fn split(list: &List, panels: usize, outlines: &mut Outlines) -> Layers {
        let background: Vec<&Part> = list
            .parts
            .iter()
            .filter(|p| role(p.id) == BACKGROUND)
            .collect();
        let layers = (0..panels)
            .map(|panel| {
                let moving: Vec<&Part> = list
                    .parts
                    .iter()
                    .filter(|p| moves(p.id) == Some(panel))
                    .collect();
                let mut parts = background.clone();
                let mut runs = background
                    .iter()
                    .map(|p| p.commands.clone())
                    .collect::<Vec<_>>();
                if let (Some(first), Some(last)) = (moving.first(), moving.last()) {
                    // The marks' group ends with a Pop outside every part.
                    let mut end = last.commands.end;
                    if role(last.id) == MARK && matches!(list.commands.get(end), Some(Command::Pop))
                    {
                        end += 1;
                    }
                    runs.push(first.commands.start..end);
                }
                parts.extend(&moving);
                let hash = parts.iter().fold(FNV, |hash, p| fnv(hash, p.hash));
                let style = parts
                    .iter()
                    .fold(FNV, |hash, p| fnv(hash, style::hash(list, p)));
                let source = Source::of(sub(list, &runs, &parts), outlines);
                Layer {
                    source,
                    hash,
                    style,
                }
            })
            .collect();
        let still: Vec<&Part> = list
            .parts
            .iter()
            .filter(|p| moves(p.id).is_none())
            .collect();
        let chrome = sub(list, &runs(&still), &still);
        let fixed: Vec<&Part> = still
            .iter()
            .copied()
            .filter(|p| role(p.id) != BACKGROUND)
            .collect();
        let fixed = sub(list, &runs(&fixed), &fixed);
        Layers {
            chrome: Source::of(chrome, outlines),
            fixed: Source::of(fixed, outlines),
            layers,
        }
    }
}

impl Source {
    /// `list` and its bounds, shared.
    fn of(list: List, outlines: &mut Outlines) -> Source {
        let bounds = Bounds::of(&list, outlines);
        Source {
            list: Arc::new(list),
            bounds: Arc::new(bounds),
        }
    }
}

/// A part id's value. rustd3-figure keeps it private and orders ids by it,
/// so it is the greatest value whose id is not past `id`.
fn value(id: part::Id) -> u64 {
    let (mut low, mut high) = (0_u64, u64::MAX);
    while low < high {
        let middle = low + (high - low).div_ceil(2);
        if part::Id::new(middle) <= id {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    low
}

/// The role, panel and index a part id packs: `role << 56 | panel << 32 |
/// index`.
pub(crate) fn decode(id: part::Id) -> (u64, usize, usize) {
    let v = value(id);
    let field = |shift: u32, bits: u32| {
        usize::try_from((v >> shift) & ((1 << bits) - 1)).expect("invariant: at most 32 bits")
    };
    (v >> 56, field(32, 24), field(0, 32))
}

/// The role of a part: the top byte of its id.
fn role(id: part::Id) -> u64 {
    decode(id).0
}

/// The panel of a part that moves with a plot area's data (a panel's
/// background, grid lines or marks). `None` for any other part.
fn moves(id: part::Id) -> Option<usize> {
    let (role, panel, _) = decode(id);
    [PANEL, MARK, GRID].contains(&role).then_some(panel)
}

/// The command runs of `parts`.
fn runs(parts: &[&Part]) -> Vec<std::ops::Range<usize>> {
    parts.iter().map(|p| p.commands.clone()).collect()
}

/// The list of `list`'s commands in `runs`, in order, with `parts` (each
/// inside one run) moved to where their commands land.
fn sub(list: &List, runs: &[std::ops::Range<usize>], parts: &[&Part]) -> List {
    let mut commands = Vec::with_capacity(runs.iter().map(ExactSizeIterator::len).sum());
    let mut moved = Vec::with_capacity(parts.len());
    for run in runs {
        let at = commands.len();
        commands.extend_from_slice(&list.commands[run.clone()]);
        for p in parts
            .iter()
            .filter(|p| run.start <= p.commands.start && p.commands.end <= run.end)
        {
            let start = at + (p.commands.start - run.start);
            moved.push(Part {
                commands: start..start + p.commands.len(),
                ..(*p).clone()
            });
        }
    }
    let sub = List {
        size: list.size,
        fonts: list.fonts.clone(),
        commands,
        parts: moved,
    };
    debug_assert!(
        list::check(&sub).is_ok(),
        "a sub-list keeps its groups whole"
    );
    sub
}

/// FNV-1a 64's offset basis.
pub(crate) const FNV: u64 = 0xcbf2_9ce4_8422_2325;

/// `hash` with the eight bytes of `word` folded in, FNV-1a 64.
pub(crate) fn fnv(hash: u64, word: u64) -> u64 {
    word.to_le_bytes().iter().fold(hash, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

#[cfg(test)]
mod tests {
    use rustd3_figure::kurbo::Size;
    use rustd3_figure::list::{Command, Outlines, part};
    use rustd3_figure::{Held, Target};

    use super::{Layers, decode, moves, role};
    use crate::plot::{fixture, state};

    #[test]
    fn part_ids_tell_role_and_panel() {
        let id =
            |role: u64, panel: u64, index: u64| part::Id::new(role << 56 | panel << 32 | index);
        assert_eq!(role(id(0, 0, 0)), 0);
        assert_eq!(role(id(6, 3, 7)), 6);
        assert_eq!(moves(id(6, 3, 7)), Some(3));
        assert_eq!(moves(id(2, 0, 0)), Some(0));
        assert_eq!(moves(id(9, 1, 0)), Some(1));
        assert_eq!(moves(id(4, 1, 0)), None, "an axis stays");
        assert_eq!(moves(id(8, 0, 1)), None, "a spine stays");
        assert_eq!(moves(id(7, 0, 0)), None, "an inset legend stays");
        assert_eq!(decode(id(4, 3, 1)), (4, 3, 1));
        assert_eq!(decode(id(7, (1 << 24) - 1, 2)), (7, (1 << 24) - 1, 2));
        assert_eq!(
            decode(id(255, 0, u64::from(u32::MAX))),
            (255, 0, u32::MAX as usize)
        );
    }

    #[test]
    fn layers_and_chrome_hold_every_command_once() {
        let fig = fixture::grid();
        let mut cache = state::cache();
        let mut held = Held::default();
        held.overscan(100.0);
        let list = fig.draw_with(
            &mut cache,
            Size::new(640.0, 480.0),
            Target::Screen { scale: 1.0 },
            &held,
        );
        let layers = Layers::split(&list, 4, &mut Outlines::default());
        // Every layer starts with the figure's background. Past it, the
        // layers and the chrome hold each command once, with the marks'
        // groups whole.
        let background = list.parts[0].commands.len();
        let layered: usize = layers
            .layers
            .iter()
            .map(|layer| layer.source.list.commands.len() - background)
            .sum();
        assert_eq!(
            layered + layers.chrome.list.commands.len(),
            list.commands.len()
        );
        assert_eq!(
            layers.fixed.list.commands.len() + background,
            layers.chrome.list.commands.len()
        );
        for layer in &layers.layers {
            let pushes = layer
                .source
                .list
                .commands
                .iter()
                .filter(|c| matches!(c, Command::Push { .. }))
                .count();
            assert_eq!(pushes, 1, "one group of marks per panel");
        }
        assert!(
            layers
                .chrome
                .list
                .parts
                .iter()
                .all(|p| moves(p.id).is_none())
        );
    }
}
