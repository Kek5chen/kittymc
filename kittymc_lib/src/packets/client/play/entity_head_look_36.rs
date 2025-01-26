use crate::packets::packet_serialization::{write_angle, write_varint_i32, SerializablePacket};
use crate::packets::wrap_packet;
use kittymc_macros::Packet;

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct EntityHeadLookPacket {
    pub entity_id: i32,
    pub yaw: f32,
}

impl SerializablePacket for EntityHeadLookPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        let mut yaw = self.yaw;
        while yaw > 180. {
            yaw -= 360.;
        }
        while yaw < -180. {
            yaw += 360.;
        }

        write_varint_i32(&mut packet, self.entity_id);
        write_angle(&mut packet, yaw);

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        0x36
    }
}
