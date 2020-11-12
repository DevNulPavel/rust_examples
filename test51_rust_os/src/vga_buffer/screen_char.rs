use super::{
    color_code::{
        ColorCode
    }
};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    pub(super) ascii_character: u8,
    pub(super) color_code: ColorCode,
}