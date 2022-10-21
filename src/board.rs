use bevy::utils::hashbrown::HashMap;

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
enum Piece {
    Blue,
    Red,
}

#[derive(Clone, Copy)]
pub struct Tile;

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
            .zip(std::iter::repeat(Tile))
            .collect();
        Self { tiles }
    }
}
