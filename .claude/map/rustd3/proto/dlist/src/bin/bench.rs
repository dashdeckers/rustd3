//! Problem #4 evidence: the wheel figure as a drawing
//! list, through every backend.

use dlist::lower::{self, ALL, ICED};
use dlist::outline::Outlines;
use dlist::shape::Shaper;
use dlist::{diff, pdf, skia, svg, vcpu, wheels};
use figmodel::plot::draw::{census, Command, List};
use std::time::Instant;

fn ms<T>(n: usize, mut f: impl FnMut() -> T) -> (f64, T) {
    let mut ts = Vec::new();
    let mut out = None;
    for _ in 0..n {
        let t = Instant::now();
        out = Some(f());
        ts.push(t.elapsed().as_secs_f64() * 1e3);
    }
    ts.sort_by(|a, b| a.partial_cmp(b).unwrap());
    (ts[n / 2], out.unwrap())
}

fn kb(n: usize) -> String {
    format!("{:.1} KB", n as f64 / 1024.0)
}

fn resvg_png(s: &str, scale: f32, fonts: bool) -> tiny_skia::Pixmap {
    let mut opt = resvg::usvg::Options::default();
    if fonts {
        let db = opt.fontdb_mut();
        db.load_font_file(
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        )
        .unwrap();
        db.load_font_file(
            "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
        )
        .unwrap();
    }
    let tree = resvg::usvg::Tree::from_str(s, &opt).unwrap();
    let sz = tree.size();
    let mut px = tiny_skia::Pixmap::new(
        (sz.width() * scale) as u32,
        (sz.height() * scale) as u32,
    )
    .unwrap();
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut px.as_mut(),
    );
    px
}

fn vpng(p: &vello_cpu::Pixmap) -> tiny_skia::Pixmap {
    let mut px = tiny_skia::Pixmap::new(
        p.width() as u32,
        p.height() as u32,
    )
    .unwrap();
    px.data_mut().copy_from_slice(p.data_as_u8_slice());
    px
}

fn show(name: &str, l: &List) {
    let c: Vec<String> = census(l)
        .iter()
        .map(|(k, n)| format!("{k} {n}"))
        .collect();
    println!("{name}: {} commands: {}", l.commands.len(), c.join(", "));
}

fn main() {
    std::fs::create_dir_all("out").unwrap();
    let samples = wheels::simulate();
    let fig = wheels::figure(&samples, false);
    let size = [960.0, 760.0];
    let s2 = 2.0;

    println!("== the list");
    let (t_draw, raw) = ms(21, || fig.draw(size));
    println!("figure → list (layout included): {t_draw:.2} ms");
    show("before shaping", &raw);
    let shaper = Shaper::new();
    let mut list = raw.clone();
    let t = Instant::now();
    shaper.shape(&mut list);
    let t_cold = t.elapsed().as_secs_f64() * 1e3;
    let (t_warm, _) = ms(5, || {
        let mut l = raw.clone();
        shaper.shape(&mut l);
    });
    println!(
        "shaping {} labels in one typst document: \
         {t_cold:.1} ms cold, {t_warm:.1} ms repeated",
        census(&raw).iter().find(|e| e.0 == "Label").unwrap().1
    );
    show("after shaping", &list);
    println!(
        "fonts in table: {} ({})",
        list.fonts.len(),
        list.fonts
            .iter()
            .map(|f| kb(f.data.data().len()))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let runs: Vec<_> = list
        .commands
        .iter()
        .filter_map(|c| match c {
            Command::Glyphs(r) => Some(r),
            _ => None,
        })
        .collect();
    let glyphs: usize = runs.iter().map(|r| r.glyphs.len()).sum();
    println!("glyph runs {}, glyphs {glyphs}", runs.len());
    let title = runs
        .iter()
        .filter(|r| r.text.contains("RL") || r.text.contains("Wheel"))
        .map(|r| format!("{:?}", r.text))
        .collect::<Vec<_>>();
    println!("title runs: {}", title.join(" "));

    let mut stats = lower::Stats::default();
    let mut o = Outlines::default();
    let expanded = lower::lower(
        &list,
        lower::Caps { markers: false, ..ALL },
        &mut o,
        &mut stats,
    );
    show("markers expanded", &expanded);

    println!("\n== SVG");
    let (t_txt, svg_txt) = ms(11, || list.svg());
    let mut o = Outlines::default();
    let (t_out, svg_out) = ms(11, || svg::outlined(&list, &mut o));
    let svg_exp = svg::outlined(&expanded, &mut o);
    println!("text as <text>: {} in {t_txt:.2} ms", kb(svg_txt.len()));
    println!(
        "text as outlines: {} in {t_out:.2} ms",
        kb(svg_out.len())
    );
    println!(
        "outlines, markers as one <path> each: {}",
        kb(svg_exp.len())
    );
    std::fs::write("out/text.svg", &svg_txt).unwrap();
    std::fs::write("out/outlined.svg", &svg_out).unwrap();

    println!("\n== PNG at 2×");
    let mut o = Outlines::default();
    let (t_skia, px_skia) = ms(11, || skia::render(&list, s2, &mut o));
    let (t_skia_cold, _) =
        ms(1, || skia::render(&list, s2, &mut Outlines::default()));
    let (t_skia_exp, _) =
        ms(11, || skia::render(&expanded, s2, &mut o));
    let (t_vc, px_vc) = ms(11, || vcpu::render(&list, s2));
    let px_vc = vpng(&px_vc);
    let (t_rs, px_rs) = ms(11, || resvg_png(&svg_out, 2.0, false));
    let (t_rs_txt, px_rs_txt) =
        ms(11, || resvg_png(&svg_txt, 2.0, true));
    let (t_rs_parse, _) = ms(11, || {
        resvg::usvg::Tree::from_str(
            &svg_out,
            &resvg::usvg::Options::default(),
        )
        .unwrap()
    });
    let (t_enc, png) = ms(5, || px_skia.encode_png().unwrap());
    println!("tiny-skia from list: {t_skia:.1} ms (cold glyph cache {t_skia_cold:.1} ms; markers expanded {t_skia_exp:.1} ms)");
    println!("vello_cpu from list: {t_vc:.1} ms");
    println!(
        "resvg on outlined SVG: {t_rs:.1} ms \
         (of which parse {t_rs_parse:.1} ms)"
    );
    println!(
        "resvg on <text> SVG (re-shapes, DejaVu loaded): \
         {t_rs_txt:.1} ms"
    );
    println!("PNG encode: {t_enc:.1} ms, {}", kb(png.len()));
    px_skia.save_png("out/skia.png").unwrap();
    px_vc.save_png("out/vcpu.png").unwrap();
    px_rs.save_png("out/resvg.png").unwrap();
    px_rs_txt.save_png("out/resvg_text.png").unwrap();
    for (n, a, b) in [
        ("tiny-skia vs resvg(outlines)", &px_skia, &px_rs),
        ("tiny-skia vs vello_cpu", &px_skia, &px_vc),
        ("resvg(outlines) vs resvg(<text>)", &px_rs, &px_rs_txt),
    ] {
        let d = diff(a.data(), b.data());
        println!(
            "diff {n}: max {} mean {:.3} >16: {:.3}%",
            d.max,
            d.mean,
            100.0 * d.over16
        );
    }

    println!("\n== PDF");
    let (t_pdf, pdf_txt) = ms(5, || pdf::write(&list, false));
    let (t_pdf_o, pdf_out) = ms(5, || pdf::write(&list, true));
    std::fs::write("out/krilla_text.pdf", &pdf_txt).unwrap();
    std::fs::write("out/krilla_outlined.pdf", &pdf_out).unwrap();
    println!("krilla, real text: {} in {t_pdf:.1} ms", kb(pdf_txt.len()));
    println!(
        "krilla, outlined glyphs: {} in {t_pdf_o:.1} ms",
        kb(pdf_out.len())
    );
    let (t_s2p, s2p) = ms(5, || {
        let tree = svg2pdf::usvg::Tree::from_str(
            &svg_out,
            &svg2pdf::usvg::Options::default(),
        )
        .unwrap();
        svg2pdf::to_pdf(
            &tree,
            svg2pdf::ConversionOptions::default(),
            svg2pdf::PageOptions::default(),
        )
        .unwrap()
    });
    std::fs::write("out/svg2pdf.pdf", &s2p).unwrap();
    println!(
        "svg2pdf on outlined SVG: {} in {t_s2p:.1} ms",
        kb(s2p.len())
    );

    println!("\n== lowering for iced");
    let mut st = lower::Stats::default();
    let mut o = Outlines::default();
    let (t_low, iced) =
        ms(11, || lower::lower(&list, ICED, &mut o, &mut lower::Stats::default()));
    lower::lower(&list, ICED, &mut Outlines::default(), &mut st);
    show("lowered for iced", &iced);
    println!("lowering {t_low:.2} ms; {st:?}");
    let px_iced = skia::render(&iced, s2, &mut Outlines::default());
    let d = diff(px_skia.data(), px_iced.data());
    println!(
        "diff original vs lowered (tiny-skia): max {} mean {:.4} >16: {:.4}%",
        d.max,
        d.mean,
        100.0 * d.over16
    );
}
