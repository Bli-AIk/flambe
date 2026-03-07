//! Preview dock — renders AM project to a texture, displayed as a WorkbenchPanel.
//!
//! The render target image matches the project resolution, and the preview panel
//! shows it with a fixed aspect ratio and configurable zoom (Auto / fixed %).

use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiTextureHandle};
use bevy_workbench::dock::WorkbenchPanel;
use bevy_workbench::game_view::ViewZoom;
use bevy_workbench::theme::gray;

use crate::editor::EditorProject;

const TEXT_SUBDUED: egui::Color32 = gray::S550;

/// Resource holding the render target and preview settings.
#[derive(Resource)]
pub struct PreviewState {
    pub render_target: Handle<Image>,
    pub zoom: ViewZoom,
    pub width: u32,
    pub height: u32,
    pub egui_texture_id: Option<egui::TextureId>,
}

/// Startup system: create the render target image and cameras.
pub fn setup_preview(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let width = 1920u32;
    let height = 1080u32;

    let image = Image::new_target_texture(
        width,
        height,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        None,
    );
    let render_target = images.add(image);

    // Window camera — renders to the window (same as the original setup_camera).
    // egui panels cover the window so the user doesn't see the background rendering.
    commands.spawn(Camera2d);

    // Preview camera — renders AM project to texture for the egui preview panel.
    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        RenderTarget::from(render_target.clone()),
    ));

    commands.insert_resource(PreviewState {
        render_target,
        zoom: ViewZoom::Auto,
        width,
        height,
        egui_texture_id: None,
    });
}

/// System: resize the render target when a project is loaded (if resolution differs).
pub fn update_preview_resolution(
    project: Option<Res<EditorProject>>,
    mut state: ResMut<PreviewState>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(project) = project else { return };
    let w = project.scene.width;
    let h = project.scene.height;
    if w == 0 || h == 0 || (state.width == w && state.height == h) {
        return;
    }

    let new_image = Image::new_target_texture(
        w,
        h,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        None,
    );
    if let Some(img) = images.get_mut(&state.render_target) {
        *img = new_image;
    }
    state.width = w;
    state.height = h;
}

/// System: register the render target as an egui texture and sync to the dock panel.
pub fn sync_preview_to_panel(
    mut state: ResMut<PreviewState>,
    mut contexts: EguiContexts,
    mut tile_state: ResMut<bevy_workbench::dock::TileLayoutState>,
    project: Option<Res<EditorProject>>,
) {
    // Register texture with egui (once)
    if state.egui_texture_id.is_none() && state.render_target != Handle::default() {
        let texture_id = contexts.add_image(EguiTextureHandle::Strong(state.render_target.clone()));
        state.egui_texture_id = Some(texture_id);
    }

    let has_project = project.is_some();

    // Sync state to the dock panel
    if let Some(panel) = tile_state.get_panel_mut::<PreviewPanel>("flambe_preview") {
        panel.egui_texture_id = state.egui_texture_id;
        panel.width = state.width;
        panel.height = state.height;
        panel.has_project = has_project;
        // Sync zoom FROM panel TO state (panel owns zoom via ComboBox)
        state.zoom = panel.zoom;

        if let Some(ref proj) = project {
            panel.resolution_text = format!("{}×{}", proj.scene.width, proj.scene.height);
        }
    }
}

/// Preview dock panel for the Flambé editor.
#[derive(Default)]
pub struct PreviewPanel {
    pub egui_texture_id: Option<egui::TextureId>,
    pub width: u32,
    pub height: u32,
    pub zoom: ViewZoom,
    pub has_project: bool,
    pub resolution_text: String,
}

fn preview_toolbar_ui(zoom: &mut ViewZoom, resolution_text: &str, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.colored_label(TEXT_SUBDUED, "Preview");
        ui.separator();

        let zoom_label = match *zoom {
            ViewZoom::Auto => "Auto".to_string(),
            ViewZoom::Fixed(z) => format!("{:.0}%", z * 100.0),
        };

        egui::ComboBox::from_id_salt("preview_zoom")
            .selected_text(&zoom_label)
            .show_ui(ui, |ui| {
                ui.selectable_value(zoom, ViewZoom::Auto, "Auto");
                ui.selectable_value(zoom, ViewZoom::Fixed(0.5), "50%");
                ui.selectable_value(zoom, ViewZoom::Fixed(0.75), "75%");
                ui.selectable_value(zoom, ViewZoom::Fixed(1.0), "100%");
                ui.selectable_value(zoom, ViewZoom::Fixed(1.25), "125%");
                ui.selectable_value(zoom, ViewZoom::Fixed(1.5), "150%");
                ui.selectable_value(zoom, ViewZoom::Fixed(2.0), "200%");
            });

        if !resolution_text.is_empty() {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.colored_label(TEXT_SUBDUED, resolution_text);
            });
        }
    });
}

impl WorkbenchPanel for PreviewPanel {
    fn id(&self) -> &str {
        "flambe_preview"
    }

    fn title(&self) -> String {
        "Preview".into()
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        if !self.has_project {
            ui.centered_and_justified(|ui| {
                ui.colored_label(TEXT_SUBDUED, "No project loaded — File > Open...");
            });
            return;
        }

        // Toolbar
        preview_toolbar_ui(&mut self.zoom, &self.resolution_text, ui);

        ui.separator();

        // Preview image
        let Some(tex_id) = self.egui_texture_id else {
            return;
        };

        let avail = ui.available_size();
        if avail.x <= 0.0 || avail.y <= 0.0 {
            return;
        }

        let aspect = self.width as f32 / self.height.max(1) as f32;

        let display_size = match self.zoom {
            ViewZoom::Auto => {
                let w = avail.x;
                let h = w / aspect;
                if h > avail.y {
                    egui::vec2(avail.y * aspect, avail.y)
                } else {
                    egui::vec2(w, h)
                }
            }
            ViewZoom::Fixed(z) => egui::vec2(self.width as f32 * z, self.height as f32 * z),
        };

        let padding = (avail - display_size).max(egui::Vec2::ZERO) * 0.5;

        if matches!(self.zoom, ViewZoom::Fixed(_))
            && (display_size.x > avail.x || display_size.y > avail.y)
        {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.image(egui::load::SizedTexture::new(tex_id, display_size));
            });
        } else {
            ui.add_space(padding.y);
            ui.vertical_centered(|ui| {
                ui.image(egui::load::SizedTexture::new(tex_id, display_size));
            });
        }
    }

    fn closable(&self) -> bool {
        false
    }
}
