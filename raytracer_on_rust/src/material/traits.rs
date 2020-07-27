use crate::{
    structs::{
        Color,
        Vector2
    }
};

pub trait Material{
    fn get_color_at_tex_coord(&self, get_tex_coord_delegate: &dyn FnOnce() -> Vector2) -> Color;
}