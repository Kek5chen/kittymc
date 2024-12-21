use kittymc_macros::SerializePacketFunc;
use crate::packets::packet_serialization::SerializablePacket;
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
}
