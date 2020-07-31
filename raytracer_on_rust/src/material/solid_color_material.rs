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
    }
};

pub struct SolidColorMaterial{
    pub diffuse_solid_color: Color,
    pub reflection_level: Option<f32>
}

impl Material for SolidColorMaterial {
    fn get_color_at_tex_coord(&self, _: TexCoordDelegate) -> Color {
        return self.diffuse_solid_color;
    }

    fn get_reflection_level(&self) -> Option<f32> {
        self.reflection_level
    }
}
