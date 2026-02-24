//! Configure egui fonts with CJK support and apply theme.

use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiContexts;

use super::theme;

/// System to configure egui fonts with CJK support and apply theme (runs once).
pub fn configure_egui_fonts(mut contexts: EguiContexts, mut done: Local<bool>) {
    if *done {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // Apply rerun-inspired dark theme
    theme::apply_theme(ctx);

    let font_data = load_cjk_font();
    if let Some(data) = font_data {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "noto_sans_cjk".to_owned(),
            egui::FontData::from_owned(data).into(),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .push("noto_sans_cjk".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("noto_sans_cjk".to_owned());
        ctx.set_fonts(fonts);
        info!("Configured egui CJK font + rerun theme");
    } else {
        warn!("No CJK font found — non-ASCII characters may not render");
    }

    *done = true;
}

/// Try to load a CJK font from known system paths.
fn load_cjk_font() -> Option<Vec<u8>> {
    let candidates = [
        // Linux (Noto CJK)
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc",
        // Linux (Source Han Sans)
        "/usr/share/fonts/adobe-source-han-sans/SourceHanSansCN-Regular.otf",
        // macOS
        "/System/Library/Fonts/PingFang.ttc",
        "/Library/Fonts/Arial Unicode.ttf",
        // Windows
        "C:\\Windows\\Fonts\\msyh.ttc",
        "C:\\Windows\\Fonts\\simsun.ttc",
    ];

    for path in &candidates {
        if let Ok(data) = std::fs::read(path) {
            info!("Loaded CJK font from {}", path);
            return Some(data);
        }
    }
    None
}
