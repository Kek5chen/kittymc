use kittymc_macros::SerializePacketFunc;
use crate::error::KittyMCError;
use crate::packets::packet_serialization::{read_varint_u32, SerializablePacket};
use crate::packets::handshake_00::HandshakePacket;

pub mod handshake_00;
pub mod packet_serialization;


#[derive(SerializePacketFunc, PartialEq, Debug, Clone)]
pub enum Packet {
    Handshake(HandshakePacket),
}

impl Packet {
    pub fn packet_id(&self) -> u32 {
        match self {
            Packet::Handshake(_) => 0,
        }
    }

    pub fn deserialize_packet(mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut packet_len_len = 0;
        let packet_len = read_varint_u32(&mut data, &mut packet_len_len)? as usize;
        let mut packet_id_len = 0;
        let packet_id = read_varint_u32(&mut data, &mut packet_id_len)? as usize;
        let total_size = packet_len_len + packet_id_len;

        if packet_len > data.len() {
            return Err(KittyMCError::NotEnoughData);
        }

        let (size, packet) = match packet_id {
            0 => {
                let result = HandshakePacket::deserialize(&data[..packet_len])?;
                result
            },
            _ => return Err(KittyMCError::NotImplemented),
        };

        Ok((total_size + size, packet))
    }
}
