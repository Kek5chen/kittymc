use crate::packets::packet_serialization::SerializablePacket;
use crate::packets::wrap_packet;
use crate::subtypes::components::{Component, TextComponent};
use crate::utils::KITTYMC_TAG;
use kittymc_macros::Packet;

#[derive(Debug, Packet)]
pub struct DisconnectLoginPacket {
    reason: Component,
}

impl Default for DisconnectLoginPacket {
    fn default() -> Self {
        DisconnectLoginPacket {
            reason: Component::Text(
                TextComponent::builder()
                    .text(format!(
                        "{KITTYMC_TAG} YOU'VE BEEN DISCONNECTED §b:<§r!\n\n§dS0RRYY, ITS OV3R"
                    ))
                    .build(),
            ),
        }
    }
}

impl DisconnectLoginPacket {
    pub fn wrong_version() -> Self {
        DisconnectLoginPacket {
            reason: Component::Text(
                TextComponent::builder()
                    .text(format!(
                        "{KITTYMC_TAG} BUUUH, WRONG VERSION. §b:<§r!\n§dHop on 1.12.2 :3"
                    ))
                    .build(),
            ),
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
