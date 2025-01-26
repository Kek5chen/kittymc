use kittymc_macros::Packet;
use crate::packets::packet_serialization::{write_varint_i32, SerializablePacket};
use crate::packets::wrap_packet;
use crate::subtypes::metadata::MetadataObject;

#[derive(Clone, PartialEq, Debug, Packet)]
pub struct EntityMetadataPacket<M> {
    entity_id: i32,
    metadata: M,
}

impl<M: MetadataObject> EntityMetadataPacket<M> {
    pub fn new(entity_id: i32, metadata: M) -> Self {
        Self { entity_id, metadata }
    }
}

impl<M: MetadataObject> SerializablePacket for EntityMetadataPacket<M> {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_varint_i32(&mut packet, self.entity_id);
        self.metadata.write_metadata(&mut packet);

        wrap_packet(&mut packet, Self::id());

        packet
    }
    fn id() -> u32 {
        0x3C
    }
}
