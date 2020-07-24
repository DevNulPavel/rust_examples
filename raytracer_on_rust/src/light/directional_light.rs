// Использование общих трейтов и классов через crate от корня
use crate::{
    structs::{
        Vector3,
        Color
    },
};
use super::{
    traits::{
        Light
    },
    light_distance::{
        LightDistance
    }
};


pub struct DirectionalLight {
    pub direction: Vector3,
    pub color: Color,
    pub intensity: f32,
}

impl Light for DirectionalLight{
    /// Направление к источнику света в конкретной точке
    fn direction_to_light(&self, _: &Vector3) -> Vector3{
        -self.direction
    }

    /// Интенсивность света для конкретной точки
    fn intensivity_for_point(&self, _: &Vector3) -> f32{
        self.intensity
    }

    /// Получаем расстояние от точки до источника света
    fn distance_to_point(&self, _: &Vector3) -> LightDistance{
        // У нас абстрактный источник света без позиции
        return LightDistance::Infinite;
    }
}