use std::thread::sleep;
use std::time::Duration;

const DEFAULT_MAX_IDLE_TIME_MS: u64 = 0;
const DEFAULT_IDLE_TIME_INCREASE_NS: u64 = 10;

pub struct IncreasingTicker {
    current_time_ns: u64,
    max_time_ns: u64,
    increment_ns: u64,
}

impl Default for IncreasingTicker {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_IDLE_TIME_MS * 1_000, DEFAULT_IDLE_TIME_INCREASE_NS)
    }
}

impl IncreasingTicker {
    pub fn new(max_time_ns: u64, increment_ns: u64) -> IncreasingTicker {
        IncreasingTicker {
            current_time_ns: 0,
            max_time_ns,
            increment_ns,
        }
    }

    pub fn wait_for_next_tick(&mut self) {
        sleep(Duration::from_millis(self.current_time_ns));
        self.current_time_ns = self
            .current_time_ns
            .saturating_add(self.increment_ns)
            .min(self.max_time_ns);
    }

    pub fn reset(&mut self) {
        self.current_time_ns = 0;
    }
}
