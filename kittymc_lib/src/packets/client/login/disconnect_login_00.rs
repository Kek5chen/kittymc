use crate::packets::packet_serialization::SerializablePacket;
use crate::packets::wrap_packet;
use crate::subtypes::components::{Component, TextComponent};
use kittymc_macros::Packet;

#[derive(Debug, Packet)]
pub struct DisconnectLoginPacket {
    reason: Component,
}

impl Default for DisconnectLoginPacket {
    fn default() -> Self {
        DisconnectLoginPacket {
            reason: Component::Text(TextComponent::builder()
                .text("§4[§5K§6I§eT§aT§bY §dMC§4]§r YOU'VE BEEN DISCONNECTED §b:<§r!\n\n§dS0RRYY, ITS OV3R".to_string())
                .build())
        }
    }
}

impl DisconnectLoginPacket {
    pub fn wrong_version() -> Self {
        DisconnectLoginPacket {
            reason: Component::Text(TextComponent::builder()
                .text("§4[§5K§6I§eT§aT§bY §dMC§4]§r BUUUH, WRONG VERSION. §b:<§r!\n§dHop on 1.12.2 :3".to_string())
                .build())
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
