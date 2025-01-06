use crate::error::KittyMCError;
use crate::packets::packet_serialization::{
    read_bool, read_length_prefixed_string, read_u8, read_varint_u32, write_u8, SerializablePacket,
};
use crate::packets::Packet;
use bitflags::bitflags;
use kittymc_macros::Packet;

#[repr(u32)]
#[derive(PartialEq, Debug, Clone)]
pub enum ChatMode {
    Enabled = 0,
    CommandsOnly = 1,
    Hidden = 2,
    Unknown = 0xFFFF_FFFF,
}

impl From<u32> for ChatMode {
    fn from(value: u32) -> Self {
        match value {
            0 => ChatMode::Enabled,
            1 => ChatMode::CommandsOnly,
            2 => ChatMode::Hidden,
            _ => ChatMode::Unknown,
        }
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(PartialEq, Debug, Clone, Packet)]
    pub struct DisplayedSkinParts : u8 {
        const cape            = 0x01;
        const jacket          = 0x02;
        const left_sleeve     = 0x04;
        const right_sleeve    = 0x08;
        const left_pants_leg  = 0x10;
        const right_pants_leg = 0x20;
        const hat             = 0x40;
    }
}

impl DisplayedSkinParts {
    pub fn write(&self, buffer: &mut Vec<u8>) {
        write_u8(buffer, self.bits());
    }
}

#[repr(u32)]
#[derive(PartialEq, Debug, Clone, Packet)]
pub enum Hand {
    Left = 0,
    Right = 1,
    Unknown = 0xFFFF_FFFF,
}

impl From<u32> for Hand {
    fn from(value: u32) -> Self {
        match value {
            0 => Hand::Left,
            1 => Hand::Right,
            _ => Hand::Unknown,
        }
    }
}

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct ClientSettingsPacket {
    pub locale: String,
    pub view_distance: u8,
    pub chat_mode: ChatMode,
    pub chat_colors: bool,
    pub displayed_skin_parts: DisplayedSkinParts,
    pub main_hand: Hand,
}

impl SerializablePacket for ClientSettingsPacket {
    fn deserialize(mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut size = 0;

        let locale = read_length_prefixed_string(&mut data, &mut size)?;
        let view_distance = read_u8(&mut data, &mut size)?;
        let chat_mode = read_varint_u32(&mut data, &mut size)?.into();
        let chat_colors = read_bool(&mut data, &mut size)?;
        let displayed_skin_parts = DisplayedSkinParts::from_bits(read_u8(&mut data, &mut size)?)
            .ok_or(KittyMCError::DeserializationError)?;
        let main_hand = read_varint_u32(&mut data, &mut size)?.into();

        Ok((
            size,
            Packet::ClientSettings(ClientSettingsPacket {
                locale,
                view_distance,
                chat_mode,
                chat_colors,
                displayed_skin_parts,
                main_hand,
            }),
        ))
    }

    fn id() -> u32 {
        4
    }
}
