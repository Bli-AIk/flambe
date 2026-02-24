//! Rerun-inspired dark theme for egui.
//!
//! Colors extracted from rerun's `re_ui` design tokens (dark_theme.ron + color_table.ron).

use egui::{Color32, Stroke, Vec2, epaint::Shadow};

/// Rerun gray scale (from color_table.ron).
pub mod gray {
    use egui::Color32;
    pub const S0: Color32 = Color32::from_rgb(0x00, 0x00, 0x00);
    pub const S100: Color32 = Color32::from_rgb(0x0d, 0x10, 0x11);
    pub const S150: Color32 = Color32::from_rgb(0x14, 0x18, 0x19);
    pub const S200: Color32 = Color32::from_rgb(0x1c, 0x21, 0x23);
    pub const S250: Color32 = Color32::from_rgb(0x26, 0x2b, 0x2e);
    pub const S300: Color32 = Color32::from_rgb(0x31, 0x38, 0x3b);
    pub const S325: Color32 = Color32::from_rgb(0x37, 0x3f, 0x42);
    pub const S350: Color32 = Color32::from_rgb(0x3e, 0x46, 0x4a);
    pub const S550: Color32 = Color32::from_rgb(0x7d, 0x8c, 0x92);
    pub const S700: Color32 = Color32::from_rgb(0xae, 0xc2, 0xca);
    pub const S775: Color32 = Color32::from_rgb(0xca, 0xd8, 0xde);
    pub const S1000: Color32 = Color32::from_rgb(0xff, 0xff, 0xff);
}

pub mod blue {
    use egui::Color32;
    pub const S350: Color32 = Color32::from_rgb(0x00, 0x3d, 0xa1);
    pub const S400: Color32 = Color32::from_rgb(0x00, 0x4b, 0xc2);
    pub const S500: Color32 = Color32::from_rgb(0x2a, 0x6c, 0xff);
    pub const S750: Color32 = Color32::from_rgb(0xc2, 0xcc, 0xff);
    pub const S900: Color32 = Color32::from_rgb(0xf0, 0xf2, 0xff);
}

/// Apply the rerun-inspired dark theme to an egui context.
pub fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // ── Typography ──────────────────────────────────────────────
    let font_size = 12.0;
    for text_style in [
        egui::TextStyle::Body,
        egui::TextStyle::Monospace,
        egui::TextStyle::Button,
    ] {
        if let Some(font_id) = style.text_styles.get_mut(&text_style) {
            font_id.size = font_size;
        }
    }
    if let Some(font_id) = style.text_styles.get_mut(&egui::TextStyle::Heading) {
        font_id.size = 16.0;
    }
    style.spacing.interact_size.y = 15.0;

    // ── Spacing ─────────────────────────────────────────────────
    style.visuals.button_frame = true;

    // No strokes on buttons
    style.visuals.widgets.inactive.bg_stroke = Stroke::NONE;
    style.visuals.widgets.hovered.bg_stroke = Stroke::NONE;
    style.visuals.widgets.active.bg_stroke = Stroke::NONE;
    style.visuals.widgets.open.bg_stroke = Stroke::NONE;

    // Expansion on hover/active
    style.visuals.widgets.hovered.expansion = 2.0;
    style.visuals.widgets.active.expansion = 2.0;
    style.visuals.widgets.open.expansion = 2.0;

    // Corner radii
    let window_radius = egui::CornerRadius::same(6);
    let small_radius = egui::CornerRadius::same(4);
    style.visuals.window_corner_radius = window_radius;
    style.visuals.menu_corner_radius = window_radius;
    style.visuals.widgets.noninteractive.corner_radius = small_radius;
    style.visuals.widgets.inactive.corner_radius = small_radius;
    style.visuals.widgets.hovered.corner_radius = small_radius;
    style.visuals.widgets.active.corner_radius = small_radius;
    style.visuals.widgets.open.corner_radius = small_radius;

    style.spacing.item_spacing = Vec2::new(8.0, 8.0);
    style.spacing.menu_margin = egui::Margin::same(12);
    style.spacing.menu_spacing = 1.0;
    style.visuals.clip_rect_margin = 0.0;
    style.visuals.striped = false;
    style.visuals.indent_has_left_vline = false;
    style.spacing.button_padding = Vec2::new(1.0, 0.0);
    style.spacing.indent = 14.0;
    style.spacing.combo_width = 8.0;
    style.spacing.scroll.bar_inner_margin = 2.0;
    style.spacing.scroll.bar_width = 6.0;
    style.spacing.scroll.bar_outer_margin = 2.0;
    style.spacing.tooltip_width = 600.0;
    style.visuals.image_loading_spinners = false;

    // ── Colors ───────────────────────────────────────────────────
    style.visuals.dark_mode = true;
    style.visuals.faint_bg_color = gray::S150;
    style.visuals.extreme_bg_color = gray::S0;

    style.visuals.widgets.noninteractive.weak_bg_fill = gray::S100;
    style.visuals.widgets.noninteractive.bg_fill = gray::S100;
    style.visuals.text_edit_bg_color = Some(gray::S200);

    // Inactive buttons: no background
    style.visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    style.visuals.widgets.inactive.bg_fill = gray::S300;

    // Hovered/active/open
    let hovered = gray::S325;
    style.visuals.widgets.hovered.weak_bg_fill = hovered;
    style.visuals.widgets.hovered.bg_fill = hovered;
    style.visuals.widgets.active.weak_bg_fill = hovered;
    style.visuals.widgets.active.bg_fill = hovered;
    style.visuals.widgets.open.weak_bg_fill = hovered;
    style.visuals.widgets.open.bg_fill = hovered;

    // Selection
    style.visuals.selection.bg_fill = blue::S350;
    style.visuals.selection.stroke.color = blue::S900;

    // Separator / non-interactive stroke
    style.visuals.widgets.noninteractive.bg_stroke.color = gray::S250;

    // Text colors
    let subdued = gray::S550;
    let default_text = gray::S775;
    let strong = gray::S1000;
    style.visuals.widgets.noninteractive.fg_stroke.color = subdued;
    style.visuals.widgets.inactive.fg_stroke.color = default_text;
    style.visuals.widgets.active.fg_stroke.color = strong;
    style.visuals.widgets.active.fg_stroke.width = 2.0;
    style.visuals.selection.stroke.width = 2.0;

    // Shadow
    let shadow = Shadow {
        offset: [0, 15],
        blur: 50,
        spread: 0,
        color: Color32::from_black_alpha(128),
    };
    style.visuals.popup_shadow = shadow;
    style.visuals.window_shadow = shadow;

    style.visuals.window_fill = gray::S200;
    style.visuals.window_stroke = Stroke::NONE;
    style.visuals.panel_fill = gray::S100;

    style.visuals.hyperlink_color = default_text;
    style.visuals.error_fg_color = Color32::from_rgb(0xAB, 0x01, 0x16);
    style.visuals.warn_fg_color = Color32::from_rgb(0xFF, 0x7A, 0x0C);

    ctx.set_style(style);
}

// ── Color constants for UI panels ───────────────────────────────
/// Header/ruler background
pub const HEADER_BG: Color32 = gray::S200;
/// Track row even background
pub const ROW_EVEN_BG: Color32 = gray::S100;
/// Track row odd background
pub const ROW_ODD_BG: Color32 = gray::S150;
/// Selected row background
pub const ROW_SELECTED_BG: Color32 = Color32::from_rgb(0x00, 0x30, 0x84);
/// Track bar color (unselected)
pub const BAR_COLOR: Color32 = blue::S500;
/// Track bar color (selected)
pub const BAR_SELECTED_COLOR: Color32 = blue::S750;
/// Separator line color
pub const SEPARATOR_COLOR: Color32 = gray::S250;
/// Ruler tick color
pub const RULER_TICK_COLOR: Color32 = gray::S350;
/// Ruler text color
pub const RULER_TEXT_COLOR: Color32 = gray::S550;
/// Layer name text
pub const LAYER_TEXT_COLOR: Color32 = gray::S700;
/// Header label text
pub const HEADER_TEXT_COLOR: Color32 = gray::S550;
/// Keyframe diamond color
pub const KEYFRAME_COLOR: Color32 = Color32::from_rgb(0xFF, 0xC8, 0x32);
/// Playhead line color
pub const PLAYHEAD_COLOR: Color32 = Color32::from_rgb(0xFF, 0x50, 0x50);
