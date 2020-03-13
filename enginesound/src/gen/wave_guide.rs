use serde::{
    Deserialize, 
    Serialize
};

use crate::gen::delay_line::DelayLine;
use crate::gen::WAVEGUIDE_MAX_AMP;


#[derive(Clone, Serialize, Deserialize)]
pub struct WaveGuide {
    // goes from x0 to x1
    pub chamber0: DelayLine,
    // goes from x1 to x0
    pub chamber1: DelayLine,
    /// reflection factor for the first value of the return tuple of `pop`
    pub alpha: f32,
    /// reflection factor for the second value of the return tuple of `pop`
    pub beta: f32,

    // running values
    #[serde(skip)]
    c1_out: f32,
    #[serde(skip)]
    c0_out: f32,
}

impl WaveGuide {
    // Создание новое описание волны
    pub fn new(delay: usize, alpha: f32, beta: f32, samples_per_second: u32) -> WaveGuide {
        WaveGuide {
            chamber0: DelayLine::new(delay, samples_per_second),
            chamber1: DelayLine::new(delay, samples_per_second),
            alpha,
            beta,
            c1_out: 0.0,
            c0_out: 0.0,
        }
    }

    pub fn pop(&mut self) -> (f32, f32, bool) {
        let (c1_out, dampened_c1) = WaveGuide::dampen(self.chamber1.pop());
        let (c0_out, dampened_c0) = WaveGuide::dampen(self.chamber0.pop());
        self.c1_out = c1_out;
        self.c0_out = c0_out;

        (
            self.c1_out * (1.0 - self.alpha.abs()),
            self.c0_out * (1.0 - self.beta.abs()),
            dampened_c1 | dampened_c0,
        )
    }
    #[inline]
    pub fn dampen(sample: f32) -> (f32, bool) {
        let sample_abs = sample.abs();
        if sample_abs > WAVEGUIDE_MAX_AMP {
            (
                sample.signum()
                    * (-1.0 / (sample_abs - WAVEGUIDE_MAX_AMP + 1.0) + 1.0 + WAVEGUIDE_MAX_AMP),
                true,
            )
        } else {
            (sample, false)
        }
    }

    pub fn push(&mut self, x0_in: f32, x1_in: f32) {
        let c0_in = self.c1_out * self.alpha + x0_in;
        let c1_in = self.c0_out * self.beta + x1_in;

        self.chamber0.push(c0_in);
        self.chamber1.push(c1_in);
        self.chamber0.samples.advance();
        self.chamber1.samples.advance();
    }

    #[allow(clippy::float_cmp)]
    pub fn get_changed(
        &mut self,
        delay: usize,
        alpha: f32,
        beta: f32,
        samples_per_second: u32,
    ) -> Option<Self> {
        // the strictly compared values will never change without user interaction (adjusting sliders)
        if delay != self.chamber0.samples.len || alpha != self.alpha || beta != self.beta {
            let mut new = Self::new(delay, alpha, beta, samples_per_second);

            // used to reduce artifacts while resizing pipes _a bit_
            fn copy_samples_faded(source: &[f32], dest: &mut [f32]) {
                let min_len = source.len().min(dest.len());

                dest[0..min_len].copy_from_slice(&source[0..min_len]);
                let (a, b) = (*source.last().unwrap(), source[0]);
                let dest_len = dest.len();
                dest[min_len..]
                    .iter_mut()
                    .enumerate()
                    .for_each(|(i, x)| *x = a + (b - a) * i as f32 / (dest_len - min_len) as f32);
            }

            copy_samples_faded(&self.chamber0.samples.data, &mut new.chamber0.samples.data);
            copy_samples_faded(&self.chamber1.samples.data, &mut new.chamber1.samples.data);

            Some(new)
        } else {
            None
        }
    }
}
