//! Layer list is now integrated into the timeline panel.
//! This module is kept for backward compatibility but is a no-op.

use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiContexts;

use crate::editor::EditorProject;

/// No-op — layer list is now part of the unified timeline panel.
pub fn layer_panel_system(_contexts: EguiContexts, _project: Option<ResMut<EditorProject>>) {
    // Integrated into timeline_ui_system
}
