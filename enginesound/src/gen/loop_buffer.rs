use serde::{
    Deserialize, 
    Serialize
};


#[allow(unused_imports)]
#[cfg(target_arch = "x86")]
use std::arch::x86::*;

#[allow(unused_imports)]
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use simdeez::{
    *,
    avx2::*, 
    scalar::*, 
    sse2::*, 
    sse41::*
};

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct LoopBuffer {
    // in seconds
    pub delay: f32,
    #[serde(skip)]
    pub len: usize,
    #[serde(skip)]
    pub data: Vec<f32>,
    #[serde(skip)]
    pub pos: usize,
}

impl LoopBuffer {
    /// Creates a new loop buffer with specifies length.
    /// The internal sample buffer size is rounded up to the currently best SIMD implementation's float vector size.
    pub fn new(len: usize, samples_per_second: u32) -> LoopBuffer {
        let bufsize = LoopBuffer::get_best_simd_size(len);
        LoopBuffer {
            delay: len as f32 / samples_per_second as f32,
            len,
            data: vec![0.0; bufsize],
            pos: 0,
        }
    }

    /// Returns `(size / SIMD_REGISTER_SIZE).ceil() * SIMD_REGISTER_SIZE`, where `SIMD` may be the best simd implementation at runtime.
    /// Used to create vectors to make simd iteration easier
    pub fn get_best_simd_size(size: usize) -> usize {
        if is_x86_feature_detected!("avx2") {
            ((size - 1) / Avx2::VF32_WIDTH + 1) * Avx2::VF32_WIDTH
        } else if is_x86_feature_detected!("sse4.1") {
            ((size - 1) / Sse41::VF32_WIDTH + 1) * Sse41::VF32_WIDTH
        } else if is_x86_feature_detected!("sse2") {
            ((size - 1) / Sse2::VF32_WIDTH + 1) * Sse2::VF32_WIDTH
        } else {
            ((size - 1) / Scalar::VF32_WIDTH + 1) * Scalar::VF32_WIDTH
        }
    }

    /// Sets the value at the current position. Must be called with `pop`.
    /// ```rust
    /// // assuming Simd is Scalar
    /// let mut lb = LoopBuffer::new(2);
    /// lb.push(1.0);
    /// lb.advance();
    ///
    /// assert_eq(lb.pop(), 1.0);
    ///
    /// ```
    pub fn push(&mut self, value: f32) {
        let len = self.len;
        self.data[self.pos % len] = value;
    }

    /// Gets the value `self.len` samples prior. Must be called with `push`.
    /// See `push` for examples
    pub fn pop(&mut self) -> f32 {
        let len = self.len;
        self.data[(self.pos + 1) % len]
    }

    /// Advances the position of this loop buffer.
    pub fn advance(&mut self) {
        self.pos += 1;
    }
}