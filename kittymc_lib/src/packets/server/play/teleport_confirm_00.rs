use crate::error::KittyMCError;
use crate::packets::packet_serialization::{read_varint_u32, SerializablePacket};
use crate::packets::Packet;
use kittymc_macros::Packet;

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct TeleportConfirmPacket {
    teleport_id: u32,
}

impl SerializablePacket for TeleportConfirmPacket {
    fn deserialize(mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut size = 0;

        let teleport_id = read_varint_u32(&mut data, &mut size)?;

        Ok((size, Packet::TeleportConfirm(TeleportConfirmPacket {
            teleport_id
        })))
    }

    fn id() -> u32 {
        0
    }
}
