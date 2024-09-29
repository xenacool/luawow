use bevy::prelude::{App, DefaultPlugins, Reflect};
use bevy_editor_pls::{AddEditorWindow, EditorPlugin};
use bevy_save::SavePlugins;
use crate::zone_editor::{TileData, TransformZoneManifest, ZoneCollectionManifest, ZoneEditorWindow, ZoneManifest};

mod zone_editor;


fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EditorPlugin::new(), SavePlugins))
        .register_type::<TileData>()
        .register_type::<ZoneManifest>()
        .register_type::<TransformZoneManifest>()
        .register_type::<ZoneCollectionManifest>()
        .add_editor_window::<ZoneEditorWindow>()
        .run();
}
