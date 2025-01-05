use crate::packets::packet_serialization::{
    write_i8, write_length_prefixed_string, SerializablePacket,
};
use crate::packets::wrap_packet;
use kittymc_macros::Packet;
use serde_json::json;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ChatPosition {
    Chat = 0,
    System = 1,
    Hotbar = 2,
}

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct ChatMessagePacket {
    text: String,
    position: ChatPosition,
}

impl Default for ChatMessagePacket {
    fn default() -> Self {
        ChatMessagePacket {
            text: "Meow".to_string(),
            position: ChatPosition::Chat,
        }
    }
}

impl ChatMessagePacket {
    pub fn new_join_message(name: &str) -> Self {
        ChatMessagePacket {
            text: format!("{} joined the game :3", name),
            position: ChatPosition::Chat,
        }
    }
}

impl SerializablePacket for ChatMessagePacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_length_prefixed_string(
            &mut packet,
            &serde_json::to_string(&json!({
                "text": self.text
            }))
            .unwrap(),
        );
        write_i8(&mut packet, self.position as i8);

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        0x0F
    }
}
