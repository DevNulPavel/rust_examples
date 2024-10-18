use super::traits::{Colorable, Figure, Intersectable, Materiable, Normalable, Texturable};
use crate::{
    material::{Material, MaterialsContainer, TexCoordDelegate},
    render::Ray,
    structs::{Color, Vector2, Vector3},
    traits::{Crossable, Dotable, Length},
};

pub struct Plane {
    pub origin: Vector3,
    pub normal: Vector3,
    pub material: MaterialsContainer,
}

impl Texturable for Plane {
    fn tex_coords_at(&self, hit_point: &Vector3) -> Vector2 {
        // Сначала находим оси нашей плоскости
        // https://bheisler.github.io/post/writing-raytracer-in-rust-part-3/
        let mut x_axis = self.normal.cross(&Vector3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        });
        if x_axis.length() == 0.0 {
            x_axis = self.normal.cross(&Vector3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            });
        }
        let y_axis = self.normal.cross(&x_axis);
        // TODO: Разобраться
        let hit_vec = *hit_point - self.origin;
        Vector2 {
            x: hit_vec.dot(&x_axis),
            y: hit_vec.dot(&y_axis),
        }
    }
}

impl Colorable for Plane {
    fn color_at(&self, hit_point: &Vector3) -> Color {
        let tex_coord_delegate = TexCoordDelegate {
            target: self,
            hit_point: hit_point,
        };
        let color = self
            .material
            .get_material()
            .get_color_at_tex_coord(tex_coord_delegate);
        color
    }
}

// Реализация проверки пересечения с лучем
impl Intersectable for Plane {
    // TODO: Разобраться
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
    fn normal_at(&self, _: &Vector3) -> Vector3 {
        -self.normal
    }
}

impl Materiable for Plane {
    fn get_material<'a>(&'a self) -> &'a dyn Material {
        self.material.get_material()
    }
}

// Пустая реализация просто чтобы пометить тип
impl Figure for Plane {}
