use bevy::prelude::*;

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DisplayScale(1.0))
            .add_event::<UpdateBoardEvent>()
            .add_event::<ActionEvent>()
            .add_event::<TextCommandEvent>()
            .add_startup_system(create_board)
            .add_startup_system(board_startup)
            .add_system(update)
            .add_system(r#move)
            .add_system(shear)
            .add_system(set)
            .add_system(text_input)
            .add_system(text_command)
            .add_system(mouse_hover);
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
            self.1 as f32 * 3f32.sqrt() / 2.0 * scale,
        )
    }

    pub fn world_coords(self, scale: f32) -> (f32, f32) {
        let mut coords = self.texture_coords(scale);
        coords.1 -= self.parity() as f32 * scale * 3f32.sqrt() / 12.0;
        coords
    }

    // Not quite accurate at side corners
    fn from_world_coords(coords: (f32, f32), scale: f32) -> Self {
        Coord(
            (coords.0 * 2.0 / scale).round() as isize,
            (coords.1 / scale * 2.0 / 3f32.sqrt()).round() as isize,
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    Blue(bool),
    Red(bool),
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
                    // piece: match pos.1 {
                    //     -3 => Some(Piece::Blue(pos.0 == 0)),
                    //     2 => Some(Piece::Red(pos.0 == 0)),
                    //     _ => None,
                    // },
                    piece: if pos.parity() == 1 && (pos.1 == -2 || pos.1 == -3) {
                        Some(Piece::Blue(pos.0 == 0))
                    } else if pos.parity() == -1 && (pos.1 == 1 || pos.1 == 2) {
                        Some(Piece::Red(pos.0 == 0))
                    } else {
                        None
                    },
                },
            )
        })
        .for_each(|(coord, tile)| {
            commands.spawn().insert(coord).insert(tile);
        });
}

pub struct BoardEntity(pub Entity);
pub struct DisplayScale(f32);

pub struct UpdateBoardEvent;
fn update(
    mut commands: Commands,
    board_entity: Res<BoardEntity>,
    mut tiles_q: Query<(&mut Coord, &Tile)>,
    mut scale_res: ResMut<DisplayScale>,
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
    *scale_res = DisplayScale(scale);

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
        let color = if parity == 1 {
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
                Piece::Blue(king) => {
                    if king {
                        Color::NAVY
                    } else {
                        Color::BLUE
                    }
                }
                Piece::Red(king) => {
                    if king {
                        Color::MAROON
                    } else {
                        Color::RED
                    }
                }
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
    Move(Coord, Coord),
    Shear(Coord, Coord),
    Set(Coord, Option<Piece>),
}

fn r#move(
    mut tiles_q: Query<(&Coord, &mut Tile)>,
    mut reader: EventReader<ActionEvent>,
    mut update_writer: EventWriter<UpdateBoardEvent>,
) {
    for event in reader.iter() {
        let (origin, end) = match event {
            ActionEvent::Move(origin, end) => (origin, end),
            _ => continue,
        };

        let (mut origin_tile, mut end_tile) = (None, None);
        for (coord, tile) in tiles_q.iter_mut() {
            if *coord == *origin {
                origin_tile = Some(tile);
            } else if *coord == *end {
                end_tile = Some(tile);
            }

            if origin_tile.as_ref().and(end_tile.as_ref()).is_some() {
                break;
            }
        }

        if let (Some(mut origin_tile), Some(mut end_tile)) = (origin_tile, end_tile) {
            std::mem::swap(origin_tile.as_mut(), end_tile.as_mut());
        }

        update_writer.send(UpdateBoardEvent);
    }
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

fn set(
    mut tiles_q: Query<(&Coord, &mut Tile)>,
    mut reader: EventReader<ActionEvent>,
    mut update_writer: EventWriter<UpdateBoardEvent>,
) {
    for event in reader.iter() {
        let (coord, piece) = match event {
            ActionEvent::Set(coord, piece) => (coord, piece),
            _ => continue,
        };

        let mut tile = match tiles_q
            .iter_mut()
            .find_map(|(tile_coord, tile)| (tile_coord == coord).then_some(tile))
        {
            Some(tile) => tile,
            None => return,
        };

        tile.piece = *piece;

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
    'outer: for TextCommandEvent(command) in reader.iter() {
        if let Some(command) = command.strip_prefix("move ") {
            let (mut origin, mut end): (Coord, Coord) = Default::default();
            let (origin_s, end_s) = match command.split_once(' ') {
                Some(val) => val,
                None => continue 'outer,
            };
            for (val, s) in [(&mut origin, origin_s), (&mut end, end_s)] {
                let (p1_s, p2_s) = match s.split_once(',') {
                    Some(val) => val,
                    None => continue 'outer,
                };
                for (val, s) in [(&mut val.0, p1_s), (&mut val.1, p2_s)] {
                    *val = match s.parse::<isize>() {
                        Ok(val) => val,
                        Err(_) => continue 'outer,
                    };
                }
            }

            writer.send(ActionEvent::Move(origin, end));
        } else if let Some(command) = command.strip_prefix("shear ") {
            let (mut origin, mut end): (Coord, Coord) = Default::default();
            let (origin_s, end_s) = match command.split_once(' ') {
                Some(val) => val,
                None => continue 'outer,
            };
            for (val, s) in [(&mut origin, origin_s), (&mut end, end_s)] {
                let (p1_s, p2_s) = match s.split_once(',') {
                    Some(val) => val,
                    None => continue 'outer,
                };
                for (val, s) in [(&mut val.0, p1_s), (&mut val.1, p2_s)] {
                    *val = match s.parse::<isize>() {
                        Ok(val) => val,
                        Err(_) => continue 'outer,
                    };
                }
            }

            writer.send(ActionEvent::Shear(origin, end));
        } else if let Some(command) = command.strip_prefix("set ") {
            let mut coord = Coord::default();
            let (coord_s, piece_s) = match command.split_once(' ') {
                Some(val) => val,
                None => continue 'outer,
            };
            let (p1_s, p2_s) = match coord_s.split_once(',') {
                Some(val) => val,
                None => continue 'outer,
            };
            for (val, s) in [(&mut coord.0, p1_s), (&mut coord.1, p2_s)] {
                *val = match s.parse::<isize>() {
                    Ok(val) => val,
                    Err(_) => continue 'outer,
                };
            }
            let piece = match piece_s {
                "empty" => None,
                "blue" => Some(Piece::Blue(false)),
                "blue-special" => Some(Piece::Blue(true)),
                "red" => Some(Piece::Red(false)),
                "red-special" => Some(Piece::Red(true)),
                _ => continue 'outer,
            };

            writer.send(ActionEvent::Set(coord, piece));
        }
    }
}

fn mouse_hover(
    windows: Res<Windows>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut reader: EventReader<CursorMoved>,
    scale: Res<DisplayScale>,
) {
    // Take the last mouse move event to get the most up-to-date position.
    let event = match reader.iter().last() {
        Some(event) => event,
        None => return,
    };
    let screen_pos = event.position;

    let (camera, camera_transform) = camera_q.single();

    let window = windows.get_primary().unwrap();
    let window_size = Vec2::new(window.width() as f32, window.height() as f32);
    let ndc = screen_pos / window_size * 2.0 - Vec2::ONE;
    let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();
    let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0)).truncate();

    let board_pos = Coord::from_world_coords(world_pos.into(), scale.0);
    use std::io::Write;
    print!("\r{},{}\x1b[J", board_pos.0, board_pos.1);
    std::io::stdout().flush().unwrap();
}
