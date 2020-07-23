use crate::{
    render::{
        Ray
    }
};

pub trait Zero {
    fn zero() -> Self;
}

pub trait Normalize {
    fn normalize(&self) -> Self;
}

pub trait Length {
    fn length(&self) -> f32;
}

pub trait Dot {
    type Operand;
    fn dot(&self, other: &Self::Operand) -> f32;
}

pub trait Intersectable {
    fn intersect(&self, ray: &Ray) -> bool;
}