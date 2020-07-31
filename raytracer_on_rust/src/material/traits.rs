use crate::{
    structs::{
        Color
    }
};
use super::{
    tex_coord_delegate::{
        TexCoordDelegate
    }
};

pub trait Material{
    fn get_color_at_tex_coord(&self, get_tex_coord_delegate: TexCoordDelegate) -> Color;
    fn get_reflection_level(&self) -> Option<f32>; // TODO: Assert
}