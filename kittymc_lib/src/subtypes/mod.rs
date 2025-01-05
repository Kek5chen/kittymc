use crate::packets::packet_serialization::write_length_prefixed_string;
use derive_builder::Builder;
use log::debug;
use nalgebra::{Vector2, Vector3};
use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign};

pub mod state;

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Color {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
}

impl Color {
    pub fn as_str(&self) -> &'static str {
        match self {
            Color::Black => "black",
            Color::DarkBlue => "dark_blue",
            Color::DarkGreen => "dark_green",
            Color::DarkAqua => "dark_aqua",
            Color::DarkRed => "dark_red",
            Color::DarkPurple => "dark_purple",
            Color::Gold => "gold",
            Color::Gray => "gray",
            Color::DarkGray => "dark_gray",
            Color::Blue => "blue",
            Color::Green => "green",
            Color::Aqua => "aqua",
            Color::Red => "red",
            Color::LightPurple => "light_purple",
            Color::Yellow => "yellow",
            Color::White => "white",
        }
    }

    pub fn as_color_code(&self) -> &'static str {
        match self {
            Color::Black => "§0",
            Color::DarkBlue => "§1",
            Color::DarkGreen => "§2",
            Color::DarkAqua => "§3",
            Color::DarkRed => "§4",
            Color::DarkPurple => "§5",
            Color::Gold => "§6",
            Color::Gray => "§7",
            Color::DarkGray => "§8",
            Color::Blue => "§9",
            Color::Green => "§a",
            Color::Aqua => "§b",
            Color::Red => "§c",
            Color::LightPurple => "§d",
            Color::Yellow => "§e",
            Color::White => "§f",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Builder)]
pub struct Chat {
    text: String,
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    bold: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option), default)]
    italic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option), default)]
    underlined: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option), default)]
    strikethrough: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option), default)]
    obfuscated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option), default)]
    color: Option<Color>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[builder(setter(custom), default)]
    extra: Vec<Chat>,
}

impl Chat {
    pub fn write(&self, buffer: &mut Vec<u8>) {
        write_length_prefixed_string(
            buffer,
            &serde_json::to_string(&self).unwrap_or_else(|_| "INVALID".to_string()),
        );
    }
}

const CHUNK_WIDTH: isize = 16;
const HALF_CHUNK_WIDTH: f32 = (CHUNK_WIDTH / 2) as f32;

pub struct ChunkPositionIterator {
    positions: Vec<ChunkPosition>,
    idx: usize,
}

impl ChunkPositionIterator {
    pub fn new(center: &Location, radius: f32, xz_only: bool) -> Self {
        // Compute bounding box in integer steVjjjjjjjjjjps of CHUNK_WIDTH
        let min_x = (center.x - radius).floor() as i32;
        let max_x = (center.x + radius).ceil() as i32;
        let min_z = (center.z - radius).floor() as i32;
        let max_z = (center.z + radius).ceil() as i32;
        let step = CHUNK_WIDTH as usize;
        let mut center = center.clone();

        // For `xz_only`, we treat y as just the center's integer y.
        // If you prefer a different approach, adjust accordingly.
        let (min_y, max_y) = if xz_only {
            let cy = center.y.floor() as i32;
            center.y = (cy - (cy % CHUNK_WIDTH as i32)) as f32;
            (cy, cy) // no vertical iteration
        } else {
            let min_y = (center.y - radius).floor() as i32;
            let max_y = (center.y + radius).ceil() as i32;
            (min_y, max_y)
        };

        let mut positions = Vec::new();

        debug!("Center: {center}");

        // Step in multiples of CHUNK_WIDTH from min..=max
        for y in (min_y..=max_y).step_by(step) {
            for z in (min_z..=max_z).step_by(step) {
                for x in (min_x..=max_x).step_by(step) {
                    let loc = ChunkPosition::from(Location::new(x as f32, y as f32, z as f32));
                    // Only keep points within the radius
                    debug!(
                        "Magnitude of center {:?} of {:?} is {}",
                        loc,
                        loc.center(),
                        (loc.center() - center).magnitude()
                    );
                    if (loc.center() - center).magnitude() <= radius {
                        positions.push(ChunkPosition::from(loc));
                    }
                }
            }
        }

        ChunkPositionIterator { positions, idx: 0 }
    }
}

impl Iterator for ChunkPositionIterator {
    type Item = ChunkPosition;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.positions.len() {
            let out = self.positions[self.idx].clone();
            self.idx += 1;
            Some(out)
        } else {
            None
        }
    }
}

#[test]
fn chunk_position_iterator_test() {
    let chunks: Vec<_> =
        ChunkPositionIterator::new(&Location::new(0., 0., 0.), (CHUNK_WIDTH + 3) as f32, true)
            .collect();

    assert_eq!(chunks.len(), 4);
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct ChunkPosition(Vector3<isize>);

impl ChunkPosition {
    // TODO: Make this return chunk position, not block position
    pub fn x(&self) -> isize {
        self.0.x
    }

    pub fn y(&self) -> isize {
        self.0.z
    }

    pub fn z(&self) -> isize {
        self.0.z
    }

    pub fn to_location(&self) -> Location {
        Location::new(self.0.x as f32, self.0.y as f32, self.0.z as f32)
    }

    pub fn center(&self) -> Location {
        Location::new(
            self.0.x as f32 + HALF_CHUNK_WIDTH,
            self.0.y as f32 + HALF_CHUNK_WIDTH,
            self.0.z as f32 + HALF_CHUNK_WIDTH,
        )
    }

    pub fn iter_sphere_in_range(location: &Location, radius: f32) -> ChunkPositionIterator {
        ChunkPositionIterator::new(location, radius, false)
    }

    pub fn iter_xz_circle_in_range(location: &Location, radius: f32) -> ChunkPositionIterator {
        ChunkPositionIterator::new(location, radius, true)
    }
}

impl Add for ChunkPosition {
    type Output = ChunkPosition;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.0 += rhs.0;

        self
    }
}

impl Add<isize> for ChunkPosition {
    type Output = ChunkPosition;

    fn add(mut self, rhs: isize) -> Self::Output {
        self.0.x += rhs * CHUNK_WIDTH;
        self.0.y += rhs * CHUNK_WIDTH;
        self.0.z += rhs * CHUNK_WIDTH;

        self
    }
}

impl AddAssign<isize> for ChunkPosition {
    fn add_assign(&mut self, rhs: isize) {
        self.0.x += rhs * CHUNK_WIDTH;
        self.0.y += rhs * CHUNK_WIDTH;
        self.0.z += rhs * CHUNK_WIDTH;
    }
}

impl From<Location> for ChunkPosition {
    fn from(loc: Location) -> ChunkPosition {
        let mut x = loc.x.floor() as isize;
        let mut y = loc.y.floor() as isize;
        let mut z = loc.z.floor() as isize;

        x = x - (x % CHUNK_WIDTH);
        y = y - (y % CHUNK_WIDTH);
        z = z - (z % CHUNK_WIDTH);

        ChunkPosition(Vector3::new(x, y, z))
    }
}

impl From<&Location> for ChunkPosition {
    fn from(loc: &Location) -> ChunkPosition {
        let mut x = loc.x.floor() as isize;
        let mut y = loc.y.floor() as isize;
        let mut z = loc.z.floor() as isize;

        x = x - (x % CHUNK_WIDTH);
        y = y - (y % CHUNK_WIDTH);
        z = z - (z % CHUNK_WIDTH);

        ChunkPosition(Vector3::new(x, y, z))
    }
}

pub type Location = Vector3<f32>;
pub type Location2 = Vector3<f64>;
pub type Direction = Vector2<f32>;
