use std::mem::size_of;
use std::ops::RangeBounds;
use integer_encoding::VarInt;
use miniz_oxide::deflate::compress_to_vec_zlib;
use miniz_oxide::inflate::decompress_to_vec_zlib_with_limit;
use crate::error::KittyMCError;
use crate::packets::Packet;
use crate::subtypes::state::State;

pub trait SerializablePacket {
    fn serialize(&self) -> Vec<u8> {
        vec![]
    }

    // not including length or packet id
    fn deserialize(_data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        Err(KittyMCError::NotImplemented)
    }
}

pub fn read_length_prefixed_string(data: &mut &[u8], total_size: &mut usize) -> Result<String, KittyMCError> {
    let len = read_varint_u32(data, total_size)? as usize;

    if data.len() < len {
        return Err(KittyMCError::DeserializationError);
    }

    let raw_bytes = &data[..len];
    let s = String::from_utf8(raw_bytes.to_vec())?;
    *data = &data[len..];
    *total_size += len;

    Ok(s)
}

pub fn read_i64(data: &mut &[u8], total_size: &mut usize) -> Result<i64, KittyMCError> {
    if data.len() < 8 {
        return Err(KittyMCError::DeserializationError);
    }

    let value = i64::from_be_bytes(data[..8]
        .try_into()
        .map_err(|_| KittyMCError::DeserializationError)?);
    *data = &data[8..];
    *total_size += 8;
    Ok(value)
}

pub fn read_u16_be(data: &mut &[u8], total_size: &mut usize) -> Result<u16, KittyMCError> {
    if data.len() < 2 {
        return Err(KittyMCError::DeserializationError);
    }
    let value = u16::from_be_bytes(data[..2]
        .try_into()
        .map_err(|_| KittyMCError::DeserializationError)?);
    *data = &data[2..];
    *total_size += size_of::<u16>();
    Ok(value)
}

pub fn read_varint_u32(data: &mut &[u8], total_size: &mut usize) -> Result<u32, KittyMCError> {
    let (value, size) = VarInt::decode_var(*data).ok_or(KittyMCError::DeserializationError)?;
    *data = &data[size..];
    *total_size += size;
    Ok(value)
}

pub fn read_state_varint(data: &mut &[u8], total_size: &mut usize) -> Result<State, KittyMCError> {
    let (raw_state, size) = u8::decode_var(*data).ok_or(KittyMCError::DeserializationError)?;
    *data = &data[size..];
    *total_size += size;
    Ok(State::from(raw_state))
}

pub fn write_i64(buffer: &mut Vec<u8>, value: i64) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

pub fn write_varint_u32(buffer: &mut Vec<u8>, value: u32) {
    buffer.extend_from_slice(&value.encode_var_vec());
}

pub fn write_varint_u32_splice<R: RangeBounds<usize>>(buffer: &mut Vec<u8>, value: u32, at: R) {
    buffer.splice(at, value.encode_var_vec());
}

pub fn write_varint_u8(buffer: &mut Vec<u8>, value: u8) {
    buffer.extend_from_slice(&value.encode_var_vec());
}

pub fn write_be_u16(buffer: &mut Vec<u8>, value: u16) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

pub fn write_length_prefixed_string(buffer: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    write_varint_u32(buffer, bytes.len() as u32);
    buffer.extend_from_slice(bytes);
}

pub fn compress_packet(mut packet: &[u8]) -> Result<Vec<u8>, KittyMCError> {
    let mut total_size = 0;
    let raw_packet_length = read_varint_u32(&mut packet, &mut total_size)?;

    let mut new_packet = compress_to_vec_zlib(packet, 5);
    let new_packet_len = new_packet.len() as u32;
    write_varint_u32_splice(&mut new_packet, raw_packet_length, ..0);
    write_varint_u32_splice(&mut new_packet, new_packet_len, ..0);

    Ok(new_packet)
}

pub fn decompress_packet(mut compressed_packet: &[u8]) -> Result<Vec<u8>, KittyMCError> {
    let mut total_size = 0;
    let compressed_packet_length = read_varint_u32(&mut compressed_packet, &mut total_size)? as usize;
    total_size = 0;
    let uncompressed_data_length = read_varint_u32(&mut compressed_packet, &mut total_size)?;

    let uncompressed_packet = decompress_to_vec_zlib_with_limit(&compressed_packet[..(compressed_packet_length - total_size)], uncompressed_data_length as usize)
        .map_err(|_| KittyMCError::DecompressionError)?;

    Ok(uncompressed_packet)
}