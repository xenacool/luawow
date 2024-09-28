use bevy::prelude::{App, DefaultPlugins, Reflect};
use bevy_editor_pls::{AddEditorWindow, EditorPlugin};
use bevy_save::SavePlugins;
use crate::zone_editor::ZoneEditorWindow;

mod zone_editor;

#[derive(Reflect)]
pub struct ZoneCollection;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EditorPlugin::new(), SavePlugins))
        .register_type::<ZoneCollection>()
        .add_editor_window::<ZoneEditorWindow>()
        .run();
}
