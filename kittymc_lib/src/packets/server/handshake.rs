use crate::error::KittyMCError;
use crate::packets::packet_serialization::{read_length_prefixed_string, read_u16, read_varint_u32, write_length_prefixed_string, write_u16, write_varint_u32, SerializablePacket};
use crate::packets::{wrap_packet, Packet};
use crate::subtypes::state::State;
use kittymc_macros::Packet;

#[derive(Debug, Clone, PartialEq, Packet)]
pub struct HandshakePacket {
    pub protocol_version: u32,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: State,
}

impl SerializablePacket for HandshakePacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_varint_u32(&mut packet, self.protocol_version);
        write_length_prefixed_string(&mut packet, &self.server_address);
        write_u16(&mut packet, self.server_port);
        write_varint_u32(&mut packet, self.next_state as u32);

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn deserialize(mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut total_size = 0;

        let protocol_version = read_varint_u32(&mut data, &mut total_size)?;
        let server_address = read_length_prefixed_string(&mut data, &mut total_size)?;
        let server_port = read_u16(&mut data, &mut total_size)?;
        let next_state = State::from(read_varint_u32(&mut data, &mut total_size)?);

        Ok((total_size, Packet::Handshake(HandshakePacket {
            protocol_version,
            server_address,
            server_port,
            next_state,
        })))
    }

    fn id() -> u32 {
        0
    }
}

