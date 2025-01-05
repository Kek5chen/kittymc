use crate::packets::packet_serialization::{write_u64, SerializablePacket};
use crate::packets::wrap_packet;
use kittymc_macros::Packet;

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct ServerKeepAlivePacket {
    pub id: u64,
}

impl ServerKeepAlivePacket {
    pub fn new(id: u64) -> Self {
        ServerKeepAlivePacket { id }
    }
}

impl SerializablePacket for ServerKeepAlivePacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_u64(&mut packet, self.id);

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        0x1F
    }
}
