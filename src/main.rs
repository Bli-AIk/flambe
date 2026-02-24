//! Flambé editor application entry point.

use bevy::prelude::*;
use bevy_alight_motion::prelude::*;
use bevy_inspector_egui::bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use flambe::io::file_loader::{handle_open_file, sync_project_loaded};
use flambe::sync::sync_playback_to_editor;
use flambe::ui::fonts::configure_egui_fonts;
use flambe::ui::layer_panel::layer_panel_system;
use flambe::ui::menu_bar::{OpenFileRequest, SaveFileRequest, menu_bar_system};
use flambe::ui::property_panel::property_panel_system;
use flambe::ui::timeline::{TimelineState, timeline_ui_system};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Flambé — Alight Motion Editor".into(),
                resolution: (1600u32, 900u32).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(AmProjectResolution::FitWindow)
        .add_plugins(AlightMotionPlugin)
        .add_plugins(EguiPlugin::default())
        .init_resource::<TimelineState>()
        .add_message::<OpenFileRequest>()
        .add_message::<SaveFileRequest>()
        .add_systems(Startup, setup_camera)
        .add_systems(
            EguiPrimaryContextPass,
            (
                configure_egui_fonts,
                menu_bar_system,
                layer_panel_system,
                property_panel_system,
                timeline_ui_system,
            ),
        )
        .add_systems(
            Update,
            (
                handle_open_file,
                sync_project_loaded,
                sync_playback_to_editor,
            ),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
