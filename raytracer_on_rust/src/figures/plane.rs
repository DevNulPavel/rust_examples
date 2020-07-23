use crate::{
    traits::{
        Intersectable,
        Dot,
        PixelColor,
        Figure,
        Normal
    },
    structs::{
        Vector3,
        Color
    },
    render::{
        Ray
    }
};

pub struct Plane {
    pub origin: Vector3,
    pub normal: Vector3,
    pub color: Color,
}

impl PixelColor for Plane {
    fn get_pixel_color<'a>(&'a self) -> &'a Color{
        let ref color = self.color;
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

impl Normal for Plane {
    fn normal_at(&self, _: &Vector3) -> Vector3{
        -self.normal
    }
}

// Пустая реализация просто чтобы пометить тип
impl Figure for Plane{
}