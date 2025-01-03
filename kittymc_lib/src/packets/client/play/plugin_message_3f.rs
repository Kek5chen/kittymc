use kittymc_macros::Packet;
use crate::error::KittyMCError;
use crate::packets::packet_serialization::{read_length_prefixed_bytes, read_length_prefixed_string, write_length_prefixed_bytes, write_length_prefixed_string, SerializablePacket};
use crate::packets::{wrap_packet, Packet};

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct PluginMessagePacket {
    pub channel: String,
    pub data: Vec<u8>,
}

impl PluginMessagePacket {
    pub fn default_brand() -> Self {
        PluginMessagePacket {
            channel: "MC|Brand".to_string(),
            data: "vanilla".as_bytes().to_vec(),
        }
    }
}

impl SerializablePacket for PluginMessagePacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_length_prefixed_string(&mut packet, &self.channel);
        write_length_prefixed_bytes(&mut packet, &self.data);

        wrap_packet(&mut packet, 0x3F);

        packet
    }

    fn deserialize(mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut total_size = 0;
        let channel = read_length_prefixed_string(&mut data, &mut total_size)?;
        let data = read_length_prefixed_bytes(&mut data, &mut total_size)?;

        Ok((total_size, Packet::PluginMessage(PluginMessagePacket {
            channel,
            data
        })))
    }
}
