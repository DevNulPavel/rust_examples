use crate::{
    traits::{
        Zero,
        Normalize
    }
};

//////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Zero for Point {
    fn zero() -> Self {
        Point{
            x: 0.0_f32,
            y: 0.0_f32,
            z: 0.0_f32,
        }   
    }
}

impl Normalize for Point{
    fn normalize(&self) -> Self{
        Point{
            se
        }
    }
}

impl Point{
    pub fn new(x: f32, y: f32, z: f32) -> Self{
        Point{
            x,
            y, 
            z
        }
    }
}

//////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Zero for Vector3 {
    fn zero() -> Self {
        Vector3{
            x: 0.0_f32,
            y: 0.0_f32,
            z: 0.0_f32,
        }   
    }
}

impl Vector3{
    pub fn new(x: f32, y: f32, z: f32) -> Vector3{
        Vector3{
            x,
            y, 
            z
        }
    }
}

//////////////////////////////////////////////////////////////////////

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

//////////////////////////////////////////////////////////////////////

pub struct Sphere {
    pub center: Point,
    pub radius: f64,
    pub color: Color,
}
