//! Raster outputs: the snapping helpers here, vello_cpu's renders in
//! `vello`, marker splatting in `splat`.

// Splatting is measured on drawn figures, which need fonts.
#[cfg(all(feature = "raster", feature = "fonts"))]
mod splat;
#[cfg(feature = "raster")]
pub mod vello;

use rustd3_figure::raster::snap;

#[test]
fn edges_snap_to_the_device_pixel_grid() {
    assert_eq!(snap::edge(10.3, 1.5), 10.0);
    assert!((snap::edge(10.5, 1.25) - 10.4).abs() < 1e-12);
    assert_eq!(snap::edge(10.3, 1.0), 10.0);
}

#[test]
fn hairlines_are_at_least_one_device_px_wide() {
    assert_eq!(snap::width(0.3, 1.0), 1.0);
    assert_eq!(snap::width(0.3, 2.0), 0.5);
    assert_eq!(snap::width(1.0, 1.5), 2.0 / 1.5);
}

#[test]
fn line_centres_cover_whole_device_pixels() {
    assert_eq!(snap::line(10.3, 1.0, 1.0), 10.5);
    assert_eq!(snap::line(10.3, 1.0, 2.0), 10.5);
    assert_eq!(snap::line(10.3, 1.0, 1.25), 10.0);
}
