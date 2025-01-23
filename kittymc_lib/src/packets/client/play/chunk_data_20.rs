use crate::subtypes::Location;
use crate::packets::packet_serialization::{
    write_bool, write_i32, write_u64, write_u8, write_varint_u32, write_varint_u32_splice,
    SerializablePacket,
};
use crate::packets::wrap_packet;
use kittymc_macros::Packet;
use lazy_static::lazy_static;
use std::collections::HashMap;
use crate::error::KittyMCError;

const NUM_SECTIONS_PER_CHUNK_COLUMN: usize = 16;

const SECTION_WIDTH: usize = 16;
const SECTION_HEIGHT: usize = 16;
const SECTION_SIZE: usize = SECTION_WIDTH * SECTION_HEIGHT * SECTION_WIDTH;

/// Max bits for section palette usage
/// (above this, we switch to "global palette" which is typically 13 bits in vanilla).
const MAX_PALETTE_BITS: u8 = 8;

/// “Global palette” bit count in vanilla.
/// (Forge can go higher if more blocks are registered; 13 is correct for standard.)
const GLOBAL_BITS_PER_BLOCK: u8 = 13;

/// If you do not have true metadata, treat “BlockStateId” as the “global ID”
pub type BlockStateId = u32;

/// A single chunk section (16×16×16).
#[derive(PartialEq, Debug, Clone)]
pub struct ChunkSection {
    /// bits_per_block: how many bits each block index requires
    bits_per_block: u8,
    /// palette: local palette of block IDs (only present if bits_per_block <= 8)
    palette: Vec<BlockStateId>,
    /// data: the compacted block indices (or “global IDs” if bits_per_block > 8)
    data: Vec<u64>,
    /// 16×16×16 blocks => half-byte per block => 2048 bytes
    block_light: Vec<u8>,
    /// same as block_light, only in Overworld
    sky_light: Vec<u8>,
    section_y: u32,
}

impl ChunkSection {
    /// Create a new empty section with a specified bits-per-block.
    /// Typically, you won’t call this directly; see `Chunk::to_chunk_sections`.
    pub fn new(bits_per_block: u8, section_y: u32) -> Self {
        ChunkSection {
            bits_per_block,
            palette: Vec::new(),
            data: Vec::new(),
            block_light: vec![16; SECTION_SIZE / 2], // 2048 bytes
            sky_light: vec![16; SECTION_SIZE / 2],   // 2048 bytes
            section_y,
        }
    }

    /// Writes the chunk section into `data` according to the spec:
    ///
    /// - 1 byte for `bitsPerBlock`
    /// - [VarInt] palette length (possibly 0)
    /// - [VarInt] palette entries if present
    /// - [VarInt] length of the data array
    /// - data array (u64s)
    /// - block light bytes
    /// - skylight bytes (if dimension has skylight)
    pub fn write(&self, out: &mut Vec<u8>, has_sky_light: bool) {
        write_u8(out, self.bits_per_block);

        if self.bits_per_block <= MAX_PALETTE_BITS {
            write_varint_u32(out, self.palette.len() as u32);
            for &pal_entry in &self.palette {
                write_varint_u32(out, pal_entry);
            }
        } else {
            // “Global palette” => no section palette. We still must write palette length=0.
            // The Notchian client expects a 0 VarInt here.
            write_varint_u32(out, 0);
        }

        write_varint_u32(out, self.data.len() as u32);

        for &val in &self.data {
            write_u64(out, val);
        }

        assert_eq!(self.block_light.len(), 2048);
        assert_eq!(self.sky_light.len(), 2048);

        // 5) Write block light (always 2048 bytes)
        for &light_byte in &self.block_light {
            write_u8(out, light_byte);
        }

        // 6) If we’re in Overworld (has_sky_light = true), write skylight
        if has_sky_light {
            for &light_byte in &self.sky_light {
                write_u8(out, light_byte);
            }
        }
    }

    pub fn section_y(&self) -> u32 {
        self.section_y
    }
}

/// A full chunk (16 sections, 16×16×16 each).
#[derive(PartialEq, Debug, Clone)]
pub struct Chunk {
    /// Flattened array of size 16*256*16 = 65536.
    /// The block state at (x, y, z) is blocks[y * 16 * 16 + z * 16 + x].
    pub blocks: [BlockStateId; SECTION_SIZE * NUM_SECTIONS_PER_CHUNK_COLUMN],
    /// 16×16 = 256 biomes for top-down columns
    pub biomes: [u8; 16 * 16],
}

impl Default for Chunk {
    fn default() -> Self {
        Chunk {
            blocks: [0; SECTION_SIZE * NUM_SECTIONS_PER_CHUNK_COLUMN],
            biomes: [0; 16 * 16],
        }
    }
}

impl Chunk {
    /// Get block ID at (x, y, z).
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<BlockStateId> {
        self.blocks.get(y * 16 * 16 + z * 16 + x).cloned()
    }

    /// Set block ID at (x, y, z).
    pub fn set_block(&mut self, x: usize, y: usize, z: usize, state: BlockStateId) -> Result<(), KittyMCError> {
        let Some(block) = self.blocks.get_mut(y * 16 * 16 + z * 16 + x) else {
            return Err(KittyMCError::InvalidBlock(Location::new(x as f32, y as f32, z as f32)));
        };

        *block = state;
        Ok(())
    }

    /// Check if a given section index (0..16) is entirely air (i.e., block == 0).
    fn is_section_empty(&self, section_y: usize) -> bool {
        let start_y = section_y * SECTION_HEIGHT;
        let end_y = start_y + SECTION_HEIGHT;
        for y in start_y..end_y {
            for z in 0..SECTION_WIDTH {
                for x in 0..SECTION_WIDTH {
                    if self.get_block(x, y, z).is_some_and(|b| b != 0) {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Converts the entire chunk’s blocks into a vector of `ChunkSection`,
    /// skipping any sections that are fully air.
    ///
    /// We also compute bits per block per section:
    ///  - If distinct block states <= 1, skip (air).
    ///  - If distinct block states == 1..=16, we set bits=4
    ///  - If 16 < distinct <= 256, bits = ceil(log2(distinct)), min 4
    ///  - If distinct > 256, use “global” bits=13, palette length=0
    ///
    /// We also fill block_light=0 and sky_light=0 as placeholders.
    pub fn to_chunk_sections(&self, primary_bit_mask_out: &mut u32) -> Vec<ChunkSection> {
        let mut sections = Vec::new();

        for section_y in 0..NUM_SECTIONS_PER_CHUNK_COLUMN {
            // If the section is fully air, skip
            if self.is_section_empty(section_y) {
                continue;
            }

            // Otherwise, set that bit in the mask
            *primary_bit_mask_out |= 1 << section_y;

            let mut block_states = Vec::with_capacity(SECTION_SIZE);
            let base_y = section_y * SECTION_HEIGHT;
            for y in 0..SECTION_HEIGHT {
                for z in 0..SECTION_WIDTH {
                    for x in 0..SECTION_WIDTH {
                        let global_y = base_y + y;
                        block_states.push(self.get_block(x, global_y, z).unwrap());
                    }
                }
            }

            // Determine how many distinct states
            let mut distinct_map = HashMap::new();
            for &block_id in &block_states {
                distinct_map.entry(block_id).or_insert(true);
            }
            let distinct_count = distinct_map.len();

            // Decide bitsPerBlock
            //    - clamp to [4..8] if using a local palette
            //    - use 13 if distinct_count > 256
            //    - if we have only 1 distinct state, bits=4, but the entire data array can be zero
            let bits_per_block = if distinct_count <= 1 {
                // If we got here, it’s not truly empty => must be a single repeated block
                // We still have to send bits_per_block=4 for the “lowest” valid
                4
            } else if distinct_count > 256 {
                GLOBAL_BITS_PER_BLOCK
            } else {
                // Next power-of-two >= distinct_count, then take log2
                let b = (64 - (distinct_count - 1).leading_zeros()) as u8;
                let b = b.max(4).min(MAX_PALETTE_BITS);
                b
            };

            let mut section = ChunkSection::new(bits_per_block, section_y as u32);

            // If bits_per_block <= 8, build a sorted local palette
            let mut palette_index_map = HashMap::new();
            if bits_per_block <= MAX_PALETTE_BITS {
                // create a sorted palette
                let mut local_palette: Vec<BlockStateId> = distinct_map.keys().copied().collect();
                local_palette.sort_unstable();

                // build map from blockstate -> local palette index
                for (idx, &bs) in local_palette.iter().enumerate() {
                    palette_index_map.insert(bs, idx as u64);
                }
                section.palette = local_palette;
            }

            // Build the “data” array (compacting each 16×16×16 block index)
            //    with a typical 64-bit word-packing.
            let mut data_words = Vec::new();
            let mut current_word = 0u64;
            let mut bits_filled = 0;

            for &block_id in &block_states {
                let index = if bits_per_block <= MAX_PALETTE_BITS {
                    // local palette index
                    *palette_index_map.get(&block_id).unwrap()
                } else {
                    // “Global palette” = just use block_id as-is
                    // (Vanilla uses (blockId << 4 | metadata), but if your block_id
                    // is already unique, we can store it directly.)
                    block_id as u64
                };

                // put index in current_word
                current_word |= index << bits_filled;
                bits_filled += bits_per_block;

                if bits_filled >= 64 {
                    data_words.push(current_word);
                    current_word = 0;
                    bits_filled = 0;
                }
            }
            if bits_filled > 0 {
                data_words.push(current_word);
            }

            section.data = data_words;

            // Just set 0 as light for now
            for byte in &mut section.block_light {
                *byte = 0x0;
            }
            for byte in &mut section.sky_light {
                *byte = 0xFF;
            }

            sections.push(section);
        }

        sections
    }

    /// Write the chunk to a buffer.
    /// - We do *not* skip chunk entirely if empty, but skip sections.
    /// - `ground_up_continuous = true` means we send biomes.
    /// - The “size” field is written as VarInt for the total chunk data, not including the packet header.
    pub fn write(
        &self,
        buf: &mut Vec<u8>,
        ground_up_continuous: bool,
        dimension_has_sky_light: bool,
        primary_bit_mask: &mut u32,
    ) {
        // We write the chunk sections into a temporary buffer, so we can prepend the final size.
        let start_len = buf.len();

        // The primary bit mask is 16 bits, one for each possible section. We build it dynamically.
        let sections = self.to_chunk_sections(primary_bit_mask);

        // Now write each section’s data in ascending Y order.
        // The order in the protocol is “bottom to top” for those that exist.
        // We can do a check on each bit in the mask and see if that section was built.
        for section_y in 0..NUM_SECTIONS_PER_CHUNK_COLUMN {
            if (*primary_bit_mask & (1 << section_y)) == 0 {
                // skip
                continue;
            }
            // Actually write the data for that section
            let section = &sections
                .iter()
                .find(|s| s.section_y() == section_y as u32)
                .unwrap();
            section.write(buf, dimension_has_sky_light);
        }

        // If “ground up continuous” is set, then we must write 256 biomes (one for each block column).
        if ground_up_continuous {
            buf.extend_from_slice(&self.biomes);
        }

        // Now that the chunk section data is written, we know how big it was:
        let full_size = (buf.len() - start_len) as u32;
        write_varint_u32_splice(buf, full_size, start_len..start_len);
    }
}

/// This is your packet.  We’ll show how we incorporate `Chunk::write`.
#[derive(PartialEq, Debug, Clone, Packet)]
pub struct ChunkDataPacket<'a> {
    x: i32,
    z: i32,
    ground_up_continuous: bool,
    /// The chunk data reference
    data: &'a Chunk,
    /// For simplicity, we’ll skip real block entities.  They’d come after the chunk data.
    block_entities: Vec<()>,
}

lazy_static! {
    pub static ref DEFAULT_FLAT_CHUNK: Box<Chunk> = {
        let mut chunk = Box::new(Chunk {
            blocks: [0; SECTION_SIZE * NUM_SECTIONS_PER_CHUNK_COLUMN],
            biomes: [1; 16 * 16],
        });

        for x in 0..SECTION_WIDTH {
            for z in 0..SECTION_WIDTH {
                chunk.set_block(x, 0, z, 0b0111_0000);
                for y in 1..4 {
                    chunk.set_block(x, y, z, 0b0001_0000);
                }
                chunk.set_block(x, 4, z, 5 << 4);
            }
        }

        for x in 0..16 {
            for z in 5..11 {
                let block: u32 = (35 << 4)
                    | match z {
                        5 => 14,
                        6 => 1,
                        7 => 4,
                        8 => 5,
                        n => n as u32,
                    };

                chunk.set_block(x, 4, z, block);
            }
        }

        chunk
    };
}

impl ChunkDataPacket<'_> {
    pub fn default_at(x: i32, z: i32) -> Self {
        ChunkDataPacket {
            x,
            z,
            ground_up_continuous: true,
            data: &DEFAULT_FLAT_CHUNK,
            block_entities: vec![],
        }
    }

    pub fn new(chunk: &Chunk, x: i32, z: i32) -> ChunkDataPacket {
        ChunkDataPacket {
            x,
            z,
            ground_up_continuous: true,
            data: chunk,
            block_entities: vec![],
        }
    }
}

impl SerializablePacket for ChunkDataPacket<'_> {
    fn serialize(&self) -> Vec<u8> {
        let mut packet = vec![];

        write_i32(&mut packet, self.x);
        write_i32(&mut packet, self.z);
        write_bool(&mut packet, self.ground_up_continuous);

        // We'll store the position in the buffer so we can back-patch the mask there.
        let mask_pos = packet.len();

        let mut primary_bit_mask = 0u32;
        self.data.write(
            &mut packet,
            self.ground_up_continuous,
            true,
            &mut primary_bit_mask,
        ); // TODO: Nobody knows if this is an overworld chunk or not yet

        self.data.to_chunk_sections(&mut primary_bit_mask);
        write_varint_u32_splice(&mut packet, primary_bit_mask, mask_pos..mask_pos);

        write_varint_u32(&mut packet, 0u32); // TODO: Implement Block Entitites

        wrap_packet(&mut packet, Self::id());

        packet
    }

    fn id() -> u32 {
        0x20
    }
}
