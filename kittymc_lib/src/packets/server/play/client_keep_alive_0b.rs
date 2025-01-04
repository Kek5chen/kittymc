use kittymc_macros::Packet;
use crate::error::KittyMCError;
use crate::packets::packet_serialization::{read_u64, SerializablePacket};
use crate::packets::Packet;

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct ClientKeepAlivePacket {
    pub id: u64,
}

impl ClientKeepAlivePacket {
    pub fn new(id: u64) -> Self {
        ClientKeepAlivePacket {
            id,
        }
    }
}

impl SerializablePacket for ClientKeepAlivePacket {
    fn deserialize(mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut size = 0;

        let id = read_u64(&mut data, &mut size)?;

        Ok((size, Packet::KeepAlive(ClientKeepAlivePacket {
            id
        })))
    }

    fn id() -> u32 {
        0x0B
    }
}
