use crate::packets::packet_serialization::{write_bool, write_i16, write_varint_i32, SerializablePacket};
use crate::packets::wrap_packet;
use kittymc_macros::Packet;

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct EntityRelativeMovePacket {
    pub entity_id: i32,
    pub delta_x: i16,
    pub delta_y: i16,
    pub delta_z: i16,
    pub on_ground: bool,
}

impl SerializablePacket for EntityRelativeMovePacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_varint_i32(&mut packet, self.entity_id);
        write_i16(&mut packet, self.delta_x);
        write_i16(&mut packet, self.delta_y);
        write_i16(&mut packet, self.delta_z);
        write_bool(&mut packet, self.on_ground);

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        0x26
    }
}
