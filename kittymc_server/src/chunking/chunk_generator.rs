use crate::chunking::chunk_manager::{
    make_chunk_file_path, ChunkPriority, SharedChunk, SharedChunkList, SharedQueue,
};
use crate::chunking::increasing_ticker::IncreasingTicker;
use kittymc_lib::error::KittyMCError;
use kittymc_lib::packets::client::play::chunk_data_20::{Chunk, DEFAULT_FLAT_CHUNK, DEFAULT_FLAT_CHUNK_2};
use kittymc_lib::subtypes::ChunkPosition;
use log::{debug, error};
use std::path::Path;
use std::sync::RwLock;

pub struct ChunkGenerator {
    collector: SharedChunkList,
    low_queue: SharedQueue,
    middle_queue: SharedQueue,
    high_queue: SharedQueue,
    ticker: IncreasingTicker,
}

impl ChunkGenerator {
    pub fn entry_thread(
        collector: SharedChunkList,
        low: SharedQueue,
        mid: SharedQueue,
        high: SharedQueue,
    ) {
        let mut gen = ChunkGenerator {
            collector,
            low_queue: low,
            middle_queue: mid,
            high_queue: high,
            ticker: IncreasingTicker::default(),
        };

        gen.run();
    }

    pub fn run(&mut self) {
        loop {
            self.bite_queue();
            self.ticker.wait_for_next_tick();
        }
    }

    fn bite_queue(&mut self) {
        if self.bite_specific_queue(ChunkPriority::HIGH)
            || self.bite_specific_queue(ChunkPriority::MID)
            || self.bite_specific_queue(ChunkPriority::LOW)
        {
            return;
        }
    }

    fn bite_specific_queue(&mut self, queue: ChunkPriority) -> bool {
        let mut queue = match queue {
            ChunkPriority::HIGH => &self.high_queue,
            ChunkPriority::MID => &self.middle_queue,
            ChunkPriority::LOW => &self.low_queue,
        }
        .write()
        .unwrap();

        let chunk_pos = queue.pop_front();

        drop(queue);

        if let Some(chunk_pos) = chunk_pos {
            let file_path = make_chunk_file_path(&chunk_pos);
            let exists = file_path.exists();
            let chunk_res = match exists {
                true => self.load_chunk(&file_path),
                false => Ok(self.start_generation(&chunk_pos)),
            };

            let chunk = match chunk_res {
                Ok(chunk) => chunk,
                Err(e) => {
                    error!("Failed to load chunk: {e}");
                    return true;
                }
            };

            self.collector
                .write()
                .unwrap()
                .insert(chunk_pos, SharedChunk::new(RwLock::new(chunk)));

            return true;
        }

        false
    }

    pub fn load_chunk(&mut self, file_path: &Path) -> Result<Box<Chunk>, KittyMCError> {
        debug!("Loaded chunk from file");
        Ok(Chunk::load_from(file_path)?)
    }

    pub fn start_generation(&mut self, chunk_pos: &ChunkPosition) -> Box<Chunk> {
        self.ticker.reset();
        self.generate(chunk_pos)
    }

    pub fn generate(&mut self, chunk_pos: &ChunkPosition) -> Box<Chunk> {
        if chunk_pos.chunk_z().abs() % 2 == 1 {
            DEFAULT_FLAT_CHUNK.clone()
        } else {
            DEFAULT_FLAT_CHUNK_2.clone()
        }
    }
}
