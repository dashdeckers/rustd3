//! rustd3-examples' tests: the core-level apps driven headlessly
//! through `iced_test`'s simulator, on `ICED_TEST_BACKEND`'s renderer (with
//! the `widget` feature), the wheel figure, and the root README's iced
//! code.

#[cfg(feature = "widget")]
mod focus_context;
// poppler's tools, shared with rustd3-figure's PDF tests.
#[path = "../../../../figure/tests/main/poppler.rs"]
mod poppler;
mod readme;
#[cfg(feature = "widget")]
mod update_pattern;
mod wheels;

/// `view` in a simulator of `size` logical px on `ICED_TEST_BACKEND`'s
/// renderer; `None`, with a message, when that is wgpu, it finds no device
/// and `RUSTD3_ALLOW_NO_GPU=1`.
#[cfg(feature = "widget")]
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
