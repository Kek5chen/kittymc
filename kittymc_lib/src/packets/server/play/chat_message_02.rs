use crate::error::KittyMCError;
use crate::packets::packet_serialization::{read_length_prefixed_string, SerializablePacket};
use crate::packets::Packet;
use kittymc_macros::Packet;

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct ServerChatMessagePacket {
    pub message: String,
}

impl SerializablePacket for ServerChatMessagePacket {
    fn deserialize(mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut size = 0;

        let message = read_length_prefixed_string(&mut data, &mut size)?;

        Ok((
            size,
            Packet::ChatMessage(ServerChatMessagePacket { message }),
        ))
    }

    fn id() -> u32 {
        2
    }
}
