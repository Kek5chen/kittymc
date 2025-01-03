use kittymc_macros::Packet;
use crate::packets::client::play::Location;
use crate::packets::packet_serialization::{write_location, SerializablePacket};
use crate::packets::wrap_packet;

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct SpawnPositionPacket {
    position: Location
}

impl Default for SpawnPositionPacket {
    fn default() -> Self {
        SpawnPositionPacket {
            position: Location::zeros(),
        }
    }
}

impl SerializablePacket for SpawnPositionPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_location(&mut packet, &self.position);

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        0x46
    }
}
