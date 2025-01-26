use crate::packets::packet_serialization::{write_varint_i32, write_varint_u32, SerializablePacket};
use crate::packets::wrap_packet;
use kittymc_macros::Packet;

#[derive(PartialEq, Debug, Clone, Default, Packet)]
pub struct DestroyEntitiesPacket {
    entity_ids: Vec<i32>,
}

impl DestroyEntitiesPacket {
    pub fn new(entity_ids: Vec<i32>) -> Self {
        DestroyEntitiesPacket { entity_ids }
    }
}

impl SerializablePacket for DestroyEntitiesPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_varint_u32(&mut packet, self.entity_ids.len() as u32);
        for entity_id in &self.entity_ids {
            write_varint_i32(&mut packet, *entity_id);
        }

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        0x32
    }
}
