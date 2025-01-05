use crate::error::KittyMCError;
use crate::packets::client::play::Location2;
use crate::packets::packet_serialization::{read_bool, read_location2, SerializablePacket};
use crate::packets::Packet;
use kittymc_macros::Packet;

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct PlayerPositionPacket {
    location: Location2, // Feet
    on_ground: bool,
}

impl SerializablePacket for PlayerPositionPacket {
    fn deserialize(mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut size = 0;

        let location = read_location2(&mut data, &mut size)?;
        let on_ground = read_bool(&mut data, &mut size)?;

        Ok((size, Packet::PlayerPosition(PlayerPositionPacket {
            location,
            on_ground,
        })))
    }

    fn id() -> u32 {
        0xD
    }
}
