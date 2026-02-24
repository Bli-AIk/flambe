//! Editor ↔ ECS synchronization.
//!
//! Pushes EditorProject changes into the Bevy ECS world
//! and reads back runtime state (e.g., playback position).

use bevy::prelude::*;
use bevy_alight_motion::prelude::AmPlayback;

use crate::editor::EditorProject;

/// System that syncs AmPlayback → EditorProject playhead position.
pub fn sync_playback_to_editor(
    playback: Option<Res<AmPlayback>>,
    project: Option<ResMut<EditorProject>>,
) {
    let (Some(pb), Some(ref mut proj)) = (playback, project) else {
        return;
    };
    proj.playhead_frame = pb.current_time_ms as u32;
}
