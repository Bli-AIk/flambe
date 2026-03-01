//! Property panel — shows transform properties of the selected layer.

use bevy::prelude::*;
use bevy_workbench::dock::WorkbenchPanel;

use crate::editor::EditorProject;

/// Property panel for bevy_workbench dock.
pub struct PropertyPanel;

impl WorkbenchPanel for PropertyPanel {
    fn id(&self) -> &str {
        "flambe_properties"
    }

    fn title(&self) -> String {
        "Properties".into()
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Properties requires world access");
    }

    fn ui_world(&mut self, ui: &mut egui::Ui, world: &mut World) {
        property_ui(ui, world);
    }

    fn needs_world(&self) -> bool {
        true
    }
}

/// Draw the property panel for the selected layer.
fn property_ui(ui: &mut egui::Ui, world: &mut World) {
    ui.heading("Properties");
    ui.separator();

    let project = world.get_resource::<EditorProject>();
    let Some(project) = project else {
        return;
    };

    let Some(idx) = project.selected_layer else {
        ui.label("No layer selected");
        return;
    };

    if idx >= project.scene.layers.len() {
        ui.label("Invalid selection");
        return;
    }

    use bevy_alight_motion::schema::AmLayer;
    let layer = &project.scene.layers[idx];

    match layer {
        AmLayer::Shape(s) => {
            ui.label(format!("Shape: {}", s.label));
            ui.separator();
            show_transform(ui, &s.transform);
            ui.separator();
            ui.label(format!("Shape type: {}", s.shape_type));
            ui.label(format!("Fill: {}", s.fill_type));
            if !s.effects.is_empty() {
                ui.separator();
                ui.label(format!("Effects: {}", s.effects.len()));
                for eff in &s.effects {
                    ui.label(format!("  • {}", eff.id));
                }
            }
        }
        AmLayer::Nullobj(n) => {
            ui.label(format!("Null Object: {}", n.label));
            ui.separator();
            show_transform(ui, &n.transform);
        }
        AmLayer::EmbedScene(e) => {
            ui.label(format!("Embed Scene: {}", e.label));
            ui.separator();
            show_transform(ui, &e.transform);
        }
        AmLayer::Text(t) => {
            ui.label(format!("Text: {}", t.label));
            ui.separator();
            show_transform(ui, &t.transform);
        }
        AmLayer::Image(img) => {
            ui.label(format!("Image: {}", img.label));
            ui.separator();
            show_transform(ui, &img.transform);
        }
        _ => {
            ui.label("(read-only layer type)");
        }
    }
}

fn show_transform(ui: &mut egui::Ui, transform: &bevy_alight_motion::schema::AmTransform) {
    ui.collapsing("Transform", |ui| {
        // Location
        if let Some(v) = &transform.location.value {
            ui.horizontal(|ui| {
                ui.label("Position:");
                ui.label(format!("{:.1}, {:.1}, {:.1}", v[0], v[1], v[2]));
            });
        }
        let loc_kf = transform.location.keyframes.len();
        if loc_kf > 0 {
            ui.label(format!("  ({loc_kf} keyframes)"));
        }

        // Rotation
        if let Some(v) = transform.rotation.value {
            ui.horizontal(|ui| {
                ui.label("Rotation:");
                ui.label(format!("{v:.1}°"));
            });
        }
        let rot_kf = transform.rotation.keyframes.len();
        if rot_kf > 0 {
            ui.label(format!("  ({rot_kf} keyframes)"));
        }

        // Scale
        if let Some(v) = &transform.scale.value {
            ui.horizontal(|ui| {
                ui.label("Scale:");
                ui.label(format!("{:.2}, {:.2}", v[0], v[1]));
            });
        }
        let scl_kf = transform.scale.keyframes.len();
        if scl_kf > 0 {
            ui.label(format!("  ({scl_kf} keyframes)"));
        }

        // Opacity
        if let Some(v) = transform.opacity.value {
            ui.horizontal(|ui| {
                ui.label("Opacity:");
                ui.label(format!("{v:.2}"));
            });
        }
        let opa_kf = transform.opacity.keyframes.len();
        if opa_kf > 0 {
            ui.label(format!("  ({opa_kf} keyframes)"));
        }

        // Pivot
        if let Some(v) = &transform.pivot.value {
            ui.horizontal(|ui| {
                ui.label("Pivot:");
                ui.label(format!("{:.1}, {:.1}", v[0], v[1]));
            });
        }
    });
}
