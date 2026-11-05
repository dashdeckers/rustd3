//! Focus and context: zoom the detail view with the wheel,
//! pan it by dragging, brush a window in the overview below. The app is
//! in `app.rs`, which rustd3-iced's tests drive too.

#[allow(dead_code, reason = "the tests read what the example does not")]
mod app;

use crate::app::App;

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("rustd3: focus and context")
        .window_size((924.0, 430.0))
        .run()
}
