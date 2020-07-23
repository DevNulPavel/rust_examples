use image::{
    Rgba
};
use crate::{
    traits::{
        Zero,
    }
};

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

impl Color {
    pub fn to_rgba(&self) -> Rgba<u8>{
        let r = (self.red * 255.0) as u8;
        let g = (self.green * 255.0) as u8;
        let b = (self.blue * 255.0) as u8;
        Rgba([r, g, b, 255])
    }
}