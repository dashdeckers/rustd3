//! Which of rustd3-iced's types cross threads: compile-time assertions.
//! An app may keep a tracker or a probe in state that iced moves between
//! threads; `Plot` and `IcedPath` live inside one `view` or `draw` call.

use rustd3_iced::pointer::Tracker;

const fn send_sync<T: Send + Sync>() {}

const _: () = send_sync::<Tracker>();

#[cfg(feature = "figure")]
const _: () = {
    use rustd3_iced::plot::{Caching, Hover, Probe, Render};
    send_sync::<Probe>();
    send_sync::<Hover>();
    send_sync::<Render>();
    send_sync::<Caching>();
};
