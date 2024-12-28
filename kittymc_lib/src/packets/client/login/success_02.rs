use crate::error::KittyMCError;
use crate::packets::packet_serialization::{read_length_prefixed_string, write_length_prefixed_string, SerializablePacket};
use crate::packets::{wrap_packet, Packet};

#[derive(PartialEq, Debug, Clone)]
pub struct LoginSuccessPacket {
    pub uuid: String,
    pub username: String,
}

impl SerializablePacket for LoginSuccessPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_length_prefixed_string(&mut packet, &self.uuid);

        write_length_prefixed_string(&mut packet, &self.username);

        wrap_packet(&mut packet, 2);

        packet
    }

    fn deserialize(mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut size = 0;

        let uuid = read_length_prefixed_string(&mut data, &mut size)?;

        let username = read_length_prefixed_string(&mut data, &mut size)?;

        Ok((size, Packet::LoginSuccess(LoginSuccessPacket{
            uuid,
            username,
        })))
    }
}