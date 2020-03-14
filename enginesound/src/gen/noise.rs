use std::time::SystemTime;
use rand_xorshift::XorShiftRng;
use rand_core::{
    RngCore, 
    SeedableRng
};

pub struct Noise {
    inner: XorShiftRng,
}

impl Default for Noise {
    fn default() -> Self {
        Noise {
            inner: XorShiftRng::from_seed(unsafe {
                std::mem::transmute::<u128, [u8; 16]>(
                    SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos(),
                )
            }),
        }
    }
}

impl Noise {
    pub fn step(&mut self) -> f32 {
        self.inner.next_u32() as f32 / (std::u32::MAX as f32 / 2.0) - 1.0
    }
}