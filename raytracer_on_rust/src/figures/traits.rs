use crate::{
    structs::{
        Vector2,
        Vector3,
        Color
    },
    render::{
        Ray
    }
};

////////////////////////////////////////////////////////////////////////

pub trait Intersectable {
    fn intersect(&self, ray: &Ray) -> Option<f32>;
}

pub trait Normalable {
    fn normal_at(&self, hit_point: &Vector3) -> Vector3;
}

pub trait Colorable{
    fn color_at(&self, hit_point: &Vector3) -> Color;
}

// Описываем обязательные свойства фигуры
pub trait Figure: Intersectable + Colorable + Normalable {
}

////////////////////////////////////////////////////////////////////////

pub trait Texturable{
    fn tex_coords_at(&self, hit_point: &Vector3) -> Vector2;
}
