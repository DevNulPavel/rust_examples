use image::{
    DynamicImage
};
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

fn wrap(val: f32, bound: u32) -> u32 {
    let signed_bound = bound as i32;
    let float_coord = val * bound as f32;
    let wrapped_coord = (float_coord as i32) % signed_bound;
    if wrapped_coord < 0 {
        (wrapped_coord + signed_bound) as u32
    } else {
        wrapped_coord as u32
    }
}

pub struct TextureMaterial{
    pub texture: DynamicImage
}

impl Material for TextureMaterial {
    fn get_color_at_tex_coord(&self, get_tex_coord_delegate: &dyn FnOnce() -> Vector2) -> Color {
        let tex_coord = get_tex_coord_delegate();
        let tex_x = wrap(tex_coord.x, self.texture.width());
        let tex_y = wrap(tex_coord.y, self.texture.height());
        Color::from_rgba(self.texture.get_pixel(tex_x, tex_y))
    }
}
