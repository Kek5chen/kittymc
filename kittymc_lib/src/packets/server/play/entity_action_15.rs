use crate::error::KittyMCError;
use crate::packets::packet_serialization::{read_varint_u32, SerializablePacket};
use crate::packets::Packet;
use kittymc_macros::Packet;

#[derive(PartialEq, Debug, Clone)]
pub enum EntityAction {
    StartSprinting,
    StopSprinting,
    LeaveBed,
    StartSneaking,
    StopSneaking,
    StartJumpingWithHorse,
    StopJumpingWithHorse,
    OpenHorseInventory,
    StartFlyingWithElytra,
    Unknown,
}

impl From<u32> for EntityAction {
    fn from(value: u32) -> Self {
        match value {
            0 => EntityAction::StartSneaking,
            1 => EntityAction::StopSneaking,
            2 => EntityAction::LeaveBed,
            3 => EntityAction::StartSprinting,
            4 => EntityAction::StopSprinting,
            5 => EntityAction::StartJumpingWithHorse,
            6 => EntityAction::StopJumpingWithHorse,
            7 => EntityAction::OpenHorseInventory,
            8 => EntityAction::StartFlyingWithElytra,
            _ => EntityAction::Unknown,
        }
    }
}

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct EntityActionPacket {
    pub entity_id: u32,
    pub action: EntityAction,
    pub jump_boost_amount: u32,
}

impl SerializablePacket for EntityActionPacket {
    fn deserialize(mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut size = 0;

        let entity_id = read_varint_u32(&mut data, &mut size)?;
        let action = read_varint_u32(&mut data, &mut size)?.into();
        let jump_boost_amount = read_varint_u32(&mut data, &mut size)?;

        Ok((
            size,
            Packet::EntityAction(EntityActionPacket {
                entity_id,
                action,
                jump_boost_amount,
            }),
        ))
    }

    fn id() -> u32 {
        0x15
    }
}
