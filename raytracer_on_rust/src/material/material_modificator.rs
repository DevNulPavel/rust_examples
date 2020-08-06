
pub enum MaterialModificator{
    None,
    Reflection(f32),
    Refraction{ 
        index: f32,
        refraction_level: f32
    }
}