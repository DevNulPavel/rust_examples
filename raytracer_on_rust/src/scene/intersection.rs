use crate::{
    figures::{
        Figure
    },
    structs::{
        Vector3
    }
};

pub struct Intersection<'a> {
    pub distance: f32,
    pub hit_point: Vector3,
    pub object: &'a dyn Figure,
}

impl<'a> Intersection<'a> {
    pub fn new(distance: f32, hit_point:Vector3, object: &'a dyn Figure) -> Intersection<'a> {
        Intersection{
            distance,
            hit_point,
            object
        }
    }

    pub fn get_normal(&self) -> Vector3{
        self.object.normal_at(&self.hit_point)
    }
}