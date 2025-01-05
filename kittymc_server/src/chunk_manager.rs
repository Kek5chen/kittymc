use kittymc_lib::error::KittyMCError;
use kittymc_lib::packets::client::play::chunk_data_20::{Chunk, DEFAULT_FLAT_CHUNK};
use kittymc_lib::subtypes::{ChunkPosition, Location};
use log::error;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;
use std::time::Instant;
use tracing::debug;

pub type SharedChunk = Arc<RwLock<Box<Chunk>>>;

pub enum LoadingChunk {
    Loading,
    Loaded(Arc<Chunk>),
}

impl LoadingChunk {}

#[derive(Debug)]
pub struct ChunkManager {
    loaded_chunks: Arc<RwLock<HashMap<ChunkPosition, SharedChunk>>>,
    access_list: HashMap<ChunkPosition, Instant>,
    actively_loading_threads: HashMap<ChunkPosition, JoinHandle<Box<Chunk>>>,
}

impl ChunkManager {
    pub fn new() -> ChunkManager {
        ChunkManager {
            loaded_chunks: Arc::new(Default::default()),
            access_list: HashMap::new(),
            actively_loading_threads: HashMap::new(),
        }
    }
}

impl ChunkManager {
    fn collect_finished_threads(&mut self) -> Result<(), KittyMCError> {
        let mut new_arr = HashMap::new();

        for (chunk_pos, thread) in self.actively_loading_threads.drain() {
            if !thread.is_finished() {
                new_arr.insert(chunk_pos, thread);
                continue;
            }

            let chunk = thread.join().map_err(|e| KittyMCError::ThreadError(e))?;

            self.loaded_chunks
                .write()
                .unwrap()
                .insert(chunk_pos, Arc::new(RwLock::new(chunk)));
        }

        self.actively_loading_threads = new_arr;

        Ok(())
    }

    pub fn is_chunk_loaded(&self, loc: &ChunkPosition) -> bool {
        self.loaded_chunks.read().unwrap().contains_key(loc)
    }

    pub fn get_chunk_at(&mut self, pos: &ChunkPosition) -> Option<SharedChunk> {
        self.loaded_chunks.read().unwrap().get(pos).cloned()
    }

    pub fn get_chunk_containing(&mut self, loc: &Location) -> Option<SharedChunk> {
        self.loaded_chunks.read().unwrap().get(&loc.into()).cloned()
    }

    pub fn request_chunk(&mut self, chunk_pos: &ChunkPosition) -> Option<SharedChunk> {
        if let Err(e) = self.collect_finished_threads() {
            error!("Ran into error when collecting from chunk thread {e}");
        }
        match self.get_chunk_at(chunk_pos) {
            Some(chunk) => return Some(chunk),
            _ => {}
        }
        if self.actively_loading_threads.contains_key(chunk_pos) {
            return None;
        }

        let chunk_pos_clone = chunk_pos.clone();
        let thread = std::thread::spawn(move || Self::load_chunk_thread(chunk_pos_clone));
        self.actively_loading_threads
            .insert(chunk_pos.clone(), thread);

        None
    }

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

        if let Err(e) = self.collect_finished_threads() {
            error!("{e}");
        }

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

    // range is blocks x 16 around loc
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

    pub fn load_chunk_thread(requested_chunk: ChunkPosition) -> Box<Chunk> {
        DEFAULT_FLAT_CHUNK.clone()
    }
}
