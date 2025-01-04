use kittymc_macros::Packet;
use crate::packets::packet_serialization::SerializablePacket;
use crate::packets::wrap_packet;
use crate::subtypes::{Chat, ChatBuilder};

#[derive(Debug, Clone, Packet)]
pub struct DisconnectLoginPacket {
    reason: Chat
}

impl Default for DisconnectLoginPacket {
    fn default() -> Self {
        DisconnectLoginPacket {
            reason: ChatBuilder::default()
                .text("§4[§5K§6I§eT§aT§bY §dMC§4]§r YOU'VE BEEN CLAWED OUT §b:<§r!\n§dS0RRYY, ITS OV3R".to_string())
                .build()
                .unwrap()
        }
    }
}

impl DisconnectLoginPacket {
    pub fn wrong_version() -> Self {
        DisconnectLoginPacket {
            reason: ChatBuilder::default()
            .text("§4[§5K§6I§eT§aT§bY §dMC§4]§r WRONG VERSION. MORON~ §b:<§r!\n§dHop on 1.12.2!!!".to_string())
            .build()
            .unwrap()
        }
    }
}

impl SerializablePacket for DisconnectLoginPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        self.reason.write(&mut packet);

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        0
    }
}
