use crate::packets::packet_serialization::{
    write_bool, write_f32, write_length_prefixed_string, write_nbt, write_u8, write_varint_u32,
};
use crate::packets::server::play::client_settings_04::DisplayedSkinParts;
use bitflags::bitflags;
use std::collections::HashMap;

bitflags! {
    #[derive(PartialEq, Eq, Debug, Copy, Clone)]
    pub struct EntityMetaState : u8 {
        const on_fire        = 0x01;
        const crouched       = 0x02;
        const unused         = 0x04; // previously riding
        const sprinting      = 0x08; // previously riding
        const unused_2       = 0x10; // previously eating/drinking/blocking (use hand state now)
        const invisible      = 0x20; // previously eating/drinking/blocking (use hand state now)
        const glowing_effect = 0x40;
        const elytra_flying  = 0x80;
    }

    #[derive(PartialEq, Eq, Debug, Copy, Clone)]
    pub struct LivingHandState : u8 {
        const is_hand_active = 0x01;
        const active_hand    = 0x02; // 0 = main hand, 1 = offhand
    }
}

impl EntityMetaState {
    pub fn write(&self, buffer: &mut Vec<u8>) {
        write_u8(buffer, self.bits());
    }
}

impl LivingHandState {
    pub fn write(&self, buffer: &mut Vec<u8>) {
        write_u8(buffer, self.bits());
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct EntityMetadata {
    pub meta_state: EntityMetaState,
    pub air: u32,
    pub custom_name: String,
    pub is_custom_name_visible: bool,
    pub is_silent: bool,
    pub no_gravity: bool,
}

impl Default for EntityMetadata {
    fn default() -> Self {
        EntityMetadata {
            meta_state: EntityMetaState::empty(),
            air: 300,
            custom_name: "".to_string(),
            is_custom_name_visible: false,
            is_silent: false,
            no_gravity: false,
        }
    }
}

impl EntityMetadata {
    pub fn write(&self, buffer: &mut Vec<u8>) {
        self.meta_state.write(buffer);
        write_varint_u32(buffer, self.air);
        write_length_prefixed_string(buffer, &self.custom_name);
        write_bool(buffer, self.is_custom_name_visible);
        write_bool(buffer, self.is_silent);
        write_bool(buffer, self.no_gravity);
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct LivingMetadata {
    pub entity: EntityMetadata,
    pub hand_state: LivingHandState,
    pub health: f32,
    pub potion_effect_color: u32,
    pub is_potion_effect_ambient: bool,
    pub number_of_arrows_in_entity: u32,
}

impl Default for LivingMetadata {
    fn default() -> Self {
        LivingMetadata {
            entity: EntityMetadata::default(),
            hand_state: LivingHandState::empty(),
            health: 1.0,
            potion_effect_color: 0,
            is_potion_effect_ambient: false,
            number_of_arrows_in_entity: 0,
        }
    }
}

impl LivingMetadata {
    pub fn write(&self, buffer: &mut Vec<u8>) {
        self.entity.write(buffer);
        self.hand_state.write(buffer);
        write_f32(buffer, self.health);
        write_varint_u32(buffer, self.potion_effect_color);
        write_bool(buffer, self.is_potion_effect_ambient);
        write_varint_u32(buffer, self.number_of_arrows_in_entity);
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct PlayerMetadata {
    pub living: LivingMetadata,
    pub additional_hearts: f32,
    pub score: u32,
    pub displayed_skin_parts: DisplayedSkinParts,
    pub main_hand: u8,
    pub left_shoulder_entity_data: fastnbt::Value, // for occupying parrot // TODO: NBT
    pub right_shoulder_entity_data: fastnbt::Value, // for occupying parrot // TODO: NBT
}

impl Default for PlayerMetadata {
    fn default() -> Self {
        PlayerMetadata {
            living: LivingMetadata::default(),
            additional_hearts: 0.,
            score: 0,
            displayed_skin_parts: DisplayedSkinParts::empty(),
            main_hand: 1,
            left_shoulder_entity_data: fastnbt::Value::Compound(HashMap::new()),
            right_shoulder_entity_data: fastnbt::Value::Compound(HashMap::new()),
        }
    }
}

impl PlayerMetadata {
    pub fn write(&self, buffer: &mut Vec<u8>) {
        self.living.write(buffer);
        write_f32(buffer, self.additional_hearts);
        write_varint_u32(buffer, self.score);
        self.displayed_skin_parts.write(buffer);
        write_u8(buffer, self.main_hand);
        write_nbt(buffer, &self.left_shoulder_entity_data);
        write_nbt(buffer, &self.right_shoulder_entity_data);
    }
}
