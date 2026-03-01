//! Flambé editor application entry point.

use bevy::prelude::*;
use bevy_alight_motion::prelude::*;
use bevy_workbench::console::console_log_layer;
use bevy_workbench::prelude::*;
use flambe::io::file_loader::{TempAssets, handle_open_file, sync_project_loaded};
use flambe::sync::sync_playback_to_editor;
use flambe::ui::menu_bar::{OpenFileRequest, SaveFileRequest, flambe_menu_system};
use flambe::ui::preview::{setup_preview, update_preview_resolution};
use flambe::ui::property_panel::PropertyPanel;
use flambe::ui::timeline::{TimelinePanel, TimelineState};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Flambé — Alight Motion Editor".into(),
                        resolution: (1600u32, 900u32).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::log::LogPlugin {
                    custom_layer: console_log_layer,
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(AmProjectResolution::None)
        .init_resource::<TempAssets>()
        .add_plugins(AlightMotionPlugin)
        .add_plugins(WorkbenchPlugin::default())
        .init_resource::<TimelineState>()
        .add_message::<OpenFileRequest>()
        .add_message::<SaveFileRequest>()
        .add_systems(Startup, setup_preview)
        .add_systems(
            bevy_egui::EguiPrimaryContextPass,
            flambe_menu_system.before(bevy_workbench::menu_bar::menu_bar_system),
        )
        .add_systems(
            Update,
            (
                handle_open_file,
                sync_project_loaded,
                sync_playback_to_editor,
                update_preview_resolution,
            ),
        )
        .register_panel(TimelinePanel)
        .register_panel(PropertyPanel)
        .run();
}
