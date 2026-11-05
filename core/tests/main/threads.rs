//! Which of rustd3-core's types cross threads: compile-time assertions.
//! Scales, formats, locales, recordings and generators are `Send` and
//! `Sync`. An `axis::Axis` holds a plain tick-format closure and is not.

use rustd3_core::{anim, color, format, interact, locale, path, scale, series, shape, ticks, time};

const fn send_sync<T: Send + Sync>() {}

const _: () = {
    send_sync::<scale::Linear>();
    send_sync::<scale::Log>();
    send_sync::<scale::AnyContinuous>();
    send_sync::<scale::Continuous<scale::transform::Identity, color::Rgb>>();
    send_sync::<scale::Band<String>>();
    send_sync::<scale::Point<String>>();
    send_sync::<scale::Ordinal<String, color::Rgb>>();
    send_sync::<scale::Sequential<color::Rgb>>();
    send_sync::<scale::Diverging<color::Rgb>>();
    send_sync::<scale::Quantize<color::Rgb>>();
    send_sync::<scale::Quantile<color::Rgb>>();
    send_sync::<scale::Threshold<color::Rgb>>();
    send_sync::<scale::Identity>();
    send_sync::<scale::TickRule>();
    send_sync::<scale::TickFormat>();
    send_sync::<time::Time>();
    send_sync::<time::Format>();
    send_sync::<time::Locale>();
    send_sync::<time::Interval>();
    send_sync::<format::Format>();
    send_sync::<format::Locale>();
    send_sync::<locale::Locale>();
    send_sync::<ticks::Set<f64, f64>>();
    send_sync::<series::Recording<f64>>();
    send_sync::<series::Recording<rustd3_core::jiff::Timestamp>>();
    send_sync::<series::Columns>();
    send_sync::<shape::Line>();
    send_sync::<shape::Area>();
    send_sync::<shape::Arc>();
    send_sync::<shape::Pie>();
    send_sync::<shape::Stack>();
    send_sync::<path::SvgPath>();
    send_sync::<interact::Zoom>();
    send_sync::<interact::BrushX>();
    send_sync::<interact::Drag>();
    send_sync::<anim::Animated<f64>>();
};
