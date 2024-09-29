use std::ffi::CString;
use std::fmt::Debug;
use std::path::PathBuf;

use egui_file_dialog::FileDialog;

use bevy::prelude::{DynamicScene, Reflect, Resource, Transform, Vec2, World, ReflectResource, info};
use bevy::reflect::DynamicStruct;
use bevy::utils::HashMap;
use bevy_editor_pls::editor_window::{EditorWindow, EditorWindowContext};
use bevy_editor_pls::egui::Ui;
use bevy_save::{Backend, Error, FileIO, JSONFormat, Pipeline, Snapshot, SnapshotBuilder, WorldSaveableExt};


#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct TileData {
    // has to work for asset loader
    relative_path: String,
    transform: Transform,
    dynamic_map: HashMap<String, String>,
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct ZoneManifest {
    size_grid_x: u32,
    size_grid_y: u32,
    size_grid_z: u32,
    // asset paths
    tileset: Vec<TileData>,
    // [x,y,z] values are tileset indices
    grid_values: Vec<Vec<Vec<u32>>>,
    // path to a dynamic scene
    zone_dynamic_root_path: String,
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct TransformZoneManifest {
    // the parent of every entity in the manifest has this transform applied to it
    transform: Transform,
    zone_manifest: ZoneManifest,
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct ZoneCollectionManifest {
    start_zone: u32,
    zones_with_transforms: Vec<TransformZoneManifest>,
}

impl ZoneCollectionManifest {
    fn new() -> Self {
        ZoneCollectionManifest {
            start_zone: 0,
            zones_with_transforms: vec![],
        }
    }
}


#[derive(Default)]
pub struct ZoneEditorWindowState {
    file_dialog: FileDialog,
    selected_file: Option<PathBuf>,
}


pub struct ZoneEditorWindow;

struct ZoneCollectionPipeline {
    file: String,
}

impl Pipeline for ZoneCollectionPipeline {
    type Backend = FileIO;
    type Format = JSONFormat;
    type Key<'a> = &'a str;

    fn key(&self) -> Self::Key<'_> {
        &self.file
    }

    fn capture(builder: SnapshotBuilder) -> Snapshot {
        builder.extract_resource::<ZoneCollectionManifest>()
            .build()
    }

}

impl EditorWindow for ZoneEditorWindow {
    type State = ZoneEditorWindowState;
    const NAME: &'static str = "Zone editor window";

    fn ui(world: &mut World, mut cx: EditorWindowContext, ui: &mut Ui) {
        let state = cx.state_mut::<ZoneEditorWindow>().unwrap();
        if ui.button("Select Zone Root Folder").clicked() {
            state.file_dialog.select_directory();
        }
        if let Some(folder) = &state.selected_file {
            assert!(folder.is_dir(), "expected folder found file aborting");
            ui.label(format!("Selected folder {:?}", folder));
            let folder_exists = folder.exists();
            let file = folder.join("manifest");
            let manifest_exists = folder.join("manifest.json").exists();
            if folder_exists && manifest_exists {
                if ui.button("load manifest").clicked() {
                    info!("lol todo");
                }
            }
            if folder_exists {
                if ui.button("create or overwrite root manifest").clicked() {
                    world.remove_resource::<ZoneCollectionManifest>();
                    world.insert_resource(ZoneCollectionManifest::new());
                    let file = file.to_str().unwrap().to_string();
                    info!("saving with file {}", file);
                    world.save(ZoneCollectionPipeline { file }).expect("failed to save zone collection");
                }
            }
        }

        state.file_dialog.update(ui.ctx());

        if let Some(path) = state.file_dialog.take_selected() {
            state.selected_file = Some(path.to_path_buf());
        }
    }
}