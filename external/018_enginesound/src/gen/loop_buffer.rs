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
    // Длительность звучания конкретного значения в буфере
    pub delay: f32,

    // Длина
    #[serde(skip)]
    pub len: usize,

    // Буффер
    #[serde(skip)]
    pub data: Vec<f32>,

    // Позиция
    #[serde(skip)]
    pub pos: usize,
}

impl LoopBuffer {
    /// Создаем новый циклический буффер специфической длины
    /// Внутренний буффер данных будет округлен до подходящего SIMD размера отдельного вектора
    pub fn new(len: usize, samples_per_second: u32) -> LoopBuffer {
        // Находим нормальный размер буффера для входящей длины
        let bufsize = LoopBuffer::get_best_simd_size(len);
        LoopBuffer {
            delay: len as f32 / samples_per_second as f32,
            len,
            data: vec![0.0; bufsize],
            pos: 0,
        }
    }

    /// Возвращает правильный размер буффера, чтобы он был кратен линии SIMD
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

    /// Пушим данные, должно быть вызвано вместе с pop
    /// ```rust
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

    /// Получаем значение `self.len` samples prior. Должно быть вызвано вместе с `push`.
    pub fn pop(&mut self) -> f32 {
        let len = self.len;
        self.data[(self.pos + 1) % len]
    }

    /// Смещает позицию циклического буффера
    pub fn advance(&mut self) {
        self.pos += 1;
    }
}