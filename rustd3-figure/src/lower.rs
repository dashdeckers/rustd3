//! Lowering: rewriting a list into the commands a backend draws natively.
//! A backend states its [`Caps`], and [`lower`] replaces what it lacks.
//! Glyph runs become outlines, markers become fills and sweep gradients
//! become images. Path clips stay in place, and what could not be drawn
//! faithfully is returned as [`Warning`]s.
//!
//! ```
//! use rustd3_figure::kurbo::Size;
//! use rustd3_figure::list::List;
//! use rustd3_figure::lower::{self, Caps};
//!
//! let list = List { size: Size::new(10.0, 10.0), ..List::default() };
//! let lowered = lower::lower(&list, Caps::ALL)?;
//! assert!(lowered.warnings.is_empty());
//! # Ok::<(), rustd3_figure::list::Error>(())
//! ```

mod clip;
mod glyphs;
mod markers;
mod sweep;
mod warning;

use kurbo::Affine;
pub use warning::Warning;

use crate::list::{self, Command, List, Outlines, Part};

/// What a backend draws natively.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Caps {
    /// Glyph runs from font data.
    pub glyphs: bool,
    /// One shape stamped at many points.
    pub markers: bool,
    /// Clips to any path, not only rectangles.
    pub clip_paths: bool,
    /// Sweep gradients.
    pub sweep: bool,
}

impl Caps {
    /// Every capability, as vello_cpu and krilla have.
    pub const ALL: Caps = Caps {
        glyphs: true,
        markers: true,
        clip_paths: true,
        sweep: true,
    };
}

/// A lowered list and what it could not draw faithfully.
#[derive(Debug, Clone, PartialEq)]
pub struct Lowered {
    /// The list, every command of which the backend draws natively (path
    /// clips aside).
    pub list: List,
    /// The warnings, in the order met; each names its input command.
    pub warnings: Vec<Warning>,
}

/// `list` rewritten for a backend with `caps`:
///
/// - without `glyphs`, each glyph run becomes one fill of its outlines;
/// - without `markers`, each point of a markers command becomes a fill;
/// - without `sweep`, a fill or stroke with a sweep gradient becomes a
///   group clipped to what it covers, holding an image of the gradient
///   rendered by vello_cpu at 2 px per list px
///   ([`Warning::SweepRasterized`]). Without the `raster` feature it takes
///   the gradient's middle stop as a solid paint instead
///   ([`Warning::SweepFlattened`]). Glyph runs and markers painted with a
///   sweep become fills first;
/// - without `clip_paths`, a path clip stays in place and is reported
///   ([`Warning::UnsupportedClip`]). This check runs after the sweep pass,
///   so a rasterized sweep reports both.
///
/// Parts are remapped to the new command ranges.
///
/// # Errors
///
/// The [`list::Error`] of [`list::check`] for a list that fails it.
pub fn lower(list: &List, caps: Caps) -> Result<Lowered, list::Error> {
    list::check(list)?;
    let mut outlines = Outlines::default();
    let mut commands = Vec::with_capacity(list.commands.len());
    let mut warnings = Vec::new();
    // Where each input command starts in the output, and the end.
    let mut starts = Vec::with_capacity(list.commands.len() + 1);
    // Each open group's transform into list px, for the sweep's resolution.
    let mut groups = vec![Affine::IDENTITY];
    for (i, command) in list.commands.iter().enumerate() {
        starts.push(commands.len());
        let to_list = *groups
            .last()
            .expect("invariant: the root group is never popped");
        let expanded = match command {
            Command::Glyphs(run) if !caps.glyphs || (!caps.sweep && sweep::is_sweep(&run.fill)) => {
                vec![glyphs::fill(&mut outlines, &list.fonts, run)]
            }
            Command::Markers {
                shape,
                at,
                paint,
                sizes,
                paints,
            } if !caps.markers
                || (!caps.sweep && markers::painted_by_sweep(paint, paints.as_ref())) =>
            {
                markers::fills(shape, at, paint, sizes.as_deref(), paints.as_ref())
            }
            Command::Push { transform, .. } => {
                groups.push(to_list * *transform);
                vec![command.clone()]
            }
            Command::Pop => {
                if groups.len() > 1 {
                    groups.pop();
                }
                vec![command.clone()]
            }
            Command::Fill { .. }
            | Command::Stroke { .. }
            | Command::Markers { .. }
            | Command::Glyphs(_)
            | Command::Image { .. } => vec![command.clone()],
        };
        let first = commands.len();
        for c in expanded {
            if !caps.sweep && sweep::painted(&c) {
                if let Some(warning) = sweep::lower(c, to_list, &mut commands, i) {
                    push_once(&mut warnings, warning);
                }
            } else {
                commands.push(c);
            }
        }
        if !caps.clip_paths && commands[first..].iter().any(clip::is_path) {
            push_once(&mut warnings, Warning::UnsupportedClip { command: i });
        }
    }
    starts.push(commands.len());
    let parts = list
        .parts
        .iter()
        .map(|part| Part {
            commands: starts[part.commands.start]..starts[part.commands.end],
            ..part.clone()
        })
        .collect();
    Ok(Lowered {
        list: List {
            size: list.size,
            fonts: list.fonts.clone(),
            commands,
            parts,
        },
        warnings,
    })
}

/// Adds `warning` unless it repeats the last one (one per input command).
fn push_once(warnings: &mut Vec<Warning>, warning: Warning) {
    if warnings.last() != Some(&warning) {
        warnings.push(warning);
    }
}
