use crate::error::KittyMCError;
use crate::packets::packet_serialization::{read_i16, SerializablePacket};
use crate::packets::Packet;
use kittymc_macros::Packet;

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct ClientHeldItemChangePacket {
    pub slot: i16,
}

impl SerializablePacket for ClientHeldItemChangePacket {
    fn deserialize(mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut size = 0;

        let slot = read_i16(&mut data, &mut size)?;

        Ok((
            size,
            Packet::ClientHeldItemChange(ClientHeldItemChangePacket { slot }),
        ))
    }

    fn id() -> u32 {
        0xA1
    }
}
