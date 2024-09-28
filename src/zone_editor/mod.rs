use std::fmt::Debug;
use std::path::PathBuf;

use egui_file_dialog::FileDialog;

use bevy::prelude::World;
use bevy_editor_pls::editor_window::{EditorWindow, EditorWindowContext};
use bevy_editor_pls::egui::Ui;


#[derive(Default)]
pub struct ZoneEditorWindowState {
    file_dialog: FileDialog,
    selected_file: Option<PathBuf>,
}

pub struct ZoneEditorWindow;

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
        }

        state.file_dialog.update(ui.ctx());

        if let Some(path) = state.file_dialog.take_selected() {
            state.selected_file = Some(path.to_path_buf());
        }
    }
}