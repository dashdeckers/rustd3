//! Use case 1, focus and context: zoom the detail view with the wheel,
//! pan it by dragging, brush a window in the overview below.

use rustd3_examples::focus_context::App;

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("rustd3: focus and context")
        .window_size((924.0, 430.0))
        .run()
}
