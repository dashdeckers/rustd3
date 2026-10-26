//! SVG with glyph outlines: each glyph used is defined
//! once at 1000 units per em, and runs `<use>` them.
//! Needs no fonts in the viewer, and draws exactly the
//! glyphs typst measured.

use crate::outline::Outlines;
use figmodel::plot::draw::{Command, GlyphRun, List, svg_d, svg_matrix};
use kurbo::Affine;
use std::collections::BTreeSet;
use std::fmt::Write;

pub fn outlined(l: &List, o: &mut Outlines) -> String {
    let used: BTreeSet<(usize, u32)> = l
        .commands
        .iter()
        .filter_map(|c| match c {
            Command::Glyphs(r) => Some(r),
            _ => None,
        })
        .flat_map(|r| r.glyphs.iter().map(|g| (r.font, g.id)))
        .collect();
    let mut defs = String::from("<defs>");
    for (f, id) in used {
        if let Some(p) = o.get(&l.fonts, f, id) {
            let p = Affine::scale(1000.0) * p.clone();
            let _ = write!(
                defs,
                "<path id='g{f}-{id}' d='{}'/>",
                svg_d(&p)
            );
        }
    }
    defs += "</defs>";
    l.svg_custom(&defs, |r: &GlyphRun, s: &mut String| {
        let [cr, cg, cb, _] = r.fill.to_rgba8().to_u8_array();
        let k = r.size as f64 / 1000.0;
        let _ = write!(
            s,
            "<g transform='{} scale({k})' \
             fill='#{cr:02x}{cg:02x}{cb:02x}'>",
            svg_matrix(&r.transform)
        );
        for g in &r.glyphs {
            let _ = write!(
                s,
                "<use xlink:href='#g{}-{}' x='{:.0}' y='{:.0}'/>",
                r.font,
                g.id,
                g.x as f64 / k,
                g.y as f64 / k
            );
        }
        s.push_str("</g>");
    })
}
