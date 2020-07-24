use crate::{
    structs::{
        Vector3,
        Color
    },
    render::{
        Ray
    }
};

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