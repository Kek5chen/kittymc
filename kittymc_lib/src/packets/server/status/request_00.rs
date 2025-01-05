use crate::error::KittyMCError;
use crate::packets::packet_serialization::SerializablePacket;
use crate::packets::Packet;
use kittymc_macros::Packet;

#[derive(PartialEq, Clone, Debug, Packet)]
pub struct StatusRequestPacket;

impl SerializablePacket for StatusRequestPacket {
    fn serialize(&self) -> Vec<u8> {
        vec![1, 0]
    }

    fn deserialize(_data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        Ok((0, Packet::StatusRequest(StatusRequestPacket)))
    }

    fn id() -> u32 {
        0
    }
}
