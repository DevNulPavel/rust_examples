use super::{
    Material,
    SolidColorMaterial,
    TextureMaterial
};

// TODO: Может быть есть вариант лучше??? Память занимает одно значение как максимум из всех структур
pub enum MaterialsContainer {
    Solid(SolidColorMaterial),
    Texture(TextureMaterial)
}

impl MaterialsContainer {
    pub fn get_material<'a>(&'a self) -> &'a dyn Material{
        match self {
            MaterialsContainer::Solid(ref val) => val,
            MaterialsContainer::Texture(ref val) => val,
        }
    }
}