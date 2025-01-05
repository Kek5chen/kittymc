use crate::packets::packet_serialization::{write_i32, SerializablePacket};
use crate::packets::wrap_packet;
use crate::subtypes::ChunkPosition;
use kittymc_macros::Packet;

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct UnloadChunkPacket {
    x: i32,
    z: i32,
}

impl UnloadChunkPacket {
    pub fn new(pos: &ChunkPosition) -> UnloadChunkPacket {
        UnloadChunkPacket {
            x: pos.chunk_x() as i32,
            z: pos.chunk_z() as i32,
        }
    }
}

impl SerializablePacket for UnloadChunkPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_i32(&mut packet, self.x);
        write_i32(&mut packet, self.z);

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        0x1D
    }
}
