use kittymc_macros::Packet;
use rand::random;
use crate::packets::packet_serialization::{write_direction, write_location2, write_u8, write_varint_u32, SerializablePacket};
use crate::packets::wrap_packet;
use crate::packets::client::play::{Direction, Location2};

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct ServerPlayerPositionAndLookPacket {
    location: Location2, // Feet
    direction: Direction,
    relative_flags: u8,
    teleport_id: u32
}

impl Default for ServerPlayerPositionAndLookPacket {
    fn default() -> Self {
        ServerPlayerPositionAndLookPacket {
            location: Location2::new(0., 5., 0.),
            direction: Default::default(),
            relative_flags: 0,
            teleport_id: random()
        }
    }
}

impl SerializablePacket for ServerPlayerPositionAndLookPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_location2(&mut packet, &self.location);
        write_direction(&mut packet, &self.direction);
        write_u8(&mut packet, self.relative_flags);
        write_varint_u32(&mut packet, self.teleport_id);

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        0x2F
    }
}
