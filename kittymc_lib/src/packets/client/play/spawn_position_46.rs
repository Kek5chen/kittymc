use crate::packets::packet_serialization::{write_block_location, SerializablePacket};
use crate::packets::wrap_packet;
use crate::subtypes::Location;
use kittymc_macros::Packet;

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct SpawnPositionPacket {
    position: Location,
}

impl Default for SpawnPositionPacket {
    fn default() -> Self {
        SpawnPositionPacket {
            position: Location::new(0., 4., 0.),
        }
    }
}

impl SerializablePacket for SpawnPositionPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_block_location(&mut packet, &self.position);

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        0x46
    }
}
