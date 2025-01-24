use std::error::Error;
use crate::packets::packet_serialization::SerializablePacket;
use crate::packets::wrap_packet;
use crate::subtypes::components::Component;
use kittymc_macros::Packet;

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct DisconnectPlayPacket {
    reason: Component,
}

impl DisconnectPlayPacket {
    pub fn default_restart() -> DisconnectPlayPacket {
        DisconnectPlayPacket {
            reason: Component::default_restart_disconnect(),
        }
    }

    pub fn default_error<E: Error>(e: &E) -> DisconnectPlayPacket {
        DisconnectPlayPacket {
            reason: Component::default_error(e),
        }
    }
}

impl SerializablePacket for DisconnectPlayPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        self.reason.write(&mut packet);
        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        0x1A
    }
}
