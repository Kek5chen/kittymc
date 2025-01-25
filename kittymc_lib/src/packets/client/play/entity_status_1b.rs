use crate::packets::packet_serialization::{write_i32, write_u8, SerializablePacket};
use crate::packets::wrap_packet;
use kittymc_macros::Packet;

#[derive(PartialEq, Debug, Clone, Default, Packet)]
pub struct EntityStatusPacket {
    pub entity_id: i32,
    pub entity_status: u8,
}

impl SerializablePacket for EntityStatusPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_i32(&mut packet, self.entity_id);
        write_u8(&mut packet, self.entity_status);

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        0x1B
    }
}
