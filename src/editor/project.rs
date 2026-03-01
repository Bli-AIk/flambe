//! EditorProject — wraps an AmScene with editor-specific state.

use bevy::prelude::*;
use bevy_alight_motion::schema::AmScene;
use std::path::PathBuf;

/// The main editor project resource, wrapping an AM scene with undo/selection state.
#[derive(Resource)]
pub struct EditorProject {
    /// The loaded AM scene data.
    pub scene: AmScene,
    /// Path to the .amproj file on disk (None if unsaved).
    pub file_path: Option<PathBuf>,
    /// Index of the currently selected layer (None if no selection).
    pub selected_layer: Option<usize>,
    /// Current playhead position in frames.
    pub playhead_frame: u32,
    /// Whether the project has unsaved modifications.
    pub dirty: bool,
}

impl EditorProject {
    pub fn new(scene: AmScene) -> Self {
        Self {
            scene,
            file_path: None,
            selected_layer: None,
            playhead_frame: 0,
            dirty: false,
        }
    }

    pub fn from_file(scene: AmScene, path: PathBuf) -> Self {
        Self {
            scene,
            file_path: Some(path),
            selected_layer: None,
            playhead_frame: 0,
            dirty: false,
        }
    }
}
