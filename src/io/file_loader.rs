//! File loading system — handles OpenFileRequest events.
//!
//! Copies .amproj files into the Bevy assets directory, then loads them
//! through the standard AlightMotionPlugin pipeline for full rendering.
//! Temporary copies are tracked and cleaned up when the editor exits.

use bevy::prelude::*;
use bevy_alight_motion::loader::AmProject;
use bevy_alight_motion::prelude::*;
use bevy_alight_motion::scene::AmProjectRoot;
use std::io::Read;
use std::path::PathBuf;

use crate::editor::EditorProject;
use crate::ui::menu_bar::OpenFileRequest;

/// Tracks temporary asset files copied into the assets directory.
/// Files are deleted when this resource is dropped (normal app exit).
#[derive(Resource, Default)]
pub struct TempAssets {
    pub paths: Vec<PathBuf>,
}

impl Drop for TempAssets {
    fn drop(&mut self) {
        for path in &self.paths {
            if path.exists() {
                match std::fs::remove_file(path) {
                    Err(e) => eprintln!("Failed to remove temp asset {:?}: {e}", path),
                    Ok(()) => eprintln!("Cleaned up temp asset {:?}", path),
                }
            }
        }
        // Try to remove the flambe_projects directory if empty.
        if let Some(parent) = self.paths.first().and_then(|p| p.parent()) {
            let _ = std::fs::remove_dir(parent);
        }
    }
}

/// System that handles file open requests:
/// copies the .amproj into assets/, loads it via the asset pipeline.
pub fn handle_open_file(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut events: MessageReader<OpenFileRequest>,
    project_roots: Query<Entity, With<AmProjectRoot>>,
    mut temp_assets: ResMut<TempAssets>,
) {
    for event in events.read() {
        // Despawn any existing AM project entities
        for entity in project_roots.iter() {
            commands.entity(entity).despawn();
        }

        let src = &event.path;
        let filename = src
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Copy file into the crate's assets/flambe_projects/ so Bevy can load it.
        // In dev builds Bevy resolves assets relative to CARGO_MANIFEST_DIR.
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let dest_dir = PathBuf::from(manifest_dir).join("assets/flambe_projects");
        if let Err(e) = std::fs::create_dir_all(&dest_dir) {
            error!("Failed to create assets dir: {e}");
            continue;
        }
        let dest = dest_dir.join(&filename);
        if let Err(e) = std::fs::copy(src, &dest) {
            error!("Failed to copy {:?} → {:?}: {e}", src, dest);
            continue;
        }
        info!("Copied project to {:?}", dest);
        temp_assets.paths.push(dest);

        // Load via asset pipeline (AlightMotionPlugin handles spawning)
        let asset_path = format!("flambe_projects/{}", filename);
        load_am_project(&mut commands, &asset_server, &asset_path);

        // Also extract the scene for editor metadata
        match extract_scene(src) {
            Ok(scene) => {
                info!(
                    "Loaded project: {} ({} layers)",
                    scene.title,
                    scene.layers.len()
                );
                commands.insert_resource(EditorProject::from_file(scene, src.clone()));
            }
            Err(e) => {
                error!("Failed to parse scene from {:?}: {e}", src);
            }
        }
    }
}

/// Sync editor state when the AM project finishes loading.
pub fn sync_project_loaded(
    projects: Res<Assets<AmProject>>,
    roots: Query<&AmProjectRoot, Changed<AmProjectRoot>>,
    mut playback: ResMut<AmPlayback>,
    project: Option<Res<EditorProject>>,
) {
    for root in roots.iter() {
        if root.spawned
            && let Some(am) = projects.get(&root.handle)
        {
            playback.total_time_ms = am.scene.total_time as f32;
            playback.playing = false; // Start paused in editor
            if let Some(ref _proj) = project {
                info!("AM project entities spawned, ready for editing");
            }
        }
    }
}

/// Extract just the AmScene from a .amproj file for editor metadata.
fn extract_scene(
    path: &std::path::Path,
) -> Result<bevy_alight_motion::schema::AmScene, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let mut xml_content = None;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        if name.ends_with(".xml") || name == "project.json" {
            let mut content = String::new();
            entry.read_to_string(&mut content)?;
            xml_content = Some(content);
            break;
        }
    }

    let xml = xml_content.ok_or("No XML file found in .amproj")?;
    let scene = quick_xml::de::from_str(&xml)?;
    Ok(scene)
}
