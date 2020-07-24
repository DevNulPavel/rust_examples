// Использование общих трейтов и классов через crate от корня
use crate::{
    traits::{
        Normalizable,
        Length
    },
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

pub struct SphericalLight {
    pub position: Vector3,
    pub color: Color,
    pub intensity: f32,
}

impl Light for SphericalLight{
    /// Направление к источнику света в конкретной точке
    fn direction_to_light(&self, point: &Vector3) -> Vector3{
        (self.position - *point).normalize()
    }

    /// Интенсивность света для конкретной точки
    fn intensivity_for_point(&self, point: &Vector3) -> f32{
        // Интенсивность точечного света считается как обратная от расстояния
        let length = self.local_distance_to_point(point) / 4.0;
        //self.intensity / (4.0 * ::std::f32::consts::PI * length)
        if length > 0.0_f32 {
            self.intensity / length
        }else{
            0.0_f32
        }
    }

    /// Получаем расстояние от точки до источника света
    fn distance_to_point(&self, point: &Vector3) -> LightDistance{
        return LightDistance::Some(self.local_distance_to_point(point));
    }
}

impl SphericalLight{
    fn local_distance_to_point(&self, point: &Vector3) -> f32{
        return (self.position - *point).length();
    }
}