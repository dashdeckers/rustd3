//! Follow-up 6: which axis titles of the wheel figure
//! outgrow their panel, per context.
use dlist::measure::TypstMeasure;
use dlist::shape::Shaper;
use dlist::wheels;
use figmodel::plot::{Context, Measure, Style, Text, Theme};
use std::rc::Rc;

fn main() {
    let samples = wheels::simulate();
    let tm = TypstMeasure::new(Rc::new(Shaper::new()));
    for ctx in [Context::Paper, Context::Notebook, Context::Talk, Context::Poster] {
        let theme = Theme::seaborn(Style::Ticks, ctx);
        let mut fig = wheels::figure(&samples, false);
        fig.theme(theme.clone());
        let r = fig.resolve([960.0, 760.0]);
        let s = theme.label_text();
        let scatter = fig.rect(&r, 0, 0).unwrap();
        let hist = fig.rect(&r, 1, 0).unwrap();
        let rows = [
            ("vibration RMS [m/s²]", scatter.h),
            ("bearing temp [°C]", scatter.h),
            ("count", hist.h),
            ("speed [km/h]", hist.w),
        ];
        let out: Vec<String> = rows
            .iter()
            .map(|(t, len)| {
                let w = tm.width(&Text::from(*t), s, false);
                let mark = if w > *len { "OVER" } else { "ok" };
                format!("{t} {w:.0}/{len:.0} {mark}")
            })
            .collect();
        println!("{ctx:?} ({s:.1} px): {}", out.join(" | "));
    }
}
