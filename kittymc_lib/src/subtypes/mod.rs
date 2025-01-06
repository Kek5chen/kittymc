use crate::packets::packet_serialization::write_length_prefixed_string;
use nalgebra::{Vector2, Vector3};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::ops::{Add, AddAssign};
use typed_builder::TypedBuilder;

pub mod state;

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Color {
    Reset,
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
            Color::Reset => "reset",
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
            Color::Reset => "§r",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, TypedBuilder)]
pub struct ClickEvent {
    #[serde(skip_serializing_if = "String::is_empty", default)]
    #[builder(default, setter(into))]
    pub open_url: String,

    #[serde(skip_serializing_if = "String::is_empty", default)]
    #[builder(default, setter(into))]
    pub run_command: String,

    #[serde(skip_serializing_if = "String::is_empty", default)]
    #[builder(default, setter(into))]
    pub suggest_command: String,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(setter(strip_option), default)]
    pub change_page: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, TypedBuilder)]
pub struct HoverEvent {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(setter(into, strip_option), default)]
    pub show_text: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(setter(strip_option), default)]
    pub show_item: Option<()>, // TODO: NBT

    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(setter(strip_option), default)]
    pub show_entity: Option<()>, // TODO: NBT
}

fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, TypedBuilder, Default)]
pub struct BaseComponent {
    #[serde(skip_serializing_if = "is_false", default)]
    #[builder(default)]
    pub bold: bool,

    #[serde(skip_serializing_if = "is_false", default)]
    #[builder(default)]
    pub italic: bool,

    #[serde(skip_serializing_if = "is_false", default)]
    #[builder(default)]
    pub underlined: bool,

    #[serde(skip_serializing_if = "is_false", default)]
    #[builder(default)]
    pub strikethrough: bool,

    #[serde(skip_serializing_if = "is_false", default)]
    #[builder(default)]
    pub obfuscated: bool,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(setter(strip_option), default)]
    pub color: Option<Color>,

    #[serde(skip_serializing_if = "String::is_empty", default)]
    #[builder(default)]
    pub insertion: String,

    #[serde(skip_serializing_if = "Option::is_none", flatten, default)]
    #[builder(setter(strip_option), default)]
    pub click_event: Option<ClickEvent>,

    #[serde(skip_serializing_if = "Option::is_none", flatten, default)]
    #[builder(setter(strip_option), default)]
    pub hover_event: Option<HoverEvent>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[builder(default)]
    pub extra: Vec<Component>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, TypedBuilder)]
pub struct TextComponent {
    #[builder(setter(into), default)]
    pub text: String,
    #[serde(flatten, default)]
    #[builder(default)]
    pub options: BaseComponent,
}

impl TextComponent {
    pub fn write(&self, buffer: &mut Vec<u8>) {
        write_length_prefixed_string(
            buffer,
            &serde_json::to_string(self).unwrap_or_else(|_| "INVALID".to_string()),
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, TypedBuilder)]
pub struct TranslationComponent {
    pub translate: String,
    pub with: Vec<Component>,
}

impl TranslationComponent {
    pub fn write(&self, buffer: &mut Vec<u8>) {
        write_length_prefixed_string(
            buffer,
            &serde_json::to_string(self).unwrap_or_else(|_| "INVALID".to_string()),
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum Component {
    Text(TextComponent),
    Translation(TranslationComponent),
    KeyBind,  // TODO
    Score,    // TODO
    Selector, // TODO
}

const CHAT_TRANSLATION_TAG: &'static str = "chat.type.text";

impl Component {
    pub fn write(&self, buffer: &mut Vec<u8>) {
        write_length_prefixed_string(
            buffer,
            &serde_json::to_string(&self).unwrap_or_else(|_| "INVALID".to_string()),
        );
    }

    pub fn default_join(player: &str) -> Self {
        Component::Text(
            TextComponent::builder()
                .text(player.to_string())
                .options(
                    BaseComponent::builder()
                        .bold(true)
                        .italic(true)
                        .color(Color::DarkPurple)
                        .extra(vec![Component::Text(
                            TextComponent::builder()
                                .text(" joined the game".to_string())
                                .options(BaseComponent::builder().color(Color::Gray).build())
                                .build(),
                        )])
                        .build(),
                )
                .build(),
        )
    }

    pub fn default_chat(player: &str, message: &str) -> Self {
        Component::Translation(
            TranslationComponent::builder()
                .translate(CHAT_TRANSLATION_TAG.to_string())
                .with(vec![
                    Component::Text(
                        TextComponent::builder()
                            .text(player)
                            .options(
                                BaseComponent::builder()
                                    .bold(true)
                                    .italic(true)
                                    .color(Color::DarkPurple)
                                    .build(),
                            )
                            .build(),
                    ),
                    Component::Text(
                        TextComponent::builder()
                            .text(message)
                            .options(BaseComponent::builder().color(Color::Gray).build())
                            .build(),
                    ),
                ])
                .build(),
        )
    }
}

const CHUNK_WIDTH: isize = 16;
const HALF_CHUNK_WIDTH: f32 = (CHUNK_WIDTH / 2) as f32;

pub struct ChunkPositionIterator {
    positions: Vec<ChunkPosition>,
    idx: usize,
}

impl ChunkPositionIterator {
    /// Return the min (lowest corner) of the chunk in block coords
    fn chunk_box_min(cpos: &ChunkPosition) -> Location {
        Location::new(
            cpos.block_x() as f32,
            cpos.block_y() as f32,
            cpos.block_z() as f32,
        )
    }

    /// Return the max corner of the chunk in block coords
    fn chunk_box_max(cpos: &ChunkPosition) -> Location {
        Self::chunk_box_min(cpos)
            + Location::new(CHUNK_WIDTH as f32, CHUNK_WIDTH as f32, CHUNK_WIDTH as f32)
    }

    /// 3D bounding box distance
    fn distance_to_chunk_box_3d(cpos: &ChunkPosition, point: &Location) -> f32 {
        let bmin = Self::chunk_box_min(cpos);
        let bmax = Self::chunk_box_max(cpos);

        // For each dimension, compute how far `point` is outside bmin..bmax.
        let dx = if point.x < bmin.x {
            bmin.x - point.x
        } else if point.x > bmax.x {
            point.x - bmax.x
        } else {
            0.0
        };

        let dy = if point.y < bmin.y {
            bmin.y - point.y
        } else if point.y > bmax.y {
            point.y - bmax.y
        } else {
            0.0
        };

        let dz = if point.z < bmin.z {
            bmin.z - point.z
        } else if point.z > bmax.z {
            point.z - bmax.z
        } else {
            0.0
        };

        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// 2D bounding box distance (XZ only), ignoring Y dimension
    fn distance_to_chunk_box_xz(cpos: &ChunkPosition, point: &Location) -> f32 {
        let bmin = Self::chunk_box_min(cpos);
        let bmax = Self::chunk_box_max(cpos);

        // 2D in XZ
        let dx = if point.x < bmin.x {
            bmin.x - point.x
        } else if point.x > bmax.x {
            point.x - bmax.x
        } else {
            0.0
        };

        let dz = if point.z < bmin.z {
            bmin.z - point.z
        } else if point.z > bmax.z {
            point.z - bmax.z
        } else {
            0.0
        };

        // In the XZ‑only approach, we just treat Y as “unchanged”.
        (dx * dx + dz * dz).sqrt()
    }

    pub fn new(center: &Location, radius: f32, xz_only: bool) -> Self {
        let center_chunk = ChunkPosition::from(center.clone());

        let radius_in_chunks = (radius / CHUNK_WIDTH as f32).ceil() as isize;

        let mut positions = Vec::new();

        let (min_y, max_y) = if xz_only {
            (0, 0)
        } else {
            (
                center_chunk.chunk_y() - radius_in_chunks,
                center_chunk.chunk_y() + radius_in_chunks,
            )
        };

        for cy in min_y..=max_y {
            for cz in (center_chunk.chunk_z() - radius_in_chunks)
                ..=(center_chunk.chunk_z() + radius_in_chunks)
            {
                for cx in (center_chunk.chunk_x() - radius_in_chunks)
                    ..=(center_chunk.chunk_x() + radius_in_chunks)
                {
                    let cpos = ChunkPosition::new(cx, cy, cz);

                    let dist = if xz_only {
                        Self::distance_to_chunk_box_xz(&cpos, center)
                    } else {
                        Self::distance_to_chunk_box_3d(&cpos, center)
                    };

                    if dist <= radius {
                        positions.push(cpos);
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
        ChunkPositionIterator::new(&Location::new(0., 5., 0.), CHUNK_WIDTH as f32 / 2., true)
            .collect();

    assert_eq!(chunks.len(), 4);
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct ChunkPosition(Vector3<isize>);

impl ChunkPosition {
    /// Constructs a new `ChunkPosition` from raw chunk indices.
    /// E.g. `ChunkPosition::new(2, 0, 4)` means chunk #2 on X, #0 on Y, #4 on Z.
    pub fn new(cx: isize, cy: isize, cz: isize) -> Self {
        ChunkPosition(Vector3::new(cx, cy, cz))
    }

    pub fn chunk_x(&self) -> isize {
        self.0.x
    }

    pub fn chunk_y(&self) -> isize {
        self.0.y
    }

    pub fn chunk_z(&self) -> isize {
        self.0.z
    }

    /// Returns the lowest corner of this chunk *in block/world coordinates*
    pub fn block_x(&self) -> isize {
        self.0.x * CHUNK_WIDTH
    }

    pub fn block_y(&self) -> isize {
        self.0.y * CHUNK_WIDTH
    }

    pub fn block_z(&self) -> isize {
        self.0.z * CHUNK_WIDTH
    }

    pub fn block_location(&self) -> Location {
        Location::new(
            self.block_x() as f32,
            self.block_y() as f32,
            self.block_z() as f32,
        )
    }

    pub fn center(&self) -> Location {
        Location::new(
            self.block_x() as f32 + HALF_CHUNK_WIDTH,
            self.block_y() as f32 + HALF_CHUNK_WIDTH,
            self.block_z() as f32 + HALF_CHUNK_WIDTH,
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
        self.0.x += rhs;
        self.0.y += rhs;
        self.0.z += rhs;
        self
    }
}

impl AddAssign<isize> for ChunkPosition {
    fn add_assign(&mut self, rhs: isize) {
        self.0.x += rhs;
        self.0.y += rhs;
        self.0.z += rhs;
    }
}

impl From<Location> for ChunkPosition {
    fn from(loc: Location) -> ChunkPosition {
        Self::from(&loc)
    }
}

impl From<&Location> for ChunkPosition {
    fn from(loc: &Location) -> ChunkPosition {
        let bx = loc.x.floor() as isize;
        let by = loc.y.floor() as isize;
        let bz = loc.z.floor() as isize;

        let cx = bx / CHUNK_WIDTH;
        let cy = by / CHUNK_WIDTH;
        let cz = bz / CHUNK_WIDTH;

        ChunkPosition(Vector3::new(cx, cy, cz))
    }
}

pub type Location = Vector3<f32>;
pub type Location2 = Vector3<f64>;
pub type Direction = Vector2<f32>;
