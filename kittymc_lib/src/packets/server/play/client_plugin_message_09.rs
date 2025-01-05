use crate::error::KittyMCError;
use crate::packets::packet_serialization::{
    read_length_prefixed_bytes, read_length_prefixed_string, SerializablePacket,
};
use crate::packets::Packet;
use kittymc_macros::Packet;
use std::fmt::{Debug, Formatter};

#[derive(PartialEq, Clone, Packet)]
pub struct ClientPluginMessagePacket {
    pub channel: String,
    pub data: Vec<u8>,
}

impl Debug for ClientPluginMessagePacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientPluginMessagePacket")
            .field("channel", &self.channel)
            .field(
                "data",
                &format!("{:?} ({})", self.data, String::from_utf8_lossy(&self.data)),
            )
            .finish()
    }
}

impl SerializablePacket for ClientPluginMessagePacket {
    fn deserialize(mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut total_size = 0;
        let channel = read_length_prefixed_string(&mut data, &mut total_size)?;
        let data = read_length_prefixed_bytes(&mut data, &mut total_size)?;

        Ok((
            total_size,
            Packet::PluginMessage(ClientPluginMessagePacket { channel, data }),
        ))
    }

    fn id() -> u32 {
        0x18
    }
}
