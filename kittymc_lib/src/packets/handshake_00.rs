use integer_encoding::VarInt;
use crate::error::KittyMCError;
use crate::packets::Packet;
use crate::packets::packet_serialization::{read_state_varint, read_u16_be, read_length_prefixed_string, read_varint_u32, write_be_u16, write_length_prefixed_string, write_varint_u32, SerializablePacket};
use crate::subtypes::state::State;

#[derive(Debug, Clone, PartialEq)]
pub struct HandshakePacket {
    pub protocol_version: u32,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: State,
}

impl SerializablePacket for HandshakePacket {
    fn serialize(&self) -> Vec<u8> {
        let mut buffer = vec![];

        write_varint_u8(&mut buffer, 0);
        write_varint_u32(&mut buffer, self.protocol_version);
        write_length_prefixed_string(&mut buffer, &self.server_address);
        write_be_u16(&mut buffer, self.server_port);
        write_varint_u32(&mut buffer, self.next_state as u32);

        let total_len = buffer.len().encode_var_vec();
        buffer.splice(0..0, total_len);

        buffer
    }

    fn deserialize(mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut total_size = 0;

        let protocol_version = read_varint_u32(&mut data, &mut total_size)?;
        let server_address = read_length_prefixed_string(&mut data, &mut total_size)?;
        let server_port = read_u16_be(&mut data, &mut total_size)?;
        let next_state = read_state_varint(&mut data, &mut total_size)?;

        Ok((total_size, Packet::Handshake(HandshakePacket {
            protocol_version,
            server_address,
            server_port,
            next_state,
        })))
    }
}

