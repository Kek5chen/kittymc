use crate::error::KittyMCError;
use crate::packets::Packet;
use crate::subtypes::{Direction, Location, Location2};
use integer_encoding::VarInt;
use log::warn;
use miniz_oxide::deflate::compress_to_vec_zlib;
use miniz_oxide::inflate::decompress_to_vec_zlib_with_limit;
use std::mem::size_of;
use std::ops::RangeBounds;
use uuid::Uuid;

pub trait NamedPacket {
    fn name() -> &'static str {
        todo!("Define a name for this packet or just derive Packet")
    }
}

pub trait SerializablePacket {
    fn serialize(&self) -> Vec<u8> {
        vec![0]
    }

    // not including length or packet id
    fn deserialize(data: &[u8]) -> Result<(usize, Packet), KittyMCError> {
        Err(KittyMCError::NotImplemented(
            Self::id() as usize,
            data.len(),
        ))
    }

    fn id() -> u32;
}

pub fn read_length_prefixed_string(
    data: &mut &[u8],
    total_size: &mut usize,
) -> Result<String, KittyMCError> {
    let len = read_varint_u32(data, total_size)? as usize;

    if data.len() < len {
        return Err(KittyMCError::NotEnoughBytesToDeserialize(
            "String",
            len,
            data.len(),
        ));
    }

    let raw_bytes = &data[..len];
    let s = String::from_utf8(raw_bytes.to_vec())?;
    *data = &data[len..];
    *total_size += len;

    Ok(s)
}

pub fn read_length_prefixed_bytes(
    data: &mut &[u8],
    total_size: &mut usize,
) -> Result<Vec<u8>, KittyMCError> {
    let len = read_varint_u32(data, total_size)? as usize;

    if data.len() < len {
        return Err(KittyMCError::NotEnoughBytesToDeserialize(
            "Byte Array",
            len,
            data.len(),
        ));
    }

    let bytes = data[..len].to_vec();
    *data = &data[len..];
    *total_size += len;

    Ok(bytes)
}

pub fn read_u64(data: &mut &[u8], total_size: &mut usize) -> Result<u64, KittyMCError> {
    if data.len() < 8 {
        return Err(KittyMCError::NotEnoughBytesToDeserialize(
            "ULong",
            8,
            data.len(),
        ));
    }

    let value = u64::from_be_bytes(
        data[..8]
            .try_into()
            .map_err(|_| KittyMCError::DeserializationError)?,
    );
    *data = &data[8..];
    *total_size += 8;
    Ok(value)
}

pub fn read_u32(data: &mut &[u8], total_size: &mut usize) -> Result<u32, KittyMCError> {
    if data.len() < 4 {
        return Err(KittyMCError::NotEnoughBytesToDeserialize(
            "UInt",
            4,
            data.len(),
        ));
    }

    let value = u32::from_be_bytes(
        data[..4]
            .try_into()
            .map_err(|_| KittyMCError::DeserializationError)?,
    );
    *data = &data[4..];
    *total_size += 4;
    Ok(value)
}

pub fn read_u16(data: &mut &[u8], total_size: &mut usize) -> Result<u16, KittyMCError> {
    if data.len() < 2 {
        return Err(KittyMCError::NotEnoughBytesToDeserialize(
            "UShort",
            2,
            data.len(),
        ));
    }

    let value = u16::from_be_bytes(
        data[..2]
            .try_into()
            .map_err(|_| KittyMCError::DeserializationError)?,
    );
    *data = &data[2..];
    *total_size += size_of::<u16>();
    Ok(value)
}

pub fn read_u8(data: &mut &[u8], total_size: &mut usize) -> Result<u8, KittyMCError> {
    if data.len() < 1 {
        return Err(KittyMCError::NotEnoughBytesToDeserialize(
            "UByte",
            1,
            data.len(),
        ));
    }

    let value = u8::from_be_bytes(
        data[..1]
            .try_into()
            .map_err(|_| KittyMCError::DeserializationError)?,
    );
    *data = &data[1..];
    *total_size += 1;
    Ok(value)
}

pub fn read_i64(data: &mut &[u8], total_size: &mut usize) -> Result<i64, KittyMCError> {
    if data.len() < 8 {
        return Err(KittyMCError::NotEnoughBytesToDeserialize(
            "Long",
            8,
            data.len(),
        ));
    }

    let value = i64::from_be_bytes(
        data[..8]
            .try_into()
            .map_err(|_| KittyMCError::DeserializationError)?,
    );
    *data = &data[8..];
    *total_size += 8;
    Ok(value)
}

pub fn read_f64(data: &mut &[u8], total_size: &mut usize) -> Result<f64, KittyMCError> {
    if data.len() < 8 {
        return Err(KittyMCError::NotEnoughBytesToDeserialize(
            "Double",
            8,
            data.len(),
        ));
    }

    let value = f64::from_be_bytes(
        data[..8]
            .try_into()
            .map_err(|_| KittyMCError::DeserializationError)?,
    );
    *data = &data[8..];
    *total_size += 8;
    Ok(value)
}

pub fn read_f32(data: &mut &[u8], total_size: &mut usize) -> Result<f32, KittyMCError> {
    if data.len() < 4 {
        return Err(KittyMCError::NotEnoughBytesToDeserialize(
            "Float",
            4,
            data.len(),
        ));
    }

    let value = f32::from_be_bytes(
        data[..4]
            .try_into()
            .map_err(|_| KittyMCError::DeserializationError)?,
    );
    *data = &data[4..];
    *total_size += 4;
    Ok(value)
}

pub fn read_bool(data: &mut &[u8], total_size: &mut usize) -> Result<bool, KittyMCError> {
    if data.len() < 1 {
        return Err(KittyMCError::NotEnoughBytesToDeserialize(
            "Bool",
            1,
            data.len(),
        ));
    }
    let value = data[0] != 0;
    *data = &data[1..];
    *total_size += 1;
    Ok(value)
}

pub fn read_varint_i32(data: &mut &[u8], total_size: &mut usize) -> Result<i32, KittyMCError> {
    let (value, size) =
        VarInt::decode_var(*data).ok_or(KittyMCError::VarDeserializationError("VarInt"))?;
    *data = &data[size..];
    *total_size += size;
    Ok(value)
}

pub fn read_varint_u64(data: &mut &[u8], total_size: &mut usize) -> Result<u64, KittyMCError> {
    let (value, size) =
        VarInt::decode_var(*data).ok_or(KittyMCError::VarDeserializationError("VarLong"))?;
    *data = &data[size..];
    *total_size += size;
    Ok(value)
}

pub fn read_varint_u32(data: &mut &[u8], total_size: &mut usize) -> Result<u32, KittyMCError> {
    let (value, size) =
        VarInt::decode_var(*data).ok_or(KittyMCError::VarDeserializationError("VarInt"))?;
    *data = &data[size..];
    *total_size += size;
    Ok(value)
}

pub fn read_location2(data: &mut &[u8], total_size: &mut usize) -> Result<Location2, KittyMCError> {
    let x = read_f64(data, total_size)?;
    let y = read_f64(data, total_size)?;
    let z = read_f64(data, total_size)?;

    Ok(Location2::new(x, y, z))
}

pub fn read_direction(data: &mut &[u8], total_size: &mut usize) -> Result<Direction, KittyMCError> {
    let yaw = read_f32(data, total_size)?;
    let pitch = read_f32(data, total_size)?;

    Ok(Direction::new(yaw, pitch))
}

pub fn write_i64(buffer: &mut Vec<u8>, value: i64) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

pub fn write_i32(buffer: &mut Vec<u8>, value: i32) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

pub fn write_i16(buffer: &mut Vec<u8>, value: i16) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

pub fn write_i8(buffer: &mut Vec<u8>, value: i8) {
    buffer.push(value as u8);
}

pub fn write_u128(buffer: &mut Vec<u8>, value: u128) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

pub fn write_u64(buffer: &mut Vec<u8>, value: u64) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

pub fn write_u32(buffer: &mut Vec<u8>, value: u32) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

pub fn write_u16(buffer: &mut Vec<u8>, value: u16) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

pub fn write_u8(buffer: &mut Vec<u8>, value: u8) {
    buffer.push(value);
}

pub fn write_f64(buffer: &mut Vec<u8>, value: f64) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

pub fn write_f32(buffer: &mut Vec<u8>, value: f32) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

pub fn write_bool(buffer: &mut Vec<u8>, value: bool) {
    let value = if value { 1 } else { 0 };
    write_u8(buffer, value);
}

pub fn write_varint_i32(buffer: &mut Vec<u8>, value: i32) {
    buffer.extend_from_slice(&value.encode_var_vec());
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

pub fn write_length_prefixed_string(buffer: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    write_varint_u32(buffer, bytes.len() as u32);
    buffer.extend_from_slice(bytes);
}

pub fn write_length_prefixed_bytes(buffer: &mut Vec<u8>, s: &[u8]) {
    write_varint_u32(buffer, s.len() as u32);
    buffer.extend_from_slice(s);
}

pub fn write_uuid(buffer: &mut Vec<u8>, uuid: &Uuid) {
    write_u128(buffer, uuid.as_u128())
}

pub fn write_location(buffer: &mut Vec<u8>, loc: &Location) {
    write_f32(buffer, loc.x);
    write_f32(buffer, loc.y);
    write_f32(buffer, loc.z);
}

pub fn write_block_location(buffer: &mut Vec<u8>, loc: &Location) {
    // Convert the float coordinates to integers.
    let x = loc.x.floor() as i64;
    let y = loc.y.floor() as i64;
    let z = loc.z.floor() as i64;

    // 1) Mask down to their signed bit ranges:
    //    x and z must fit in 26 bits  => range: -33_554_432..33_554_431
    //    y must fit in 12 bits       => range:      -2048..2047
    //
    //    & 0x3FFFFFF captures 26 bits (for x and z).
    //    & 0xFFF      captures 12 bits (for y).
    //
    //    Because Rust negative numbers in two's complement will still
    //    produce the correct lower bits, masking is enough here.

    let x_masked = (x & 0x3FFFFFF) as u64; // 26 bits
    let z_masked = (z & 0x3FFFFFF) as u64; // 26 bits
    let y_masked = (y & 0xFFF) as u64; // 12 bits

    // 2) Pack them into 64 bits.
    // Bit layout (most significant on the left):
    //  [ x: 26 bits ][ z: 26 bits ][ y: 12 bits ]
    //
    //  x occupies bits 38..63 (the top 26 bits),
    //  z occupies bits 12..37 (the middle 26 bits),
    //  y occupies bits 0..11  (the lowest 12 bits).

    let packed = (x_masked << 38) | (z_masked << 12) | y_masked;

    buffer.extend_from_slice(&packed.to_be_bytes());
}

pub fn write_location2(buffer: &mut Vec<u8>, loc: &Location2) {
    write_f64(buffer, loc.x);
    write_f64(buffer, loc.y);
    write_f64(buffer, loc.z);
}

pub fn write_direction(buffer: &mut Vec<u8>, loc: &Direction) {
    write_f32(buffer, loc.x);
    write_f32(buffer, loc.y);
}

pub fn write_angle(buffer: &mut Vec<u8>, angle: f32) {
    write_u8(buffer, (angle / 360.0 * 256.0) as u8);
}

pub fn write_direction_as_angles(buffer: &mut Vec<u8>, loc: &Direction) {
    write_angle(buffer, loc.x);
    write_angle(buffer, loc.y);
}

pub fn write_nbt(buffer: &mut Vec<u8>, value: &fastnbt::Value) {
    let bytes = match fastnbt::to_bytes(value) {
        Ok(data) => data,
        Err(e) => {
            warn!("Failed to serialize NBT data: {}", e);
            return;
        }
    };
    buffer.extend_from_slice(&bytes);
}

pub fn compress_packet(mut packet: &[u8], threshold: u32) -> Result<Vec<u8>, KittyMCError> {
    let mut total_size = 0;
    let raw_packet_length = read_varint_u32(&mut packet, &mut total_size)?;

    let mut new_packet;
    if raw_packet_length >= threshold {
        new_packet = compress_to_vec_zlib(packet, 5);
        write_varint_u32_splice(&mut new_packet, raw_packet_length, ..0);
    } else {
        new_packet = packet.to_vec();
        write_varint_u32_splice(&mut new_packet, 0, ..0);
    }

    let new_packet_len = new_packet.len() as u32;
    write_varint_u32_splice(&mut new_packet, new_packet_len, ..0);

    Ok(new_packet)
}

pub fn decompress_packet(mut compressed_packet: &[u8]) -> Result<(usize, Vec<u8>), KittyMCError> {
    let mut header_size = 0;
    let compressed_packet_length =
        read_varint_u32(&mut compressed_packet, &mut header_size)? as usize;
    let total_size = compressed_packet_length + header_size;

    if compressed_packet.len() < compressed_packet_length {
        return Err(KittyMCError::NotEnoughData(
            compressed_packet.len(),
            compressed_packet_length,
        ));
    }

    header_size = 0;
    let uncompressed_data_length = read_varint_u32(&mut compressed_packet, &mut header_size)?;

    let uncompressed_packet =
        decompress_to_vec_zlib_with_limit(compressed_packet, uncompressed_data_length as usize)
            .map_err(|e| KittyMCError::ZlibDecompressionError(e))?;

    Ok((total_size, uncompressed_packet))
}
