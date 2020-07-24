use crate::{
    traits::{
        Dotable
    },
    structs::{
        Vector3,
        Color
    },
    render::{
        Ray
    }
};
use super::{
    traits::{
        Intersectable,
        Colorable,
        Figure,
        Normalable
    }
};

pub struct Plane {
    pub origin: Vector3,
    pub normal: Vector3,
    pub diffuse_color: Color,
    pub albedo_color: Color,
}

impl Colorable for Plane {
    fn get_diffuse_color<'a>(&'a self) -> &'a Color{
        let ref color = self.diffuse_color;
        color
    }

    fn get_albedo_color<'a>(&'a self) -> &'a Color{
        let ref color = self.albedo_color;
        color
    }
}

// Реализация проверки пересечения с лучем
impl Intersectable for Plane {
    // Возвращает расстояние от начала луча до точки пересечения со сферой
    fn intersect(&self, ray: &Ray) -> Option<f32> {
        let normal = &self.normal;

        let denom = normal.dot(&ray.direction);
        if denom > 1e-6 {
            let v = self.origin - ray.origin;
            let distance = v.dot(&normal) / denom;
            if distance >= 0.0 {
                return Some(distance);
            }
        }
        None
    } 
}

impl Normalable for Plane {
    fn normal_at(&self, _: &Vector3) -> Vector3{
        -self.normal
    }
}

// Пустая реализация просто чтобы пометить тип
impl Figure for Plane{
}