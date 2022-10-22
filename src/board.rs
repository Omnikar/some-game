use bevy::prelude::*;

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UpdateBoardEvent>()
            .add_event::<ShearEvent>()
            .add_startup_system(create_board)
            .add_startup_system(board_startup)
            .add_system(update)
            .add_system(shear);
    }
}

fn board_startup(mut commands: Commands, mut render_writer: EventWriter<UpdateBoardEvent>) {
    let board_entity = BoardEntity(commands.spawn_bundle(SpatialBundle::default()).id());
    commands.insert_resource(board_entity);

    render_writer.send(UpdateBoardEvent);
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Component)]
pub struct Coord(pub isize, pub isize);

impl std::ops::Add for Coord {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Coord(self.0 + rhs.0, self.1 + rhs.1)
    }
}
impl std::ops::Sub for Coord {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Coord(self.0 - rhs.0, self.1 - rhs.1)
    }
}
impl std::ops::AddAssign for Coord {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
impl std::ops::SubAssign for Coord {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

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

#[derive(Clone, Copy, Default, Component)]
pub struct Tile {
    pub piece: Option<Piece>,
}

fn create_board(mut commands: Commands) {
    (-3..3)
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
        .for_each(|(coord, tile)| {
            commands.spawn().insert(coord).insert(tile);
        });
}

pub struct BoardEntity(pub Entity);

pub struct UpdateBoardEvent;
pub struct ShearEvent(pub Coord, pub Coord);

fn update(
    mut commands: Commands,
    board_entity: Res<BoardEntity>,
    mut tiles_q: Query<(&mut Coord, &Tile)>,
    asset_server: Res<AssetServer>,
    mut reader: EventReader<UpdateBoardEvent>,
) {
    if reader.iter().next().is_none() {
        return;
    }

    let bounds = tiles_q.iter().fold(
        ((isize::MAX, isize::MIN), (isize::MAX, isize::MIN)),
        |bounds, (next, _)| {
            (
                (bounds.0 .0.min(next.0), bounds.0 .1.max(next.0)),
                (bounds.1 .0.min(next.1), bounds.1 .1.max(next.1)),
            )
        },
    );
    // Divide by 4 then multiply by 2 to ensure the result is even.
    let center = Coord(
        (bounds.0 .0 + bounds.0 .1) / 4 * 2,
        (bounds.1 .0 + bounds.1 .1) / 4 * 2,
    );
    for (mut coord, _) in tiles_q.iter_mut() {
        *coord -= center;
    }

    let triangle = asset_server.load("triangle.png");
    let circle = asset_server.load("circle.png");

    let mut children = Vec::new();

    let scale = 96.0;

    for (coord, tile) in tiles_q.iter() {
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

fn shear(
    mut tiles_q: Query<&mut Coord>,
    mut reader: EventReader<ShearEvent>,
    mut render_writer: EventWriter<UpdateBoardEvent>,
) {
    let ShearEvent(origin, end) = match reader.iter().next() {
        Some(event) => event,
        None => return,
    };

    if origin.parity() != end.parity() {
        return;
    }

    let parity = origin.parity();
    if origin.1 == end.1 {
        let shear_coords = tiles_q
            .iter_mut()
            .filter(|coord| coord.1 * parity >= origin.1 * parity);
        let shear_distance = end.0 - origin.0;
        for mut coord in shear_coords {
            coord.0 += shear_distance;
        }
    } else if end.0 - origin.0 == end.1 - origin.1 {
        let parity = origin.parity();
        let shear_coords = tiles_q
            .iter_mut()
            .filter(|coord| (coord.0 - origin.0) * parity >= (coord.1 - origin.1) * parity);
        let shear_delta = *end - *origin;
        for mut coord in shear_coords {
            *coord += shear_delta;
        }
    } else if origin.0 - end.0 == end.1 - origin.1 {
        let shear_coords = tiles_q
            .iter_mut()
            .filter(|coord| (origin.0 - coord.0) * parity >= (coord.1 - origin.1) * parity);
        let shear_delta = *end - *origin;
        for mut coord in shear_coords {
            *coord += shear_delta;
        }
    } else {
        return;
    }

    render_writer.send(UpdateBoardEvent);
}
