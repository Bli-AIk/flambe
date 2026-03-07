//! Playback transport controls (play/pause/loop/seek).

use bevy::prelude::*;
use bevy_alight_motion::prelude::AmPlayback;
use bevy_egui::EguiContexts;

use crate::editor::EditorProject;

/// System that draws playback transport controls in egui.
pub fn playback_controls_system(
    mut contexts: EguiContexts,
    mut playback: Option<ResMut<AmPlayback>>,
    mut project: Option<ResMut<EditorProject>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    egui::TopBottomPanel::bottom("playback_controls")
        .exact_height(32.0)
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                transport_controls_ui(ui, &mut playback, &mut project);
            });
        });
}

fn transport_controls_ui(
    ui: &mut egui::Ui,
    playback: &mut Option<ResMut<AmPlayback>>,
    project: &mut Option<ResMut<EditorProject>>,
) {
    let (playing, looping, current_ms, total_ms, speed) = if let Some(pb) = playback.as_ref() {
        (
            pb.playing,
            pb.looping,
            pb.current_time_ms,
            pb.total_time_ms,
            pb.speed,
        )
    } else {
        (false, false, 0.0, 1000.0, 1.0)
    };

    // Reset button
    if ui.button("⏮").clicked() {
        if let Some(pb) = playback.as_mut() {
            pb.reset();
        }
        if let Some(proj) = project.as_mut() {
            proj.playhead_frame = 0;
        }
    }

    // Play/Pause
    let play_label = if playing { "⏸" } else { "▶" };
    if ui.button(play_label).clicked()
        && let Some(pb) = playback.as_mut()
    {
        pb.toggle();
    }

    // Loop toggle
    let loop_label = if looping { "🔁" } else { "🔂" };
    if ui.button(loop_label).on_hover_text("Toggle loop").clicked()
        && let Some(pb) = playback.as_mut()
    {
        pb.looping = !pb.looping;
    }

    // Time display
    let secs = current_ms / 1000.0;
    let total_secs = total_ms / 1000.0;
    ui.label(format!("{secs:.2}s / {total_secs:.2}s"));

    // Speed control
    ui.separator();
    ui.label("Speed:");
    let mut new_speed = speed;
    ui.add(
        egui::DragValue::new(&mut new_speed)
            .range(0.1..=4.0)
            .speed(0.05)
            .suffix("×"),
    );
    if let Some(pb) = playback.as_mut() {
        pb.speed = new_speed;
    }
}
