//! The general update pattern: a top-five bar chart that
//! re-ranks every 1.5 s, bars entering, moving and leaving in staggered
//! transitions. The app is in `app.rs`, which rustd3-iced's tests drive
//! too.

#[allow(dead_code, reason = "the tests read what the example does not")]
mod app;

use std::thread;
use std::time::Duration;

use iced::Subscription;
use iced::futures::channel::mpsc;
use rustd3_core::anim::Instant;

use crate::app::{App, Message};

#[expect(
    clippy::disallowed_methods,
    reason = "the app reads the clock and passes the instants to the library"
)]
fn main() -> iced::Result {
    iced::application(|| App::new(Instant::now()), App::update, App::view)
        .subscription(|app: &App| {
            let data = every(Duration::from_millis(1500)).map(Message::NewData);
            Subscription::batch([data, app.subscription()])
        })
        .title("rustd3: the general update pattern")
        .window_size((600.0, 260.0))
        .run()
}

/// The time every `period`, from a thread of its own: iced's `time::every`
/// needs its `tokio` or `smol` feature, which the workspace leaves off.
#[expect(
    clippy::disallowed_methods,
    reason = "the app reads the clock and passes the instants to the library"
)]
fn every(period: Duration) -> Subscription<Instant> {
    Subscription::run_with(period, |&period| {
        let (mut sender, receiver) = mpsc::channel(1);
        thread::spawn(move || {
            while !sender.is_closed() {
                thread::sleep(period);
                // A tick the app has not taken yet stands for this one.
                let _ = sender.try_send(Instant::now());
            }
        });
        receiver
    })
}
