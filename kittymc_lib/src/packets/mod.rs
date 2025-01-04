use integer_encoding::VarInt;
use kittymc_macros::PacketHelperFuncs;
use log::{warn};
use crate::error::KittyMCError;
use crate::packets::client::login::set_compression_03::SetCompressionPacket;
use crate::packets::client::login::success_02::LoginSuccessPacket;
use crate::packets::client::status::response_00::StatusResponsePacket;
use crate::packets::packet_serialization::{decompress_packet, read_varint_u32, SerializablePacket};
use crate::packets::server::handshake::HandshakePacket;
use crate::packets::server::login::login_start_00::LoginStartPacket;
use crate::packets::server::status::ping_01::StatusPingPongPacket;
use crate::packets::server::status::request_00::StatusRequestPacket;
use crate::subtypes::state::State;
use crate::packets::packet_serialization::NamedPacket;
use crate::packets::server::play::client_keep_alive_0b::ClientKeepAlivePacket;
use crate::packets::server::play::client_player_position_and_look_0e::ClientPlayerPositionAndLookPacket;
use crate::packets::server::play::client_settings_04::ClientSettingsPacket;
use crate::packets::server::play::client_plugin_message_09::ClientPluginMessagePacket;
use crate::packets::server::play::teleport_confirm_00::TeleportConfirmPacket;

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
    KeepAlive(ClientKeepAlivePacket),
    SetCompression(SetCompressionPacket),
    PluginMessage(ClientPluginMessagePacket),
    ClientSettings(ClientSettingsPacket),
    TeleportConfirm(TeleportConfirmPacket),
    PlayerPositionAndLook(ClientPlayerPositionAndLookPacket),
}

impl Packet {
    pub fn deserialize_packet(state: State, raw_data: &[u8], compression: &CompressionInfo) -> Result<(usize, Packet), KittyMCError> {
        let mut data_part = raw_data;
        let threshold_hit = {
            let mut size = 0;
            let _packet_length = read_varint_u32(&mut data_part, &mut size)?;
            let data_len = read_varint_u32(&mut data_part, &mut size)?;
            data_len != 0 && data_len >= compression.compression_threshold
        };

        let decompressed;
        let mut data: &[u8];
        if compression.enabled {
            if threshold_hit {
                decompressed = decompress_packet(&raw_data)?;
                data = &decompressed[..];
            } else {
                data = data_part;
            }
        } else {
            data = raw_data;
        }

        let mut header_size = 0;
        let packet_data_and_id_len = read_varint_u32(&mut data, &mut header_size)? as usize;
        let full_packet_len = packet_data_and_id_len + header_size;
        let packet_data_len = packet_data_and_id_len - header_size;
        let packet_id = read_varint_u32(&mut data, &mut header_size)? as usize;

        if packet_data_len > data.len() {
            warn!("Packet length was bigger than data length. Waiting for more data");
            return Err(KittyMCError::NotEnoughData(data.len(), packet_data_len));
        }

        // TODO: Macro-ize this
        let (packet_size, packet) = match state {
            State::Handshake => {
                match packet_id {
                    0 => HandshakePacket::deserialize(&data[..packet_data_len])?,
                    _ => return Err(KittyMCError::NotImplemented(packet_id, full_packet_len)),
                }
            }
            State::Status => {
                match packet_id {
                    0 => StatusRequestPacket::deserialize(&data[..packet_data_len])?,
                    1 => StatusPingPongPacket::deserialize(&data[..packet_data_len])?,
                    _ => return Err(KittyMCError::NotImplemented(packet_id, full_packet_len)),
                }
            }
            State::Login => {
                match packet_id {
                    0 => LoginStartPacket::deserialize(&data[..packet_data_len])?,
                    _ => return Err(KittyMCError::NotImplemented(packet_id, full_packet_len)),
                }
            }
            State::Play => {
                match packet_id {
                    0 => TeleportConfirmPacket::deserialize(&data[..packet_data_len])?,
                    4 => ClientSettingsPacket::deserialize(&data[..packet_data_len])?,
                    9 => ClientPluginMessagePacket::deserialize(&data[..packet_data_len])?,
                    0xB => ClientKeepAlivePacket::deserialize(&data[..packet_data_len])?,
                    0xE => ClientPlayerPositionAndLookPacket::deserialize(&data[..packet_data_len])?,
                    _ => return Err(KittyMCError::NotImplemented(packet_id, full_packet_len)),
                }
            }
            _ => return Err(KittyMCError::NotImplemented(packet_id, full_packet_len)),
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