use crate::error::KittyMCError;
use crate::packets::packet_serialization::{read_i16, SerializablePacket};
use crate::packets::Packet;
use kittymc_macros::Packet;
use crate::packets::client::play::window_items_14::SlotData;

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct CreativeInventoryActionPacket {
    pub slot: i16,
    pub clicked_item: SlotData
}

impl SerializablePacket for CreativeInventoryActionPacket {
    fn deserialize(mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut size = 0;

        let slot = read_i16(&mut data, &mut size)?;
        let clicked_item = SlotData::read(&mut data, &mut size)?;

        Ok((size, Packet::CreativeInventoryAction(CreativeInventoryActionPacket {
            slot,
            clicked_item
        })))
    }

    fn id() -> u32 {
        0x1B
    }
}
