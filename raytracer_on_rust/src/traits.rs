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

pub trait Normalizable {
    fn normalize(&self) -> Self;
}

pub trait Length {
    fn length(&self) -> f32;
}

pub trait Clamp<T> {
    fn clamp(self, min: T, max: T) -> Self;
}

pub trait Dotable {
    type Operand;
    fn dot(&self, other: &Self::Operand) -> f32;
}

pub trait Intersectable {
    fn intersect(&self, ray: &Ray) -> Option<f32>;
}

pub trait Colorable{
    fn get_diffuse_color<'a>(&'a self) -> &'a Color;
    fn get_albedo_color<'a>(&'a self) -> &'a Color;
}

pub trait Normalable {
    fn normal_at(&self, hit_point: &Vector3) -> Vector3;
}

pub trait Figure: Intersectable + Colorable + Normalable {

}