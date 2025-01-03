use kittymc_macros::Packet;
use crate::error::KittyMCError;
use crate::packets::{wrap_packet, Packet};
use crate::packets::packet_serialization::{read_i64, write_i64, SerializablePacket};

// Special Packet. Is being used for serializing the clientbound Ping and deserializing the serverbound Pong
#[derive(PartialEq, Debug, Clone, Packet)]
pub struct StatusPingPongPacket {
    payload: i64
}

impl SerializablePacket for StatusPingPongPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_i64(&mut packet, self.payload);

        wrap_packet(&mut packet, 1);

        packet
    }
    fn deserialize(mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut size = 0;
        let payload = read_i64(&mut data, &mut size)?;

        Ok((size, Packet::StatusPing(StatusPingPongPacket {
            payload
        })))
    }
}