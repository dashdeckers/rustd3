//! Focus and context: zoom the detail view with the wheel,
//! pan it by dragging, brush a window in the overview below. The app is
//! in `app.rs`, which rustd3-iced's tests drive too.

#[allow(dead_code, reason = "the tests read what the example does not")]
mod app;

use crate::app::App;

fn main() -> iced::Result {
    // DirectX on Windows, where Intel's Vulkan driver shows frames it has
    // not finished drawing. Without a GPU, iced still falls back to its
    // software renderer.
    #[cfg(windows)]
    if std::env::var_os("WGPU_BACKEND").is_none() {
        // SAFETY: no other thread runs yet to read the environment.
        #[allow(unsafe_code, reason = "wgpu takes its API from the environment alone")]
        unsafe {
            std::env::set_var("WGPU_BACKEND", "dx12");
        }
    }
    iced::application(App::new, App::update, App::view)
        .title("rustd3: focus and context")
        .window_size((924.0, 430.0))
        .run()
}
