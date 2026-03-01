//! Flambe menu items integrated into bevy_workbench menu bar via extensions.

use bevy::prelude::*;
use bevy_workbench::prelude::*;

use crate::editor::EditorProject;

/// Event sent when the user requests to open a file.
#[derive(Message)]
pub struct OpenFileRequest {
    pub path: std::path::PathBuf,
}

/// Event sent when the user requests to save the current project.
#[derive(Message)]
pub struct SaveFileRequest;

/// System that updates menu bar extensions based on project state.
pub fn flambe_menu_sync_system(
    project: Option<Res<EditorProject>>,
    mut extensions: ResMut<MenuBarExtensions>,
) {
    let has_project = project.is_some();

    extensions.file_items = vec![
        MenuExtItem {
            id: "open",
            label: "Open...".into(),
            enabled: true,
        },
        MenuExtItem {
            id: "save",
            label: "Save".into(),
            enabled: has_project,
        },
    ];

    extensions.info_text = project.map(|proj| {
        let dirty = if proj.dirty { " •" } else { "" };
        format!(
            "{}{dirty}  {}×{} @ {}fps",
            proj.scene.title, proj.scene.width, proj.scene.height, proj.scene.fps
        )
    });
}

/// System that handles menu actions from the workbench menu bar.
pub fn flambe_menu_action_system(
    mut actions: MessageReader<MenuAction>,
    mut open_events: MessageWriter<OpenFileRequest>,
    mut save_events: MessageWriter<SaveFileRequest>,
) {
    for action in actions.read() {
        match action.id {
            "open" => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Alight Motion Project", &["amproj"])
                    .pick_file()
                {
                    open_events.write(OpenFileRequest { path });
                }
            }
            "save" => {
                save_events.write(SaveFileRequest);
            }
            _ => {}
        }
    }
}
