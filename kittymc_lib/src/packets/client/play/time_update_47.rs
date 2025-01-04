use kittymc_macros::Packet;
use crate::packets::packet_serialization::{write_u64, SerializablePacket};
use crate::packets::wrap_packet;

#[derive(PartialEq, Debug, Clone, Default, Packet)]
pub struct TimeUpdatePacket {
    world_age: u64,
    time_of_day: u64,
}

impl SerializablePacket for TimeUpdatePacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_u64(&mut packet, self.world_age);
        write_u64(&mut packet, self.time_of_day);

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        0x47
    }
}
