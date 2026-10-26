//! Option B: a layout tree whose nodes say which channels
//! their children share, as Vega-Lite's `resolve`. Concat
//! nodes default to independent, layers to shared.

use crate::fig::{self, AnyScale, Fit, Mark};
use d3::axis::Orient;
use d3::scale;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolve {
    Shared,
    Independent,
}

/// One chart: its own x and y scale and its lines.
#[derive(Debug, Clone)]
pub struct View {
    scales: [(AnyScale, Fit); 2],
    lines: Vec<Vec<[f64; 2]>>,
}

pub fn view() -> View {
    let fit = (scale::linear().into(), Fit::Data);
    View { scales: [fit.clone(), fit], lines: vec![] }
}

impl View {
    pub fn line(
        mut self,
        data: impl IntoIterator<Item = (f64, f64)>,
    ) -> Self {
        let pts = data.into_iter().map(|(x, y)| [x, y]);
        self.lines.push(pts.collect());
        self
    }

    pub fn x(mut self, s: impl Into<AnyScale>) -> Self {
        self.scales[0] = (s.into(), Fit::Given);
        self
    }
}

#[derive(Debug, Clone)]
pub enum Node {
    View(View),
    Layer(Vec<View>, [Resolve; 2]),
    Concat(bool, Vec<Node>, [Resolve; 2]),
}

impl From<View> for Node {
    fn from(v: View) -> Self {
        Node::View(v)
    }
}

pub fn layer(views: impl IntoIterator<Item = View>) -> Node {
    let r = [Resolve::Shared; 2];
    Node::Layer(views.into_iter().collect(), r)
}

pub fn hconcat(
    nodes: impl IntoIterator<Item = impl Into<Node>>,
) -> Node {
    let n = nodes.into_iter().map(Into::into).collect();
    Node::Concat(true, n, [Resolve::Independent; 2])
}

pub fn vconcat(
    nodes: impl IntoIterator<Item = impl Into<Node>>,
) -> Node {
    let n = nodes.into_iter().map(Into::into).collect();
    Node::Concat(false, n, [Resolve::Independent; 2])
}

impl Node {
    pub fn resolve_x(self, r: Resolve) -> Self {
        self.resolve(0, r)
    }

    pub fn resolve_y(self, r: Resolve) -> Self {
        self.resolve(1, r)
    }

    fn resolve(mut self, ch: usize, r: Resolve) -> Self {
        match &mut self {
            Node::Layer(_, rs) | Node::Concat(_, _, rs) => {
                rs[ch] = r
            }
            Node::View(_) => {}
        }
        self
    }

    /// Rows and columns of grid cells the subtree spans.
    fn size(&self) -> (usize, usize) {
        match self {
            Node::View(_) | Node::Layer(..) => (1, 1),
            Node::Concat(horizontal, kids, _) => {
                let s = kids.iter().map(Node::size);
                let (r, c): (Vec<_>, Vec<_>) = s.unzip();
                if *horizontal {
                    (r.into_iter().max().unwrap_or(1), c.iter().sum())
                } else {
                    (r.iter().sum(), c.into_iter().max().unwrap_or(1))
                }
            }
        }
    }
}

/// A placed view with one scale slot per channel.
struct Placed<'a> {
    view: &'a View,
    cell: (usize, usize),
    slot: [usize; 2],
    right: bool,
}

#[derive(Default)]
struct Build<'a> {
    placed: Vec<Placed<'a>>,
    parent: Vec<usize>,
}

impl<'a> Build<'a> {
    fn find(&self, mut s: usize) -> usize {
        while self.parent[s] != s {
            s = self.parent[s];
        }
        s
    }

    fn slot(&mut self) -> usize {
        self.parent.push(self.parent.len());
        self.parent.len() - 1
    }

    /// Places the subtree; returns, per channel, the scale
    /// slots it exposes to its parent.
    fn walk(
        &mut self,
        n: &'a Node,
        at: (usize, usize),
    ) -> [Vec<usize>; 2] {
        match n {
            Node::View(v) => {
                let slot = [self.slot(), self.slot()];
                let right = false;
                let cell = at;
                self.placed.push(Placed { view: v, cell, slot, right });
                slot.map(|s| vec![s])
            }
            Node::Layer(views, rs) => {
                let kids: Vec<_> = views
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        let slot = [self.slot(), self.slot()];
                        let right =
                            i > 0 && rs[1] == Resolve::Independent;
                        let cell = at;
                        self.placed.push(Placed {
                            view: v,
                            cell,
                            slot,
                            right,
                        });
                        slot.map(|s| vec![s])
                    })
                    .collect();
                self.merge(kids, rs)
            }
            Node::Concat(horizontal, nodes, rs) => {
                let mut at = at;
                let mut kids = Vec::new();
                for k in nodes {
                    kids.push(self.walk(k, at));
                    let (r, c) = k.size();
                    if *horizontal {
                        at.1 += c;
                    } else {
                        at.0 += r;
                    }
                }
                self.merge(kids, rs)
            }
        }
    }

    /// Vega-Lite's rule: a shared channel merges children
    /// that each expose one scale; others keep theirs.
    fn merge(
        &mut self,
        kids: Vec<[Vec<usize>; 2]>,
        rs: &[Resolve; 2],
    ) -> [Vec<usize>; 2] {
        [0, 1].map(|ch| {
            let all: Vec<usize> = kids
                .iter()
                .flat_map(|k| k[ch].clone())
                .collect();
            let single = kids.iter().all(|k| k[ch].len() == 1);
            if rs[ch] == Resolve::Shared && single {
                let root = self.find(all[0]);
                for s in &all[1..] {
                    let r = self.find(*s);
                    self.parent[r] = root;
                }
                vec![root]
            } else {
                all
            }
        })
    }
}

/// Compiles the tree to option A.
pub fn build(root: &Node) -> fig::Figure {
    let mut b = Build::default();
    b.walk(root, (0, 0));
    let mut out = fig::Figure::new();
    let mut handle = vec![None; b.parent.len()];
    for p in &b.placed {
        for ch in 0..2 {
            let r = b.find(p.slot[ch]);
            let (s, fit) = &p.view.scales[ch];
            let h = handle[r].get_or_insert_with(|| {
                out.add::<()>(s.clone(), *fit).index()
            });
            let _ = h;
        }
    }
    for p in &b.placed {
        let [x, y] = p.slot.map(|s| handle[b.find(s)].unwrap());
        let panel = out.panel(p.cell.0, p.cell.1);
        for points in &p.view.lines {
            let points = points.clone();
            panel.marks.push(Mark::Line { x, y, points });
        }
        let side = if p.right { Orient::Right } else { Orient::Left };
        for a in [(Orient::Bottom, x), (side, y)] {
            if !panel.axes.contains(&a) {
                panel.axes.push(a);
            }
        }
    }
    out
}

/// How many distinct x and y scales the tree resolves to.
pub fn count(root: &Node) -> (usize, usize) {
    let mut b = Build::default();
    b.walk(root, (0, 0));
    let mut roots = [vec![], vec![]];
    for p in &b.placed {
        for ch in 0..2 {
            roots[ch].push(b.find(p.slot[ch]));
        }
    }
    let [x, y] = roots.map(|mut r| {
        r.sort();
        r.dedup();
        r.len()
    });
    (x, y)
}
