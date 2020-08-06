
pub enum MaterialModificator{
    None,
    Reflection(f32),
    Refraction{ 
        index: f32, // Индекс преломления: 1.0 - без преломления, 0.0 - максимум преломления
        transparense_level: f32 
    }
}