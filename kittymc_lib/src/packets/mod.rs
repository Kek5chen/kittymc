use integer_encoding::VarInt;
use kittymc_macros::PacketHelperFuncs;
use log::{trace, warn};
use crate::error::KittyMCError;
use crate::packets::client::login::set_compression_03::SetCompressionPacket;
use crate::packets::client::login::success_02::LoginSuccessPacket;
use crate::packets::client::status::response_00::StatusResponsePacket;
use crate::packets::packet_serialization::{decompress_packet, read_varint_u32, write_varint_u32_splice, SerializablePacket};
use crate::packets::server::handshake::HandshakePacket;
use crate::packets::server::login::login_start_00::LoginStartPacket;
use crate::packets::server::status::ping_01::StatusPingPongPacket;
use crate::packets::server::status::request_00::StatusRequestPacket;
use crate::subtypes::state::State;
use crate::packets::packet_serialization::NamedPacket;
use crate::packets::server::play::client_held_item_change_a1::ClientHeldItemChangePacket;
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
    ClientHeldItemChange(ClientHeldItemChangePacket),
}

impl Packet {
    pub fn deserialize_compressed(state: State, rawr_data: &[u8], compression: &CompressionInfo) -> Result<(usize, Packet), KittyMCError> {
        let mut data_part = rawr_data;
        let mut full_size;
        let uncompressed_data_len;
        let uncompressed_packet_len;

        let is_packet_compressed = {
            let mut size = 0;
            full_size = read_varint_u32(&mut data_part, &mut size)? as usize;
            full_size += size;

            size = 0;
            uncompressed_data_len = read_varint_u32(&mut data_part, &mut size)? as usize;
            uncompressed_packet_len = uncompressed_data_len + size;

            uncompressed_data_len != 0 && uncompressed_data_len >= compression.compression_threshold as usize
        };

        let mut decompressed;
        let data;

        if is_packet_compressed {
            let (size, owned_data) = decompress_packet(&rawr_data)?;
            decompressed = owned_data;
            write_varint_u32_splice(&mut decompressed, uncompressed_data_len as u32, ..0);
            trace!("Complete Uncompressed Packet : {:?}", decompressed);

            if full_size != size {
                warn!("Handled size of decompression function should be equal to deserialize_compressed functions size. Using decompression size.");
                full_size = size;
            }

            data = &decompressed[..];
        } else {
            data = data_part;
        }

        let (decompressed_size, packet) = match Self::deserialize_uncompressed(state, data) {
            Ok(p) => Ok(p),
            Err(KittyMCError::NotImplemented(id, _len)) => { Err(KittyMCError::NotImplemented(id, full_size)) } // Replace the length with the compressed length
            Err(e) => Err(e)
        }?;

        if decompressed_size != uncompressed_packet_len {
            return Err(KittyMCError::InvalidDecompressedPacketLength);
        }

        Ok((full_size, packet))
    }

    pub fn deserialize_uncompressed(state: State, mut data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        let mut header_size = 0;
        let packet_data_and_id_len = read_varint_u32(&mut data, &mut header_size)? as usize;
        let full_packet_len = packet_data_and_id_len + header_size;

        if packet_data_and_id_len > data.len() {
            trace!("Not enough data yet. Packet length: {}. Current Data length: {}", packet_data_and_id_len, data.len());
            return Err(KittyMCError::NotEnoughData(data.len(), packet_data_and_id_len));
        }

        let packet_id = read_varint_u32(&mut data, &mut header_size)? as usize;
        let Some(packet_data_len) = full_packet_len.checked_sub(header_size) else {
            return Err(KittyMCError::InvalidPacketLength);
        };

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
                    0x1A => ClientHeldItemChangePacket::deserialize(&data[..packet_data_len])?,
                    _ => return Err(KittyMCError::NotImplemented(packet_id, full_packet_len)),
                }
            }
            _ => return Err(KittyMCError::NotImplemented(packet_id, full_packet_len)),
        };

        let full_size = header_size + packet_size;
        Ok((full_size, packet))
    }

    pub fn deserialize(state: State, raw_data: &[u8], compression: &CompressionInfo) -> Result<(usize, Packet), KittyMCError> {
        if compression.enabled {
            Self::deserialize_compressed(state, raw_data, compression)
        } else {
            Self::deserialize_uncompressed(state, raw_data)
        }
    }
}

pub fn wrap_packet(packet: &mut Vec<u8>, id: u32) {
    // add packet id
    packet.splice(0..0, id.encode_var_vec());

    // set total length
    let total_len = packet.len().encode_var_vec();
    packet.splice(0..0, total_len);
}