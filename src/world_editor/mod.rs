use std::path::PathBuf;
use bevy::ecs::system::RunSystemOnce;
use egui_file_dialog::FileDialog;

use bevy::prelude::{info, Resource, Reflect, ReflectResource, ResMut, Transform, World};
use bevy::utils::HashMap;
use bevy_editor_pls::editor_window::{EditorWindow, EditorWindowContext};
use bevy_editor_pls::egui::Ui;
use bevy_save::{Backend, Error, FileIO, JSONFormat, Pipeline, Snapshot, SnapshotBuilder, WorldSaveableExt};
use bevy::app::{App, Plugin};
use bevy_editor_pls::{egui, AddEditorWindow};
use bevy_editor_pls::editor::Editor;

impl Default for WorldEditorPlugin {
    fn default() -> Self {
        WorldEditorPlugin {
            zone_root: None,
        }
    }
}
pub struct WorldEditorPlugin {
    pub zone_root: Option<PathBuf>,
}

impl Plugin for WorldEditorPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ZoneLoader::default())
            .insert_resource(ZoneLoaderEditorState {
                boot_zone_root: self.zone_root.clone(),
                ..Default::default()
            })
            .register_type::<ZoneLoader>()
            .register_type::<ZoneLoaderEditorState>()
            .register_type::<TileData>()
            .register_type::<ZoneManifest>()
            .register_type::<TransformZoneManifest>()
            .register_type::<WorldManifest>()
            .add_editor_window::<WorldEditorWindow>();
    }
}

#[derive(Resource, Reflect, Default, Clone)]
#[reflect(Resource)]
pub struct TileData {
    // has to work for asset loader
    tile_data_path: String,
    transform: Transform,
    dynamic_map: HashMap<String, String>,
}

#[derive(Resource, Reflect, Default, Clone)]
#[reflect(Resource)]
pub struct ZoneManifest {
    size_grid_x: u32,
    size_grid_y: u32,
    // layers
    size_grid_z: u32,
    // asset paths
    tileset: Vec<TileData>,
    // [x,y,z] values are tileset indices
    grid_values: Vec<u32>,
    // path to a dynamic scene
    dynamic_scene_path: Option<String>,
}

#[derive(Resource, Reflect, Default, Clone)]
#[reflect(Resource)]
pub struct TransformZoneManifest {
    // the parent of every entity in the manifest has this transform applied to it
    // this is external to the manifest so that the overworld can be re-arranged
    transform: Transform,
    zone_manifest: ZoneManifest,
}


#[derive(Resource, Reflect, Clone)]
#[reflect(Resource)]
pub struct WorldManifest {
    pub zones_with_transforms: Vec<TransformZoneManifest>,
}


impl Default for WorldManifest {
    fn default() -> Self {
        WorldManifest {
            zones_with_transforms: vec![],
        }
    }
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct ZoneLoader {
    pub manifest: WorldManifest,
    pub dirty: bool,
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct ZoneLoaderEditorState {
    pub boot_zone_root: Option<PathBuf>,
    pub manifest: WorldManifest,
    pub focused_zone_index:  usize,
}

#[derive(Default)]
pub struct WorldEditorWindowState {
    file_dialog: FileDialog,
    selected_file: Option<PathBuf>,
}

pub struct WorldEditorWindow;

pub struct ZoneLoaderPipeline {
    pub(crate) file: String,
}

impl Pipeline for ZoneLoaderPipeline {
    type Backend = FileIO;
    type Format = JSONFormat;
    type Key<'a> = &'a str;

    fn key(&self) -> Self::Key<'_> {
        &self.file
    }

    fn capture(builder: SnapshotBuilder) -> Snapshot {
        builder
            .extract_resource::<ZoneLoader>()
            .build()
    }

    fn apply(world: &mut World, snapshot: &Snapshot) -> Result<(), Error> {
        world.remove_resource::<ZoneLoader>();
        snapshot
            .applier(world)
            .apply()
    }
}

impl EditorWindow for WorldEditorWindow {
    type State = WorldEditorWindowState;
    const NAME: &'static str = "Zone editor window";

    fn ui(world: &mut World, mut cx: EditorWindowContext, ui: &mut Ui) {
        let state = cx.state_mut::<WorldEditorWindow>().unwrap();
        if ui.button("Select Zone Root Folder").clicked() {
            state.file_dialog.select_directory();
        }
        if let Some(folder) = &state.selected_file {
            // assert!(folder.is_dir(), "expected folder found file aborting");
            ui.label(format!("Selected folder {:?}", folder));
            let folder_exists = folder.exists();
            let manifest_path = folder.join("manifest");
            let manifest_path_str = manifest_path.to_str().unwrap().to_string();

            let manifest_exists = folder.join("manifest.json").exists();
            if folder_exists && manifest_exists {
                let mut zone_loader_editor_state = world.resource_mut::<ZoneLoaderEditorState>();
                ui.label("Focused zone index");
                ui.add(egui::Slider::new(&mut zone_loader_editor_state.focused_zone_index, 0usize..=100));

                if ui.button("load zone (from disk to mem)").clicked() {
                    world.load(ZoneLoaderPipeline { file: manifest_path_str.clone() }).expect("failed to load zone collection");
                    world.run_system_once(|mut zone_loader: ResMut<ZoneLoader>| {
                        info!("marking zone loader as dirty");
                        zone_loader.dirty = true;
                    });
                }
                if ui.button("add zone (in-mem)").clicked() {
                    world.run_system_once(|mut zone_loader: ResMut<ZoneLoader>| {
                        zone_loader.manifest.zones_with_transforms.push(TransformZoneManifest::default());
                        zone_loader.dirty = true;
                    });
                }
            }
            if folder_exists {
                if ui.button("create or overwrite active root manifest (disk)").clicked() {
                    info!("saving with file {}", manifest_path_str.clone());
                    world.save(ZoneLoaderPipeline { file: manifest_path_str }).expect("failed to save zone collection");
                }
                if ui.button("copy editor to root manifest (in-mem)").clicked() {
                    world.run_system_once(|mut zone_loader: ResMut<ZoneLoader>, zone_loader_editor_state: ResMut<ZoneLoaderEditorState>| {
                        zone_loader.manifest = zone_loader_editor_state.manifest.clone();
                    });
                }
            }
        }

        state.file_dialog.update(ui.ctx());

        if let Some(path) = state.file_dialog.take_selected() {
            state.selected_file = Some(path.to_path_buf());
        }
    }

    fn app_setup(app: &mut App) {
        let zone_loader_editor_state = app.world().resource::<ZoneLoaderEditorState>();
        let selected_file_path = zone_loader_editor_state.boot_zone_root.clone();
        let mut editor = app.world_mut().get_resource_mut::<Editor>().unwrap();
        let state = editor.window_state_mut::<Self>().unwrap();
        state.selected_file = selected_file_path;
    }
}

mod test {
    
    use crate::world_editor::{TileData, TransformZoneManifest, WorldManifest, ZoneLoader, ZoneManifest};
    use bevy::prelude::App;
    use bevy::MinimalPlugins;
    use bevy_save::SavePlugins;

    #[test]
    fn test_save_run_over_run() {
        let mut app = minimal_app();
        let world = app.world_mut();
        world.save(ZoneLoaderPipeline { file: "test_saves/reversible/manifest".to_string() }).expect("failed to save");

        let mut app2 = minimal_app();
        let world2 = app2.world_mut();
        world2.load(ZoneLoaderPipeline { file: "test_saves/reversible/manifest".to_string() }).expect("failed to load");
        let entities = world2.entities();
        assert_eq!(entities.total_count(), 0);
        world2.run_system_once(|q: Res<ZoneLoader>| {
            println!("Found zones with transforms {}", q.manifest.zones_with_transforms.len());
        });
    }

    #[test]
    fn test_save_stability() {
        let mut app2 = minimal_app();
        let world2 = app2.world_mut();
        world2.load(ZoneLoaderPipeline { file: "test_saves/basic/manifest".to_string() }).expect("failed to load");
        let entities = world2.entities();
        assert_eq!(entities.total_count(), 0);
        world2.run_system_once(|q: Res<ZoneLoader>| {
            println!("Found zones with transforms {}", q.manifest.zones_with_transforms.len());
        });
    }

    fn minimal_app() -> App {
        let mut app = App::new();
        app.add_plugins((
                            MinimalPlugins,
                            SavePlugins,
                        ),
        )
            .insert_resource(ZoneLoader::default())
            .register_type::<TileData>()
            .register_type::<ZoneManifest>()
            .register_type::<TransformZoneManifest>()
            .register_type::<ZoneLoader>()
            .register_type::<WorldManifest>()
        ;
        app
    }
}
