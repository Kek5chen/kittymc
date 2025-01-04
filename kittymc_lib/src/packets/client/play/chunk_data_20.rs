use kittymc_macros::Packet;
use crate::packets::packet_serialization::{write_bool, write_i32, write_u64, write_u8, write_varint_u32, write_varint_u32_splice, SerializablePacket};
use crate::packets::wrap_packet;

pub type BlockStateId = u32;

#[derive(PartialEq, Debug, Clone)]
pub struct ChunkSection {
    bits_per_block: u8,
    palette: Vec<BlockStateId>, // Block state IDs
    data: Vec<u64>, // Indices into palette. compacted varints as u64s
    block_light: Vec<u8>, // 16 Light levels. Half byte per block
    sky_light: Vec<u8>, // only in overworld. Half byte per block
}

impl ChunkSection {
    pub fn new(bits_per_block: u8) -> Self {
        ChunkSection {
            bits_per_block,
            palette: Vec::new(),
            data: Vec::new(),
            block_light: vec![0; 16 * 16 * 16 / 2], // half-byte per block => 4096/2 = 2048 bytes
            sky_light: vec![0; 16 * 16 * 16 / 2],
        }
    }

    pub fn write(&self, data: &mut Vec<u8>) {
        write_u8(data, self.bits_per_block);

        write_varint_u32(data, self.palette.len() as u32);
        for block in &self.palette {
            write_varint_u32(data, *block);
        }

        write_varint_u32(data, self.data.len() as u32);
        for byte in &self.data {
            write_u64(data, *byte);
        }

        for byte in &self.block_light {
            write_u8(data, *byte);
        }

        for byte in &self.sky_light {
            write_u8(data, *byte);
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Chunk {
    /// Each block in a 16×256×16 chunk.
    pub blocks: [BlockStateId; 16 * 256 * 16],

    /// 256 biomes, one per column in 16×16 top-down.
    pub biomes: [u8; 16 * 16],
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            blocks: [0; 16 * 256 * 16],
            biomes: [0; 16 * 16],
        }
    }
}

impl Chunk {
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> BlockStateId {
        self.blocks[y * 16 * 16 + z * 16 + x]
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, state: BlockStateId) {
        self.blocks[y * 16 * 16 + z * 16 + x] = state;
    }

    pub fn to_chunk_sections(&self) -> Vec<ChunkSection> {
        let mut sections = Vec::new();

        for section_y in 0..16 {
            let mut section = ChunkSection::new(8);

            // We won't do “proper” palette packing,
            // but we will store *every* block ID in the section's palette

            // Collect all block states in this section
            let mut block_states = Vec::new();
            for y in 0..16 {
                for z in 0..16 {
                    for x in 0..16 {
                        let global_y = section_y * 16 + y;
                        let block_id = self.get_block(x, global_y, z);
                        block_states.push(block_id);
                    }
                }
            }

            // Build a local palette and a map from block_id -> palette_index
            let mut palette_map = std::collections::HashMap::new();
            for &block_id in &block_states {
                if !palette_map.contains_key(&block_id) {
                    let next_index = palette_map.len() as u32;
                    palette_map.insert(block_id, next_index);
                }
            }

            section.palette = palette_map.keys().copied().collect();
            // Sort for stable ordering (for reproducibility's sake)
            section.palette.sort_unstable();

            // Re-map block IDs to palette indices
            let mut index_map = std::collections::HashMap::new();
            for (i, &block_id) in section.palette.iter().enumerate() {
                index_map.insert(block_id, i as u64);
            }

            let indices: Vec<u64> = block_states
                .iter()
                .map(|b| index_map[b])
                .collect();

            // Bit packing
            let bits_per_block = section.bits_per_block as usize;
            let mut data_words = vec![];
            let mut current_word: u64 = 0;
            let mut current_bits_filled = 0;

            for &idx in &indices {
                current_word |= idx << current_bits_filled;
                current_bits_filled += bits_per_block;
                if current_bits_filled >= 64 {
                    data_words.push(current_word);
                    current_word = 0;
                    current_bits_filled = 0;
                }
            }
            // If there's leftover
            if current_bits_filled > 0 {
                data_words.push(current_word);
            }

            section.data = data_words;

            for light_byte in &mut section.block_light {
                *light_byte = 0x00; // each byte is two 4-bit light values
            }
            for light_byte in &mut section.sky_light {
                *light_byte = 0x00;
            }

            sections.push(section);
        }

        sections
    }

    pub fn write(&self, data: &mut Vec<u8>, ground_up_continuous: bool) {
        let length_before = data.len();

        let sections = self.to_chunk_sections();
        for section in &sections {
            section.write(data);
        }

        write_varint_u32_splice(data, data.len() as u32 - length_before as u32, length_before..length_before);
        if ground_up_continuous {
            data.extend_from_slice(&self.biomes);
        }

        let size = (data.len() - length_before) as u32;
        write_varint_u32_splice(data, size, length_before..length_before);
    }
}

#[derive(PartialEq, Debug, Clone, Packet)]
pub struct ChunkDataPacket<'a> {
    x: i32,
    z: i32,
    ground_up_continuous: bool,
    primary_bit_mask: u32,
    data: &'a Chunk,
    block_entities: Vec<()>,
}

static DEFAULT_CHUNK: Chunk = Chunk {
    blocks: [1; 16 * 256 * 16],
    biomes: [0; 16 * 16],
};

impl Default for ChunkDataPacket<'_> {
    fn default() -> Self {
        ChunkDataPacket {
            x: 0,
            z: 0,
            ground_up_continuous: false,
            primary_bit_mask: 0,
            data: &DEFAULT_CHUNK,
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
        write_varint_u32(&mut packet, self.primary_bit_mask);
        self.data.write(&mut packet, self.ground_up_continuous);
        write_varint_u32(&mut packet, self.block_entities.len() as u32);

        wrap_packet(&mut packet, 0x20);

        packet
    }
}
