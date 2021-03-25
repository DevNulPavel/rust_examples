mod image_magic;
mod webp;

pub use self::{
    image_magic::{
        convert_with_image_magic,
        ImageMagicError
    },
    webp::{
        convert_webp,
        WebPerror
    }
};