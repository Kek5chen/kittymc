use crate::packets::packet_serialization::{write_bool, write_varint_u32, SerializablePacket};
use crate::packets::wrap_packet;
use kittymc_macros::Packet;

#[repr(u32)]
#[derive(PartialEq, Debug, Clone, Copy, Packet)]
pub enum UnlockAction {
    Init = 0,
    Add = 1,
    Remove = 2,
}

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct UnlockRecipesPacket {
    action: UnlockAction,
    crafting_book_open: bool,
    filtering_craftable: bool,
    recipe_ids: Vec<u32>,
    recipe_ids_2: Vec<u32>,
}

impl Default for UnlockRecipesPacket {
    fn default() -> Self {
        UnlockRecipesPacket {
            action: UnlockAction::Init,
            crafting_book_open: false,
            filtering_craftable: false,
            recipe_ids: vec![],
            recipe_ids_2: vec![],
        }
    }
}

impl SerializablePacket for UnlockRecipesPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_varint_u32(&mut packet, self.action as u32);
        write_bool(&mut packet, self.crafting_book_open);
        write_bool(&mut packet, self.filtering_craftable);

        write_varint_u32(&mut packet, self.recipe_ids.len() as u32);
        for id in &self.recipe_ids {
            write_varint_u32(&mut packet, *id);
        }

        write_varint_u32(&mut packet, self.recipe_ids_2.len() as u32);
        for id in &self.recipe_ids_2 {
            write_varint_u32(&mut packet, *id);
        }

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        0x31
    }
}
