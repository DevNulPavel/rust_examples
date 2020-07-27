use crate::{
    structs::{
        Color,
        Vector2
    }
};
use super::{
    traits::{
        Material
    }
};

pub struct SolidColorMaterial{
    pub diffuse_solid_color: Color
}

impl Material for SolidColorMaterial {
    fn get_color_at_tex_coord(&self, _: &dyn FnOnce() -> Vector2) -> Color {
        return self.diffuse_solid_color;
    }
}
