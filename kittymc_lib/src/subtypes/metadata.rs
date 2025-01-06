use crate::packets::client::play::window_items_14::SlotData;
use crate::packets::packet_serialization::{
    write_bool, write_direction, write_f32, write_length_prefixed_string, write_location2,
    write_nbt, write_rotation, write_u8, write_uuid, write_varint_u32,
};
use crate::packets::server::play::client_settings_04::DisplayedSkinParts;
use crate::subtypes::components::Component;
use crate::subtypes::{Direction, Location2, Rotation};
use bitflags::bitflags;
use std::collections::{BTreeMap, HashMap};
use typed_builder::TypedBuilder;
use uuid::Uuid;

pub enum MetaData {
    Byte(u8),
    VarInt(u32),
    Float(f32),
    String(String),
    Chat(Component),
    Slot(SlotData),
    Boolean(bool),
    Rotation(Rotation),
    Position(Location2),
    OptPosition(Option<Location2>), // Boolean + Optional Position
    Direction(Direction),           // varints
    OptUuid(Option<Uuid>),          // Boolean + Optional Uuid
    OptBlockId(Option<u32>),        // Boolean + Optional VarInt
    NBTTag(fastnbt::Value),
}

impl MetaData {
    pub fn type_id(&self) -> u8 {
        match self {
            MetaData::Byte(_) => 0,
            MetaData::VarInt(_) => 1,
            MetaData::Float(_) => 2,
            MetaData::String(_) => 3,
            MetaData::Chat(_) => 4,
            MetaData::Slot(_) => 5,
            MetaData::Boolean(_) => 6,
            MetaData::Rotation(_) => 7,
            MetaData::Position(_) => 8,
            MetaData::OptPosition(_) => 9,
            MetaData::Direction(_) => 10,
            MetaData::OptUuid(_) => 11,
            MetaData::OptBlockId(_) => 12,
            MetaData::NBTTag(_) => 13,
        }
    }

    pub fn write(&self, buffer: &mut Vec<u8>) {
        match self {
            MetaData::Byte(b) => write_u8(buffer, *b),
            MetaData::VarInt(i) => write_varint_u32(buffer, *i),
            MetaData::Float(f) => write_f32(buffer, *f),
            MetaData::String(s) => write_length_prefixed_string(buffer, s),
            MetaData::Chat(c) => c.write(buffer),
            MetaData::Slot(s) => s.write(buffer),
            MetaData::Boolean(b) => write_bool(buffer, *b),
            MetaData::Rotation(rotation) => write_rotation(buffer, rotation),
            MetaData::Position(pos) => write_location2(buffer, pos),
            MetaData::OptPosition(pos) => {
                write_bool(buffer, pos.is_some());
                if let Some(pos) = pos {
                    write_location2(buffer, pos);
                }
            }
            MetaData::Direction(dir) => write_direction(buffer, dir),
            MetaData::OptUuid(uuid) => {
                write_bool(buffer, uuid.is_some());
                if let Some(uuid) = uuid {
                    write_uuid(buffer, uuid);
                }
            }
            MetaData::OptBlockId(block_id) => {
                write_bool(buffer, block_id.is_some());
                if let Some(block_id) = block_id {
                    write_varint_u32(buffer, *block_id);
                }
            }
            MetaData::NBTTag(tag) => write_nbt(buffer, tag),
        }
    }
}

pub fn write_metadata(buffer: &mut Vec<u8>, meta_data: &BTreeMap<u8, MetaData>) {
    for (index, value) in meta_data {
        write_u8(buffer, *index);
        write_u8(buffer, value.type_id());
        value.write(buffer);
    }
    write_u8(buffer, 0xFF);
}

bitflags! {
    #[derive(PartialEq, Eq, Debug, Copy, Clone)]
    pub struct EntityMetaState : u8 {
        const on_fire        = 0x01;
        const crouched       = 0x02;
        const unused         = 0x04; // previously riding
        const sprinting      = 0x08;
        const unused_2       = 0x10;
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
    pub fn write_to_metadata(&self, meta_data: &mut BTreeMap<u8, MetaData>, index: u8) {
        meta_data.insert(index, MetaData::Byte(self.bits()));
    }
}

impl LivingHandState {
    pub fn write_to_metadata(&self, meta_data: &mut BTreeMap<u8, MetaData>, index: u8) {
        meta_data.insert(index, MetaData::Byte(self.bits()));
    }
}

#[derive(PartialEq, Debug, Clone, TypedBuilder)]
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
    pub fn write_to_metadata(&self, mut meta_data: &mut BTreeMap<u8, MetaData>) {
        self.meta_state.write_to_metadata(&mut meta_data, 0);
        meta_data.insert(1, MetaData::VarInt(self.air));
        meta_data.insert(2, MetaData::String(self.custom_name.clone()));
        meta_data.insert(3, MetaData::Boolean(self.is_custom_name_visible));
        meta_data.insert(4, MetaData::Boolean(self.is_silent));
        meta_data.insert(5, MetaData::Boolean(self.no_gravity));
    }

    pub fn write_metadata(&self, buffer: &mut Vec<u8>) {
        let mut metadata = BTreeMap::new();
        self.write_to_metadata(&mut metadata);
        write_metadata(buffer, &metadata);
    }
}

#[derive(PartialEq, Debug, Clone, TypedBuilder)]
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
    pub fn write_to_metadata(&self, mut meta_data: &mut BTreeMap<u8, MetaData>) {
        self.entity.write_to_metadata(&mut meta_data);
        self.hand_state.write_to_metadata(&mut meta_data, 6);
        meta_data.insert(6, MetaData::Float(self.health));
        meta_data.insert(7, MetaData::VarInt(self.potion_effect_color));
        meta_data.insert(8, MetaData::Boolean(self.is_potion_effect_ambient));
        meta_data.insert(9, MetaData::VarInt(self.number_of_arrows_in_entity));
    }

    pub fn write_metadata(&self, buffer: &mut Vec<u8>) {
        let mut metadata = BTreeMap::new();
        self.write_to_metadata(&mut metadata);
        write_metadata(buffer, &metadata);
    }
}

#[derive(PartialEq, Debug, Clone, TypedBuilder)]
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
    pub fn write_to_metadata(&self, mut meta_data: &mut BTreeMap<u8, MetaData>) {
        self.living.write_to_metadata(&mut meta_data);
        meta_data.insert(11, MetaData::Float(self.additional_hearts));
        meta_data.insert(12, MetaData::VarInt(self.score));
        meta_data.insert(13, MetaData::Byte(self.displayed_skin_parts.bits()));
        meta_data.insert(14, MetaData::Byte(self.main_hand));
        meta_data.insert(15, MetaData::NBTTag(self.left_shoulder_entity_data.clone()));
        meta_data.insert(
            16,
            MetaData::NBTTag(self.right_shoulder_entity_data.clone()),
        );
    }

    pub fn write_metadata(&self, buffer: &mut Vec<u8>) {
        let mut metadata = BTreeMap::new();
        self.write_to_metadata(&mut metadata);
        write_metadata(buffer, &metadata);
    }
}
