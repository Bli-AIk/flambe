//! Unified layer+timeline panel — AE-style layout with layer list and timeline tracks.
//!
//! Left: layer names (selectable), Right: timeline tracks with keyframes and playhead.
//! Transport controls integrated into the header bar (like Rerun).

use bevy::prelude::*;
use bevy_alight_motion::prelude::AmPlayback;
use bevy_workbench::dock::WorkbenchPanel;
use bevy_workbench::theme::gray;

use crate::editor::EditorProject;

const TRACK_HEIGHT: f32 = 24.0;
const LAYER_COL_WIDTH: f32 = 180.0;
const RULER_HEIGHT: f32 = 22.0;
const KF_SIZE: f32 = 4.0;
/// Height of the transport controls bar above the timeline.
const TRANSPORT_HEIGHT: f32 = 28.0;

// ── Color constants for timeline ────────────────────────────────
const PANEL_BG: egui::Color32 = gray::S100;
const HEADER_BG: egui::Color32 = gray::S150;
const ROW_EVEN_BG: egui::Color32 = gray::S100;
const ROW_ODD_BG: egui::Color32 = gray::S125;
const ROW_SELECTED_BG: egui::Color32 = egui::Color32::from_rgb(0x00, 0x25, 0x69);
const BAR_COLOR: egui::Color32 = egui::Color32::from_rgb(0x00, 0x4b, 0xc2);
const BAR_SELECTED_COLOR: egui::Color32 = egui::Color32::from_rgb(0x00, 0x5a, 0xe6);
const SEPARATOR_COLOR: egui::Color32 = gray::S250;
const RULER_TICK_COLOR: egui::Color32 = gray::S300;
const RULER_TEXT_COLOR: egui::Color32 = gray::S500;
const LAYER_TEXT_COLOR: egui::Color32 = gray::S700;
const HEADER_TEXT_COLOR: egui::Color32 = gray::S550;
const KEYFRAME_COLOR: egui::Color32 = gray::S775;
const PLAYHEAD_COLOR: egui::Color32 = egui::Color32::from_rgb(0xFF, 0x50, 0x50);
const TEXT_SUBDUED: egui::Color32 = gray::S550;
const TEXT_DEFAULT: egui::Color32 = gray::S775;
const TEXT_STRONG: egui::Color32 = gray::S1000;

/// Timeline view state (zoom, scroll, etc.).
#[derive(Resource)]
pub struct TimelineState {
    pub px_per_ms: f32,
    pub scroll_ms: f32,
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

struct TrackInfo {
    label: String,
    icon: &'static str,
    start_ms: f32,
    end_ms: f32,
    keyframe_times: Vec<f32>,
}

/// Timeline panel for bevy_workbench dock.
pub struct TimelinePanel;

impl WorkbenchPanel for TimelinePanel {
    fn id(&self) -> &str {
        "flambe_timeline"
    }

    fn title(&self) -> String {
        "Timeline".into()
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Timeline requires world access");
    }

    fn ui_world(&mut self, ui: &mut egui::Ui, world: &mut World) {
        timeline_ui(ui, world);
    }

    fn needs_world(&self) -> bool {
        true
    }

    fn closable(&self) -> bool {
        false
    }
}

/// Draw the unified timeline panel with integrated transport.
fn timeline_ui(ui: &mut egui::Ui, world: &mut World) {
    let mut project = world.get_resource_mut::<EditorProject>();
    let Some(ref mut project) = project else {
        ui.centered_and_justified(|ui| {
            ui.colored_label(TEXT_SUBDUED, "No project loaded — File > Open...");
        });
        return;
    };

    let scene_layers = project.scene.layers.clone();
    let tracks = collect_tracks_from_layers(&scene_layers);
    let total_time = project.scene.total_time as f32;
    let playhead_ms = project.playhead_frame as f32;
    let selected_layer = project.selected_layer;

    // Read playback state
    let (playing, looping, speed) = world
        .get_resource::<AmPlayback>()
        .map(|pb| (pb.playing, pb.looping, pb.speed))
        .unwrap_or((false, false, 1.0));

    let mut state = world.remove_resource::<TimelineState>().unwrap_or_default();

    // ── Transport bar ────────────────────────────────────
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), TRANSPORT_HEIGHT),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = 4.0;

            // Transport buttons
            if ui.small_button("⏮").clicked() {
                if let Some(mut pb) = world.get_resource_mut::<AmPlayback>() {
                    pb.reset();
                }
                if let Some(mut proj) = world.get_resource_mut::<EditorProject>() {
                    proj.playhead_frame = 0;
                }
            }
            let play_label = if playing { "⏸" } else { "▶" };
            if ui.small_button(play_label).clicked() {
                if let Some(mut pb) = world.get_resource_mut::<AmPlayback>() {
                    pb.toggle();
                }
            }
            let loop_label = if looping { "🔁" } else { "🔂" };
            if ui.small_button(loop_label).clicked() {
                if let Some(mut pb) = world.get_resource_mut::<AmPlayback>() {
                    pb.looping = !pb.looping;
                }
            }

            ui.separator();

            // Time display
            ui.colored_label(
                TEXT_DEFAULT,
                format!("{:.2}s / {:.2}s", playhead_ms / 1000.0, total_time / 1000.0),
            );

            ui.separator();

            // Speed
            ui.colored_label(TEXT_SUBDUED, "Speed:");
            let mut new_speed = speed;
            ui.add(
                egui::DragValue::new(&mut new_speed)
                    .range(0.1..=4.0)
                    .speed(0.05)
                    .suffix("×"),
            );
            if let Some(mut pb) = world.get_resource_mut::<AmPlayback>() {
                pb.speed = new_speed;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Zoom controls on the right
                if ui.small_button("+").clicked() {
                    state.px_per_ms = (state.px_per_ms * 1.25).min(5.0);
                }
                ui.colored_label(
                    TEXT_SUBDUED,
                    format!("{:.0}%", state.px_per_ms * 1000.0),
                );
                if ui.small_button("−").clicked() {
                    state.px_per_ms = (state.px_per_ms * 0.8).max(0.01);
                }
            });
        },
    );

    // Thin separator below transport
    let sep_rect =
        egui::Rect::from_min_size(ui.cursor().min, egui::vec2(ui.available_width(), 1.0));
    ui.painter()
        .rect_filled(sep_rect, 0.0, SEPARATOR_COLOR);
    ui.advance_cursor_after_rect(sep_rect);

    // ── Main timeline area ───────────────────────────────
    let avail = ui.available_rect_before_wrap();

    // Claim the full available space so the panel remains resizable.
    ui.allocate_rect(avail, egui::Sense::hover());

    // Fill the entire timeline area with background
    let bg_painter = ui.painter_at(avail);
    bg_painter.rect_filled(avail, 0.0, PANEL_BG);

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
        egui::Stroke::new(1.0, SEPARATOR_COLOR),
    );

    // Header: layer column
    let header_rect = egui::Rect::from_min_size(
        layer_rect.min,
        egui::vec2(LAYER_COL_WIDTH, RULER_HEIGHT),
    );
    painter.rect_filled(header_rect, 0.0, HEADER_BG);
    painter.text(
        egui::pos2(header_rect.min.x + 8.0, header_rect.min.y + 4.0),
        egui::Align2::LEFT_TOP,
        "Layers",
        egui::FontId::proportional(11.0),
        HEADER_TEXT_COLOR,
    );

    // Ruler (timeline column)
    let ruler_rect = egui::Rect::from_min_size(
        timeline_rect.min,
        egui::vec2(timeline_rect.width(), RULER_HEIGHT),
    );
    painter.rect_filled(ruler_rect, 0.0, HEADER_BG);
    draw_ruler(&painter, &state, ruler_rect, total_time);

    // Track rows
    let tracks_top = avail.min.y + RULER_HEIGHT;
    let mut new_selection = selected_layer;

    for (i, track) in tracks.iter().enumerate() {
        let y = tracks_top + i as f32 * TRACK_HEIGHT;
        if y + TRACK_HEIGHT < avail.min.y || y > avail.max.y {
            continue;
        }

        let is_selected = selected_layer == Some(i);
        let bg = if is_selected {
            ROW_SELECTED_BG
        } else if i % 2 == 0 {
            ROW_EVEN_BG
        } else {
            ROW_ODD_BG
        };

        // Layer name cell
        let name_rect = egui::Rect::from_min_size(
            egui::pos2(layer_rect.min.x, y),
            egui::vec2(LAYER_COL_WIDTH, TRACK_HEIGHT),
        );
        painter.rect_filled(name_rect, 0.0, bg);

        let text_color = if is_selected {
            TEXT_STRONG
        } else {
            LAYER_TEXT_COLOR
        };
        painter.text(
            egui::pos2(layer_rect.min.x + 8.0, y + 5.0),
            egui::Align2::LEFT_TOP,
            format!("{} {}", track.icon, track.label),
            egui::FontId::proportional(11.0),
            text_color,
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
                egui::pos2(bar_x0.max(timeline_rect.min.x), y + 5.0),
                egui::pos2(bar_x1.min(timeline_rect.max.x), y + TRACK_HEIGHT - 5.0),
            );
            let bar_color = if is_selected {
                BAR_SELECTED_COLOR
            } else {
                BAR_COLOR
            };
            painter.rect_filled(bar, 3.0, bar_color);

            // Keyframe dots
            for &kf_t in &track.keyframe_times {
                let kf_ms = track.start_ms + kf_t * (track.end_ms - track.start_ms);
                let kx = timeline_rect.min.x + state.ms_to_x(kf_ms);
                if kx >= timeline_rect.min.x && kx <= timeline_rect.max.x {
                    let cy = y + TRACK_HEIGHT / 2.0;
                    painter.circle_filled(
                        egui::pos2(kx, cy),
                        KF_SIZE,
                        KEYFRAME_COLOR,
                    );
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

    // Playhead
    let ph_x = timeline_rect.min.x + state.ms_to_x(playhead_ms);
    if ph_x >= timeline_rect.min.x && ph_x <= timeline_rect.max.x {
        painter.line_segment(
            [egui::pos2(ph_x, avail.min.y), egui::pos2(ph_x, avail.max.y)],
            egui::Stroke::new(1.5, PLAYHEAD_COLOR),
        );
        // Small triangle marker at ruler
        let tri = [
            egui::pos2(ph_x - 5.0, ruler_rect.min.y),
            egui::pos2(ph_x + 5.0, ruler_rect.min.y),
            egui::pos2(ph_x, ruler_rect.min.y + 8.0),
        ];
        painter.add(egui::Shape::convex_polygon(
            tri.to_vec(),
            PLAYHEAD_COLOR,
            egui::Stroke::NONE,
        ));
    }

    // Playhead drag on ruler
    let ruler_resp = ui.interact(
        ruler_rect,
        ui.id().with("ruler_drag"),
        egui::Sense::click_and_drag(),
    );
    if ruler_resp.dragged() || ruler_resp.clicked() {
        if let Some(pos) = ruler_resp.interact_pointer_pos() {
            let new_ms = state
                .x_to_ms(pos.x - timeline_rect.min.x)
                .clamp(0.0, total_time);
            if let Some(mut proj) = world.get_resource_mut::<EditorProject>() {
                proj.playhead_frame = new_ms as u32;
            }
            if let Some(mut pb) = world.get_resource_mut::<AmPlayback>() {
                pb.current_time_ms = new_ms;
            }
            state.dragging_playhead = true;
        }
    }
    if ruler_resp.drag_stopped() {
        state.dragging_playhead = false;
    }

    // Click on timeline track area also seeks
    let track_area = egui::Rect::from_min_max(
        egui::pos2(timeline_rect.min.x, tracks_top),
        timeline_rect.max,
    );
    let track_resp = ui.interact(
        track_area,
        ui.id().with("track_area_click"),
        egui::Sense::click(),
    );
    if track_resp.clicked() {
        if let Some(pos) = track_resp.interact_pointer_pos() {
            let new_ms = state
                .x_to_ms(pos.x - timeline_rect.min.x)
                .clamp(0.0, total_time);
            if let Some(mut proj) = world.get_resource_mut::<EditorProject>() {
                proj.playhead_frame = new_ms as u32;
            }
            if let Some(mut pb) = world.get_resource_mut::<AmPlayback>() {
                pb.current_time_ms = new_ms;
            }
        }
    }

    // Mouse wheel scroll/zoom
    let scroll = ui.input(|i| i.raw_scroll_delta);
    if scroll.x != 0.0 {
        state.scroll_ms = (state.scroll_ms - scroll.x / state.px_per_ms).max(0.0);
    }
    if scroll.y != 0.0 && ui.input(|i| i.modifiers.ctrl) {
        let factor = if scroll.y > 0.0 { 1.1 } else { 0.9 };
        state.px_per_ms = (state.px_per_ms * factor).clamp(0.01, 5.0);
    }

    // Update selection
    if let Some(mut proj) = world.get_resource_mut::<EditorProject>() {
        proj.selected_layer = new_selection;
    }

    world.insert_resource(state);
}


fn collect_tracks_from_layers(layers: &[bevy_alight_motion::schema::AmLayer]) -> Vec<TrackInfo> {
    use bevy_alight_motion::schema::AmLayer;
    layers
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
                egui::Stroke::new(1.0, RULER_TICK_COLOR),
            );
            let secs = ms / 1000.0;
            painter.text(
                egui::pos2(x + 2.0, ruler_rect.min.y + 3.0),
                egui::Align2::LEFT_TOP,
                format!("{secs:.1}s"),
                egui::FontId::proportional(10.0),
                RULER_TEXT_COLOR,
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
