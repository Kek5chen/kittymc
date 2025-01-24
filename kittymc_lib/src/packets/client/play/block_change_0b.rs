use crate::packets::packet_serialization::{
    write_block_location, write_varint_u32, SerializablePacket,
};
use crate::packets::wrap_packet;
use crate::subtypes::Location;
use kittymc_macros::Packet;
use log::info;
use crate::packets::client::play::chunk_data_20::BlockStateId;

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct BlockChangePacket {
    pub location: Location,
    pub block_id: BlockStateId,
}

impl BlockChangePacket {
    pub fn new(location: Location, block_id: BlockStateId) -> BlockChangePacket {
        BlockChangePacket { location, block_id }
    }

    pub fn new_empty(location: Location) -> BlockChangePacket {
        BlockChangePacket {
            location,
            block_id: 0b0000_0000,
        }
    }
}

impl SerializablePacket for BlockChangePacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];
        info!("Chaging block to {:?}", self);

        write_block_location(&mut packet, &self.location);
        write_varint_u32(&mut packet, self.block_id);

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        0x0B
    }
}
