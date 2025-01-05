use crate::error::KittyMCError;
use crate::packets::packet_serialization::{read_varint_u32, SerializablePacket};
use crate::packets::server::play::client_settings_04::Hand;
use crate::packets::Packet;
use kittymc_macros::Packet;

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct ClientAnimationPacket {
    hand: Hand,
}

impl SerializablePacket for ClientAnimationPacket {
    fn deserialize(mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut size = 0;

        let hand = read_varint_u32(&mut data, &mut size)?.into();

        Ok((
            size,
            Packet::ClientAnimation(ClientAnimationPacket { hand }),
        ))
    }

    fn id() -> u32 {
        0x1D
    }
}
