//! rustd3-examples' tests: the core-level use cases driven headlessly
//! through `iced_test`'s simulator, on `ICED_TEST_BACKEND`'s renderer.

mod focus_context;
mod update_pattern;

use iced::Settings;
use iced::advanced::renderer::{self, Headless as _};
use iced_test::Simulator;

/// `view` in a simulator of `size` logical px on `ICED_TEST_BACKEND`'s
/// renderer; `None`, with a message, when that is wgpu, it finds no device
/// and `RUSTD3_ALLOW_NO_GPU=1`.
fn simulator<'a, M: 'a>(
    size: (f32, f32),
    view: impl iced::Widget<M> + 'a,
) -> Option<Simulator<'a, M>> {
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
    Some(Simulator::with_size(Settings::default(), size, view))
}
