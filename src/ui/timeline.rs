//! Unified layer+timeline panel — AE-style layout with layer list and timeline tracks.
//!
//! Left: layer names (selectable), Right: timeline tracks with keyframes and playhead.

use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiContexts;

use crate::editor::EditorProject;
use crate::ui::theme;

/// Height of one track row in pixels.
const TRACK_HEIGHT: f32 = 24.0;
/// Width of the layer name column in pixels.
const LAYER_COL_WIDTH: f32 = 180.0;
/// Height of the ruler/header bar in pixels.
const RULER_HEIGHT: f32 = 22.0;
/// Keyframe diamond half-size in pixels.
const KF_SIZE: f32 = 5.0;

/// Timeline view state (zoom, scroll, etc.).
#[derive(Resource)]
pub struct TimelineState {
    /// Pixels per millisecond (zoom level).
    pub px_per_ms: f32,
    /// Horizontal scroll offset in milliseconds.
    pub scroll_ms: f32,
    /// Whether the playhead is being dragged.
    pub dragging_playhead: bool,
}

impl Default for TimelineState {
    fn default() -> Self {
        Self {
            px_per_ms: 0.15,
            scroll_ms: 0.0,
            dragging_playhead: false,
        }
    }
}

impl TimelineState {
    fn ms_to_x(&self, ms: f32) -> f32 {
        (ms - self.scroll_ms) * self.px_per_ms
    }

    fn x_to_ms(&self, x: f32) -> f32 {
        x / self.px_per_ms + self.scroll_ms
    }
}

/// Collected info for one track.
struct TrackInfo {
    label: String,
    icon: &'static str,
    start_ms: f32,
    end_ms: f32,
    keyframe_times: Vec<f32>,
}

/// System that draws the unified layer+timeline panel.
pub fn timeline_ui_system(
    mut contexts: EguiContexts,
    mut project: Option<ResMut<EditorProject>>,
    mut state: ResMut<TimelineState>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::TopBottomPanel::bottom("timeline_panel")
        .resizable(true)
        .min_height(100.0)
        .default_height(240.0)
        .show(ctx, |ui| {
            let Some(ref mut project) = project else {
                ui.centered_and_justified(|ui| {
                    ui.label("No project loaded — use File > Open...");
                });
                return;
            };

            let tracks = collect_tracks(&project.scene);
            let total_time = project.scene.total_time as f32;
            let playhead_ms = project.playhead_frame as f32;

            // Zoom controls bar
            ui.horizontal(|ui| {
                if ui.button("−").clicked() {
                    state.px_per_ms = (state.px_per_ms * 0.8).max(0.01);
                }
                ui.label(format!("{:.0}%", state.px_per_ms * 1000.0));
                if ui.button("+").clicked() {
                    state.px_per_ms = (state.px_per_ms * 1.25).min(5.0);
                }
                ui.separator();
                let secs = playhead_ms / 1000.0;
                let total_secs = total_time / 1000.0;
                ui.label(format!("{secs:.2}s / {total_secs:.2}s"));
            });

            // Main area: layer list on left, timeline on right
            let avail = ui.available_rect_before_wrap();

            let layer_rect = egui::Rect::from_min_max(
                avail.min,
                egui::pos2(avail.min.x + LAYER_COL_WIDTH, avail.max.y),
            );
            let timeline_rect = egui::Rect::from_min_max(
                egui::pos2(avail.min.x + LAYER_COL_WIDTH, avail.min.y),
                avail.max,
            );

            let painter = ui.painter_at(avail);

            // Column separator
            painter.line_segment(
                [
                    egui::pos2(layer_rect.max.x, avail.min.y),
                    egui::pos2(layer_rect.max.x, avail.max.y),
                ],
                egui::Stroke::new(1.0, theme::SEPARATOR_COLOR),
            );

            // Header row (layer column)
            let header_rect = egui::Rect::from_min_size(
                layer_rect.min,
                egui::vec2(LAYER_COL_WIDTH, RULER_HEIGHT),
            );
            painter.rect_filled(header_rect, 0.0, theme::HEADER_BG);
            painter.text(
                egui::pos2(header_rect.min.x + 6.0, header_rect.min.y + 3.0),
                egui::Align2::LEFT_TOP,
                "Layers",
                egui::FontId::proportional(12.0),
                theme::HEADER_TEXT_COLOR,
            );

            // Ruler (timeline column)
            let ruler_rect = egui::Rect::from_min_size(
                timeline_rect.min,
                egui::vec2(timeline_rect.width(), RULER_HEIGHT),
            );
            painter.rect_filled(ruler_rect, 0.0, theme::HEADER_BG);
            draw_ruler(&painter, &state, ruler_rect, total_time);

            // Track rows
            let tracks_top = avail.min.y + RULER_HEIGHT;
            let mut new_selection = project.selected_layer;

            for (i, track) in tracks.iter().enumerate() {
                let y = tracks_top + i as f32 * TRACK_HEIGHT;
                if y + TRACK_HEIGHT < avail.min.y || y > avail.max.y {
                    continue;
                }

                let is_selected = project.selected_layer == Some(i);
                let bg = if is_selected {
                    theme::ROW_SELECTED_BG
                } else if i % 2 == 0 {
                    theme::ROW_EVEN_BG
                } else {
                    theme::ROW_ODD_BG
                };

                // Layer name cell
                let name_rect = egui::Rect::from_min_size(
                    egui::pos2(layer_rect.min.x, y),
                    egui::vec2(LAYER_COL_WIDTH, TRACK_HEIGHT),
                );
                painter.rect_filled(name_rect, 0.0, bg);
                painter.text(
                    egui::pos2(layer_rect.min.x + 6.0, y + 4.0),
                    egui::Align2::LEFT_TOP,
                    format!("{} {}", track.icon, track.label),
                    egui::FontId::proportional(12.0),
                    theme::LAYER_TEXT_COLOR,
                );

                // Timeline track cell
                let track_row = egui::Rect::from_min_size(
                    egui::pos2(timeline_rect.min.x, y),
                    egui::vec2(timeline_rect.width(), TRACK_HEIGHT),
                );
                painter.rect_filled(track_row, 0.0, bg);

                // Track bar
                let bar_x0 = timeline_rect.min.x + state.ms_to_x(track.start_ms);
                let bar_x1 = timeline_rect.min.x + state.ms_to_x(track.end_ms);
                if bar_x1 > timeline_rect.min.x && bar_x0 < timeline_rect.max.x {
                    let bar = egui::Rect::from_min_max(
                        egui::pos2(bar_x0.max(timeline_rect.min.x), y + 4.0),
                        egui::pos2(
                            bar_x1.min(timeline_rect.max.x),
                            y + TRACK_HEIGHT - 4.0,
                        ),
                    );
                    let bar_color = if is_selected {
                        theme::BAR_SELECTED_COLOR
                    } else {
                        theme::BAR_COLOR
                    };
                    painter.rect_filled(bar, 3.0, bar_color);

                    // Keyframe diamonds
                    for &kf_t in &track.keyframe_times {
                        let kf_ms =
                            track.start_ms + kf_t * (track.end_ms - track.start_ms);
                        let kx = timeline_rect.min.x + state.ms_to_x(kf_ms);
                        if kx >= timeline_rect.min.x && kx <= timeline_rect.max.x {
                            let cy = y + TRACK_HEIGHT / 2.0;
                            let diamond = [
                                egui::pos2(kx, cy - KF_SIZE),
                                egui::pos2(kx + KF_SIZE, cy),
                                egui::pos2(kx, cy + KF_SIZE),
                                egui::pos2(kx - KF_SIZE, cy),
                            ];
                            painter.add(egui::Shape::convex_polygon(
                                diamond.to_vec(),
                                theme::KEYFRAME_COLOR,
                                egui::Stroke::NONE,
                            ));
                        }
                    }
                }

                // Click to select layer
                let name_resp = ui.interact(
                    name_rect,
                    ui.id().with(("layer_sel", i)),
                    egui::Sense::click(),
                );
                if name_resp.clicked() {
                    new_selection = Some(i);
                }
            }

            project.selected_layer = new_selection;

            // Playhead
            let ph_x = timeline_rect.min.x + state.ms_to_x(playhead_ms);
            if ph_x >= timeline_rect.min.x && ph_x <= timeline_rect.max.x {
                painter.line_segment(
                    [
                        egui::pos2(ph_x, avail.min.y),
                        egui::pos2(ph_x, avail.max.y),
                    ],
                    egui::Stroke::new(2.0, theme::PLAYHEAD_COLOR),
                );
                let tri = [
                    egui::pos2(ph_x - 6.0, ruler_rect.min.y),
                    egui::pos2(ph_x + 6.0, ruler_rect.min.y),
                    egui::pos2(ph_x, ruler_rect.max.y),
                ];
                painter.add(egui::Shape::convex_polygon(
                    tri.to_vec(),
                    theme::PLAYHEAD_COLOR,
                    egui::Stroke::NONE,
                ));
            }

            // Playhead drag
            let ruler_resp = ui.interact(
                ruler_rect,
                ui.id().with("ruler_drag"),
                egui::Sense::click_and_drag(),
            );
            if ruler_resp.dragged() || ruler_resp.clicked() {
                if let Some(pos) = ruler_resp.interact_pointer_pos() {
                    let _new_ms = state
                        .x_to_ms(pos.x - timeline_rect.min.x)
                        .clamp(0.0, total_time);
                    state.dragging_playhead = true;
                }
            }
            if ruler_resp.drag_stopped() {
                state.dragging_playhead = false;
            }

            // Mouse wheel scroll/zoom
            let scroll = ui.input(|i| i.raw_scroll_delta);
            if scroll.x != 0.0 {
                state.scroll_ms =
                    (state.scroll_ms - scroll.x / state.px_per_ms).max(0.0);
            }
            if scroll.y != 0.0 && ui.input(|i| i.modifiers.ctrl) {
                let factor = if scroll.y > 0.0 { 1.1 } else { 0.9 };
                state.px_per_ms = (state.px_per_ms * factor).clamp(0.01, 5.0);
            }
        });
}

fn collect_tracks(scene: &bevy_alight_motion::schema::AmScene) -> Vec<TrackInfo> {
    use bevy_alight_motion::schema::AmLayer;
    scene
        .layers
        .iter()
        .filter_map(|layer| match layer {
            AmLayer::Shape(s) => Some(TrackInfo {
                label: s.label.clone(),
                icon: "■",
                start_ms: s.start_time as f32,
                end_ms: s.end_time as f32,
                keyframe_times: collect_kf_times(&s.transform),
            }),
            AmLayer::Nullobj(n) => Some(TrackInfo {
                label: n.label.clone(),
                icon: "○",
                start_ms: n.start_time as f32,
                end_ms: n.end_time as f32,
                keyframe_times: collect_kf_times(&n.transform),
            }),
            AmLayer::EmbedScene(e) => Some(TrackInfo {
                label: e.label.clone(),
                icon: "⊞",
                start_ms: e.start_time as f32,
                end_ms: e.end_time as f32,
                keyframe_times: collect_kf_times(&e.transform),
            }),
            AmLayer::Text(t) => Some(TrackInfo {
                label: t.label.clone(),
                icon: "T",
                start_ms: t.start_time as f32,
                end_ms: t.end_time as f32,
                keyframe_times: collect_kf_times(&t.transform),
            }),
            AmLayer::Image(img) => Some(TrackInfo {
                label: img.label.clone(),
                icon: "🖼",
                start_ms: img.start_time as f32,
                end_ms: img.end_time as f32,
                keyframe_times: collect_kf_times(&img.transform),
            }),
            AmLayer::Bookmark(b) => Some(TrackInfo {
                label: b.label.clone(),
                icon: "🔖",
                start_ms: 0.0,
                end_ms: 0.0,
                keyframe_times: vec![],
            }),
            _ => None,
        })
        .collect()
}

fn draw_ruler(
    painter: &egui::Painter,
    state: &TimelineState,
    ruler_rect: egui::Rect,
    total_time: f32,
) {
    let step_ms = calc_ruler_step(state.px_per_ms);
    let start = ((state.scroll_ms / step_ms).floor() as i32).max(0) as f32 * step_ms;
    let mut ms = start;
    while ms <= total_time {
        let x = ruler_rect.min.x + state.ms_to_x(ms);
        if x >= ruler_rect.min.x && x <= ruler_rect.max.x {
            painter.line_segment(
                [
                    egui::pos2(x, ruler_rect.min.y),
                    egui::pos2(x, ruler_rect.max.y),
                ],
                egui::Stroke::new(1.0, theme::RULER_TICK_COLOR),
            );
            let secs = ms / 1000.0;
            painter.text(
                egui::pos2(x + 2.0, ruler_rect.min.y + 3.0),
                egui::Align2::LEFT_TOP,
                format!("{secs:.1}s"),
                egui::FontId::proportional(10.0),
                theme::RULER_TEXT_COLOR,
            );
        }
        ms += step_ms;
    }
}

fn collect_kf_times(transform: &bevy_alight_motion::schema::AmTransform) -> Vec<f32> {
    let mut times = Vec::new();
    for kf in &transform.location.keyframes {
        times.push(kf.time);
    }
    for kf in &transform.rotation.keyframes {
        times.push(kf.time);
    }
    for kf in &transform.scale.keyframes {
        times.push(kf.time);
    }
    for kf in &transform.opacity.keyframes {
        times.push(kf.time);
    }
    times
}

fn calc_ruler_step(px_per_ms: f32) -> f32 {
    let target_px = 80.0;
    let raw_ms = target_px / px_per_ms;
    let steps = [100.0, 200.0, 500.0, 1000.0, 2000.0, 5000.0, 10000.0];
    for &s in &steps {
        if s >= raw_ms {
            return s;
        }
    }
    10000.0
}
