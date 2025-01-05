use crate::error::KittyMCError;
use crate::packets::packet_serialization::{read_varint_u32, write_varint_u32, SerializablePacket};
use crate::packets::{wrap_packet, Packet};
use kittymc_macros::Packet;

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct SetCompressionPacket {
    pub threshold: u32,
}

impl Default for SetCompressionPacket {
    fn default() -> Self {
        SetCompressionPacket {
            threshold: 256
        }
    }
}

impl SerializablePacket for SetCompressionPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_varint_u32(&mut packet, self.threshold);

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn deserialize(mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut size = 0;

        let threshold = read_varint_u32(&mut data, &mut size)?;

        Ok((size, Packet::SetCompression(SetCompressionPacket {
            threshold
        })))
    }

    fn id() -> u32 {
        3
    }
}