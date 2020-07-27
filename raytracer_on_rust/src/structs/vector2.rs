use std::{
    ops::{
        Sub,
        Add,
        Neg,
        Mul
    }
};
use crate::{
    traits::{
        Length,
        Zero,
        Normalizable,
        Dotable
    }
};

//////////////////////////////////////////////////////////////////////

/*#[derive(Default)]
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

impl Into<Vector3> for Point{
    fn into(self) -> Vector3 {
        Vector3{
            x: self.x,
            y: self.y,
            z: self.z,
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
}*/


//////////////////////////////////////////////////////////////////////

#[derive(Default, Copy, Clone)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32
}

impl Zero for Vector2 {
    fn zero() -> Self {
        Vector2{
            x: 0.0_f32,
            y: 0.0_f32
        }   
    }
}

impl Length for Vector2{
    fn length(&self) -> f32 {
        let length: f32 = (self.x*self.x + self.y * self.y).sqrt();
        length
    }
}

impl Normalizable for Vector2{
    fn normalize(&self) -> Self{
        let length: f32 = self.length();
        assert!(length != 0.0_f32);
        Vector2{
            x: self.x / length,
            y: self.y / length
        }
    }
}

// TODO: Test
impl Dotable for Vector2{
    type Operand = Vector2;
    fn dot(&self, other: &Self::Operand) -> f32 {
            self.x * other.x + 
            self.y * other.y
    }
}


// TODO: Tests
impl Sub for Vector2{
    type Output = Vector2;
    fn sub(self, rhs: Self) -> Self::Output {
        Vector2{
            x: self.x - rhs.x,
            y: self.y - rhs.y
        }  
    }
}

// TODO: Tests
impl Add for Vector2{
    type Output = Vector2;
    fn add(self, rhs: Self) -> Self::Output {
        Vector2{
            x: self.x + rhs.x,
            y: self.y + rhs.y
        }  
    }
}

// TODO: Tests
impl Neg for Vector2{
    type Output = Vector2;
    fn neg(self) -> Self::Output {
        Vector2{
            x: -self.x,
            y: -self.y
        }
    }
}

// TODO: Tests
impl Mul<Vector2> for Vector2{
    type Output = Vector2;
    fn mul(self, rhs: Vector2) -> Self::Output {
        Vector2{
            x: self.x * rhs.x,
            y: self.y * rhs.y
        }
    }
}

// TODO: Tests
impl Mul<f32> for Vector2 {
    type Output = Vector2;
    fn mul(self, rhs: f32) -> Self::Output {
        Vector2{
            x: self.x * rhs,
            y: self.y * rhs
        }
    }
}

impl Vector2{
    pub fn new(x: f32, y: f32, z: f32) -> Vector2{
        Vector2{
            x,
            y
        }
    }
}
