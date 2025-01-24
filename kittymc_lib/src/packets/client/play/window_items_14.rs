use crate::packets::packet_serialization::{read_nbt, read_u16, read_u8, write_i16, write_u16, write_u8, SerializablePacket};
use crate::packets::wrap_packet;
use kittymc_macros::Packet;
use crate::error::KittyMCError;

#[derive(PartialEq, Debug, Clone)]
pub struct SlotData {
    pub id: u16, // 0xffff is empty
    pub item_count: u8,
    pub item_damage: u16,
    pub nbt: Option<fastnbt::Value>,
}

impl Default for SlotData {
    fn default() -> Self {
        SlotData {
            id: u16::MAX,
            item_count: 0,
            item_damage: 0,
            nbt: None,
        }
    }
}

impl SlotData {
    pub fn write(&self, data: &mut Vec<u8>) {
        write_u16(data, self.id);
        if self.id != u16::MAX {
            write_u8(data, self.item_count);
            write_u16(data, self.item_damage);
        }
    }

    pub fn read(data: &mut &[u8], size: &mut usize) -> Result<Self, KittyMCError> {
        let block_id = read_u16(data, size)?;
        let mut item_count: u8 = 0;
        let mut item_damage: u16 = 0;
        if block_id != u16::MAX {
            item_count = read_u8(data, size)?;
            item_damage = read_u16(data, size)?;
        }
        let nbt = read_nbt(data, size).ok();

        Ok(Self {
            id: block_id,
            item_count,
            item_damage,
            nbt,
        })
    }
}

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct WindowItemsPacket {
    window_id: u8,
    slot_data: Vec<SlotData>,
}

impl Default for WindowItemsPacket {
    fn default() -> Self {
        WindowItemsPacket {
            window_id: 0,
            slot_data: vec![SlotData::default(); 45],
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
