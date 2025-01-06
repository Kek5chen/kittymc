pub mod chat_message_0f;
pub mod chunk_data_20;
pub mod entity_status_1b;
pub mod join_game_23;
pub mod keep_alive_1f;
pub mod map_chunk_bulk_26;
pub mod player_abilities_2c;
pub mod player_list_item_2e;
pub mod player_position_and_look_2f;
pub mod server_difficulty_0d;
pub mod server_held_item_change_3a;
pub mod server_plugin_message_18;
pub mod spawn_player_05;
pub mod spawn_position_46;
pub mod time_update_47;
pub mod unload_chunk_1d;
pub mod unlock_recipes_31;
pub mod window_items_14;

pub use chat_message_0f::ClientChatMessagePacket;
pub use chunk_data_20::ChunkDataPacket;
pub use entity_status_1b::EntityStatusPacket;
pub use join_game_23::JoinGamePacket;
pub use keep_alive_1f::ServerKeepAlivePacket;
pub use map_chunk_bulk_26::MapChunkBulkPacket;
pub use player_abilities_2c::PlayerAbilitiesPacket;
pub use player_list_item_2e::PlayerListItemPacket;
pub use player_position_and_look_2f::ServerPlayerPositionAndLookPacket;
pub use server_difficulty_0d::ServerDifficultyPacket;
pub use server_held_item_change_3a::ServerHeldItemChangePacket;
pub use server_plugin_message_18::ServerPluginMessagePacket;
pub use spawn_player_05::SpawnPlayerPacket;
pub use spawn_position_46::SpawnPositionPacket;
pub use time_update_47::TimeUpdatePacket;
pub use unload_chunk_1d::UnloadChunkPacket;
pub use unlock_recipes_31::UnlockRecipesPacket;
pub use window_items_14::WindowItemsPacket;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Dimension {
    Nether = -1,
    Overworld = 0,
    End = 1,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Difficulty {
    Peaceful = 0,
    Easy = 1,
    Normal = 2,
    Hard = 3,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LevelType {
    Default,
    Flat,
    LargeBiomes,
    Amplified,
    Default11,
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
