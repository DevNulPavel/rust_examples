use super::refraction_info::RefractionInfo;

pub enum MaterialModificator {
    None,
    Reflection(f32),
    Refraction(RefractionInfo),
}
