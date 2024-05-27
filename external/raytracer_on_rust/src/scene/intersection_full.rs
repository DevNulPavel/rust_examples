use super::Intersection;
use crate::{
    figures::Figure,
    structs::{Color, Vector3},
};

// Данный класс служит для кеширования нормали и цвета

pub struct IntersectionFull<'a> {
    intersection: Intersection<'a>,
    normal: Vector3,
    color: Color,
}

// Into реализуется автоматически, его не нужно реализовать
impl<'a> From<Intersection<'a>> for IntersectionFull<'a> {
    fn from(other: Intersection<'a>) -> IntersectionFull<'a> {
        let normal = other.get_object().normal_at(other.get_hit_point());
        let color = other.get_object().color_at(other.get_hit_point());
        IntersectionFull {
            intersection: other,
            normal,
            color,
        }
    }
}

// Into реализуется автоматически, его не нужно реализовать
impl<'a> From<IntersectionFull<'a>> for Intersection<'a> {
    fn from(other: IntersectionFull<'a>) -> Intersection<'a> {
        other.intersection
    }
}

impl<'a> IntersectionFull<'a> {
    /// Для найденной фигуры и точки пересечения получаем дистанцию
    pub fn get_distance(&self) -> f32 {
        self.intersection.get_distance()
    }

    /// Для найденной фигуры и точки пересечения получаем точку пересечения
    pub fn get_hit_point(&'a self) -> &'a Vector3 {
        self.intersection.get_hit_point()
    }

    /// Найденная фигура
    pub fn get_object(&'a self) -> &'a dyn Figure {
        self.intersection.get_object()
    }

    /// Для найденной фигуры и точки пересечения получаем нормаль в этой точке
    pub fn get_normal(&'a self) -> &'a Vector3 {
        &self.normal
    }

    /// Для найденной фигуры и точки пересечения получаем цвет в этой точке
    pub fn get_color(&'a self) -> &'a Color {
        &self.color
    }
}
