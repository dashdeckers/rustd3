//! Option C: every axes owns one x and one y scale, as in
//! matplotlib and Makie; sharing is declared afterwards as
//! link groups and resolved with union-find.

use crate::fig::{self, AnyScale, Fit, Mark};
use d3::axis::Orient;
use d3::scale;
use std::mem::discriminant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AxesId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    X,
    Y,
}

#[derive(Debug, Clone)]
struct Axes {
    cell: (usize, usize),
    scales: [(AnyScale, Fit); 2],
    lines: Vec<Vec<[f64; 2]>>,
    twin: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Figure {
    axes: Vec<Axes>,
    parent: Vec<usize>,
}

/// Linked axes that disagree; found only at build time.
#[derive(Debug, Clone, PartialEq)]
pub enum Conflict {
    Domains(AxesId, AxesId, [f64; 2], [f64; 2]),
    Kinds(AxesId, AxesId),
}

impl Figure {
    pub fn subplots(
        rows: usize,
        cols: usize,
    ) -> (Self, Vec<Vec<AxesId>>) {
        let mut f = Self::default();
        let ids = (0..rows)
            .map(|r| {
                (0..cols)
                    .map(|c| f.add((r, c), false))
                    .collect()
            })
            .collect();
        (f, ids)
    }

    fn add(&mut self, cell: (usize, usize), twin: bool) -> AxesId {
        let fit = (scale::linear().into(), Fit::Data);
        self.axes.push(Axes {
            cell,
            scales: [fit.clone(), fit],
            lines: Vec::new(),
            twin,
        });
        let n = self.parent.len();
        self.parent.extend([n, n + 1]);
        AxesId(self.axes.len() - 1)
    }

    /// A second axes in the same cell whose x is linked:
    /// matplotlib's `twinx`.
    pub fn twinx(&mut self, a: AxesId) -> AxesId {
        let cell = self.axes[a.0].cell;
        let t = self.add(cell, true);
        self.share(Dir::X, &[a, t]);
        t
    }

    pub fn set_scale(
        &mut self,
        a: AxesId,
        dir: Dir,
        s: impl Into<AnyScale>,
    ) {
        self.axes[a.0].scales[dir as usize] =
            (s.into(), Fit::Given);
    }

    pub fn set_lim(&mut self, a: AxesId, dir: Dir, d: [f64; 2]) {
        let (s, _) = &self.axes[a.0].scales[dir as usize];
        let s = s.clone().with_domain(d);
        self.axes[a.0].scales[dir as usize] = (s, Fit::Given);
    }

    pub fn plot(
        &mut self,
        a: AxesId,
        data: impl IntoIterator<Item = (f64, f64)>,
    ) {
        let pts = data.into_iter().map(|(x, y)| [x, y]);
        self.axes[a.0].lines.push(pts.collect());
    }

    pub fn share(&mut self, dir: Dir, axes: &[AxesId]) {
        let slot = |a: &AxesId| a.0 * 2 + dir as usize;
        for w in axes.windows(2) {
            let (a, b) = (slot(&w[0]), slot(&w[1]));
            let (ra, rb) = (self.find(a), self.find(b));
            self.parent[rb] = ra;
        }
    }

    fn find(&self, mut s: usize) -> usize {
        while self.parent[s] != s {
            s = self.parent[s];
        }
        s
    }

    /// Compiles to option A: one handle per link group.
    pub fn build(&self) -> Result<fig::Figure, Conflict> {
        let mut out = fig::Figure::new();
        let n = self.parent.len();
        let mut handle: Vec<Option<usize>> = vec![None; n];
        let mut owner: Vec<usize> = vec![0; n];
        for s in 0..n {
            let root = self.find(s);
            let (scale, fit) = &self.axes[s / 2].scales[s % 2];
            let Some(h) = handle[root] else {
                let h = out.add::<()>(scale.clone(), *fit);
                handle[root] = Some(h.index());
                owner[root] = s;
                continue;
            };
            let (have, have_fit) = &mut out.scales[h];
            let (a, b) = (AxesId(owner[root] / 2), AxesId(s / 2));
            if discriminant(have) != discriminant(scale) {
                return Err(Conflict::Kinds(a, b));
            }
            match (*have_fit, fit) {
                (Fit::Given, Fit::Given) if have != scale => {
                    let (da, db) = (have.domain(), scale.domain());
                    return Err(Conflict::Domains(a, b, da, db));
                }
                (Fit::Data, Fit::Given) => {
                    (*have, *have_fit) = (scale.clone(), Fit::Given);
                }
                _ => {}
            }
        }
        for (i, ax) in self.axes.iter().enumerate() {
            let [x, y] = [0, 1].map(|d| {
                handle[self.find(i * 2 + d)].expect("resolved")
            });
            let p = out.panel(ax.cell.0, ax.cell.1);
            for points in &ax.lines {
                let points = points.clone();
                p.marks.push(Mark::Line { x, y, points });
            }
            if ax.twin {
                p.axes.push((Orient::Right, y));
            } else {
                p.axes.push((Orient::Bottom, x));
                p.axes.push((Orient::Left, y));
            }
        }
        Ok(out)
    }
}
