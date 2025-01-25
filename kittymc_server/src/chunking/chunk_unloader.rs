use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::Duration;
use log::error;
use kittymc_lib::error::KittyMCError;
use kittymc_lib::subtypes::ChunkPosition;
use crate::chunking::chunk_manager::{make_chunk_file_path, SharedChunk, SharedChunkAccessList, SharedChunkList};

const CHUNK_REMOVE_TIME_S: u64 = 30;

pub struct ChunkUnloader {
    collection: SharedChunkList,
    access_list: SharedChunkAccessList,
}

impl ChunkUnloader {
    pub fn entry_thread(collection: Arc<RwLock<HashMap<ChunkPosition, SharedChunk>>>, access_list: SharedChunkAccessList) {
        let mut unloader = ChunkUnloader {
            collection,
            access_list,
        };

        let _ = fs::create_dir("world");
        unloader.run();
    }

    fn run(&mut self) {
        loop {
            self.save_old();
            sleep(Duration::from_secs(CHUNK_REMOVE_TIME_S));
        }
    }

    fn save_old(&self) {
        let mut to_save = vec![];

        for (chunk_pos, time) in self.access_list.write().unwrap().iter() {
            if time.elapsed() >= Duration::from_secs(CHUNK_REMOVE_TIME_S) {
                to_save.push(chunk_pos.clone());
            }
        }

        let mut access_list_lock = self.access_list.write().unwrap();
        for pos in to_save {
            if let Err(e) = self.save_chunk(&pos) {
                error!("LOST! Failed to save chunk: {}", e);
            }

            access_list_lock.remove(&pos);
        }
        drop(access_list_lock);
    }

    fn save_chunk(&self, chunk_pos: &ChunkPosition) -> Result<(), KittyMCError> {
        let mut collection_lock = self.collection.write().unwrap();
        let chunk = collection_lock.remove(chunk_pos);
        drop(collection_lock);

        let Some(chunk) = chunk else {
            error!("Uhm.. chunk was so old it got dementia and forgot it existed.");
            return Err(KittyMCError::InvalidChunk(chunk_pos.block_location()));
        };

        let chunk = chunk.read().unwrap();
        chunk.save_to(&make_chunk_file_path(chunk_pos))?;

        Ok(())
    }
}