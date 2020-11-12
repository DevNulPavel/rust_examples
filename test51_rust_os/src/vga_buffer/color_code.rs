use super::{
    color::{
        Color
    }
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)] // Для гарантированно точно такого же лаяута с u8
pub struct ColorCode(u8);

impl ColorCode {
    pub(super) fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}