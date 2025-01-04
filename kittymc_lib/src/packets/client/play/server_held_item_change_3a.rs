use kittymc_macros::Packet;
use crate::packets::packet_serialization::{write_u8, SerializablePacket};
use crate::packets::wrap_packet;

#[derive(PartialEq, Debug, Clone, Default, Packet)]
pub struct ServerHeldItemChangePacket {
    slot: u8
}

impl SerializablePacket for ServerHeldItemChangePacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_u8(&mut packet, self.slot);

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        0x3A
    }
}
