use super::light_distance::LightDistance;
use crate::structs::Vector3;

pub trait Light {
    /// Направление к источнику света в конкретной точке
    fn direction_to_light(&self, point: &Vector3) -> Vector3;

    /// Интенсивность света для конкретной точки
    fn intensivity_for_point(&self, point: &Vector3) -> f32;

    /// Получаем расстояние от точки до источника света
    fn distance_to_point(&self, point: &Vector3) -> LightDistance;
}
