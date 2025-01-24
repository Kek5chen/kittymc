use kittymc_macros::Packet;
use crate::error::KittyMCError;
use crate::packets::Packet;
use crate::packets::packet_serialization::{read_block_location, read_f32, read_varint_u32, SerializablePacket};
use crate::packets::server::play::client_settings_04::Hand;
use crate::packets::server::play::player_digging_14::BlockFace;
use crate::subtypes::Location;

#[derive(Debug, Clone, PartialEq, Packet)]
pub struct PlayerBlockPlacementPacket {
    pub location: Location,
    pub face: BlockFace,
    pub hand: Hand,
    pub cursor_pos_x: f32,
    pub cursor_pos_y: f32,
    pub cursor_pos_z: f32,
}

impl SerializablePacket for PlayerBlockPlacementPacket {
    fn deserialize(mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut size = 0usize;

        let location = read_block_location(&mut data, &mut size)?;
        let face = read_varint_u32(&mut data, &mut size)?.into();
        let hand = read_varint_u32(&mut data, &mut size)?.into();
        let cursor_x = read_f32(&mut data, &mut size)?;
        let cursor_y = read_f32(&mut data, &mut size)?;
        let cursor_z = read_f32(&mut data, &mut size)?;

        Ok((size, Packet::PlayerBlockPlacement(Self {
            location,
            face,
            hand,
            cursor_pos_x: cursor_x,
            cursor_pos_y: cursor_y,
            cursor_pos_z: cursor_z,
        })))
    }

    fn id() -> u32 {
        0x1B
    }
}