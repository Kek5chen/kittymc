use nalgebra::{Vector2, Vector3};

pub mod keep_alive_00;
pub mod join_game_01;
pub mod spawn_position_05;
pub mod chat_message_02;
pub mod plugin_message_3f;
pub mod server_difficulty_41;
pub mod player_abilities_39;
pub mod player_position_and_look;
pub mod map_chunk_bulk_26;
pub mod window_items_14;
pub mod chunk_data_20;

#[derive(Copy, Clone, Debug)]
pub enum GameMode {
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    Spectator = 3,
    SurvivalH = 0 | 8,
    CreativeH = 1 | 8,
    AdventureH = 2 | 8,
    SpectatorH = 3 | 8,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Dimension {
    Nether = -1,
    Overworld = 0,
    End = 1,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Difficulty {
    Peaceful = 0,
    Easy = 1,
    Normal = 2,
    Hard = 3,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LevelType {
    Default,
    Flat,
    LargeBiomes,
    Amplified,
    Default11
}

impl LevelType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LevelType::Default => "default",
            LevelType::Flat => "flat",
            LevelType::LargeBiomes => "largeBiomes",
            LevelType::Amplified => "amplified",
            LevelType::Default11 => "default_1_1",
        }
    }
}

pub type Location = Vector3<f32>;
pub type Location2 = Vector3<f64>;
pub type Direction = Vector2<f32>;
