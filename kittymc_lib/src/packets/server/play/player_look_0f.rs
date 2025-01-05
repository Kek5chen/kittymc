use crate::error::KittyMCError;
use crate::packets::client::play::Direction;
use crate::packets::packet_serialization::{read_bool, read_direction, SerializablePacket};
use crate::packets::Packet;
use kittymc_macros::Packet;

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct PlayerLookPacket {
    direction: Direction,
    on_ground: bool,
}

impl SerializablePacket for PlayerLookPacket {
    fn deserialize(mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut size = 0;

        let direction = read_direction(&mut data, &mut size)?;
        let on_ground = read_bool(&mut data, &mut size)?;

        Ok((size, Packet::PlayerLook(PlayerLookPacket {
            direction,
            on_ground,
        })))
    }

    fn id() -> u32 {
        0xF
    }
}
