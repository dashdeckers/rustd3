//! rustd3-iced's integration tests, and its examples' core-level apps
//! (focus and context, the update pattern) driven through `iced_test`'s
//! simulator on `ICED_TEST_BACKEND`'s renderer, and the root README's
//! `Plot` code.

#[cfg(all(feature = "figure", feature = "fonts"))]
mod brush;
mod canvas;
#[cfg(all(feature = "figure", feature = "fonts"))]
mod classify;
mod focus_context;
#[cfg(all(feature = "figure", feature = "fonts"))]
mod gestures;
mod headless;
#[cfg(all(feature = "figure", feature = "fonts"))]
mod hostile;
#[cfg(all(feature = "figure", feature = "fonts"))]
mod hover;
mod path;
#[cfg(all(feature = "figure", feature = "fonts"))]
mod pipeline;
mod placement;
#[cfg(all(feature = "figure", feature = "fonts"))]
mod plot;
mod pointer;
mod readme;
#[cfg(all(feature = "figure", feature = "fonts"))]
mod reference;
mod threads;
#[cfg(all(feature = "figure", feature = "fonts"))]
mod tiles;
mod update_pattern;

/// `view` in a simulator of `size` logical px on `ICED_TEST_BACKEND`'s
/// renderer; `None`, with a message, when that is wgpu, it finds no device
/// and `RUSTD3_ALLOW_NO_GPU=1`.
fn simulator<'a, M: 'a>(
    size: (f32, f32),
    view: impl iced::Widget<M> + 'a,
) -> Option<iced_test::Simulator<'a, M>> {
    use iced::advanced::renderer::{self, Headless as _};

    let backend = std::env::var("ICED_TEST_BACKEND").ok();
    let allowed = std::env::var("RUSTD3_ALLOW_NO_GPU").is_ok_and(|v| v == "1");
    if backend.as_deref() == Some("wgpu") && allowed {
        let probe = iced::futures::executor::block_on(iced::Renderer::new(
            renderer::Settings::default(),
            Some("wgpu"),
        ));
        if probe.is_none() {
            eprintln!("skipped: wgpu found no device (RUSTD3_ALLOW_NO_GPU=1)");
            return None;
        }
    }
    Some(iced_test::Simulator::with_size(
        iced::Settings::default(),
        size,
        view,
    ))
}
