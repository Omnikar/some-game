use bevy::prelude::*;

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UpdateBoardEvent>()
            .add_event::<ActionEvent>()
            .add_event::<TextCommandEvent>()
            .add_startup_system(create_board)
            .add_startup_system(board_startup)
            .add_system(update)
            .add_system(shear)
            .add_system(text_input)
            .add_system(text_command);
    }
}

fn board_startup(mut commands: Commands, mut render_writer: EventWriter<UpdateBoardEvent>) {
    let board_entity = BoardEntity(commands.spawn_bundle(SpatialBundle::default()).id());
    commands.insert_resource(board_entity);

    render_writer.send(UpdateBoardEvent);
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Default, Component)]
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
fn update(
    mut commands: Commands,
    board_entity: Res<BoardEntity>,
    mut tiles_q: Query<(&mut Coord, &Tile)>,
    asset_server: Res<AssetServer>,
    mut reader: EventReader<UpdateBoardEvent>,
) {
    // Only check if there is a single update event or not, no need to handle multiple.
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

    let scale = 600.0
        / ((bounds.0 .1 - bounds.0 .0) as f32 / 2.0)
            .max((bounds.1 .1 - bounds.1 .0) as f32 * 2.0 / 3f32.sqrt());

    let triangle = asset_server.load("triangle.png");
    let circle = asset_server.load("circle.png");

    let mut children = Vec::new();

    for (coord, tile) in tiles_q.iter() {
        let pos = coord.texture_coords(scale);
        let mut transform =
            Transform::from_xyz(pos.0, pos.1, 1.0).with_scale(Vec3::from_array([scale / 240.0; 3]));
        let parity = coord.parity();
        transform.scale.y *= parity as f32;
        transform.translation.z += 0.1 * parity as f32;
        let color = if *coord == Coord(0, 0) {
            Color::AQUAMARINE
        } else if parity == 1 {
            Color::GRAY
        } else {
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
            let transform = Transform::from_xyz(pos.0, pos.1, 1.5)
                .with_scale(Vec3::from_array([scale / 685.714; 3]));

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

pub enum ActionEvent {
    Shear(Coord, Coord),
}

fn shear(
    mut tiles_q: Query<&mut Coord, With<Tile>>,
    mut reader: EventReader<ActionEvent>,
    mut update_writer: EventWriter<UpdateBoardEvent>,
) {
    for event in reader.iter() {
        let (origin, end) = match event {
            ActionEvent::Shear(origin, end) => (origin, end),
            _ => continue,
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

        update_writer.send(UpdateBoardEvent);
    }
}

fn text_input(
    mut reader: EventReader<ReceivedCharacter>,
    mut text: Local<String>,
    mut writer: EventWriter<TextCommandEvent>,
) {
    let mut updated = false;

    for event in reader.iter() {
        if event.char == '\x0d' {
            writer.send(TextCommandEvent(text.clone()));
            text.clear();
        } else if event.char == '\x7f' {
            text.pop();
        } else {
            text.push(event.char);
        }
        updated = true;
    }

    if updated {
        use std::io::Write;
        print!("\r{}\x1b[J", *text);
        std::io::stdout().flush().unwrap();
    }
}

struct TextCommandEvent(String);
fn text_command(mut reader: EventReader<TextCommandEvent>, mut writer: EventWriter<ActionEvent>) {
    for TextCommandEvent(command) in reader.iter() {
        if let Some(command) = command.strip_prefix("shear ") {
            let (mut origin, mut end): (Coord, Coord) = Default::default();
            let (origin_s, end_s) = command.split_once(' ').unwrap();
            for (val, s) in [(&mut origin, origin_s), (&mut end, end_s)] {
                let (p1_s, p2_s) = s.split_once(',').unwrap();
                for (val, s) in [(&mut val.0, p1_s), (&mut val.1, p2_s)] {
                    *val = s.parse::<isize>().unwrap();
                }
            }

            writer.send(ActionEvent::Shear(origin, end));
        }
    }
}
