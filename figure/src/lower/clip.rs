//! The clip pass: path clips stay in place.

use crate::list::{Clip, Command};

/// Whether `command` opens a group clipped to a path.
pub(super) fn is_path(command: &Command) -> bool {
    matches!(
        command,
        Command::Push {
            clip: Some(Clip::Path(_)),
            ..
        }
    )
}
