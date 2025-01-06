use crate::packets::packet_serialization::{write_u8, write_varint_u32, SerializablePacket};
use crate::packets::wrap_packet;
use kittymc_macros::Packet;

#[derive(PartialEq, Debug, Clone)]
pub enum AnimationType {
    SwingMainArm = 0,
    TakeDamage = 1,
    LeaveBed = 2,
    SwingOffHand = 3,
    CriticalEffect = 4,
    MagicCriticalEffect = 5,
}

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct ServerAnimationPacket {
    pub entity_id: u32,
    pub animation: AnimationType,
}

impl SerializablePacket for ServerAnimationPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_varint_u32(&mut packet, self.entity_id);
        write_u8(&mut packet, self.animation.clone() as u8);
        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        6
    }
}
