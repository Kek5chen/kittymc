use kittymc_macros::Packet;
use crate::packets::packet_serialization::{write_bool, write_length_prefixed_string, write_varint_u32, SerializablePacket};
use crate::packets::wrap_packet;
use crate::packets::client::play::GameMode;
use crate::subtypes::Chat;

#[derive(PartialEq, Debug, Clone)]
pub struct PlayerListItemProperties {
    name: String,
    value: String,
    // is_signed: bool, // determined by signature Optional
    signature: Option<String>,
}

impl PlayerListItemProperties {
    pub fn write(&self, buffer: &mut Vec<u8>) {
        write_length_prefixed_string(buffer, &self.name);
        write_length_prefixed_string(buffer, &self.name);
        write_bool(buffer, self.signature.is_some());
        if let Some(sig) = &self.signature {
            write_bool(buffer, true);
            write_length_prefixed_string(buffer, sig);
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum PlayerListItemAction {
    AddPlayer {
        name: String,
        properties: Vec<PlayerListItemProperties>,
        game_mode: GameMode,
        ping: u32,
        // has_display_name: bool, // determined by display_name Optional
        display_name: Option<Chat>,
    },
    UpdateGameMode(GameMode),
    UpdateLatency(u32),
    UpdateDisplayName(Option<Chat>),
    RemovePlayer
}

impl PlayerListItemAction {
    pub fn id(&self) -> u32 {
        match self {
            PlayerListItemAction::AddPlayer { .. } => 0,
            PlayerListItemAction::UpdateGameMode(_) => 1,
            PlayerListItemAction::UpdateLatency(_) => 2,
            PlayerListItemAction::UpdateDisplayName(_) => 3,
            PlayerListItemAction::RemovePlayer => 4,
        }
    }

    pub fn write(&self, buffer: &mut Vec<u8>) {
        match self {
            PlayerListItemAction::AddPlayer {
                name,
                properties,
                game_mode,
                ping,
                display_name
            } => {
                write_length_prefixed_string(buffer, name);
                write_varint_u32(buffer, properties.len() as u32);
                for property in properties {
                    property.write(buffer);
                }
                write_varint_u32(buffer, *game_mode as u32);
                write_varint_u32(buffer, *ping);
                write_bool(buffer, display_name.is_some());
                if let Some(display) = display_name {
                    display.write(buffer);
                }
            }
            PlayerListItemAction::UpdateGameMode(game_mode) => {
                write_varint_u32(buffer, *game_mode as u32);
            }
            PlayerListItemAction::UpdateLatency(latency) => {
                write_varint_u32(buffer, *latency);
            }
            PlayerListItemAction::UpdateDisplayName(display_name) => {
                write_bool(buffer, display_name.is_some());
                if let Some(display) = display_name {
                    display.write(buffer);
                }
            }
            PlayerListItemAction::RemovePlayer => ()
        }
    }
}

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct PlayerListItemPacket {
    actions: Vec<PlayerListItemAction>,
}

impl Default for PlayerListItemPacket {
    fn default() -> Self {
        PlayerListItemPacket { actions: vec![] }
    }
}

impl SerializablePacket for PlayerListItemPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        if !self.actions.is_empty() {
            let first = &self.actions[0];
            for action in &self.actions {
                if first.id() != action.id() {
                    return vec![3, 0x2E, 0, 0];
                }
            }
        } else {
            return vec![3, 0x2E, 0, 0];
        }

        write_varint_u32(&mut packet, self.actions[0].id());
        write_varint_u32(&mut packet, self.actions.len() as u32);
        for action in &self.actions {
            action.write(&mut packet);
        }

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        0x2E
    }
}
