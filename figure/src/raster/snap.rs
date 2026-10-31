//! Device-pixel snapping for raster targets: plot-area edges on the pixel
//! grid and hairlines that cover whole pixels. Vector targets never call
//! these.

/// A plot-area edge on the device-pixel grid: `round(x·s) / s`.
///
/// ```
/// use rustd3_figure::raster::snap;
///
/// assert_eq!(snap::edge(10.3, 1.5), 10.0);
/// ```
#[must_use]
pub fn edge(x: f64, scale: f64) -> f64 {
    (x * scale).round() / scale
}

/// The width of a hairline in list px: at least one device px,
/// `max(1, round(w·s)) / s`.
#[must_use]
pub fn width(w: f64, scale: f64) -> f64 {
    device_width(w, scale) / scale
}

/// A line centre such that a line `w` wide covers whole device pixels:
/// `(floor(x·s) + 0.5) / s` when the snapped device width is odd,
/// `round(x·s) / s` when it is even.
///
/// ```
/// use rustd3_figure::raster::snap;
///
/// assert_eq!(snap::line(10.3, 1.0, 2.0), 10.5);
/// ```
#[must_use]
pub fn line(x: f64, w: f64, scale: f64) -> f64 {
    let device = x * scale;
    if device_width(w, scale) % 2.0 == 1.0 {
        (device.floor() + 0.5) / scale
    } else {
        device.round() / scale
    }
}

/// `max(1, round(w·s))`: a hairline's width in device px.
fn device_width(w: f64, scale: f64) -> f64 {
    (w * scale).round().max(1.0)
}
