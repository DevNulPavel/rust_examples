
use serde::{Deserialize, Serialize};
use crate::gen::LoopBuffer;


#[allow(unused_imports)]
#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[allow(unused_imports)]
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use simdeez::{avx2::*, scalar::*, sse2::*, sse41::*, *};

#[derive(Clone, Serialize, Deserialize)]
pub struct LowPassFilter {
    pub delay: f32,
    #[serde(skip)]
    pub len: f32,
    #[serde(skip)]
    pub samples: LoopBuffer,
}

impl LowPassFilter {
    pub fn new(freq: f32, samples_per_second: u32) -> LowPassFilter {
        let len = (samples_per_second as f32 / freq)
            .min(samples_per_second as f32)
            .max(1.0);
        LowPassFilter {
            samples: LoopBuffer::new(len.ceil() as usize, samples_per_second),
            delay: 1.0 / freq,
            len,
        }
    }

    #[inline]
    pub fn get_freq(&self, samples_per_second: u32) -> f32 {
        samples_per_second as f32 / self.len
    }

    pub fn filter(&mut self, sample: f32) -> f32 {
        if self.len == 0.0 {
            self.len = self.samples.len as f32;
        }

        self.samples.push(sample);
        self.samples.advance();

        #[inline(always)]
        unsafe fn sum<S: Simd>(samples: &[f32], flen: f32) -> f32 {
            let mut i = S::VF32_WIDTH;
            let len = samples.len();
            assert_eq!(
                len % S::VF32_WIDTH,
                0,
                "LoopBuffer length is not a multiple of the SIMD vector size"
            );

            // rolling sum
            let mut rolling_sum = S::loadu_ps(&samples[0]);

            while i != len {
                rolling_sum += S::loadu_ps(&samples[i]);
                i += S::VF32_WIDTH;
            }

            let fract = flen.fract();
            // only use fractional averaging if flen.fract() > 0.0
            if fract != 0.0 {
                // subtract the last element and add it onto the sum again but multiplied with the fractional part of the length
                (S::horizontal_add_ps(rolling_sum) - samples[flen as usize] * (1.0 - fract)) / flen
            } else {
                // normal average
                S::horizontal_add_ps(rolling_sum) / flen
            }
        }

        // expanded 'simd_runtime_select' macro for feature independency (proc_macro_hygiene)
        if is_x86_feature_detected!("avx2") {
            #[target_feature(enable = "avx2")]
            unsafe fn call(samples: &[f32], len: f32) -> f32 {
                sum::<Avx2>(samples, len)
            }
            unsafe { call(&self.samples.data, self.len) }
        } else if is_x86_feature_detected!("sse4.1") {
            #[target_feature(enable = "sse4.1")]
            unsafe fn call(samples: &[f32], len: f32) -> f32 {
                sum::<Sse41>(samples, len)
            }
            unsafe { call(&self.samples.data, self.len) }
        } else if is_x86_feature_detected!("sse2") {
            #[target_feature(enable = "sse2")]
            unsafe fn call(samples: &[f32], len: f32) -> f32 {
                sum::<Sse2>(samples, len)
            }
            unsafe { call(&self.samples.data, self.len) }
        } else {
            unsafe { sum::<Scalar>(&self.samples.data, self.len) }
        }
    }

    #[allow(clippy::float_cmp)]
    pub fn get_changed(&mut self, freq: f32, samples_per_second: u32) -> Option<Self> {
        let newfreq_len = (samples_per_second as f32 / freq)
            .min(samples_per_second as f32)
            .max(1.0);

        // the strictly compared values will never change without user interaction (adjusting sliders)
        if newfreq_len != self.len {
            Some(Self::new(freq, samples_per_second))
        } else {
            None
        }
    }
}