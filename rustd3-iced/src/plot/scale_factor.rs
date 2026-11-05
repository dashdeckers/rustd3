//! [`resolve`]: the scale-factor chain. [`fit`]: the lower factor a widget
//! renders at when it is too large for the renderer at its own.

use iced_core::{Size, renderer};

/// The largest factor the widget renders at: twice the largest a display
/// reports. Its tiles and strips cover the plot areas at this many device
/// px per logical px, so a factor of 1e9 would ask for more tiles than any
/// machine holds.
pub(crate) const MOST: f32 = 8.0;

/// Resolves the factor from, in order, explicit input, the renderer's
/// hint, the window's factor, then 1.0. Non-finite or non-positive values
/// are skipped, and one above [`MOST`] is taken as `MOST`.
///
/// tiny-skia's renderer reports no scale. wgpu's reports the window's and
/// the application's factors, whose product it draws at
/// (`core/src/renderer.rs:75,214`). Window events carry the window's
/// factor alone (`winit/src/lib.rs:671-675`).
pub(crate) fn resolve(
    explicit: Option<f32>,
    renderer: Option<renderer::Scale>,
    window: Option<f32>,
) -> f32 {
    [explicit, renderer.map(renderer::Scale::total), window]
        .into_iter()
        .flatten()
        .find(|k| k.is_finite() && *k > 0.0)
        .unwrap_or(1.0)
        .min(MOST)
}

/// The most device px a side that the figure's raster renderer draws.
/// Beyond it a render fails with `raster::Error::TooLarge`.
const SIDE: f32 = 65_408.0;

/// `factor`, lowered where the widget's logical `size` would be wider or
/// taller than [`SIDE`] device px at it, so that every strip and tile of
/// the widget renders.
pub(crate) fn fit(factor: f32, size: Size) -> f32 {
    let longest = size.width.max(size.height);
    if longest * factor > SIDE {
        SIDE / longest
    } else {
        factor
    }
}

#[cfg(test)]
mod tests {
    use iced_core::Size;
    use iced_core::renderer::Scale;

    use super::{fit, resolve};

    #[test]
    fn a_widget_wider_than_the_renderer_draws_lowers_its_factor() {
        assert_eq!(fit(2.0, Size::new(640.0, 480.0)), 2.0);
        assert_eq!(fit(2.0, Size::new(1e6, 2.0)), 0.065408);
        assert_eq!(fit(2.0, Size::ZERO), 2.0);
    }

    #[test]
    fn an_explicit_factor_wins() {
        let wgpu = Scale {
            window: 2.0,
            application: 1.0,
        };
        assert_eq!(resolve(Some(1.5), Some(wgpu), Some(2.0)), 1.5);
    }

    #[test]
    fn the_renderer_multiplies_window_and_application_factors() {
        let wgpu = Scale {
            window: 2.0,
            application: 0.75,
        };
        assert_eq!(resolve(None, Some(wgpu), Some(2.0)), 1.5);
    }

    #[test]
    fn without_a_renderer_hint_the_window_factor_applies() {
        assert_eq!(resolve(None, None, Some(1.25)), 1.25);
    }

    #[test]
    fn with_nothing_known_the_factor_is_one() {
        assert_eq!(resolve(None, None, None), 1.0);
    }

    #[test]
    fn a_factor_above_the_most_is_taken_as_the_most() {
        assert_eq!(resolve(Some(1e9), None, None), super::MOST);
        let huge = Scale {
            window: 1e6,
            application: 1.0,
        };
        assert_eq!(resolve(None, Some(huge), None), super::MOST);
    }

    #[test]
    fn non_finite_and_non_positive_factors_are_skipped() {
        assert_eq!(resolve(Some(f32::NAN), None, Some(2.0)), 2.0);
        assert_eq!(resolve(Some(f32::INFINITY), None, Some(2.0)), 2.0);
        assert_eq!(resolve(Some(0.0), None, None), 1.0);
        let broken = Scale {
            window: -1.0,
            application: 1.0,
        };
        assert_eq!(resolve(Some(-2.0), Some(broken), Some(1.25)), 1.25);
    }
}
