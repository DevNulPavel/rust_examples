use crate::{
    figures::Texturable,
    structs::{Vector2, Vector3},
};

pub struct TexCoordDelegate<'a> {
    pub hit_point: &'a Vector3,
    pub target: &'a dyn Texturable,
}

impl<'a> TexCoordDelegate<'a> {
    pub fn get_tex_coord(&'a self) -> Vector2 {
        self.target.tex_coords_at(self.hit_point)
    }
}
