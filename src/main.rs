use core::iter::Iterator;
use std::env;
use std::path::{Path, PathBuf};
use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::pbr::DirectionalLightShadowMap;
use bevy::prelude::{info, warn, App, Camera, Camera3dBundle, Commands, DefaultPlugins, Startup, Transform, Vec3};
use bevy_editor_pls::EditorPlugin;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_save::SavePlugins;
use world_editor::WorldEditorPlugin;

mod world_editor;

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins,
                     #[cfg(feature = "dev")]
                     EditorPlugin::new(),
                     SavePlugins,
                     DefaultPickingPlugins,
                     #[cfg(feature = "dev")]
                     FrameTimeDiagnosticsPlugin::default(),
                     #[cfg(feature = "dev")]
                     EntityCountDiagnosticsPlugin::default(),

                    ))
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_systems(Startup, setup_ui);

    info!("Starting luawow");
    let args: Vec<String> = env::args().collect();
    let zone_root = "--zone-root";
    if let Some(zone_root_arg_index) = args.iter().position(|e| e == zone_root) {
        info!("Found zone root arg index {}", zone_root_arg_index);
        assert!(args.len() > zone_root_arg_index + 1, "Expected argument after --zone-root");
        let path = Path::new(&args[zone_root_arg_index + 1]);
        assert!(path.exists(), "Expected folder to exist but \"{}\" doesn't exist", path.to_str().unwrap());
        app.add_plugins(WorldEditorPlugin { zone_root: Some(PathBuf::from(path)) });
    } else {
        warn!("No --zone-root selected, starting editor only");
        app.add_plugins(WorldEditorPlugin::default());
    }

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