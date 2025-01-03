use crate::packets::client::play::{Difficulty, Dimension, GameMode, LevelType};
use crate::packets::packet_serialization::{write_bool, write_i32, write_i8, write_length_prefixed_string, write_u8, SerializablePacket};
use crate::packets::wrap_packet;

#[derive(Clone, Debug)]
pub struct JoinGamePacket {
    entity_id: i32,
    gamemode: GameMode,
    dimension: Dimension,
    difficulty: Difficulty,
    max_players: u8,
    level_type: LevelType,
    reduced_debug_info: bool
}

impl Default for JoinGamePacket {
    fn default() -> Self {
        Self {
            entity_id: 0,
            gamemode: GameMode::Survival,
            dimension: Dimension::Nether,
            difficulty: Difficulty::Peaceful,
            max_players: 0,
            level_type: LevelType::Default,
            reduced_debug_info: false,
        }
    }
}

impl SerializablePacket for JoinGamePacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_i32(&mut packet, self.entity_id);
        write_u8(&mut packet, self.gamemode as u8);
        write_i8(&mut packet, self.dimension as i8);
        write_u8(&mut packet, self.difficulty as u8);
        write_u8(&mut packet, 5); // TODO: Actual max players
        write_length_prefixed_string(&mut packet, self.level_type.as_str());
        write_bool(&mut packet, self.reduced_debug_info);

        wrap_packet(&mut packet, 1);

        packet
    }
}