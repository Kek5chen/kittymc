use nalgebra::{Vector2, Vector3};
use crate::error::KittyMCError;
use crate::packets::packet_serialization::{write_bool, write_direction, write_location, write_location2, SerializablePacket};
use crate::packets::{wrap_packet, Packet};
use crate::packets::client::play::Location2;

#[derive(PartialEq, Debug, Clone)]
pub struct PlayerPositionAndLookPacket {
    location: Location2, // Feet
    direction: Vector2<f32>,
    on_ground: bool,
}

impl Default for PlayerPositionAndLookPacket {
    fn default() -> Self {
        PlayerPositionAndLookPacket {
            location: Default::default(),
            direction: Default::default(),
            on_ground: false,
        }
    }
}

impl SerializablePacket for PlayerPositionAndLookPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_location2(&mut packet, &self.location);
        write_direction(&mut packet, &self.direction);
        write_bool(&mut packet, self.on_ground);

        wrap_packet(&mut packet, 0x06);

        packet
    }
}