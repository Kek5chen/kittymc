use crate::error::KittyMCError;
use crate::packets::packet_serialization::{
    read_block_location, read_u8, read_varint_u32, write_block_location, write_u8,
    write_varint_u32, SerializablePacket,
};
use crate::packets::{wrap_packet, Packet};
use crate::subtypes::Location;
use kittymc_macros::Packet;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum PlayerDiggingStatus {
    StartedDigging,
    CancelledDigging,
    FinishedDigging,
    DropItemStack,
    DropItem,
    ShootArrowFishEating,
    SwapItemInHand,
    Unknown,
}

impl From<u32> for PlayerDiggingStatus {
    fn from(value: u32) -> Self {
        match value {
            0 => PlayerDiggingStatus::StartedDigging,
            1 => PlayerDiggingStatus::CancelledDigging,
            2 => PlayerDiggingStatus::FinishedDigging,
            3 => PlayerDiggingStatus::DropItemStack,
            4 => PlayerDiggingStatus::DropItem,
            5 => PlayerDiggingStatus::ShootArrowFishEating,
            6 => PlayerDiggingStatus::SwapItemInHand,
            _ => PlayerDiggingStatus::Unknown,
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum BlockFace {
    Top,
    Bottom,
    North,
    South,
    West,
    East,
    Unknown,
}

impl From<u8> for BlockFace {
    fn from(value: u8) -> Self {
        match value {
            0 => BlockFace::Top,
            1 => BlockFace::Bottom,
            2 => BlockFace::North,
            3 => BlockFace::South,
            4 => BlockFace::West,
            5 => BlockFace::East,
            _ => BlockFace::Unknown,
        }
    }
}

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct PlayerDiggingPacket {
    pub status: PlayerDiggingStatus,
    pub location: Location,
    pub face: BlockFace,
}

impl SerializablePacket for PlayerDiggingPacket {
    fn deserialize(mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut size = 0;

        let status = read_varint_u32(&mut data, &mut size)?.into();
        let location = read_block_location(&mut data, &mut size)?;
        let face = read_u8(&mut data, &mut size)?.into();

        Ok((
            size,
            Packet::PlayerDigging(PlayerDiggingPacket {
                status,
                location,
                face,
            }),
        ))
    }

    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_varint_u32(&mut packet, self.status as u32);
        write_block_location(&mut packet, &self.location);
        write_u8(&mut packet, self.face as u8);

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        0x14
    }
}
