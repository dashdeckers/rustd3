//! [`resolve`]: the scale-factor chain.

use iced_core::renderer;

/// Explicit input, the renderer's hint, the window's factor, then 1.0.
/// Non-finite or non-positive values are skipped.
///
/// tiny-skia's renderer reports no scale; wgpu's reports the window's and
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
}

#[cfg(test)]
mod tests {
    use iced_core::renderer::Scale;

    use super::resolve;

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
