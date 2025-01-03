use crate::error::KittyMCError;
use crate::packets::packet_serialization::{read_varint_u32, write_varint_u32, SerializablePacket};
use crate::packets::{wrap_packet, Packet};

#[derive(PartialEq, Debug, Clone)]
pub struct SetCompressionPacket {
    threshold: u32,
}

impl Default for SetCompressionPacket {
    fn default() -> Self {
        SetCompressionPacket {
            threshold: 0
        }
    }
}

impl SerializablePacket for SetCompressionPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_varint_u32(&mut packet, self.threshold);

        wrap_packet(&mut packet, 3);

        packet
    }

    fn deserialize(mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut size = 0;

        let threshold = read_varint_u32(&mut data, &mut size)?;

        Ok((size, Packet::SetCompression(SetCompressionPacket {
            threshold
        })))
    }

}