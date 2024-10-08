use std::path::PathBuf;
use bevy::ecs::system::RunSystemOnce;
use egui_file_dialog::FileDialog;

use bevy::prelude::{info, Resource, Reflect, ReflectResource, ResMut, Transform, World, Res, Query, Component, ReflectComponent, Commands};
use bevy::utils::HashMap;
use bevy_editor_pls::editor_window::{EditorWindow, EditorWindowContext};
use bevy_editor_pls::egui::Ui;
use bevy_save::{Error, FileIO, JSONFormat, Pipeline, Snapshot, SnapshotBuilder, WorldSaveableExt};
use bevy::app::{App, Plugin};
use bevy::pbr::PbrBundle;
use bevy_editor_pls::{egui, AddEditorWindow};
use bevy_editor_pls::editor::Editor;
use bevy_mod_picking::PickableBundle;

impl Default for ZoneSaveLoaderPlugin {
    fn default() -> Self {
        ZoneSaveLoaderPlugin {
            zone_root: None,
        }
    }
}
pub struct ZoneSaveLoaderPlugin {
    pub zone_root: Option<PathBuf>,
}

impl Plugin for ZoneSaveLoaderPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ZoneLoaderEditorState {
                boot_zone_root: self.zone_root.clone(),
                ..Default::default()
            })
            .insert_resource(ZoneLoader::default())
            .register_type::<ZoneLoader>()
            .register_type::<DebugGrid>()
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

#[derive(Default, Component, Reflect, Clone)]
#[reflect(Component)]
pub struct DebugGrid {
    size_grid_x: u32,
    size_grid_y: u32,
    size_grid_z: u32,
    zone_id: u32,
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

// Zone loaders are the main data type for the campaign
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
    pub focused_zone_index: u32,
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
    const NAME: &'static str = "Zone loader editor state";

    fn ui(mut world: &mut World, mut cx: EditorWindowContext, ui: &mut Ui) {
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
                let mut zone_loader_editor_state = world.resource_mut::<ZoneLoaderEditorState>();
                ui.label("Focused zone index (mem)");
                let actual_zone_count = zone_loader_editor_state.manifest.zones_with_transforms.len() as u32;
                let editor_max_zone_count = actual_zone_count.max(1);
                let focused_zone_index = zone_loader_editor_state.focused_zone_index.clone();
                ui.add(egui::Slider::new(&mut zone_loader_editor_state.focused_zone_index, 0u32..=(editor_max_zone_count - 1)));
                if focused_zone_index < actual_zone_count {
                    let mut active_transform_zone_manifest: &mut TransformZoneManifest = &mut zone_loader_editor_state.manifest.zones_with_transforms[focused_zone_index as usize];
                    ui.label("grid size x");
                    let changed_x = ui.add(egui::Slider::new(&mut active_transform_zone_manifest.zone_manifest.size_grid_x, 0..=1000));
                    ui.label("grid size y");
                    let changed_y = ui.add(egui::Slider::new(&mut active_transform_zone_manifest.zone_manifest.size_grid_y, 0..=1000));
                    ui.label("grid size z");
                    let changed_z = ui.add(egui::Slider::new(&mut active_transform_zone_manifest.zone_manifest.size_grid_z, 0..=1000));

                    let grid_x = active_transform_zone_manifest.zone_manifest.size_grid_x;
                    let grid_y = active_transform_zone_manifest.zone_manifest.size_grid_y;
                    let grid_z = active_transform_zone_manifest.zone_manifest.size_grid_z;
                    let focused_zone_index = zone_loader_editor_state.focused_zone_index;
                    if changed_x.changed() || changed_y.changed() || changed_z.changed() {
                        world.run_system_once(move |mut commands: ResMut<Commands>, mut debug_grid: Query<&mut DebugGrid>|
                            for grid in debug_grid.iter_mut() {
                                {
                                    if grid.zone_id == focused_zone_index {
                                        commands.spawn((
                                            PbrBundle::default(),
                                            PickableBundle::default(),
                                        ));
                                        for x in 0..grid_x {
                                            for y in 0..grid_y {
                                                for z in 0..grid_z {}
                                            }
                                        }
                                    }
                                };
                            });
                    };
                }
                if ui.button("removed focused zone").clicked() {
                    world.run_system_once(move |mut zone_loader_editor_state: ResMut<ZoneLoaderEditorState>| {
                        let focused_zone_index = zone_loader_editor_state.focused_zone_index;
                        if zone_loader_editor_state.focused_zone_index < zone_loader_editor_state.manifest.zones_with_transforms.len() as u32 {
                            zone_loader_editor_state.manifest.zones_with_transforms.remove(focused_zone_index as usize);
                        }
                    });
                }
                if ui.button("append zone to loader editor (mem)").clicked() {
                    world.run_system_once(|mut zone_loader_editor_state: ResMut<ZoneLoaderEditorState>| {
                        info!("Adding zone to editor");
                        zone_loader_editor_state.manifest.zones_with_transforms.push(TransformZoneManifest::default());
                    });
                }
                if ui.button("read loader from disk (mem)").clicked() {
                    info!("loading with file {}", manifest_path_str.clone());
                    world.load(ZoneLoaderPipeline { file: manifest_path_str.clone() }).expect("failed to load zone collection");
                }
            }
            if folder_exists {
                if ui.button("copy loader editor to loader (mem)").clicked() {
                    world.run_system_once(|mut zone_loader: ResMut<ZoneLoader>, zone_loader_editor_state: Res<ZoneLoaderEditorState>| {
                        zone_loader.manifest = zone_loader_editor_state.manifest.clone();
                        zone_loader.dirty = false;
                    });
                }
                if ui.button("copy loader to loader editor (mem)").clicked() {
                    world.run_system_once(|mut zone_loader: ResMut<ZoneLoader>, mut zone_loader_editor_state: ResMut<ZoneLoaderEditorState>| {
                        zone_loader_editor_state.manifest = zone_loader.manifest.clone();
                        zone_loader.dirty = false;
                    });
                }
                if ui.button("write loader to persistence (disk)").clicked() {
                    info!("saving with file {}", manifest_path_str.clone());
                    world.save(ZoneLoaderPipeline { file: manifest_path_str.clone() }).expect("failed to save zone collection");
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
        state.selected_file = selected_file_path.clone();

        if let Some(manifest_path) = selected_file_path {
            let file = manifest_path.as_path().join("manifest").to_str().unwrap().to_string();
            info!("loading with file {}", file);
            app.world_mut().load(ZoneLoaderPipeline { file }).expect("failed to load zone collection");
            app.world_mut().run_system_once(|mut zone_loader_editor_state: ResMut<ZoneLoaderEditorState>, zone_loader: Res<ZoneLoader>| {
                zone_loader_editor_state.manifest = zone_loader.manifest.clone();
            });
        }
    }
}

fn draw_debug(world: &mut &mut World, x: u32, y: u32, z: u32) {
    todo!()
}

mod test {
    use bevy::ecs::system::RunSystemOnce;
    use crate::save_loader::{TileData, TransformZoneManifest, WorldManifest, ZoneLoader, ZoneLoaderPipeline, ZoneManifest};
    use bevy::prelude::{App, Res};
    use bevy::MinimalPlugins;
    use bevy_save::{SavePlugins, WorldSaveableExt};

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
