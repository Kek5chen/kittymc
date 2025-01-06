use crate::packets::packet_serialization::{write_i8, SerializablePacket};
use crate::packets::wrap_packet;
use crate::subtypes::components::Component;
use kittymc_macros::Packet;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ChatPosition {
    Chat = 0,
    System = 1,
    Hotbar = 2,
}

#[derive(Debug, Packet)]
pub struct ClientChatMessagePacket {
    text: Component,
    position: ChatPosition,
}

impl ClientChatMessagePacket {
    pub fn new_join_message(name: &str) -> Self {
        ClientChatMessagePacket {
            text: Component::default_join(name),
            position: ChatPosition::Chat,
        }
    }

    pub fn new_quit_message(name: &str) -> Self {
        ClientChatMessagePacket {
            text: Component::default_quit(name),
            position: ChatPosition::Chat,
        }
    }

    pub fn new_chat_message(name: &str, message: &str) -> Self {
        ClientChatMessagePacket {
            text: Component::default_chat(name, message),
            position: ChatPosition::Chat,
        }
    }
}

impl SerializablePacket for ClientChatMessagePacket {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        // write_length_prefixed_string(
        //     &mut packet,
        //     &serde_json::to_string(&json!({
        //         "text": self.text
        //     }))
        //     .unwrap(),
        // );

        self.text.write(&mut packet);
        write_i8(&mut packet, self.position as i8);

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        0x0F
    }
}
