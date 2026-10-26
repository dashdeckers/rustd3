//! Layout's text widths from typst, cached per
//! (markup, size, weight). Misses compile one label
//! each; `prefetch` batches a figure's labels first.

use crate::shape::Shaper;
use figmodel::plot::{Measure, Text};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

pub struct TypstMeasure {
    pub shaper: Rc<Shaper>,
    cache: RefCell<HashMap<(String, u32, bool), f64>>,
    pub misses: Cell<usize>,
    pub hits: Cell<usize>,
}

impl TypstMeasure {
    pub fn new(shaper: Rc<Shaper>) -> Self {
        TypstMeasure {
            shaper,
            cache: RefCell::default(),
            misses: Cell::new(0),
            hits: Cell::new(0),
        }
    }

    fn key(t: &Text, size: f64, bold: bool) -> (String, u32, bool) {
        (t.typst(), (size * 100.0).round() as u32, bold)
    }

    /// Measures all of `labels` in one document.
    pub fn prefetch(&self, labels: &[(Text, f64, bool)]) {
        let todo: Vec<_> = labels
            .iter()
            .map(|(t, s, b)| Self::key(t, *s, *b))
            .filter(|k| !self.cache.borrow().contains_key(k))
            .collect();
        let args: Vec<(String, f64, bool)> = todo
            .iter()
            .map(|(m, s, b)| (m.clone(), *s as f64 / 100.0, *b))
            .collect();
        let ws = self.shaper.widths(&args);
        let mut c = self.cache.borrow_mut();
        for (k, w) in todo.into_iter().zip(ws) {
            c.insert(k, w);
        }
    }
}

impl Measure for TypstMeasure {
    fn width(&self, t: &Text, size: f64, bold: bool) -> f64 {
        let k = Self::key(t, size, bold);
        if let Some(w) = self.cache.borrow().get(&k) {
            self.hits.set(self.hits.get() + 1);
            return *w;
        }
        self.misses.set(self.misses.get() + 1);
        let w = self.shaper.widths(&[(k.0.clone(), size, bold)])[0];
        self.cache.borrow_mut().insert(k, w);
        w
    }
}
