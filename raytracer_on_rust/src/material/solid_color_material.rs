use super::{
    material_modificator::MaterialModificator, tex_coord_delegate::TexCoordDelegate,
    traits::Material,
};
use crate::structs::Color;

pub struct SolidColorMaterial {
    pub diffuse_solid_color: Color,
    pub modificator: MaterialModificator,
}

impl Material for SolidColorMaterial {
    fn get_color_at_tex_coord(&self, _: TexCoordDelegate) -> Color {
        return self.diffuse_solid_color;
    }

    fn get_modificator(&self) -> &MaterialModificator {
        &self.modificator
    }
}
