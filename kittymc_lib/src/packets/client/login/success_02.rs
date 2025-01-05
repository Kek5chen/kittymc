use crate::error::KittyMCError;
use crate::packets::packet_serialization::{read_length_prefixed_string, write_length_prefixed_string, SerializablePacket};
use crate::packets::{wrap_packet, Packet};
use crate::utils::generate_cracked_uuid;
use kittymc_macros::Packet;
use std::str::FromStr;
use uuid::Uuid;

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct LoginSuccessPacket {
    pub uuid: Uuid,
    pub username: String,
}

impl LoginSuccessPacket {
    pub fn from_name_cracked(name: &str) -> Result<Self, KittyMCError> {
        let uuid = generate_cracked_uuid(name)?;

        Ok(LoginSuccessPacket {
            uuid,
            username: name.to_string(),
        })
    }
}

impl SerializablePacket for LoginSuccessPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_length_prefixed_string(&mut packet, &self.uuid.hyphenated().to_string());
        write_length_prefixed_string(&mut packet, &self.username);

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn deserialize(mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut size = 0;

        let uuid = read_length_prefixed_string(&mut data, &mut size)?;
        let uuid = Uuid::from_str(&uuid)?;

        let username = read_length_prefixed_string(&mut data, &mut size)?;

        Ok((size, Packet::LoginSuccess(LoginSuccessPacket {
            uuid,
            username,
        })))
    }

    fn id() -> u32 {
        2
    }
}