//! Which of rustd3-figure's types cross threads: compile-time assertions.
//! The `Plot` widget's pool renders drawing lists, each thread with its
//! own renderer; a `Figure`, its `Held` scales and its `Resolved` layout
//! stay on the thread that built them (marks and scales are boxed trait
//! objects without a `Send` bound).

use rustd3_figure::cache::Cache;
use rustd3_figure::{Target, Theme, hit, list, text};

const fn send_sync<T: Send + Sync>() {}

#[cfg(feature = "raster")]
const fn send<T: Send>() {}

const _: () = {
    send_sync::<list::List>();
    send_sync::<list::Bounds>();
    send_sync::<list::Outlines>();
    send_sync::<text::Fonts>();
    send_sync::<text::Cache>();
    send_sync::<text::Text>();
    send_sync::<Cache>();
    send_sync::<Theme>();
    send_sync::<Target>();
    send_sync::<hit::Index>();
};

#[cfg(feature = "raster")]
const _: () = {
    send::<rustd3_figure::raster::Renderer>();
    send_sync::<rustd3_figure::raster::Image>();
    send_sync::<rustd3_figure::raster::Settings>();
};
