mod board;

use board::BoardPlugin;

use bevy::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_startup_system(setup)
        .add_plugin(BoardPlugin)
        .add_plugins(DefaultPlugins)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}
