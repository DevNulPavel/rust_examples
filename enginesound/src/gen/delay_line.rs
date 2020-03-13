use serde::{
    Deserialize, 
    Serialize
};

use crate::gen::loop_buffer::LoopBuffer;

#[derive(Clone, Serialize, Deserialize)]
pub struct DelayLine {
    pub samples: LoopBuffer,
}

impl DelayLine {
    pub fn new(delay: usize, samples_per_second: u32) -> DelayLine {
        DelayLine {
            samples: LoopBuffer::new(delay, samples_per_second),
        }
    }

    pub fn pop(&mut self) -> f32 {
        self.samples.pop()
    }

    pub fn push(&mut self, sample: f32) {
        self.samples.push(sample);
    }
}