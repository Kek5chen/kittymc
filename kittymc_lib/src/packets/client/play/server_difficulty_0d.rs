use crate::packets::client::play::Difficulty;
use crate::packets::packet_serialization::{write_u8, SerializablePacket};
use crate::packets::wrap_packet;
use kittymc_macros::Packet;

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct ServerDifficultyPacket {
    difficulty: Difficulty,
}

impl Default for ServerDifficultyPacket {
    fn default() -> Self {
        ServerDifficultyPacket {
            difficulty: Difficulty::Normal,
        }
    }
}

impl SerializablePacket for ServerDifficultyPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_u8(&mut packet, self.difficulty as u8);

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        0x0D
    }
}
