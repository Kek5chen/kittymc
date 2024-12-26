use std::str::FromStr;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::KittyMCError;
use crate::packets::{wrap_packet, Packet};
use crate::packets::packet_serialization::{read_length_prefixed_string, write_length_prefixed_string, SerializablePacket};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StatusResponseVersion {
    name: String,
    protocol: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StatusResponsePlayer {
    name: String,
    id: Uuid,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StatusResponsePlayers {
    max: u32,
    online: u32,
    sample: Vec<StatusResponsePlayer>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StatusResponseText {
    text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StatusResponsePacket {
    version: StatusResponseVersion,
    players: StatusResponsePlayers,
    description: StatusResponseText,
    favicon: String,
    #[serde(rename = "enforcesSecureChat")]
    enforces_secure_chat: bool,
}

impl Default for StatusResponsePacket {
    fn default() -> Self {

        StatusResponsePacket {
            version: StatusResponseVersion {
                name: "1.8.8".to_string(), // TODO: Replace this with global versioned consts
                protocol: 47,
            },
            players: StatusResponsePlayers {
                max: 69,
                online: 55,
                sample: vec![
                    StatusResponsePlayer {
                        name: "will_owo".to_string(),
                        id: Uuid::from_str("6eab089f-9698-47fb-8fe5-c95fb5d20b6c").unwrap(),
                    },
                    StatusResponsePlayer {
                        name: "IT0NA31".to_string(),
                        id: Uuid::from_str("f7671649-0271-4749-aa00-2a5ea2cb573d").unwrap(),
                    }
                ],
            },
            description: StatusResponseText { text: "A kittyful MC Server".to_string() },
            favicon: "".to_string(),
            enforces_secure_chat: false,
        }
    }
}

impl SerializablePacket for StatusResponsePacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];
        let json_response = serde_json::to_string(self).unwrap_or_else(|_| "".to_string());

        write_length_prefixed_string(&mut packet, json_response.as_str());

        wrap_packet(&mut packet, 0);

        packet
    }

    fn deserialize(mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut size = 0;

        let string = read_length_prefixed_string(&mut data, &mut size)?;

        let response = serde_json::from_str::<StatusResponsePacket>(string.as_str())?;

        Ok((size, Packet::StatusResponse(response)))
    }
}
