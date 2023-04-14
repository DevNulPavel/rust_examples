use crate::{
    material::Material,
    render::Ray,
    structs::{Color, Vector2, Vector3},
};

////////////////////////////////////////////////////////////////////////

pub trait Intersectable {
    fn intersect(&self, ray: &Ray) -> Option<f32>;
}

pub trait Normalable {
    fn normal_at(&self, hit_point: &Vector3) -> Vector3;
}

pub trait Colorable {
    fn color_at(&self, hit_point: &Vector3) -> Color;
}

pub trait Materiable {
    fn get_material<'a>(&'a self) -> &'a dyn Material;
}

// Описываем обязательные свойства фигуры
pub trait Figure: Intersectable + Colorable + Normalable + Materiable {}

////////////////////////////////////////////////////////////////////////

pub trait Texturable {
    fn tex_coords_at(&self, hit_point: &Vector3) -> Vector2;
}
