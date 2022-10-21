use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        let board = Board::default();

        app.insert_resource(board)
            .add_event::<RenderBoardEvent>()
            .add_startup_system(board_startup)
            .add_system(Board::render);
    }
}

fn board_startup(mut commands: Commands, mut render_writer: EventWriter<RenderBoardEvent>) {
    let board_entity = BoardEntity(commands.spawn_bundle(SpatialBundle::default()).id());
    commands.insert_resource(board_entity);

    render_writer.send(RenderBoardEvent);
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct Coord(pub isize, pub isize);

impl Coord {
    pub fn parity(self) -> isize {
        ((self.0 ^ self.1) & 1) * 2 - 1
    }

    pub fn texture_coords(self, scale: f32) -> (f32, f32) {
        (
            self.0 as f32 * scale / 2.0,
            (self.1 as f32 * 3f32.sqrt() / 2.0) * scale,
        )
    }

    pub fn world_coords(self, scale: f32) -> (f32, f32) {
        let mut coords = self.texture_coords(scale);
        coords.1 -= self.parity() as f32 * scale * 3f32.sqrt() / 12.0;
        coords
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    Blue,
    Red,
}

#[derive(Clone, Copy, Default)]
pub struct Tile {
    pub piece: Option<Piece>,
}

pub struct Board {
    pub tiles: HashMap<Coord, Tile>,
}

impl Default for Board {
    fn default() -> Self {
        let tiles = (-3..3)
            .flat_map(|row: isize| {
                let bound = 5 - row.abs().min((row + 1).abs());
                (-bound..=bound).map(move |col| Coord(col, row))
            })
            .map(|pos| {
                (
                    pos,
                    Tile {
                        piece: match pos.1 {
                            -3 => Some(Piece::Blue),
                            2 => Some(Piece::Red),
                            _ => None,
                        },
                    },
                )
            })
            .collect();
        Self { tiles }
    }
}

pub struct BoardEntity(pub Entity);
pub struct RenderBoardEvent;

impl Board {
    fn render(
        mut commands: Commands,
        board_entity: Res<BoardEntity>,
        board: Res<Board>,
        asset_server: Res<AssetServer>,
        mut reader: EventReader<RenderBoardEvent>,
    ) {
        if reader.iter().next().is_none() {
            return;
        }

        let triangle = asset_server.load("triangle.png");
        let circle = asset_server.load("circle.png");

        let mut children = Vec::new();

        let scale = 96.0;

        for (&coord, &tile) in board.tiles.iter() {
            let pos = coord.texture_coords(scale);
            let mut transform =
                Transform::from_xyz(pos.0, pos.1, 1.0).with_scale(Vec3::from_array([0.4; 3]));
            let color = if coord.parity() == 1 {
                Color::GRAY
            } else {
                transform.scale.y = -transform.scale.y;
                transform.translation.z -= 0.1;
                Color::WHITE
            };
            let child = commands.spawn_bundle(SpriteBundle {
                texture: triangle.clone(),
                sprite: Sprite {
                    color,
                    ..Default::default()
                },
                transform,
                ..Default::default()
            });
            children.push(child.id());
            if let Some(piece) = tile.piece {
                let color = match piece {
                    Piece::Blue => Color::BLUE,
                    Piece::Red => Color::RED,
                };
                let pos = coord.world_coords(scale);
                let transform =
                    Transform::from_xyz(pos.0, pos.1, 1.5).with_scale(Vec3::from_array([0.14; 3]));
                let piece_child = commands.spawn_bundle(SpriteBundle {
                    texture: circle.clone(),
                    sprite: Sprite {
                        color,
                        ..Default::default()
                    },
                    transform,
                    ..Default::default()
                });
                children.push(piece_child.id());
            }
        }

        let mut board_entity = commands.entity(board_entity.0);
        board_entity.despawn_descendants();
        board_entity.push_children(&children);
    }
}
