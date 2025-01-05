use crate::packets::packet_serialization::{write_length_prefixed_bytes, write_length_prefixed_string, SerializablePacket};
use crate::packets::wrap_packet;
use kittymc_macros::Packet;

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct ServerPluginMessagePacket {
    pub channel: String,
    pub data: Vec<u8>,
}

impl ServerPluginMessagePacket {
    pub fn default_brand() -> Self {
        ServerPluginMessagePacket {
            channel: "MC|Brand".to_string(),
            data: "vanilla".as_bytes().to_vec(),
        }
    }
}

impl SerializablePacket for ServerPluginMessagePacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_length_prefixed_string(&mut packet, &self.channel);
        write_length_prefixed_bytes(&mut packet, &self.data);

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        0x18
    }
}
