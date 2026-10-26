//! iced integration at two levels: at the core level, `IcedPath` draws
//! rustd3's shapes into an iced canvas and `pointer::Tracker` turns iced
//! events into pointer input for zoom, brush and drag; at the figure
//! level, behind the `figure` feature, the `Plot` widget shows rendered
//! images of a figure with interaction feedback drawn on top.
