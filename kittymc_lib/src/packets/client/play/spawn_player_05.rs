use crate::packets::packet_serialization::{
    write_direction, write_location2, write_u32, write_uuid, SerializablePacket,
};
use crate::packets::wrap_packet;
use crate::subtypes::metadata::PlayerMetadata;
use crate::subtypes::{Direction, Location2};
use kittymc_macros::Packet;
use uuid::Uuid;

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct SpawnPlayerPacket {
    pub entity_id: u32,
    pub player_uuid: Uuid,
    pub location: Location2,
    pub direction: Direction,
    pub metadata: PlayerMetadata, // TODO: Metadata
}

impl SerializablePacket for SpawnPlayerPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_u32(&mut packet, self.entity_id);
        write_uuid(&mut packet, &self.player_uuid);
        write_location2(&mut packet, &self.location);
        write_direction(&mut packet, &self.direction);

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        5
    }
}
