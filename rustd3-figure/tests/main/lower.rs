//! Lowering: command counts, warnings, parts, and pixels against the
//! input.

use rustd3_figure::kurbo::{Affine, Point, Rect, Shape, Size};
use rustd3_figure::list::{self, Clip, Command, List};
use rustd3_figure::lower::{self, Caps, Warning};

use crate::sink;

fn markers(points: usize) -> Command {
    Command::Markers {
        shape: Rect::new(-0.5, -0.5, 0.5, 0.5).to_path(0.1),
        at: (0..points).map(|k| Point::new(k as f64, 5.0)).collect(),
        paint: sink::rgb(0x55a868),
        sizes: Some(vec![2.0; points]),
        paints: None,
    }
}

fn fill() -> Command {
    Command::Fill {
        path: Rect::new(0.0, 0.0, 4.0, 4.0).to_path(0.1),
        paint: sink::rgb(0x4c72b0),
    }
}

#[test]
fn markers_become_one_fill_per_point() {
    let mut commands = vec![fill(); 7];
    commands.insert(1, markers(10));
    commands.insert(4, markers(20));
    commands.insert(9, markers(30));
    let list = List {
        size: Size::new(40.0, 10.0),
        commands,
        ..List::default()
    };
    assert_eq!(list.commands.len(), 10);
    let caps = Caps {
        markers: false,
        ..Caps::ALL
    };
    let lowered = lower::lower(&list, caps).unwrap();
    assert_eq!(lowered.list.commands.len(), 10 - 3 + 60);
    assert!(lowered.warnings.is_empty());
    assert_eq!(
        lowered.list.commands[1],
        Command::Fill {
            path: (Affine::translate((0.0, 5.0)) * Affine::scale(2.0))
                * Rect::new(-0.5, -0.5, 0.5, 0.5).to_path(0.1),
            paint: sink::rgb(0x55a868),
        }
    );
}

#[test]
fn everything_a_backend_draws_is_left_as_it_is() {
    let list = sink::with_sweep();
    let lowered = lower::lower(&list, Caps::ALL).unwrap();
    assert_eq!(lowered.list, list);
    assert!(lowered.warnings.is_empty());
}

#[test]
fn glyph_runs_become_one_fill_each() {
    let list = sink::list();
    let caps = Caps {
        glyphs: false,
        ..Caps::ALL
    };
    let lowered = lower::lower(&list, caps).unwrap();
    assert_eq!(lowered.list.commands.len(), list.commands.len());
    let fills = lowered.list.commands[15..]
        .iter()
        .filter(|c| matches!(c, Command::Fill { .. }))
        .count();
    assert_eq!(fills, 2);
    assert!(
        !lowered
            .list
            .commands
            .iter()
            .any(|c| matches!(c, Command::Glyphs(_)))
    );
}

#[test]
fn lower_leaves_path_clips_in_place_without_clip_paths() {
    let list = List {
        size: Size::new(10.0, 10.0),
        commands: vec![
            fill(),
            Command::Push {
                transform: Affine::IDENTITY,
                clip: Some(Clip::Path(Rect::new(1.0, 1.0, 5.0, 5.0).to_path(0.1))),
            },
            fill(),
            Command::Pop,
            Command::Push {
                transform: Affine::IDENTITY,
                clip: Some(Clip::Rect(Rect::new(1.0, 1.0, 5.0, 5.0))),
            },
            Command::Pop,
        ],
        ..List::default()
    };
    let caps = Caps {
        clip_paths: false,
        ..Caps::ALL
    };
    let lowered = lower::lower(&list, caps).unwrap();
    assert_eq!(lowered.list.commands[1], list.commands[1]);
    assert_eq!(lowered.list, list);
    assert_eq!(lowered.warnings, [Warning::UnsupportedClip { command: 1 }]);
}

#[test]
fn parts_follow_their_commands() {
    let list = sink::list();
    let caps = Caps {
        markers: false,
        ..Caps::ALL
    };
    let lowered = lower::lower(&list, caps).unwrap();
    let ranges: Vec<_> = lowered
        .list
        .parts
        .iter()
        .map(|p| p.commands.clone())
        .collect();
    // 3 + 4 markers become 7 fills: the plot part grows by 5.
    assert_eq!(ranges, [0..1, 1..20, 20..21, 21..22]);
    assert_eq!(list::check(&lowered.list), Ok(()));
}

#[test]
fn a_list_that_fails_check_is_not_lowered() {
    let list = List {
        commands: vec![Command::Pop],
        ..List::default()
    };
    assert_eq!(
        lower::lower(&list, Caps::ALL),
        Err(list::Error::UnmatchedPop { command: 0 })
    );
}

#[cfg(feature = "raster")]
mod raster {
    use rustd3_figure::list::{Clip, Command};
    use rustd3_figure::lower::{self, Caps, Warning};
    use rustd3_figure::svg;

    use crate::raster::vello::{diff, whole};
    use crate::sink;

    /// The sweep disc of `sink::with_sweep`.
    const SWEEP: usize = 14;

    #[test]
    fn a_sweep_becomes_an_image_clipped_to_its_path() {
        let list = sink::with_sweep();
        let lowered = lower::lower(&list, svg::CAPS).unwrap();
        assert_eq!(
            lowered.warnings,
            [Warning::SweepRasterized { command: SWEEP }]
        );
        let commands = &lowered.list.commands[SWEEP..SWEEP + 3];
        assert!(matches!(
            commands[0],
            Command::Push {
                clip: Some(Clip::Path(_)),
                ..
            }
        ));
        let Command::Image {
            rect,
            image,
            smooth,
        } = &commands[1]
        else {
            panic!("an image: {:?}", commands[1]);
        };
        assert!(*smooth);
        assert_eq!(
            (image.width, image.height),
            (96, 96),
            "48 list px at 2 px each"
        );
        assert!((rect.width() - 48.0).abs() < 1e-9, "{rect:?}");
        assert!(matches!(commands[2], Command::Pop));
        assert_eq!(lowered.list.commands.len(), list.commands.len() + 2);
        assert!(svg::write(&lowered.list, &svg::Options::default()).is_ok());
    }

    #[test]
    fn a_rasterized_sweep_without_clip_paths_reports_both() {
        let caps = Caps {
            clip_paths: false,
            sweep: false,
            ..Caps::ALL
        };
        let lowered = lower::lower(&sink::with_sweep(), caps).unwrap();
        assert_eq!(
            lowered.warnings,
            [
                Warning::UnsupportedClip { command: 9 },
                Warning::SweepRasterized { command: SWEEP },
                Warning::UnsupportedClip { command: SWEEP },
            ]
        );
    }

    #[test]
    fn lowering_glyphs_markers_and_sweeps_keeps_every_channel_within_16() {
        let list = sink::with_sweep();
        let caps = Caps {
            glyphs: false,
            markers: false,
            sweep: false,
            clip_paths: true,
        };
        let lowered = lower::lower(&list, caps).unwrap();
        assert_eq!(lowered.warnings.len(), 1);
        let d = diff(&whole(&lowered.list, 2.0), &whole(&list, 2.0));
        assert!(
            d.max() <= 16,
            "max {}, worst tile at {:?}",
            d.max(),
            d.worst_tile()
        );
    }
}

#[cfg(not(feature = "raster"))]
#[test]
fn without_raster_a_sweep_takes_its_middle_stop() {
    use rustd3_figure::peniko::{Brush, Color};

    let lowered = lower::lower(&sink::with_sweep(), rustd3_figure::svg::CAPS).unwrap();
    assert_eq!(lowered.warnings, [Warning::SweepFlattened { command: 14 }]);
    let Command::Fill { paint, .. } = &lowered.list.commands[14] else {
        panic!("a fill");
    };
    assert_eq!(*paint, Brush::Solid(Color::from_rgb8(0x40, 0xe0, 0x40)));
}
