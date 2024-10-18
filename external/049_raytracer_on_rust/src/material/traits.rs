use super::{material_modificator::MaterialModificator, tex_coord_delegate::TexCoordDelegate};
use crate::structs::Color;

pub trait Material {
    fn get_color_at_tex_coord(&self, get_tex_coord_delegate: TexCoordDelegate) -> Color;
    fn get_modificator(&self) -> &MaterialModificator;
}
