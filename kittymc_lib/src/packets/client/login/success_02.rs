use std::str::FromStr;
use uuid::{Builder, Uuid};
use crate::error::KittyMCError;
use crate::packets::packet_serialization::{read_length_prefixed_string, write_length_prefixed_string, SerializablePacket};
use crate::packets::{wrap_packet, Packet};

#[derive(PartialEq, Debug, Clone)]
pub struct LoginSuccessPacket {
    pub uuid: Uuid,
    pub username: String,
}

impl LoginSuccessPacket {
    pub fn generate_cracked_uuid(name: &str) -> Result<Uuid, KittyMCError> {
        if name.len() > 16 {
            return Err(KittyMCError::TooMuchData(name.len(), 16));
        }

        let md5 = md5::compute(format!("OfflinePlayer:{name}"));

        Ok(Builder::from_md5_bytes(md5.0).into_uuid())
    }

    pub fn from_name_cracked(name: &str) -> Result<Self, KittyMCError> {
        let uuid = Self::generate_cracked_uuid(name)?;

        Ok(LoginSuccessPacket{
            uuid,
            username: name.to_string(),
        })
    }
}

#[test]
fn test_cracked_uuid() {
    assert_eq!(LoginSuccessPacket::generate_cracked_uuid("will_owo").unwrap(), Uuid::from_str("0e22d127-3477-35f9-a65a-6fb3611c78fb").unwrap());
    assert_eq!(LoginSuccessPacket::generate_cracked_uuid("meow").unwrap(), Uuid::from_str("dadfb5ef-c239-3cb3-b316-aec3a76dbc71").unwrap());
    assert_eq!(LoginSuccessPacket::generate_cracked_uuid("IT0NA31").unwrap(), Uuid::from_str("fe86cee2-9d18-3100-bc41-6740712ec780").unwrap());
}


impl SerializablePacket for LoginSuccessPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_length_prefixed_string(&mut packet, &self.uuid.hyphenated().to_string());
        write_length_prefixed_string(&mut packet, &self.username);

        wrap_packet(&mut packet, 2);

        packet
    }

    fn deserialize(mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut size = 0;

        let uuid = read_length_prefixed_string(&mut data, &mut size)?;
        let uuid = Uuid::from_str(&uuid)?;

        let username = read_length_prefixed_string(&mut data, &mut size)?;

        Ok((size, Packet::LoginSuccess(LoginSuccessPacket{
            uuid,
            username,
        })))
    }
}