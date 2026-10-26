//! Follow-up 5: layout with typst-measured widths
//! instead of 0.6 em per character.

use dlist::measure::TypstMeasure;
use dlist::shape::Shaper;
use dlist::{vcpu, wheels};
use figmodel::plot::{Estimate, Measure, Text, Theme, label_trace};
use std::rc::Rc;
use std::time::Instant;

fn main() {
    let samples = wheels::simulate();
    let size = [960.0, 760.0];
    let theme = Theme::default();
    let s = theme.tick_text();
    let shaper = Rc::new(Shaper::new());

    // Every tick label the layout sizes, plus the
    // legend entries and the vertical axis titles.
    let mut fig = wheels::figure(&samples, false);
    let r = fig.resolve(size);
    let mut texts: Vec<String> = label_trace(&fig, &r)
        .into_iter()
        .map(|(_, _, t, _, _)| t)
        .collect();
    texts.sort();
    texts.dedup();
    let tm = TypstMeasure::new(shaper.clone());
    let mut rel: Vec<(f64, String, f64, f64)> = texts
        .iter()
        .map(|t| {
            let e = Estimate.width(&Text::from(t.as_str()), s, false);
            let m = tm.width(&Text::from(t.as_str()), s, false);
            ((e - m) / m, t.clone(), e, m)
        })
        .collect();
    rel.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let mean = rel.iter().map(|r| r.0.abs()).sum::<f64>() / rel.len() as f64;
    println!("{} distinct tick labels at {s:.1} px: mean |error| {:.0}%", rel.len(), 100.0 * mean);
    for (e, t, est, m) in rel.iter().take(3).chain(rel.iter().rev().take(3)) {
        println!("  {t:>8}: estimate {est:5.1} px, typst {m:5.1} px ({:+.0}%)", 100.0 * e);
    }
    for t in ["accel x", "bearing temp", "vibration RMS [m/s²]", "speed [km/h]", "1111", "WWW", "iii", "−0.001", "2026-03-07", "Wednesday"] {
        let e = Estimate.width(&Text::from(t), s, false);
        let m = tm.width(&Text::from(t), s, false);
        println!("  {t:>22}: estimate {e:5.1}, typst {m:5.1} ({:+.0}%)", 100.0 * (e - m) / m);
    }

    // Layout with each measurer.
    let est = fig.resolve(size);
    let tm = Rc::new(TypstMeasure::new(shaper.clone()));
    fig.measure(tm.clone());
    let t = Instant::now();
    let typ = fig.resolve(size);
    let cold = t.elapsed().as_secs_f64() * 1e3;
    let (m1, h1) = (tm.misses.get(), tm.hits.get());
    let t = Instant::now();
    let _ = fig.resolve(size);
    let warm = t.elapsed().as_secs_f64() * 1e3;
    let area = |r: &figmodel::plot::Resolved| {
        r.rects.iter().map(|q| q.w * q.h).sum::<f64>() / (size[0] * size[1])
    };
    let dx = est
        .rects
        .iter()
        .zip(&typ.rects)
        .map(|(a, b)| (a.x - b.x).abs().max((a.w - b.w).abs()))
        .fold(0.0, f64::max);
    let dropped = |r: &figmodel::plot::Resolved| {
        r.keep.iter().flatten().flatten().filter(|k| !**k).count()
    };
    println!(
        "plot area {:.1}% → {:.1}%; largest change of a panel's x or width {dx:.1} px; dropped labels {} → {}",
        100.0 * area(&est),
        100.0 * area(&typ),
        dropped(&est),
        dropped(&typ)
    );
    println!(
        "typst layout: cold {cold:.1} ms ({m1} misses, {h1} hits), warm {warm:.2} ms; estimate layout {:.2} ms",
        {
            let mut f2 = wheels::figure(&samples, false);
            f2.theme(theme.clone());
            let t = Instant::now();
            let _ = f2.resolve(size);
            t.elapsed().as_secs_f64() * 1e3
        }
    );
    // Batched: one document for every label first.
    let tm2 = Rc::new(TypstMeasure::new(shaper.clone()));
    let batch: Vec<(Text, f64, bool)> = texts
        .iter()
        .map(|t| (Text::from(t.as_str()), s, false))
        .collect();
    let t = Instant::now();
    tm2.prefetch(&batch);
    let pre = t.elapsed().as_secs_f64() * 1e3;
    fig.measure(tm2.clone());
    let t = Instant::now();
    let _ = fig.resolve(size);
    println!(
        "batched: prefetch {} labels {pre:.1} ms, then layout {:.1} ms ({} misses left)",
        batch.len(),
        t.elapsed().as_secs_f64() * 1e3,
        tm2.misses.get()
    );

    // Renders for the comparison.
    std::fs::create_dir_all("out/measure").unwrap();
    for (name, m) in [("estimate", None), ("typst", Some(tm.clone()))] {
        let mut f = wheels::figure(&samples, false);
        if let Some(m) = m {
            f.measure(m);
        }
        let mut l = f.draw(size);
        shaper.shape(&mut l);
        let px = vcpu::render(&l, 1.0);
        let mut out = tiny_skia::Pixmap::new(px.width() as u32, px.height() as u32).unwrap();
        out.data_mut().copy_from_slice(px.data_as_u8_slice());
        out.save_png(format!("out/measure/{name}.png")).unwrap();
    }
}
