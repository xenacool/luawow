// disable console on windows for release builds
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

use bevy::prelude::*;
use luawow::GamePlugin;

fn main() {
    App::new().add_plugins(GamePlugin).run();
}
