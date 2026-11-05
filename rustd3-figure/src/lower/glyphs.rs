//! Glyph runs as fills of their outlines.

use peniko::FontData;

use crate::list::{Command, GlyphRun, Outlines};

/// One fill of the run's outlines in its group's space, painted as the
/// run; a glyph without an outline (a space) adds nothing.
pub(super) fn fill(outlines: &mut Outlines, fonts: &[FontData], run: &GlyphRun) -> Command {
    Command::Fill {
        path: outlines.run(fonts, run),
        paint: run.fill.clone(),
    }
}
