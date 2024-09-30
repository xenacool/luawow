use std::path::PathBuf;
use bevy::asset::AssetServer;
use bevy::ecs::system::RunSystemOnce;
use egui_file_dialog::FileDialog;

use bevy::prelude::{info, Commands, Component, DirectAssetAccessExt, DynamicScene, DynamicSceneBundle, Query, Reflect, ReflectComponent, ReflectResource, Res, ResMut, Resource, Transform, World};
use bevy::utils::HashMap;
use bevy_editor_pls::editor_window::{EditorWindow, EditorWindowContext};
use bevy_editor_pls::egui::Ui;
use bevy_save::{Backend, Error, FileIO, JSONFormat, Pipeline, Snapshot, SnapshotBuilder, WorldSaveableExt};
use bevy::app::{App, Plugin};
use bevy_editor_pls::AddEditorWindow;




#[derive(Default)]
pub struct WorldEditorPlugin;

impl Plugin for WorldEditorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ZoneLoader::default())
            .register_type::<ZoneLoader>()
            .register_type::<TileData>()
            .register_type::<ZoneManifest>()
            .register_type::<TransformZoneManifest>()
            .register_type::<WorldManifest>()
            .add_editor_window::<WorldEditorWindow>();
    }
}

#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct TileData {
    // has to work for asset loader
    tile_data_path: String,
    transform: Transform,
    dynamic_map: HashMap<String, String>,
}

#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
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

#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct TransformZoneManifest {
    // the parent of every entity in the manifest has this transform applied to it
    // this is external to the manifest so that the overworld can be re-arranged
    transform: Transform,
    zone_manifest: ZoneManifest,
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct ZoneLoader {
    pub manifest: WorldManifest,
    pub dirty: bool,
}

#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct WorldManifest {
    pub start_zones: Vec<u32>,
    pub zones_with_transforms: Vec<TransformZoneManifest>,
}

impl Default for WorldManifest {
    fn default() -> Self {
        WorldManifest {
            start_zones: vec![],
            zones_with_transforms: vec![],
        }
    }
}

#[derive(Default)]
pub struct WorldEditorWindowState {
    file_dialog: FileDialog,
    selected_file: Option<PathBuf>,
}

pub struct WorldEditorWindow;

struct WorldManifestPipeline {
    file: String,
}

impl Pipeline for WorldManifestPipeline {
    type Backend = FileIO;
    type Format = JSONFormat;
    type Key<'a> = &'a str;

    fn key(&self) -> Self::Key<'_> {
        &self.file
    }

    fn capture(builder: SnapshotBuilder) -> Snapshot {
        builder
            .extract_entities_matching(|entity_ref|
                entity_ref.contains::<WorldManifest>()
            )
            .build()
    }

    fn apply(world: &mut World, snapshot: &Snapshot) -> Result<(), Error> {
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
            assert!(folder.is_dir(), "expected folder found file aborting");
            ui.label(format!("Selected folder {:?}", folder));
            let folder_exists = folder.exists();
            let manifest_path = folder.join("manifest");
            let manifest_path_str = manifest_path.to_str().unwrap().to_string();

            let manifest_exists = folder.join("manifest.json").exists();
            if folder_exists && manifest_exists {
                if ui.button("load zone").clicked() {
                    world.load(WorldManifestPipeline { file: manifest_path_str.clone() }).expect("failed to load zone collection");
                    world.run_system_once(|mut zone_loader: ResMut<ZoneLoader>| {
                        info!("marking zone loader as dirty");
                        zone_loader.dirty = true;
                    });
                }
            }
            if folder_exists {
                if ui.button("create or overwrite root manifest").clicked() {
                    info!("saving with file {}", manifest_path_str.clone());
                    world.save(WorldManifestPipeline { file: manifest_path_str }).expect("failed to save zone collection");
                }
            }
        }

        state.file_dialog.update(ui.ctx());

        if let Some(path) = state.file_dialog.take_selected() {
            state.selected_file = Some(path.to_path_buf());
        }
    }
}

mod test {
    use bevy::ecs::system::RunSystemOnce;
    use crate::world_editor::{TileData, TransformZoneManifest, WorldManifest, WorldManifestPipeline, ZoneManifest};
    use bevy::prelude::{App, Query};
    use bevy::MinimalPlugins;
    use bevy_save::{SavePlugins, WorldSaveableExt};

    #[test]
    fn test_save_run_over_run() {
        let mut app = minimal_app();
        let world = app.world_mut();
        let world_manifest = WorldManifest {
            start_zones: vec![],
            zones_with_transforms: vec![],
        };
        world.spawn(world_manifest);
        world.save(WorldManifestPipeline { file: "test_saves/reversible/manifest".to_string() }).expect("failed to save");

        let mut app2 = minimal_app();
        let world2 = app2.world_mut();
        world2.load(WorldManifestPipeline { file: "test_saves/reversible/manifest".to_string() }).expect("failed to load");
        let entities = world2.entities();
        assert_eq!(entities.total_count(), 1);
        world2.run_system_once(|q: Query<&WorldManifest>| {
            assert_eq!(1, q.iter().len(), "expected 1");
            for e in &q {
                println!("Found zones with transforms {}", e.zones_with_transforms.len());
                println!("Found start zones {}", e.start_zones.len());
            }
        });
    }

    #[test]
    fn test_save_stability() {
        let mut app2 = minimal_app();
        let world2 = app2.world_mut();
        world2.load(WorldManifestPipeline { file: "test_saves/basic/manifest".to_string() }).expect("failed to load");
        let entities = world2.entities();
        assert_eq!(entities.total_count(), 1);
        world2.run_system_once(|q: Query<&WorldManifest>| {
            assert_eq!(1, q.iter().len(), "expected 1");
            for e in &q {
                println!("Found zones with transforms {}", e.zones_with_transforms.len());
                println!("Found start zones {}", e.start_zones.len());
            }
        });
    }

    fn minimal_app() -> App {
        let mut app = App::new();
        app.add_plugins((
                            MinimalPlugins,
                            SavePlugins,
                        ),
        )
            .register_type::<TileData>()
            .register_type::<ZoneManifest>()
            .register_type::<TransformZoneManifest>()
            .register_type::<WorldManifest>()
        ;
        app
    }
}
