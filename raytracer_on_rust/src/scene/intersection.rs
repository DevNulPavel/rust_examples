use crate::{
    figures::{
        Figure
    },
    structs::{
        Vector3
    }
};
// use super::{
//     intersection_full::{
//         IntersectionFull
//     }
// };

pub struct Intersection<'a> {
    distance: f32,
    hit_point: Vector3,
    object: &'a dyn Figure
}

impl<'a> Intersection<'a> {
    pub fn new(distance: f32, hit_point:Vector3, object: &'a dyn Figure) -> Intersection<'a> {
        Intersection{
            distance,
            hit_point,
            object
        }
    }

    /// Для найденной фигуры и точки пересечения получаем дистанцию
    pub fn get_distance(&self) -> f32{
        self.distance
    }

    /// Для найденной фигуры и точки пересечения получаем точку пересечения
    pub fn get_hit_point(&'a self) -> &'a Vector3{
        &self.hit_point
    }

    /// Найденная фигура
    pub fn get_object(&'a self) -> &'a dyn Figure {
        self.object
    }
}
