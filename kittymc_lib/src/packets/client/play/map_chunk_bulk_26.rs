use kittymc_macros::Packet;
use crate::packets::packet_serialization::SerializablePacket;
use crate::packets::wrap_packet;

pub struct ChunkMeta {
    pos_x: i32,
    pos_y: i32,
    primary_bit_mask: u16,
}

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct MapChunkBulkPacket {
    sky_light_sent: bool,
}

impl Default for MapChunkBulkPacket {
    fn default() -> Self {
        MapChunkBulkPacket {
            sky_light_sent: false,
        }
    }
}

impl SerializablePacket for MapChunkBulkPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];


        wrap_packet(&mut packet, 0x26);

        packet
    }
}
