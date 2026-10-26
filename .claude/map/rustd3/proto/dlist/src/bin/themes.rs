//! Problem #6: the wheel figure under seaborn's styles
//! and contexts, and two palettes.

use d3::color::Rgb;
use dlist::shape::Shaper;
use dlist::{vcpu, wheels};
use figmodel::plot::{Context, Domains, Style, Theme};

fn main() {
    std::fs::create_dir_all("out/themes").unwrap();
    let samples = wheels::simulate();
    let shaper = Shaper::new();
    let size = [960.0, 760.0];
    let deep = wheels::PALETTE;
    let colorblind = [
        Rgb::hex(0x0173b2),
        Rgb::hex(0xde8f05),
        Rgb::hex(0x029e73),
    ];
    let cases: Vec<(&str, Theme, [Rgb; 3])> = vec![
        ("ticks_notebook", Theme::seaborn(Style::Ticks, Context::Notebook), deep),
        ("darkgrid_notebook", Theme::seaborn(Style::DarkGrid, Context::Notebook), deep),
        ("whitegrid_notebook", Theme::seaborn(Style::WhiteGrid, Context::Notebook), deep),
        ("ticks_paper", Theme::seaborn(Style::Ticks, Context::Paper), deep),
        ("ticks_talk", Theme::seaborn(Style::Ticks, Context::Talk), deep),
        ("ticks_notebook_nice", Theme::seaborn(Style::Ticks, Context::Notebook).domains(Domains::Nice), deep),
        ("ticks_notebook_colorblind", Theme::seaborn(Style::Ticks, Context::Notebook), colorblind),
    ];
    println!("case, plot-area share, layout passes, tick labels dropped, tick label px");
    for (name, theme, pal) in cases {
        let mut fig = wheels::figure_with(&samples, false, pal);
        fig.theme(theme.clone());
        let r = fig.resolve(size);
        let area: f64 = r.rects.iter().map(|q| q.w * q.h).sum();
        let dropped: usize = r
            .keep
            .iter()
            .flatten()
            .flatten()
            .filter(|k| !**k)
            .count();
        let mut l = fig.draw(size);
        shaper.shape(&mut l);
        let px = vcpu::render(&l, 1.0);
        let mut out = tiny_skia::Pixmap::new(px.width() as u32, px.height() as u32).unwrap();
        out.data_mut().copy_from_slice(px.data_as_u8_slice());
        out.save_png(format!("out/themes/{name}.png")).unwrap();
        println!(
            "{name}, {:.1}%, {}, {dropped}, {:.1}",
            100.0 * area / (size[0] * size[1]),
            r.passes,
            theme.tick_text()
        );
    }
}
