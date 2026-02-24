//! Menu bar with File operations (Open, Save).

use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiContexts;

use crate::editor::EditorProject;

/// Event sent when the user requests to open a file.
#[derive(Message)]
pub struct OpenFileRequest {
    pub path: std::path::PathBuf,
}

/// Event sent when the user requests to save the current project.
#[derive(Message)]
pub struct SaveFileRequest;

/// System that draws the top menu bar.
pub fn menu_bar_system(
    mut contexts: EguiContexts,
    project: Option<Res<EditorProject>>,
    mut open_events: MessageWriter<OpenFileRequest>,
    mut save_events: MessageWriter<SaveFileRequest>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open...").clicked() {
                    ui.close();
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Alight Motion Project", &["amproj"])
                        .pick_file()
                    {
                        open_events.write(OpenFileRequest { path });
                    }
                }

                let has_project = project.is_some();
                if ui
                    .add_enabled(has_project, egui::Button::new("Save"))
                    .clicked()
                {
                    ui.close();
                    save_events.write(SaveFileRequest);
                }

                ui.separator();

                if ui.button("Quit").clicked() {
                    std::process::exit(0);
                }
            });

            // Show project info in menu bar
            if let Some(ref proj) = project {
                ui.separator();
                let title = &proj.scene.title;
                let dirty = if proj.dirty { " •" } else { "" };
                ui.label(format!("{title}{dirty}"));
                ui.label(format!(
                    "{}×{} @ {}fps",
                    proj.scene.width, proj.scene.height, proj.scene.fps
                ));
            }
        });
    });
}
