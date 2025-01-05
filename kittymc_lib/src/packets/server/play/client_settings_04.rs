use crate::error::KittyMCError;
use crate::packets::packet_serialization::{read_bool, read_length_prefixed_string, read_u8, read_varint_u32, SerializablePacket};
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
            _ => ChatMode::Unknown
        }
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(PartialEq, Debug, Clone, Packet)]
    pub struct DisplayedSkinParts : u8 {
        const cape = 0b0000_0001;
        const jacket = 0b0000_0010;
        const left_sleeve = 0b0000_0100;
        const right_sleeve = 0b0000_1000;
        const left_pants_leg = 0b0001_0000;
        const right_pants_leg = 0b0010_0000;
        const hat = 0b0100_0000;
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
            _ => Hand::Unknown
        }
    }
}

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct ClientSettingsPacket {
    locale: String,
    view_distance: u8,
    chat_mode: ChatMode,
    chat_colors: bool,
    displayed_skin_parts: DisplayedSkinParts,
    main_hand: Hand,
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

        Ok((size, Packet::ClientSettings(ClientSettingsPacket {
            locale,
            view_distance,
            chat_mode,
            chat_colors,
            displayed_skin_parts,
            main_hand,
        })))
    }

    fn id() -> u32 {
        4
    }
}
