use crate::packets::packet_serialization::{
    write_block_location, write_u8, write_varint_u32, SerializablePacket,
};
use crate::packets::wrap_packet;
use crate::subtypes::Location;
use kittymc_macros::Packet;

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct BlockBreakAnimationPacket {
    entity_id: u32,
    location: Location,
    destroy_stage: u8,
}

impl BlockBreakAnimationPacket {
    pub fn new(entity_id: u32, location: Location, destroy_stage: u8) -> BlockBreakAnimationPacket {
        BlockBreakAnimationPacket {
            entity_id,
            location,
            destroy_stage,
        }
    }
}

impl SerializablePacket for BlockBreakAnimationPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_varint_u32(&mut packet, self.entity_id);
        write_block_location(&mut packet, &self.location);
        write_u8(&mut packet, self.destroy_stage);

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        8
    }
}
