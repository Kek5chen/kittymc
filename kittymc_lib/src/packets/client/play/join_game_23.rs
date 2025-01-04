use kittymc_macros::Packet;
use crate::packets::client::play::{Difficulty, Dimension, GameMode, LevelType};
use crate::packets::packet_serialization::{write_bool, write_i32, write_length_prefixed_string, write_u8, SerializablePacket};
use crate::packets::wrap_packet;

#[derive(Clone, Debug, Packet)]
#[allow(dead_code)]
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
            entity_id: 129,
            gamemode: GameMode::Creative,
            dimension: Dimension::Overworld,
            difficulty: Difficulty::Peaceful,
            max_players: 1,
            level_type: LevelType::Flat,
            reduced_debug_info: false,
        }
    }
}

impl SerializablePacket for JoinGamePacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_i32(&mut packet, self.entity_id);
        write_u8(&mut packet, self.gamemode as u8);
        write_i32(&mut packet, self.dimension as i32);
        write_u8(&mut packet, self.difficulty as u8);
        write_u8(&mut packet, 69); // TODO: Actual max players
        write_length_prefixed_string(&mut packet, self.level_type.as_str());
        write_bool(&mut packet, self.reduced_debug_info);

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        0x23
    }
}