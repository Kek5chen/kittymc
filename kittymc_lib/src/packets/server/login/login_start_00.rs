use kittymc_macros::Packet;
use crate::error::KittyMCError;
use crate::packets::{wrap_packet, Packet};
use crate::packets::packet_serialization::{read_length_prefixed_string, write_length_prefixed_string, SerializablePacket};

#[derive(Debug, Clone, PartialEq, Packet)]
pub struct LoginStartPacket {
    pub name: String,
}

impl SerializablePacket for LoginStartPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_length_prefixed_string(&mut packet, &self.name);
        wrap_packet(&mut packet, 0);

        packet
    }

    // not including length or packet id
    fn deserialize(mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut size = 0;
        let name = read_length_prefixed_string(&mut data, &mut size)?;

        Ok((size, Packet::LoginStart(LoginStartPacket {
            name,
        })))
    }
}