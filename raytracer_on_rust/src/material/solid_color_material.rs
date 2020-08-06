use crate::{
    structs::{
        Color
    }
};
use super::{
    traits::{
        Material
    },
    tex_coord_delegate::{
        TexCoordDelegate
    },
    material_modificator::{
        MaterialModificator
    }
};

pub struct SolidColorMaterial{
    pub diffuse_solid_color: Color,
    pub modificator: MaterialModificator
}

impl Material for SolidColorMaterial {
    fn get_color_at_tex_coord(&self, _: TexCoordDelegate) -> Color {
        return self.diffuse_solid_color;
    }

    fn get_modificator(&self) -> &MaterialModificator {
        &self.modificator
    }
}
