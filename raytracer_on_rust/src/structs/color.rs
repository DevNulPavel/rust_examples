use std::{
    ops::{
        Mul
    }
};
use image::{
    Rgba,
    Rgb
};
use crate::{
    traits::{
        Zero,
        Clamp
    }
};

#[derive(Copy, Clone)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

impl Zero for Color {
    fn zero() -> Self {
        Color{
            red: 0.0_f32,
            green: 0.0_f32,
            blue: 0.0_f32,
        }   
    }
}

impl Clamp<f32> for Color {
    fn clamp(self, min: f32, max: f32) -> Color {
        Color{
            red: self.red.max(min).min(max),
            green: self.green.max(min).min(max),
            blue: self.blue.max(min).min(max)
        }   
    }
}

// TODO: TEST
impl Mul<f32> for Color{
    type Output = Color;
    fn mul(self, rhs: f32) -> Self::Output {
        Color{
            red: self.red * rhs,
            green: self.green * rhs,
            blue: self.blue * rhs,
        }
    }
}

impl Color {
    pub fn to_rgba(&self) -> Rgba<u8>{
        let r = (self.red * 255.0) as u8;
        let g = (self.green * 255.0) as u8;
        let b = (self.blue * 255.0) as u8;
        Rgba([r, g, b, 255])
    }
    pub fn from_rgba(pixel: &Rgba<u8>) -> Color {
        let r = pixel.0[0] as f32 / 255.0_f32;
        let g = pixel.0[1] as f32 / 255.0_f32;
        let b = pixel.0[2] as f32 / 255.0_f32;
        Color{
            red: r,
            green: g,
            blue: b
        }
    }
    pub fn from_rgb(pixel: &Rgb<u8>) -> Color {
        let r = pixel.0[0] as f32 / 255.0_f32;
        let g = pixel.0[1] as f32 / 255.0_f32;
        let b = pixel.0[2] as f32 / 255.0_f32;
        Color{
            red: r,
            green: g,
            blue: b
        }
    }
}