use std::default::Default;
use kittymc_lib::error::KittyMCError;
use kittymc_lib::packets::client::play::chunk_data_20::{BlockStateId, Chunk};
use kittymc_lib::subtypes::{ChunkPosition, Location};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;
use std::time::Instant;
use crate::chunking::chunk_generator::ChunkGenerator;
use crate::chunking::chunk_unloader::ChunkUnloader;

pub type SharedChunk = Arc<RwLock<Box<Chunk>>>;
pub type SharedQueue = Arc<RwLock<VecDeque<ChunkPosition>>>;
pub type SharedChunkList = Arc<RwLock<HashMap<ChunkPosition, SharedChunk>>>;
pub type SharedChunkAccessList = Arc<RwLock<HashMap<ChunkPosition, Instant>>>;

const GENERATOR_THREADS: usize = 4;
const UNLOADER_THREADS: usize = 1;

pub enum ChunkPriority {
    HIGH,
    MID,
    LOW,
}

#[derive(Debug)]
pub struct ChunkManager {
    loaded_chunks: SharedChunkList,
    access_list: SharedChunkAccessList,

    generator_threads: Vec<JoinHandle<()>>,
    unloader_threads: Vec<JoinHandle<()>>,

    high_priority_queue: SharedQueue,
    medium_priority_queue: SharedQueue,
    low_priority_queue: SharedQueue,
}

impl ChunkManager {
    pub fn new() -> ChunkManager {
        let mut manager = ChunkManager {
            loaded_chunks: Arc::new(Default::default()),
            access_list: Arc::new(Default::default()),

            generator_threads: Vec::new(),
            unloader_threads: Vec::new(),

            high_priority_queue: Arc::new(Default::default()),
            medium_priority_queue: Arc::new(Default::default()),
            low_priority_queue: Arc::new(Default::default()),
        };

        manager.init_threads();

        manager
    }

    fn init_threads(&mut self) {
        for _ in 0..GENERATOR_THREADS {
            let collector = self.loaded_chunks.clone();
            let high_queue = self.high_priority_queue.clone();
            let medium_queue = self.medium_priority_queue.clone();
            let low_priority_queue = self.low_priority_queue.clone();

            self.generator_threads.push(std::thread::spawn(|| ChunkGenerator::entry_thread(collector, high_queue, medium_queue, low_priority_queue)));
        }

        for _ in 0..UNLOADER_THREADS {
            let collector = self.loaded_chunks.clone();
            let access_list = self.access_list.clone();

            self.unloader_threads.push(std::thread::spawn(|| ChunkUnloader::entry_thread(collector, access_list)));
        }
    }

    #[allow(dead_code)]
    pub fn is_chunk_loaded(&self, pos: &ChunkPosition) -> bool {
        let mut pos = pos.clone();
        pos.set_chunk_y(0);
        self.loaded_chunks.read().unwrap().contains_key(&pos)
    }

    pub fn get_chunk_at(&mut self, pos: &ChunkPosition) -> Option<SharedChunk> {
        let mut pos = pos.clone();
        pos.set_chunk_y(0);
        self.access_list.write().unwrap().insert(pos.clone(), Instant::now());
        self.loaded_chunks.read().unwrap().get(&pos).cloned()
    }

    #[allow(dead_code)]
    pub fn get_chunk_containing_block(&mut self, loc: &Location) -> Option<SharedChunk> {
        let mut chunk: ChunkPosition = loc.into();
        chunk.set_chunk_y(0);
        self.loaded_chunks.read().unwrap().get(&chunk).cloned()
    }

    pub fn is_queued(&self, chunk_pos: &ChunkPosition) -> bool {
        let mut chunk_pos = chunk_pos.clone();
        chunk_pos.set_chunk_y(0);

        self.high_priority_queue.read().unwrap().contains(&chunk_pos) ||
            self.medium_priority_queue.read().unwrap().contains(&chunk_pos) ||
            self.low_priority_queue.read().unwrap().contains(&chunk_pos)
    }

    pub fn request_chunk(&mut self, chunk_pos: &ChunkPosition) -> Option<SharedChunk> {
        let mut chunk_pos = chunk_pos.clone();
        chunk_pos.set_chunk_y(0);

        match self.get_chunk_at(&chunk_pos) {
            Some(chunk) => return Some(chunk),
            _ => {}
        }
        if self.is_queued(&chunk_pos) {
            return None;
        }

        self.high_priority_queue.write().unwrap().push_back(chunk_pos);

        None
    }

    #[allow(dead_code)]
    pub fn request_chunks_bulk(
        &mut self,
        chunks: &[ChunkPosition],
    ) -> HashMap<ChunkPosition, SharedChunk> {
        let mut loaded = HashMap::new();

        for pos in chunks {
            match self.request_chunk(pos) {
                None => continue,
                Some(chunk) => {
                    loaded.insert(pos.clone(), chunk);
                }
            }
        }

        loaded
    }

    #[allow(dead_code)]
    pub fn poll_chunks_in_range(
        &mut self,
        loc: &Location,
        radius: u32,
    ) -> Option<HashMap<ChunkPosition, SharedChunk>> {
        let mut loaded_chunks = HashMap::new();
        let requested_chunks: Vec<ChunkPosition> =
            ChunkPosition::iter_xz_circle_in_range(loc, radius as f32).collect();
        let requested_count = requested_chunks.len();

        //debug!("Requested: {requested_chunks:?}");

        for chunk_pos in requested_chunks {
            let Some(chunk) = self.request_chunk(&chunk_pos) else {
                continue;
            };

            loaded_chunks.insert(chunk_pos, chunk);
        }

        if loaded_chunks.len() != requested_count {
            return None;
        }

        Some(loaded_chunks)
    }

    #[allow(dead_code)]
    pub fn request_chunks_in_range(
        &mut self,
        loc: &Location,
        radius: u32,
    ) -> Vec<(ChunkPosition, SharedChunk)> {
        let mut loaded_chunks = Vec::new();
        let requested_chunks: Vec<ChunkPosition> =
            ChunkPosition::iter_xz_circle_in_range(loc, radius as f32).collect();

        for chunk_pos in requested_chunks {
            match self.request_chunk(&chunk_pos) {
                Some(chunk) => loaded_chunks.push((chunk_pos.clone(), chunk)),
                _ => {}
            }
        }

        loaded_chunks
    }

    pub fn set_block(&mut self, loc: &Location, block_id: BlockStateId) -> Result<(), KittyMCError> {
        let chunk = self.get_chunk_containing_block(loc)
            .ok_or_else(|| KittyMCError::InvalidChunk(loc.clone()))?;
        let mut chunk_lock = chunk.write()
            .map_err(|_| KittyMCError::LockPoisonError)?;

        let chunk_pos = ChunkPosition::from(loc);

        let x = (loc.x - chunk_pos.block_x() as f32).floor() as usize;
        let y = loc.y.floor() as usize;
        let z = (loc.z - chunk_pos.block_z() as f32).floor() as usize;

        chunk_lock.set_block(x, y, z, block_id)
    }
}

pub fn make_chunk_file_path(chunk_pos: &ChunkPosition) -> PathBuf {
    format!("world/{}me{}ow{}.kitty", chunk_pos.chunk_x(), chunk_pos.chunk_y(), chunk_pos.chunk_z()).into()
}