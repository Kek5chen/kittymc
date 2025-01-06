use crate::packets::packet_serialization::{
    write_angle, write_u8, write_varint_u32, SerializablePacket,
};
use crate::packets::wrap_packet;
use crate::subtypes::Direction;
use kittymc_macros::Packet;

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct EntityLookPacket {
    pub entity_id: u32,
    pub direction: Direction,
    pub on_ground: bool,
}

impl SerializablePacket for EntityLookPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        let mut yaw = self.direction.x;
        while yaw > 180. {
            yaw -= 360.;
        }
        while yaw < -180. {
            yaw += 360.;
        }
        let mut pitch = self.direction.y;
        while pitch > 180. {
            pitch -= 360.;
        }
        while pitch < -180. {
            pitch += 360.;
        }

        write_varint_u32(&mut packet, self.entity_id);
        write_angle(&mut packet, yaw);
        write_angle(&mut packet, pitch);
        write_u8(&mut packet, self.on_ground as u8);

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        0x28
    }
}
