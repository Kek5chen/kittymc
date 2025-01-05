use crate::error::KittyMCError;
use crate::packets::client::login::*;
use crate::packets::client::status::*;
use crate::packets::packet_serialization::NamedPacket;
use crate::packets::packet_serialization::{
    decompress_packet, read_varint_u32, write_varint_u32_splice, SerializablePacket,
};
use crate::packets::server::handshake::*;
use crate::packets::server::login::*;
use crate::packets::server::play::*;
use crate::packets::server::status::*;
use crate::subtypes::state::State;
use integer_encoding::VarInt;
use kittymc_macros::PacketHelperFuncs;
use log::{trace, warn};

pub mod client;
pub mod packet_serialization;
pub mod server;

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
    PlayerPosition(PlayerPositionPacket),
    PlayerLook(PlayerLookPacket),
    ClientAnimation(ClientAnimationPacket),
    ChatMessage(ServerChatMessagePacket),
}

impl Packet {
    pub fn deserialize_compressed(
        state: State,
        rawr_data: &[u8],
        compression: &CompressionInfo,
    ) -> Result<(usize, Packet), KittyMCError> {
        let mut data_part = rawr_data;
        let mut compressed_packet_len;
        let compressed_packet_data_len;
        let mut compressed_packet_len_len = 0;
        let mut uncompressed_data_len_len = 0;
        let uncompressed_packet_data_len;
        let uncompressed_data_len;

        let is_packet_compressed = {
            compressed_packet_data_len =
                read_varint_u32(&mut data_part, &mut compressed_packet_len_len)? as usize;
            compressed_packet_len = compressed_packet_data_len + compressed_packet_len_len;

            uncompressed_data_len =
                read_varint_u32(&mut data_part, &mut uncompressed_data_len_len)? as usize;
            uncompressed_data_len != 0
                && uncompressed_data_len >= compression.compression_threshold as usize
        };

        uncompressed_packet_data_len = match is_packet_compressed {
            true => uncompressed_data_len + uncompressed_data_len_len,
            false => compressed_packet_data_len - uncompressed_data_len_len,
        };

        let mut b_packet;

        if is_packet_compressed {
            let (size, owned_data) = decompress_packet(&rawr_data)?;
            b_packet = owned_data;
            write_varint_u32_splice(&mut b_packet, uncompressed_data_len as u32, ..0);
            trace!("Complete Uncompressed Packet : {:?}", b_packet);

            if compressed_packet_len != size {
                warn!("Handled size of decompression function should be equal to deserialize_compressed functions size. Using decompression size.");
                compressed_packet_len = size;
            }
        } else {
            b_packet = data_part[..uncompressed_packet_data_len].to_vec();
            write_varint_u32_splice(&mut b_packet, uncompressed_packet_data_len as u32, ..0);
        }

        let (decompressed_size, packet) = match Self::deserialize_uncompressed(state, &b_packet) {
            Ok(p) => Ok(p),
            Err(KittyMCError::NotImplemented(id, _len)) => {
                Err(KittyMCError::NotImplemented(id, compressed_packet_len))
            } // Replace the length with the compressed length
            Err(e) => Err(e),
        }?;

        if b_packet.len() != decompressed_size {
            return Err(KittyMCError::InvalidDecompressedPacketLength(
                b_packet.len(),
                decompressed_size,
            ));
        }

        Ok((compressed_packet_len, packet))
    }

    pub fn deserialize_uncompressed(
        state: State,
        mut data: &[u8],
    ) -> Result<(usize, Packet), KittyMCError> {
        let mut header_size = 0;
        let packet_data_and_id_len = read_varint_u32(&mut data, &mut header_size)? as usize;
        let full_packet_len = packet_data_and_id_len + header_size;

        if packet_data_and_id_len > data.len() {
            trace!(
                "Not enough data yet. Packet length: {}. Current Data length: {}",
                packet_data_and_id_len,
                data.len()
            );
            return Err(KittyMCError::NotEnoughData(
                data.len(),
                packet_data_and_id_len,
            ));
        }

        let packet_id = read_varint_u32(&mut data, &mut header_size)? as usize;
        let Some(packet_data_len) = full_packet_len.checked_sub(header_size) else {
            return Err(KittyMCError::InvalidPacketLength);
        };

        let (packet_size, packet) = Self::deserialize_by_state(
            state,
            &mut &data[..packet_data_len],
            full_packet_len,
            packet_id,
        )?;

        let full_size = header_size + packet_size;
        Ok((full_size, packet))
    }

    fn deserialize_by_state(
        state: State,
        data: &mut &[u8],
        full_packet_len: usize,
        packet_id: usize,
    ) -> Result<(usize, Packet), KittyMCError> {
        let (packet_size, packet) = match state {
            State::Handshake => match packet_id {
                0 => HandshakePacket::deserialize(data)?,
                _ => return Err(KittyMCError::NotImplemented(packet_id, full_packet_len)),
            },
            State::Status => match packet_id {
                0 => StatusRequestPacket::deserialize(data)?,
                1 => StatusPingPongPacket::deserialize(data)?,
                _ => return Err(KittyMCError::NotImplemented(packet_id, full_packet_len)),
            },
            State::Login => match packet_id {
                0 => LoginStartPacket::deserialize(data)?,
                _ => return Err(KittyMCError::NotImplemented(packet_id, full_packet_len)),
            },
            State::Play => match packet_id {
                0 => TeleportConfirmPacket::deserialize(data)?,
                2 => ServerChatMessagePacket::deserialize(data)?,
                4 => ClientSettingsPacket::deserialize(data)?,
                9 => ClientPluginMessagePacket::deserialize(data)?,
                0xB => ClientKeepAlivePacket::deserialize(data)?,
                0xD => PlayerPositionPacket::deserialize(data)?,
                0xE => ClientPlayerPositionAndLookPacket::deserialize(data)?,
                0xF => PlayerLookPacket::deserialize(data)?,
                0x1A => ClientHeldItemChangePacket::deserialize(data)?,
                0x1D => ClientAnimationPacket::deserialize(data)?,
                _ => return Err(KittyMCError::NotImplemented(packet_id, full_packet_len)),
            },
            _ => return Err(KittyMCError::NotImplemented(packet_id, full_packet_len)),
        };
        Ok((packet_size, packet))
    }

    pub fn deserialize(
        state: State,
        raw_data: &[u8],
        compression: &CompressionInfo,
    ) -> Result<(usize, Packet), KittyMCError> {
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
