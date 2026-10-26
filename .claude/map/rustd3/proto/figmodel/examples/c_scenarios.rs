//! Option C: axes own their scales; links come after.

use d3::time::time;
use figmodel::links::{Dir, Figure};
use jiff::tz::TimeZone;
use std::collections::HashMap;

fn line(k: f64) -> Vec<(f64, f64)> {
    (0..50)
        .map(|i| i as f64 * k)
        .map(|x| (x, (x / 9.0 / k).sin() * k))
        .collect()
}

fn main() {
    // x shared per column, y per row.
    let (mut f, ax) = Figure::subplots(2, 2);
    for (r, c) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
        let k = [1.0, 10.0][c] * [1.0, 2.0][r];
        f.plot(ax[r][c], line(k));
    }
    for c in 0..2 {
        f.share(Dir::X, &[ax[0][c], ax[1][c]]);
    }
    for r in 0..2 {
        f.share(Dir::Y, &[ax[r][0], ax[r][1]]);
    }
    let fig = f.build().unwrap();
    let r = fig.resolve([720.0, 480.0], &HashMap::new());
    let d: Vec<_> = r.scales.iter().map(|s| s.domain()).collect();
    println!("grid: {} scales {d:?}", d.len());

    // Twin axes: a second axes in the same cell.
    let (mut f, ax) = Figure::subplots(1, 1);
    let twin = f.twinx(ax[0][0]);
    f.plot(ax[0][0], line(1.0));
    f.plot(twin, line(30.0));
    let n = f.build().unwrap().resolve([1.0; 2], &HashMap::new());
    println!("twinx: {} scales", n.scales.len());

    // Conflicting limits on linked axes.
    let (mut f, ax) = Figure::subplots(1, 2);
    f.share(Dir::X, &[ax[0][0], ax[0][1]]);
    f.set_lim(ax[0][0], Dir::X, [0.0, 10.0]);
    f.set_lim(ax[0][1], Dir::X, [0.0, 20.0]);
    println!("conflict: {:?}", f.build().err());

    // A time axis linked to a linear one.
    let (mut f, ax) = Figure::subplots(1, 2);
    f.share(Dir::X, &[ax[0][0], ax[0][1]]);
    f.set_scale(ax[0][0], Dir::X, time(TimeZone::UTC));
    println!("kinds: {:?}", f.build().err());
}
