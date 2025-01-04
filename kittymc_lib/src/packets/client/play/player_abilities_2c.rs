use bitflags::bitflags;
use kittymc_macros::Packet;
use crate::packets::packet_serialization::{write_f32, write_u8, SerializablePacket};
use crate::packets::wrap_packet;

bitflags! {

    #[repr(transparent)]
    #[derive(PartialEq, Debug, Clone)]
    pub struct PlayerAbilitiesFlags: u8 {
        const invulnerable = 0b00000001;
        const flying = 0b00000010;
        const allow_flying = 0b00000100;
        const creative_mode = 0b00001000;
    }
}

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct PlayerAbilitiesPacket {
    flags: PlayerAbilitiesFlags,
    flying_speed: f32,
    field_of_view_modifier: f32,
}

impl Default for PlayerAbilitiesPacket {
    fn default() -> Self {
        PlayerAbilitiesPacket {
            flags: PlayerAbilitiesFlags::all(),
            flying_speed: 1.0,
            field_of_view_modifier: 1.0,
        }
    }
}

impl SerializablePacket for PlayerAbilitiesPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_u8(&mut packet, self.flags.bits());
        write_f32(&mut packet, self.flying_speed);
        write_f32(&mut packet, self.field_of_view_modifier);

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        0x2C
    }
}
