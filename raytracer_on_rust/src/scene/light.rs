// Использование общих трейтов и классов через crate от корня
use crate::{
    structs::{
        Vector3,
        Color
    },
};

pub struct Light {
    pub direction: Vector3,
    pub color: Color,
    pub intensity: f32,
}
