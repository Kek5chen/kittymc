use integer_encoding::VarInt;
use crate::error::KittyMCError;
use crate::packets::client::login::success_02::LoginSuccessPacket;
use crate::packets::client::status::response_00::StatusResponsePacket;
use crate::packets::packet_serialization::{read_varint_u32, SerializablePacket};
use crate::packets::server::handshake::HandshakePacket;
use crate::packets::server::login::login_start_00::LoginStartPacket;
use crate::packets::server::status::ping_01::StatusPingPongPacket;
use crate::subtypes::state::State;

pub mod server;
pub mod client;
pub mod packet_serialization;

#[derive(PartialEq, Debug, Clone)]
pub enum Packet {
    Handshake(HandshakePacket),
    LoginStart(LoginStartPacket),
    LoginSuccess(LoginSuccessPacket),
    StatusRequest,
    StatusResponse(StatusResponsePacket),
    StatusPing(StatusPingPongPacket),
    StatusPong(StatusPingPongPacket),
}

impl Packet {
    pub fn packet_id(&self) -> u32 {
        match self {
            Packet::Handshake(_) |
            Packet::LoginStart(_) |
            Packet::StatusRequest |
            Packet::StatusResponse(_) => 0,

            Packet::StatusPing(_) |
            Packet::StatusPong(_) => 1,

            Packet::LoginSuccess(_) => 2,
        }
    }
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            Self::StatusRequest => vec![1, 0],
            Self::Handshake(inner) => inner.serialize(),
            Self::LoginStart(inner) => inner.serialize(),
            Self::LoginSuccess(inner) => inner.serialize(),
            Self::StatusResponse(inner) => inner.serialize(),
            Self::StatusPing(inner) => inner.serialize(),
            Self::StatusPong(inner) => inner.serialize(),
        }
    }

    pub fn deserialize_packet(state: State, mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut packet_len_len = 0;
        let packet_len = read_varint_u32(&mut data, &mut packet_len_len)? as usize;
        let packet_len = packet_len - packet_len_len;
        let mut packet_id_len = 0;
        let packet_id = read_varint_u32(&mut data, &mut packet_id_len)? as usize;
        let total_size = packet_len_len + packet_id_len;

        if packet_len > data.len() {
            return Err(KittyMCError::NotEnoughData(data.len(), packet_len));
        }

        let (size, packet) = match state {
            State::Handshake => {
                match packet_id {
                    0 => HandshakePacket::deserialize(&data[..packet_len])?,
                    _ => return Err(KittyMCError::NotImplemented),
                }
            }
            State::Status => {
                match packet_id {
                    0 => (0, Packet::StatusRequest),
                    1 => StatusPingPongPacket::deserialize(&data[..packet_len])?,
                    _ => return Err(KittyMCError::NotImplemented),
                }
            }
            State::Login => {
                match packet_id {
                    0 => LoginStartPacket::deserialize(&data[..packet_len])?,
                    _ => return Err(KittyMCError::NotImplemented),
                }
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