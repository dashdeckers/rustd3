//! Paints as SVG attributes: colors, opacities and gradient elements.

use std::fmt::Write;

use peniko::color::Srgb;
use peniko::{Brush, Color, Extend, Gradient, GradientKind};

use crate::svg::number;

/// How a paint is written: an attribute value, and the gradient element it
/// refers to, if any.
pub(crate) struct Written {
    /// `fill='…'` or `stroke='…'` and its opacity, with a leading space.
    pub(crate) attrs: String,
    /// `<linearGradient …>…</linearGradient>` or `<radialGradient …>…`.
    pub(crate) def: Option<String>,
}

/// `paint` as the attribute `attr` (`fill` or `stroke`); a gradient takes
/// the id `p{id}`. Sweep gradients and image brushes have no SVG paint:
/// the error says which it is.
pub(crate) fn write(attr: &str, paint: &Brush, id: usize) -> Result<Written, &'static str> {
    match paint {
        Brush::Solid(color) => Ok(Written {
            attrs: color_attrs(attr, &format!("{attr}-opacity"), *color),
            def: None,
        }),
        Brush::Gradient(gradient) => Ok(Written {
            attrs: format!(" {attr}='url(#p{id})'"),
            def: Some(gradient_element(gradient, id)?),
        }),
        Brush::Image(_) => Err("an image brush"),
    }
}

/// ` {attr}='#rrggbb'`, and ` {opacity}='a'` when not opaque.
fn color_attrs(attr: &str, opacity: &str, color: Color) -> String {
    let [r, g, b, _] = color.to_rgba8().to_u8_array();
    let mut s = format!(" {attr}='#{r:02x}{g:02x}{b:02x}'");
    let alpha = color.components[3];
    if alpha < 1.0 {
        let _ = write!(s, " {opacity}='{}'", number::unit(f64::from(alpha)));
    }
    s
}

/// The gradient element, in user space, its stops in sRGB.
fn gradient_element(gradient: &Gradient, id: usize) -> Result<String, &'static str> {
    let spread = match gradient.extend {
        Extend::Pad => "",
        Extend::Repeat => " spreadMethod='repeat'",
        Extend::Reflect => " spreadMethod='reflect'",
    };
    let px = number::px;
    let (tag, geometry) = match gradient.kind {
        GradientKind::Linear(p) => (
            "linearGradient",
            format!(
                " x1='{}' y1='{}' x2='{}' y2='{}'",
                px(p.start.x),
                px(p.start.y),
                px(p.end.x),
                px(p.end.y)
            ),
        ),
        GradientKind::Radial(p) => {
            let mut g = format!(
                " cx='{}' cy='{}' r='{}'",
                px(p.end_center.x),
                px(p.end_center.y),
                px(f64::from(p.end_radius))
            );
            if p.start_center != p.end_center || p.start_radius != 0.0 {
                let _ = write!(
                    g,
                    " fx='{}' fy='{}' fr='{}'",
                    px(p.start_center.x),
                    px(p.start_center.y),
                    px(f64::from(p.start_radius))
                );
            }
            ("radialGradient", g)
        }
        GradientKind::Sweep(_) => return Err("a sweep gradient"),
    };
    let mut s = format!("<{tag} id='p{id}' gradientUnits='userSpaceOnUse'{geometry}{spread}>");
    for stop in gradient.stops.iter() {
        let color = stop.color.to_alpha_color::<Srgb>();
        let _ = write!(
            s,
            "<stop offset='{}'{}/>",
            number::unit(f64::from(stop.offset)),
            color_attrs("stop-color", "stop-opacity", color)
        );
    }
    let _ = write!(s, "</{tag}>");
    Ok(s)
}
