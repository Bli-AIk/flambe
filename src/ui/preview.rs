//! Preview dock — renders AM project to a texture, displayed in an egui CentralPanel.
//!
//! The render target image matches the project resolution, and the preview panel
//! shows it with a fixed aspect ratio and configurable zoom (Auto / fixed %).

use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiTextureHandle};
use bevy_workbench::theme::gray;

use crate::editor::EditorProject;

const PANEL_BG: egui::Color32 = gray::S100;
const TEXT_SUBDUED: egui::Color32 = gray::S550;

/// Zoom mode for the preview panel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PreviewZoom {
    /// Fit the image within the available panel space, preserving aspect ratio.
    Auto,
    /// Display at a fixed scale factor (1.0 = 100%).
    Fixed(f32),
}

impl Default for PreviewZoom {
    fn default() -> Self {
        Self::Auto
    }
}

/// Resource holding the render target and preview settings.
#[derive(Resource)]
pub struct PreviewState {
    pub render_target: Handle<Image>,
    pub zoom: PreviewZoom,
    pub width: u32,
    pub height: u32,
    egui_registered: bool,
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
        zoom: PreviewZoom::Auto,
        width,
        height,
        egui_registered: false,
    });
}

/// System: resize the render target when a project is loaded (if resolution differs).
pub fn update_preview_resolution(
    project: Option<Res<EditorProject>>,
    mut state: ResMut<PreviewState>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(project) = project else { return };
    let w = project.scene.width as u32;
    let h = project.scene.height as u32;
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

/// System: draw the preview panel showing the render-target texture.
pub fn preview_panel_system(
    mut contexts: EguiContexts,
    mut state: ResMut<PreviewState>,
    project: Option<Res<EditorProject>>,
) {
    if !state.egui_registered {
        contexts.add_image(EguiTextureHandle::Strong(state.render_target.clone()));
        state.egui_registered = true;
    }

    let texture_id = contexts.image_id(state.render_target.id());

    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(PANEL_BG))
        .show(ctx, |ui| {
            // ── Toolbar ──────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.colored_label(TEXT_SUBDUED, "Preview");
                ui.separator();

                let zoom_label = match state.zoom {
                    PreviewZoom::Auto => "Auto".to_string(),
                    PreviewZoom::Fixed(z) => format!("{:.0}%", z * 100.0),
                };

                egui::ComboBox::from_id_salt("preview_zoom")
                    .selected_text(&zoom_label)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut state.zoom, PreviewZoom::Auto, "Auto");
                        ui.selectable_value(&mut state.zoom, PreviewZoom::Fixed(0.5), "50%");
                        ui.selectable_value(&mut state.zoom, PreviewZoom::Fixed(0.75), "75%");
                        ui.selectable_value(&mut state.zoom, PreviewZoom::Fixed(1.0), "100%");
                        ui.selectable_value(&mut state.zoom, PreviewZoom::Fixed(1.25), "125%");
                        ui.selectable_value(&mut state.zoom, PreviewZoom::Fixed(1.5), "150%");
                        ui.selectable_value(&mut state.zoom, PreviewZoom::Fixed(2.0), "200%");
                    });

                if let Some(ref proj) = project {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.colored_label(
                            TEXT_SUBDUED,
                            format!("{}×{}", proj.scene.width, proj.scene.height),
                        );
                    });
                }
            });

            ui.separator();

            // ── Preview image ────────────────────────────────────
            let Some(tex_id) = texture_id else { return };

            let avail = ui.available_size();
            if avail.x <= 0.0 || avail.y <= 0.0 {
                return;
            }

            let aspect = state.width as f32 / state.height.max(1) as f32;

            let display_size = match state.zoom {
                PreviewZoom::Auto => {
                    let w = avail.x;
                    let h = w / aspect;
                    if h > avail.y {
                        egui::vec2(avail.y * aspect, avail.y)
                    } else {
                        egui::vec2(w, h)
                    }
                }
                PreviewZoom::Fixed(z) => {
                    egui::vec2(state.width as f32 * z, state.height as f32 * z)
                }
            };

            // Center the image in the available space.
            let padding = (avail - display_size).max(egui::Vec2::ZERO) * 0.5;

            // For fixed zoom that overflows, use a scroll area.
            if matches!(state.zoom, PreviewZoom::Fixed(_))
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
        });
}
