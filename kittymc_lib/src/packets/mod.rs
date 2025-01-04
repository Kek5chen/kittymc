use integer_encoding::VarInt;
use kittymc_macros::PacketHelperFuncs;
use crate::error::KittyMCError;
use crate::packets::client::login::set_compression_03::SetCompressionPacket;
use crate::packets::client::login::success_02::LoginSuccessPacket;
use crate::packets::client::play::keep_alive_1f::KeepAlivePacket;
use crate::packets::client::play::plugin_message_18::PluginMessagePacket;
use crate::packets::client::status::response_00::StatusResponsePacket;
use crate::packets::packet_serialization::{decompress_packet, read_varint_u32, SerializablePacket};
use crate::packets::server::handshake::HandshakePacket;
use crate::packets::server::login::login_start_00::LoginStartPacket;
use crate::packets::server::status::ping_01::StatusPingPongPacket;
use crate::packets::server::status::request_00::StatusRequestPacket;
use crate::subtypes::state::State;
use crate::packets::packet_serialization::NamedPacket;

pub mod server;
pub mod client;
pub mod packet_serialization;

#[derive(Debug, Default)]
pub struct CompressionInfo {
    pub enabled: bool,
    pub compression_threshold: u32,
}

#[derive(PartialEq, Debug, Clone, PacketHelperFuncs)]
pub enum Packet {
    Handshake(HandshakePacket),
    LoginStart(LoginStartPacket),
    LoginSuccess(LoginSuccessPacket),
    StatusRequest(StatusRequestPacket),
    StatusResponse(StatusResponsePacket),
    StatusPing(StatusPingPongPacket),
    StatusPong(StatusPingPongPacket),
    KeepAlive(KeepAlivePacket),
    SetCompression(SetCompressionPacket),
    PluginMessage(PluginMessagePacket),
}

impl Packet {
    pub fn packet_id(&self) -> u32 {
        match self {
            Packet::Handshake(_) |
            Packet::LoginStart(_) |
            Packet::StatusRequest(_) |
            Packet::StatusResponse(_) |
            Packet::KeepAlive(_) => 0,

            Packet::StatusPing(_) |
            Packet::StatusPong(_) => 1,

            Packet::LoginSuccess(_) => 2,

            Packet::SetCompression(_) => 3,

            Packet::PluginMessage(_) => 0x17,
        }
    }

    pub fn deserialize_packet(state: State, raw_data: &[u8], compression: &CompressionInfo) -> Result<(usize, Packet), KittyMCError> {
        let threshold_hit = {
            let mut raw_data = raw_data;
            let mut size = 0;
            let data_len = read_varint_u32(&mut raw_data, &mut size)?;
            data_len >= compression.compression_threshold
        };

        let decompressed;
        let mut data: &[u8];
        if compression.enabled && threshold_hit {
            decompressed = decompress_packet(&raw_data)?;
            data = &decompressed[..];
        } else {
            data = raw_data;
        }

        let mut header_size = 0;
        let packet_len = read_varint_u32(&mut data, &mut header_size)? as usize;
        let packet_len = packet_len - header_size;
        let packet_id = read_varint_u32(&mut data, &mut header_size)? as usize;

        if packet_len > data.len() {
            return Err(KittyMCError::NotEnoughData(data.len(), packet_len));
        }

        let (packet_size, packet) = match state {
            State::Handshake => {
                match packet_id {
                    0 => HandshakePacket::deserialize(&data[..packet_len])?,
                    _ => return Err(KittyMCError::NotImplemented),
                }
            }
            State::Status => {
                match packet_id {
                    0 => StatusRequestPacket::deserialize(&data[..packet_len])?,
                    1 => StatusPingPongPacket::deserialize(&data[..packet_len])?,
                    _ => return Err(KittyMCError::NotImplemented),
                }
            }
            State::Login => {
                match packet_id {
                    0 => LoginStartPacket::deserialize(&data[..packet_len])?,
                    _ => return Err(KittyMCError::NotImplemented),
                }
            }
            State::Play => {
                match packet_id {
                    0 => KeepAlivePacket::deserialize(&data[..packet_len])?,
                    0x17 => PluginMessagePacket::deserialize(&data[..packet_len])?,
                    _ => return Err(KittyMCError::NotImplemented),
                }
            }
            _ => return Err(KittyMCError::NotImplemented),
        };

        Ok((header_size + packet_size, packet))
    }
}

pub fn wrap_packet(packet: &mut Vec<u8>, id: u32) {
    // add packet id
    packet.splice(0..0, id.encode_var_vec());

    // set total length
    let total_len = packet.len().encode_var_vec();
    packet.splice(0..0, total_len);
}