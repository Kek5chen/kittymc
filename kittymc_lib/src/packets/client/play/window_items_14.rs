use kittymc_macros::Packet;
use crate::packets::packet_serialization::{write_i16, write_u16, write_u8, SerializablePacket};
use crate::packets::wrap_packet;

#[derive(PartialEq, Debug, Clone)]
pub struct SlotData {
    block_id: u16, // 0xffff is empty
    item_count: u8,
    item_damage: u16,
    nbt: Option<()> // TODO: NBT STUFF
}

impl Default for SlotData {
    fn default() -> Self {
        SlotData {
            block_id: u16::MAX,
            item_count: 0,
            item_damage: 0,
            nbt: None,
        }
    }
}

impl SlotData {
    pub fn write(&self, data: &mut Vec<u8>) {
        write_u16(data, self.block_id);
        if self.block_id != u16::MAX {
            write_u8(data, self.item_count);
            write_u16(data, self.item_damage);
        }
    }
}

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct WindowItemsPacket {
    window_id: u8,
    slot_data: Vec<SlotData>
}

impl Default for WindowItemsPacket {
    fn default() -> Self {
        WindowItemsPacket {
            window_id: 0,
            slot_data: vec![SlotData::default(); 45]
        }
    }
}

impl SerializablePacket for WindowItemsPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_u8(&mut packet, self.window_id);
        write_i16(&mut packet, self.slot_data.len() as i16);
        for slot in &self.slot_data {
            slot.write(&mut packet);
        }

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        0x14
    }
}
