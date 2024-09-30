use std::sync::{Arc, Mutex};
use crate::world_editor::{TileData, TransformZoneManifest, WorldManifest, WorldEditorWindow, ZoneManifest};
use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::pbr::DirectionalLightShadowMap;
use bevy::prelude::{App, Camera, Camera3dBundle, Commands, DefaultPlugins, Startup, Transform, Vec3};
use bevy_editor_pls::{AddEditorWindow, EditorPlugin};
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_save::SavePlugins;

mod world_editor;


fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins,
                      #[cfg(feature = "dev")]
                      EditorPlugin::new(),
                      SavePlugins,
                      DefaultPickingPlugins,
                      #[cfg(feature = "dev")]
                      FrameTimeDiagnosticsPlugin::default(), EntityCountDiagnosticsPlugin::default(),
                     ),
        )
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_systems(Startup, setup_ui)
        .register_type::<TileData>()
        .register_type::<ZoneManifest>()
        .register_type::<TransformZoneManifest>()
        .register_type::<WorldManifest>()
        .add_editor_window::<WorldEditorWindow>();

    app.run();
}

fn setup_ui(mut commands: Commands) {
    let camera_position = Vec3::new(0., 3. * 7.0f32, 0.75 * 70.0f32);
    commands.spawn(Camera3dBundle {
        camera: Camera {
            hdr: true,
            ..Default::default()
        },
        transform: Transform::from_translation(camera_position).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}