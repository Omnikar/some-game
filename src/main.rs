mod board;

use board::Board;

use bevy::prelude::*;

fn main() {
    let board = Board::default();

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(board)
        .add_startup_system(setup)
        .add_startup_system(create_board)
        .add_plugins(DefaultPlugins)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

fn create_board(mut commands: Commands, board: Res<Board>, asset_server: Res<AssetServer>) {
    let triangle = asset_server.load("triangle.png");
    for &coord in board.tiles.keys() {
        let pos = coord.texture_coords(96.0);
        let mut transform =
            Transform::from_xyz(pos.0, pos.1, 1.0).with_scale(Vec3::from_array([0.4; 3]));
        let color = if coord.parity() == 1 {
            Color::GRAY
        } else {
            transform.scale.y = -transform.scale.y;
            transform.translation.z -= 0.1;
            Color::WHITE
        };
        commands.spawn_bundle(SpriteBundle {
            texture: triangle.clone(),
            sprite: Sprite {
                color,
                ..Default::default()
            },
            transform,
            ..Default::default()
        });
    }
}
