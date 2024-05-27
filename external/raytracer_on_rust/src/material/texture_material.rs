use super::{
    material_modificator::MaterialModificator, tex_coord_delegate::TexCoordDelegate,
    traits::Material,
};
use crate::structs::Color;
use image::DynamicImage;

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

// Макрос для устранения дублирования кода
macro_rules! get_pixel {
    ($tex_coord:ident, $texture:ident) => {{
        let tex_x = wrap($tex_coord.x, $texture.width());
        let tex_y = wrap($tex_coord.y, $texture.height());

        let pixel = $texture.get_pixel(tex_x, tex_y);

        pixel
    }};
}

pub struct TextureMaterial {
    pub texture: DynamicImage,
    pub modificator: MaterialModificator,
}

impl Material for TextureMaterial {
    fn get_color_at_tex_coord(&self, get_tex_coord_delegate: TexCoordDelegate) -> Color {
        let tex_coord = get_tex_coord_delegate.get_tex_coord();

        match self.texture {
            DynamicImage::ImageRgb8(ref texture) => {
                let pixel = get_pixel!(tex_coord, texture);
                Color::from_rgb(pixel)
            }
            DynamicImage::ImageRgba8(ref texture) => {
                let pixel = get_pixel!(tex_coord, texture);
                Color::from_rgba(pixel)
            }
            _ => {
                panic!("Invalid image format")
            }
        }
    }

    fn get_modificator(&self) -> &MaterialModificator {
        &self.modificator
    }
}
