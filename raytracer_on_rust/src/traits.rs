use crate::{
    render::{
        Ray
    },
    structs::{
        Color,
        Vector3
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
    fn intersect(&self, ray: &Ray) -> Option<f32>;
}

pub trait PixelColor{
    fn get_pixel_color<'a>(&'a self) -> &'a Color;
}

pub trait Normal {
    fn normal_at(&self, hit_point: &Vector3) -> Vector3;
}

pub trait Figure: Intersectable + PixelColor + Normal {

}