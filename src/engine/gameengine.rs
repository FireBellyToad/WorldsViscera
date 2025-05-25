use macroquad::time::get_frame_time;

use crate::constants::{SECONDS_TO_WAIT};

pub struct GameEngine {
    engine_time: f32,
    tick_delay: f32,
}

impl GameEngine {
    pub fn new() -> Self {
        Self {
            engine_time: 0.0,
            tick_delay: 0.0,
        }
    }

    /// Wait for next tick, returns true if tick is exhausted
    pub fn next_tick(&mut self) -> bool {
        self.engine_time += get_frame_time();

        // needed for the engine
        if self.engine_time > self.tick_delay + SECONDS_TO_WAIT {
            self.engine_time = 0.0;
            self.tick_delay = 0.0;
        }

        self.is_tick_done()
    }

    fn is_tick_done(&self) -> bool {
        self.engine_time == 0.0
    }
}
