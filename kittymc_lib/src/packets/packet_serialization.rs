use crate::error::KittyMCError;
use crate::packets::Packet;
use crate::subtypes::{Direction, Location, Location2, Rotation};
use crate::utils::axis_to_angle;
use integer_encoding::VarInt;
use log::warn;
use miniz_oxide::deflate::compress_to_vec_zlib;
use miniz_oxide::inflate::decompress_to_vec_zlib_with_limit;
use paste::paste;
use std::mem;
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

macro_rules! type_rw_impl {
    ($($ty:ty$(,)?)*) => {
        $(
        paste! {
            pub fn [<read_ $ty>](data: &mut &[u8], total_size: &mut usize) -> Result<$ty, KittyMCError> {
                let size = mem::size_of::<$ty>();
                if data.len() < size {
                    return Err(KittyMCError::NotEnoughBytesToDeserialize(
                        stringify!($ty),
                        mem::size_of::<$ty>(),
                        data.len(),
                    ));
                }

                let value = <$ty>::from_be_bytes(
                    data[..size]
                        .try_into()
                        .map_err(|_| KittyMCError::DeserializationError)?,
                );
                *data = &data[size..];
                *total_size += size;
                Ok(value)
            }

            pub fn [<write_ $ty>](buffer: &mut Vec<u8>, value: $ty) {
                buffer.extend_from_slice(&value.to_be_bytes());
            }
        }
    )*};
}

macro_rules! type_rw_varint_impl {
    ($($ty:ty$(,)?)*) => {
        $(
        paste! {
            pub fn [<read_varint_ $ty>](data: &mut &[u8], total_size: &mut usize) -> Result<$ty, KittyMCError> {
                let (value, size) =
                    VarInt::decode_var(*data).ok_or(KittyMCError::VarDeserializationError(concat!("var_", stringify!($ty))))?;
                *data = &data[size..];
                *total_size += size;
                Ok(value)
            }

            pub fn [<write_varint_ $ty>](buffer: &mut Vec<u8>, value: $ty) {
                buffer.extend_from_slice(&value.encode_var_vec());
            }

            pub fn [<write_varint_ $ty _splice>]<R: RangeBounds<usize>>(buffer: &mut Vec<u8>, value: $ty, at: R) {
                buffer.splice(at, value.encode_var_vec());
            }
        }
    )*};
}

type_rw_impl!(u128, u64, u32, u16, u8, i128, i64, i32, i16, i8, f64, f32);
type_rw_varint_impl!(u64, u32, u16, u8, i64, i32, i16, i8);

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

pub fn read_location2(data: &mut &[u8], total_size: &mut usize) -> Result<Location2, KittyMCError> {
    let x = read_f64(data, total_size)?;
    let y = read_f64(data, total_size)?;
    let z = read_f64(data, total_size)?;

    Ok(Location2::new(x, y, z))
}

// Position
pub fn read_block_location(
    data: &mut &[u8],
    total_size: &mut usize,
) -> Result<Location, KittyMCError> {
    // Ensure there are at least 8 bytes to read
    if data.len() < 8 {
        return Err(KittyMCError::NotEnoughBytesToDeserialize(
            "Position",
            8,
            data.len(),
        ));
    }

    // Read the first 8 bytes as a u64 in big-endian order
    let encoded = u64::from_be_bytes(
        data[..8]
            .try_into()
            .map_err(|_| KittyMCError::DeserializationError)?,
    );

    // Update the data slice and total_size
    *data = &data[8..];
    *total_size += 8;

    // Extract bits for x, y, z
    let x_bits = (encoded >> 38) & 0x3FFFFFF; // 26 bits for x
    let y_bits = (encoded >> 26) & 0xFFF; // 12 bits for y
    let z_bits = encoded & 0x3FFFFFF; // 26 bits for z

    // Convert the extracted bits to signed integers using two's complement
    let x = if x_bits & (1 << 25) != 0 {
        (x_bits as i32) - (1 << 26)
    } else {
        x_bits as i32
    };

    let y = if y_bits & (1 << 11) != 0 {
        (y_bits as i32) - (1 << 12)
    } else {
        y_bits as i32
    };

    let z = if z_bits & (1 << 25) != 0 {
        (z_bits as i32) - (1 << 26)
    } else {
        z_bits as i32
    };

    // Create and return the Location instance
    Ok(Location::new(x as f32, y as f32, z as f32))
}

pub fn read_direction(data: &mut &[u8], total_size: &mut usize) -> Result<Direction, KittyMCError> {
    let yaw = read_f32(data, total_size)?;
    let pitch = read_f32(data, total_size)?;

    Ok(Direction::new(yaw, pitch))
}

pub fn write_bool(buffer: &mut Vec<u8>, value: bool) {
    let value = if value { 1 } else { 0 };
    write_u8(buffer, value);
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

    // Mask to signed bit ranges
    // x, z in 26 bits: -33_554_432..=33_554_431
    // y in 12 bits:        -2048..=2047
    let x_masked = (x & 0x3FFFFFF) as u64; // 26 bits
    let y_masked = (y & 0xFFF) as u64; // 12 bits
    let z_masked = (z & 0x3FFFFFF) as u64; // 26 bits

    // Bit layout: [ x: 26 bits ][ y: 12 bits ][ z: 26 bits ]
    // x -> bits 38..63, y -> bits 26..37, z -> bits 0..25
    let packed = (x_masked << 38) | (y_masked << 26) | z_masked;

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

pub fn write_rotation(buffer: &mut Vec<u8>, loc: &Rotation) {
    let eulers = loc.euler_angles();
    write_f32(buffer, eulers.0);
    write_f32(buffer, eulers.1);
    write_f32(buffer, eulers.2);
}

pub fn write_angle(buffer: &mut Vec<u8>, angle: f32) {
    write_i8(buffer, axis_to_angle(angle));
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
