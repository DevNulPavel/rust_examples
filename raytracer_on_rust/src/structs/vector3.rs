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

impl Length for Vector3{
    fn length(&self) -> f32 {
        let length: f32 = (self.x*self.x + self.y * self.y + self.z*self.z).sqrt();
        length
    }
}

impl Normalizable for Vector3{
    fn normalize(&self) -> Self{
        let length: f32 = self.length();
        assert!(length != 0.0_f32);
        Vector3{
            x: self.x / length,
            y: self.y / length,
            z: self.z / length,
        }
    }
}

// TODO: Test
impl Dotable for Vector3{
    type Operand = Vector3;
    fn dot(&self, other: &Self::Operand) -> f32 {
            self.x * other.x + 
            self.y * other.y +
            self.z * other.z
    }
}


// TODO: Tests
impl Sub for Vector3{
    type Output = Vector3;
    fn sub(self, rhs: Self) -> Self::Output {
        Vector3{
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }  
    }
}

// TODO: Tests
impl Add for Vector3{
    type Output = Vector3;
    fn add(self, rhs: Self) -> Self::Output {
        Vector3{
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }  
    }
}

// TODO: Tests
impl Neg for Vector3{
    type Output = Vector3;
    fn neg(self) -> Self::Output {
        Vector3{
            x: -self.x,
            y: -self.y,
            z: -self.z
        }
    }
}

// TODO: Tests
impl Mul<Vector3> for Vector3{
    type Output = Vector3;
    fn mul(self, rhs: Vector3) -> Self::Output {
        Vector3{
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

// TODO: Tests
impl Mul<f32> for Vector3 {
    type Output = Vector3;
    fn mul(self, rhs: f32) -> Self::Output {
        Vector3{
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
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
