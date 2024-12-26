use integer_encoding::VarInt;
use kittymc_macros::SerializePacketFunc;
use crate::error::KittyMCError;
use crate::packets::packet_serialization::{read_varint_u32, SerializablePacket};
use crate::packets::server::handshake::HandshakePacket;
use crate::packets::server::login::login_00::LoginStartPacket;
use crate::subtypes::state::State;

pub mod server;
pub mod client;
pub mod packet_serialization;

#[derive(SerializePacketFunc, PartialEq, Debug, Clone)]
pub enum Packet {
    Handshake(HandshakePacket),
    LoginStart(LoginStartPacket),
}

impl Packet {
    pub fn packet_id(&self) -> u32 {
        match self {
            Packet::Handshake(_) => 0,
            Packet::LoginStart(_) => 0,
        }
    }

    pub fn deserialize_packet(state: State, mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut packet_len_len = 0;
        let packet_len = read_varint_u32(&mut data, &mut packet_len_len)? as usize;
        let mut packet_id_len = 0;
        let packet_id = read_varint_u32(&mut data, &mut packet_id_len)? as usize;
        let total_size = packet_len_len + packet_id_len;

        if packet_len > data.len() {
            return Err(KittyMCError::NotEnoughData);
        }

        let (size, packet) = match packet_id {
            0 if state == State::Handshake => {
                HandshakePacket::deserialize(&data[..packet_len])?
            },
            0 if state == State::Login => {
                LoginStartPacket::deserialize(&data[..packet_len])?
            }
            _ => return Err(KittyMCError::NotImplemented),
        };

        Ok((total_size + size, packet))
    }
}

pub fn wrap_packet(packet: &mut Vec<u8>, id: u32) {
    // add packet id
    packet.splice(0..0, id.encode_var_vec());

    // set total length
    let total_len = packet.len().encode_var_vec();
    packet.splice(0..0, total_len);
}