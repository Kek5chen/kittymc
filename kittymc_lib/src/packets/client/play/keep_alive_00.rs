use crate::error::KittyMCError;
use crate::packets::packet_serialization::{read_varint_u32, write_varint_u32, SerializablePacket};
use crate::packets::{wrap_packet, Packet};

#[derive(PartialEq, Debug, Clone)]
pub struct KeepAlivePacket {
    id: u32,
}

impl KeepAlivePacket {
    pub fn new(id: u32) -> Self {
        KeepAlivePacket {
            id,
        }
    }
}

impl SerializablePacket for KeepAlivePacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_varint_u32(&mut packet, self.id);

        wrap_packet(&mut packet, 2);

        packet
    }

    fn deserialize(mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut size = 0;

        let id = read_varint_u32(&mut data, &mut size)?;

        Ok((size, Packet::KeepAlive(KeepAlivePacket{
            id
        })))
    }
}
